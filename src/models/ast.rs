#[derive(Debug, Clone, Default)]
pub struct AstNode {
    pub label: String,
    pub children: Vec<AstNode>,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub annotated_type: Option<String>,
    pub attributed_value: Option<String>,
}

impl AstNode {
    pub fn new(label: impl Into<String>) -> Self {
        AstNode {
            label: label.into(),
            children: Vec::new(),
            line: None,
            column: None,
            annotated_type: None,
            attributed_value: None,
        }
    }

    pub fn with_pos(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    pub fn with_opt_pos(mut self, line: Option<usize>, column: Option<usize>) -> Self {
        self.line = line;
        self.column = column;
        self
    }

    pub fn add_child(&mut self, child: AstNode) {
        self.children.push(child);
    }

    pub fn annotate_type(&mut self, type_name: &str) {
        self.annotated_type = Some(type_name.to_string());
    }

    pub fn annotate_value(&mut self, value: &str) {
        self.attributed_value = Some(value.to_string());
    }
}
