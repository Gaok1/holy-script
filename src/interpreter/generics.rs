/// Built-in generic covenants: `grace of T` and `verdict of T, E`.
///
/// These are pre-registered at interpreter startup and receive first-class
/// type checking (unlike user-defined generic covenants, which are erased).
use std::collections::HashMap;

use crate::ast::{CovenantVariantDecl, HolyType};

use super::{HolyError, Value};
use super::builtins::builtin_sin;

/// Returns the pre-registered covenants and their variant maps for
/// `grace` and `verdict`.
pub fn builtin_covenants() -> (
    HashMap<String, Vec<CovenantVariantDecl>>,
    HashMap<String, (String, Vec<(String, HolyType)>)>,
) {
    let mut covenants: HashMap<String, Vec<CovenantVariantDecl>> = HashMap::new();
    let mut covenant_variants: HashMap<String, (String, Vec<(String, HolyType)>)> = HashMap::new();

    // grace of T  →  granted (data, 1 field) | absent (unit)
    // Fields are intentionally empty here; the type arg is enforced at the call site.
    covenants.insert("grace".into(), vec![
        CovenantVariantDecl { name: "granted".into(), fields: vec![] },
        CovenantVariantDecl { name: "absent".into(),  fields: vec![] },
    ]);
    covenant_variants.insert("granted".into(), ("grace".into(), vec![]));
    covenant_variants.insert("absent".into(),  ("grace".into(), vec![]));

    // verdict of T, E  →  righteous (data, 1 field) | condemned (data, 1 field)
    covenants.insert("verdict".into(), vec![
        CovenantVariantDecl { name: "righteous".into(), fields: vec![] },
        CovenantVariantDecl { name: "condemned".into(), fields: vec![] },
    ]);
    covenant_variants.insert("righteous".into(), ("verdict".into(), vec![]));
    covenant_variants.insert("condemned".into(), ("verdict".into(), vec![]));

    (covenants, covenant_variants)
}

/// Default value for `grace of T` → `absent` (no inner value).
pub fn default_grace() -> Value {
    Value::CovenantVariant {
        covenant:  "grace".into(),
        type_args: vec![],
        variant:   "absent".into(),
        fields:    vec![],
    }
}

/// Default value for `verdict of T, E` → `condemned` with an empty word.
pub fn default_verdict() -> Value {
    Value::CovenantVariant {
        covenant:  "verdict".into(),
        type_args: vec![],
        variant:   "condemned".into(),
        fields:    vec![Value::Str(String::new())],
    }
}

/// Instantiate a `grace` data variant (`granted`) with type-checked inner value.
pub fn make_granted(inner_ty: &HolyType, value: Value, check: impl Fn(&HolyType, &Value) -> bool)
    -> Result<Value, HolyError>
{
    if !check(inner_ty, &value) {
        return Err(builtin_sin(
            "TypeError",
            format!("'granted' expects a value of the declared inner type"),
        ));
    }
    Ok(Value::CovenantVariant {
        covenant:  "grace".into(),
        type_args: vec![inner_ty.clone()],
        variant:   "granted".into(),
        fields:    vec![value],
    })
}

/// Instantiate a `verdict` data variant with type-checked inner value.
/// `which` is `"righteous"` or `"condemned"`, `field_ty` is the matching type param.
pub fn make_verdict_variant(
    which: &str,
    field_ty: &HolyType,
    value: Value,
    check: impl Fn(&HolyType, &Value) -> bool,
) -> Result<Value, HolyError> {
    if !check(field_ty, &value) {
        return Err(builtin_sin(
            "TypeError",
            format!("'{}' expects a value of the declared type", which),
        ));
    }
    Ok(Value::CovenantVariant {
        covenant:  "verdict".into(),
        type_args: vec![field_ty.clone()],
        variant:   which.into(),
        fields:    vec![value],
    })
}
