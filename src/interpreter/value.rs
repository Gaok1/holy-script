use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Legion(Vec<Value>),
    Void,
    /// `type_args` carries the resolved generic type parameters (e.g. `[Atom]` for
    /// `Stack of atom`).  Empty vec means "pending" — type params not yet resolved.
    CovenantVariant { covenant: String, type_args: Vec<crate::ast::HolyType>, variant: String, fields: Vec<Value> },
    /// Same semantics as CovenantVariant.type_args.
    Scripture { type_name: String, type_args: Vec<crate::ast::HolyType>, fields: HashMap<String, Value> },
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(x) => write!(f, "{}", x),
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", if *b { "blessed" } else { "forsaken" }),
            Value::Legion(items) => {
                let args = items.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ");
                write!(f, "[{}]", args)
            }
            Value::Void => write!(f, "void"),
            Value::CovenantVariant { covenant, variant, fields, .. } => {
                if fields.is_empty() {
                    write!(f, "{}::{}", covenant, variant)
                } else {
                    let args = fields.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ");
                    write!(f, "{}::{}({})", covenant, variant, args)
                }
            }
            Value::Scripture { type_name, fields, .. } => {
                write!(f, "{}{{", type_name)?;
                let mut first = true;
                let mut keys: Vec<_> = fields.keys().collect();
                keys.sort();
                for k in keys {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, fields[k])?;
                    first = false;
                }
                write!(f, "}}")
            }
        }
    }
}

pub fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Int(_) => "atom",
        Value::Float(_) => "fractional",
        Value::Str(_) => "word",
        Value::Bool(_) => "dogma",
        Value::Legion(_) => "legion",
        Value::Void => "void",
        Value::CovenantVariant { .. } => "covenant",
        Value::Scripture { .. } => "scripture",
    }
}
