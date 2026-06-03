use crate::compiler::lexer;
use crate::models::ast::AstNode;
use crate::models::error::SyntaxError;
use crate::models::Token;

pub struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
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

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn previous(&self) -> Option<&Token> {
        if self.pos == 0 {
            None
        } else {
            self.tokens.get(self.pos - 1)
        }
    }

    fn advance(&mut self) -> Option<&Token> {
        if !self.at_end() {
            self.pos += 1;
        }
        self.previous()
    }

    fn at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn check_type(&self, token_type: &str) -> bool {
        matches!(self.current(), Some(token) if token.token_type == token_type)
    }

    fn check_lexeme(&self, token_type: &str, lexeme: &str) -> bool {
        matches!(self.current(), Some(token) if token.token_type == token_type && token.lexeme == lexeme)
    }

    fn match_type(&mut self, token_type: &str) -> bool {
        if self.check_type(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    #[allow(dead_code)]
    fn consume(&mut self, token_type: &str, message: &str) -> Option<&Token> {
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

    fn synchronize(&mut self) {
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

    #[allow(dead_code)]
    fn parse_statement_list(&mut self) -> AstNode {
        let mut node = AstNode::new("StatementList");
        while !self.at_end()
            && !self.check_lexeme("SYM", "}")
            && !self.check_type("END")
            && !self.check_type("ELSE")
            && !self.check_type("WHILE")
            && !self.check_type("DO")
            && !self.check_type("THEN")
        {
            if self.can_start_statement() {
                node.add_child(self.parse_statement());
            } else {
                break;
            }
        }
        node
    }

    fn can_start_statement(&self) -> bool {
        matches!(self.current().map(|t| t.token_type.as_str()),
            Some("IF") | Some("WHILE") | Some("DO") | Some("CIN") | Some("COUT") | Some("ID")
        )
    }

    fn parse_statement(&mut self) -> AstNode {
        let (line, column) = if let Some(token) = self.current() {
            (Some(token.line), Some(token.column))
        } else {
            (None, None)
        };
        match self.current().map(|t| t.token_type.as_str()) {
            Some("IF") => self.parse_selection().with_opt_pos(line, column),
            Some("WHILE") => self.parse_iteration().with_opt_pos(line, column),
            Some("DO") => self.parse_repetition().with_opt_pos(line, column),
            Some("CIN") => self.parse_input().with_opt_pos(line, column),
            Some("COUT") => self.parse_output().with_opt_pos(line, column),
            Some("ID") => self.parse_id_statement().with_opt_pos(line, column),
            Some(_token_type) => {
                let (line_val, column_val, found) = self.location_details();
                self.errors.push(SyntaxError::new(
                    &format!("Unexpected token '{}' in statement", found),
                    line_val,
                    column_val,
                ));
                self.advance();
                AstNode::new("UnknownStatement").with_pos(line_val, column_val)
            }
            None => AstNode::new("EmptyStatement"),
        }
    }

    fn parse_id_statement(&mut self) -> AstNode {
        let id_name = self.current().unwrap().lexeme.clone();
        let (id_line, id_col) = (self.current().unwrap().line, self.current().unwrap().column);
        self.advance();

        // Check for postfix operators (++ or --)
        if self.check_lexeme("OP", "++") || self.check_lexeme("OP", "--") {
            let op = self.current().unwrap().lexeme.clone();
            let (op_line, op_col) = (self.current().unwrap().line, self.current().unwrap().column);
            self.advance();
            let mut node = AstNode::new("ExpressionStatement").with_pos(id_line, id_col);
            let mut expr_node = AstNode::new(format!("Postfix({})", op)).with_pos(op_line, op_col);
            expr_node.add_child(AstNode::new(format!("id: {}", id_name)).with_pos(id_line, id_col));
            node.add_child(expr_node);
            if !self.match_lexeme("SYM", ";") {
                let (line, column) = self.location_or_zero();
                self.errors.push(SyntaxError::new("Expected ';' after expression", line, column));
            }
            return node;
        }

        // Otherwise it's an assignment
        let mut node = AstNode::new("Assignment").with_pos(id_line, id_col);
        node.add_child(AstNode::new(format!("id: {}", id_name)).with_pos(id_line, id_col));

        if !self.match_type("ASIG") {
            let (line, column) = self.location_or_zero();
            self.errors.push(SyntaxError::new("Expected '=' in assignment", line, column));
        }
        node.add_child(self.parse_sent_expression());
        node
    }

    fn parse_sent_expression(&mut self) -> AstNode {
        let mut node = AstNode::new("ExpressionStatement");
        if self.match_lexeme("SYM", ";") {
            node.add_child(AstNode::new("empty"));
            return node;
        }

        node.add_child(self.parse_expression());
        if !self.match_lexeme("SYM", ";") {
            let (line, column) = self.location_or_zero();
            self.errors.push(SyntaxError::new("Expected ';' after expression", line, column));
            self.synchronize();
        }
        node
    }

    fn parse_selection(&mut self) -> AstNode {
        let (line_val, col_val) = self.location_or_zero();
        let mut node = AstNode::new("Selection").with_pos(line_val, col_val);
        self.advance();
        node.add_child(AstNode::new("if").with_pos(line_val, col_val));
        node.add_child(self.parse_expression());

        if !self.match_type("THEN") {
            let (line, column) = self.location_or_zero();
            self.errors.push(SyntaxError::new("Expected 'then' after condition", line, column));
        }

        let mut then_block = AstNode::new("ThenBlock");
        while !self.at_end() && !self.check_type("ELSE") && !self.check_type("END") {
            if self.can_start_statement() {
                then_block.add_child(self.parse_statement());
            } else {
                break;
            }
        }
        node.add_child(then_block);

        if self.match_type("ELSE") {
            let mut else_block = AstNode::new("ElseBlock");
            while !self.at_end() && !self.check_type("END") {
                if self.can_start_statement() {
                    else_block.add_child(self.parse_statement());
                } else {
                    break;
                }
            }
            node.add_child(else_block);
        }

        if !self.match_type("END") {
            let (line, column) = self.location_or_zero();
            self.errors.push(SyntaxError::new("Expected 'end' after selection", line, column));
            self.synchronize();
        }
        if self.match_lexeme("SYM", ";") {}
        node
    }

    fn parse_iteration(&mut self) -> AstNode {
        let (line_val, col_val) = self.location_or_zero();
        let mut node = AstNode::new("Iteration").with_pos(line_val, col_val);
        self.advance();
        node.add_child(AstNode::new("while").with_pos(line_val, col_val));
        node.add_child(self.parse_expression());

        let mut body = AstNode::new("WhileBody");
        while !self.at_end() && !self.check_type("END") {
            if self.can_start_statement() {
                body.add_child(self.parse_statement());
            } else {
                break;
            }
        }

        node.add_child(body);
        if !self.match_type("END") {
            let (line, column) = self.location_or_zero();
            self.errors.push(SyntaxError::new("Expected 'end' after while loop", line, column));
            self.synchronize();
        }
        if self.match_lexeme("SYM", ";") {}
        node
    }

    fn parse_repetition(&mut self) -> AstNode {
        let (line_val, col_val) = self.location_or_zero();
        let mut node = AstNode::new("Repetition").with_pos(line_val, col_val);
        self.advance();
        let mut body = AstNode::new("DoBody");
        while !self.at_end() && !self.check_type("WHILE") {
            if self.can_start_statement() {
                body.add_child(self.parse_statement());
            } else {
                break;
            }
        }
        node.add_child(body);

        if !self.match_type("WHILE") {
            let (line, column) = self.location_or_zero();
            self.errors.push(SyntaxError::new("Expected 'while' after do body", line, column));
        }
        node.add_child(self.parse_expression());
        
        if self.match_lexeme("SYM", ";") {}
        node
    }

    fn parse_input(&mut self) -> AstNode {
        let (line_val, col_val) = self.location_or_zero();
        let mut node = AstNode::new("Input").with_pos(line_val, col_val);
        self.advance();
        if !self.match_lexeme("OP", ">>") {
            let (line, column) = self.location_or_zero();
            self.errors.push(SyntaxError::new("Expected '>>' after cin", line, column));
        }

        if let Some(token) = self.current() {
            if token.token_type == "ID" {
                node.add_child(AstNode::new(format!("id: {}", token.lexeme)).with_pos(token.line, token.column));
                self.advance();
            } else {
                let (line, column) = self.location_or_zero();
                self.errors.push(SyntaxError::new("Expected identifier after cin >>", line, column));
            }
        }

        if !self.match_lexeme("SYM", ";") {
            let (line, column) = self.location_or_zero();
            self.errors.push(SyntaxError::new("Expected ';' after input", line, column));
        }

        node
    }

    fn parse_output(&mut self) -> AstNode {
        let (line_val, col_val) = self.location_or_zero();
        let mut node = AstNode::new("Output").with_pos(line_val, col_val);
        self.advance();

        if !self.match_lexeme("OP", "<<") {
            let (line, column) = self.location_or_zero();
            self.errors.push(SyntaxError::new("Expected '<<' after cout", line, column));
        }

        let mut output_chain = AstNode::new("OutputChain");
        output_chain.add_child(self.parse_output_operand());

        while self.match_lexeme("OP", "<<") {
            output_chain.add_child(self.parse_output_operand());
        }

        node.add_child(output_chain);
        if self.match_lexeme("SYM", ";") {}
        node
    }

    fn parse_output_operand(&mut self) -> AstNode {
        if self.check_type("STRING") {
            let token = self.current().unwrap();
            let lexeme = token.lexeme.clone();
            let (t_line, t_col) = (token.line, token.column);
            self.advance();
            AstNode::new(format!("string: {}", lexeme)).with_pos(t_line, t_col)
        } else {
            self.parse_expression()
        }
    }

    fn parse_expression(&mut self) -> AstNode {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> AstNode {
        let mut node = self.parse_logical_and();
        while self.check_lexeme("OP", "||") {
            let token = self.current().unwrap();
            let mut op_node = AstNode::new("LogicalOr").with_pos(token.line, token.column);
            self.advance();
            op_node.add_child(node);
            op_node.add_child(self.parse_logical_and());
            node = op_node;
        }
        node
    }

    fn parse_logical_and(&mut self) -> AstNode {
        let mut node = self.parse_relational();
        while self.check_lexeme("OP", "&&") {
            let token = self.current().unwrap();
            let mut op_node = AstNode::new("LogicalAnd").with_pos(token.line, token.column);
            self.advance();
            op_node.add_child(node);
            op_node.add_child(self.parse_relational());
            node = op_node;
        }
        node
    }

    fn parse_relational(&mut self) -> AstNode {
        let mut node = self.parse_simple_expression();
        if self.check_type("REL") {
            let token = self.current().unwrap();
            let op = token.lexeme.clone();
            let mut op_node = AstNode::new(format!("Relational({})", op)).with_pos(token.line, token.column);
            self.advance();
            op_node.add_child(node);
            op_node.add_child(self.parse_simple_expression());
            node = op_node;
        }
        node
    }

    fn parse_simple_expression(&mut self) -> AstNode {
        let mut node = self.parse_term();
        while let Some(token) = self.current() {
            if token.token_type == "ARIT" && (token.lexeme == "+" || token.lexeme == "-") {
                let op = token.lexeme.clone();
                let mut op_node = AstNode::new(format!("AddOp({})", op)).with_pos(token.line, token.column);
                self.advance();
                op_node.add_child(node);
                op_node.add_child(self.parse_term());
                node = op_node;
            } else if token.token_type == "OP" && (token.lexeme == "++" || token.lexeme == "--") {
                let op = token.lexeme.clone();
                let mut op_node = AstNode::new(format!("UnaryOp({})", op)).with_pos(token.line, token.column);
                self.advance();
                op_node.add_child(node);
                node = op_node;
            } else {
                break;
            }
        }
        node
    }

    fn parse_term(&mut self) -> AstNode {
        let mut node = self.parse_factor();
        while let Some(token) = self.current() {
            if token.token_type == "ARIT" && (token.lexeme == "*" || token.lexeme == "/" || token.lexeme == "%") {
                let op = token.lexeme.clone();
                let mut op_node = AstNode::new(format!("MulOp({})", op)).with_pos(token.line, token.column);
                self.advance();
                op_node.add_child(node);
                op_node.add_child(self.parse_factor());
                node = op_node;
            } else {
                break;
            }
        }
        node
    }

    fn parse_factor(&mut self) -> AstNode {
        let mut node = self.parse_power();
        while self.check_type("ARIT") && self.current().unwrap().lexeme == "^" {
            let token = self.current().unwrap();
            let mut op_node = AstNode::new("Power").with_pos(token.line, token.column);
            self.advance();
            op_node.add_child(node);
            op_node.add_child(self.parse_power());
            node = op_node;
        }
        node
    }

    fn parse_power(&mut self) -> AstNode {
        if self.check_lexeme("OP", "++") || self.check_lexeme("OP", "--") {
            let token = self.current().unwrap();
            let op = token.lexeme.clone();
            let mut node = AstNode::new(format!("Prefix({})", op)).with_pos(token.line, token.column);
            self.advance();
            node.add_child(self.parse_power());
            return node;
        }

        if self.check_lexeme("REL", "!") {
            let token = self.current().unwrap();
            let mut node = AstNode::new("Not").with_pos(token.line, token.column);
            self.advance();
            node.add_child(self.parse_power());
            return node;
        }

        self.parse_component()
    }

    fn parse_component(&mut self) -> AstNode {
        if self.match_lexeme("SYM", "(") {
            let node = self.parse_expression();
            if !self.match_lexeme("SYM", ")") {
                let (line, column) = self.location_or_zero();
                self.errors.push(SyntaxError::new("Expected ')'", line, column));
            }
            return node;
        }

        if let Some(token) = self.current() {
            let node = match token.token_type.as_str() {
                "INT" | "FLOAT" => {
                    let value = token.lexeme.clone();
                    let (t_line, t_col) = (token.line, token.column);
                    self.advance();
                    AstNode::new(format!("number: {}", value)).with_pos(t_line, t_col)
                }
                "TRUE" | "FALSE" => {
                    let value = token.lexeme.clone();
                    let (t_line, t_col) = (token.line, token.column);
                    self.advance();
                    AstNode::new(format!("bool: {}", value)).with_pos(t_line, t_col)
                }
                "ID" => {
                    let value = token.lexeme.clone();
                    let (t_line, t_col) = (token.line, token.column);
                    self.advance();
                    AstNode::new(format!("id: {}", value)).with_pos(t_line, t_col)
                }
                "STRING" => {
                    let value = token.lexeme.clone();
                    let (t_line, t_col) = (token.line, token.column);
                    self.advance();
                    AstNode::new(format!("string: {}", value)).with_pos(t_line, t_col)
                }
                _ => {
                    let (line, column, found) = self.location_details();
                    self.errors.push(SyntaxError::new(
                        &format!("Unexpected component '{}'", found),
                        line,
                        column,
                    ));
                    self.advance();
                    AstNode::new("ErrorComponent").with_pos(line, column)
                }
            };

            if self.check_lexeme("OP", "++") || self.check_lexeme("OP", "--") {
                let op_token = self.current().unwrap();
                let op = op_token.lexeme.clone();
                let mut op_node = AstNode::new(format!("Postfix({})", op)).with_pos(op_token.line, op_token.column);
                self.advance();
                op_node.add_child(node);
                return op_node;
            }

            return node;
        }

        let (line, column) = self.location_or_zero();
        self.errors
            .push(SyntaxError::new("Unexpected end of expression", line, column));
        AstNode::new("Empty").with_pos(line, column)
    }

    fn match_lexeme(&mut self, token_type: &str, lexeme: &str) -> bool {
        if self.check_lexeme(token_type, lexeme) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn location_or_zero(&self) -> (usize, usize) {
        if let Some(token) = self.current() {
            (token.line, token.column)
        } else {
            (0, 0)
        }
    }

    fn location_details(&self) -> (usize, usize, String) {
        if let Some(token) = self.current() {
            (token.line, token.column, token.lexeme.clone())
        } else {
            (0, 0, "EOF".to_string())
        }
    }
}

#[allow(dead_code)]
pub fn build_ast(source: &str) -> AstNode {
    let (tokens, _) = lexer::analyze(source);
    let mut parser = Parser::new(&tokens);
    parser.parse()
}

pub fn build_ast_with_errors(source: &str) -> (AstNode, Vec<SyntaxError>) {
    let (tokens, _) = lexer::analyze(source);
    let mut parser = Parser::new(&tokens);
    let ast = parser.parse();
    (ast, parser.errors)
}

#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn build_ast_from_tokens(tokens: &[Token]) -> (AstNode, Vec<SyntaxError>) {
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    (ast, parser.errors)
}
