//! Semantic attributes and diagnostics derived from a copy of the parser AST.
mod value;

use crate::models::{SemanticError, ast::AstNode};
use std::collections::{HashMap, HashSet};
use value::Constant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticType {
    Int,
    Float,
    Bool,
    String,
    Unknown,
}

impl SemanticType {
    fn from_label(label: &str) -> Self {
        match label.trim() {
            "int" => Self::Int,
            "float" | "real" => Self::Float,
            "bool" => Self::Bool,
            "string" => Self::String,
            _ => Self::Unknown,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Int => "int",
            Self::Float => "float",
            Self::Bool => "bool",
            Self::String => "string",
            Self::Unknown => "unknown",
        }
    }
    fn is_numeric(self) -> bool {
        matches!(self, Self::Int | Self::Float)
    }
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub data_type: SemanticType,
    pub line: usize,
    pub column: usize,
    pub scope_id: usize,
    /// Logical storage slot, not a machine address or a byte offset.
    pub offset: usize,
    pub references: Vec<(usize, usize)>,
}

#[derive(Debug)]
pub struct SemanticResult {
    pub annotated_ast: AstNode,
    pub symbols: Vec<Symbol>,
    pub errors: Vec<SemanticError>,
}

#[derive(Default)]
struct SymbolTable {
    scopes: Vec<(usize, HashMap<String, usize>)>,
    symbols: Vec<Symbol>,
    next_scope: usize,
}

impl SymbolTable {
    fn push_scope(&mut self) {
        self.scopes.push((self.next_scope, HashMap::new()));
        self.next_scope += 1;
    }
    fn lookup(&self, name: &str) -> Option<usize> {
        self.scopes
            .iter()
            .rev()
            .find_map(|(_, entries)| entries.get(name).copied())
    }
    fn declare(&mut self, name: &str, data_type: SemanticType, line: usize, column: usize) -> bool {
        let (scope_id, entries) = self.scopes.last_mut().expect("active semantic scope");
        if entries.contains_key(name) {
            return false;
        }
        let offset = self.symbols.len();
        entries.insert(name.to_string(), offset);
        self.symbols.push(Symbol {
            name: name.to_string(),
            data_type,
            line,
            column,
            scope_id: *scope_id,
            offset,
            references: Vec::new(),
        });
        true
    }
}

#[derive(Clone)]
struct Attribute {
    kind: SemanticType,
    value: Option<Constant>,
}
impl Attribute {
    fn unknown() -> Self {
        Self {
            kind: SemanticType::Unknown,
            value: None,
        }
    }
    fn typed(kind: SemanticType) -> Self {
        Self { kind, value: None }
    }
}

pub struct SemanticAnalyzer {
    errors: Vec<SemanticError>,
    table: SymbolTable,
    initialized: HashSet<usize>,
}

impl SemanticAnalyzer {
    #[allow(dead_code)] // Compatibility API for callers that only need diagnostics.
    pub fn analyze(ast: &AstNode) -> Vec<SemanticError> {
        Self::analyze_with_details(ast).errors
    }

    pub fn analyze_with_details(ast: &AstNode) -> SemanticResult {
        let mut analyzer = Self {
            errors: Vec::new(),
            table: SymbolTable::default(),
            initialized: HashSet::new(),
        };
        analyzer.table.push_scope();
        let mut annotated_ast = ast.clone();
        analyzer.visit(&mut annotated_ast);
        SemanticResult {
            annotated_ast,
            symbols: analyzer.table.symbols,
            errors: analyzer.errors,
        }
    }

    fn error(&mut self, node: &AstNode, message: impl AsRef<str>) {
        self.errors.push(SemanticError::new(
            message.as_ref(),
            node.line.unwrap_or(0),
            node.column.unwrap_or(0),
        ));
    }

    fn identifier(&mut self, node: &mut AstNode, read: bool) -> Option<usize> {
        let name = node.label.strip_prefix("id:")?.trim().to_string();
        let Some(id) = self.table.lookup(&name) else {
            self.error(
                node,
                format!("Variable '{name}' is not declared in this scope."),
            );
            node.annotate_type("unknown");
            return None;
        };
        let symbol = &mut self.table.symbols[id];
        symbol
            .references
            .push((node.line.unwrap_or(0), node.column.unwrap_or(0)));
        node.annotate_type(symbol.data_type.as_str());
        if read && !self.initialized.contains(&id) {
            self.error(
                node,
                format!("Variable '{name}' is used before initialization."),
            );
        }
        Some(id)
    }

