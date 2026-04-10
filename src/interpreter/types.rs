use crate::ast::HolyType;

use super::{builtins::builtin_sin, HolyError, Interpreter, Value};

impl Interpreter {
    // ── Type matching ─────────────────────────────────────────────────────────

    pub(super) fn value_matches_type(&self, ty: &HolyType, value: &Value) -> bool {
        match ty {
            HolyType::Atom       => matches!(value, Value::Int(_)),
            HolyType::Fractional => matches!(value, Value::Float(_)),
            HolyType::Word       => matches!(value, Value::Str(_)),
            HolyType::Dogma      => matches!(value, Value::Bool(_)),
            HolyType::Void       => matches!(value, Value::Void),

            HolyType::Custom(name) => match value {
                Value::Scripture { type_name, .. }               => type_name == name,
                Value::CovenantVariant { covenant, .. }          => covenant == name,
                _ => false,
            },

            HolyType::Generic(name, args) => match (name.as_str(), value) {
                ("grace", Value::CovenantVariant { covenant, variant, fields, .. }) if covenant == "grace" => {
                    match (args.first(), variant.as_str()) {
                        (Some(inner), "granted") => fields.len() == 1 && self.value_matches_type(inner, &fields[0]),
                        (_, "absent")            => fields.is_empty(),
                        _                        => false,
                    }
                }
                ("verdict", Value::CovenantVariant { covenant, variant, fields, .. }) if covenant == "verdict" => {
                    match (args.first(), args.get(1), variant.as_str()) {
                        (Some(ok_ty), _, "righteous")  => fields.len() == 1 && self.value_matches_type(ok_ty, &fields[0]),
                        (_, Some(err_ty), "condemned") => fields.len() == 1 && self.value_matches_type(err_ty, &fields[0]),
                        _                              => false,
                    }
                }
                ("legion", Value::Legion(items)) => match args.as_slice() {
                    [inner_ty] => items.iter().all(|item| self.value_matches_type(inner_ty, item)),
                    // no type arg (inferred from empty literal) — accept any legion
                    [] => true,
                    _  => false,
                },
                // User-defined generic types (e.g. `Stack of atom`):
                // A value with empty type_args is "pending" — accepted anywhere by name.
                // A value with resolved type_args must match exactly.
                _ => match value {
                    Value::Scripture { type_name, type_args: val_targs, .. } => {
                        type_name == name && (val_targs.is_empty() || val_targs == args)
                    }
                    Value::CovenantVariant { covenant, type_args: val_targs, .. } => {
                        covenant == name && (val_targs.is_empty() || val_targs == args)
                    }
                    _ => false,
                },
            },
        }
    }

    pub(super) fn expect_type(&self, ty: &HolyType, value: &Value, context: &str) -> Result<(), HolyError> {
        if self.value_matches_type(ty, value) {
            Ok(())
        } else {
            Err(builtin_sin(
                "TypeError",
                format!(
                    "{}: the holy scripture demands {}, but received the profane {}",
                    context,
                    self.describe_type(ty),
                    self.describe_value(value),
                ),
            ))
        }
    }

    // ── Type existence checks ─────────────────────────────────────────────────

    /// Lenient check used at runtime — unregistered Custom names are treated as
    /// abstract type params (strict validation already ran during `validate_declared_types`).
    pub(super) fn ensure_type_exists(&self, ty: &HolyType) -> Result<(), HolyError> {
        self.ensure_type_exists_lenient(ty)
    }

