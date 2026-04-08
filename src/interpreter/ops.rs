use crate::ast::{BinOp, HolyType, Literal};

use super::builtins::builtin_sin;
use super::{EvalResult, HolyError, Value, value_type_name};

pub fn eval_literal(lit: &Literal) -> Value {
    match lit {
        Literal::Int(n) => Value::Int(*n),
        Literal::Float(f) => Value::Float(*f),
        Literal::Str(s) => Value::Str(s.clone()),
        Literal::Bool(b) => Value::Bool(*b),
    }
}

pub fn is_truthy(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::Int(n) => *n != 0,
        Value::Float(f) => *f != 0.0,
        Value::Str(s) => !s.is_empty(),
        Value::Void => false,
        Value::CovenantVariant { .. } => true,
        Value::Scripture { .. } => true,
    }
}

pub fn default_value(ty: &HolyType) -> Value {
    match ty {
        HolyType::Atom => Value::Int(0),
        HolyType::Fractional => Value::Float(0.0),
        HolyType::Word => Value::Str(String::new()),
        HolyType::Dogma => Value::Bool(false),
        HolyType::Void => Value::Void,
        HolyType::Custom(_) => Value::Void,
    }
}

pub fn get_field(val: &Value, field: &str) -> EvalResult {
    match val {
        Value::Scripture { fields, .. } => fields.get(field).cloned().ok_or_else(|| {
            builtin_sin("UndefinedField", format!("field '{}' not found", field))
        }),
        _ => Err(builtin_sin(
            "TypeError",
            "field access on a non-scripture value",
        )),
    }
}

pub fn eval_binop(op: &BinOp, l: Value, r: Value) -> EvalResult {
    use Value::*;

    let (l, r) = match (&l, &r) {
        (Int(a), Float(b)) => (Float(*a as f64), Float(*b)),
        (Float(a), Int(b)) => (Float(*a), Float(*b as f64)),
        _ => (l, r),
    };

    match op {
        BinOp::Add => match (&l, &r) {
            (Int(a), Int(b)) => Ok(Int(a + b)),
            (Float(a), Float(b)) => Ok(Float(a + b)),
            (Str(a), Str(b)) => Ok(Str(format!("{}{}", a, b))),
            _ => type_err("plus", &l, &r),
        },
        BinOp::Sub => match (&l, &r) {
            (Int(a), Int(b)) => Ok(Int(a - b)),
            (Float(a), Float(b)) => Ok(Float(a - b)),
            _ => type_err("minus", &l, &r),
        },
        BinOp::Mul => match (&l, &r) {
            (Int(a), Int(b)) => Ok(Int(a * b)),
            (Float(a), Float(b)) => Ok(Float(a * b)),
            _ => type_err("times", &l, &r),
        },
        BinOp::Div => match (&l, &r) {
            (Int(a), Int(b)) => {
                if *b == 0 {
                    Err(builtin_sin("DivisionByZero", "division by zero"))
                } else {
                    Ok(Int(a / b))
                }
            }
            (Float(a), Float(b)) => {
                if *b == 0.0 {
                    Err(builtin_sin("DivisionByZero", "division by zero"))
                } else {
                    Ok(Float(a / b))
                }
            }
            _ => type_err("over", &l, &r),
        },
        BinOp::Rem => match (&l, &r) {
            (Int(a), Int(b)) => {
                if *b == 0 {
                    Err(builtin_sin("DivisionByZero", "division by zero"))
                } else {
                    Ok(Int(a % b))
                }
            }
            _ => type_err("remainder", &l, &r),
        },
        BinOp::Eq => Ok(Bool(values_equal(&l, &r))),
        BinOp::Ne => Ok(Bool(!values_equal(&l, &r))),
        BinOp::Gt => numeric_cmp(&l, &r, |a, b| a > b),
        BinOp::Lt => numeric_cmp(&l, &r, |a, b| a < b),
        BinOp::Ge => numeric_cmp(&l, &r, |a, b| a >= b),
        BinOp::Le => numeric_cmp(&l, &r, |a, b| a <= b),
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => x == y,
        (Value::Float(x), Value::Float(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Void, Value::Void) => true,
        _ => false,
    }
}

fn numeric_cmp(l: &Value, r: &Value, f: impl Fn(f64, f64) -> bool) -> EvalResult {
    let lf = to_float(l).ok_or_else(|| invalid_numeric_comparison())?;
    let rf = to_float(r).ok_or_else(|| invalid_numeric_comparison())?;
    Ok(Value::Bool(f(lf, rf)))
}

fn to_float(v: &Value) -> Option<f64> {
    match v {
        Value::Int(n) => Some(*n as f64),
        Value::Float(f) => Some(*f),
        _ => None,
    }
}

fn invalid_numeric_comparison() -> HolyError {
    builtin_sin("TypeError", "invalid numeric comparison")
}

fn type_err(op: &str, l: &Value, r: &Value) -> EvalResult {
    Err(builtin_sin(
        "TypeError",
        format!(
            "operation '{}' not supported between {} and {}",
            op,
            value_type_name(l),
            value_type_name(r)
        ),
    ))
}
