#[derive(Debug)]
pub struct Token {
    pub tipo: String,
    pub lexema: String,
    pub linea: usize,
    pub columna: usize,
}

#[derive(Debug)]
pub struct ErrorLexico {
    pub mensaje: String,
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

    while i < chars.len() {

        let c = chars[i];

        // ---------------- ESPACIOS ----------------

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

        // ---------------- COMENTARIO LINEA // ----------------

        if i + 1 < chars.len() && chars[i] == '/' && chars[i+1] == '/' {

            let col = columna;

            i += 2;
            columna += 2;

            while i < chars.len() && chars[i] != '\n' {
                i += 1;
                columna += 1;
            }

            tokens.push(Token {
                tipo: "COMMENT".into(),
                lexema: "//".into(),
                linea,
                columna: col,
            });

            continue;
        }

        // ---------------- COMENTARIO BLOQUE /* */ ----------------

        if i + 1 < chars.len() && chars[i] == '/' && chars[i+1] == '*' {

            let col = columna;

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

            i += 2;
            columna += 2;

            tokens.push(Token {
                tipo: "COMMENT".into(),
                lexema: "/* */".into(),
                linea,
                columna: col,
            });

            continue;
        }

        // ---------------- STRING ----------------

        if c == '"' {

            let col = columna;
            let start = i;

            i += 1;
            columna += 1;

            while i < chars.len() && chars[i] != '"' {
                i += 1;
                columna += 1;
            }

            i += 1;
            columna += 1;

            let lexema: String = chars[start..i].iter().collect();

            tokens.push(Token {
                tipo: "STRING".into(),
                lexema,
                linea,
                columna: col,
            });

            continue;
        }

        // ---------------- CHAR ----------------

        if c == '\'' {

            let col = columna;
            let start = i;

            i += 1;
            columna += 1;

            while i < chars.len() && chars[i] != '\'' {
                i += 1;
                columna += 1;
            }

            i += 1;
            columna += 1;

            let lexema: String = chars[start..i].iter().collect();

            tokens.push(Token {
                tipo: "CHAR".into(),
                lexema,
                linea,
                columna: col,
            });

            continue;
        }

        // ---------------- NUMERO ----------------

        if c.is_ascii_digit() {

            let start = i;
            let col = columna;

            let mut punto = false;

            while i < chars.len() {

                if chars[i].is_ascii_digit() {
                    i += 1;
                    columna += 1;
                }

                else if chars[i] == '.' && !punto {

                    if i+1 < chars.len() && chars[i+1].is_ascii_digit() {
                        punto = true;
                        i += 1;
                        columna += 1;
                    } else {
                        break;
                    }
                }
                else {
                    break;
                }
            }

            let lexema: String = chars[start..i].iter().collect();

            tokens.push(Token {
                tipo: if punto { "FLOAT" } else { "INT" }.into(),
                lexema,
                linea,
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
                "switch" => "SWITCH",
                "case" => "CASE",
                "int" => "INT_T",
                "float" => "FLOAT_T",
                "main" => "MAIN",
                "cin" => "CIN",
                "cout" => "COUT",

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

        // ---------------- DOBLES ----------------

        if i + 1 < chars.len() {

            let dos: String = chars[i..i+2].iter().collect();

            if [
                "==","!=","<=",">=",
                "&&","||",
                "++","--"
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

        // ---------------- ARITMETICOS ----------------

        if "+-*/%^".contains(c) {

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

        if "=<>!".contains(c) {

            tokens.push(Token {
                tipo: "REL".into(),
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
            mensaje: format!("Caracter invalido {}", c),
            linea,
            columna,
        });

        i += 1;
        columna += 1;
    }

    (tokens, errores)
}