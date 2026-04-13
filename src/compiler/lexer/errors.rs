use crate::models::LexicalError;

pub fn skip_whitespace(chars: &[char], start: usize) -> (Option<char>, usize, usize, usize) {
    let mut i = start;
    let mut line_breaks = 0;
    let mut spaces = 0;

    while i < chars.len() && (chars[i] == ' ' || chars[i] == '\t' || chars[i] == '\n') {
        if chars[i] == '\n' {
            line_breaks += 1;
            spaces = 0;
        } else {
            spaces += 1;
        }
        i += 1;
    }

    let next = if i < chars.len() {
        Some(chars[i])
    } else {
        None
    };
    (next, i, line_breaks, spaces)
}

pub fn invalid_character(c: char, line: usize, column: usize) -> LexicalError {
    LexicalError::invalid_character(c, line, column)
}