    fn block(&mut self, node: &mut AstNode) {
        self.table.push_scope();
        for child in &mut node.children {
            self.visit(child);
        }
        let (_, entries) = self.table.scopes.pop().expect("active semantic scope");
        for id in entries.values() {
            self.initialized.remove(id);
        }
    }

    fn condition(&mut self, node: &mut AstNode, context: &str) {
        let kind = self.visit(node).kind;
        if kind != SemanticType::Bool && kind != SemanticType::Unknown {
            self.error(node, format!("Condition in '{context}' must be boolean."));
        }
    }

    fn visit(&mut self, node: &mut AstNode) -> Attribute {
        // Re-running on a previously annotated tree must not retain stale attributes.
        node.annotated_type = None;
        node.attributed_value = None;
        node.coerced_type = None;
        let result = match node.label.as_str() {
            "VariableDeclaration" => {
                let kind = node
                    .children
                    .iter()
                    .find_map(|n| n.label.strip_prefix("type:"))
                    .map(SemanticType::from_label)
                    .unwrap_or(SemanticType::Unknown);
                for child in &mut node.children {
                    if child.label.starts_with("type:") {
                        child.annotate_type(kind.as_str());
                    }
                    if child.label == "Identifiers" {
                        child.annotate_type(kind.as_str());
                        for identifier in &mut child.children {
                            let name = identifier
                                .label
                                .strip_prefix("id:")
                                .unwrap_or(&identifier.label)
                                .trim()
                                .to_string();
                            identifier.annotate_type(kind.as_str());
                            if !self.table.declare(
                                &name,
                                kind,
                                identifier.line.unwrap_or(0),
                                identifier.column.unwrap_or(0),
                            ) {
                                self.error(
                                    identifier,
                                    format!("Variable '{name}' is already declared in this scope."),
                                );
                            }
                        }
                    }
                }
                Attribute::typed(kind)
            }
            "Assignment" => {
                let before = self.errors.len();
                let id = node
                    .children
                    .first_mut()
                    .and_then(|n| self.identifier(n, false));
                let source = node
                    .children
                    .get_mut(1)
                    .map(|n| self.visit(n))
                    .unwrap_or_else(Attribute::unknown);
                let target = id
                    .map(|i| self.table.symbols[i].data_type)
                    .unwrap_or(SemanticType::Unknown);
                let compatible = target != SemanticType::Unknown
                    && source.kind != SemanticType::Unknown
                    && (target == source.kind
                        || (target == SemanticType::Float && source.kind == SemanticType::Int));
                if !compatible
                    && target != SemanticType::Unknown
                    && source.kind != SemanticType::Unknown
                {
                    self.error(
                        node,
                        format!(
                            "Cannot assign value of type '{}' to variable of type '{}'.",
                            source.kind.as_str(),
                            target.as_str()
                        ),
                    );
                }
                if compatible && target != source.kind {
                    if let Some(rhs) = node.children.get_mut(1) {
                        Self::coerce(rhs);
                    }
                }
                if compatible && self.errors.len() == before {
                    if let Some(id) = id {
                        self.initialized.insert(id);
                    }
                }
                Attribute::typed(target)
            }
            "Selection" => {
                if let Some(condition) = node.children.get_mut(1) {
                    self.condition(condition, "if");
                }
                let incoming = self.initialized.clone();
                let mut branches = Vec::new();
                for child in &mut node.children {
                    if matches!(child.label.as_str(), "ThenBlock" | "ElseBlock") {
                        self.initialized = incoming.clone();
                        self.block(child);
                        branches.push(self.initialized.clone());
                    }
                }
                self.initialized = if branches.len() == 2 {
                    branches[0].intersection(&branches[1]).copied().collect()
                } else {
                    incoming
                };
                Attribute::unknown()
            }
            "Iteration" => {
                if let Some(condition) = node.children.get_mut(1) {
                    self.condition(condition, "while");
                }
                let incoming = self.initialized.clone();
                for child in &mut node.children {
                    if child.label == "WhileBody" {
                        self.block(child);
                    }
                }
                self.initialized = incoming;
                Attribute::unknown()
            }
            "Repetition" => {
                for child in &mut node.children {
                    match child.label.as_str() {
                        "DoBody" => self.block(child),
                        "do" | "while" | "until" => {}
                        _ => self.condition(child, "repetition"),
                    }
                }
                Attribute::unknown()
            }
            "Input" => {
                for child in &mut node.children {
                    if let Some(id) = self.identifier(child, false) {
                        self.initialized.insert(id);
                    }
                }
                Attribute::unknown()
            }
            "ExpressionStatement" => node
                .children
                .iter_mut()
                .filter(|n| n.label != "empty")
                .map(|n| self.visit(n))
                .last()
                .unwrap_or_else(Attribute::unknown),
            _ if node.label.starts_with("id:") => {
                let id = self.identifier(node, true);
                Attribute::typed(
                    id.map(|i| self.table.symbols[i].data_type)
                        .unwrap_or(SemanticType::Unknown),
                )
            }
            _ if node.label.starts_with("number:")
                || node.label.starts_with("bool:")
                || node.label.starts_with("string:") =>
            {
                match Constant::literal(&node.label) {
                    Some(value) => Attribute {
                        kind: value.kind(),
                        value: Some(value),
                    },
                    None => {
                        self.error(node, "Constant is outside the supported numeric range.");
                        Attribute::unknown()
                    }
                }
            }
            _ if Self::is_operator(&node.label) => self.expression(node),
            _ => {
                for child in &mut node.children {
                    self.visit(child);
                }
                Attribute::unknown()
            }
        };
        if result.kind != SemanticType::Unknown || Self::is_operator(&node.label) {
            node.annotate_type(result.kind.as_str());
        }
        if let Some(value) = &result.value {
            node.annotate_value(&value.display());
        }
        result
    }

