//! Present all phases from one current-buffer snapshot, never from stale token files.
use crate::{
    compiler,
    models::{LexicalError, SemanticError, SyntaxError},
    ui::panels::notebook::PanelNotebooks,
};
use gtk::{gio, prelude::*};

pub(crate) fn register(app: &gtk::Application, buffer: &gtk::TextBuffer, panels: &PanelNotebooks) {
    let changed_panels = panels.clone();
    buffer.connect_changed(move |_| changed_panels.clear());

    for (name, page) in [("semantic", 2), ("syntax", 1)] {
        let action = gio::SimpleAction::new(name, None);
        let buffer = buffer.clone();
        let panels = panels.clone();
        action.connect_activate(move |_, _| {
            let source = buffer.text(&buffer.start_iter(), &buffer.end_iter(), true);
            let result = compiler::analyze_source(&source);
            panels.clear();
            panels.debug.set_current_page(Some(page));
            panels.errors.set_current_page(Some(page));
            let tokens = panels.lexic_view.borrow().buffer();
            for token in &result.tokens {
                let color = super::actions::lexical_token_color(&token.token_type, &token.lexeme);
                let tag = tokens.create_tag(None, &[("foreground", &color)]);
                let line = format!(
                    "{}: '{}' ({}:{})\n",
                    token.token_type, token.lexeme, token.line, token.column
                );
                if let Some(tag) = tag {
                    tokens.insert_with_tags(&mut tokens.end_iter(), &line, &[&tag]);
                } else {
                    tokens.insert(&mut tokens.end_iter(), &line);
                }
            }
            let lexical = result
                .lexical_errors
                .iter()
                .map(
                    |LexicalError {
                         message,
                         line,
                         column,
                     }| (message.as_str(), *line, *column),
                )
                .collect::<Vec<_>>();
            diagnostics(
                &panels.errors_view.borrow(),
                "Lexical error",
                &lexical,
                None,
            );
            if !lexical.is_empty() {
                panels
                    .semantic_errors_view
                    .borrow()
                    .buffer()
                    .set_text("Semantic analysis not run: resolve lexical errors first.");
                return;
            }
            let syntax = result
                .syntax_errors
                .iter()
                .map(
                    |SyntaxError {
                         message,
                         line,
                         column,
                     }| (message.as_str(), *line, *column),
                )
                .collect::<Vec<_>>();
            diagnostics(
                &panels.syntax_errors_view.borrow(),
                "Syntax error",
                &syntax,
                None,
            );
            if let Some(ast) = &result.ast {
                panels.ast_view.borrow().populate(ast);
            }
            if let Some(semantic) = result.semantic {
                panels
                    .semantic_view
                    .borrow()
                    .populate(&semantic.annotated_ast);
                panels.symbols_view.populate(&semantic.symbols);
                let errors = semantic
                    .errors
                    .iter()
                    .map(
                        |SemanticError {
                             message,
                             line,
                             column,
                         }| (message.as_str(), *line, *column),
                    )
                    .collect::<Vec<_>>();
                let warning = (!syntax.is_empty())
                    .then_some("Syntax errors are present: semantic results are partial.\n");
                diagnostics(
                    &panels.semantic_errors_view.borrow(),
                    "Semantic error",
                    &errors,
                    warning,
                );
            }
        });
        app.add_action(&action);
    }
}

fn diagnostics(
    view: &gtk::TextView,
    heading: &str,
    errors: &[(&str, usize, usize)],
    warning: Option<&str>,
) {
    let buffer = view.buffer();
    buffer.set_text(warning.unwrap_or(""));
    let tag = buffer.tag_table().lookup("analysis-location").or_else(|| {
        buffer.create_tag(
            Some("analysis-location"),
            &[
                ("foreground", &"#1a73e8"),
                ("underline", &gtk::pango::Underline::Single),
            ],
        )
    });
    for (message, line, column) in errors {
        buffer.insert(&mut buffer.end_iter(), &format!("{heading}: {message} "));
        let location = format!("({line}:{column})\n");
        if let Some(tag) = &tag {
            buffer.insert_with_tags(&mut buffer.end_iter(), &location, &[tag]);
        } else {
            buffer.insert(&mut buffer.end_iter(), &location);
        }
    }
    if errors.is_empty() {
        let status = if warning.is_some() {
            "No additional semantic diagnostics on the recovered AST.".to_string()
        } else {
            format!("No {}s detected.", heading.to_lowercase())
        };
        buffer.insert(&mut buffer.end_iter(), &status);
    }
}
