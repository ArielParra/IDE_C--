// Compiler module for C-- language
pub mod lexer;
pub mod parser;
pub mod codegen;

pub struct Compiler {
    source: String,
}

impl Compiler {
    pub fn new(source: String) -> Self {
        Compiler { source }
    }

    pub fn compile(&self) -> Result<String, String> {
        // Tokenize
        let tokens = lexer::tokenize(&self.source)?;
        
        // Parse
        let ast = parser::parse(tokens)?;
        
        // Generate code
        let output = codegen::generate(&ast)?;
        
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile() {
        let code = "int main() { return 0; }".to_string();
        let compiler = Compiler::new(code);
        assert!(compiler.compile().is_ok());
    }
}