    fn coerce(node: &mut AstNode) {
        node.coerced_type = Some("float".to_string());
        if node.label == "ExpressionStatement" {
            if let Some(child) = node.children.first_mut() {
                Self::coerce(child);
            }
        }
    }

    fn is_operator(label: &str) -> bool {
        label == "Not"
            || [
                "AddOp(",
                "MulOp(",
                "Relational(",
                "Logical",
                "Power",
                "Prefix(",
                "Postfix(",
                "UnaryOp(",
            ]
            .iter()
            .any(|p| label.starts_with(p))
    }

    fn expression(&mut self, node: &mut AstNode) -> Attribute {
        let before = self.errors.len();
        let args: Vec<_> = node
            .children
            .iter_mut()
            .map(|child| self.visit(child))
            .collect();
        if args.is_empty() || args.iter().any(|a| a.kind == SemanticType::Unknown) {
            return Attribute::unknown();
        }
        let unary_update = ["Prefix(", "Postfix(", "UnaryOp("]
            .iter()
            .any(|p| node.label.starts_with(p));
        let kind = if unary_update {
            if args[0].kind.is_numeric() && node.children[0].label.starts_with("id:") {
                args[0].kind
            } else {
                self.error(
                    node,
                    "Increment and decrement operators require numeric variables.",
                );
                return Attribute::unknown();
            }
        } else if node.label == "Not" || node.label.starts_with("Logical") {
            if args.iter().all(|a| a.kind == SemanticType::Bool) {
                SemanticType::Bool
            } else {
                self.error(node, "Logical operators require boolean operands.");
                return Attribute::unknown();
            }
        } else if args.len() < 2 {
            return Attribute::unknown();
        } else if node.label.starts_with("Relational(") {
            let numeric = args.iter().all(|a| a.kind.is_numeric());
            let equality = matches!(node.label.as_str(), "Relational(==)" | "Relational(!=)");
            if numeric || (equality && args.iter().all(|a| a.kind == SemanticType::Bool)) {
                SemanticType::Bool
            } else {
                self.error(node, "Relational operators require comparable operands.");
                return Attribute::unknown();
            }
        } else if args.iter().all(|a| a.kind.is_numeric()) {
            if args.iter().any(|a| a.kind == SemanticType::Float) {
                SemanticType::Float
            } else {
                SemanticType::Int
            }
        } else {
            self.error(node, "Arithmetic operators require numeric operands.");
            return Attribute::unknown();
        };
        // Explicitly record numeric promotions on operands without changing the syntax tree.
        if args.len() == 2
            && args.iter().all(|a| a.kind.is_numeric())
            && args.iter().any(|a| a.kind == SemanticType::Float)
        {
            for (child, arg) in node.children.iter_mut().zip(&args) {
                if arg.kind == SemanticType::Int {
                    Self::coerce(child);
                }
            }
        }
        let value = if unary_update || self.errors.len() != before {
            None
        } else {
            let constants: Option<Vec<_>> = args.iter().map(|a| a.value.clone()).collect();
            match constants {
                Some(values) => match Constant::evaluate(&node.label, &values) {
                    Ok(value) => Some(value),
                    Err(message) => {
                        self.error(node, message);
                        None
                    }
                },
                None => None,
            }
        };
        Attribute { kind, value }
    }
}
