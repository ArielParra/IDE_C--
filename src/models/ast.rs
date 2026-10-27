#[derive(Debug, Clone, Default)]
pub struct AstNode {
    pub label: String,
    pub children: Vec<AstNode>,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub annotated_type: Option<String>,
    pub attributed_value: Option<String>,
    pub coerced_type: Option<String>,
}

impl AstNode {
    pub fn display_label(&self) -> String {
        let mut label = self.label.clone();
        if let Some(kind) = &self.annotated_type {
            label.push_str(&format!(" [type: {kind}"));
            if let Some(coerced) = &self.coerced_type {
                label.push_str(&format!(" -> {coerced}"));
            }
            if let Some(value) = &self.attributed_value {
                label.push_str(&format!(", value: {value}"));
            }
            label.push(']');
        }
        label
    }

    pub fn new(label: impl Into<String>) -> Self {
        AstNode {
            label: label.into(),
            children: Vec::new(),
            line: None,
            column: None,
            annotated_type: None,
            attributed_value: None,
            coerced_type: None,
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
