use std::collections::HashMap;

use crate::ast::{Expr, HolyType};

use super::{
    builtins::builtin_sin,
    ops::{eval_binop, eval_literal, get_field},
    value::value_type_name,
    EvalResult, Interpreter, Value,
};

impl Interpreter {
    pub(super) fn eval_expr(&mut self, expr: &Expr) -> EvalResult {
        match expr {
            Expr::Lit(lit) => Ok(eval_literal(lit)),

            Expr::Var(name) => self.eval_var(name),

            Expr::Negate(inner) => {
                match self.eval_expr(inner)? {
                    Value::Int(n)   => Ok(Value::Int(-n)),
                    Value::Float(x) => Ok(Value::Float(-x)),
                    other => Err(builtin_sin(
                        "TypeError",
                        format!("'negate' may only invert that which is of atom or fractional — {} is profane", value_type_name(&other)),
                    )),
                }
            }

            Expr::BinOp { op, left, right } => {
                let lv = self.eval_expr(left)?;
                let rv = self.eval_expr(right)?;
                eval_binop(op, lv, rv)
            }

            Expr::FnCall { name, args, type_args } => {
                let vals = self.eval_args(args)?;
                self.call_salm(name, type_args, vals)
            }

            Expr::MethodCall { method, target, args } => {
                let tv        = self.eval_expr(target)?;
                let target_ty = self.expr_declared_type(target);
                let vals      = self.eval_args(args)?;
                self.call_method(method, tv, target_ty, vals)
            }

            Expr::Manifest { scripture, args } => self.eval_manifest(scripture, args),

            Expr::ManifestVariant { variant, covenant, type_args, args } => {
                self.eval_manifest_variant(variant, covenant, type_args, args)
            }

            Expr::TypedUnitVariant { variant, covenant, type_args } => {
                self.eval_typed_unit_variant(variant, covenant, type_args)
            }

            Expr::FieldAccess { field, object } => {
                let obj = self.eval_expr(object)?;
                get_field(&obj, field)
            }

            Expr::SelfFieldAccess { field } => {
                let its = self.env.get("its").ok_or_else(|| {
                    builtin_sin("InvalidContext", "'its' is forbidden outside a method_salm — the sacred self may only be accessed within its own rites")
                })?;
                get_field(&its, field)
            }
        }
    }

    pub(super) fn eval_args(&mut self, args: &[Expr]) -> Result<Vec<Value>, super::HolyError> {
        args.iter().map(|a| self.eval_expr(a)).collect()
    }

    // ── Atom / variable lookup ────────────────────────────────────────────────

    fn eval_var(&self, name: &str) -> EvalResult {
        if let Some(value) = self.env.get(name) {
            return Ok(value);
        }
        if let Some((covenant, fields)) = self.covenant_variants.get(name) {
            if !fields.is_empty() {
                return Err(builtin_sin(
                    "InvalidArgumentCount",
                    format!("the variant '{}' demands its fields — thou must use 'manifest {} praying ...'", name, name),
                ));
            }
            return Ok(Value::CovenantVariant {
                covenant:  covenant.clone(),
                type_args: vec![],
                variant:   name.to_string(),
                fields:    Vec::new(),
            });
        }
        Err(builtin_sin(
            "UndefinedVariable",
            format!("'{}' has not been anointed — it walks in darkness, unknown to the congregation", name),
        ))
    }

    // ── Manifest (scripture or data variant) ──────────────────────────────────

    fn eval_manifest(&mut self, name: &str, args: &[Expr]) -> EvalResult {
        // Data variant takes priority over scripture
        if let Some((covenant, field_defs)) = self.covenant_variants.get(name).cloned() {
            return self.eval_manifest_covenant_variant(name, &covenant, &field_defs, args);
        }

        let def = self.scriptures.get(name)
            .ok_or_else(|| builtin_sin("UndefinedScripture", format!("'{}' is neither a scripture nor a covenant variant — it cannot be manifested", name)))?
            .clone();

        if args.len() != def.fields.len() {
            return Err(builtin_sin(
                "InvalidArgumentCount",
                format!("the scripture '{}' demands {} field(s), but {} were offered", name, def.fields.len(), args.len()),
            ));
        }

        let mut fields = HashMap::new();
        for ((fname, field_ty), arg) in def.fields.iter().zip(args.iter()) {
            let v = self.eval_expr(arg)?;
            let resolved_field_ty = self.resolve_type(field_ty);
            if self.is_concrete_type(&resolved_field_ty) {
                self.expect_type(&resolved_field_ty, &v, &format!("the offering for field '{}.{}' is profane and rejected", name, fname))?;
            }
            fields.insert(fname.clone(), v);
        }

        // Populate type_args from current bindings for this scripture's type params.
        let type_args = self.resolve_type_args(&def.type_params);

        Ok(Value::Scripture { type_name: name.to_string(), type_args, fields })
    }

