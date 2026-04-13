use gtk::prelude::*;
use gtk::{Label, Notebook, ScrolledWindow, TextView};
use std::cell::RefCell;
use std::rc::Rc;

pub struct PanelNotebooks {
    pub debug: Notebook,
    pub errors: Notebook,
    pub lexic_view: Rc<RefCell<TextView>>,
    pub errors_view: Rc<RefCell<TextView>>,
}

pub fn create_panels() -> PanelNotebooks {
    let lexic_view = Rc::new(RefCell::new(TextView::new()));
    let errors_view = Rc::new(RefCell::new(TextView::new()));
    
    let debug_notebook = create_debug_notebook(&lexic_view);
    let errors_notebook = create_errors_notebook(&errors_view);

    PanelNotebooks {
        debug: debug_notebook,
        errors: errors_notebook,
        lexic_view,
        errors_view,
    }
}

fn create_debug_notebook(lexic_view: &Rc<RefCell<TextView>>) -> Notebook {
    let notebook = Notebook::new();
    let labels = [
        "Lexic",
        "Syntax",
        "Semantic",
        "Hash Table",
        "Intermediate Code",
    ];

    for (i, name) in labels.iter().enumerate() {
        let widget: gtk::Widget = if i == 0 {
            let tv = lexic_view.borrow().clone();
            tv.set_editable(false);
            let scrolled = ScrolledWindow::builder()
                .child(&tv)
                .vexpand(true)
                .hexpand(true)
                .build();
            scrolled.upcast()
        } else {
            let tv = TextView::new();
            tv.set_editable(false);
            let scrolled = ScrolledWindow::builder()
                .child(&tv)
                .vexpand(true)
                .hexpand(true)
                .build();
            scrolled.upcast()
        };

        let label = Label::new(Some(name));
        notebook.append_page(&widget, Some(&label));
    }

    notebook
}

fn create_errors_notebook(errors_view: &Rc<RefCell<TextView>>) -> Notebook {
    let notebook = Notebook::new();
    let labels = [
        "Lexic Errors",
        "Syntax Errors",
        "Semantic Errors",
        "Results",
    ];

    for (i, name) in labels.iter().enumerate() {
        let widget: gtk::Widget = if i == 0 {
            let tv = errors_view.borrow().clone();
            tv.set_editable(false);
            let scrolled = ScrolledWindow::builder()
                .child(&tv)
                .vexpand(true)
                .hexpand(true)
                .build();
            scrolled.upcast()
        } else {
            let tv = TextView::new();
            tv.set_editable(false);
            let scrolled = ScrolledWindow::builder()
                .child(&tv)
                .vexpand(true)
                .hexpand(true)
                .build();
            scrolled.upcast()
        };

        let label = Label::new(Some(name));
        notebook.append_page(&widget, Some(&label));
    }

    notebook
}
