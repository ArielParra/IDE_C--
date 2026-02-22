use std::process::Command;

pub fn ejecutar_compilador(
    window: &ApplicationWindow,
    buffer: gtk::TextBuffer,
    current_file: Rc<RefCell<Option<PathBuf>>>,
) {
    let output = Command::new("./compilador.exe")
        .arg("archivo.cmm") // si necesita argumentos
        .output()
        .expect("No se pudo ejecutar el compilador");

    println!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
}