    fn ensure_type_exists_lenient(&self, ty: &HolyType) -> Result<(), HolyError> {
        match ty {
            HolyType::Custom(_) => Ok(()), // abstract params are allowed
            HolyType::Generic(name, args) => {
                self.ensure_custom_type_exists(name)?;
                for arg in args {
                    self.ensure_type_exists_lenient(arg)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Strict check used during declaration validation; `type_params` are allowed as abstracts.
    pub(super) fn ensure_type_exists_with_params(
        &self,
        ty: &HolyType,
        type_params: &[String],
    ) -> Result<(), HolyError> {
        match ty {
            HolyType::Custom(name) => {
                if !type_params.contains(name) {
                    self.ensure_custom_type_exists(name)?;
                }
            }
            HolyType::Generic(name, args) => {
                self.ensure_custom_type_exists(name)?;
                for arg in args {
                    self.ensure_type_exists_with_params(arg, type_params)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub(super) fn ensure_custom_type_exists(&self, name: &str) -> Result<(), HolyError> {
        if name == "legion"
            || self.scriptures.contains_key(name)
            || self.covenants.contains_key(name)
            || self.sins.contains_key(name)
        {
            Ok(())
        } else {
            Err(builtin_sin("UndefinedType", format!("the type '{}' has never been proclaimed to this congregation", name)))
        }
    }

    // ── Type predicates ───────────────────────────────────────────────────────

    /// Returns true if `ty` is a fully concrete, registered type (not an abstract param).
    pub(super) fn is_concrete_type(&self, ty: &HolyType) -> bool {
        match ty {
            HolyType::Custom(name) => {
                self.scriptures.contains_key(name) || self.covenants.contains_key(name)
            }
            HolyType::Generic(name, args) => {
                (self.scriptures.contains_key(name) || self.covenants.contains_key(name))
                    && args.iter().all(|a| self.is_concrete_type(a))
            }
            _ => true, // primitives are always concrete
        }
    }

    /// Returns true if `ty` references any abstract type param from `type_params`.
    pub(super) fn is_abstract_type(&self, ty: &HolyType, type_params: &[String]) -> bool {
        if type_params.is_empty() { return false; }
        match ty {
            HolyType::Custom(name)        => type_params.contains(name),
            HolyType::Generic(_, args)    => args.iter().any(|a| self.is_abstract_type(a, type_params)),
            _                             => false,
        }
    }

    // ── Type inference & description ──────────────────────────────────────────

    pub(super) fn infer_type_from_value(&self, value: &Value) -> HolyType {
        match value {
            Value::Int(_)                           => HolyType::Atom,
            Value::Float(_)                         => HolyType::Fractional,
            Value::Str(_)                           => HolyType::Word,
            Value::Bool(_)                          => HolyType::Dogma,
            Value::Legion(items)                    => {
                // Peek at the first element to infer the inner type.
                // Empty legions get no type arg (accepted by value_matches_type).
                let inner = items.first()
                    .map(|v| vec![self.infer_type_from_value(v)])
                    .unwrap_or_default();
                HolyType::Generic("legion".into(), inner)
            }
            Value::Void                                         => HolyType::Void,
            Value::CovenantVariant { covenant, type_args, .. } => {
                if type_args.is_empty() {
                    HolyType::Custom(covenant.clone())
                } else {
                    HolyType::Generic(covenant.clone(), type_args.clone())
                }
            }
            Value::Scripture { type_name, type_args, .. } => {
                if type_args.is_empty() {
                    HolyType::Custom(type_name.clone())
                } else {
                    HolyType::Generic(type_name.clone(), type_args.clone())
                }
            }
        }
    }

    pub(super) fn describe_type(&self, ty: &HolyType) -> String {
        match ty {
            HolyType::Atom              => "atom".into(),
            HolyType::Fractional        => "fractional".into(),
            HolyType::Word              => "word".into(),
            HolyType::Dogma             => "dogma".into(),
            HolyType::Void              => "void".into(),
            HolyType::Custom(name)      => name.clone(),
            HolyType::Generic(name, args) => {
                let args_str = args.iter().map(|a| self.describe_type(a)).collect::<Vec<_>>().join(", ");
                format!("{} of {}", name, args_str)
            }
        }
    }

    pub(super) fn describe_value(&self, value: &Value) -> String {
        match value {
            Value::Int(_)                           => "atom".into(),
            Value::Float(_)                         => "fractional".into(),
            Value::Str(_)                           => "word".into(),
            Value::Bool(_)                          => "dogma".into(),
            Value::Legion(_)                        => "legion".into(),
            Value::Void                             => "void".into(),
            Value::CovenantVariant { covenant, .. } => covenant.clone(),
            Value::Scripture { type_name, .. }      => type_name.clone(),
        }
    }
}
