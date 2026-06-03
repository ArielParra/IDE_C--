pub mod core;
pub mod expressions;
pub mod statements;
pub mod utils;

use crate::models::ast::AstNode;
use crate::models::error::SyntaxError;
use crate::models::Token;

pub use self::core::Parser;
#[allow(unused_imports)]
pub use utils::parse_tokens_from_text;

#[allow(dead_code)]
pub fn build_ast(source: &str) -> AstNode {
    let (tokens, _) = crate::compiler::lexer::analyze(source);
    let mut parser = Parser::new(&tokens);
    parser.parse()
}

pub fn build_ast_with_errors(source: &str) -> (AstNode, Vec<SyntaxError>) {
    let (tokens, _) = crate::compiler::lexer::analyze(source);
    let mut parser = Parser::new(&tokens);
    let ast = parser.parse();
    (ast, parser.errors)
}

#[allow(dead_code)]
pub fn build_ast_from_tokens(tokens: &[Token]) -> (AstNode, Vec<SyntaxError>) {
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    (ast, parser.errors)
}
