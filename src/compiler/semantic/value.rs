use super::SemanticType;

/// Only source constants are folded; identifiers never acquire runtime values.
#[derive(Debug, Clone)]
pub(super) enum Constant {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}
impl Constant {
    pub fn literal(label: &str) -> Option<Self> {
        if let Some(raw) = label.strip_prefix("number:") {
            let raw = raw.trim();
            if raw.contains('.') {
                let value: f64 = raw.parse().ok()?;
                value.is_finite().then_some(Self::Float(value))
            } else {
                raw.parse().ok().map(Self::Int)
            }
        } else if let Some(raw) = label.strip_prefix("bool:") {
            raw.trim().parse().ok().map(Self::Bool)
        } else {
            label
                .strip_prefix("string:")
                .map(|raw| Self::String(raw.trim().to_string()))
        }
    }
    pub fn kind(&self) -> SemanticType {
        match self {
            Self::Int(_) => SemanticType::Int,
            Self::Float(_) => SemanticType::Float,
            Self::Bool(_) => SemanticType::Bool,
            Self::String(_) => SemanticType::String,
        }
    }
    pub fn display(&self) -> String {
        match self {
            Self::Int(v) => v.to_string(),
            Self::Float(v) => format!("{v:?}"),
            Self::Bool(v) => v.to_string(),
            Self::String(v) => v.clone(),
        }
    }
    fn float(&self) -> f64 {
        match self {
            Self::Int(v) => *v as f64,
            Self::Float(v) => *v,
            _ => unreachable!("type checked numeric operand"),
        }
    }
    pub fn evaluate(label: &str, args: &[Self]) -> Result<Self, &'static str> {
        let overflow = "Constant arithmetic overflow or unsupported exponent.";
        if label == "Not" {
            if let Self::Bool(v) = args[0] {
                return Ok(Self::Bool(!v));
            }
        }
        if args.len() != 2 {
            return Err("Invalid constant expression.");
        }
        if let (Self::Bool(a), Self::Bool(b)) = (&args[0], &args[1]) {
            return match label {
                "LogicalAnd" => Ok(Self::Bool(*a && *b)),
                "LogicalOr" => Ok(Self::Bool(*a || *b)),
                "Relational(==)" => Ok(Self::Bool(a == b)),
                "Relational(!=)" => Ok(Self::Bool(a != b)),
                _ => Err("Invalid boolean operator."),
            };
        }
        let (a, b) = (&args[0], &args[1]);
        if label.starts_with("Relational(") {
            // Preserve exact integer comparisons beyond f64's integer precision.
            let order = match (a, b) {
                (Self::Int(a), Self::Int(b)) => Some(a.cmp(b)),
                _ => a.float().partial_cmp(&b.float()),
            };
            use std::cmp::Ordering::*;
            return Ok(Self::Bool(match label {
                "Relational(<)" => order == Some(Less),
                "Relational(>)" => order == Some(Greater),
                "Relational(<=)" => matches!(order, Some(Less | Equal)),
                "Relational(>=)" => matches!(order, Some(Greater | Equal)),
                "Relational(==)" => order == Some(Equal),
                "Relational(!=)" => order != Some(Equal),
                _ => return Err("Invalid relational operator."),
            }));
        }
        if matches!(label, "MulOp(/)" | "MulOp(%)") && b.float() == 0.0 {
            return Err("Division by zero in constant expression.");
        }
        if let (Self::Int(a), Self::Int(b)) = (a, b) {
            let result = match label {
                "AddOp(+)" => a.checked_add(*b),
                "AddOp(-)" => a.checked_sub(*b),
                "MulOp(*)" => a.checked_mul(*b),
                "MulOp(/)" => a.checked_div(*b),
                "MulOp(%)" => a.checked_rem(*b),
                "Power" => u32::try_from(*b).ok().and_then(|exp| a.checked_pow(exp)),
                _ => return Err("Invalid arithmetic operator."),
            };
            return result.map(Self::Int).ok_or(overflow);
        }
        let (a, b) = (a.float(), b.float());
        let value = match label {
            "AddOp(+)" => a + b,
            "AddOp(-)" => a - b,
            "MulOp(*)" => a * b,
            "MulOp(/)" => a / b,
            "MulOp(%)" => a % b,
            "Power" => a.powf(b),
            _ => return Err("Invalid arithmetic operator."),
        };
        if value.is_finite() {
            Ok(Self::Float(value))
        } else {
            Err(overflow)
        }
    }
}
