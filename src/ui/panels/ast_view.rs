use gtk::prelude::*;
use gtk::{gio, glib, Label, ListView, ScrolledWindow, SignalListItemFactory, SingleSelection, TreeExpander, TreeListModel};
use crate::models::ast::AstNode;
use crate::ui::language_style::LanguageStyle;

mod imp {
    use std::cell::RefCell;
    use gtk::gio;
    use gtk::glib;
    use gtk::glib::subclass::prelude::*;
    use gtk::prelude::*;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::AstNodeObject)]
    pub struct AstNodeObject {
        #[property(get, set)]
        pub label: RefCell<String>,
        #[property(get, set)]
        pub color: RefCell<String>,
        #[property(get, set)]
        pub line: RefCell<u32>,
        #[property(get, set)]
        pub column: RefCell<u32>,
        #[property(get, set)]
        pub children_model: RefCell<Option<gio::ListStore>>,
        #[property(get, set)]
        pub is_last_child: RefCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AstNodeObject {
        const NAME: &'static str = "AstNodeObject";
        type Type = super::AstNodeObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for AstNodeObject {}
}

gtk::glib::wrapper! {
    pub struct AstNodeObject(ObjectSubclass<imp::AstNodeObject>);
}

impl AstNodeObject {
    pub fn new(label: &str, color: &str, line: u32, column: u32, children_model: Option<gio::ListStore>, is_last_child: bool) -> Self {
        gtk::glib::Object::builder()
            .property("label", label)
            .property("color", color)
            .property("line", line)
            .property("column", column)
            .property("children-model", children_model)
            .property("is-last-child", is_last_child)
            .build()
    }
}

pub struct AstView {
    pub list_view: ListView,
    root_store: gio::ListStore,
    is_dark: bool,
}

impl AstView {
    pub fn new() -> Self {
        let root_store = gio::ListStore::new::<AstNodeObject>();

        let tree_model = TreeListModel::new(
            root_store.clone(),
            false,
            true, // autoexpand
            |item| {
                let node = item.downcast_ref::<AstNodeObject>().unwrap();
                node.children_model().map(|m| m.upcast::<gio::ListModel>())
            }
        );

        let selection_model = SingleSelection::new(Some(tree_model));

        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_, list_item| {
            let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();

            // Layout: [prefix_label] [TreeExpander > content_label]
            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);

            let prefix_label = Label::new(None);
            prefix_label.set_halign(gtk::Align::Start);
            prefix_label.add_css_class("monospace");
            prefix_label.set_widget_name("tree-prefix");

            let content_label = Label::new(None);
            content_label.set_halign(gtk::Align::Start);

            let expander = TreeExpander::new();
            expander.set_indent_for_depth(false);
            expander.set_child(Some(&content_label));

            hbox.append(&prefix_label);
            hbox.append(&expander);

            list_item.set_child(Some(&hbox));
        });

        factory.connect_bind(move |_, list_item| {
            let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
            let hbox = list_item.child().and_downcast::<gtk::Box>().unwrap();

            // Get widgets: prefix_label is first child, expander is second
            let prefix_label = hbox.first_child().and_downcast::<Label>().unwrap();
            let expander = prefix_label.next_sibling().and_downcast::<TreeExpander>().unwrap();
            let content_label = expander.child().and_downcast::<Label>().unwrap();

            let row = list_item.item().and_downcast::<gtk::TreeListRow>().unwrap();
            expander.set_list_row(Some(&row));

            let node = row.item().and_downcast::<AstNodeObject>().unwrap();

            // Build tree-line prefix using box-drawing characters
            let depth = row.depth();
            let prefix = Self::build_tree_prefix(&row, depth);
            prefix_label.set_text(&prefix);

            // Set colored content label
            let color = node.color();
            let escaped_label = glib::markup_escape_text(&node.label());
            content_label.set_markup(&format!("<span foreground=\"{}\">{}</span>", color, escaped_label));
        });

        let list_view = ListView::new(Some(selection_model), Some(factory));
        list_view.add_css_class("navigation-sidebar");
        list_view.set_single_click_activate(true);

        let is_dark = LanguageStyle::is_dark_mode();
        AstView { list_view, root_store, is_dark }
    }

    /// Build box-drawing prefix like: "│  ├── " or "   └── "
    /// by walking up the TreeListRow's ancestors.
    fn build_tree_prefix(row: &gtk::TreeListRow, depth: u32) -> String {
        if depth == 0 {
            return String::new();
        }

        // Collect ancestor "is_last_child" flags from root down to current node
        let mut ancestor_last_flags: Vec<bool> = Vec::with_capacity(depth as usize);

        // Walk up from the current row to the root, collecting is_last_child info
        let mut current = row.clone();
        for _ in 0..depth {
            let node = current.item().and_downcast::<AstNodeObject>().unwrap();
            ancestor_last_flags.push(node.is_last_child());
            if let Some(parent) = current.parent() {
                current = parent;
            } else {
                break;
            }
        }
        // Reverse so index 0 = shallowest ancestor, last = current node
        ancestor_last_flags.reverse();

        let mut prefix = String::new();
        for (i, is_last) in ancestor_last_flags.iter().enumerate() {
            if i == ancestor_last_flags.len() - 1 {
                // Current node connector
                if *is_last {
                    prefix.push_str("└── ");
                } else {
                    prefix.push_str("├── ");
                }
            } else {
                // Ancestor continuation line
                if *is_last {
                    prefix.push_str("    "); // no continuation line (ancestor was last child)
                } else {
                    prefix.push_str("│   "); // continuation line
                }
            }
        }
        prefix
    }

    pub fn widget(&self) -> ScrolledWindow {
        ScrolledWindow::builder()
            .child(&self.list_view)
            .vexpand(true)
            .hexpand(true)
            .build()
    }

    pub fn clear(&self) {
        self.root_store.remove_all();
    }

    pub fn populate(&self, root: &AstNode) {
        self.clear();
        let root_obj = self.create_node_object(root, true);
        self.root_store.append(&root_obj);
    }

    fn create_node_object(&self, node: &AstNode, is_last_child: bool) -> AstNodeObject {
        let color = Self::label_color(&node.label, self.is_dark);
        let line = node.line.unwrap_or(0) as u32;
        let column = node.column.unwrap_or(0) as u32;

        let children_store = if !node.children.is_empty() {
            let store = gio::ListStore::new::<AstNodeObject>();
            let last_idx = node.children.len() - 1;
            for (i, child) in node.children.iter().enumerate() {
                store.append(&self.create_node_object(child, i == last_idx));
            }
            Some(store)
        } else {
            None
        };

        AstNodeObject::new(&node.label, &color, line, column, children_store, is_last_child)
    }

    fn label_color(label: &str, is_dark: bool) -> String {
        let (error, id, number, string, bool_val, type_color, statement, operator, relational) = if is_dark {
            (
                "#f44747", // error
                "#ff57f4", // id
                "#b5cea8", // number
                "#ce9178", // string
                "#569cd6", // bool
                "#569cd6", // type
                "#569cd6", // statement
                "#f44747", // operator
                "#569cd6", // relational
            )
        } else {
            (
                "#d91919", // error
                "#000000", // id
                "#098658", // number
                "#a31515", // string
                "#0000ff", // bool
                "#0000ff", // type
                "#0000ff", // statement
                "#000000", // operator
                "#0000ff", // relational
            )
        };

        if label.contains("Error") || label.contains("Unexpected") || label.contains("Unknown") {
            error.to_string()
        } else if label.starts_with("id:") {
            id.to_string()
        } else if label.starts_with("number:") {
            number.to_string()
        } else if label.starts_with("string:") {
            string.to_string()
        } else if label.starts_with("bool:") {
            bool_val.to_string()
        } else if label.starts_with("type:") {
            type_color.to_string()
        } else if label == "main" || label == "if" || label == "while" || label == "do" || label == "until" {
            statement.to_string()
        } else if label.contains("Expression")
            || label.contains("Selection")
            || label.contains("Iteration")
            || label.contains("Repetition")
            || label.contains("Input")
            || label.contains("Output")
            || label.contains("Declaration")
            || label.contains("Block")
        {
            statement.to_string()
        } else if label.contains("Relational") || label == "Not" {
            relational.to_string()
        } else if label.contains("Assignment")
            || label.contains("Postfix")
            || label.contains("Prefix")
            || label.contains("UnaryOp")
            || label.contains("AddOp")
            || label.contains("MulOp")
            || label.contains("Logical")
            || label.contains("Power")
        {
            operator.to_string()
        } else {
            if is_dark {
                "#ffffff".to_string()
            } else {
                "#000000".to_string()
            }
        }
    }
}
