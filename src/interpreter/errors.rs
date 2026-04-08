use std::collections::HashMap;
use std::fmt;

use super::Value;

#[derive(Debug)]
pub enum HolyError {
    Return(Value),
    Sin { type_name: String, fields: HashMap<String, Value> },
    Break,
    Continue,
}

impl fmt::Display for HolyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HolyError::Return(_) => write!(f, "'reveal' used outside a salm"),
            HolyError::Sin { type_name, fields } => {
                if let Some(Value::Str(message)) = fields.get("message") {
                    write!(f, "unhandled sin: {} ({})", type_name, message)
                } else {
                    write!(f, "unhandled sin: {}", type_name)
                }
            }
            HolyError::Break => write!(f, "'forsake' used outside a litany"),
            HolyError::Continue => write!(f, "'ascend' used outside a litany"),
        }
    }
}
