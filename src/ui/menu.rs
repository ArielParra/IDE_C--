use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, PopoverMenuBar, gio};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::file_manager;

pub fn build_menu(
    app: &Application,
    window: &ApplicationWindow,
    buffer: &impl IsA<gtk::TextBuffer>,
    file_state: Rc<RefCell<Option<PathBuf>>>,
) -> PopoverMenuBar {

    // === Menu Model ===
    let menu_model = gio::Menu::new();

    let text_buffer: gtk::TextBuffer = buffer.as_ref().clone();

    let file_menu = gio::Menu::new();
    file_menu.append(Some("New"), Some("app.new"));
    file_menu.append(Some("Open"), Some("app.open"));
    file_menu.append(Some("Close"), Some("app.close"));
    file_menu.append(Some("Save"), Some("app.save"));
    file_menu.append(Some("Save As"), Some("app.save_as"));
    file_menu.append(Some("Exit"), Some("app.exit"));

    menu_model.append_submenu(Some("File"), &file_menu);

    // === Actions ===
// NEW
    let buffer_clone = text_buffer.clone();
    let file_state_clone = file_state.clone();
    let new_action = gio::SimpleAction::new("new", None);
    new_action.connect_activate(move |_, _| {
        file_manager::file_ops::new_file(&buffer_clone, file_state_clone.clone());
    });
    app.add_action(&new_action);

    // OPEN
    let window_clone = window.clone();
    let buffer_clone = text_buffer.clone();
    let file_state_clone = file_state.clone();
    let open_action = gio::SimpleAction::new("open", None);
    open_action.connect_activate(move |_, _| {
        file_manager::file_ops::open_file_dialog(
            &window_clone,
            buffer_clone.clone(),
            file_state_clone.clone(),
        );
    });
    app.add_action(&open_action);

    // CLOSE
    let app_clone = app.clone();
    let close_action = gio::SimpleAction::new("close", None);
    close_action.connect_activate(move |_, _| {
        app_clone.activate_action("new", None);
    });
    app.add_action(&close_action);

    // SAVE
    let window_clone = window.clone();
    let buffer_clone = text_buffer.clone();
    let file_state_clone = file_state.clone();
    let save_action = gio::SimpleAction::new("save", None);
    save_action.connect_activate(move |_, _| {
        file_manager::file_ops::save_file(
            &window_clone,
            buffer_clone.clone(),
            file_state_clone.clone(),
        );
    });
    app.add_action(&save_action);

    // SAVE AS
    let window_clone = window.clone();
    let buffer_clone = text_buffer.clone();
    let file_state_clone = file_state.clone();
    let save_as_action = gio::SimpleAction::new("save_as", None);
    save_as_action.connect_activate(move |_, _| {
        file_manager::file_ops::save_as_file_dialog(
            &window_clone,
            buffer_clone.clone(),
            file_state_clone.clone(),
        );
    });
    app.add_action(&save_as_action);

    // EXIT
    let app_clone = app.clone();
    let exit_action = gio::SimpleAction::new("exit", None);
    exit_action.connect_activate(move |_, _| {
        app_clone.quit();
    });
    app.add_action(&exit_action);

    PopoverMenuBar::from_model(Some(&menu_model))
}
