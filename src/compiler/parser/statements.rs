use crate::models::ast::AstNode;
use crate::models::error::SyntaxError;

use super::core::Parser;

impl Parser<'_> {
    pub(crate) fn parse_statement(&mut self) -> AstNode {
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
        if self.match_lexeme("SYM", "{") {
            while !self.at_end() && !self.check_lexeme("SYM", "}") {
                if self.can_start_statement() {
                    body.add_child(self.parse_statement());
                } else {
                    break;
                }
            }

            if !self.match_lexeme("SYM", "}") {
                let (line, column) = self.location_or_zero();
                self.errors.push(SyntaxError::new("Expected '}' after while body", line, column));
                self.synchronize();
            }
        } else {
            while !self.at_end() && !self.check_type("END") {
                if self.can_start_statement() {
                    body.add_child(self.parse_statement());
                } else {
                    break;
                }
            }

            if !self.match_type("END") {
                let (line, column) = self.location_or_zero();
                self.errors.push(SyntaxError::new("Expected 'end' or '{...}' body after while loop", line, column));
                self.synchronize();
            }
        }
        node.add_child(body);
        if self.match_lexeme("SYM", ";") {}
        node
    }

    fn parse_repetition(&mut self) -> AstNode {
        let (line_val, col_val) = self.location_or_zero();
        let mut node = AstNode::new("Repetition").with_pos(line_val, col_val);
        self.advance();
        node.add_child(AstNode::new("do").with_pos(line_val, col_val));

        let mut body = AstNode::new("DoBody");
        if self.match_lexeme("SYM", "{") {
            while !self.at_end() && !self.check_lexeme("SYM", "}") {
                if self.can_start_statement() {
                    body.add_child(self.parse_statement());
                } else {
                    break;
                }
            }

            if !self.match_lexeme("SYM", "}") {
                let (line, column) = self.location_or_zero();
                self.errors.push(SyntaxError::new("Expected '}' after do body", line, column));
                self.synchronize();
            }
        } else {
            while !self.at_end() && !self.check_type("WHILE") && !self.check_type("UNTIL") {
                if self.can_start_statement() {
                    body.add_child(self.parse_statement());
                } else {
                    break;
                }
            }
        }
        node.add_child(body);

        if self.check_type("WHILE") || self.check_type("UNTIL") {
            let token = self.current().unwrap();
            node.add_child(AstNode::new(token.lexeme.clone()).with_pos(token.line, token.column));
            self.advance();
        } else {
            let (line, column) = self.location_or_zero();
            self.errors.push(SyntaxError::new("Expected 'while' or 'until' after do body", line, column));
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
}
