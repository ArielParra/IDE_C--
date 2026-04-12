use gtk::{TextView, prelude::*};
use gtk::{Application, ApplicationWindow, PopoverMenuBar, gio};
use gtk::pango::Underline;
use std::cell::RefCell;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::thread;
use std::path::PathBuf;
use std::rc::Rc;
use sourceview5::View as SourceView;

use crate::file_manager;
use crate::compiler;

pub fn build_menu(
    app: &Application,
    window: &ApplicationWindow,
    buffer: &impl IsA<gtk::TextBuffer>,
    editor_view: SourceView,
    file_state: Rc<RefCell<Option<PathBuf>>>,
    lexic_view: Rc<RefCell<TextView>>,
    errors_view: Rc<RefCell<TextView>>,
) -> PopoverMenuBar {

    fn parse_error_position(line: &str) -> Option<(usize, usize)> {
        let open = line.rfind('(')?;
        let close = line.rfind(')')?;

        if close <= open + 1 {
            return None;
        }

        let pos = &line[open + 1..close];
        let (line_s, col_s) = pos.split_once(':')?;

        let line_n = line_s.trim().parse::<usize>().ok()?;
        let col_n = col_s.trim().parse::<usize>().ok()?;

        if line_n == 0 || col_n == 0 {
            return None;
        }

        Some((line_n, col_n))
    }

    fn go_to_error_position(editor_view: &SourceView, editor_buffer: &gtk::TextBuffer, line: usize, col: usize) {
        let line_idx = (line - 1) as i32;
        let col_idx = (col - 1) as i32;

        let mut iter = match editor_buffer.iter_at_line_offset(line_idx, col_idx) {
            Some(it) => it,
            None => match editor_buffer.iter_at_line(line_idx) {
                Some(it) => it,
                None => return,
            },
        };

        editor_buffer.place_cursor(&iter);

        let mut end = iter;
        if !end.ends_line() {
            end.forward_char();
        }
        editor_buffer.select_range(&iter, &end);

        editor_view.scroll_to_iter(&mut iter, 0.2, false, 0.0, 0.0);
        editor_view.grab_focus();
    }

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
    build_debug_menu.append(Some("Compile1"), Some("app.compile1")); // Will test in the compile section
    build_debug_menu.append(Some("Run"), Some("app.run"));
    build_debug_menu.append(Some("Debug"), Some("app.debug"));

    //LEXICAL
    let lexico_menu = gio::Menu::new();
    lexico_menu.append(Some("Run Lexical Analysis"), Some("app.lexical"));

    //SYNTAX
    let sintactico_menu = gio::Menu::new();
    sintactico_menu.append(Some("Run Syntax Analysis"), Some("app.syntax"));

    //SEMANTIC
    let semantico_menu = gio::Menu::new();
    semantico_menu.append(Some("Run Semantic Analysis"), Some("app.semantic"));

    //COMPILE
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

    // LEXICAL ANALYSIS
    let buffer_clone = text_buffer.clone();
    let lex_view_clone = lexic_view.clone();
    let err_view_clone = errors_view.clone();

    let lexical_action = gio::SimpleAction::new("lexical", None);

    lexical_action.connect_activate(move |_, _| {
        let texto = buffer_clone.text(&buffer_clone.start_iter(), &buffer_clone.end_iter(), true);

        let (tokens, errores) = compiler::lexer::analizar(&texto);

        let lex_buffer = lex_view_clone.borrow().buffer();
        lex_buffer.set_text("");
        for t in &tokens {
            lex_buffer.insert_at_cursor(&format!("{}: '{}' ({}:{})\n",
                t.tipo, t.lexema, t.linea, t.columna));
        }

        let err_buffer = err_view_clone.borrow().buffer();
        err_buffer.set_text("");
        let error_link_tag = err_buffer
            .create_tag(
                None,
                &[("foreground", &"#1a73e8"), ("underline", &Underline::Single)],
            )
            .expect("failed to create error link tag");
        for e in &errores {
            let mut message_iter = err_buffer.end_iter();
            err_buffer.insert(&mut message_iter, &format!("{} ", e.message));

            let mut link_iter = err_buffer.end_iter();
            let link_text = format!("({}:{})", e.linea, e.columna);
            err_buffer.insert_with_tags(&mut link_iter, &link_text, &[&error_link_tag]);

            let mut newline_iter = err_buffer.end_iter();
            err_buffer.insert(&mut newline_iter, "\n");
        }
    });
    app.add_action(&lexical_action);

    // Navigate to editor when clicking on a lexical error line with format: "... (line:column)"
    let errors_textview_for_nav = errors_view.borrow().clone();
    let errors_buffer_for_nav = errors_textview_for_nav.buffer();
    let editor_buffer_for_nav = text_buffer.clone();
    let editor_view_for_nav = editor_view.clone();

    errors_buffer_for_nav.connect_mark_set(move |buf, iter, mark| {
        if mark.name().as_deref() != Some("insert") {
            return;
        }

        let mut start = *iter;
        start.set_line_offset(0);

        let mut end = start;
        end.forward_to_line_end();

        let line_text = buf.text(&start, &end, true).to_string();

        if let Some((linea, columna)) = parse_error_position(&line_text) {
            go_to_error_position(&editor_view_for_nav, &editor_buffer_for_nav, linea, columna);
        }
    });

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



    PopoverMenuBar::from_model(Some(&menu_model))
}
