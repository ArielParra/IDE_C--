use std::{cell::RefCell, path::PathBuf, process::Command, rc::Rc};

use gtk::ApplicationWindow;

#[allow(dead_code)]
pub fn execute_compiler(
    _window: &ApplicationWindow,
    _buffer: gtk::TextBuffer,
    _current_file: Rc<RefCell<Option<PathBuf>>>,
) {
    let output = Command::new("./compilador.exe")
        .arg("archivo.cmm")
        .output()
        .expect("Could not execute compiler");

    println!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
}