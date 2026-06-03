pub struct AstNode {
    pub label: String,
    pub children: Vec<AstNode>,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

impl AstNode {
    pub fn new(label: impl Into<String>) -> Self {
        AstNode {
            label: label.into(),
            children: Vec::new(),
            line: None,
            column: None,
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
}

impl Default for AstNode {
    fn default() -> Self {
        AstNode::new("")
    }
}
