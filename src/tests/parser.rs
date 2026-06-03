use crate::compiler::lexer::analyzer::analyze;
use crate::compiler::parser::build_ast_from_tokens;

#[test]
fn test_valid_program() {
    let code = "main { int x; }";
    let (tokens, lex_errors) = analyze(code);
    assert!(lex_errors.is_empty(), "Expected no lexical errors: {:?}", lex_errors);

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(syn_errors.is_empty(), "Expected no syntax errors: {:?}", syn_errors);

    // AST structure: Program -> MainProgram -> (main, DeclarationList)
    assert_eq!(ast.label, "Program");
    assert_eq!(ast.children.len(), 1);
    
    let main_prog = &ast.children[0];
    assert_eq!(main_prog.label, "MainProgram");
    assert!(main_prog.children.len() >= 2); // 'main' and 'DeclarationList'
    
    assert_eq!(main_prog.children[0].label, "main");
    assert_eq!(main_prog.children[1].label, "DeclarationList");
}

#[test]
fn test_syntax_errors() {
    let code = "main { int x }"; // missing semicolon
    let (tokens, lex_errors) = analyze(code);
    assert!(lex_errors.is_empty(), "Expected no lexical errors");

    let (_, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(!syn_errors.is_empty(), "Expected syntax error due to missing semicolon");
}
