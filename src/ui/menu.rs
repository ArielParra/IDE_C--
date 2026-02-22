use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, PopoverMenuBar, gio};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::process::Command;
//use gtk::{HeaderBar, Button, Picture, Box, Orientation};

use crate::file_manager;
//use crate::ui::menu;

pub fn build_menu(
    app: &Application,
    window: &ApplicationWindow,
    buffer: &impl IsA<gtk::TextBuffer>,
    file_state: Rc<RefCell<Option<PathBuf>>>,
) -> PopoverMenuBar {

    // === Menu Model ===
    let menu_model = gio::Menu::new();
    let text_buffer: gtk::TextBuffer = buffer.as_ref().clone();

    //FILE
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

    /*// === Creat section of icons ===    
    // ICON 1 - New
    let new_item = gio::MenuItem::new(Some("New1"), Some("app.new1"));
    let new_icon = gio::ThemedIcon::new("document-new1");
    //let new_icon = gio::FileIcon::new(&gio::File::for_path("icons/new.svg"));
    new_item.set_icon(&new_icon);
    file_menu.append_item(&new_item);

    // ICON 2 - Save
    let save_item = gio::MenuItem::new(Some("Save2"), Some("app.save2"));
    let save_icon = gio::FileIcon::new(&gio::File::for_path("icons/save.svg"));
    save_item.set_icon(&save_icon);
    file_menu.append_item(&save_item);

    // ICON 3 - Export
    let export_item = gio::MenuItem::new(Some("Export"), Some("app.export"));
    let export_icon = gio::FileIcon::new(&gio::File::for_path("icons/export.svg"));
    export_item.set_icon(&export_icon);
    file_menu.append_item(&export_item);*/

    //CLOSE
    let close_menu = gio::Menu::new();
    close_menu.append(Some("Close"), Some("app.close"));

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
    compilar_menu.append(Some("Compile"), Some("app.compile"));
    compilar_menu.append(Some("Análisis Léxico"), Some("app.lexico"));
    compilar_menu.append(Some("Análisis Sintáctico"), Some("app.sintactico"));
    compilar_menu.append(Some("Análisis Semántico"), Some("app.semantico"));
    compilar_menu.append(Some("Código Intermedio"), Some("app.intermedio"));
    compilar_menu.append(Some("Ejecución"), Some("app.ejecutar"));

    //Ramas del menu
    //FILE, EDIT, BUILD & DEBUG, ICONO1, ICONO2, ICONO3, CLOSE, LEXICO, SINTACTICO, SEMANTICO, COMPILAR  ETC.
    menu_model.append_submenu(Some("File"), &file_menu);
    menu_model.append_submenu(Some("Edit"), &edit_menu);
    menu_model.append_submenu(Some("Build & Debug"), &build_debug_menu);
    menu_model.append_submenu(Some("Close"), &close_menu);  

    // Visible Icons

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
        file_manager::file_ops::open_file(
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
    let compile_action = gio::SimpleAction::new("compile", None);
    compile_action.connect_activate(move |_, _| {

    if let Some(path) = &*file_state_clone.borrow() {

        let output = Command::new("./compilador.exe") // Ruta de lo que se va a compilar
            .arg(path)
            .output()
            .expect("No se pudo ejecutar el compilador");

        println!("Salida:");
        println!("{}", String::from_utf8_lossy(&output.stdout));

        println!("Errores:");
        println!("{}", String::from_utf8_lossy(&output.stderr));

        } else {
            println!("No hay archivo abierto.");
        }
    });

    app.add_action(&compile_action);




    PopoverMenuBar::from_model(Some(&menu_model))
}