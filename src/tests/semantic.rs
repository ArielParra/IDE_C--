use crate::compiler::lexer::analyzer::analyze;
use crate::compiler::parser::build_ast_from_tokens;
use crate::compiler::semantic::SemanticAnalyzer;

#[test]
fn test_semantic_analysis_accepts_valid_program() {
    let code = "main { int x; x = 10; }";
    let (tokens, lex_errors) = analyze(code);
    assert!(lex_errors.is_empty(), "Expected no lexical errors: {:?}", lex_errors);

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(syn_errors.is_empty(), "Expected no syntax errors: {:?}", syn_errors);

    let errors = SemanticAnalyzer::analyze(&ast);
    assert!(errors.is_empty(), "Expected no semantic errors: {:?}", errors);
}

#[test]
fn test_semantic_analysis_collects_symbol_table_positions_and_appearance_lines() {
    let code = "main {\n int x;\n x = 1;\n if (true) { int y; y = x; }\n x = x + 1;\n}";
    let (tokens, lex_errors) = analyze(code);
    assert!(lex_errors.is_empty(), "Expected no lexical errors: {:?}", lex_errors);

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(syn_errors.is_empty(), "Expected no syntax errors: {:?}", syn_errors);

    let (errors, symbols) = SemanticAnalyzer::analyze_with_symbol_table(&ast);
    assert!(errors.is_empty(), "Expected no semantic errors: {:?}", errors);
    assert_eq!(symbols.len(), 2);

    assert_eq!(symbols[0].name, "x");
    assert_eq!(symbols[0].data_type.as_str(), "int");
    assert_eq!(symbols[0].memory_position, 1);
    assert_eq!(symbols[0].lines, vec![2, 3, 4, 5]);

    assert_eq!(symbols[1].name, "y");
    assert_eq!(symbols[1].data_type.as_str(), "int");
    assert_eq!(symbols[1].memory_position, 2);
    assert_eq!(symbols[1].lines, vec![4]);
}

#[test]
fn test_semantic_analysis_collects_all_identifiers_from_multiple_declarations() {
    let code = "main {\n int x, y, z;\n float a, b, c;\n}";
    let (tokens, lex_errors) = analyze(code);
    assert!(lex_errors.is_empty(), "Expected no lexical errors: {:?}", lex_errors);

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(syn_errors.is_empty(), "Expected no syntax errors: {:?}", syn_errors);

    let (errors, symbols) = SemanticAnalyzer::analyze_with_symbol_table(&ast);
    assert!(errors.is_empty(), "Expected no semantic errors: {:?}", errors);
    assert_eq!(
        symbols
            .iter()
            .map(|symbol| (symbol.name.as_str(), symbol.data_type.as_str(), symbol.line))
            .collect::<Vec<_>>(),
        vec![
            ("x", "int", 2),
            ("y", "int", 2),
            ("z", "int", 2),
            ("a", "float", 3),
            ("b", "float", 3),
            ("c", "float", 3),
        ]
    );
    assert!(symbols.iter().all(|symbol| symbol.lines == vec![symbol.line]));
}

#[test]
fn test_semantic_analysis_tracks_values_at_each_identifier_occurrence() {
    let code = "main {\n int x;\n int y;\n x = 2;\n y = x + 3;\n x = y * 2;\n y = x;\n}";
    let (tokens, lex_errors) = analyze(code);
    assert!(lex_errors.is_empty(), "Expected no lexical errors: {:?}", lex_errors);

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(syn_errors.is_empty(), "Expected no syntax errors: {:?}", syn_errors);

    let (errors, _, values) = SemanticAnalyzer::analyze_with_symbol_values(&ast);
    assert!(errors.is_empty(), "Expected no semantic errors: {:?}", errors);
    assert_eq!(values.get(&(4, 2)).map(String::as_str), Some("2"));
    assert_eq!(values.get(&(5, 2)).map(String::as_str), Some("5"));
    assert_eq!(values.get(&(5, 6)).map(String::as_str), Some("2"));
    assert_eq!(values.get(&(6, 2)).map(String::as_str), Some("10"));
    assert_eq!(values.get(&(6, 6)).map(String::as_str), Some("5"));
    assert_eq!(values.get(&(7, 6)).map(String::as_str), Some("10"));
}

