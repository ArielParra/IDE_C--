use std::collections::HashMap;

use crate::models::ast::AstNode;
use crate::models::SemanticError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticType {
    Int,
    Float,
    Bool,
    String,
    Unknown,
}

impl SemanticType {
    fn from_label(value: &str) -> Self {
        let label = value.trim();
        match label {
            "int" | "INT_T" | "INT" => Self::Int,
            "float" | "real" | "FLOAT_T" | "FLOAT" => Self::Float,
            "bool" | "BOOL_T" | "BOOL" => Self::Bool,
            "string" | "STRING" => Self::String,
            _ => Self::Unknown,
        }
    }

    fn from_literal(label: &str) -> Self {
        let normalized = label.trim();
        if normalized.starts_with("number:") {
            let raw = normalized.trim_start_matches("number:").trim();
            if raw.contains('.') {
                Self::Float
            } else {
                Self::Int
            }
        } else if normalized.starts_with("bool:") {
            Self::Bool
        } else if normalized.starts_with("string:") {
            Self::String
        } else {
            Self::Unknown
        }
    }

    fn is_numeric(&self) -> bool {
        matches!(self, Self::Int | Self::Float)
    }

    fn as_str(&self) -> &str {
        match self {
            Self::Int => "int",
            Self::Float => "float",
            Self::Bool => "bool",
            Self::String => "string",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub data_type: SemanticType,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn current_scope(&mut self) -> &mut HashMap<String, Symbol> {
        self.scopes.last_mut().expect("semantic scope missing")
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        None
    }

    pub fn declare(
        &mut self,
        name: &str,
        data_type: SemanticType,
        line: usize,
        column: usize,
    ) -> bool {
        let scope = self.current_scope();
        if scope.contains_key(name) {
            return false;
        }

        scope.insert(
            name.to_string(),
            Symbol {
                name: name.to_string(),
                data_type,
                line,
                column,
            },
        );
        true
    }
}

pub struct SemanticAnalyzer {
    errors: Vec<SemanticError>,
    symbol_table: SymbolTable,
    initialized_scopes: Vec<HashMap<String, bool>>,
}

impl SemanticAnalyzer {
    pub fn analyze(ast: &AstNode) -> Vec<SemanticError> {
        let mut analyzer = Self::new();
        analyzer.visit_node(ast);
        analyzer.errors
    }

    fn new() -> Self {
        Self {
            errors: Vec::new(),
            symbol_table: SymbolTable::new(),
            initialized_scopes: vec![HashMap::new()],
        }
    }

    fn current_scope(&mut self) -> &mut HashMap<String, Symbol> {
        self.symbol_table.current_scope()
    }

    fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.symbol_table.lookup(name)
    }

    fn declare(&mut self, name: &str, data_type: SemanticType, line: usize, column: usize) {
        if !self
            .symbol_table
            .declare(name, data_type.clone(), line, column)
        {
            self.errors.push(SemanticError::new(
                &format!("Variable '{}' is already declared in this scope.", name),
                line,
                column,
            ));
            return;
        }

        self.initialized_scopes
            .last_mut()
            .expect("semantic scope missing")
            .insert(name.to_string(), false);
    }

    fn push_scope(&mut self) {
        self.symbol_table.push_scope();
        self.initialized_scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.symbol_table.pop_scope();
        if self.initialized_scopes.len() > 1 {
            self.initialized_scopes.pop();
        }
    }

    fn mark_initialized(&mut self, name: &str) {
        for scope in self.initialized_scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), true);
                return;
            }
        }
    }

    fn check_initialized_use(&mut self, name: &str, line: usize, column: usize) {
        for scope in self.initialized_scopes.iter().rev() {
            if let Some(initialized) = scope.get(name) {
                if !initialized {
                    self.errors.push(SemanticError::new(
                        &format!("Variable '{}' is used before initialization.", name),
                        line,
                        column,
                    ));
                }
                return;
            }
        }
    }

    fn annotate_node_type(&self, node: &AstNode, result: SemanticType) {
        let mut node_copy = node.clone();
        node_copy.annotated_type = Some(result.as_str().to_string());
    }

    fn visit_node(&mut self, node: &AstNode) -> SemanticType {
        match node.label.as_str() {
            "Program" => {
                for child in &node.children {
                    self.visit_node(child);
                }
                SemanticType::Unknown
            }
            "MainProgram" => {
                for child in &node.children {
                    if child.label != "main" {
                        self.visit_node(child);
                    }
                }
                SemanticType::Unknown
            }
            "DeclarationList" => {
                for child in &node.children {
                    self.visit_node(child);
                }
                SemanticType::Unknown
            }
            "VariableDeclaration" => {
                let declared_type = self.extract_decl_type(node);
                let identifiers = node
                    .children
                    .iter()
                    .find(|child| child.label == "Identifiers");

                if let Some(list) = identifiers {
                    for identifier in &list.children {
                        let name = extract_identifier_name(identifier);
                        if name.is_empty() {
                            continue;
                        }
                        self.declare(
                            &name,
                            declared_type.clone(),
                            identifier.line.unwrap_or(0),
                            identifier.column.unwrap_or(0),
                        );
                    }
                }
                SemanticType::Unknown
            }
            "Assignment" => {
                let target_name = node
                    .children
                    .first()
                    .map(extract_identifier_name)
                    .unwrap_or_default();

                if target_name.is_empty() {
                    return SemanticType::Unknown;
                }

                let target_type = match self.lookup(&target_name).cloned() {
                    Some(symbol) => symbol.data_type,
                    None => {
                        self.errors.push(SemanticError::new(
                            &format!("Variable '{}' is not declared in this scope.", target_name),
                            node.line.unwrap_or(0),
                            node.column.unwrap_or(0),
                        ));
                        SemanticType::Unknown
                    }
                };
                let expr_type = node
                    .children
                    .iter()
                    .skip(1)
                    .find(|child| {
                        child.label != "ExpressionStatement" || !child.children.is_empty()
                    })
                    .map(|child| self.expression_type_from_node(child))
                    .unwrap_or(SemanticType::Unknown);

                if !self.is_assignable(target_type.clone(), expr_type.clone()) {
                    self.errors.push(SemanticError::new(
                        &format!(
                            "Cannot assign value of type '{}' to variable '{}' of type '{}'.",
                            expr_type.as_str(),
                            target_name,
                            target_type.as_str()
                        ),
                        node.line.unwrap_or(0),
                        node.column.unwrap_or(0),
                    ));
                }

                self.mark_initialized(&target_name);
                target_type
            }
            "ExpressionStatement" => {
                let mut result = SemanticType::Unknown;
                for child in &node.children {
                    if child.label == "empty" {
                        continue;
                    }
                    result = self.expression_type_from_node(child);
                }
                result
            }
            "Selection" => {
                let condition = node
                    .children
                    .get(1)
                    .map(|child| self.visit_node(child))
                    .unwrap_or(SemanticType::Unknown);
                if condition != SemanticType::Bool {
                    self.errors.push(SemanticError::new(
                        "Condition in 'if' must be boolean.",
                        node.line.unwrap_or(0),
                        node.column.unwrap_or(0),
                    ));
                }

                for child in &node.children {
                    if matches!(child.label.as_str(), "ThenBlock" | "ElseBlock") {
                        self.visit_block(child);
                    }
                }
                SemanticType::Unknown
            }
            "Iteration" => {
                let condition = node
                    .children
                    .get(1)
                    .map(|child| self.visit_node(child))
                    .unwrap_or(SemanticType::Unknown);
                if condition != SemanticType::Bool {
                    self.errors.push(SemanticError::new(
                        "Condition in 'while' must be boolean.",
                        node.line.unwrap_or(0),
                        node.column.unwrap_or(0),
                    ));
                }

                for child in &node.children {
                    if child.label == "WhileBody" {
                        self.visit_block(child);
                    }
                }
                SemanticType::Unknown
            }
            "Repetition" => {
                let mut condition_type = SemanticType::Unknown;
                for child in &node.children {
                    if child.label == "DoBody" {
                        self.visit_block(child);
                    } else if child.label == "while" || child.label == "until" {
                        continue;
                    } else if child.label != "do" {
                        condition_type = self.visit_node(child);
                        if condition_type != SemanticType::Bool {
                            self.errors.push(SemanticError::new(
                                "Condition in repetition loop must be boolean.",
                                node.line.unwrap_or(0),
                                node.column.unwrap_or(0),
                            ));
                        }
                    }
                }
                condition_type
            }
            "Input" => {
                for child in &node.children {
                    let name = extract_identifier_name(child);
                    if !name.is_empty() {
                        self.resolve_variable_type(
                            &name,
                            child.line.unwrap_or(0),
                            child.column.unwrap_or(0),
                        );
                        self.mark_initialized(&name);
                    }
                }
                SemanticType::Unknown
            }
            "Output" | "OutputChain" => {
                for child in &node.children {
                    self.visit_node(child);
                }
                SemanticType::Unknown
            }
            _ => {
                if node.label.starts_with("id: ") {
                    let name = extract_identifier_name(node);
                    return self.resolve_variable_type(
                        &name,
                        node.line.unwrap_or(0),
                        node.column.unwrap_or(0),
                    );
                }

                if node.label.starts_with("number:")
                    || node.label.starts_with("bool:")
                    || node.label.starts_with("string:")
                {
                    return SemanticType::from_literal(&node.label);
                }

                if node.label.starts_with("Relational(")
                    || node.label.starts_with("LogicalAnd")
                    || node.label.starts_with("LogicalOr")
                    || node.label.starts_with("AddOp(")
                    || node.label.starts_with("MulOp(")
                    || node.label.starts_with("Power")
                    || node.label.starts_with("Prefix(")
                    || node.label.starts_with("Postfix(")
                    || node.label.starts_with("UnaryOp(")
                    || node.label == "Not"
                {
                    return self.visit_expression(node);
                }

                for child in &node.children {
                    self.visit_node(child);
                }
                SemanticType::Unknown
            }
        }
    }

    fn visit_block(&mut self, node: &AstNode) {
        self.push_scope();
        for child in &node.children {
            self.visit_node(child);
        }
        self.pop_scope();
    }

    fn visit_expression(&mut self, node: &AstNode) -> SemanticType {
        if node.label == "Not" {
            let operand = node
                .children
                .first()
                .map(|child| self.visit_node(child))
                .unwrap_or(SemanticType::Unknown);
            if operand != SemanticType::Bool {
                self.errors.push(SemanticError::new(
                    "Logical negation requires a boolean operand.",
                    node.line.unwrap_or(0),
                    node.column.unwrap_or(0),
                ));
            }
            return SemanticType::Bool;
        }

        if node.label.starts_with("Relational(") {
            let left = node
                .children
                .first()
                .map(|child| self.visit_node(child))
                .unwrap_or(SemanticType::Unknown);
            let right = node
                .children
                .get(1)
                .map(|child| self.visit_node(child))
                .unwrap_or(SemanticType::Unknown);

            if !self.is_comparable(left.clone(), right.clone()) {
                self.errors.push(SemanticError::new(
                    "Relational operators require comparable operands.",
                    node.line.unwrap_or(0),
                    node.column.unwrap_or(0),
                ));
            }
            return SemanticType::Bool;
        }

        if node.label.starts_with("LogicalAnd") || node.label.starts_with("LogicalOr") {
            let left = node
                .children
                .first()
                .map(|child| self.visit_node(child))
                .unwrap_or(SemanticType::Unknown);
            let right = node
                .children
                .get(1)
                .map(|child| self.visit_node(child))
                .unwrap_or(SemanticType::Unknown);

            if left != SemanticType::Bool || right != SemanticType::Bool {
                self.errors.push(SemanticError::new(
                    "Logical operators require boolean operands.",
                    node.line.unwrap_or(0),
                    node.column.unwrap_or(0),
                ));
            }
            return SemanticType::Bool;
        }

        if node.label.starts_with("AddOp(")
            || node.label.starts_with("MulOp(")
            || node.label.starts_with("Power")
        {
            let left = node
                .children
                .first()
                .map(|child| self.visit_node(child))
                .unwrap_or(SemanticType::Unknown);
            let right = node
                .children
                .get(1)
                .map(|child| self.visit_node(child))
                .unwrap_or(SemanticType::Unknown);

            let result = if left.is_numeric() && right.is_numeric() {
                if left == SemanticType::Float || right == SemanticType::Float {
                    SemanticType::Float
                } else {
                    SemanticType::Int
                }
            } else {
                self.errors.push(SemanticError::new(
                    "Arithmetic operators require numeric operands.",
                    node.line.unwrap_or(0),
                    node.column.unwrap_or(0),
                ));
                SemanticType::Unknown
            };

            return result;
        }

        if node.label.starts_with("Prefix(")
            || node.label.starts_with("Postfix(")
            || node.label.starts_with("UnaryOp(")
        {
            let operand = node
                .children
                .first()
                .map(|child| self.visit_node(child))
                .unwrap_or(SemanticType::Unknown);
            if !operand.is_numeric() {
                self.errors.push(SemanticError::new(
                    "Increment and decrement operators require numeric variables.",
                    node.line.unwrap_or(0),
                    node.column.unwrap_or(0),
                ));
            }
            return operand;
        }

        if node.label.starts_with("id: ") {
            let name = extract_identifier_name(node);
            return self.resolve_variable_type(
                &name,
                node.line.unwrap_or(0),
                node.column.unwrap_or(0),
            );
        }

        if node.label.starts_with("number:")
            || node.label.starts_with("bool:")
            || node.label.starts_with("string:")
        {
            return SemanticType::from_literal(&node.label);
        }

        for child in &node.children {
            self.visit_node(child);
        }
        SemanticType::Unknown
    }

    fn resolve_variable_type(&mut self, name: &str, line: usize, column: usize) -> SemanticType {
        match self.lookup(name).cloned() {
            Some(symbol) => {
                self.check_initialized_use(name, line, column);
                symbol.data_type
            }
            None => {
                self.errors.push(SemanticError::new(
                    &format!("Variable '{}' is not declared in this scope.", name),
                    line,
                    column,
                ));
                SemanticType::Unknown
            }
        }
    }

    fn extract_decl_type(&self, node: &AstNode) -> SemanticType {
        node.children
            .iter()
            .find(|child| child.label.starts_with("type:"))
            .map(|child| child.label.trim_start_matches("type:").trim().to_string())
            .map(|kind| SemanticType::from_label(&kind))
            .unwrap_or(SemanticType::Unknown)
    }

    fn expression_type_from_node(&mut self, node: &AstNode) -> SemanticType {
        if node.label == "ExpressionStatement" {
            for child in &node.children {
                if child.label == "empty" {
                    continue;
                }
                return self.expression_type_from_node(child);
            }
            return SemanticType::Unknown;
        }

        if node.label.starts_with("id: ") {
            let name = extract_identifier_name(node);
            return self.resolve_variable_type(
                &name,
                node.line.unwrap_or(0),
                node.column.unwrap_or(0),
            );
        }

        if node.label.starts_with("number:")
            || node.label.starts_with("bool:")
            || node.label.starts_with("string:")
        {
            return SemanticType::from_literal(&node.label);
        }

        self.visit_expression(node)
    }

    fn is_assignable(&self, target: SemanticType, source: SemanticType) -> bool {
        if target == source {
            return true;
        }
        if target == SemanticType::Float && source == SemanticType::Int {
            return true;
        }
        false
    }

    fn is_comparable(&self, left: SemanticType, right: SemanticType) -> bool {
        if left == SemanticType::Unknown || right == SemanticType::Unknown {
            return false;
        }
        if left == SemanticType::Bool && right == SemanticType::Bool {
            return true;
        }
        if left.is_numeric() && right.is_numeric() {
            return true;
        }
        false
    }
}

fn extract_identifier_name(node: &AstNode) -> String {
    let label = node.label.trim();
    if let Some(rest) = label.strip_prefix("id: ") {
        rest.to_string()
    } else {
        label.to_string()
    }
}
