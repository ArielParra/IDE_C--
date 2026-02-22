use gtk::prelude::*;
use gtk::{ApplicationWindow, FileChooserNative, FileChooserAction, ResponseType};
use std::cell::RefCell;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::rc::Rc;

pub fn new_file(
    buffer: &gtk::TextBuffer,
    current_file: Rc<RefCell<Option<PathBuf>>>,
) {
    buffer.set_text("");
    *current_file.borrow_mut() = None;
}

pub fn open_file(
    window: &ApplicationWindow,
    buffer: gtk::TextBuffer,
    current_file: Rc<RefCell<Option<PathBuf>>>,
) {
    let dialog = FileChooserNative::new(
        Some("Open File"),
        Some(window),
        FileChooserAction::Open,
        Some("Open"),
        Some("Cancel"),
    );

    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            if let Some(file) = dialog.file() {
                if let Some(path) = file.path() {
                    if let Ok(mut f) = fs::File::open(&path) {
                        let mut contents = String::new();
                        if f.read_to_string(&mut contents).is_ok() {
                            buffer.set_text(&contents);
                            *current_file.borrow_mut() = Some(path);
                        }
                    }
                }
            }
        }
        dialog.destroy();
    });

    dialog.show();
}

pub fn save_file(
    window: &ApplicationWindow,
    buffer: gtk::TextBuffer,
    current_file: Rc<RefCell<Option<PathBuf>>>,
) {
    if let Some(path) = current_file.borrow().clone() {
        let start = buffer.start_iter();
        let end = buffer.end_iter();
        let text = buffer.text(&start, &end, true);

        if let Ok(mut f) = fs::File::create(path) {
            let _ = f.write_all(text.as_bytes());
        }
    } else {
        save_as_file_dialog(window, buffer, current_file);
    }
}

pub fn save_as_file_dialog(
    window: &ApplicationWindow,
    buffer: gtk::TextBuffer,
    current_file: Rc<RefCell<Option<PathBuf>>>,
) {
    let dialog = FileChooserNative::new(
        Some("Save File"),
        Some(window),
        FileChooserAction::Save,
        Some("Save"),
        Some("Cancel"),
    );

    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            if let Some(file) = dialog.file() {
                if let Some(path) = file.path() {
                    let start = buffer.start_iter();
                    let end = buffer.end_iter();
                    let text = buffer.text(&start, &end, true);

                    if let Ok(mut f) = fs::File::create(&path) {
                        let _ = f.write_all(text.as_bytes());
                        *current_file.borrow_mut() = Some(path);
                    }
                }
            }
        }
        dialog.destroy();
    });

    dialog.show();
}