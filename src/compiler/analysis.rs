//! A single editor snapshot feeds every compiler phase.
use super::{
    lexer, parser,
    semantic::{SemanticAnalyzer, SemanticResult},
};
use crate::models::{LexicalError, SyntaxError, Token, ast::AstNode};

pub struct AnalysisResult {
    pub tokens: Vec<Token>,
    pub lexical_errors: Vec<LexicalError>,
    pub syntax_errors: Vec<SyntaxError>,
    pub ast: Option<AstNode>,
    pub semantic: Option<SemanticResult>,
}

pub fn analyze_source(source: &str) -> AnalysisResult {
    let (tokens, lexical_errors) = lexer::analyze(source);
    if !lexical_errors.is_empty() {
        return AnalysisResult {
            tokens,
            lexical_errors,
            syntax_errors: Vec::new(),
            ast: None,
            semantic: None,
        };
    }
    let (ast, syntax_errors) = parser::build_ast_from_tokens(&tokens);
    let semantic = SemanticAnalyzer::analyze_with_details(&ast);
    AnalysisResult {
        tokens,
        lexical_errors,
        syntax_errors,
        ast: Some(ast),
        semantic: Some(semantic),
    }
}
