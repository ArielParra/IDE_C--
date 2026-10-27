pub mod comp_ops;
pub mod lexer;
pub mod parser;
pub mod semantic;

pub use lexer::analyze;
pub mod analysis;
pub use analysis::analyze_source;