    fn eval_manifest_covenant_variant(
        &mut self,
        variant:    &str,
        covenant:   &str,
        field_defs: &[(String, HolyType)],
        args:       &[Expr],
    ) -> EvalResult {
        if field_defs.is_empty() {
            return Err(builtin_sin(
                "InvalidArgumentCount",
                format!("the variant '{}' bears no fields — thou shalt not use 'manifest' to summon it", variant),
            ));
        }
        if args.len() != field_defs.len() {
            return Err(builtin_sin(
                "InvalidArgumentCount",
                format!("the variant '{}' demands {} field(s), but received {}", variant, field_defs.len(), args.len()),
            ));
        }
        let mut values = Vec::new();
        for ((fname, field_ty), arg) in field_defs.iter().zip(args.iter()) {
            let v = self.eval_expr(arg)?;
            self.expect_type(field_ty, &v, &format!("the offering for variant field '{}.{}' is profane and rejected", variant, fname))?;
            values.push(v);
        }
        Ok(Value::CovenantVariant { covenant: covenant.to_string(), type_args: vec![], variant: variant.to_string(), fields: values })
    }

    // ── Explicit covenant variant construction ────────────────────────────────

    pub(super) fn eval_manifest_variant(
        &mut self,
        variant:   &str,
        covenant:  &str,
        type_args: &[HolyType],
        args:      &[Expr],
    ) -> EvalResult {
        if !self.covenants.contains_key(covenant) {
            return Err(builtin_sin("UndefinedType", format!("the covenant '{}' has never been proclaimed — it is unknown to this congregation", covenant)));
        }
        match self.covenant_variants.get(variant) {
            Some((cov, _)) if cov != covenant => {
                return Err(builtin_sin(
                    "InvalidDiscern",
                    format!("the variant '{}' does not belong to the covenant '{}' — this is heresy", variant, covenant),
                ));
            }
            None => {
                return Err(builtin_sin("UndefinedVariable", format!("the variant '{}' is not known to the congregation", variant)));
            }
            _ => {}
        }

        match covenant {
            "grace"   => self.eval_grace_variant(variant, type_args, args),
            "verdict" => self.eval_verdict_variant(variant, type_args, args),
            _         => self.eval_user_covenant_variant(variant, covenant, args),
        }
    }

    fn eval_grace_variant(&mut self, variant: &str, type_args: &[HolyType], args: &[Expr]) -> EvalResult {
        use super::generics::make_granted;

        if variant != "granted" {
            return Err(builtin_sin("TypeError",
                format!("'{}' is a pure and empty vessel of grace — thou shalt not profane it with 'manifest'", variant)));
        }
        if args.len() != 1 {
            return Err(builtin_sin("InvalidArgumentCount", "grace::granted demands exactly 1 offering — no more, no less"));
        }
        let val = self.eval_expr(&args[0])?;
        match type_args.first() {
            Some(ty) if self.is_concrete_type(ty) => {
                make_granted(ty, val, |ty, v| self.value_matches_type(ty, v))
            }
            _ => Ok(Value::CovenantVariant { covenant: "grace".into(), type_args: vec![], variant: "granted".into(), fields: vec![val] }),
        }
    }

