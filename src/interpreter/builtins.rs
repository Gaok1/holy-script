use std::collections::HashMap;

use crate::ast::HolyType;

use super::{HolyError, Value};

const MESSAGE_FIELD: &str = "message";

pub fn builtin_sins() -> HashMap<String, Vec<(String, HolyType)>> {
    [
        "DivisionByZero",
        "InvalidArgumentCount",
        "InvalidContext",
        "InvalidDiscern",
        "InvalidReturnType",
        "TypeError",
        "UndefinedField",
        "UndefinedMethod",
        "UndefinedSalm",
        "UndefinedScripture",
        "UndefinedSin",
        "UndefinedType",
        "UndefinedVariable",
    ]
    .into_iter()
    .map(|name| {
        (
            name.to_string(),
            vec![(MESSAGE_FIELD.to_string(), HolyType::Word)],
        )
    })
    .collect()
}

pub fn builtin_sin(name: &str, message: impl Into<String>) -> HolyError {
    let mut fields = HashMap::new();
    fields.insert(MESSAGE_FIELD.to_string(), Value::Str(message.into()));
    HolyError::Sin {
        type_name: name.to_string(),
        fields,
    }
}
