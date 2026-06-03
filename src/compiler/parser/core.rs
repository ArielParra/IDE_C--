use crate::models::ast::AstNode;
use crate::models::error::SyntaxError;
use crate::models::Token;

pub struct Parser<'a> {
    pub(crate) tokens: &'a [Token],
    pub(crate) pos: usize,
    pub errors: Vec<SyntaxError>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> AstNode {
        let mut root = AstNode::new("Program");
        root.add_child(self.parse_program());
        root
    }

    pub(crate) fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    pub(crate) fn previous(&self) -> Option<&Token> {
        if self.pos == 0 {
            None
        } else {
            self.tokens.get(self.pos - 1)
        }
    }

    pub(crate) fn advance(&mut self) -> Option<&Token> {
        if !self.at_end() {
            self.pos += 1;
        }
        self.previous()
    }

    pub(crate) fn at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    pub(crate) fn check_type(&self, token_type: &str) -> bool {
        matches!(self.current(), Some(token) if token.token_type == token_type)
    }

    pub(crate) fn check_lexeme(&self, token_type: &str, lexeme: &str) -> bool {
        matches!(self.current(), Some(token) if token.token_type == token_type && token.lexeme == lexeme)
    }

    pub(crate) fn match_type(&mut self, token_type: &str) -> bool {
        if self.check_type(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub(crate) fn match_lexeme(&mut self, token_type: &str, lexeme: &str) -> bool {
        if self.check_lexeme(token_type, lexeme) {
            self.advance();
            true
        } else {
            false
        }
    }

    #[allow(dead_code)]
    pub(crate) fn consume(&mut self, token_type: &str, message: &str) -> Option<&Token> {
        if self.check_type(token_type) {
            return self.advance();
        }

        let (line, column, found) = if let Some(token) = self.current() {
            (token.line, token.column, token.lexeme.clone())
        } else {
            (0, 0, "EOF".to_string())
        };
        self.errors.push(SyntaxError::expected_token(message, &found, line, column));
        None
    }

    pub(crate) fn synchronize(&mut self) {
        while !self.at_end() {
            if let Some(token) = self.previous() {
                if token.token_type == "SYM" && token.lexeme == ";" {
                    return;
                }
            }

            match self.current().map(|t| t.token_type.as_str()) {
                Some("MAIN") | Some("IF") | Some("WHILE") | Some("DO") | Some("CIN") | Some("COUT") | Some("INT_T") | Some("FLOAT_T") | Some("BOOL_T") | Some("END") | Some("ELSE") => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    pub(crate) fn location_or_zero(&self) -> (usize, usize) {
        if let Some(token) = self.current() {
            (token.line, token.column)
        } else {
            (0, 0)
        }
    }

    pub(crate) fn location_details(&self) -> (usize, usize, String) {
        if let Some(token) = self.current() {
            (token.line, token.column, token.lexeme.clone())
        } else {
            (0, 0, "EOF".to_string())
        }
    }

    pub(crate) fn can_start_statement(&self) -> bool {
        matches!(self.current().map(|t| t.token_type.as_str()),
            Some("IF") | Some("WHILE") | Some("DO") | Some("CIN") | Some("COUT") | Some("ID")
        )
    }

    fn parse_program(&mut self) -> AstNode {
        let (prog_line, prog_col) = self.location_or_zero();
        let mut node = AstNode::new("MainProgram").with_pos(prog_line, prog_col);

        if self.match_type("MAIN") {
            let prev = self.previous().unwrap();
            node.add_child(AstNode::new("main").with_pos(prev.line, prev.column));
        } else {
            let (line, column) = self.location_or_zero();
            self.errors.push(SyntaxError::new("Expected 'main'", line, column));
        }

        if self.match_lexeme("SYM", "{") {
            node.add_child(self.parse_declaration_list());
            if !self.match_lexeme("SYM", "}") {
                let (line, column) = self.location_or_zero();
                self.errors.push(SyntaxError::new("Expected '}' at end of program", line, column));
            }
        } else {
            let (line, column) = self.location_or_zero();
            self.errors.push(SyntaxError::new("Expected '{' after main", line, column));
            self.synchronize();
        }

        node
    }

    fn parse_declaration_list(&mut self) -> AstNode {
        let mut node = AstNode::new("DeclarationList");
        while !self.at_end() && !self.check_lexeme("SYM", "}") {
            if self.check_type("INT_T") || self.check_type("FLOAT_T") || self.check_type("BOOL_T") {
                node.add_child(self.parse_variable_declaration());
            } else if self.can_start_statement() {
                node.add_child(self.parse_statement());
            } else {
                let (line, column, found) = self.location_details();
                self.errors.push(SyntaxError::new(
                    &format!("Unexpected token '{}' in declaration list", found),
                    line,
                    column,
                ));
                self.advance();
            }
        }
        node
    }

    fn parse_variable_declaration(&mut self) -> AstNode {
        let (decl_line, decl_col) = self.location_or_zero();
        let mut node = AstNode::new("VariableDeclaration").with_pos(decl_line, decl_col);
        if let Some(token) = self.advance() {
            node.add_child(AstNode::new(format!("type: {}", token.lexeme)).with_pos(token.line, token.column));
        }

        let mut ids = AstNode::new("Identifiers");
        if self.check_type("ID") {
            let token = self.current().unwrap();
            ids.add_child(AstNode::new(token.lexeme.clone()).with_pos(token.line, token.column));
            self.advance();
        } else {
            let (line, column) = self.location_or_zero();
            self.errors.push(SyntaxError::new("Expected identifier in declaration", line, column));
        }

        while self.match_lexeme("SYM", ",") {
            if self.check_type("ID") {
                let token = self.current().unwrap();
                ids.add_child(AstNode::new(token.lexeme.clone()).with_pos(token.line, token.column));
                self.advance();
            } else {
                let (line, column) = self.location_or_zero();
                self.errors.push(SyntaxError::new("Expected identifier after comma", line, column));
                self.synchronize();
                break;
            }
        }

        node.add_child(ids);
        if !self.match_lexeme("SYM", ";") {
            let (line, column) = self.location_or_zero();
            self.errors.push(SyntaxError::new("Expected ';' after declaration", line, column));
            self.synchronize();
        }

        node
    }
}
