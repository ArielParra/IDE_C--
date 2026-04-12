use gtk::prelude::*;
use gtk::{Label, Notebook, ScrolledWindow, TextView};
use std::rc::Rc;
use std::cell::RefCell;

// ================= NOTEBOOKS =================

// Creates notebooks for debugging and errors.
// Also returns references to the TextViews that will be used to display tokens and errors.
pub fn create_notebooks() -> (
    Notebook,
    Rc<RefCell<TextView>>,
    Notebook,
    Rc<RefCell<TextView>>, 
) {
    let debugnotebook = Notebook::new();
    let errorsnotebook = Notebook::new();

    let labels_debug = [
        "Lexic",
        "Syntax",
        "Semantic",
        "Hash Table",
        "Intermediate Code",
    ];

    let lexic_textview = Rc::new(RefCell::new(TextView::new()));

    for (i, name) in labels_debug.iter().enumerate() {
        let textview = if i == 0 {
            lexic_textview.borrow().clone() 
        } else {
            let tv = TextView::new();
            tv.set_editable(false);
            tv
        };

        let scrolled = ScrolledWindow::builder()
            .child(&textview)
            .vexpand(true)
            .hexpand(true)
            .build();

        let label = Label::new(Some(name));
        debugnotebook.append_page(&scrolled, Some(&label));
    }

    let labels_errors = [
        "Lexic Errors",
        "Syntax Errors",
        "Semantic Errors",
        "Results",
    ];

    let errors_textview = Rc::new(RefCell::new(TextView::new()));

    for (i, name) in labels_errors.iter().enumerate() {
        let textview = if i == 0 {
            errors_textview.borrow().clone()
        } else {
            let tv = TextView::new();
            tv.set_editable(false);
            tv
        };

        let scrolled = ScrolledWindow::builder()
            .child(&textview)
            .vexpand(true)
            .hexpand(true)
            .build();

        let label = Label::new(Some(name));
        errorsnotebook.append_page(&scrolled, Some(&label));
    }

    (debugnotebook, lexic_textview, errorsnotebook, errors_textview)
}