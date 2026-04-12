// ================= BUILD UI =================
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Orientation };
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use super::editor::*;
use super::layout::*;
use super::headerbar::*;
use super::language_style::*;
use super::notebook::*;
use super::menu::*;

// ================= BUILD UI =================

// Builds the entire IDE interface:
// main window, editor, notebooks, layout, header bar and menu.
// Assembles everything in a vertical Box and shows the window.

pub fn build_ui(app: &Application) {

    let window = ApplicationWindow::builder()
        .application(app)
        .title("IDE_C--")
        .default_width(900)
        .default_height(600)
        .build();


   let (lm, sm, lang, scheme) = create_language_and_style();

    let (buffer, view, codigo) =
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
        view.clone(),
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