use crate::compiler::lexer::analyzer::analyze;

#[test]
fn test_standard_tokens() {
    let code = "int x = 5;";
    let (tokens, errors) = analyze(code);
    assert!(errors.is_empty(), "Expected no errors");
    assert_eq!(tokens.len(), 5);
    assert_eq!(tokens[0].token_type, "INT_T");
    assert_eq!(tokens[1].token_type, "ID");
    assert_eq!(tokens[1].lexeme, "x");
    assert_eq!(tokens[2].token_type, "ASIG");
    assert_eq!(tokens[2].lexeme, "=");
    assert_eq!(tokens[3].token_type, "INT");
    assert_eq!(tokens[3].lexeme, "5");
    assert_eq!(tokens[4].token_type, "SYM");
    assert_eq!(tokens[4].lexeme, ";");
}

#[test]
fn test_strings_and_chars() {
    let code = r#" "hello" 'a' "#;
    let (tokens, errors) = analyze(code);
    assert!(errors.is_empty(), "Expected no errors");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].token_type, "STRING");
    assert_eq!(tokens[0].lexeme, "\"hello\"");
    assert_eq!(tokens[1].token_type, "CHAR");
    assert_eq!(tokens[1].lexeme, "'a'");
}

#[test]
fn test_comments_ignored() {
    let code = "int // line comment\nx /* block comment */ = 5;";
    let (tokens, errors) = analyze(code);
    assert!(errors.is_empty(), "Expected no errors");
    assert_eq!(tokens.len(), 5);
    assert_eq!(tokens[0].token_type, "INT_T");
    assert_eq!(tokens[1].token_type, "ID");
    assert_eq!(tokens[1].lexeme, "x");
    assert_eq!(tokens[2].token_type, "ASIG");
}

#[test]
fn test_lexical_errors() {
    let code = "int @ = 5; /* unclosed";
    let (tokens, errors) = analyze(code);
    assert_eq!(errors.len(), 2, "Expected an invalid character error and an unclosed comment error");
    assert_eq!(tokens.len(), 4);
}

#[test]
fn test_real_and_until_alias_tokens() {
    let code = "real x; do x = x + 1; until x > 3;";
    let (tokens, errors) = analyze(code);
    assert!(errors.is_empty(), "Expected no errors: {:?}", errors);

    assert!(tokens.iter().any(|token| token.token_type == "FLOAT_T" && token.lexeme == "real"));
    assert!(tokens.iter().any(|token| token.token_type == "UNTIL" && token.lexeme == "until"));
}
