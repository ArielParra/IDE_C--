use gtk::prelude::*;
use gtk::{Label, Notebook, ScrolledWindow, TextView};
use std::cell::RefCell;
use std::rc::Rc;
use crate::ui::panels::AstView;

pub struct PanelNotebooks {
    pub debug: Notebook,
    pub errors: Notebook,
    pub lexic_view: Rc<RefCell<TextView>>,
    pub errors_view: Rc<RefCell<TextView>>,
    pub syntax_errors_view: Rc<RefCell<TextView>>,
    pub ast_view: Rc<RefCell<AstView>>,
}

pub fn create_panels() -> PanelNotebooks {
    let lexic_view = Rc::new(RefCell::new(TextView::new()));
    let errors_view = Rc::new(RefCell::new(TextView::new()));
    let syntax_errors_view = Rc::new(RefCell::new(TextView::new()));
    let ast_view = Rc::new(RefCell::new(AstView::new()));

    let debug_notebook = create_debug_notebook(&lexic_view, &ast_view);
    let errors_notebook = create_errors_notebook(&errors_view, &syntax_errors_view);

    PanelNotebooks {
        debug: debug_notebook,
        errors: errors_notebook,
        lexic_view,
        errors_view,
        syntax_errors_view,
        ast_view,
    }
}

fn create_debug_notebook(
    lexic_view: &Rc<RefCell<TextView>>,
    ast_view: &Rc<RefCell<AstView>>,
) -> Notebook {
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
            with_copy_button(&tv)
        } else if i == 1 {
            let scrolled = ast_view.borrow().widget();
            scrolled.upcast()
        } else {
            let tv = TextView::new();
            tv.set_editable(false);
            with_copy_button(&tv)
        };

        let label = Label::new(Some(name));
        notebook.append_page(&widget, Some(&label));
    }

    notebook
}

fn create_errors_notebook(
    errors_view: &Rc<RefCell<TextView>>, 
    syntax_errors_view: &Rc<RefCell<TextView>>,
) -> Notebook {
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
            with_copy_button(&tv)
        } else if i == 1 {
            let tv = syntax_errors_view.borrow().clone();
            tv.set_editable(false);
            with_copy_button(&tv)
        } else {
            let tv = TextView::new();
            tv.set_editable(false);
            with_copy_button(&tv)
        };

        let label = Label::new(Some(name));
        notebook.append_page(&widget, Some(&label));
    }

    notebook
}

fn with_copy_button(tv: &TextView) -> gtk::Widget {
    let overlay = gtk::Overlay::new();
    
    let scrolled = ScrolledWindow::builder()
        .child(tv)
        .vexpand(true)
        .hexpand(true)
        .build();
        
    overlay.set_child(Some(&scrolled));
    
    let copy_btn = gtk::Button::builder()
        .tooltip_text("Copy Output")
        .halign(gtk::Align::End)
        .valign(gtk::Align::Start)
        .margin_top(8)
        .margin_end(24)
        .build();
        
    let icon = gtk::Image::from_icon_name("edit-copy-symbolic");
    copy_btn.set_child(Some(&icon));
    copy_btn.add_css_class("flat");
    
    let tv_clone = tv.clone();
    copy_btn.connect_clicked(move |_| {
        let buffer = tv_clone.buffer();
        let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), true).to_string();
        tv_clone.clipboard().set_text(&text);
    });
    
    overlay.add_overlay(&copy_btn);
    
    overlay.upcast()
}