    fn eval_verdict_variant(&mut self, variant: &str, type_args: &[HolyType], args: &[Expr]) -> EvalResult {
        use super::generics::make_verdict_variant;

        if args.len() != 1 {
            return Err(builtin_sin("InvalidArgumentCount",
                format!("verdict::{} demands exactly 1 offering — no more, no less", variant)));
        }
        let val      = self.eval_expr(&args[0])?;
        let field_ty = match variant {
            "righteous" => type_args.get(0),
            "condemned" => type_args.get(1),
            _ => return Err(builtin_sin("TypeError", format!("'{}' is not among the sacred variants of verdict", variant))),
        };
        match field_ty {
            Some(ty) if self.is_concrete_type(ty) => {
                make_verdict_variant(variant, ty, val, |ty, v| self.value_matches_type(ty, v))
            }
            _ => Ok(Value::CovenantVariant { covenant: "verdict".into(), type_args: vec![], variant: variant.into(), fields: vec![val] }),
        }
    }

    fn eval_user_covenant_variant(&mut self, variant: &str, covenant: &str, args: &[Expr]) -> EvalResult {
        let (_, field_defs) = self.covenant_variants.get(variant)
            .ok_or_else(|| builtin_sin("UndefinedVariable", format!("the variant '{}' is not known to the congregation", variant)))?
            .clone();

        if args.len() != field_defs.len() {
            return Err(builtin_sin("InvalidArgumentCount",
                format!("the variant '{}' demands {} field(s), but {} were offered", variant, field_defs.len(), args.len())));
        }

        let mut values = Vec::new();
        for ((fname, field_ty), arg) in field_defs.iter().zip(args.iter()) {
            let v = self.eval_expr(arg)?;
            // Skip type check for abstract type params (Custom names not registered anywhere)
            let is_abstract = matches!(field_ty, HolyType::Custom(n) if !self.scriptures.contains_key(n) && !self.covenants.contains_key(n));
            if !is_abstract {
                self.expect_type(field_ty, &v, &format!("the offering for variant field '{}.{}' is profane and rejected", variant, fname))?;
            }
            values.push(v);
        }
        Ok(Value::CovenantVariant { covenant: covenant.into(), type_args: vec![], variant: variant.into(), fields: values })
    }

    pub(super) fn eval_typed_unit_variant(
        &mut self,
        variant:   &str,
        covenant:  &str,
        _type_args: &[HolyType],
    ) -> EvalResult {
        if !self.covenants.contains_key(covenant) {
            return Err(builtin_sin("UndefinedType", format!("the covenant '{}' has never been proclaimed — it is unknown to this congregation", covenant)));
        }
        match self.covenant_variants.get(variant) {
            Some((cov, fields)) if cov == covenant && fields.is_empty() => {
                Ok(Value::CovenantVariant { covenant: covenant.into(), type_args: vec![], variant: variant.into(), fields: vec![] })
            }
            Some((cov, _)) if cov != covenant => Err(builtin_sin(
                "InvalidDiscern",
                format!("the variant '{}' does not belong to the covenant '{}' — this is heresy", variant, covenant),
            )),
            Some((_, fields)) if !fields.is_empty() => Err(builtin_sin(
                "InvalidArgumentCount",
                format!("the variant '{}' carries sacred data — thou must use 'manifest {} of {}' to bring it forth", variant, variant, covenant),
            )),
            _ => Err(builtin_sin("UndefinedVariable", format!("the variant '{}' is not known to the congregation", variant))),
        }
    }

    // ── Type introspection on expressions ────────────────────────────────────

    pub(super) fn expr_declared_type(&self, expr: &Expr) -> Option<HolyType> {
        match expr {
            Expr::Var(name) => self.env.get_type(name),
            Expr::FieldAccess { field, object } => {
                let object_ty = self.expr_declared_type(object)?;
                self.field_type_from_type(&object_ty, field)
            }
            Expr::SelfFieldAccess { field } => {
                let its_ty = self.env.get_type("its")?;
                self.field_type_from_type(&its_ty, field)
            }
            _ => None,
        }
    }

    fn field_type_from_type(&self, object_ty: &HolyType, field: &str) -> Option<HolyType> {
        match object_ty {
            HolyType::Custom(name) | HolyType::Generic(name, _) => {
                self.scriptures
                    .get(name)
                    .and_then(|def| def.fields.iter().find(|(fname, _)| fname == field))
                    .map(|(_, field_ty)| field_ty.clone())
            }
            _ => None,
        }
    }
}
