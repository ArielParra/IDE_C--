use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box, Orientation,
    TextView, ScrolledWindow,
};
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
    let text_view = TextView::new();
    let buffer = text_view.buffer();
    let file_state: Rc<RefCell<Option<PathBuf>>> =
        Rc::new(RefCell::new(None));

    let scrolled = ScrolledWindow::builder()
        .child(&text_view)
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
