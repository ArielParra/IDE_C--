use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, PopoverMenuBar, gio};
use std::cell::RefCell;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::thread;
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


    //EDIT
    let edit_menu = gio::Menu::new();
    edit_menu.append(Some("Undo"), Some("app.undo"));
    edit_menu.append(Some("Redo"), Some("app.redo"));
    edit_menu.append(Some("Cut"), Some("app.cut"));
    edit_menu.append(Some("Copy"), Some("app.copy"));
    //BUILD & DEBUG
    let build_debug_menu = gio::Menu::new();
    build_debug_menu.append(Some("Compile1"), Some("app.compile1"));//probaremos en la seccion compilar esta funcion
    build_debug_menu.append(Some("Run"), Some("app.run"));
    build_debug_menu.append(Some("Debug"), Some("app.debug"));

    //LEXICO
    let lexico_menu = gio::Menu::new();
    lexico_menu.append(Some("Run Lexical Analysis"), Some("app.lexical"));

    //SINTACTICO
    let sintactico_menu = gio::Menu::new();
    sintactico_menu.append(Some("Run Syntax Analysis"), Some("app.syntax"));

    //SEMANTICO
    let semantico_menu = gio::Menu::new();
    semantico_menu.append(Some("Run Semantic Analysis"), Some("app.semantic"));

    //COMPILAR
    let compilar_menu = gio::Menu::new();
    compilar_menu.append(Some("Compile"), Some("app.c--compiler"));
    compilar_menu.append(Some("Lexical Analysis"), Some("app.lexico"));
    compilar_menu.append(Some("Syntax Analysis"), Some("app.sintactico"));
    compilar_menu.append(Some("Semantic Analysis"), Some("app.semantico"));
    compilar_menu.append(Some("Intermediate Code"), Some("app.intermedio"));
    compilar_menu.append(Some("Execute"), Some("app.ejecutar"));
    menu_model.append_submenu(Some("File"), &file_menu);
    menu_model.append_submenu(Some("Edit"), &edit_menu);
    menu_model.append_submenu(Some("Build & Debug"), &build_debug_menu);
    menu_model.append_submenu(Some("Lexical Analysis"), &lexico_menu);
    menu_model.append_submenu(Some("Syntax Analysis"), &sintactico_menu);
    menu_model.append_submenu(Some("Semantic Analysis"), &semantico_menu);
    menu_model.append_submenu(Some("Compile"), &compilar_menu);

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

    // COMPILE
    /*let window_clone = window.clone();
    let buffer_clone = text_buffer.clone();
    let file_state_clone = file_state.clone();*/
    let file_state_clone = file_state.clone();

    let compile_action = gio::SimpleAction::new("c--compiler", None);

    compile_action.connect_activate(move |_, _| {

    let path = match &*file_state_clone.borrow() {
        Some(p) => p.clone(),
        None => {
            eprintln!("No file selected to compile.");
            return;
        }
    };

    let compiler_path = if cfg!(target_os = "windows") {
        "bin/windows/c--compiler.exe"
    } else if cfg!(target_os = "macos") {
        "bin/macos/c--compiler"
    } else {
        "bin/linux/c--compiler"
    };

    let mut child = match Command::new(compiler_path)
        .arg(&path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to start compiler: {}", e);
            return;
        }
    };

    // === STDOUT THREAD ===
    if let Some(stdout) = child.stdout.take() {
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(l) = line {
                    println!("OUT: {}", l);
                }
            }
        });
    }

    // === STDERR THREAD ===
    if let Some(stderr) = child.stderr.take() {
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(l) = line {
                    eprintln!("ERR: {}", l);
                }
            }
        });
    }
    });

    app.add_action(&compile_action);

    //============ACTION OF BUTTONS IN HEADERBAR===========
    // -------- OPEN FILE ICON ------  
    let window_clone = window.clone();
    let buffer_clone = text_buffer.clone();
    let file_state_clone = file_state.clone();
    let open_btn = gio::SimpleAction::new("open", None);
    open_btn.connect_activate(move |_, _| {
        file_manager::file_ops::open_file_dialog(
            &window_clone,
            buffer_clone.clone(),
            file_state_clone.clone(),
        );
    });
    app.add_action(&open_btn);
    
    // -------- NEW FILE ICON -------
    let buffer_clone = text_buffer.clone();
    let file_state_clone = file_state.clone();
    let new_btn = gio::SimpleAction::new("new", None);
    new_btn.connect_activate(move |_, _| {
        file_manager::file_ops::new_file(&buffer_clone, file_state_clone.clone());
    });
    app.add_action(&new_btn);

    // -------- SAVE FILE ICON ------
    let window_clone = window.clone();
    let buffer_clone = text_buffer.clone();
    let file_state_clone = file_state.clone();
    let save_btn = gio::SimpleAction::new("save", None);
    save_btn.connect_activate(move |_, _| {
        file_manager::file_ops::save_file(
            &window_clone,
            buffer_clone.clone(),
            file_state_clone.clone(),
        );
    });
    app.add_action(&save_btn);

    PopoverMenuBar::from_model(Some(&menu_model))
}
