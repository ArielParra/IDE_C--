use gtk::prelude::*;
use gtk::{gio, pango::Underline, Application, ApplicationWindow, TextView, SearchBar, SearchEntry};
use sourceview5::View as SourceView;
use std::cell::RefCell;
use std::io::BufRead;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::rc::Rc;

use crate::compiler;
use crate::file_manager;

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
        syntax_errors_view: Rc<RefCell<TextView>>,
        ast_view: Rc<RefCell<crate::ui::panels::AstView>>,
        debug_notebook: gtk::Notebook,
        errors_notebook: gtk::Notebook,
        search_bar: SearchBar,
        search_entry: SearchEntry,
    ) {
        let text_buffer: gtk::TextBuffer = buffer.as_ref().clone();
        let buffer_clone = text_buffer.clone();

        Self::register_file_actions(
            app,
            &window,
            &buffer_clone,
            file_state.clone(),
            lex_view.clone(),
            errors_view.clone(),
            syntax_errors_view.clone(),
            ast_view.clone(),
        );
        Self::register_find_action(app, window, &buffer_clone, &editor_view, search_bar, search_entry);
        Self::register_lexical_action(app, &buffer_clone, lex_view, errors_view, debug_notebook.clone(), errors_notebook.clone(), file_state.clone());
        Self::register_compile_action(app, window, file_state.clone());
        Self::register_syntax_action(app, &buffer_clone, syntax_errors_view, ast_view, debug_notebook, errors_notebook, file_state);
    }

    fn register_file_actions(
        app: &Application,
        window: &ApplicationWindow,
        buffer: &gtk::TextBuffer,
        file_state: Rc<RefCell<Option<PathBuf>>>,
        lex_view: Rc<RefCell<TextView>>,
        errors_view: Rc<RefCell<TextView>>,
        syntax_errors_view: Rc<RefCell<TextView>>,
        ast_view: Rc<RefCell<crate::ui::panels::AstView>>,
    ) {
        let buffer_clone = buffer.clone();
        let new_action = gio::SimpleAction::new("new", None);
        let file_state_clone = file_state.clone();
        let lex_view_clone = lex_view.clone();
        let errors_view_clone = errors_view.clone();
        let syntax_errors_view_clone = syntax_errors_view.clone();
        let ast_view_clone = ast_view.clone();
        let window_clone_new = window.clone();
        new_action.connect_activate(move |_, _| {
            file_manager::file_ops::flash_action_button(&window_clone_new, "app.new");
            file_manager::file_ops::new_file(
                &buffer_clone,
                file_state_clone.clone(),
                lex_view_clone.clone(),
                errors_view_clone.clone(),
                syntax_errors_view_clone.clone(),
                ast_view_clone.clone(),
            );
        });
        app.add_action(&new_action);

        let open_action = gio::SimpleAction::new("open", None);
        let window_clone = window.clone();
        let buffer_clone = buffer.clone();
        let file_state_clone = file_state.clone();
        let lex_view_clone = lex_view.clone();
        let errors_view_clone = errors_view.clone();
        let syntax_errors_view_clone = syntax_errors_view.clone();
        let ast_view_clone2 = ast_view.clone();
        open_action.connect_activate(move |_, _| {
            file_manager::file_ops::flash_action_button(&window_clone, "app.open");
            file_manager::file_ops::open_file_dialog(
                &window_clone,
                buffer_clone.clone(),
                file_state_clone.clone(),
                lex_view_clone.clone(),
                errors_view_clone.clone(),
                syntax_errors_view_clone.clone(),
                ast_view_clone2.clone(),
            );
        });
        app.add_action(&open_action);

        let close_action = gio::SimpleAction::new("close", None);
        let app_clone = app.clone();
        let window_clone_close = window.clone();
        close_action.connect_activate(move |_, _| {
            file_manager::file_ops::flash_action_button(&window_clone_close, "app.close");
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
            file_manager::file_ops::flash_action_button(&window_clone, "app.save_as");
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

    fn register_find_action(
        app: &Application,
        window: &ApplicationWindow,
        buffer: &gtk::TextBuffer,
        editor_view: &SourceView,
        search_bar: SearchBar,
        search_entry: SearchEntry,
    ) {
        let find_action = gio::SimpleAction::new("find", None);
        let search_bar_clone = search_bar.clone();
        let search_entry_clone = search_entry.clone();

        let window_clone_find = window.clone();
        find_action.connect_activate(move |_, _| {
            file_manager::file_ops::flash_action_button(&window_clone_find, "app.find");
            search_bar_clone.set_search_mode(true);
            search_entry_clone.grab_focus();
        });

        let buffer_for_response = buffer.clone();
        let view_for_response = editor_view.clone();
        let entry_for_response = search_entry.clone();

        search_entry.connect_activate(move |_| {
            let query = entry_for_response.text().to_string();
            if !query.is_empty() {
                find_next_in_buffer(&buffer_for_response, &view_for_response, &query);
            }
        });

        app.add_action(&find_action);
    }

    fn register_lexical_action(
        app: &Application,
        buffer: &gtk::TextBuffer,
        lex_view: Rc<RefCell<TextView>>,
        errors_view: Rc<RefCell<TextView>>,
        debug_notebook: gtk::Notebook,
        errors_notebook: gtk::Notebook,
        file_state: Rc<RefCell<Option<PathBuf>>>,
    ) {
        let lexical_action = gio::SimpleAction::new("lexical", None);
        let buffer_clone = buffer.clone();
        let lex_view_clone = lex_view.clone();
        let err_view_clone = errors_view.clone();
        let file_state_clone = file_state.clone();

        lexical_action.connect_activate(move |_, _| {
            debug_notebook.set_current_page(Some(0));
            errors_notebook.set_current_page(Some(0));

            let text =
                buffer_clone.text(&buffer_clone.start_iter(), &buffer_clone.end_iter(), true);
            let (tokens, errors) = compiler::analyze(&text);

            // Save tokens to file for the syntax analyzer
            let tokens_path = match &*file_state_clone.borrow() {
                Some(p) => {
                    let mut p = p.clone();
                    let file_name = p.file_name().unwrap().to_string_lossy().into_owned();
                    p.set_file_name(format!("{}.tokens", file_name));
                    p
                }
                None => std::path::PathBuf::from("untitled.c--.tokens"),
            };

            if let Err(e) = crate::compiler::parser::write_tokens_to_file(&tokens, &tokens_path) {
                eprintln!("Failed to write tokens file: {}", e);
            }

            let lex_buffer = lex_view_clone.borrow().buffer();
            lex_buffer.set_text("");

            let link_tag = lex_buffer.create_tag(
                None,
                &[
                    ("foreground", &"#1a73e8"),
                    ("underline", &Underline::Single),
                ],
            );

            for t in &tokens {
                let color = lexical_token_color(&t.token_type, &t.lexeme);
                let color_tag = lex_buffer.create_tag(None, &[("foreground", &color)]);

                let mut iter = lex_buffer.end_iter();
                if let Some(ref tag) = color_tag {
                    lex_buffer.insert_with_tags(
                        &mut iter,
                        &format!("{}: '{}' ", t.token_type, t.lexeme),
                        &[tag],
                    );
                } else {
                    lex_buffer.insert(&mut iter, &format!("{}: '{}' ", t.token_type, t.lexeme));
                }

                let mut link_iter = lex_buffer.end_iter();
                let link_text = format!("({}:{})\n", t.line, t.column);
                if let Some(ref tag) = link_tag {
                    lex_buffer.insert_with_tags(&mut link_iter, &link_text, &[tag]);
                } else {
                    lex_buffer.insert(&mut link_iter, &link_text);
                }
            }

            let err_buffer = err_view_clone.borrow().buffer();
            err_buffer.set_text("");

            let error_link_tag = err_buffer.create_tag(
                None,
                &[
                    ("foreground", &"#1a73e8"),
                    ("underline", &Underline::Single),
                ],
            );

            if errors.is_empty() {
                err_buffer.set_text("No lexical errors detected.");
            } else {
                for e in &errors {
                    let mut message_iter = err_buffer.end_iter();
                    err_buffer.insert(&mut message_iter, &format!("{} ", e.message));

                    let mut link_iter = err_buffer.end_iter();
                    let link_text = format!("({}:{})", e.line, e.column);
                    if let Some(ref tag) = error_link_tag {
                        err_buffer.insert_with_tags(&mut link_iter, &link_text, &[tag]);
                    } else {
                        err_buffer.insert(&mut link_iter, &link_text);
                    }

                    let mut newline_iter = err_buffer.end_iter();
                    err_buffer.insert(&mut newline_iter, "\n");
                }
            }
        });

        app.add_action(&lexical_action);
    }

    fn register_syntax_action(
        app: &Application,
        buffer: &gtk::TextBuffer,
        syntax_errors_view: Rc<RefCell<TextView>>,
        ast_view: Rc<RefCell<crate::ui::panels::AstView>>,
        debug_notebook: gtk::Notebook,
        errors_notebook: gtk::Notebook,
        file_state: Rc<RefCell<Option<PathBuf>>>,
    ) {
        let syntax_action = gio::SimpleAction::new("syntax", None);
        let _buffer_clone = buffer.clone();
        let syntax_errors_view_clone = syntax_errors_view.clone();
        let ast_view_clone = ast_view.clone();
        let file_state_clone = file_state.clone();

        syntax_action.connect_activate(move |_, _| {
            debug_notebook.set_current_page(Some(1));
            errors_notebook.set_current_page(Some(1));

            let err_buffer = syntax_errors_view_clone.borrow().buffer();
            err_buffer.set_text("");

            // Read tokens from the file generated by lexical analysis
            let tokens_path = match &*file_state_clone.borrow() {
                Some(p) => {
                    let mut p = p.clone();
                    let file_name = p.file_name().unwrap().to_string_lossy().into_owned();
                    p.set_file_name(format!("{}.tokens", file_name));
                    p
                }
                None => std::path::PathBuf::from("untitled.c--.tokens"),
            };
            let tokens = crate::compiler::parser::read_tokens_from_file(&tokens_path);

            if tokens.is_empty() {
                let error_tag = err_buffer.create_tag(
                    None,
                    &[
                        ("foreground", &"#f44747"),
                        ("weight", &700i32),
                    ],
                );
                let mut iter = err_buffer.end_iter();
                let message = "⚠ Error: No tokens found. Please run Lexical Analysis first.";
                if let Some(ref tag) = error_tag {
                    err_buffer.insert_with_tags(&mut iter, message, &[tag]);
                } else {
                    err_buffer.insert(&mut iter, message);
                }
                return;
            }

            let (ast, syntax_errors) = crate::compiler::parser::build_ast_from_tokens(&tokens);
            ast_view_clone.borrow().populate(&ast);

            let error_link_tag = err_buffer.create_tag(
                None,
                &[
                    ("foreground", &"#1a73e8"),
                    ("underline", &Underline::Single),
                ],
            );

            if syntax_errors.is_empty() {
                err_buffer.set_text("No syntax errors detected.");
            } else {
                for error in &syntax_errors {
                    let mut message_iter = err_buffer.end_iter();
                    err_buffer.insert(&mut message_iter, &format!("Syntax error: {} ", error.message));

                    let mut link_iter = err_buffer.end_iter();
                    let link_text = format!("({}:{})", error.line, error.column);
                    if let Some(ref tag) = error_link_tag {
                        err_buffer.insert_with_tags(&mut link_iter, &link_text, &[tag]);
                    } else {
                        err_buffer.insert(&mut link_iter, &link_text);
                    }

                    let mut newline_iter = err_buffer.end_iter();
                    err_buffer.insert(&mut newline_iter, "\n");
                }
            }
        });

        app.add_action(&syntax_action);
    }

    fn register_compile_action(app: &Application, window: &ApplicationWindow, file_state: Rc<RefCell<Option<PathBuf>>>) {
        let compile_action = gio::SimpleAction::new("c--compiler", None);
        let file_state_clone = file_state.clone();

        let window_clone_compile = window.clone();

        compile_action.connect_activate(move |_, _| {
            file_manager::file_ops::flash_action_button(&window_clone_compile, "app.c--compiler");
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
        "MAIN" | "IF" | "ELSE" | "END" | "DO" | "WHILE" | "FOR" | "SWITCH" | "CASE" | "RETURN"
        | "VOID" | "INT_T" | "FLOAT_T" | "CHAR_T" | "BOOL_T" | "TRUE" | "FALSE" | "CIN"
        | "COUT" | "INCLUDE" | "DEFINE" | "STRUCT" | "BREAK" | "CONTINUE" => "#569cd6",
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

fn find_next_in_buffer(buffer: &gtk::TextBuffer, view: &SourceView, query: &str) {
    let text = buffer
        .text(&buffer.start_iter(), &buffer.end_iter(), true)
        .to_string();

    let start_offset = if let Some((_, end)) = buffer.selection_bounds() {
        end.offset().max(0) as usize
    } else {
        buffer.cursor_position().max(0) as usize
    };

    let found_offset = find_char_offset(&text, query, start_offset)
        .or_else(|| find_char_offset(&text, query, 0));

    if let Some(found_offset) = found_offset {
        let query_len = query.chars().count();
        let mut start_iter = buffer.iter_at_offset(found_offset as i32);
        let end_iter = buffer.iter_at_offset((found_offset + query_len) as i32);
        buffer.select_range(&start_iter, &end_iter);
        view.scroll_to_iter(&mut start_iter, 0.1, false, 0.0, 0.0);
    }
}

fn find_char_offset(text: &str, query: &str, start_char: usize) -> Option<usize> {
    let haystack: Vec<char> = text.chars().collect();
    let needle: Vec<char> = query.chars().collect();

    if needle.is_empty() || start_char >= haystack.len() {
        return None;
    }

    haystack[start_char..]
        .windows(needle.len())
        .position(|window| window == needle.as_slice())
        .map(|index| start_char + index)
}
