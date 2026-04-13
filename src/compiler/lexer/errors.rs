use crate::models::LexicalError;

pub fn invalid_character(c: char, line: usize, column: usize) -> LexicalError {
    LexicalError::invalid_character(c, line, column)
}
