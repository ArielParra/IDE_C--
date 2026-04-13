#[derive(Debug, Clone)]
pub struct LexicalError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl LexicalError {
    pub fn new(message: &str, line: usize, column: usize) -> Self {
        Self {
            message: message.to_string(),
            line,
            column,
        }
    }

    pub fn unclosed_block_comment(line: usize, column: usize) -> Self {
        Self::new("Unclosed block comment", line, column)
    }

    pub fn unclosed_string(content: &str, line: usize, column: usize) -> Self {
        Self::new(&format!("Unclosed string: '{}'", content), line, column)
    }

    pub fn unclosed_char(content: &str, line: usize, column: usize) -> Self {
        Self::new(&format!("Unclosed char: '{}'", content), line, column)
    }

    pub fn malformed_number(lexeme: &str, line: usize, column: usize) -> Self {
        Self::new(&format!("Malformed number: '{}'", lexeme), line, column)
    }

    pub fn invalid_character(c: char, line: usize, column: usize) -> Self {
        Self::new(&format!("Invalid character {}", c), line, column)
    }
}
