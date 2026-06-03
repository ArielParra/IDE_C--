use gtk::prelude::*;
use gtk::{ApplicationWindow, FileDialog, TextView};
use std::cell::RefCell;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;

use crate::models::FileState;

pub fn new_file(
    buffer: &gtk::TextBuffer,
    current_file: FileState,
    lex_view: Rc<RefCell<TextView>>,
    errors_view: Rc<RefCell<TextView>>,
    syntax_errors_view: Rc<RefCell<TextView>>,
    ast_view: Rc<RefCell<crate::ui::panels::AstView>>,
) {
    buffer.set_text("");
    lex_view.borrow().buffer().set_text("");
    errors_view.borrow().buffer().set_text("");
    syntax_errors_view.borrow().buffer().set_text("");
    ast_view.borrow().clear();
    *current_file.borrow_mut() = None;
}

pub fn open_file_dialog(
    window: &ApplicationWindow,
    buffer: gtk::TextBuffer,
    current_file: FileState,
    lex_view: Rc<RefCell<TextView>>,
    errors_view: Rc<RefCell<TextView>>,
    syntax_errors_view: Rc<RefCell<TextView>>,
    ast_view: Rc<RefCell<crate::ui::panels::AstView>>,
) {
    let dialog = FileDialog::builder().title("Open File").modal(true).build();

    dialog.open(
        Some(window),
        None::<&gtk::gio::Cancellable>,
        move |result| match result {
            Ok(file) => {
                if let Some(path) = file.path() {
                    match fs::read(&path) {
                        Ok(bytes) => {
                            let contents = String::from_utf8_lossy(&bytes);
                            buffer.set_text(&contents);
                            lex_view.borrow().buffer().set_text("");
                            errors_view.borrow().buffer().set_text("");
                            syntax_errors_view.borrow().buffer().set_text("");
                            ast_view.borrow().clear();
                            *current_file.borrow_mut() = Some(path);
                        }
                        Err(e) => {
                            eprintln!("Failed to read file: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Open file dialog error: {}", e);
            }
        },
    );
}

pub fn save_file(window: &ApplicationWindow, buffer: gtk::TextBuffer, current_file: FileState) {
    let path_opt = current_file.borrow().clone();
    if let Some(path) = path_opt {
        if let Err(e) = write_to_file(&path, &buffer) {
            eprintln!("Failed to save file: {}", e);
        } else {
            show_save_indicator(window);
        }
    } else {
        save_as_file_dialog(window, buffer, current_file);
    }
}

pub fn save_as_file_dialog(
    window: &ApplicationWindow,
    buffer: gtk::TextBuffer,
    current_file: FileState,
) {
    let dialog = FileDialog::builder().title("Save File").modal(true).build();
    let window_clone = window.clone();

    dialog.save(
        Some(window),
        None::<&gtk::gio::Cancellable>,
        move |result| match result {
            Ok(file) => {
                if let Some(path) = file.path() {
                    if let Err(e) = write_to_file(&path, &buffer) {
                        eprintln!("Failed to save file: {}", e);
                    } else {
                        *current_file.borrow_mut() = Some(path);
                        show_save_indicator(&window_clone);
                    }
                }
            }
            Err(e) => {
                eprintln!("Save dialog error: {}", e);
            }
        },
    );
}

fn write_to_file(path: &PathBuf, buffer: &gtk::TextBuffer) -> std::io::Result<()> {
    let start = buffer.start_iter();
    let end = buffer.end_iter();
    let text = buffer.text(&start, &end, true);

    let mut file = fs::File::create(path)?;
    file.write_all(text.as_bytes())?;
    Ok(())
}

fn show_save_indicator(window: &ApplicationWindow) {
    let original_title = window
        .title()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "IDE_C--".to_string());

    if original_title.ends_with(" - Guardado") {
        return;
    }

    window.set_title(Some(&format!("{} - Guardado", original_title)));

    let window_clone = window.clone();
    gtk::glib::timeout_add_local(std::time::Duration::from_millis(1500), move || {
        window_clone.set_title(Some(&original_title));
        gtk::glib::ControlFlow::Break
    });

    if let Some(titlebar) = window.titlebar() {
        if let Some(btn) = find_save_button(&titlebar) {
            btn.set_state_flags(gtk::StateFlags::ACTIVE, false);
            let btn_clone = btn.clone();
            gtk::glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
                btn_clone.unset_state_flags(gtk::StateFlags::ACTIVE);
                gtk::glib::ControlFlow::Break
            });
        }
    }
}

fn find_save_button(widget: &gtk::Widget) -> Option<gtk::Button> {
    if let Some(btn) = widget.downcast_ref::<gtk::Button>() {
        if let Some(action_name) = btn.action_name() {
            if action_name == "app.save" {
                return Some(btn.clone());
            }
        }
    }
    
    let mut child = widget.first_child();
    while let Some(c) = child {
        if let Some(btn) = find_save_button(&c) {
            return Some(btn);
        }
        child = c.next_sibling();
    }
    
    None
}
