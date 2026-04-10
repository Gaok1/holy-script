use std::collections::HashMap;
use std::io::{self, BufRead, Write};

use crate::ast::HolyType;

use super::{EvalResult, HolyError, Value};

const MESSAGE_FIELD: &str = "message";

/// Returns a `HolyError::Sin` with a single `message` field.
pub fn builtin_sin(name: &str, message: impl Into<String>) -> HolyError {
    let mut fields = HashMap::new();
    fields.insert(MESSAGE_FIELD.to_string(), Value::Str(message.into()));
    HolyError::Sin { type_name: name.to_string(), fields, stack_trace: vec![] }
}

/// Returns the schema for all built-in sin types.
pub fn builtin_sins() -> HashMap<String, Vec<(String, HolyType)>> {
    [
        "DivisionByZero",
        "InvalidArgumentCount",
        "InvalidContext",
        "InvalidDiscern",
        "IndexOutOfBounds",
        "InvalidReturnType",
        "TypeError",
        "UndefinedField",
        "UndefinedMethod",
        "UndefinedSalm",
        "UndefinedScripture",
        "UndefinedSin",
        "UndefinedType",
        "UndefinedTestament",
        "UndefinedVariable",
    ]
    .into_iter()
    .map(|name| (name.to_string(), vec![(MESSAGE_FIELD.to_string(), HolyType::Word)]))
    .collect()
}

// ── Covenant value helpers ────────────────────────────────────────────────────

pub fn grace_granted(v: Value) -> Value {
    Value::CovenantVariant {
        covenant:  "grace".to_string(),
        type_args: vec![],
        variant:   "granted".to_string(),
        fields:    vec![v],
    }
}

pub fn grace_absent() -> Value {
    Value::CovenantVariant {
        covenant:  "grace".to_string(),
        type_args: vec![],
        variant:   "absent".to_string(),
        fields:    vec![],
    }
}

pub fn verdict_righteous(v: Value) -> Value {
    Value::CovenantVariant {
        covenant:  "verdict".to_string(),
        type_args: vec![],
        variant:   "righteous".to_string(),
        fields:    vec![v],
    }
}

pub fn verdict_condemned(msg: impl Into<String>) -> Value {
    Value::CovenantVariant {
        covenant:  "verdict".to_string(),
        type_args: vec![],
        variant:   "condemned".to_string(),
        fields:    vec![Value::Str(msg.into())],
    }
}

// ── Value equality (for contains / index_of) ─────────────────────────────────

pub fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(x),   Value::Int(y))   => x == y,
        (Value::Float(x), Value::Float(y)) => x == y,
        (Value::Str(x),   Value::Str(y))   => x == y,
        (Value::Bool(x),  Value::Bool(y))  => x == y,
        (Value::Void,     Value::Void)     => true,
        _ => false,
    }
}

// ── Built-in function dispatch ────────────────────────────────────────────────

