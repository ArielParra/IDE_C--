pub struct AstNode {
    pub label: String,
    pub children: Vec<AstNode>,
}

impl AstNode {
    pub fn new(label: impl Into<String>) -> Self {
        AstNode {
            label: label.into(),
            children: Vec::new(),
        }
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
