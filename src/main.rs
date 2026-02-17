use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box, Orientation,
    ScrolledWindow,
};
use sourceview5::{View, Buffer};
use sourceview5::prelude::ViewExt;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

mod file_manager;
mod ui;

fn main() {
    let app = Application::builder()
        .application_id("com.ide_cmm.ide")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("IDE_C--")
        .default_width(900)
        .default_height(600)
        .build();

    let vbox = Box::new(Orientation::Vertical, 0);

    // === Editor ===
    let buffer = Buffer::new(None);
    let source_view = View::with_buffer(&buffer);
    source_view.set_show_line_numbers(true);
    source_view.set_highlight_current_line(true);
    source_view.set_monospace(true);


    let file_state: Rc<RefCell<Option<PathBuf>>> =
        Rc::new(RefCell::new(None));

    let scrolled = ScrolledWindow::builder()
        .child(&source_view)
        .vexpand(true)
        .hexpand(true)
        .build();

    // === Menu ===
    let menubar =
        ui::menu::build_menu(app, &window, &buffer, file_state.clone());

    vbox.append(&menubar);
    vbox.append(&scrolled);

    window.set_child(Some(&vbox));
    window.present();
}
