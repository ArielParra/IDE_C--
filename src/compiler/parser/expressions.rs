use crate::models::ast::AstNode;
use crate::models::error::SyntaxError;

use super::core::Parser;

impl Parser<'_> {
    pub(crate) fn parse_expression(&mut self) -> AstNode {
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
}
