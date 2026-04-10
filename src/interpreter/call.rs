use std::collections::HashMap;

use crate::ast::{Expr, HolyType};

use super::{
    builtins::{builtin_sin, call_builtin_fn, grace_absent, grace_granted, values_equal},
    EvalResult, HolyError, Interpreter, SalmDef, Value,
};

impl Interpreter {
    // ── Top-level dispatch ────────────────────────────────────────────────────

    pub(super) fn call_salm(
        &mut self,
        name:       &str,
        _type_args: &[HolyType],
        args:       Vec<Value>,
    ) -> EvalResult {
        // `args` builtin needs interpreter state — handle before the free fn
        if name == "args" {
            if !args.is_empty() {
                return Err(builtin_sin("InvalidArgumentCount", "'args' takes no offerings"));
            }
            let script_args: Vec<Value> =
                self.script_args.iter().map(|s| Value::Str(s.clone())).collect();
            return Ok(Value::Legion(script_args));
        }

        if let Some(result) = call_builtin_fn(name, args.clone()) {
            return result;
        }

        let def = self
            .salms
            .get(name)
            .ok_or_else(|| {
                builtin_sin(
                    "UndefinedSalm",
                    format!("the salm '{}' is unknown to the congregation", name),
                )
            })?
            .clone();

        // Build type param bindings: use explicit type_args if provided,
        // otherwise infer from the actual argument values.
        let bindings = if !_type_args.is_empty() && !def.type_params.is_empty() {
            def.type_params.iter().cloned()
                .zip(_type_args.iter().cloned())
                .collect()
        } else if !def.type_params.is_empty() {
            self.infer_type_bindings(&def.type_params, &def.params, &args)
        } else {
            std::collections::HashMap::new()
        };

        let salm_name = name.to_string();
        self.exec_salm_body(&def, None, args, bindings)
            .map_err(|e| e.push_frame(salm_name))
    }

    pub(super) fn call_method(
        &mut self,
        method:    &str,
        target:    Value,
        target_ty: Option<HolyType>,
        args:      Vec<Value>,
    ) -> EvalResult {
        if let Some(result) = self.call_builtin_method(method, &target, target_ty.as_ref(), &args)? {
            return Ok(result);
        }

        let type_name = match &target {
            Value::Scripture { type_name, .. } => type_name.clone(),
            other => {
                return Err(builtin_sin(
                    "TypeError",
                    format!(
                        "thou shalt not invoke rites upon '{}' — only scriptures may receive the sacred methods",
                        super::value::value_type_name(other)
                    ),
                ));
            }
        };

        let def = self
            .methods
            .get(&(method.to_string(), type_name.clone()))
            .ok_or_else(|| {
                builtin_sin("UndefinedMethod", format!("the rite '{}' is not known to the holy order", method))
            })?
            .clone();

        let bindings = if !def.type_params.is_empty() {
            self.infer_type_bindings(&def.type_params, &def.params, &args)
        } else {
            std::collections::HashMap::new()
        };

        let frame = format!("{} upon {}", method, type_name);
        self.exec_salm_body(&def, Some(target), args, bindings)
            .map_err(|e| e.push_frame(frame))
    }

    // ── Built-in method dispatch ──────────────────────────────────────────────

    fn call_builtin_method(
        &self,
        method:    &str,
        target:    &Value,
        target_ty: Option<&HolyType>,
        args:      &[Value],
    ) -> Result<Option<Value>, HolyError> {
        match target {
            Value::Str(s)        => self.call_word_method(method, s, args).map(Some),
            Value::Legion(items) => self.call_legion_method(method, items, target_ty, args).map(Some),
            _                    => Ok(None),
        }
    }

    // ── word methods ──────────────────────────────────────────────────────────

