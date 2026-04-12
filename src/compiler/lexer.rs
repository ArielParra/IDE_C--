#[derive(Debug)]
pub struct Token {
    pub tipo: String,
    pub lexema: String,
    pub linea: usize,
    pub columna: usize,
}

#[derive(Debug)]
pub struct ErrorLexico {
    pub message: String,
    pub linea: usize,
    pub columna: usize,
}

pub fn analizar(text: &str) -> (Vec<Token>, Vec<ErrorLexico>) {

    let mut tokens = Vec::new();
    let mut errores = Vec::new();

    let mut linea = 1;
    let mut columna = 1;

    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    fn lookahead_skip_whitespace(chars: &[char], start: usize) -> (Option<char>, usize, usize, usize) {
        let mut i = start;
        let mut line_breaks = 0;
        let mut spaces = 0;
        
        while i < chars.len() && (chars[i] == ' ' || chars[i] == '\t' || chars[i] == '\n') {
            if chars[i] == '\n' {
                line_breaks += 1;
                spaces = 0;
            } else {
                spaces += 1;
            }
            i += 1;
        }
        
        let next = if i < chars.len() { Some(chars[i]) } else { None };
        (next, i, line_breaks, spaces)
    }

    while i < chars.len() {

        let c = chars[i];

        // ---------------- WHITESPACE ----------------

        if c == ' ' || c == '\t' {
            columna += 1;
            i += 1;
            continue;
        }

        if c == '\n' {
            linea += 1;
            columna = 1;
            i += 1;
            continue;
        }

        // ---------------- INCREMENT/DECREMENT OPERATORS WITH LOOKAHEAD ----------------

        if c == '+' {
            let (next, pos_next, breaks, spaces) = lookahead_skip_whitespace(&chars, i + 1);
            
            if let Some('+') = next {
                tokens.push(Token {
                    tipo: "OP".into(),
                    lexema: "++".into(),
                    linea,
                    columna,
                });
                
                i = pos_next + 1;
                linea += breaks;
                if breaks > 0 {
                    columna = 1 + spaces;
                } else {
                    columna += 2 + spaces;
                }
                continue;
            } else {
                tokens.push(Token {
                    tipo: "ARIT".into(),
                    lexema: "+".into(),
                    linea,
                    columna,
                });
                i += 1;
                columna += 1;
                continue;
            }
        }
        
        if c == '-' {
            let (next, pos_next, breaks, spaces) = lookahead_skip_whitespace(&chars, i + 1);
            
            if let Some('-') = next {
                tokens.push(Token {
                    tipo: "OP".into(),
                    lexema: "--".into(),
                    linea,
                    columna,
                });
                
                i = pos_next + 1;
                linea += breaks;
                if breaks > 0 {
                    columna = 1 + spaces;
                } else {
                    columna += 2 + spaces;
                }
                continue;
            } else {
                tokens.push(Token {
                    tipo: "ARIT".into(),
                    lexema: "-".into(),
                    linea,
                    columna,
                });
                i += 1;
                columna += 1;
                continue;
            }
        }

       // ---------------- LINE COMMENT // ----------------
        if i + 1 < chars.len() && chars[i] == '/' && chars[i+1] == '/' {
            i += 2;
            columna += 2;
            
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
                columna += 1;
            }
            
            continue;
        }

        // ---------------- BLOCK COMMENT /* */ ----------------
        if i + 1 < chars.len() && chars[i] == '/' && chars[i+1] == '*' {
            let start_linea = linea;
            let start_col = columna;
            i += 2;
            columna += 2;
            
            while i + 1 < chars.len() && 
                !(chars[i] == '*' && chars[i+1] == '/') {
                if chars[i] == '\n' {
                    linea += 1;
                    columna = 1;
                } else {
                    columna += 1;
                }
                i += 1;
            }
            
            if i + 1 >= chars.len() {
                errores.push(ErrorLexico {
                    message: "Unclosed block comment".into(),
                    linea: start_linea,
                    columna: start_col,
                });
                continue;
            }
            
            i += 2;
            columna += 2;
            
            continue;
        }

        // ---------------- STRING ----------------

        if c == '"' {

            let col = columna;
            let start = i;
            let start_linea = linea;

            i += 1;
            columna += 1;

            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\n' {
                    linea += 1;
                    columna = 1;
                } else {
                    columna += 1;
                }
                i += 1;
            }

            if i >= chars.len() {
                errores.push(ErrorLexico {
                    message: format!("Unclosed string: '{}'", chars[start..].iter().collect::<String>()),
                    linea: start_linea,
                    columna: col,
                });
                i += 1;
                continue;
            }

            i += 1;
            columna += 1;

            let lexema: String = chars[start..i].iter().collect();

            tokens.push(Token {
                tipo: "STRING".into(),
                lexema,
                linea: start_linea,
                columna: col,
            });

            continue;
        }

        // ---------------- CHAR ----------------

        if c == '\'' {

            let col = columna;
            let start = i;
            let start_linea = linea;

            i += 1;
            columna += 1;

            while i < chars.len() && chars[i] != '\'' {
                if chars[i] == '\n' {
                    linea += 1;
                    columna = 1;
                } else {
                    columna += 1;
                }
                i += 1;
            }

            if i >= chars.len() {
                errores.push(ErrorLexico {
                    message: format!("Unclosed char: '{}'", chars[start..].iter().collect::<String>()),
                    linea: start_linea,
                    columna: col,
                });
                i += 1;
                continue;
            }

            i += 1;
            columna += 1;

            let lexema: String = chars[start..i].iter().collect();

            tokens.push(Token {
                tipo: "CHAR".into(),
                lexema,
                linea: start_linea,
                columna: col,
            });

            continue;
        }

                // ---------------- NUMBER ----------------

        if c.is_ascii_digit() {

            let start = i;
            let col = columna;
            let start_linea = linea;

            let mut has_dot = false;
            let mut dot_error = false;

            while i < chars.len() {

                if chars[i].is_ascii_digit() {
                    i += 1;
                    columna += 1;
                }

                else if chars[i] == '.' && !has_dot {
                    if i+1 < chars.len() && chars[i+1].is_ascii_digit() {
                        has_dot = true;
                        i += 1;
                        columna += 1;
                    } else {
                        dot_error = true;
                        break;
                    }
                }
                else {
                    break;
                }
            }

            if dot_error {
                let lexema_error: String = chars[start..=i].iter().collect();
                errores.push(ErrorLexico {
                    message: format!("Malformed number: '{}'", lexema_error),
                    linea: start_linea,
                    columna,
                });
                i += 1;
                columna += 1;
                continue;
            }

            let lexema: String = chars[start..i].iter().collect();

            tokens.push(Token {
                tipo: if has_dot { "FLOAT" } else { "INT" }.into(),
                lexema,
                linea: start_linea,
                columna: col,
            });

            continue;
        }
        // ---------------- IDENT / RESERVADAS ----------------

        if c.is_ascii_alphabetic() || c == '_' {

            let start = i;
            let col = columna;

            while i < chars.len() &&
                (chars[i].is_ascii_alphanumeric() || chars[i] == '_')
            {
                i += 1;
                columna += 1;
            }

            let lexema: String = chars[start..i].iter().collect();

            let tipo = match lexema.as_str() {

                "if" => "IF",
                "else" => "ELSE",
                "end" => "END",
                "do" => "DO",
                "while" => "WHILE",
                "for" => "FOR",
                "switch" => "SWITCH",
                "case" => "CASE",
                "return" => "RETURN",
                "void" => "VOID",
                "int" => "INT_T",
                "float" => "FLOAT_T",
                "char" => "CHAR_T",
                "bool" => "BOOL_T",
                "true" => "TRUE",
                "false" => "FALSE",
                "main" => "MAIN",
                "cin" => "CIN",
                "cout" => "COUT",
                "include" => "INCLUDE",
                "define" => "DEFINE",
                "struct" => "STRUCT",
                "break" => "BREAK",
                "continue" => "CONTINUE",

                _ => "ID",
            };

            tokens.push(Token {
                tipo: tipo.into(),
                lexema,
                linea,
                columna: col,
            });

            continue;
        }

        // ---------------- DOUBLE OPERATORS (no lookahead for other operators) ----------------

        if i + 1 < chars.len() {

            let dos: String = chars[i..i+2].iter().collect();

            if [
                "==","!=","<=",">=",
                "&&","||"
            ].contains(&dos.as_str()) {

                tokens.push(Token {
                    tipo: "OP".into(),
                    lexema: dos,
                    linea,
                    columna,
                });

                i += 2;
                columna += 2;
                continue;
            }
        }

        // ---------------- ARITHMETIC ----------------

        if "*/%^".contains(c) {

            tokens.push(Token {
                tipo: "ARIT".into(),
                lexema: c.to_string(),
                linea,
                columna,
            });

            i += 1;
            columna += 1;
            continue;
        }

        // ---------------- REL / LOG / ASIG ----------------

        if c == '=' || c == '<' || c == '>' || c == '!' {

            if i + 1 < chars.len() && chars[i + 1] == '=' {
                let dos: String = chars[i..i+2].iter().collect();
                tokens.push(Token {
                    tipo: "REL".into(),
                    lexema: dos,
                    linea,
                    columna,
                });
                i += 2;
                columna += 2;
                continue;
            }

            tokens.push(Token {
                tipo: if c == '=' { "ASIG" } else { "REL" }.into(),
                lexema: c.to_string(),
                linea,
                columna,
            });

            i += 1;
            columna += 1;
            continue;
        }

        // ---------------- SIMBOLOS ----------------

        if "(){};, ".contains(c) {

            tokens.push(Token {
                tipo: "SYM".into(),
                lexema: c.to_string(),
                linea,
                columna,
            });

            i += 1;
            columna += 1;
            continue;
        }

        // ---------------- ERROR ----------------

        errores.push(ErrorLexico {
            message: format!("Invalid character {}", c),
            linea,
            columna,
        });

        i += 1;
        columna += 1;
    }

    (tokens, errores)
}