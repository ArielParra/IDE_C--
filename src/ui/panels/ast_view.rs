use gtk::prelude::*;
#[allow(deprecated)]
use gtk::{CellRendererText, ScrolledWindow, TreeIter, TreeStore, TreeView, TreeViewColumn};
use crate::models::ast::AstNode;
use crate::ui::language_style::LanguageStyle;

#[allow(deprecated)]
pub struct AstView {
    pub tree_view: TreeView,
    store: TreeStore,
    is_dark: bool,
}

impl AstView {
    #[allow(deprecated)]
    pub fn new() -> Self {
        let store = TreeStore::new(&[
            String::static_type(),
            String::static_type(),
            u32::static_type(),
            u32::static_type(),
        ]);
        let tree_view = TreeView::with_model(&store);

        let renderer = CellRendererText::new();
        let column = TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.add_attribute(&renderer, "text", 0);
        column.add_attribute(&renderer, "foreground", 1);
        tree_view.append_column(&column);

        tree_view.set_headers_visible(false);
        tree_view.set_enable_tree_lines(true);
        tree_view.set_show_expanders(true);
        tree_view.set_activate_on_single_click(true);

        let is_dark = LanguageStyle::is_dark_mode();
        AstView { tree_view, store, is_dark }
    }

    pub fn widget(&self) -> ScrolledWindow {
        ScrolledWindow::builder()
            .child(&self.tree_view)
            .vexpand(true)
            .hexpand(true)
            .build()
    }

    #[allow(deprecated)]
    pub fn clear(&self) {
        self.store.clear();
    }

    #[allow(deprecated)]
    pub fn populate(&self, root: &AstNode) {
        self.store.clear();
        self.insert_node(None, root);
        // Try expand all; TreeView has expand_all on some bindings
        self.tree_view.expand_all();
    }

    #[allow(deprecated)]
    fn insert_node(&self, parent: Option<&TreeIter>, node: &AstNode) {
        let iter = match parent {
            Some(p) => self.store.append(Some(p)),
            None => self.store.append(None),
        };

        let color = Self::label_color(&node.label, self.is_dark);
        let line = node.line.unwrap_or(0) as u32;
        let column = node.column.unwrap_or(0) as u32;
        self.store.set(
            &iter,
            &[
                (0, &node.label),
                (1, &color),
                (2, &line),
                (3, &column),
            ],
        );

        for child in &node.children {
            self.insert_node(Some(&iter), child);
        }
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
        } else if label == "main" || label == "if" || label == "while" || label == "do" {
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