    fn call_word_method(&self, method: &str, target: &str, args: &[Value]) -> EvalResult {
        match method {
            "length" => {
                self.expect_builtin_arg_count(method, args, 0)?;
                Ok(Value::Int(target.chars().count() as i64))
            }

            "is_empty" => {
                self.expect_builtin_arg_count(method, args, 0)?;
                Ok(Value::Bool(target.is_empty()))
            }

            "at" => {
                self.expect_builtin_arg_count(method, args, 1)?;
                let index = self.expect_atom_arg(method, &args[0])?;
                let ch = target.chars().nth(index as usize).ok_or_else(|| {
                    builtin_sin(
                        "IndexOutOfBounds",
                        format!("the index {} strays beyond the sacred bounds", index),
                    )
                })?;
                Ok(Value::Str(ch.to_string()))
            }

            "slice" => {
                self.expect_builtin_arg_count(method, args, 2)?;
                let start = self.expect_atom_arg(method, &args[0])? as usize;
                let end   = self.expect_atom_arg(method, &args[1])? as usize;
                let chars: Vec<char> = target.chars().collect();
                if start > end || end > chars.len() {
                    return Err(builtin_sin(
                        "IndexOutOfBounds",
                        format!(
                            "the cut {}..{} reaches beyond the sacred bounds of a word of length {}",
                            start, end, chars.len()
                        ),
                    ));
                }
                Ok(Value::Str(chars[start..end].iter().collect()))
            }

            "contains" => {
                self.expect_builtin_arg_count(method, args, 1)?;
                let sub = self.expect_str_arg(method, &args[0])?;
                Ok(Value::Bool(target.contains(sub.as_str())))
            }

            "starts_with" => {
                self.expect_builtin_arg_count(method, args, 1)?;
                let prefix = self.expect_str_arg(method, &args[0])?;
                Ok(Value::Bool(target.starts_with(prefix.as_str())))
            }

            "ends_with" => {
                self.expect_builtin_arg_count(method, args, 1)?;
                let suffix = self.expect_str_arg(method, &args[0])?;
                Ok(Value::Bool(target.ends_with(suffix.as_str())))
            }

            "index_of" => {
                self.expect_builtin_arg_count(method, args, 1)?;
                let sub = self.expect_str_arg(method, &args[0])?;
                let result = target.find(sub.as_str()).map(|byte_idx| {
                    target[..byte_idx].chars().count() as i64
                });
                Ok(match result {
                    Some(i) => grace_granted(Value::Int(i)),
                    None    => grace_absent(),
                })
            }

            "to_upper" => {
                self.expect_builtin_arg_count(method, args, 0)?;
                Ok(Value::Str(target.to_uppercase()))
            }

            "to_lower" => {
                self.expect_builtin_arg_count(method, args, 0)?;
                Ok(Value::Str(target.to_lowercase()))
            }

            "trim" => {
                self.expect_builtin_arg_count(method, args, 0)?;
                Ok(Value::Str(target.trim().to_string()))
            }

            "replace" => {
                self.expect_builtin_arg_count(method, args, 2)?;
                let old = self.expect_str_arg(method, &args[0])?;
                let new = self.expect_str_arg(method, &args[1])?;
                Ok(Value::Str(target.replace(old.as_str(), new.as_str())))
            }

            "split" => {
                self.expect_builtin_arg_count(method, args, 1)?;
                let sep = self.expect_str_arg(method, &args[0])?;
                let parts: Vec<Value> =
                    target.split(sep.as_str()).map(|s| Value::Str(s.to_string())).collect();
                Ok(Value::Legion(parts))
            }

            _ => Err(builtin_sin(
                "UndefinedMethod",
                format!("the rite '{}' is not known to the holy order", method),
            )),
        }
    }

    // ── legion methods ────────────────────────────────────────────────────────

