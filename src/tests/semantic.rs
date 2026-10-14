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
fn test_semantic_analysis_tracks_values_on_arithmetic_parent_nodes() {
    let code = "main { int x; x = 2 + 3 * 4; }";
    let (tokens, lex_errors) = analyze(code);
    assert!(lex_errors.is_empty(), "Expected no lexical errors: {:?}", lex_errors);

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(syn_errors.is_empty(), "Expected no syntax errors: {:?}", syn_errors);

    let (errors, _, values) = SemanticAnalyzer::analyze_with_symbol_values(&ast);
    assert!(errors.is_empty(), "Expected no semantic errors: {:?}", errors);

    fn find_node<'a>(
        node: &'a crate::models::ast::AstNode,
        label: &str,
    ) -> Option<&'a crate::models::ast::AstNode> {
        if node.label == label {
            return Some(node);
        }
        node.children
            .iter()
            .find_map(|child| find_node(child, label))
    }

    for (label, expected) in [("MulOp(*)", "12"), ("AddOp(+)", "14")] {
        let node = find_node(&ast, label).expect("expected arithmetic AST node");
        let location = (node.line.unwrap(), node.column.unwrap());
        assert_eq!(values.get(&location).map(String::as_str), Some(expected));
    }
}

#[test]
fn test_mixed_numeric_operations_coerce_results_to_float() {
    let code = "main { int x; float y; float z; x = 1; y = 2; z = x + 2.0; y = x + y; }";
    let (tokens, lex_errors) = analyze(code);
    assert!(lex_errors.is_empty(), "Expected no lexical errors: {:?}", lex_errors);

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(syn_errors.is_empty(), "Expected no syntax errors: {:?}", syn_errors);

    let (errors, _, values) = SemanticAnalyzer::analyze_with_symbol_values(&ast);
    assert!(errors.is_empty(), "Expected no semantic errors: {:?}", errors);

    fn find_nodes<'a>(
        node: &'a crate::models::ast::AstNode,
        label: &str,
        matches: &mut Vec<&'a crate::models::ast::AstNode>,
    ) {
        if node.label == label {
            matches.push(node);
        }
        for child in &node.children {
            find_nodes(child, label, matches);
        }
    }

    let mut additions = Vec::new();
    find_nodes(&ast, "AddOp(+)", &mut additions);
    assert_eq!(additions.len(), 2);
    for addition in additions {
        let location = (addition.line.unwrap(), addition.column.unwrap());
        assert_eq!(values.get(&location).map(String::as_str), Some("3.0"));
    }
}

#[test]
fn test_multiplication_and_division_take_the_types_of_their_operands() {
    let code = "main { int x; float y; float product; float quotient; x = 6; y = 2; product = x * y; quotient = x / y; }";
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

    let (errors, symbols, values) = SemanticAnalyzer::analyze_with_symbol_values(&ast);
    assert!(
        errors.is_empty(),
        "Expected no semantic errors: {:?}",
        errors
    );

    for (name, expected) in [("product", "12.0"), ("quotient", "3.0")] {
        let symbol = symbols
            .iter()
            .find(|symbol| symbol.name == name)
            .expect("expected result variable");
        assert_eq!(
            values
                .get(&(symbol.line, symbol.column))
                .map(String::as_str),
            Some(expected),
            "{name} should preserve the float type from its operands"
        );
    }
}

#[test]
fn test_division_of_integers_produces_a_float_result() {
    let code = "main { int numerator; int denominator; float quotient; numerator = 5; denominator = 2; quotient = numerator / denominator; }";
    let (tokens, lex_errors) = analyze(code);
    assert!(lex_errors.is_empty(), "Expected no lexical errors: {:?}", lex_errors);

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(syn_errors.is_empty(), "Expected no syntax errors: {:?}", syn_errors);

    let (errors, symbols, values) = SemanticAnalyzer::analyze_with_symbol_values(&ast);
    assert!(errors.is_empty(), "Expected no semantic errors: {:?}", errors);

    fn find_node<'a>(
        node: &'a crate::models::ast::AstNode,
        label: &str,
    ) -> Option<&'a crate::models::ast::AstNode> {
        if node.label == label {
            return Some(node);
        }
        node.children
            .iter()
            .find_map(|child| find_node(child, label))
    }

    let division = find_node(&ast, "MulOp(/)").expect("expected division AST node");
    let division_location = (division.line.unwrap(), division.column.unwrap());
    assert_eq!(
        values.get(&division_location).map(String::as_str),
        Some("2.5"),
        "the division node should retain its fractional result"
    );

    let quotient = symbols
        .iter()
        .find(|symbol| symbol.name == "quotient")
        .expect("expected quotient variable");
    assert_eq!(
        values
            .get(&(quotient.line, quotient.column))
            .map(String::as_str),
        Some("2.5")
    );
}

