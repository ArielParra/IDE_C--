#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: String,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: &str, lexeme: &str, line: usize, column: usize) -> Self {
        Self {
            token_type: token_type.to_string(),
            lexeme: lexeme.to_string(),
            line,
            column,
        }
    }
}
