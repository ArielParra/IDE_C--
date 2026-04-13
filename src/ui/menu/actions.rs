use gtk::prelude::*;
use gtk::{gio, pango::Underline, Application, ApplicationWindow, TextView};
use sourceview5::View as SourceView;
use std::cell::RefCell;
use std::io::BufRead;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::rc::Rc;

use crate::file_manager;
use crate::compiler;
use super::navigator::{ErrorNavigator, LexicNavigator};

pub struct ActionHandlers;

impl ActionHandlers {
    pub fn register_all(
        app: &Application,
        window: &ApplicationWindow,
        buffer: &impl IsA<gtk::TextBuffer>,
        editor_view: SourceView,
        file_state: Rc<RefCell<Option<PathBuf>>>,
        lex_view: Rc<RefCell<TextView>>,
        errors_view: Rc<RefCell<TextView>>,
    ) {
        let text_buffer: gtk::TextBuffer = buffer.as_ref().clone();
        let buffer_clone = text_buffer.clone();

        Self::register_file_actions(app, &window, &buffer_clone, file_state.clone());
        Self::register_lexical_action(
            app,
            &buffer_clone,
            lex_view,
            errors_view,
            editor_view,
        );
        Self::register_compile_action(app, file_state);
    }

    fn register_file_actions(
        app: &Application,
        window: &ApplicationWindow,
        buffer: &gtk::TextBuffer,
        file_state: Rc<RefCell<Option<PathBuf>>>,
    ) {
        let buffer_clone = buffer.clone();
        let new_action = gio::SimpleAction::new("new", None);
        let file_state_clone = file_state.clone();
        new_action.connect_activate(move |_, _| {
            file_manager::file_ops::new_file(&buffer_clone, file_state_clone.clone());
        });
        app.add_action(&new_action);

        let open_action = gio::SimpleAction::new("open", None);
        let window_clone = window.clone();
        let buffer_clone = buffer.clone();
        let file_state_clone = file_state.clone();
        open_action.connect_activate(move |_, _| {
            file_manager::file_ops::open_file_dialog(
                &window_clone,
                buffer_clone.clone(),
                file_state_clone.clone(),
            );
        });
        app.add_action(&open_action);

        let close_action = gio::SimpleAction::new("close", None);
        let app_clone = app.clone();
        close_action.connect_activate(move |_, _| {
            app_clone.activate_action("new", None);
        });
        app.add_action(&close_action);

        let save_action = gio::SimpleAction::new("save", None);
        let window_clone = window.clone();
        let buffer_clone = buffer.clone();
        let file_state_clone = file_state.clone();
        save_action.connect_activate(move |_, _| {
            file_manager::file_ops::save_file(
                &window_clone,
                buffer_clone.clone(),
                file_state_clone.clone(),
            );
        });
        app.add_action(&save_action);

        let save_as_action = gio::SimpleAction::new("save_as", None);
        let window_clone = window.clone();
        let buffer_clone = buffer.clone();
        let file_state_clone = file_state.clone();
        save_as_action.connect_activate(move |_, _| {
            file_manager::file_ops::save_as_file_dialog(
                &window_clone,
                buffer_clone.clone(),
                file_state_clone.clone(),
            );
        });
        app.add_action(&save_as_action);

        let exit_action = gio::SimpleAction::new("exit", None);
        let app_clone = app.clone();
        exit_action.connect_activate(move |_, _| {
            app_clone.quit();
        });
        app.add_action(&exit_action);
    }

