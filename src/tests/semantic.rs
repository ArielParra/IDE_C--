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
fn test_prefix_increment_updates_variable_in_expression_and_statement_workflows() {
    let code = "main {\n int a;\n int b;\n a = 4;\n b = ++a;\n ++a;\n b = a;\n}";
    let (tokens, lex_errors) = analyze(code);
    assert!(lex_errors.is_empty(), "Expected no lexical errors: {:?}", lex_errors);

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(syn_errors.is_empty(), "Expected no syntax errors: {:?}", syn_errors);

    let (errors, _, values) = SemanticAnalyzer::analyze_with_symbol_values(&ast);
    assert!(errors.is_empty(), "Expected no semantic errors: {:?}", errors);
    assert_eq!(values.get(&(5, 2)).map(String::as_str), Some("5"));
    assert_eq!(values.get(&(6, 2)).map(String::as_str), Some("6"));
    assert_eq!(values.get(&(7, 2)).map(String::as_str), Some("6"));
}

#[test]
fn test_increment_requires_a_numeric_variable() {
    let code = "main { int x; x = 1; ++2; }";
    let (tokens, lex_errors) = analyze(code);
    assert!(lex_errors.is_empty(), "Expected no lexical errors: {:?}", lex_errors);

    let (ast, syn_errors) = build_ast_from_tokens(&tokens);
    assert!(syn_errors.is_empty(), "Expected no syntax errors: {:?}", syn_errors);

    let errors = SemanticAnalyzer::analyze(&ast);
    assert!(errors.iter().any(|error| error.message.contains("require a numeric variable")),
        "Expected an increment lvalue error, got: {:?}", errors);
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
fn test_integer_division_truncates_and_mixed_division_produces_float() {
    let code = "main { int numerator; int denominator; int integer_quotient; float decimal_denominator; float float_quotient; numerator = 5; denominator = 2; integer_quotient = numerator / denominator; decimal_denominator = 2.0; float_quotient = numerator / decimal_denominator; }";
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

    let division = find_node(&ast, "MulOp(/)").expect("expected integer division AST node");
    let division_location = (division.line.unwrap(), division.column.unwrap());
    assert_eq!(
        values.get(&division_location).map(String::as_str),
        Some("2"),
        "integer division should truncate toward zero"
    );

    let integer_quotient = symbols
        .iter()
        .find(|symbol| symbol.name == "integer_quotient")
        .expect("expected integer quotient variable");
    assert_eq!(
        values
            .get(&(integer_quotient.line, integer_quotient.column))
            .map(String::as_str),
        Some("2")
    );

    let float_quotient = symbols
        .iter()
        .find(|symbol| symbol.name == "float_quotient")
        .expect("expected float quotient variable");
    assert_eq!(
        values
            .get(&(float_quotient.line, float_quotient.column))
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

#[test]
fn test_semantic_analysis_shows_evaluated_if_condition_values() {
    let code = "main { int x; bool ready; x = 3; ready = true; if (x != 2) {} if (ready && x > 1) {} if (!ready) {} }";
    let (tokens, lex_errors) = analyze(code);
    assert!(lex_errors.is_empty(), "Expected no lexical errors: {:?}", lex_errors);

    let (ast, syntax_errors) = build_ast_from_tokens(&tokens);
    assert!(syntax_errors.is_empty(), "Expected no syntax errors: {:?}", syntax_errors);

    let (errors, _, values) = SemanticAnalyzer::analyze_with_symbol_values(&ast);
    assert!(errors.is_empty(), "Expected no semantic errors: {:?}", errors);

    fn collect_selections<'a>(
        node: &'a crate::models::ast::AstNode,
        selections: &mut Vec<&'a crate::models::ast::AstNode>,
    ) {
        if node.label == "Selection" {
            selections.push(node);
        }
        for child in &node.children {
            collect_selections(child, selections);
        }
    }

    let mut selections = Vec::new();
    collect_selections(&ast, &mut selections);
    assert_eq!(selections.len(), 3);

    let condition_values = selections
        .iter()
        .map(|selection| {
            let location = (selection.line.unwrap(), selection.column.unwrap());
            values.get(&location).map(String::as_str)
        })
        .collect::<Vec<_>>();
    assert_eq!(condition_values, vec![Some("true"), Some("true"), Some("false")]);
}

#[test]
fn test_semantic_analysis_propagates_values_to_expression_roots() {
    let code = "main { int y; y = ((2 + 3) * 4 - 6) / 2 % 5 ^ 2; if (y > 6 && !(y == 7) || false) {} }";
    let (tokens, lex_errors) = analyze(code);
    assert!(lex_errors.is_empty(), "Expected no lexical errors: {:?}", lex_errors);

    let (ast, syntax_errors) = build_ast_from_tokens(&tokens);
    assert!(syntax_errors.is_empty(), "Expected no syntax errors: {:?}", syntax_errors);

    let (errors, _, values) = SemanticAnalyzer::analyze_with_symbol_values(&ast);
    assert!(errors.is_empty(), "Expected no semantic errors: {:?}", errors);

    fn collect_nodes<'a>(
        node: &'a crate::models::ast::AstNode,
        nodes: &mut Vec<&'a crate::models::ast::AstNode>,
    ) {
        nodes.push(node);
        for child in &node.children {
            collect_nodes(child, nodes);
        }
    }

    let mut nodes = Vec::new();
    collect_nodes(&ast, &mut nodes);
    let value_at = |node: &crate::models::ast::AstNode| {
        values
            .get(&(node.line.unwrap(), node.column.unwrap()))
            .map(String::as_str)
    };

    for (label, expected) in [
        ("AddOp(+)", "5"),
        ("MulOp(*)", "20"),
        ("AddOp(-)", "14"),
        ("MulOp(/)", "7"),
        ("MulOp(%)", "7"),
        ("Power", "25"),
        ("Relational(>)", "true"),
        ("Relational(==)", "true"),
        ("Not", "false"),
        ("LogicalAnd", "false"),
        ("LogicalOr", "false"),
    ] {
        let node = nodes
            .iter()
            .find(|node| node.label == label)
            .unwrap_or_else(|| panic!("expected AST node {label}"));
        assert_eq!(value_at(node), Some(expected), "wrong value on {label}");
    }

    let expression_root = nodes
        .iter()
        .find(|node| node.label == "ExpressionStatement")
        .expect("expected assignment expression wrapper");
    assert_eq!(value_at(expression_root), Some("7"));
}

#[test]
fn test_semantic_analysis_distinguishes_prefix_and_postfix_values() {
    let code = "main { int x; int y; int z; x = 2; y = x++; z = ++x; }";
    let (tokens, lex_errors) = analyze(code);
    assert!(lex_errors.is_empty(), "Expected no lexical errors: {:?}", lex_errors);

    let (ast, syntax_errors) = build_ast_from_tokens(&tokens);
    assert!(syntax_errors.is_empty(), "Expected no syntax errors: {:?}", syntax_errors);

    let (errors, symbols, values) = SemanticAnalyzer::analyze_with_symbol_values(&ast);
    assert!(errors.is_empty(), "Expected no semantic errors: {:?}", errors);

    for (name, expected) in [("x", "4"), ("y", "2"), ("z", "4")] {
        let symbol = symbols
            .iter()
            .find(|symbol| symbol.name == name)
            .expect("expected variable in symbol table");
        assert_eq!(
            values
                .get(&(symbol.line, symbol.column))
                .map(String::as_str),
            Some(expected),
            "incorrect final value for {name}"
        );
    }
}
