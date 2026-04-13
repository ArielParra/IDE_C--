use gtk::prelude::*;
use gtk::{ApplicationWindow, FileDialog};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::models::FileState;

pub fn new_file(buffer: &gtk::TextBuffer, current_file: FileState) {
    buffer.set_text("");
    *current_file.borrow_mut() = None;
}

pub fn open_file_dialog(
    window: &ApplicationWindow,
    buffer: gtk::TextBuffer,
    current_file: FileState,
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
