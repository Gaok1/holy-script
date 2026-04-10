use crate::ast::{DiscernBranch, Expr, SinHandler, Stmt};

use super::{
    builtins::builtin_sin,
    ops::is_truthy,
    value::value_type_name,
    ExecResult, HolyError, HolyType, Interpreter, Value,
};

impl Interpreter {
    pub(super) fn exec_stmts(&mut self, stmts: &[Stmt]) -> ExecResult {
        for stmt in stmts {
            self.exec_stmt(stmt)?;
        }
        Ok(())
    }

    pub(super) fn exec_stmt(&mut self, stmt: &Stmt) -> ExecResult {
        match stmt {
            Stmt::DeclNoVal { name, ty: Some(ty) } => {
                self.ensure_type_exists(ty)?;
                self.env.define(name, ty.clone(), None);
            }

            // `let there be x` — untyped; type locked on first `become`
            Stmt::DeclNoVal { name, ty: None } => {
                self.env.define_untyped(name);
            }

            Stmt::DeclVal { name, ty, val } => {
                self.ensure_type_exists(ty)?;
                let v = self.eval_expr(val)?;
                self.expect_type(ty, &v, &format!("the offering for '{}' is profane and rejected", name))?;
                self.env.define(name, ty.clone(), Some(v));
            }

            // `let there x be expr` — infer type from the expression value
            Stmt::DeclInfer { name, val } => {
                let v  = self.eval_expr(val)?;
                let ty = self.infer_type_from_value(&v);
                self.env.define(name, ty, Some(v));
            }

            Stmt::Assign { name, val } => {
                let v = self.eval_expr(val)?;
                match self.env.get_binding_state(name) {
                    // typed variable — enforce type
                    Some(Some(var_ty)) => {
                        self.expect_type(&var_ty, &v, &format!("the offering for '{}' is profane and rejected", name))?;
                        self.env.assign(name, v);
                    }
                    // untyped variable — lock type on first assignment (divine grace)
                    Some(None) => {
                        let ty = self.infer_type_from_value(&v);
                        self.env.lock_type(name, ty, v);
                    }
                    // variable doesn't exist — infer and create (flexible `become`)
                    None => {
                        let inferred_ty = self.infer_type_from_value(&v);
                        self.env.define(name, inferred_ty, Some(v));
                    }
                }
            }

            Stmt::FnCallStmt { name, args, type_args } => {
                let vals = self.eval_args(args)?;
                self.call_salm(name, type_args, vals)?;
            }

            Stmt::MethodCallStmt { method, target, args } => {
                let target_val = self.eval_expr(target)?;
                let target_ty  = self.expr_declared_type(target);
                let vals       = self.eval_args(args)?;
                self.call_method(method, target_val, target_ty, vals)?;
            }

            Stmt::Reveal(expr) => {
                let v = self.eval_expr(expr)?;
                return Err(HolyError::Return(v));
            }

            Stmt::Conditional { branches, otherwise } => {
                self.exec_conditional(branches, otherwise)?;
            }

            Stmt::Litany { cond, body } => {
                self.exec_litany(cond, body)?;
            }

            Stmt::Forsake => return Err(HolyError::Break),
            Stmt::Ascend  => return Err(HolyError::Continue),

            Stmt::Confess { try_block, handlers, absolve } => {
                self.exec_confess(try_block, handlers, absolve)?;
            }

            Stmt::Discern { target, branches, otherwise } => {
                self.exec_discern(target, branches, otherwise)?;
            }

            Stmt::Transgress { sin_type, args } => {
                let fields = self.build_sin_fields(sin_type, args)?;
                return Err(HolyError::Sin { type_name: sin_type.clone(), fields, stack_trace: vec![] });
            }
        }
        Ok(())
    }

    // ── Control flow helpers ──────────────────────────────────────────────────

