use super::errors::invalid_character;
use super::handlers::LexerHandlers;
use crate::models::{LexicalError, Token};

pub fn analyze(text: &str) -> (Vec<Token>, Vec<LexicalError>) {
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    let mut line = 1;
    let mut column = 1;
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if LexerHandlers::handle_whitespace(c, &mut line, &mut column, &mut i) {
            continue;
        }

        if let Some(token) =
            LexerHandlers::handle_plus(c, &chars, line, column, &mut i, &mut line, &mut column)
        {
            tokens.push(token);
            continue;
        }

        if let Some(token) =
            LexerHandlers::handle_minus(c, &chars, line, column, &mut i, &mut line, &mut column)
        {
            tokens.push(token);
            continue;
        }

        if LexerHandlers::handle_line_comment(&chars, c, &mut i, &mut column) {
            continue;
        }

        if let Some(result) = LexerHandlers::handle_block_comment_start(
            c,
            &chars,
            line,
            column,
            &mut i,
            &mut line,
            &mut column,
        ) {
            if let Err(error) = result {
                errors.push(error);
            }
            continue;
        }

        if let Some((token, error)) =
            LexerHandlers::handle_string(c, &chars, line, column, &mut i, &mut line, &mut column)
        {
            push_token_or_error(&mut tokens, &mut errors, token, error);
            continue;
        }

        if let Some((token, error)) =
            LexerHandlers::handle_char(c, &chars, line, column, &mut i, &mut line, &mut column)
        {
            push_token_or_error(&mut tokens, &mut errors, token, error);
            continue;
        }

        if let Some((token, error)) =
            LexerHandlers::handle_number(c, &chars, line, column, &mut i, &mut column)
        {
            push_token_or_error(&mut tokens, &mut errors, token, error);
            continue;
        }

        if let Some(token) =
            LexerHandlers::handle_identifier(c, &chars, line, column, &mut i, &mut column)
        {
            tokens.push(token);
            continue;
        }

        if let Some(token) =
            LexerHandlers::handle_double_operator(&chars, line, column, &mut i, &mut column)
        {
            tokens.push(token);
            continue;
        }

        if let Some(token) = LexerHandlers::handle_arithmetic(c, line, column, &mut i, &mut column)
        {
            tokens.push(token);
            continue;
        }

if let Some(token) = LexerHandlers::handle_relational(c, &chars, line, column, &mut i, &mut column, &mut line) {
    tokens.push(token);
    continue;
}
        if let Some(token) = LexerHandlers::handle_symbol(c, line, column, &mut i, &mut column) {
            tokens.push(token);
            continue;
        }

        errors.push(invalid_character(c, line, column));
        i += 1;
        column += 1;
    }

    (tokens, errors)
}

fn push_token_or_error(
    tokens: &mut Vec<Token>,
    errors: &mut Vec<LexicalError>,
    token: Option<Token>,
    error: Option<LexicalError>,
) {
    if let Some(t) = token {
        tokens.push(t);
    }
    if let Some(e) = error {
        errors.push(e);
    }
}
