pub mod core;
pub mod expressions;
pub mod statements;
pub mod utils;

use crate::models::error::SyntaxError;
use crate::models::Token;

pub use self::core::Parser;
pub use utils::{write_tokens_to_file, read_tokens_from_file};

pub fn build_ast_from_tokens(tokens: &[Token]) -> (AstNode, Vec<SyntaxError>) {
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    (ast, parser.errors)
}

use crate::models::ast::AstNode;
