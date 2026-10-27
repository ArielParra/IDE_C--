use crate::compiler::lexer::analyzer::analyze;
use crate::compiler::parser::build_ast_from_tokens;
use crate::compiler::semantic::SemanticAnalyzer;

#[test]
fn test_semantic_analysis_accepts_valid_program() {
    let code = "main { int x; x = 10; }";
    let (tokens, lex_errors) = analyze(code);
    assert!(
        lex_errors.is_empty(),
        "Expected no lexical errors: {:?}",
        lex_errors
    );

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(
        syn_errors.is_empty(),
        "Expected no syntax errors: {:?}",
        syn_errors
    );

    let errors = SemanticAnalyzer::analyze(&ast);
    assert!(
        errors.is_empty(),
        "Expected no semantic errors: {:?}",
        errors
    );
}

#[test]
fn test_semantic_analysis_detects_undeclared_variable() {
    let code = "main { x = 5; }";
    let (tokens, lex_errors) = analyze(code);
    assert!(
        lex_errors.is_empty(),
        "Expected no lexical errors: {:?}",
        lex_errors
    );

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(
        syn_errors.is_empty(),
        "Expected no syntax errors: {:?}",
        syn_errors
    );

    let errors = SemanticAnalyzer::analyze(&ast);
    assert!(
        !errors.is_empty(),
        "Expected semantic errors for undeclared variable"
    );
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("undeclared") || e.message.contains("not declared"))
    );
}

#[test]
fn test_semantic_analysis_detects_duplicate_declaration() {
    let code = "main { int x; int x; }";
    let (tokens, lex_errors) = analyze(code);
    assert!(
        lex_errors.is_empty(),
        "Expected no lexical errors: {:?}",
        lex_errors
    );

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(
        syn_errors.is_empty(),
        "Expected no syntax errors: {:?}",
        syn_errors
    );

    let errors = SemanticAnalyzer::analyze(&ast);
    assert!(!errors.is_empty(), "Expected duplicate declaration error");
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("already declared") || e.message.contains("duplicate"))
    );
}

#[test]
fn test_semantic_analysis_detects_type_mismatch() {
    let code = "main { int x; x = true; }";
    let (tokens, lex_errors) = analyze(code);
    assert!(
        lex_errors.is_empty(),
        "Expected no lexical errors: {:?}",
        lex_errors
    );

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(
        syn_errors.is_empty(),
        "Expected no syntax errors: {:?}",
        syn_errors
    );

    let errors = SemanticAnalyzer::analyze(&ast);
    assert!(!errors.is_empty(), "Expected type mismatch error");
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("type") || e.message.contains("cannot assign"))
    );
}

#[test]
fn test_semantic_analysis_allows_valid_block_scoping() {
    let code = "main { int x; x = 0; if (true) { int y; y = 2; } x = x + 1; }";
    let (tokens, lex_errors) = analyze(code);
    assert!(
        lex_errors.is_empty(),
        "Expected no lexical errors: {:?}",
        lex_errors
    );

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(
        syn_errors.is_empty(),
        "Expected no syntax errors: {:?}",
        syn_errors
    );

    let errors = SemanticAnalyzer::analyze(&ast);
    assert!(
        errors.is_empty(),
        "Expected no semantic errors for valid scoped program: {:?}",
        errors
    );
}

#[test]
fn test_semantic_analysis_catches_invalid_boolean_condition() {
    let code = "main { int x; while (x) { x = x + 1; } }";
    let (tokens, lex_errors) = analyze(code);
    assert!(
        lex_errors.is_empty(),
        "Expected no lexical errors: {:?}",
        lex_errors
    );

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(
        syn_errors.is_empty(),
        "Expected no syntax errors: {:?}",
        syn_errors
    );

    let errors = SemanticAnalyzer::analyze(&ast);
    assert!(
        !errors.is_empty(),
        "Expected boolean-condition semantic error"
    );
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("boolean") || e.message.contains("Condition"))
    );
}
#[test]
fn test_semantic_analysis_detects_uninitialized_variable_use() {
    let code = "main { int c; c--; }";
    let (tokens, lex_errors) = analyze(code);
    assert!(
        lex_errors.is_empty(),
        "Expected no lexical errors: {:?}",
        lex_errors
    );

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(
        syn_errors.is_empty(),
        "Expected no syntax errors: {:?}",
        syn_errors
    );

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
    assert!(
        lex_errors.is_empty(),
        "Expected no lexical errors in semantic sample: {:?}",
        lex_errors
    );

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(
        !syn_errors.is_empty(),
        "Expected syntax errors in the semantic sample because the file intentionally contains invalid constructs: {:?}",
        syn_errors
    );

    let errors = SemanticAnalyzer::analyze(&ast);
    assert!(
        !errors.is_empty(),
        "Expected semantic errors in the sample file"
    );

    let messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
    let joined = messages.join("\n");

    assert!(
        joined.contains("not declared")
            || joined.contains("undeclared")
            || joined.contains("is not declared"),
        "Missing undeclared variable report: {:?}",
        messages
    );
    assert!(
        joined.contains("Cannot assign") || joined.contains("type") || joined.contains("initializ"),
        "Missing type or initialization report: {:?}",
        messages
    );
}

