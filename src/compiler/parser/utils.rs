use crate::models::Token;

#[allow(dead_code)]
pub fn parse_tokens_from_text(input: &str) -> Vec<Token> {
    input
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }

            let parts: Vec<&str> = trimmed.rsplitn(3, ' ').collect();
            if parts.len() != 3 {
                return None;
            }
            let column = parts[0].parse::<usize>().unwrap_or(0);
            let line = parts[1].parse::<usize>().unwrap_or(0);
            let rest = parts[2];
            let first_space = rest.find(' ')?;
            let token_type = &rest[..first_space];
            let lexeme = rest[first_space + 1..].to_string();
            Some(Token::new(token_type, &lexeme, line, column))
        })
        .collect()
}
