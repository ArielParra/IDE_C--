use crate::models::Token;
use std::io::Write;
use std::path::Path;

/// Writes tokens to a file in the format: `TOKEN_TYPE lexeme line column`
/// One token per line. This file can later be read by `parse_tokens_from_text`.
pub fn write_tokens_to_file(tokens: &[Token], path: &Path) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    for t in tokens {
        writeln!(file, "{} {} {} {}", t.token_type, t.lexeme, t.line, t.column)?;
    }
    Ok(())
}

/// Reads tokens from a text file where each line has the format:
/// `TOKEN_TYPE lexeme line column`
///
/// The parser splits from the right to handle lexemes that contain spaces
/// (e.g. string literals).
pub fn parse_tokens_from_text(input: &str) -> Vec<Token> {
    input
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }

            let parts: Vec<&str> = trimmed.rsplitn(3, ' ').collect();
            if parts.len() != 3 {
                return None;
            }
            let column = parts[0].parse::<usize>().unwrap_or(0);
            let line = parts[1].parse::<usize>().unwrap_or(0);
            let rest = parts[2];
            let first_space = rest.find(' ')?;
            let token_type = &rest[..first_space];
            let lexeme = rest[first_space + 1..].to_string();
            Some(Token::new(token_type, &lexeme, line, column))
        })
        .collect()
}

/// Reads tokens from a file path, returning an empty vec on error.
pub fn read_tokens_from_file(path: &Path) -> Vec<Token> {
    match std::fs::read_to_string(path) {
        Ok(contents) => parse_tokens_from_text(&contents),
        Err(_) => Vec::new(),
    }
}
