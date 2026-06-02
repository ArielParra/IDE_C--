use gtk::prelude::*;
#[allow(deprecated)]
use gtk::{CellRendererText, ScrolledWindow, TreeIter, TreeStore, TreeView, TreeViewColumn};
use crate::models::ast::AstNode;

#[allow(deprecated)]
pub struct AstView {
    pub tree_view: TreeView,
    store: TreeStore,
}

impl AstView {
    #[allow(deprecated)]
    pub fn new() -> Self {
        let store = TreeStore::new(&[String::static_type(), String::static_type()]);
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

        AstView { tree_view, store }
    }

    pub fn widget(&self) -> ScrolledWindow {
        ScrolledWindow::builder()
            .child(&self.tree_view)
            .vexpand(true)
            .hexpand(true)
            .build()
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

        let color = Self::label_color(&node.label);
        self.store.set(&iter, &[(0, &node.label), (1, &color)]);

        for child in &node.children {
            self.insert_node(Some(&iter), child);
        }
    }

    fn label_color(label: &str) -> &str {
        if label.contains("Error") || label.contains("Unexpected") {
            "#f44747"
        } else if label.starts_with("id:") {
            "#ff57f4"
        } else if label.starts_with("number:") || label.starts_with("string:") || label.starts_with("bool:") {
            "#b5cea8"
        } else if label.starts_with("type:") {
            "#dcdcaa"
        } else if label.contains("Assignment")
            || label.contains("Expression")
            || label.contains("Selection")
            || label.contains("Iteration")
            || label.contains("Repetition")
            || label.contains("Input")
            || label.contains("Output")
            || label.contains("Declaration")
            || label.contains("Block")
        {
            "#569cd6"
        } else if label.contains("Postfix")
            || label.contains("Prefix")
            || label.contains("AddOp")
            || label.contains("MulOp")
            || label.contains("Relational")
            || label.contains("Logical")
            || label.contains("Power")
        {
            "#c586c0"
        } else {
            "#ffffff"
        }
    }

    #[allow(dead_code, deprecated)]
    pub fn expand_all(&self) {
        self.tree_view.expand_all();
    }

    #[allow(dead_code, deprecated)]
    pub fn collapse_all(&self) {
        self.tree_view.collapse_all();
    }
}
