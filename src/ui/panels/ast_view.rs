use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gio, glib, Label, ListView, ScrolledWindow, SignalListItemFactory, SingleSelection, TreeExpander, TreeListModel};

use crate::models::ast::AstNode;
use crate::compiler::semantic::{SemanticValueMap, Symbol};
use crate::models::SemanticError;
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
        #[property(get, set)]
        pub node_type: RefCell<String>,
        #[property(get, set)]
        pub node_value: RefCell<String>,
        #[property(get, set)]
        pub node_attribute: RefCell<String>,
        #[property(get, set)]
        pub node_location: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AstNodeObject {
        const NAME: &'static str = "AstNodeObject";
        type Type = super::AstNodeObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for AstNodeObject {}
}

mod symbol_row_imp {
    use std::cell::RefCell;
    use gtk::glib;
    use gtk::glib::subclass::prelude::*;
    use gtk::prelude::*;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::SymbolTableRowObject)]
    pub struct SymbolTableRowObject {
        #[property(get, set)]
        pub id: RefCell<String>,
        #[property(get, set)]
        pub data_type: RefCell<String>,
        #[property(get, set)]
        pub memory_position: RefCell<String>,
        #[property(get, set)]
        pub lines: RefCell<String>,
        #[property(get, set)]
        pub line_positions: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SymbolTableRowObject {
        const NAME: &'static str = "SymbolTableRowObject";
        type Type = super::SymbolTableRowObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for SymbolTableRowObject {}
}

gtk::glib::wrapper! {
    pub struct AstNodeObject(ObjectSubclass<imp::AstNodeObject>);
}

gtk::glib::wrapper! {
    pub struct SymbolTableRowObject(ObjectSubclass<symbol_row_imp::SymbolTableRowObject>);
}

impl SymbolTableRowObject {
    fn new(
        id: &str,
        data_type: &str,
        memory_position: &str,
        lines: &str,
        line_positions: &str,
    ) -> Self {
        gtk::glib::Object::builder()
            .property("id", id)
            .property("data-type", data_type)
            .property("memory-position", memory_position)
            .property("lines", lines)
            .property("line-positions", line_positions)
            .build()
    }
}

impl AstNodeObject {
    pub fn new(
        label: &str,
        color: &str,
        line: u32,
        column: u32,
        children_model: Option<gio::ListStore>,
        is_last_child: bool,
        node_type: &str,
        node_value: &str,
        node_attribute: &str,
        node_location: &str,
    ) -> Self {
        gtk::glib::Object::builder()
            .property("label", label)
            .property("color", color)
            .property("line", line)
            .property("column", column)
            .property("children-model", children_model)
            .property("is-last-child", is_last_child)
            .property("node-type", node_type)
            .property("node-value", node_value)
            .property("node-attribute", node_attribute)
            .property("node-location", node_location)
            .build()
    }
}

pub struct AstView {
    pub list_view: ListView,
    semantic_table: gtk::ColumnView,
    hash_table: gtk::ColumnView,
    root_store: gio::ListStore,
    symbol_store: gio::ListStore,
    location_navigator: Rc<RefCell<Option<Rc<dyn Fn(usize, usize)>>>>,
    is_dark: bool,
}

impl AstView {
    pub fn new() -> Self {
        let root_store = gio::ListStore::new::<AstNodeObject>();

        let tree_model = TreeListModel::new(
            root_store.clone(),
            false,
            false,
            |item| {
                let node = item.downcast_ref::<AstNodeObject>().unwrap();
                node.children_model().map(|m| m.upcast::<gio::ListModel>())
            }
        );

        let list_view = Self::create_list_view(tree_model.clone());
        let location_navigator = Rc::new(RefCell::new(None));
        let semantic_table =
            Self::create_semantic_table(tree_model, location_navigator.clone());
        let symbol_store = gio::ListStore::new::<SymbolTableRowObject>();
        let hash_table =
            Self::create_hash_table(symbol_store.clone(), location_navigator.clone());
        list_view.add_css_class("navigation-sidebar");
        list_view.set_single_click_activate(true);

        let is_dark = LanguageStyle::is_dark_mode();
        AstView {
            list_view,
            semantic_table,
            hash_table,
            root_store,
            symbol_store,
            location_navigator,
            is_dark,
        }
    }

    fn create_list_view(tree_model: TreeListModel) -> ListView {
        let selection_model = SingleSelection::new(Some(tree_model));
        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_, list_item| {
            let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
            let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);

            let prefix_label = Label::new(None);
            prefix_label.set_halign(gtk::Align::Start);
            prefix_label.add_css_class("monospace");
            prefix_label.set_widget_name("tree-prefix");

            let content_label = Label::new(None);
            content_label.set_halign(gtk::Align::Start);

            let expander = TreeExpander::new();
            expander.set_indent_for_depth(false);
            expander.set_child(Some(&content_label));

            row_box.append(&prefix_label);
            row_box.append(&expander);
            list_item.set_child(Some(&row_box));
        });

        factory.connect_bind(move |_, list_item| {
            let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
            let row_box = list_item.child().and_downcast::<gtk::Box>().unwrap();
            let prefix_label = row_box.first_child().and_downcast::<Label>().unwrap();
            let expander = prefix_label.next_sibling().and_downcast::<TreeExpander>().unwrap();
            let content_label = expander.child().and_downcast::<Label>().unwrap();
            let row = list_item.item().and_downcast::<gtk::TreeListRow>().unwrap();
            expander.set_list_row(Some(&row));

            let node = row.item().and_downcast::<AstNodeObject>().unwrap();
            prefix_label.set_text(&Self::build_tree_prefix(&row, row.depth()));
            let color = node.color();
            let escaped_label = glib::markup_escape_text(&node.label());
            content_label.set_markup(&format!(
                "<span foreground=\"{}\">{}</span>",
                color, escaped_label
            ));
        });

        ListView::new(Some(selection_model), Some(factory))
    }

    fn create_semantic_table(
        tree_model: TreeListModel,
        location_navigator: Rc<RefCell<Option<Rc<dyn Fn(usize, usize)>>>>,
    ) -> gtk::ColumnView {
        let selection_model = gtk::NoSelection::new(Some(tree_model));
        let table = gtk::ColumnView::new(Some(selection_model));
        table.set_show_column_separators(true);
        table.set_show_row_separators(true);

        for (index, (title, width)) in [
            ("Nodo", 450),
            ("Tipo", 72),
            ("Valor", 80),
            ("Atributos (H/S)", 175),
            ("Fila:Col", 72),
        ]
        .into_iter()
        .enumerate()
        {
            let factory = SignalListItemFactory::new();
            let setup_navigator = location_navigator.clone();
            factory.connect_setup(move |_, list_item| {
                let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
                if index == 0 {
                    let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                    let prefix = Label::new(None);
                    prefix.add_css_class("monospace");

                    let content = Label::new(None);
                    content.set_halign(gtk::Align::Start);
                    content.set_ellipsize(gtk::pango::EllipsizeMode::End);

                    let expander = TreeExpander::new();
                    expander.set_indent_for_depth(false);
                    expander.set_child(Some(&content));
                    row_box.append(&prefix);
                    row_box.append(&expander);
                    list_item.set_child(Some(&row_box));
                } else if index == 4 {
                    let button = gtk::Button::new();
                    button.add_css_class("flat");
                    button.add_css_class("link");
                    button.connect_clicked({
                        let location_navigator = setup_navigator.clone();
                        move |button| {
                            let Some(position) = button.label() else {
                                return;
                            };
                            let Some((line, column)) = position.split_once(':') else {
                                return;
                            };
                            let (Ok(line), Ok(column)) =
                                (line.parse::<usize>(), column.parse::<usize>())
                            else {
                                return;
                            };
                            if line == 0 || column == 0 {
                                return;
                            }
                            if let Some(navigate) = location_navigator.borrow().clone() {
                                navigate(line, column);
                            }
                        }
                    });
                    button.set_halign(gtk::Align::Start);
                    list_item.set_child(Some(&button));
                } else {
                    let label = Label::new(None);
                    label.set_halign(gtk::Align::Start);
                    label.set_ellipsize(gtk::pango::EllipsizeMode::End);
                    label.add_css_class("monospace");
                    list_item.set_child(Some(&label));
                }
            });

            factory.connect_bind(move |_, list_item| {
                let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
                let row = list_item.item().and_downcast::<gtk::TreeListRow>().unwrap();
                let node = row.item().and_downcast::<AstNodeObject>().unwrap();

                if index == 0 {
                    let row_box = list_item.child().and_downcast::<gtk::Box>().unwrap();
                    let prefix = row_box.first_child().and_downcast::<Label>().unwrap();
                    let expander = prefix.next_sibling().and_downcast::<TreeExpander>().unwrap();
                    let content = expander.child().and_downcast::<Label>().unwrap();
                    expander.set_list_row(Some(&row));
                    prefix.set_text(&Self::build_tree_prefix(&row, row.depth()));
                    content.set_markup(&format!(
                        "<span foreground=\"{}\">{}</span>",
                        node.color(),
                        glib::markup_escape_text(&node.label())
                    ));
                } else {
                    let value = match index {
                        1 => node.node_type(),
                        2 => node.node_value(),
                        3 => node.node_attribute(),
                        _ => node.node_location(),
                    };
                    if index == 4 {
                        list_item
                            .child()
                            .and_downcast::<gtk::Button>()
                            .unwrap()
                            .set_label(&value);
                        return;
                    }
                    let label = list_item.child().and_downcast::<Label>().unwrap();
                    if node.node_type() == "error" && index <= 2 {
                        label.set_markup(&format!(
                            "<span foreground=\"#f44747\" weight=\"bold\">{}</span>",
                            glib::markup_escape_text(&value)
                        ));
                    } else {
                        label.set_text(&value);
                    }
                }
            });

            let column = gtk::ColumnViewColumn::new(Some(title), Some(factory));
            column.set_fixed_width(width);
            column.set_resizable(false);
            table.append_column(&column);
        }

        table
    }

    fn create_hash_table(
        symbol_store: gio::ListStore,
        location_navigator: Rc<RefCell<Option<Rc<dyn Fn(usize, usize)>>>>,
    ) -> gtk::ColumnView {
        let selection_model = gtk::NoSelection::new(Some(symbol_store));
        let table = gtk::ColumnView::new(Some(selection_model));
        table.set_show_column_separators(true);
        table.set_show_row_separators(true);
        table.set_hexpand(true);
        table.set_vexpand(true);

        for (index, title) in [
            "ID",
            "Tipo",
            "Memoria",
            "Linea",
        ]
        .into_iter()
        .enumerate()
        {
            let factory = SignalListItemFactory::new();
            let setup_navigator = location_navigator.clone();
            factory.connect_setup(move |_, list_item| {
                let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
                if index == 3 {
                    let links = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                    links.set_hexpand(true);
                    list_item.set_child(Some(&links));
                } else {
                    let label = Label::new(None);
                    label.set_halign(gtk::Align::Start);
                    label.set_ellipsize(gtk::pango::EllipsizeMode::End);
                    label.set_hexpand(true);
                    label.add_css_class("monospace");
                    list_item.set_child(Some(&label));
                }
            });
            factory.connect_bind(move |_, list_item| {
                let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
                let row = list_item
                    .item()
                    .and_downcast::<SymbolTableRowObject>()
                    .unwrap();
                let value = match index {
                    0 => row.id(),
                    1 => row.data_type(),
                    2 => row.memory_position(),
                    _ => row.lines(),
                };
                if index == 3 {
                    let links = list_item.child().and_downcast::<gtk::Box>().unwrap();
                    while let Some(child) = links.first_child() {
                        links.remove(&child);
                    }
                    let line_positions = row.line_positions();
                    for (line, column) in line_positions
                        .split(';')
                        .filter_map(Self::parse_line_position)
                    {
                        let button = gtk::Button::with_label(&line.to_string());
                        button.add_css_class("flat");
                        button.add_css_class("link");
                        button.add_css_class("hash-line-link");
                        button.set_margin_start(0);
                        button.set_margin_end(0);
                        button.set_halign(gtk::Align::Start);
                        button.connect_clicked({
                            let location_navigator = setup_navigator.clone();
                            move |_| {
                                if let Some(navigate) = location_navigator.borrow().clone() {
                                    navigate(line, column);
                                }
                            }
                        });
                        links.append(&button);
                    }
                } else {
                    list_item
                        .child()
                        .and_downcast::<Label>()
                        .unwrap()
                        .set_text(&value);
                }
        });

        let column = gtk::ColumnViewColumn::new(Some(title), Some(factory));
        column.set_expand(false);
        column.set_resizable(false);
        table.append_column(&column);
    }

    table
    }

    pub fn set_location_navigator(
        &self,
        navigate: impl Fn(usize, usize) + 'static,
    ) {
        *self.location_navigator.borrow_mut() = Some(Rc::new(navigate));
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

    pub fn widget_with_attributes(&self) -> gtk::Widget {
        let tree = ScrolledWindow::builder()
            .child(&self.semantic_table)
            .vexpand(true)
            .hexpand(true)
            .build();
        tree.upcast()
    }

    pub fn widget_hash_table(&self) -> gtk::Widget {
        let scrolled = ScrolledWindow::builder()
            .child(&self.hash_table)
            .vexpand(true)
            .hexpand(true)
            .build();
        scrolled.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
        scrolled.upcast()
    }

    pub fn clear(&self) {
        self.root_store.remove_all();
        self.symbol_store.remove_all();
    }

    pub fn populate_symbol_table(&self, symbols: &[Symbol]) {
        self.symbol_store.remove_all();
        for symbol in symbols {
            let entries = Self::symbol_line_entries(symbol);
            let lines = Self::line_labels(&entries);
            let line_positions = entries
                .iter()
                .map(|(line, column)| format!("{line}:{column}"))
                .collect::<Vec<_>>()
                .join(";");
            self.symbol_store.append(&SymbolTableRowObject::new(
                &symbol.name,
                symbol.data_type.as_str(),
                &symbol.memory_position.to_string(),
                &lines,
                &line_positions,
            ));
        }
    }

    fn parse_line_position(position: &str) -> Option<(usize, usize)> {
        let (line, column) = position.split_once(':')?;
        Some((line.parse().ok()?, column.parse().ok()?))
    }

    pub fn symbol_line_summary(symbol: &Symbol) -> String {
        Self::line_labels(&Self::symbol_line_entries(symbol))
    }

    fn symbol_line_entries(symbol: &Symbol) -> Vec<(usize, usize)> {
        let mut entries = symbol
            .appearances
            .iter()
            .map(|appearance| (appearance.line, appearance.column))
            .collect::<Vec<_>>();
        if entries.is_empty() {
            entries.extend(symbol.lines.iter().map(|line| (*line, symbol.column)));
        }
        entries.sort_unstable();
        entries
    }

    fn line_labels(entries: &[(usize, usize)]) -> String {
        entries
            .iter()
            .map(|(line, _)| line.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn populate(&self, root: &AstNode) {
        self.clear();
        let root_obj = self.create_node_object(root, true, None, None, &[]);
        self.root_store.append(&root_obj);
    }

    pub fn populate_semantic(
        &self,
        root: &AstNode,
        symbols: &[Symbol],
        values: &SemanticValueMap,
        errors: &[SemanticError],
    ) {
        self.clear();
        let root_obj = self.create_node_object(root, true, Some(symbols), Some(values), errors);
        self.root_store.append(&root_obj);
    }

    fn create_node_object(
        &self,
        node: &AstNode,
        is_last_child: bool,
        symbols: Option<&[Symbol]>,
        values: Option<&SemanticValueMap>,
        errors: &[SemanticError],
    ) -> AstNodeObject {
        let color = Self::label_color(&node.label, self.is_dark);
        let line = node.line.unwrap_or(0) as u32;
        let column = node.column.unwrap_or(0) as u32;
        let has_semantic_error = node.line.zip(node.column).is_some_and(|(line, column)| {
            errors
                .iter()
                .any(|error| error.line == line && error.column == column)
        });
        let node_type = if has_semantic_error {
            "error".to_string()
        } else {
            Self::node_type(node, symbols)
        };
        let node_value = Self::node_value(node, symbols, values);
        let node_attribute = Self::node_attribute(node, symbols);
        let node_location = match (node.line, node.column) {
            (Some(line), Some(column)) => format!("{line}:{column}"),
            (Some(line), None) => format!("{line}:—"),
            _ => "—".to_string(),
        };

        let children_store = if !node.children.is_empty() {
            let store = gio::ListStore::new::<AstNodeObject>();
            let last_idx = node.children.len() - 1;
            for (i, child) in node.children.iter().enumerate() {
                store.append(&self.create_node_object(
                    child,
                    i == last_idx,
                    symbols,
                    values,
                    errors,
                ));
            }
            Some(store)
        } else {
            None
        };

        AstNodeObject::new(
            &node.label,
            &color,
            line,
            column,
            children_store,
            is_last_child,
            &node_type,
            &node_value,
            &node_attribute,
            &node_location,
        )
    }

    fn node_type(node: &AstNode, symbols: Option<&[Symbol]>) -> String {
        if let Some(annotated_type) = &node.annotated_type {
            return annotated_type.clone();
        }
        let label = node.label.trim();
        if let Some(declared_type) = label.strip_prefix("type:") {
            return declared_type.trim().to_string();
        }
        if label.starts_with("number:") {
            return if label.contains('.') { "float" } else { "int" }.to_string();
        }
        if label.starts_with("bool:") {
            return "bool".to_string();
        }
        if label.starts_with("string:") {
            return "string".to_string();
        }
        if label == "Assignment" {
            if let Some(name) = node.children.first().and_then(|child| {
                child.label.trim().strip_prefix("id: ")
            }) {
                return symbols
                    .and_then(|symbols| symbols.iter().rev().find(|symbol| symbol.name == name))
                    .map(|symbol| symbol.data_type.as_str().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
            }
        }
        if let Some(name) = label.strip_prefix("id: ") {
            return symbols
                .and_then(|symbols| symbols.iter().rev().find(|symbol| symbol.name == name))
                .map(|symbol| symbol.data_type.as_str().to_string())
                .unwrap_or_else(|| "unknown".to_string());
        }
        if let Some(symbol) = Self::declared_symbol(node, symbols) {
            return symbol.data_type.as_str().to_string();
        }
        if label.starts_with("Relational(")
            || label.starts_with("LogicalAnd")
            || label.starts_with("LogicalOr")
            || label == "Not"
        {
            return "bool".to_string();
        }
        if label.starts_with("AddOp(") || label.starts_with("MulOp(") || label == "Power" {
            let child_types = node
                .children
                .iter()
                .map(|child| Self::node_type(child, symbols))
                .collect::<Vec<_>>();
            if child_types.len() < 2
                || child_types
                    .iter()
                    .any(|type_name| type_name != "int" && type_name != "float")
            {
                return "unknown".to_string();
            }
            if label.starts_with("MulOp(/)")
                || child_types.iter().any(|type_name| type_name == "float")
            {
                return "float".to_string();
            }
            return "int".to_string();
        }
        if label.starts_with("Prefix(")
            || label.starts_with("Postfix(")
            || label.starts_with("UnaryOp(")
        {
            let operand_type = node
                .children
                .first()
                .map(|child| Self::node_type(child, symbols))
                .unwrap_or_else(|| "unknown".to_string());
            return match operand_type.as_str() {
                "int" | "float" => operand_type,
                _ => "unknown".to_string(),
            };
        }
        "—".to_string()
    }

    fn node_value(
        node: &AstNode,
        symbols: Option<&[Symbol]>,
        values: Option<&SemanticValueMap>,
    ) -> String {
        if let Some(value) = &node.attributed_value {
            return value.clone();
        }
        if let Some(value) = node
            .line
            .zip(node.column)
            .and_then(|location| values?.get(&location))
        {
            return value.clone();
        }
        if node.label.trim().starts_with("id: ") || Self::declared_symbol(node, symbols).is_some() {
            return node
                .line
                .zip(node.column)
                .and_then(|location| values?.get(&location))
                .cloned()
                .unwrap_or_else(|| "—".to_string());
        }
        for prefix in ["id: ", "number:", "bool:", "string:", "type:"] {
            if let Some(value) = node.label.strip_prefix(prefix) {
                return value.trim().to_string();
            }
        }
        "—".to_string()
    }

    fn node_attribute(node: &AstNode, symbols: Option<&[Symbol]>) -> String {
        if node.label.trim().starts_with("id: ") || Self::declared_symbol(node, symbols).is_some() {
            "H".to_string()
        } else {
            "S".to_string()
        }
    }

    fn declared_symbol<'a>(
        node: &AstNode,
        symbols: Option<&'a [Symbol]>,
    ) -> Option<&'a Symbol> {
        let (line, column) = (node.line?, node.column?);
        let symbols = symbols?;
        symbols
            .iter()
            .find(|symbol| {
            symbol.name == node.label.trim()
                && symbol.line == line
                && symbol.column == column
            })
            .or_else(|| {
                symbols
                    .iter()
                    .find(|symbol| symbol.name == node.label.trim() && symbol.line == line)
            })
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

#[cfg(test)]
mod tests {
    use super::AstView;
    use crate::compiler::semantic::{SemanticType, SemanticValueMap, Symbol, SymbolAppearance};
    use crate::models::ast::AstNode;

    #[test]
    fn declaration_identifiers_show_their_symbol_attributes() {
        let declarations = [
            ("x", 6, 2, SemanticType::Int, 1),
            ("y", 9, 2, SemanticType::Int, 2),
            ("z", 12, 2, SemanticType::Int, 3),
            ("a", 8, 3, SemanticType::Float, 4),
            ("b", 11, 3, SemanticType::Float, 5),
            ("c", 14, 3, SemanticType::Float, 6),
        ];
        let symbols = declarations
            .iter()
            .map(|(name, column, line, data_type, memory_position)| Symbol {
                name: (*name).to_string(),
                data_type: data_type.clone(),
                line: *line,
                column: *column,
                memory_position: *memory_position,
                lines: vec![*line],
                appearances: vec![],
            })
            .collect::<Vec<_>>();
        let values = declarations
            .iter()
            .map(|(name, column, line, _, _)| ((*line, *column), format!("value-{name}")))
            .collect::<SemanticValueMap>();

        for (name, column, line, data_type, _) in declarations {
            let node = AstNode::new(name).with_pos(line, column);
            assert_eq!(
                AstView::node_type(&node, Some(&symbols)),
                data_type.as_str()
            );
            assert_eq!(
                AstView::node_value(&node, Some(&symbols), Some(&values)),
                format!("value-{name}")
            );
            assert_eq!(AstView::node_attribute(&node, Some(&symbols)), "H");
        }

        let int_identifier = AstNode::new("x").with_pos(2, 999);
        assert_eq!(AstView::node_type(&int_identifier, Some(&symbols)), "int");
    }

    #[test]
    fn arithmetic_parent_nodes_show_their_computed_value() {
        let node = AstNode::new("AddOp(+)").with_pos(3, 8);
        let values = [((3, 8), "14".to_string())]
            .into_iter()
            .collect::<SemanticValueMap>();

        assert_eq!(AstView::node_value(&node, None, Some(&values)), "14");
    }

    #[test]
    fn arithmetic_operator_nodes_show_their_result_type() {
        let mut multiply = AstNode::new("MulOp(*)");
        multiply.add_child(AstNode::new("number: 2"));
        multiply.add_child(AstNode::new("number: 3"));

        let mut add = AstNode::new("AddOp(+)");
        add.add_child(multiply);
        add.add_child(AstNode::new("number: 4.5"));

        assert_eq!(AstView::node_type(&add, None), "float");
        assert_eq!(AstView::node_type(&add.children[0], None), "int");
    }

    #[test]
    fn multiplication_and_division_use_the_types_of_their_children() {
        let symbols = [
            Symbol {
                name: "count".to_string(),
                data_type: SemanticType::Int,
                line: 1,
                column: 1,
                memory_position: 1,
                lines: vec![1],
                appearances: vec![],
            },
            Symbol {
                name: "factor".to_string(),
                data_type: SemanticType::Float,
                line: 1,
                column: 8,
                memory_position: 2,
                lines: vec![1],
                appearances: vec![],
            },
        ];

        for label in ["MulOp(*)", "MulOp(/)"] {
            let mut node = AstNode::new(label);
            node.add_child(AstNode::new("id: count"));
            node.add_child(AstNode::new("id: factor"));
            assert_eq!(
                AstView::node_type(&node, Some(&symbols)),
                "float",
                "{label}"
            );
        }
    }

    #[test]
    fn logical_and_relational_operator_nodes_show_boolean_type() {
        let mut relational = AstNode::new("Relational(<)");
        relational.add_child(AstNode::new("number: 2"));
        relational.add_child(AstNode::new("number: 3"));
        let mut logical = AstNode::new("LogicalAnd");
        logical.add_child(relational);
        logical.add_child(AstNode::new("bool: true"));

        assert_eq!(AstView::node_type(&logical, None), "bool");
        assert_eq!(AstView::node_type(&logical.children[0], None), "bool");
    }

    #[test]
    fn remaining_operator_nodes_show_their_result_types() {
        for label in ["AddOp(-)", "MulOp(%)", "Power"] {
            let mut node = AstNode::new(label);
            node.add_child(AstNode::new("number: 8"));
            node.add_child(AstNode::new("number: 2"));
            assert_eq!(AstView::node_type(&node, None), "int", "{label}");
        }

        let mut integer_division = AstNode::new("MulOp(/)");
        integer_division.add_child(AstNode::new("number: 5"));
        integer_division.add_child(AstNode::new("number: 2"));
        assert_eq!(AstView::node_type(&integer_division, None), "float");

        let mut mixed_add = AstNode::new("AddOp(-)");
        mixed_add.add_child(AstNode::new("number: 8"));
        mixed_add.add_child(AstNode::new("number: 2.5"));
        assert_eq!(AstView::node_type(&mixed_add, None), "float");

        for label in ["LogicalOr", "LogicalAnd", "Relational(>)", "Not"] {
            let mut node = AstNode::new(label);
            node.add_child(AstNode::new("bool: true"));
            if label != "Not" {
                node.add_child(AstNode::new("bool: false"));
            }
            assert_eq!(AstView::node_type(&node, None), "bool", "{label}");
        }

        for label in ["Prefix(++)", "Postfix(--)", "UnaryOp(++)"] {
            let mut node = AstNode::new(label);
            node.add_child(AstNode::new("number: 1.5"));
            assert_eq!(AstView::node_type(&node, None), "float", "{label}");
        }

        let mut invalid_unary = AstNode::new("Prefix(++)");
        invalid_unary.add_child(AstNode::new("bool: true"));
        assert_eq!(AstView::node_type(&invalid_unary, None), "unknown");
    }

    #[test]
    fn symbol_table_repeats_lines_for_each_appearance_without_values() {
        let symbol = Symbol {
            name: "x".to_string(),
            data_type: SemanticType::Int,
            line: 1,
            column: 5,
            memory_position: 1,
            lines: vec![1, 2, 4],
            appearances: vec![
                SymbolAppearance {
                    line: 1,
                    column: 5,
                    value: None,
                },
                SymbolAppearance {
                    line: 2,
                    column: 3,
                    value: Some("2".to_string()),
                },
                SymbolAppearance {
                    line: 2,
                    column: 9,
                    value: Some("5".to_string()),
                },
                SymbolAppearance {
                    line: 4,
                    column: 7,
                    value: Some("8".to_string()),
                },
            ],
        };

        assert_eq!(AstView::symbol_line_summary(&symbol), "1, 2, 2, 4");
    }

    #[test]
    fn line_link_target_preserves_column_for_repeated_lines() {
        assert_eq!(AstView::parse_line_position("2:3"), Some((2, 3)));
        assert_eq!(AstView::parse_line_position("2:9"), Some((2, 9)));
        assert_eq!(AstView::parse_line_position("Linea"), None);
    }
}