#[test]
fn test_semantic_analysis_detects_undeclared_variable() {
    let code = "main { x = 5; }";
    let (tokens, lex_errors) = analyze(code);
    assert!(lex_errors.is_empty(), "Expected no lexical errors: {:?}", lex_errors);

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(syn_errors.is_empty(), "Expected no syntax errors: {:?}", syn_errors);

    let errors = SemanticAnalyzer::analyze(&ast);
    assert!(!errors.is_empty(), "Expected semantic errors for undeclared variable");
    assert!(errors.iter().any(|e| e.message.contains("undeclared") || e.message.contains("not declared")));
}

#[test]
fn test_semantic_analysis_detects_duplicate_declaration() {
    let code = "main { int x; int x; }";
    let (tokens, lex_errors) = analyze(code);
    assert!(lex_errors.is_empty(), "Expected no lexical errors: {:?}", lex_errors);

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(syn_errors.is_empty(), "Expected no syntax errors: {:?}", syn_errors);

    let errors = SemanticAnalyzer::analyze(&ast);
    assert!(!errors.is_empty(), "Expected duplicate declaration error");
    assert!(errors.iter().any(|e| e.message.contains("already declared") || e.message.contains("duplicate")));
}

#[test]
fn test_semantic_analysis_detects_type_mismatch() {
    let code = "main { int x; x = true; }";
    let (tokens, lex_errors) = analyze(code);
    assert!(lex_errors.is_empty(), "Expected no lexical errors: {:?}", lex_errors);

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(syn_errors.is_empty(), "Expected no syntax errors: {:?}", syn_errors);

    let errors = SemanticAnalyzer::analyze(&ast);
    assert!(!errors.is_empty(), "Expected type mismatch error");
    assert!(errors.iter().any(|e| e.message.contains("type") || e.message.contains("cannot assign")));
}

#[test]
fn test_semantic_analysis_allows_valid_block_scoping() {
    let code = "main { int x; x = 0; if (true) { int y; y = 2; } x = x + 1; }";
    let (tokens, lex_errors) = analyze(code);
    assert!(lex_errors.is_empty(), "Expected no lexical errors: {:?}", lex_errors);

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(syn_errors.is_empty(), "Expected no syntax errors: {:?}", syn_errors);

    let errors = SemanticAnalyzer::analyze(&ast);
    assert!(errors.is_empty(), "Expected no semantic errors for valid scoped program: {:?}", errors);
}

#[test]
fn test_semantic_analysis_catches_invalid_boolean_condition() {
    let code = "main { int x; while (x) { x = x + 1; } }";
    let (tokens, lex_errors) = analyze(code);
    assert!(lex_errors.is_empty(), "Expected no lexical errors: {:?}", lex_errors);

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(syn_errors.is_empty(), "Expected no syntax errors: {:?}", syn_errors);

    let errors = SemanticAnalyzer::analyze(&ast);
    assert!(!errors.is_empty(), "Expected boolean-condition semantic error");
    assert!(errors.iter().any(|e| e.message.contains("boolean") || e.message.contains("Condition")));
}
#[test]
fn test_semantic_analysis_detects_uninitialized_variable_use() {
    let code = "main { int c; c--; }";
    let (tokens, lex_errors) = analyze(code);
    assert!(lex_errors.is_empty(), "Expected no lexical errors: {:?}", lex_errors);

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(syn_errors.is_empty(), "Expected no syntax errors: {:?}", syn_errors);

    let errors = SemanticAnalyzer::analyze(&ast);
    assert!(
        errors.iter().any(|e| e.message.contains("initializ")),
        "Expected uninitialized variable error: {:?}",
        errors
    );
}

#[test]
fn test_semantic_analysis_detects_expected_errors_from_semantic_file() {
    let code = include_str!("../../semantic.c--");
    let (tokens, lex_errors) = analyze(code);
    assert!(lex_errors.is_empty(), "Expected no lexical errors in semantic sample: {:?}", lex_errors);

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(
        !syn_errors.is_empty(),
        "Expected syntax errors in the semantic sample because the file intentionally contains invalid constructs: {:?}",
        syn_errors
    );

    let errors = SemanticAnalyzer::analyze(&ast);
    assert!(!errors.is_empty(), "Expected semantic errors in the sample file");

    let messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
    let joined = messages.join("\n");

    assert!(joined.contains("not declared") || joined.contains("undeclared") || joined.contains("is not declared"), "Missing undeclared variable report: {:?}", messages);
    assert!(joined.contains("Cannot assign") || joined.contains("type") || joined.contains("initializ"), "Missing type or initialization report: {:?}", messages);
}
