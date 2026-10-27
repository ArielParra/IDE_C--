use crate::compiler::semantic::Symbol;
use gtk::{gio, glib, prelude::*};

const HEADERS: [&str; 7] = [
    "Name",
    "Type",
    "Scope",
    "Declaration",
    "Lines",
    "Offset (slot)",
    "References",
];

pub struct SymbolsView {
    pub view: gtk::ColumnView,
    store: gio::ListStore,
}

fn cells(symbol: &Symbol) -> [String; 7] {
    let mut lines = vec![symbol.line];
    lines.extend(symbol.references.iter().map(|(line, _)| *line));
    lines.sort_unstable();
    lines.dedup();
    [
        symbol.name.clone(),
        symbol.data_type.as_str().to_string(),
        symbol.scope_id.to_string(),
        format!("{}:{}", symbol.line, symbol.column),
        lines
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(", "),
        symbol.offset.to_string(),
        symbol
            .references
            .iter()
            .map(|(line, col)| format!("{line}:{col}"))
            .collect::<Vec<_>>()
            .join(", "),
    ]
}

impl SymbolsView {
    pub fn new() -> Self {
        let store = gio::ListStore::new::<glib::BoxedAnyObject>();
        let selection = gtk::SingleSelection::new(Some(store.clone()));
        let view = gtk::ColumnView::new(Some(selection));
        view.set_single_click_activate(true);
        view.set_show_column_separators(true);
        view.set_show_row_separators(true);
        for (index, title) in HEADERS.iter().enumerate() {
            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(|_, item| {
                let item = item.downcast_ref::<gtk::ListItem>().unwrap();
                let label = gtk::Label::new(None);
                label.set_xalign(0.0);
                label.set_margin_start(8);
                label.set_margin_end(8);
                item.set_child(Some(&label));
            });
            factory.connect_bind(move |_, item| {
                let item = item.downcast_ref::<gtk::ListItem>().unwrap();
                let object = item.item().and_downcast::<glib::BoxedAnyObject>().unwrap();
                let symbol = object.borrow::<Symbol>();
                let label = item.child().and_downcast::<gtk::Label>().unwrap();
                label.set_text(&cells(&symbol)[index]);
            });
            let column = gtk::ColumnViewColumn::new(Some(title), Some(factory));
            column.set_resizable(true);
            view.append_column(&column);
        }
        Self { view, store }
    }
    pub fn populate(&self, symbols: &[Symbol]) {
        self.clear();
        for symbol in symbols {
            self.store
                .append(&glib::BoxedAnyObject::new(symbol.clone()));
        }
    }
    pub fn clear(&self) {
        self.store.remove_all();
    }
    pub fn widget(&self) -> gtk::Widget {
        let overlay = gtk::Overlay::new();
        let scroll = gtk::ScrolledWindow::builder()
            .child(&self.view)
            .hexpand(true)
            .vexpand(true)
            .build();
        overlay.set_child(Some(&scroll));
        let button = gtk::Button::builder()
            .icon_name("edit-copy-symbolic")
            .tooltip_text("Copy Symbol Table")
            .halign(gtk::Align::End)
            .valign(gtk::Align::End)
            .margin_end(8)
            .margin_bottom(8)
            .build();
        let store = self.store.clone();
        button.connect_clicked(move |button| {
            let mut rows = vec![HEADERS.join("\t")];
            for index in 0..store.n_items() {
                if let Some(object) = store.item(index).and_downcast::<glib::BoxedAnyObject>() {
                    rows.push(cells(&object.borrow::<Symbol>()).join("\t"));
                }
            }
            button.clipboard().set_text(&rows.join("\n"));
        });
        overlay.add_overlay(&button);
        overlay.upcast()
    }
}