#[test]
fn test_semantic_analysis_keeps_distinct_values_for_same_line_declarations() {
    let code = "main { int x, y; x = 2; y = 5; }";
    let (tokens, lex_errors) = analyze(code);
    assert!(lex_errors.is_empty(), "Expected no lexical errors: {:?}", lex_errors);

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(syn_errors.is_empty(), "Expected no syntax errors: {:?}", syn_errors);

    let (errors, symbols, values) = SemanticAnalyzer::analyze_with_symbol_values(&ast);
    assert!(errors.is_empty(), "Expected no semantic errors: {:?}", errors);

    let x = symbols.iter().find(|symbol| symbol.name == "x").unwrap();
    let y = symbols.iter().find(|symbol| symbol.name == "y").unwrap();
    assert_eq!(x.line, y.line);
    assert_ne!(x.column, y.column);
    assert_eq!(values.get(&(x.line, x.column)).map(String::as_str), Some("2"));
    assert_eq!(values.get(&(y.line, y.column)).map(String::as_str), Some("5"));
    assert!(x.appearances.iter().any(|appearance| {
        appearance.value.as_deref() == Some("2") && appearance.line == 1
    }));
    assert!(y.appearances.iter().any(|appearance| {
        appearance.value.as_deref() == Some("5") && appearance.line == 1
    }));
}

#[test]
fn test_semantic_analysis_preserves_each_same_line_value_for_repeated_variable() {
    let code = "main { int x; x = 2; x = 5; }";
    let (tokens, lex_errors) = analyze(code);
    assert!(lex_errors.is_empty(), "Expected no lexical errors: {:?}", lex_errors);

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(syn_errors.is_empty(), "Expected no syntax errors: {:?}", syn_errors);

    let (errors, symbols, _) = SemanticAnalyzer::analyze_with_symbol_values(&ast);
    assert!(errors.is_empty(), "Expected no semantic errors: {:?}", errors);
    let x = symbols.iter().find(|symbol| symbol.name == "x").unwrap();
    let same_line_values = x
        .appearances
        .iter()
        .filter(|appearance| appearance.line == 1)
        .filter_map(|appearance| appearance.value.as_deref())
        .collect::<Vec<_>>();
    assert!(same_line_values.contains(&"2"));
    assert!(same_line_values.contains(&"5"));
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

    let (errors, symbols) = SemanticAnalyzer::analyze_with_symbol_table(&ast);
    assert!(!errors.is_empty(), "Expected duplicate declaration error");
    assert!(errors.iter().any(|e| e.message.contains("already declared") || e.message.contains("duplicate")));
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].appearances.len(), 2);
    assert_eq!(
        crate::ui::panels::AstView::symbol_line_summary(&symbols[0]),
        "1, 1"
    );
}

#[test]
fn test_semantic_analysis_detects_type_mismatch() {
    let code = "main { int x; x = true; }";
    let (tokens, lex_errors) = analyze(code);
    assert!(lex_errors.is_empty(), "Expected no lexical errors: {:?}", lex_errors);

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(syn_errors.is_empty(), "Expected no syntax errors: {:?}", syn_errors);

    let (errors, symbols, values) = SemanticAnalyzer::analyze_with_symbol_values(&ast);
    assert!(!errors.is_empty(), "Expected type mismatch error");
    assert!(errors.iter().any(|e| e.message.contains("type") || e.message.contains("cannot assign")));
    let x = symbols.iter().find(|symbol| symbol.name == "x").unwrap();
    assert!(x.appearances.iter().any(|appearance| {
        appearance.value.as_deref() == Some("true")
            && values
                .get(&(appearance.line, appearance.column))
                .map(String::as_str)
                == Some("true")
    }));
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
