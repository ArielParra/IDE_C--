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

    // Función auxiliar para mirar el siguiente caracter no-whitespace
    fn lookahead_skip_whitespace(chars: &[char], start: usize) -> (Option<char>, usize, usize, usize) {
        let mut i = start;
        let mut saltos_linea = 0;
        let mut espacios = 0;
        
        while i < chars.len() && (chars[i] == ' ' || chars[i] == '\t' || chars[i] == '\n') {
            if chars[i] == '\n' {
                saltos_linea += 1;
                espacios = 0;
            } else {
                espacios += 1;
            }
            i += 1;
        }
        
        let siguiente = if i < chars.len() { Some(chars[i]) } else { None };
        (siguiente, i, saltos_linea, espacios)
    }

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

        // ---------------- OPERADORES INCREMENTO/DECREMENTO CON LOOKAHEAD ----------------

        // Detectar ++ con saltos de línea y espacios intermedios
        if c == '+' {
            let (sig, pos_sig, saltos, espacios) = lookahead_skip_whitespace(&chars, i + 1);
            
            if let Some('+') = sig {
                // Es un operador ++ aunque haya saltos de línea
                tokens.push(Token {
                    tipo: "OP".into(),
                    lexema: "++".into(),
                    linea,
                    columna,
                });
                
                // Actualizar posición saltando los caracteres intermedios
                i = pos_sig + 1;
                linea += saltos;
                if saltos > 0 {
                    columna = 1 + espacios;
                } else {
                    columna += 2 + espacios;
                }
                continue;
            } else {
                // Es un + normal
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
        
        // Detectar -- con saltos de línea y espacios intermedios
        if c == '-' {
            let (sig, pos_sig, saltos, espacios) = lookahead_skip_whitespace(&chars, i + 1);
            
            if let Some('-') = sig {
                tokens.push(Token {
                    tipo: "OP".into(),
                    lexema: "--".into(),
                    linea,
                    columna,
                });
                
                i = pos_sig + 1;
                linea += saltos;
                if saltos > 0 {
                    columna = 1 + espacios;
                } else {
                    columna += 2 + espacios;
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

       // ---------------- COMENTARIO LINEA // ----------------
        if i + 1 < chars.len() && chars[i] == '/' && chars[i+1] == '/' {
            i += 2;
            columna += 2;
            
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
                columna += 1;
            }
            
            continue;
        }

        // ---------------- COMENTARIO BLOQUE /* */ ----------------
        if i + 1 < chars.len() && chars[i] == '/' && chars[i+1] == '*' {
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
            let start_linea = linea;

            let mut punto = false;
            let mut error_punto = false;

            while i < chars.len() {

                if chars[i].is_ascii_digit() {
                    i += 1;
                    columna += 1;
                }

                else if chars[i] == '.' && !punto {
                    // Verificar si después del punto hay dígitos
                    if i+1 < chars.len() && chars[i+1].is_ascii_digit() {
                        punto = true;
                        i += 1;
                        columna += 1;
                    } else {
                        // Punto sin dígitos después -> error
                        error_punto = true;
                        break;
                    }
                }
                else {
                    break;
                }
            }

            if error_punto {
                let lexema_error: String = chars[start..=i].iter().collect();
                errores.push(ErrorLexico {
                    mensaje: format!("Número mal formado: '{}'", lexema_error),
                    linea: start_linea,
                    columna: col,
                });
                i += 1;
                columna += 1;
                continue;
            }

            let lexema: String = chars[start..i].iter().collect();

            tokens.push(Token {
                tipo: if punto { "FLOAT" } else { "INT" }.into(),
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

        // ---------------- DOBLES (sin lookahead para otros operadores) ----------------

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

        // ---------------- ARITMETICOS ----------------

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