fn parse_semantic_source(source: &str) -> crate::models::ast::AstNode {
    let (tokens, errors) = analyze(source);
    assert!(errors.is_empty(), "{errors:?}");
    let (ast, errors) = build_ast_from_tokens(&tokens);
    assert!(errors.is_empty(), "{errors:?}");
    ast
}

fn find_node<'a>(
    node: &'a crate::models::ast::AstNode,
    label: &str,
) -> &'a crate::models::ast::AstNode {
    if node.label == label {
        return node;
    }
    fn search<'a>(
        node: &'a crate::models::ast::AstNode,
        label: &str,
    ) -> Option<&'a crate::models::ast::AstNode> {
        if node.label == label {
            return Some(node);
        }
        node.children.iter().find_map(|child| search(child, label))
    }
    search(node, label).unwrap_or_else(|| panic!("Missing node {label}"))
}

#[test]
fn semantic_report_preserves_ast_and_propagates_attributes() {
    let ast = parse_semantic_source("main { float x; x = 2 + 3 * 4; }");
    let original = format!("{ast:?}");
    let report = SemanticAnalyzer::analyze_with_details(&ast);
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    assert_eq!(format!("{ast:?}"), original);
    let sum = find_node(&report.annotated_ast, "AddOp(+)");
    assert_eq!(sum.annotated_type.as_deref(), Some("int"));
    assert_eq!(sum.attributed_value.as_deref(), Some("14"));
    assert_eq!(sum.coerced_type.as_deref(), Some("float"));
    assert_eq!(
        find_node(&report.annotated_ast, "id: x")
            .annotated_type
            .as_deref(),
        Some("float")
    );
    assert_eq!(report.symbols[0].offset, 0);
    assert_eq!(report.symbols[0].name, "x");
}

#[test]
fn semantic_report_retains_closed_scopes_and_resolves_shadowing() {
    let ast = parse_semantic_source(
        "main { int x; x = 1; if (true) { float x; x = 2.5; cout << x; } cout << x; }",
    );
    let report = SemanticAnalyzer::analyze_with_details(&ast);
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    assert_eq!(report.symbols.len(), 2);
    assert_ne!(report.symbols[0].scope_id, report.symbols[1].scope_id);
    assert_eq!(report.symbols[1].offset, 1);
    assert_eq!(report.symbols[0].references.len(), 2);
    assert_eq!(report.symbols[1].references.len(), 2);
}

#[test]
fn semantic_input_initializes_but_invalid_assignment_does_not() {
    let ast = parse_semantic_source("main { int x; cin >> x; cout << x; }");
    assert!(SemanticAnalyzer::analyze(&ast).is_empty());
    let ast = parse_semantic_source("main { int x; x = true; cout << x; }");
    let errors = SemanticAnalyzer::analyze(&ast);
    assert!(errors.iter().any(|e| e.message.contains("Cannot assign")));
    assert!(errors.iter().any(|e| e.message.contains("initializ")));
}

#[test]
fn semantic_initialization_is_definite_across_control_flow() {
    for source in [
        "main { int x; if (true) { x = 1; } cout << x; }",
        "main { int x; while (false) { x = 1; } cout << x; }",
        "main { int x; if (true) { x = 1; } else { cout << x; } }",
    ] {
        let errors = SemanticAnalyzer::analyze(&parse_semantic_source(source));
        assert!(
            errors.iter().any(|e| e.message.contains("initializ")),
            "{source}: {errors:?}"
        );
    }
    for source in [
        "main { int x; if (true) { x = 1; } else { x = 2; } cout << x; }",
        "main { int x; do { x = 1; } until (true); cout << x; }",
    ] {
        let errors = SemanticAnalyzer::analyze(&parse_semantic_source(source));
        assert!(errors.is_empty(), "{source}: {errors:?}");
    }
}

#[test]
fn semantic_constant_evaluation_checks_failures_and_runtime_values() {
    let ast = parse_semantic_source("main { int x, y; x = 1; y = x + 2; }");
    let report = SemanticAnalyzer::analyze_with_details(&ast);
    assert!(report.errors.is_empty());
    assert!(
        find_node(&report.annotated_ast, "AddOp(+)")
            .attributed_value
            .is_none()
    );
    for source in [
        "main { int x; x = 1 / 0; }",
        "main { int x; x = 9223372036854775807 + 1; }",
        "main { int x; x = 2 ^ 100; }",
        "main { bool x; x = 1 < true; }",
        "main { int x; x = 2++; }",
    ] {
        let report = SemanticAnalyzer::analyze_with_details(&parse_semantic_source(source));
        assert!(!report.errors.is_empty(), "{source}");
    }
    let ast = parse_semantic_source("main { bool x; x = (2 < 3) && !false; }");
    let report = SemanticAnalyzer::analyze_with_details(&ast);
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    assert_eq!(
        find_node(&report.annotated_ast, "LogicalAnd")
            .attributed_value
            .as_deref(),
        Some("true")
    );
}
