use crate::models::{LexicalError, Token};
use super::errors::skip_whitespace;
use super::tokenizer::Tokenizer;

pub struct LexerHandlers;

impl LexerHandlers {
    pub fn handle_whitespace(c: char, line: &mut usize, column: &mut usize, i: &mut usize) -> bool {
        if c == ' ' || c == '\t' {
            *column += 1;
            *i += 1;
            return true;
        }
        if c == '\n' {
            *line += 1;
            *column = 1;
            *i += 1;
            return true;
        }
        false
    }

    pub fn handle_plus(c: char, chars: &[char], line: usize, column: usize, i: &mut usize, line_out: &mut usize, column_out: &mut usize) -> Option<Token> {
        if c != '+' {
            return None;
        }
        let (next, pos_next, breaks, spaces) = skip_whitespace(chars, *i + 1);
        if let Some('+') = next {
            *i = pos_next + 1;
            *line_out += breaks;
            *column_out = if breaks > 0 { 1 + spaces } else { column + 2 + spaces };
            Some(Tokenizer::new_token("OP", "++", line, column))
        } else {
            *i += 1;
            *column_out = column + 1;
            Some(Tokenizer::new_token("ARIT", "+", line, column))
        }
    }

    pub fn handle_minus(c: char, chars: &[char], line: usize, column: usize, i: &mut usize, line_out: &mut usize, column_out: &mut usize) -> Option<Token> {
        if c != '-' {
            return None;
        }
        let (next, pos_next, breaks, spaces) = skip_whitespace(chars, *i + 1);
        if let Some('-') = next {
            *i = pos_next + 1;
            *line_out += breaks;
            *column_out = if breaks > 0 { 1 + spaces } else { column + 2 + spaces };
            Some(Tokenizer::new_token("OP", "--", line, column))
        } else {
            *i += 1;
            *column_out = column + 1;
            Some(Tokenizer::new_token("ARIT", "-", line, column))
        }
    }

    pub fn handle_line_comment(chars: &[char], c: char, i: &mut usize, column: &mut usize) -> bool {
        if c != '/' || *i + 1 >= chars.len() || chars[*i + 1] != '/' {
            return false;
        }
        *i += 2;
        *column += 2;
        while *i < chars.len() && chars[*i] != '\n' {
            *i += 1;
            *column += 1;
        }
        true
    }

    pub fn handle_block_comment_start(c: char, chars: &[char], line: usize, column: usize, i: &mut usize, line_out: &mut usize, column_out: &mut usize) -> Option<LexicalError> {
        if c != '/' || *i + 1 >= chars.len() || chars[*i + 1] != '*' {
            return None;
        }
        let start_line = line;
        let start_col = column;
        *i += 2;
        *column_out += 2;
        while *i + 1 < chars.len() && !(chars[*i] == '*' && chars[*i + 1] == '/') {
            if chars[*i] == '\n' {
                *line_out += 1;
                *column_out = 1;
            } else {
                *column_out += 1;
            }
            *i += 1;
        }
        if *i + 1 >= chars.len() {
            *i += 1;
            return Some(LexicalError::unclosed_block_comment(start_line, start_col));
        }
        *i += 2;
        *column_out += 2;
        None
    }

    pub fn handle_string(c: char, chars: &[char], line: usize, column: usize, i: &mut usize, line_out: &mut usize, column_out: &mut usize) -> Option<(Option<Token>, Option<LexicalError>)> {
        if c != '"' {
            return None;
        }
        let start_col = column;
        let start = *i;
        let start_line = line;
        *i += 1;
        *column_out += 1;
        while *i < chars.len() && chars[*i] != '"' {
            if chars[*i] == '\n' {
                *line_out += 1;
                *column_out = 1;
            } else {
                *column_out += 1;
            }
            *i += 1;
        }
        if *i >= chars.len() {
            *i += 1;
            let content: String = chars[start..].iter().collect();
            return Some((None, Some(LexicalError::unclosed_string(&content, start_line, start_col))));
        }
        *i += 1;
        *column_out += 1;
        let lexeme: String = chars[start..*i].iter().collect();
        Some((Some(Tokenizer::new_token("STRING", &lexeme, start_line, start_col)), None))
    }

