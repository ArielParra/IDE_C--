use crate::models::Token;

pub struct Tokenizer;

impl Tokenizer {
    pub fn new_token(token_type: &str, lexeme: &str, line: usize, column: usize) -> Token {
        Token::new(token_type, lexeme, line, column)
    }

    pub fn keyword_token_type(lexeme: &str) -> &'static str {
        match lexeme {
            "if" => "IF",
            "else" => "ELSE",
            "end" => "END",
            "do" => "DO",
            "while" => "WHILE",
            "for" => "FOR",
            "switch" => "SWITCH",
            "case" => "CASE",
            "return" => "RETURN",
            "void" => "VOID",
            "int" => "INT_T",
            "float" => "FLOAT_T",
            "char" => "CHAR_T",
            "bool" => "BOOL_T",
            "true" => "TRUE",
            "false" => "FALSE",
            "main" => "MAIN",
            "cin" => "CIN",
            "cout" => "COUT",
            "include" => "INCLUDE",
            "define" => "DEFINE",
            "struct" => "STRUCT",
            "break" => "BREAK",
            "continue" => "CONTINUE",
            _ => "ID",
        }
    }

    pub fn is_double_operator(chars: &[char], i: usize) -> Option<String> {
        if i + 1 >= chars.len() {
            return None;
        }
        let pair: String = chars[i..i + 2].iter().collect();
        if ["&&", "||"].contains(&pair.as_str()) {
            Some(pair)
        } else {
            None
        }
    }

    pub fn is_arithmetic(c: char) -> bool {
        "*/%^".contains(c)
    }

    pub fn is_relational(c: char) -> bool {
        c == '=' || c == '<' || c == '>' || c == '!'
    }

    pub fn is_symbol(c: char) -> bool {
        "(){};, ".contains(c)
    }
}