/// Tries to execute a built-in function by name.
/// Returns `Some(result)` if the name matched a builtin, `None` otherwise.
pub fn call_builtin_fn(name: &str, args: Vec<Value>) -> Option<EvalResult> {
    match name {
        // ── Collections ───────────────────────────────────────────────────────
        "legion" => Some(Ok(Value::Legion(args))),

        // ── I/O ───────────────────────────────────────────────────────────────
        "proclaim" => {
            let text = args.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" ");
            println!("{}", text);
            Some(Ok(Value::Void))
        }

        "herald" => {
            let text = args.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" ");
            print!("{}", text);
            io::stdout().flush().ok();
            Some(Ok(Value::Void))
        }

        "inquire" => {
            let mut line = String::new();
            io::stdin().lock().read_line(&mut line).ok();
            Some(Ok(Value::Str(line.trim_end_matches('\n').to_string())))
        }

        "read_file" => {
            let path = match args.first() {
                Some(Value::Str(s)) => s.clone(),
                _ => return Some(Err(builtin_sin("TypeError", "'read_file' demands a word path"))),
            };
            let result = std::fs::read_to_string(&path);
            Some(Ok(match result {
                Ok(content) => verdict_righteous(Value::Str(content)),
                Err(e)      => verdict_condemned(e.to_string()),
            }))
        }

        "write_file" => {
            if args.len() != 2 {
                return Some(Err(builtin_sin("InvalidArgumentCount", "'write_file' demands a path and content")));
            }
            let path = match &args[0] {
                Value::Str(s) => s.clone(),
                _ => return Some(Err(builtin_sin("TypeError", "'write_file' demands a word path"))),
            };
            let content = match &args[1] {
                Value::Str(s) => s.clone(),
                _ => return Some(Err(builtin_sin("TypeError", "'write_file' demands word content"))),
            };
            let result = std::fs::write(&path, content.as_bytes());
            Some(Ok(match result {
                Ok(())  => verdict_righteous(Value::Bool(true)),
                Err(e)  => verdict_condemned(e.to_string()),
            }))
        }

        "exit" => {
            let code = match args.first() {
                Some(Value::Int(n)) => *n as i32,
                None => 0,
                _ => 0,
            };
            std::process::exit(code);
        }

        // ── Type conversion ───────────────────────────────────────────────────
        "atom_of" => {
            let n = match args.first() {
                Some(Value::Str(s)) => s.trim().parse().unwrap_or(0),
                Some(Value::Int(n)) => *n,
                _ => 0,
            };
            Some(Ok(Value::Int(n)))
        }

        "parse_atom" => {
            let s = match args.first() {
                Some(Value::Str(s)) => s.trim().to_string(),
                _ => return Some(Ok(verdict_condemned("'parse_atom' demands a word argument"))),
            };
            Some(Ok(match s.parse::<i64>() {
                Ok(n)  => verdict_righteous(Value::Int(n)),
                Err(e) => verdict_condemned(format!("'{}' is not a valid atom: {}", s, e)),
            }))
        }

        "fractional_of" => {
            let f = match args.first() {
                Some(Value::Int(n))   => *n as f64,
                Some(Value::Float(x)) => *x,
                Some(Value::Str(s))   => s.trim().parse::<f64>().unwrap_or(0.0),
                _ => 0.0,
            };
            Some(Ok(Value::Float(f)))
        }

        "word_of" => {
            Some(Ok(Value::Str(args.first().map(|v| v.to_string()).unwrap_or_default())))
        }

        // ── Math ──────────────────────────────────────────────────────────────
        "abs" => {
            Some(match args.first() {
                Some(Value::Int(n))   => Ok(Value::Int(n.abs())),
                Some(Value::Float(x)) => Ok(Value::Float(x.abs())),
                _ => Err(builtin_sin("TypeError", "'abs' demands an atom or fractional")),
            })
        }

        "floor" => {
            Some(match args.first() {
                Some(Value::Int(n))   => Ok(Value::Int(*n)),
                Some(Value::Float(x)) => Ok(Value::Int(x.floor() as i64)),
                _ => Err(builtin_sin("TypeError", "'floor' demands a fractional")),
            })
        }

        "ceil" => {
            Some(match args.first() {
                Some(Value::Int(n))   => Ok(Value::Int(*n)),
                Some(Value::Float(x)) => Ok(Value::Int(x.ceil() as i64)),
                _ => Err(builtin_sin("TypeError", "'ceil' demands a fractional")),
            })
        }

        "round" => {
            Some(match args.first() {
                Some(Value::Int(n))   => Ok(Value::Int(*n)),
                Some(Value::Float(x)) => Ok(Value::Int(x.round() as i64)),
                _ => Err(builtin_sin("TypeError", "'round' demands a fractional")),
            })
        }

        "min" => {
            if args.len() != 2 {
                return Some(Err(builtin_sin("InvalidArgumentCount", "'min' demands 2 offerings")));
            }
            Some(match (&args[0], &args[1]) {
                (Value::Int(a),   Value::Int(b))   => Ok(Value::Int(*a.min(b))),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.min(*b))),
                (Value::Int(a),   Value::Float(b)) => Ok(Value::Float((*a as f64).min(*b))),
                (Value::Float(a), Value::Int(b))   => Ok(Value::Float(a.min(*b as f64))),
                _ => Err(builtin_sin("TypeError", "'min' demands two numeric values")),
            })
        }

        "max" => {
            if args.len() != 2 {
                return Some(Err(builtin_sin("InvalidArgumentCount", "'max' demands 2 offerings")));
            }
            Some(match (&args[0], &args[1]) {
                (Value::Int(a),   Value::Int(b))   => Ok(Value::Int(*a.max(b))),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.max(*b))),
                (Value::Int(a),   Value::Float(b)) => Ok(Value::Float((*a as f64).max(*b))),
                (Value::Float(a), Value::Int(b))   => Ok(Value::Float(a.max(*b as f64))),
                _ => Err(builtin_sin("TypeError", "'max' demands two numeric values")),
            })
        }

        "pow" => {
            if args.len() != 2 {
                return Some(Err(builtin_sin("InvalidArgumentCount", "'pow' demands 2 offerings")));
            }
            Some(match (&args[0], &args[1]) {
                (Value::Int(base),   Value::Int(exp)) if *exp >= 0 => Ok(Value::Int(base.pow(*exp as u32))),
                (Value::Int(base),   Value::Int(exp))              => Ok(Value::Float((*base as f64).powi(*exp as i32))),
                (Value::Float(base), Value::Int(exp))              => Ok(Value::Float(base.powi(*exp as i32))),
                (Value::Float(base), Value::Float(exp))            => Ok(Value::Float(base.powf(*exp))),
                (Value::Int(base),   Value::Float(exp))            => Ok(Value::Float((*base as f64).powf(*exp))),
                _ => Err(builtin_sin("TypeError", "'pow' demands numeric arguments")),
            })
        }

        "sqrt" => {
            let x = match args.first() {
                Some(Value::Int(n))   => *n as f64,
                Some(Value::Float(x)) => *x,
                _ => return Some(Err(builtin_sin("TypeError", "'sqrt' demands a numeric argument"))),
            };
            Some(Ok(Value::Float(x.sqrt())))
        }

        // ── Trigonometry (arguments and results in radians) ───────────────────
        "sine" => {
            let x = match args.first() {
                Some(Value::Int(n))   => *n as f64,
                Some(Value::Float(x)) => *x,
                _ => return Some(Err(builtin_sin("TypeError", "'sine' demands a numeric argument"))),
            };
            Some(Ok(Value::Float(x.sin())))
        }

        "cos" => {
            let x = match args.first() {
                Some(Value::Int(n))   => *n as f64,
                Some(Value::Float(x)) => *x,
                _ => return Some(Err(builtin_sin("TypeError", "'cos' demands a numeric argument"))),
            };
            Some(Ok(Value::Float(x.cos())))
        }

        "tan" => {
            let x = match args.first() {
                Some(Value::Int(n))   => *n as f64,
                Some(Value::Float(x)) => *x,
                _ => return Some(Err(builtin_sin("TypeError", "'tan' demands a numeric argument"))),
            };
            Some(Ok(Value::Float(x.tan())))
        }

        "asin" => {
            let x = match args.first() {
                Some(Value::Int(n))   => *n as f64,
                Some(Value::Float(x)) => *x,
                _ => return Some(Err(builtin_sin("TypeError", "'asin' demands a numeric argument"))),
            };
            Some(Ok(Value::Float(x.asin())))
        }

        "acos" => {
            let x = match args.first() {
                Some(Value::Int(n))   => *n as f64,
                Some(Value::Float(x)) => *x,
                _ => return Some(Err(builtin_sin("TypeError", "'acos' demands a numeric argument"))),
            };
            Some(Ok(Value::Float(x.acos())))
        }

        "atan" => {
            let x = match args.first() {
                Some(Value::Int(n))   => *n as f64,
                Some(Value::Float(x)) => *x,
                _ => return Some(Err(builtin_sin("TypeError", "'atan' demands a numeric argument"))),
            };
            Some(Ok(Value::Float(x.atan())))
        }

        "atan2" => {
            if args.len() != 2 {
                return Some(Err(builtin_sin("InvalidArgumentCount", "'atan2' demands 2 offerings (y, x)")));
            }
            let y = match &args[0] {
                Value::Int(n)   => *n as f64,
                Value::Float(x) => *x,
                _ => return Some(Err(builtin_sin("TypeError", "'atan2' demands numeric arguments"))),
            };
            let x = match &args[1] {
                Value::Int(n)   => *n as f64,
                Value::Float(x) => *x,
                _ => return Some(Err(builtin_sin("TypeError", "'atan2' demands numeric arguments"))),
            };
            Some(Ok(Value::Float(y.atan2(x))))
        }

        // ── Logarithms ────────────────────────────────────────────────────────
        "ln" => {
            let x = match args.first() {
                Some(Value::Int(n))   => *n as f64,
                Some(Value::Float(x)) => *x,
                _ => return Some(Err(builtin_sin("TypeError", "'ln' demands a numeric argument"))),
            };
            Some(Ok(Value::Float(x.ln())))
        }

        "log2" => {
            let x = match args.first() {
                Some(Value::Int(n))   => *n as f64,
                Some(Value::Float(x)) => *x,
                _ => return Some(Err(builtin_sin("TypeError", "'log2' demands a numeric argument"))),
            };
            Some(Ok(Value::Float(x.log2())))
        }

        "log10" => {
            let x = match args.first() {
                Some(Value::Int(n))   => *n as f64,
                Some(Value::Float(x)) => *x,
                _ => return Some(Err(builtin_sin("TypeError", "'log10' demands a numeric argument"))),
            };
            Some(Ok(Value::Float(x.log10())))
        }

        _ => None,
    }
}
