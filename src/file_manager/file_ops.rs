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
    if let Some(path) = current_file.borrow().clone() {
        if let Err(e) = write_to_file(&path, &buffer) {
            eprintln!("Failed to save file: {}", e);
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