    fn register_lexical_action(
        app: &Application,
        buffer: &gtk::TextBuffer,
        lex_view: Rc<RefCell<TextView>>,
        errors_view: Rc<RefCell<TextView>>,
        editor_view: SourceView,
    ) {
        let lexical_action = gio::SimpleAction::new("lexical", None);
        let buffer_clone = buffer.clone();
        let lex_view_clone = lex_view.clone();
        let err_view_clone = errors_view.clone();
        let editor_view_clone = editor_view.clone();

        lexical_action.connect_activate(move |_, _| {
            let text = buffer_clone.text(&buffer_clone.start_iter(), &buffer_clone.end_iter(), true);
            let (tokens, errors) = compiler::analyze(&text);

            let lex_buffer = lex_view_clone.borrow().buffer();
            lex_buffer.set_text("");
            
            let link_tag = lex_buffer
                .create_tag(Some("link"), &[("foreground", &"#1a73e8"), ("underline", &Underline::Single)])
                .expect("failed to create link tag");

            for t in &tokens {
                let color = lexical_token_color(&t.token_type, &t.lexeme);
                let color_tag = lex_buffer
                    .create_tag(None, &[("foreground", &color)])
                    .expect("failed to create color tag");

                let mut iter = lex_buffer.end_iter();
                lex_buffer.insert_with_tags(&mut iter, &format!("{}: '{}' ", t.token_type, t.lexeme), &[&color_tag]);
                
                let mut link_iter = lex_buffer.end_iter();
                let link_text = format!("({}:{})\n", t.line, t.column);
                lex_buffer.insert_with_tags(&mut link_iter, &link_text, &[&link_tag]);
            }

            let err_buffer = err_view_clone.borrow().buffer();
            err_buffer.set_text("");

            let error_link_tag = err_buffer
                .create_tag(None, &[("foreground", &"#1a73e8"), ("underline", &Underline::Single)])
                .expect("failed to create error link tag");

            for e in &errors {
                let mut message_iter = err_buffer.end_iter();
                err_buffer.insert(&mut message_iter, &format!("{} ", e.message));

                let mut link_iter = err_buffer.end_iter();
                let link_text = format!("({}:{})", e.line, e.column);
                err_buffer.insert_with_tags(&mut link_iter, &link_text, &[&error_link_tag]);

                let mut newline_iter = err_buffer.end_iter();
                err_buffer.insert(&mut newline_iter, "\n");
            }

            LexicNavigator::connect_position_click(&lex_view_clone, &buffer_clone, &editor_view_clone);
            ErrorNavigator::connect_error_click(&err_view_clone, &buffer_clone, &editor_view_clone);
        });

        app.add_action(&lexical_action);
    }

    fn register_compile_action(
        app: &Application,
        file_state: Rc<RefCell<Option<PathBuf>>>,
    ) {
        let compile_action = gio::SimpleAction::new("c--compiler", None);
        let file_state_clone = file_state.clone();

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

            if let Some(stdout) = child.stdout.take() {
                std::thread::spawn(move || {
                    let reader = std::io::BufReader::new(stdout);
                    for line in reader.lines() {
                        if let Ok(l) = line {
                            println!("OUT: {}", l);
                        }
                    }
                });
            }

            if let Some(stderr) = child.stderr.take() {
                std::thread::spawn(move || {
                    let reader = std::io::BufReader::new(stderr);
                    for line in reader.lines() {
                        if let Ok(l) = line {
                            eprintln!("ERR: {}", l);
                        }
                    }
                });
            }
        });

        app.add_action(&compile_action);
    }
}

fn lexical_token_color(tipo: &str, lexema: &str) -> &'static str {
    match tipo {
        "MAIN" | "IF" | "ELSE" | "END" | "DO" | "WHILE" | "FOR" | "SWITCH"
        | "CASE" | "RETURN" | "VOID" | "INT_T" | "FLOAT_T" | "CHAR_T" | "BOOL_T"
        | "TRUE" | "FALSE" | "CIN" | "COUT" | "INCLUDE" | "DEFINE" | "STRUCT"
        | "BREAK" | "CONTINUE" => "#569cd6",
        "INT" | "FLOAT" => "#b5cea8",
        "STRING" | "CHAR" => "#ce9178",
        "ID" => "#ff57f4",
        "SYM" => "#d7ba7d",
        "ARIT" | "OP" | "ASIG" => "#f44747",
        "REL" => {
            if lexema == "=" {
                "#f44747"
            } else {
                "#569cd6"
            }
        }
        _ => "#ffffff",
    }
}
