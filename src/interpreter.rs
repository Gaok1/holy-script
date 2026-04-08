mod builtins;
mod env;
mod errors;
mod ops;
mod value;

use std::collections::HashMap;
use std::io::{self, BufRead, Write};

use crate::ast::*;

use self::builtins::{builtin_sin, builtin_sins};
use self::env::Env;
pub use self::errors::HolyError;
use self::ops::{default_value, eval_binop, eval_literal, get_field, is_truthy};
pub use self::value::{value_type_name, Value};

type EvalResult = Result<Value, HolyError>;
type ExecResult = Result<(), HolyError>;

#[derive(Clone)]
struct SalmDef {
    params: Vec<(String, HolyType)>,
    ret_type: HolyType,
    body:   Vec<Stmt>,
}

pub struct Interpreter {
    env:               Env,
    salms:             HashMap<String, SalmDef>,
    methods:           HashMap<(String, String), SalmDef>,
    scriptures:        HashMap<String, Vec<(String, HolyType)>>,
    sins:              HashMap<String, Vec<(String, HolyType)>>,
    covenants:         HashMap<String, Vec<CovenantVariantDecl>>,
    /// Maps variant name → (covenant name, ordered field types)
    covenant_variants: HashMap<String, (String, Vec<(String, HolyType)>)>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            env:               Env::new(),
            salms:             HashMap::new(),
            methods:           HashMap::new(),
            scriptures:        HashMap::new(),
            sins:              builtin_sins(),
            covenants:         HashMap::new(),
            covenant_variants: HashMap::new(),
        }
    }

    pub fn env_get(&self, name: &str) -> Option<Value> {
        self.env.get(name)
    }

    pub fn run(&mut self, program: &Program) -> Result<(), HolyError> {
        for decl in &program.top_decls {
            self.register_top_decl(decl);
        }
        self.validate_declared_types(program)?;
        self.exec_stmts(&program.stmts)
    }

    fn register_top_decl(&mut self, decl: &TopDecl) {
        match decl {
            TopDecl::Salm { name, params, ret_type, body } => {
                self.salms.insert(name.clone(), SalmDef {
                    params: params.clone(),
                    ret_type: ret_type.clone(),
                    body: body.clone(),
                });
            }
            TopDecl::MethodSalm { name, target_type, params, ret_type, body } => {
                self.methods.insert((name.clone(), target_type.clone()), SalmDef {
                    params: params.clone(),
                    ret_type: ret_type.clone(),
                    body: body.clone(),
                });
            }
            TopDecl::Scripture { name, fields } => {
                self.scriptures.insert(name.clone(), fields.clone());
            }
            TopDecl::SinDecl { name, fields } => {
                self.sins.insert(name.clone(), fields.clone());
            }
            TopDecl::Covenant { name, variants } => {
                self.covenants.insert(name.clone(), variants.clone());
                for v in variants {
                    self.covenant_variants.insert(
                        v.name.clone(),
                        (name.clone(), v.fields.clone()),
                    );
                }
            }
        }
    }

    fn exec_stmts(&mut self, stmts: &[Stmt]) -> ExecResult {
        for stmt in stmts {
            self.exec_stmt(stmt)?;
        }
        Ok(())
    }

    fn exec_stmt(&mut self, stmt: &Stmt) -> ExecResult {
        match stmt {
            Stmt::DeclNoVal { name, ty } => {
                self.ensure_type_exists(ty)?;
                let val = default_value(ty);
                self.env.define(name, ty.clone(), val);
            }
            Stmt::DeclVal { name, ty, val } => {
                self.ensure_type_exists(ty)?;
                let v = self.eval_expr(val)?;
                self.expect_type(ty, &v, &format!("invalid value for '{}'", name))?;
                self.env.define(name, ty.clone(), v);
            }
            Stmt::Assign { name, val } => {
                let v = self.eval_expr(val)?;
                if let Some(var_ty) = self.env.get_type(name) {
                    self.expect_type(&var_ty, &v, &format!("invalid value for '{}'", name))?;
                    self.env.assign(name, v);
                } else {
                    let inferred_ty = self.infer_type_from_value(&v);
                    self.env.define(name, inferred_ty, v);
                }
            }
            Stmt::FnCallStmt { name, args } => {
                let vals = self.eval_args(args)?;
                self.call_salm(name, vals)?;
            }
            Stmt::MethodCallStmt { method, target, args } => {
                let target_val = self.lookup_var(target)?;
                let vals = self.eval_args(args)?;
                self.call_method(method, target_val, vals)?;
            }
            Stmt::Reveal(expr) => {
                let v = self.eval_expr(expr)?;
                return Err(HolyError::Return(v));
            }
            Stmt::Conditional { branches, otherwise } => {
                let mut executed = false;
                for (cond, body) in branches {
                    let cv = self.eval_expr(cond)?;
                    if is_truthy(&cv) {
                        self.env.push();
                        let r = self.exec_stmts(body);
                        self.env.pop();
                        r?;
                        executed = true;
                        break;
                    }
                }
                if !executed {
                    if let Some(else_body) = otherwise {
                        self.env.push();
                        let r = self.exec_stmts(else_body);
                        self.env.pop();
                        r?;
                    }
                }
            }
            Stmt::Litany { cond, body } => {
                loop {
                    let cv = self.eval_expr(cond)?;
                    if !is_truthy(&cv) {
                        break;
                    }
                    self.env.push();
                    let r = self.exec_stmts(body);
                    self.env.pop();
                    match r {
                        Ok(()) => {}
                        Err(HolyError::Break) => break,
                        Err(HolyError::Continue) => continue,
                        Err(e) => return Err(e),
                    }
                }
            }
            Stmt::Forsake => return Err(HolyError::Break),
            Stmt::Ascend => return Err(HolyError::Continue),
            Stmt::Confess { try_block, handlers, absolve } => {
                self.exec_confess(try_block, handlers, absolve)?;
            }
            Stmt::Discern { target, branches, otherwise } => {
                self.exec_discern(target, branches, otherwise)?;
            }
            Stmt::Transgress { sin_type, args } => {
                let fields = self.build_sin_fields(sin_type, args)?;
                return Err(HolyError::Sin { type_name: sin_type.clone(), fields });
            }
        }
        Ok(())
    }

    fn exec_confess(
        &mut self,
        try_block: &[Stmt],
        handlers: &[SinHandler],
        absolve: &Option<Vec<Stmt>>,
    ) -> ExecResult {
        self.env.push();
        let try_result = self.exec_stmts(try_block);
        self.env.pop();

        let after = match try_result {
            Err(HolyError::Sin { ref type_name, ref fields }) => {
                let matched = handlers.iter().find(|h| &h.sin_type == type_name);
                match matched {
                    Some(h) => {
                        let sin_val = Value::Scripture {
                            type_name: type_name.clone(),
                            fields: fields.clone(),
                        };
                        let body = h.body.clone();
                        let bind = h.binding.clone();
                        self.env.push();
                        if let Some(b) = &bind {
                            self.env.define(b, HolyType::Custom(type_name.clone()), sin_val);
                        }
                        let r = self.exec_stmts(&body);
                        self.env.pop();
                        r
                    }
                    None => try_result,
                }
            }
            other => other,
        };

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

    fn validate_declared_types(&self, program: &Program) -> Result<(), HolyError> {
        for decl in &program.top_decls {
            match decl {
                TopDecl::Salm { params, ret_type, .. } => {
                    for (_, ty) in params {
                        self.ensure_type_exists(ty)?;
                    }
                    self.ensure_type_exists(ret_type)?;
                }
                TopDecl::MethodSalm { target_type, params, ret_type, .. } => {
                    self.ensure_custom_type_exists(target_type)?;
                    for (_, ty) in params {
                        self.ensure_type_exists(ty)?;
                    }
                    self.ensure_type_exists(ret_type)?;
                }
                TopDecl::Scripture { fields, .. } | TopDecl::SinDecl { fields, .. } => {
                    for (_, ty) in fields {
                        self.ensure_type_exists(ty)?;
                    }
                }
                TopDecl::Covenant { variants, .. } => {
                    for v in variants {
                        for (_, ty) in &v.fields {
                            self.ensure_type_exists(ty)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn exec_discern(
        &mut self,
        target: &str,
        branches: &[DiscernBranch],
        otherwise: &Option<Vec<Stmt>>,
    ) -> ExecResult {
        let target_value = self.lookup_var(target)?;

        let (matched_variant, variant_fields) = match target_value {
            Value::CovenantVariant { variant, fields, .. } => (variant, fields),
            other => {
                return Err(builtin_sin(
                    "InvalidDiscern",
                    format!(
                        "'discern' expects a covenant variant in '{}', got {}",
                        target,
                        value_type_name(&other)
                    ),
                ));
            }
        };

        if let Some(branch) = branches.iter().find(|b| b.variant == matched_variant) {
            if !branch.bindings.is_empty() && branch.bindings.len() != variant_fields.len() {
                return Err(builtin_sin(
                    "InvalidDiscern",
                    format!(
                        "variant '{}' has {} field(s) but 'bearing' lists {} binding(s)",
                        matched_variant, variant_fields.len(), branch.bindings.len()
                    ),
                ));
            }
            let body = branch.body.clone();
            let bindings: Vec<(String, Value)> = branch.bindings.iter()
                .cloned()
                .zip(variant_fields.into_iter())
                .collect();
            self.env.push();
            for (bname, bval) in &bindings {
                let ty = self.infer_type_from_value(bval);
                self.env.define(bname, ty, bval.clone());
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
            format!("no 'discern' branch covers variant '{}'", matched_variant),
        ))
    }

    fn call_salm(&mut self, name: &str, args: Vec<Value>) -> EvalResult {
        match name {
            "proclaim" => {
                let s = args.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" ");
                println!("{}", s);
                return Ok(Value::Void);
            }
            "herald" => {
                let s = args.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" ");
                print!("{}", s);
                io::stdout().flush().ok();
                return Ok(Value::Void);
            }
            "inquire" => {
                let mut line = String::new();
                io::stdin().lock().read_line(&mut line).ok();
                return Ok(Value::Str(line.trim_end_matches('\n').to_string()));
            }
            "atom_of" => {
                let n = match args.first() {
                    Some(Value::Str(s)) => s.trim().parse().unwrap_or(0),
                    _ => 0,
                };
                return Ok(Value::Int(n));
            }
            "word_of" => {
                return Ok(Value::Str(args.first().map(|v| v.to_string()).unwrap_or_default()));
            }
            _ => {}
        }

        let def = self.salms.get(name)
            .ok_or_else(|| builtin_sin("UndefinedSalm", format!("salm '{}' is not defined", name)))?
            .clone();

        self.exec_salm_body(&def, None, args)
    }

    fn call_method(&mut self, method: &str, target: Value, args: Vec<Value>) -> EvalResult {
        let type_name = match &target {
            Value::Scripture { type_name, .. } => type_name.clone(),
            other => {
                return Err(builtin_sin(
                    "TypeError",
                    format!("cannot call a method on a value of type '{}'", value_type_name(other)),
                ));
            }
        };

        let def = self.methods.get(&(method.to_string(), type_name))
            .ok_or_else(|| builtin_sin("UndefinedMethod", format!("method '{}' not found", method)))?
            .clone();

        self.exec_salm_body(&def, Some(target), args)
    }

    fn exec_salm_body(&mut self, def: &SalmDef, self_val: Option<Value>, args: Vec<Value>) -> EvalResult {
        if args.len() != def.params.len() {
            return Err(builtin_sin(
                "InvalidArgumentCount",
                format!("salm expects {} arguments, got {}", def.params.len(), args.len()),
            ));
        }

        let saved = self.env.enter_call();
        if let Some(val) = self_val {
            let self_ty = self.infer_type_from_value(&val);
            self.env.define("its", self_ty, val);
        }
        for ((pname, pty), val) in def.params.iter().zip(args) {
            self.expect_type(pty, &val, &format!("invalid argument for parameter '{}'", pname))?;
            self.env.define(pname, pty.clone(), val);
        }
        let result = self.exec_stmts(&def.body);
        self.env.exit_call(saved);

        match result {
            Ok(()) => {
                let value = Value::Void;
                self.expect_type(&def.ret_type, &value, "invalid salm return")?;
                Ok(value)
            }
            Err(HolyError::Return(v)) => {
                self.expect_type(&def.ret_type, &v, "invalid salm return")?;
                Ok(v)
            }
            Err(e) => Err(e),
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> EvalResult {
        match expr {
            Expr::Lit(lit) => Ok(eval_literal(lit)),

            Expr::Var(name) => {
                if let Some(value) = self.env.get(name) {
                    Ok(value)
                } else if let Some((covenant, fields)) = self.covenant_variants.get(name) {
                    if !fields.is_empty() {
                        return Err(builtin_sin(
                            "InvalidArgumentCount",
                            format!("variant '{}' requires fields — use 'manifest {} praying ...'", name, name),
                        ));
                    }
                    Ok(Value::CovenantVariant {
                        covenant: covenant.clone(),
                        variant: name.clone(),
                        fields: Vec::new(),
                    })
                } else {
                    Err(builtin_sin(
                        "UndefinedVariable",
                        format!("undefined variable '{}'", name),
                    ))
                }
            }

            Expr::Negate(expr) => {
                let value = self.eval_expr(expr)?;
                match value {
                    Value::Int(n) => Ok(Value::Int(-n)),
                    Value::Float(x) => Ok(Value::Float(-x)),
                    other => Err(builtin_sin(
                        "TypeError",
                        format!("'negate' expects atom or fractional, got {}", value_type_name(&other)),
                    )),
                }
            }

            Expr::BinOp { op, left, right } => {
                let lv = self.eval_expr(left)?;
                let rv = self.eval_expr(right)?;
                eval_binop(op, lv, rv)
            }

            Expr::FnCall { name, args } => {
                let vals = self.eval_args(args)?;
                self.call_salm(name, vals)
            }

            Expr::MethodCall { method, target, args } => {
                let tv = self.lookup_var(target)?;
                let vals = self.eval_args(args)?;
                self.call_method(method, tv, vals)
            }

            Expr::Manifest { scripture, args } => {
                // Covenant data variant takes priority over scripture
                if let Some((covenant, field_defs)) = self.covenant_variants.get(scripture).cloned() {
                    if field_defs.is_empty() {
                        return Err(builtin_sin(
                            "InvalidArgumentCount",
                            format!("variant '{}' is a unit variant — instantiate it without 'manifest'", scripture),
                        ));
                    }
                    if args.len() != field_defs.len() {
                        return Err(builtin_sin(
                            "InvalidArgumentCount",
                            format!("variant '{}' expects {} field(s), got {}", scripture, field_defs.len(), args.len()),
                        ));
                    }
                    let mut values = Vec::new();
                    for ((fname, field_ty), arg) in field_defs.iter().zip(args.iter()) {
                        let v = self.eval_expr(arg)?;
                        self.expect_type(field_ty, &v, &format!("invalid value for variant field '{}.{}'", scripture, fname))?;
                        values.push(v);
                    }
                    return Ok(Value::CovenantVariant {
                        covenant,
                        variant: scripture.clone(),
                        fields: values,
                    });
                }

                let def = self.scriptures.get(scripture)
                    .ok_or_else(|| builtin_sin("UndefinedScripture", format!("'{}' is not a scripture or data variant", scripture)))?
                    .clone();

                if args.len() != def.len() {
                    return Err(builtin_sin(
                        "InvalidArgumentCount",
                        format!("scripture '{}' expects {} fields, got {}", scripture, def.len(), args.len()),
                    ));
                }

                let mut fields = HashMap::new();
                for ((fname, field_ty), arg) in def.iter().zip(args.iter()) {
                    let v = self.eval_expr(arg)?;
                    self.expect_type(field_ty, &v, &format!("invalid value for field '{}.{}'", scripture, fname))?;
                    fields.insert(fname.clone(), v);
                }
                Ok(Value::Scripture { type_name: scripture.clone(), fields })
            }

            Expr::FieldAccess { field, object } => {
                let obj = self.eval_expr(object)?;
                get_field(&obj, field)
            }

            Expr::SelfFieldAccess { field } => {
                let its = self.env.get("its")
                    .ok_or_else(|| builtin_sin("InvalidContext", "'its' is not available outside a method_salm"))?;
                get_field(&its, field)
            }
        }
    }

    fn eval_args(&mut self, args: &[Expr]) -> Result<Vec<Value>, HolyError> {
        args.iter().map(|a| self.eval_expr(a)).collect()
    }

    fn build_sin_fields(&mut self, sin_type: &str, args: &[Expr]) -> Result<HashMap<String, Value>, HolyError> {
        let def = self.sins.get(sin_type)
            .ok_or_else(|| builtin_sin("UndefinedSin", format!("sin '{}' is not declared", sin_type)))?
            .clone();

        if args.len() != def.len() {
            return Err(builtin_sin(
                "InvalidArgumentCount",
                format!("sin '{}' expects {} fields, got {}", sin_type, def.len(), args.len()),
            ));
        }

        let mut fields = HashMap::new();
        for ((fname, field_ty), arg) in def.iter().zip(args.iter()) {
            let v = self.eval_expr(arg)?;
            self.expect_type(field_ty, &v, &format!("invalid value for sin field '{}.{}'", sin_type, fname))?;
            fields.insert(fname.clone(), v);
        }
        Ok(fields)
    }

    fn lookup_var(&self, name: &str) -> Result<Value, HolyError> {
        self.env.get(name).ok_or_else(|| {
            builtin_sin("UndefinedVariable", format!("undefined variable '{}'", name))
        })
    }

    fn ensure_type_exists(&self, ty: &HolyType) -> Result<(), HolyError> {
        if let HolyType::Custom(name) = ty {
            self.ensure_custom_type_exists(name)?;
        }
        Ok(())
    }

    fn ensure_custom_type_exists(&self, name: &str) -> Result<(), HolyError> {
        if self.scriptures.contains_key(name) || self.covenants.contains_key(name) || self.sins.contains_key(name) {
            Ok(())
        } else {
            Err(builtin_sin("UndefinedType", format!("type '{}' is not declared", name)))
        }
    }

    fn expect_type(&self, ty: &HolyType, value: &Value, context: &str) -> Result<(), HolyError> {
        if self.value_matches_type(ty, value) {
            Ok(())
        } else {
            Err(builtin_sin(
                "TypeError",
                format!("{}: expected {}, got {}", context, self.describe_type(ty), self.describe_value(value)),
            ))
        }
    }

    fn value_matches_type(&self, ty: &HolyType, value: &Value) -> bool {
        match ty {
            HolyType::Atom => matches!(value, Value::Int(_)),
            HolyType::Fractional => matches!(value, Value::Float(_)),
            HolyType::Word => matches!(value, Value::Str(_)),
            HolyType::Dogma => matches!(value, Value::Bool(_)),
            HolyType::Void => matches!(value, Value::Void),
            HolyType::Custom(name) => match value {
                Value::Scripture { type_name, .. } => type_name == name,
                Value::CovenantVariant { covenant, .. } => covenant == name,
                _ => false,
            },
        }
    }

    fn infer_type_from_value(&self, value: &Value) -> HolyType {
        match value {
            Value::Int(_) => HolyType::Atom,
            Value::Float(_) => HolyType::Fractional,
            Value::Str(_) => HolyType::Word,
            Value::Bool(_) => HolyType::Dogma,
            Value::Void => HolyType::Void,
            Value::CovenantVariant { covenant, .. } => HolyType::Custom(covenant.clone()),
            Value::Scripture { type_name, .. } => HolyType::Custom(type_name.clone()),
        }
    }

    fn describe_type(&self, ty: &HolyType) -> String {
        match ty {
            HolyType::Atom => "atom".into(),
            HolyType::Fractional => "fractional".into(),
            HolyType::Word => "word".into(),
            HolyType::Dogma => "dogma".into(),
            HolyType::Void => "void".into(),
            HolyType::Custom(name) => name.clone(),
        }
    }

    fn describe_value(&self, value: &Value) -> String {
        match value {
            Value::Int(_) => "atom".into(),
            Value::Float(_) => "fractional".into(),
            Value::Str(_) => "word".into(),
            Value::Bool(_) => "dogma".into(),
            Value::Void => "void".into(),
            Value::CovenantVariant { covenant, .. } => covenant.clone(),
            Value::Scripture { type_name, .. } => type_name.clone(),
        }
    }
}