    fn call_legion_method(
        &self,
        method:    &str,
        items:     &[Value],
        target_ty: Option<&HolyType>,
        args:      &[Value],
    ) -> EvalResult {
        match method {
            "length" => {
                self.expect_builtin_arg_count(method, args, 0)?;
                Ok(Value::Int(items.len() as i64))
            }

            "is_empty" => {
                self.expect_builtin_arg_count(method, args, 0)?;
                Ok(Value::Bool(items.is_empty()))
            }

            "at" => {
                self.expect_builtin_arg_count(method, args, 1)?;
                let index = self.expect_atom_arg(method, &args[0])?;
                items.get(index as usize).cloned().ok_or_else(|| {
                    builtin_sin(
                        "IndexOutOfBounds",
                        format!("the index {} strays beyond the sacred bounds", index),
                    )
                })
            }

            "first" => {
                self.expect_builtin_arg_count(method, args, 0)?;
                Ok(match items.first() {
                    Some(v) => grace_granted(v.clone()),
                    None    => grace_absent(),
                })
            }

            "last" => {
                self.expect_builtin_arg_count(method, args, 0)?;
                Ok(match items.last() {
                    Some(v) => grace_granted(v.clone()),
                    None    => grace_absent(),
                })
            }

            "contains" => {
                self.expect_builtin_arg_count(method, args, 1)?;
                let found = items.iter().any(|item| values_equal(item, &args[0]));
                Ok(Value::Bool(found))
            }

            "index_of" => {
                self.expect_builtin_arg_count(method, args, 1)?;
                let pos = items.iter().position(|item| values_equal(item, &args[0]));
                Ok(match pos {
                    Some(i) => grace_granted(Value::Int(i as i64)),
                    None    => grace_absent(),
                })
            }

            "reverse" => {
                self.expect_builtin_arg_count(method, args, 0)?;
                let mut out = items.to_vec();
                out.reverse();
                Ok(Value::Legion(out))
            }

            "push" => {
                self.expect_builtin_arg_count(method, args, 1)?;
                if let Some(HolyType::Generic(name, type_args)) = target_ty {
                    if name == "legion" {
                        if let Some(inner_ty) = type_args.first() {
                            self.expect_type(inner_ty, &args[0], "this element is profane and unworthy of the legion")?;
                        }
                    }
                }
                let mut out = items.to_vec();
                out.push(args[0].clone());
                Ok(Value::Legion(out))
            }

            "slice" => {
                self.expect_builtin_arg_count(method, args, 2)?;
                let start = self.expect_atom_arg(method, &args[0])? as usize;
                let end   = self.expect_atom_arg(method, &args[1])? as usize;
                if start > end || end > items.len() {
                    return Err(builtin_sin(
                        "IndexOutOfBounds",
                        format!(
                            "the cut {}..{} reaches beyond the sacred bounds of a legion of length {}",
                            start, end, items.len()
                        ),
                    ));
                }
                Ok(Value::Legion(items[start..end].to_vec()))
            }

            "concat" => {
                self.expect_builtin_arg_count(method, args, 1)?;
                if let Some(ty) = target_ty {
                    self.expect_type(ty, &args[0], "this legion is unworthy of the sacred union")?;
                }
                let other = match &args[0] {
                    Value::Legion(other) => other,
                    value => {
                        return Err(builtin_sin(
                            "TypeError",
                            format!(
                                "the rite 'concat' demands a legion, but received the profane {}",
                                self.describe_value(value)
                            ),
                        ));
                    }
                };
                let mut out = items.to_vec();
                out.extend(other.iter().cloned());
                Ok(Value::Legion(out))
            }

            _ => Err(builtin_sin(
                "UndefinedMethod",
                format!("the rite '{}' is not known to the holy order", method),
            )),
        }
    }

    // ── Salm body execution ───────────────────────────────────────────────────