    pub fn handle_char(c: char, chars: &[char], line: usize, column: usize, i: &mut usize, line_out: &mut usize, column_out: &mut usize) -> Option<(Option<Token>, Option<LexicalError>)> {
        if c != '\'' {
            return None;
        }
        let start_col = column;
        let start = *i;
        let start_line = line;
        *i += 1;
        *column_out += 1;
        while *i < chars.len() && chars[*i] != '\'' {
            if chars[*i] == '\n' {
                *line_out += 1;
                *column_out = 1;
            } else {
                *column_out += 1;
            }
            *i += 1;
        }
        if *i >= chars.len() {
            *i += 1;
            let content: String = chars[start..].iter().collect();
            return Some((None, Some(LexicalError::unclosed_char(&content, start_line, start_col))));
        }
        *i += 1;
        *column_out += 1;
        let lexeme: String = chars[start..*i].iter().collect();
        Some((Some(Tokenizer::new_token("CHAR", &lexeme, start_line, start_col)), None))
    }

    pub fn handle_number(c: char, chars: &[char], line: usize, column: usize, i: &mut usize, column_out: &mut usize) -> Option<(Option<Token>, Option<LexicalError>)> {
        if !c.is_ascii_digit() {
            return None;
        }
        let start = *i;
        let start_col = column;
        let start_line = line;
        let mut has_dot = false;
        let mut dot_error_col = 0;
        while *i < chars.len() {
            if chars[*i].is_ascii_digit() {
                *i += 1;
                *column_out += 1;
            } else if chars[*i] == '.' && !has_dot {
                if *i + 1 < chars.len() && chars[*i + 1].is_ascii_digit() {
                    has_dot = true;
                    *i += 1;
                    *column_out += 1;
                } else {
                    dot_error_col = *column_out;
                    break;
                }
            } else {
                break;
            }
        }
        if dot_error_col > 0 {
            let lexeme_error: String = chars[start..=*i].iter().collect();
            *i += 1;
            *column_out += 1;
            return Some((None, Some(LexicalError::malformed_number(&lexeme_error, start_line, dot_error_col))));
        }
        let lexeme: String = chars[start..*i].iter().collect();
        let token_type = if has_dot { "FLOAT" } else { "INT" };
        Some((Some(Tokenizer::new_token(token_type, &lexeme, start_line, start_col)), None))
    }

    pub fn handle_identifier(c: char, chars: &[char], line: usize, column: usize, i: &mut usize, column_out: &mut usize) -> Option<Token> {
        if !c.is_ascii_alphabetic() && c != '_' {
            return None;
        }
        let start = *i;
        let start_col = column;
        while *i < chars.len() && (chars[*i].is_ascii_alphanumeric() || chars[*i] == '_') {
            *i += 1;
            *column_out += 1;
        }
        let lexeme: String = chars[start..*i].iter().collect();
        let token_type = Tokenizer::keyword_token_type(&lexeme);
        Some(Tokenizer::new_token(token_type, &lexeme, line, start_col))
    }

    pub fn handle_double_operator(chars: &[char], line: usize, column: usize, i: &mut usize, column_out: &mut usize) -> Option<Token> {
        if let Some(pair) = Tokenizer::is_double_operator(chars, *i) {
            *i += 2;
            *column_out += 2;
            Some(Tokenizer::new_token("OP", &pair, line, column))
        } else {
            None
        }
    }

    pub fn handle_arithmetic(c: char, line: usize, column: usize, i: &mut usize, column_out: &mut usize) -> Option<Token> {
        if !Tokenizer::is_arithmetic(c) {
            return None;
        }
        *i += 1;
        *column_out += 1;
        Some(Tokenizer::new_token("ARIT", &c.to_string(), line, column))
    }

    pub fn handle_relational(c: char, chars: &[char], line: usize, column: usize, i: &mut usize, column_out: &mut usize) -> Option<Token> {
        if !Tokenizer::is_relational(c) {
            return None;
        }
        if *i + 1 < chars.len() && chars[*i + 1] == '=' {
            let pair: String = chars[*i..*i + 2].iter().collect();
            *i += 2;
            *column_out += 2;
            return Some(Tokenizer::new_token("REL", &pair, line, column));
        }
        let token_type = if c == '=' { "ASIG" } else { "REL" };
        *i += 1;
        *column_out += 1;
        Some(Tokenizer::new_token(token_type, &c.to_string(), line, column))
    }

    pub fn handle_symbol(c: char, line: usize, column: usize, i: &mut usize, column_out: &mut usize) -> Option<Token> {
        if !Tokenizer::is_symbol(c) {
            return None;
        }
        *i += 1;
        *column_out += 1;
        Some(Tokenizer::new_token("SYM", &c.to_string(), line, column))
    }
}
