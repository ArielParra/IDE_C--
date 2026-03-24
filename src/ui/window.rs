// ================= BUILD UI =================
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Orientation };
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

// si usas tu módulo menu
use super::editor::*;
use super::layout::*;
use super::headerbar::*;
use super::language_style::*;
use super::notebook::*;
use super::menu::*;

// ================= BUILD UI =================

// Construye toda la interfaz del IDE:
// ventana principal, editor, notebooks, layout, header bar y menú.
// Ensambla todo en un Box vertical y muestra la ventana.

pub fn build_ui(app: &Application) {

    let window = ApplicationWindow::builder()
        .application(app)
        .title("IDE_C--")
        .default_width(900)
        .default_height(600)
        .build();


   let (lm, sm, lang, scheme) = create_language_and_style();

   let (buffer, _view, codigo) =
    create_editor(lm, sm, lang, scheme);

   let (debugnotebook, lexic_textview, errorsnotebook, errors_textview) = create_notebooks();
   
    let layout =
        create_layout(&codigo, &debugnotebook, &errorsnotebook);

    let header =
        create_headerbar(app);


    let file_state: Rc<RefCell<Option<PathBuf>>> =
        Rc::new(RefCell::new(None));


    let menubar = build_menu(
        app,
        &window,
        &buffer,
        file_state,
        lexic_textview.clone(),
        errors_textview.clone(),
    );


    let windowbox = Box::new(Orientation::Vertical, 0);

    windowbox.append(&menubar);
    windowbox.append(&layout);


    window.set_titlebar(Some(&header));

    window.set_child(Some(&windowbox));

    window.present();
}