    pub(super) fn exec_salm_body(
        &mut self,
        def:      &SalmDef,
        self_val: Option<Value>,
        args:     Vec<Value>,
        bindings: std::collections::HashMap<String, crate::ast::HolyType>,
    ) -> EvalResult {
        if args.len() != def.params.len() {
            return Err(builtin_sin(
                "InvalidArgumentCount",
                format!(
                    "the salm demands {} offering(s), but received {}",
                    def.params.len(),
                    args.len()
                ),
            ));
        }

        // Push type param bindings so that resolve_type() and resolve_type_args()
        // work correctly throughout this salm's body.
        self.push_type_bindings(bindings);
        let saved = self.env.enter_call();

        if let Some(val) = self_val {
            let self_ty = self.infer_type_from_value(&val);
            self.env.define("its", self_ty, Some(val));
        }

        for ((pname, pty), val) in def.params.iter().zip(args) {
            let resolved = self.resolve_type(pty);
            if self.is_concrete_type(&resolved) {
                self.expect_type(&resolved, &val, &format!("the offering for parameter '{}' is profane and rejected", pname))?;
            }
            self.env.define(pname, resolved, Some(val));
        }

        let result = self.exec_stmts(&def.body);
        self.env.exit_call(saved);
        self.pop_type_bindings();

        match result {
            Ok(()) => {
                let value = Value::Void;
                let resolved_ret = self.resolve_type(&def.ret_type);
                if self.is_concrete_type(&resolved_ret) {
                    self.expect_type(&resolved_ret, &value, "the salm's revelation is profane — its return defies the holy covenant")?;
                }
                Ok(value)
            }
            Err(HolyError::Return(v)) => {
                let resolved_ret = self.resolve_type(&def.ret_type);
                if self.is_concrete_type(&resolved_ret) {
                    self.expect_type(&resolved_ret, &v, "invalid salm return")?;
                }
                Ok(v)
            }
            Err(e) => Err(e),
        }
    }

    // ── Sin instantiation ─────────────────────────────────────────────────────

    pub(super) fn build_sin_fields(
        &mut self,
        sin_type: &str,
        args:     &[Expr],
    ) -> Result<HashMap<String, Value>, HolyError> {
        let def = self
            .sins
            .get(sin_type)
            .ok_or_else(|| {
                builtin_sin(
                    "UndefinedSin",
                    format!(
                        "the sin '{}' has never been declared — thou canst not transgress what is unknown",
                        sin_type
                    ),
                )
            })?
            .clone();

        if args.len() != def.len() {
            return Err(builtin_sin(
                "InvalidArgumentCount",
                format!(
                    "the sin '{}' demands {} field(s) to be committed, but {} were offered",
                    sin_type,
                    def.len(),
                    args.len()
                ),
            ));
        }

        let mut fields = HashMap::new();
        for ((fname, field_ty), arg) in def.iter().zip(args.iter()) {
            let v = self.eval_expr(arg)?;
            self.expect_type(
                field_ty,
                &v,
                &format!("the offering for sin field '{}.{}' is profane and rejected", sin_type, fname),
            )?;
            fields.insert(fname.clone(), v);
        }
        Ok(fields)
    }

    // ── Argument validation helpers ───────────────────────────────────────────

    pub(super) fn expect_builtin_arg_count(
        &self,
        method:   &str,
        args:     &[Value],
        expected: usize,
    ) -> Result<(), HolyError> {
        if args.len() == expected {
            Ok(())
        } else {
            Err(builtin_sin(
                "InvalidArgumentCount",
                format!(
                    "the rite '{}' demands {} offering(s), but received {}",
                    method, expected, args.len()
                ),
            ))
        }
    }

    pub(super) fn expect_atom_arg(&self, method: &str, value: &Value) -> Result<i64, HolyError> {
        match value {
            Value::Int(n) if *n >= 0 => Ok(*n),
            Value::Int(n) => Err(builtin_sin(
                "IndexOutOfBounds",
                format!("the index {} strays beyond the sacred bounds", n),
            )),
            other => Err(builtin_sin(
                "TypeError",
                format!(
                    "the rite '{}' demands a sacred atom as index, but received the profane {}",
                    method,
                    self.describe_value(other)
                ),
            )),
        }
    }

    fn expect_str_arg(&self, method: &str, value: &Value) -> Result<String, HolyError> {
        match value {
            Value::Str(s) => Ok(s.clone()),
            other => Err(builtin_sin(
                "TypeError",
                format!(
                    "the rite '{}' demands a sacred word, but received the profane {}",
                    method,
                    self.describe_value(other)
                ),
            )),
        }
    }
}