    fn exec_conditional(
        &mut self,
        branches: &[(Expr, Vec<Stmt>)],
        otherwise: &Option<Vec<Stmt>>,
    ) -> ExecResult {
        for (cond, body) in branches {
            let cv = self.eval_expr(cond)?;
            if is_truthy(&cv) {
                self.env.push();
                let r = self.exec_stmts(body);
                self.env.pop();
                return r;
            }
        }
        if let Some(else_body) = otherwise {
            self.env.push();
            let r = self.exec_stmts(else_body);
            self.env.pop();
            return r;
        }
        Ok(())
    }

    fn exec_litany(&mut self, cond: &Expr, body: &[Stmt]) -> ExecResult {
        loop {
            let cv = self.eval_expr(cond)?;
            if !is_truthy(&cv) { break; }

            self.env.push();
            let r = self.exec_stmts(body);
            self.env.pop();

            match r {
                Ok(())                  => {}
                Err(HolyError::Break)   => break,
                Err(HolyError::Continue)=> continue,
                Err(e)                  => return Err(e),
            }
        }
        Ok(())
    }

    pub(super) fn exec_confess(
        &mut self,
        try_block: &[Stmt],
        handlers:  &[SinHandler],
        absolve:   &Option<Vec<Stmt>>,
    ) -> ExecResult {
        self.env.push();
        let try_result = self.exec_stmts(try_block);
        self.env.pop();

        let after = self.handle_sin(try_result, handlers);

        if let Some(abs) = absolve {
            let abs = abs.clone();
            self.env.push();
            let ar = self.exec_stmts(&abs);
            self.env.pop();
            if ar.is_err() {
                return ar;
            }
        }

        after
    }

    fn handle_sin(&mut self, try_result: ExecResult, handlers: &[SinHandler]) -> ExecResult {
        match try_result {
            Err(HolyError::Sin { ref type_name, ref fields, .. }) => {
                if let Some(h) = handlers.iter().find(|h| &h.sin_type == type_name) {
                    let sin_val = Value::Scripture {
                        type_name: type_name.clone(),
                        type_args: vec![],
                        fields:    fields.clone(),
                    };
                    let body = h.body.clone();
                    let bind = h.binding.clone();

                    self.env.push();
                    if let Some(b) = &bind {
                        self.env.define(b, HolyType::Custom(type_name.clone()), Some(sin_val));
                    }
                    let r = self.exec_stmts(&body);
                    self.env.pop();
                    r
                } else {
                    try_result
                }
            }
            other => other,
        }
    }

    pub(super) fn exec_discern(
        &mut self,
        target:    &Expr,
        branches:  &[DiscernBranch],
        otherwise: &Option<Vec<Stmt>>,
    ) -> ExecResult {
        let target_value = self.eval_expr(target)?;

        let (matched_variant, variant_fields) = match target_value {
            Value::CovenantVariant { variant, fields, .. } => (variant, fields),
            other => {
                return Err(builtin_sin(
                    "InvalidDiscern",
                    format!("'discern' demands a covenant variant, but received the profane {}", value_type_name(&other)),
                ));
            }
        };

        if let Some(branch) = branches.iter().find(|b| b.variant == matched_variant) {
            if !branch.bindings.is_empty() && branch.bindings.len() != variant_fields.len() {
                return Err(builtin_sin(
                    "InvalidDiscern",
                    format!(
                        "the variant '{}' bears {} field(s) but 'bearing' names {} binding(s) — this is heresy",
                        matched_variant, variant_fields.len(), branch.bindings.len()
                    ),
                ));
            }

            let body     = branch.body.clone();
            let bindings = branch.bindings.iter().cloned().zip(variant_fields).collect::<Vec<_>>();

            self.env.push();
            for (bname, bval) in &bindings {
                let ty = self.infer_type_from_value(bval);
                self.env.define(bname, ty, Some(bval.clone()));
            }
            let result = self.exec_stmts(&body);
            self.env.pop();
            return result;
        }

        if let Some(body) = otherwise {
            let body = body.clone();
            self.env.push();
            let result = self.exec_stmts(&body);
            self.env.pop();
            return result;
        }

        Err(builtin_sin(
            "InvalidDiscern",
            format!("no 'discern' branch stands in judgment over variant '{}' — it has escaped unjudged", matched_variant),
        ))
    }
}
