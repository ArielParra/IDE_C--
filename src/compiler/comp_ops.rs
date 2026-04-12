use std::{cell::RefCell, path::PathBuf, process::Command, rc::Rc};

use gtk::ApplicationWindow;

#[allow(dead_code)]
pub fn ejecutar_compilador(
    _window: &ApplicationWindow,
    _buffer: gtk::TextBuffer,
    _current_file: Rc<RefCell<Option<PathBuf>>>,
) {
    let output = Command::new("./compilador.exe")
        .arg("archivo.cmm") // si necesita argumentos
        .output()
        .expect("No se pudo ejecutar el compilador");

    println!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
}