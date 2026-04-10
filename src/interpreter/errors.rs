use std::collections::HashMap;
use std::fmt;

use super::Value;

#[derive(Debug)]
pub enum HolyError {
    Return(Value),
    Sin { type_name: String, fields: HashMap<String, Value>, stack_trace: Vec<String> },
    Break,
    Continue,
}

impl HolyError {
    /// Append a call-frame name to the stack trace of a Sin error.
    /// Non-Sin errors pass through unchanged.
    pub fn push_frame(self, frame: String) -> Self {
        match self {
            HolyError::Sin { type_name, fields, mut stack_trace } => {
                stack_trace.push(frame);
                HolyError::Sin { type_name, fields, stack_trace }
            }
            other => other,
        }
    }
}

impl fmt::Display for HolyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HolyError::Return(_) => write!(f, "'reveal' was defiled outside a salm — revelation belongs only within the sacred salms"),
            HolyError::Sin { type_name, fields, stack_trace } => {
                if let Some(Value::Str(message)) = fields.get("message") {
                    write!(f, "an unabsolved sin has escaped into the world: {} — {}", type_name, message)?;
                } else {
                    write!(f, "an unabsolved sin has escaped into the world: {}", type_name)?;
                }
                for frame in stack_trace {
                    write!(f, "\n    at {}", frame)?;
                }
                Ok(())
            }
            HolyError::Break => write!(f, "'forsake' was invoked outside a litany — abandonment is permitted only within the litany"),
            HolyError::Continue => write!(f, "'ascend' was invoked outside a litany — ascension is permitted only within the litany"),
        }
    }
}
