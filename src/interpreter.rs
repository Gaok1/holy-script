mod builtins;
mod call;
mod env;
mod errors;
mod eval;
mod exec;
mod generics;
mod ops;
mod types;
mod value;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::ast::*;
use crate::lexer::tokenize;
use crate::parser::Parser;

use self::builtins::builtin_sins;
use self::env::Env;
use self::generics::builtin_covenants;
pub use self::errors::HolyError;
pub use self::value::{value_type_name, Value};

type EvalResult = Result<Value, HolyError>;
type ExecResult = Result<(), HolyError>;

/// Definition of a user-declared salm (function or method).
#[derive(Clone)]
struct SalmDef {
    type_params: Vec<String>,
    params:      Vec<(String, HolyType)>,
    ret_type:    HolyType,
    body:        Vec<Stmt>,
}

/// Stores a scripture's type parameters alongside its field definitions.
#[derive(Clone)]
struct ScriptureDef {
    type_params: Vec<String>,
    fields:      Vec<(String, HolyType)>,
}

pub struct Interpreter {
    env:               Env,
    salms:             HashMap<String, SalmDef>,
    methods:           HashMap<(String, String), SalmDef>,
    scriptures:        HashMap<String, ScriptureDef>,
    sins:              HashMap<String, Vec<(String, HolyType)>>,
    covenants:         HashMap<String, Vec<CovenantVariantDecl>>,
    /// Maps variant name → (covenant name, ordered field types).
    covenant_variants: HashMap<String, (String, Vec<(String, HolyType)>)>,
    /// CLI args forwarded to the running script (accessible via `hail args`).
    script_args:       Vec<String>,
    /// Directory of the entry file, used to resolve `testament` imports.
    source_dir:        Option<PathBuf>,
    /// Tracks already-loaded module names to prevent circular imports.
    loaded_modules:    HashSet<String>,
    /// Stack of active generic type parameter bindings, pushed/popped per salm call.
    /// Each frame maps type param name → resolved concrete type.
    type_bindings:     Vec<HashMap<String, HolyType>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let (builtin_cov, builtin_cov_variants) = builtin_covenants();
        Interpreter {
            env:               Env::new(),
            salms:             HashMap::new(),
            methods:           HashMap::new(),
            scriptures:        HashMap::new(),
            sins:              builtin_sins(),
            covenants:         builtin_cov,
            covenant_variants: builtin_cov_variants,
            script_args:       Vec::new(),
            source_dir:        None,
            loaded_modules:    HashSet::new(),
            type_bindings:     Vec::new(),
        }
    }

    /// Pass the script's command-line arguments (everything after the filename).
    pub fn with_script_args(mut self, args: Vec<String>) -> Self {
        self.script_args = args;
        self
    }

    /// Set the directory used to resolve `testament` imports.
    pub fn with_source_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.source_dir = Some(dir.into());
        self
    }

    pub fn env_get(&self, name: &str) -> Option<Value> {
        self.env.get(name)
    }

    // ── Generic type binding helpers ──────────────────────────────────────────

    /// Push a new frame of resolved type param bindings (e.g. `{T: Atom}`).
    pub(self) fn push_type_bindings(&mut self, bindings: HashMap<String, HolyType>) {
        self.type_bindings.push(bindings);
    }

    /// Pop the innermost type binding frame.
    pub(self) fn pop_type_bindings(&mut self) {
        self.type_bindings.pop();
    }

    /// Substitute any bound type params in `ty` with their resolved concrete types.
    /// Unbound params (e.g. `T` when T has no binding) are left as `Custom("T")`.
    pub(self) fn resolve_type(&self, ty: &HolyType) -> HolyType {
        match ty {
            HolyType::Custom(name) => {
                for frame in self.type_bindings.iter().rev() {
                    if let Some(resolved) = frame.get(name) {
                        return resolved.clone();
                    }
                }
                ty.clone()
            }
            HolyType::Generic(name, args) => {
                HolyType::Generic(name.clone(), args.iter().map(|a| self.resolve_type(a)).collect())
            }
            _ => ty.clone(),
        }
    }

    /// Infer type param bindings by unifying declared parameter types against
    /// the types of the actual argument values.
    pub(self) fn infer_type_bindings(
        &self,
        type_params: &[String],
        params:      &[(String, HolyType)],
        args:        &[Value],
    ) -> HashMap<String, HolyType> {
        let mut bindings = HashMap::new();
        for ((_, pty), val) in params.iter().zip(args.iter()) {
            let val_ty = self.infer_type_from_value(val);
            self.unify_param_type(pty, &val_ty, type_params, &mut bindings);
        }
        bindings
    }

    /// Recursively unify `param_ty` against `val_ty`, binding free type params.
    /// If a param is already bound inconsistently, the new binding is silently ignored
    /// (a proper type error will surface when the resolved type is checked against the value).
    pub(self) fn unify_param_type(
        &self,
        param_ty:    &HolyType,
        val_ty:      &HolyType,
        type_params: &[String],
        bindings:    &mut HashMap<String, HolyType>,
    ) {
        match param_ty {
            HolyType::Custom(name) if type_params.contains(name) => {
                // Only bind if not already bound (first concrete binding wins)
                bindings.entry(name.clone()).or_insert_with(|| val_ty.clone());
            }
            HolyType::Generic(pname, pargs) => {
                match val_ty {
                    HolyType::Generic(vname, vargs) if pname == vname && pargs.len() == vargs.len() => {
                        for (pa, va) in pargs.iter().zip(vargs.iter()) {
                            self.unify_param_type(pa, va, type_params, bindings);
                        }
                    }
                    // val has pending type args (Custom with same name) — skip inner unification
                    HolyType::Custom(vname) if vname == pname => {}
                    _ => {}
                }
            }
            _ => {}
        }
    }

    /// Resolve the current type bindings into a type_args vec for a generic definition.
    /// Returns `vec![]` (pending) if any param is unresolved or not concrete.
    pub(self) fn resolve_type_args(&self, type_params: &[String]) -> Vec<HolyType> {
        let resolved: Vec<HolyType> = type_params.iter()
            .map(|p| self.resolve_type(&HolyType::Custom(p.clone())))
            .collect();
        if resolved.iter().all(|t| self.is_concrete_type(t)) {
            resolved
        } else {
            vec![]  // still pending — not all type params are concrete
        }
    }

    pub fn run(&mut self, program: &Program) -> Result<(), HolyError> {
        // Load testament imports first so their symbols are visible to the
        // current program's declarations and statements.
        for testament in &program.testaments.clone() {
            self.load_testament(testament)?;
        }

        for decl in &program.top_decls {
            self.register_top_decl(decl);
        }
        self.validate_declared_types(program)?;
        self.exec_stmts(&program.stmts)
    }

    // ── Testament loading ─────────────────────────────────────────────────────

    fn load_testament(&mut self, testament: &Testament) -> Result<(), HolyError> {
        use self::builtins::builtin_sin;

        // Build the canonical module key: "pasta1/pasta2/name" (or just "name")
        let module_key = if testament.path.is_empty() {
            testament.name.clone()
        } else {
            format!("{}/{}", testament.path.join("/"), testament.name)
        };

        if self.loaded_modules.contains(&module_key) {
            return Ok(()); // already loaded — skip silently
        }

        // Resolve the source: check filesystem first, then fall back to stdlib
        let source = self.resolve_testament_source(testament).map_err(|e| {
            builtin_sin("UndefinedTestament", e)
        })?;

        let tokens = tokenize(&source);

        let mut parser = Parser::new(tokens);
        let module_program = parser.parse_program().map_err(|e| {
            builtin_sin(
                "UndefinedTestament",
                format!("testament '{}' contains a transgression: {}", module_key, e),
            )
        })?;

        self.loaded_modules.insert(module_key.clone());

        // Recursively load the module's own testaments first
        for dep in &module_program.testaments.clone() {
            self.load_testament(dep)?;
        }

        // Register declarations first (so type validation can resolve them)
        for decl in &module_program.top_decls {
            if should_import(decl, testament.revealing.as_deref()) {
                self.register_top_decl(decl);
            }
        }

        // Validate types declared in this module
        self.validate_declared_types(&module_program)?;

        Ok(())
    }

    /// Resolves the source text for a testament.
    /// Tries the filesystem path first; if not found and the name matches a
    /// known built-in stdlib module, returns the embedded source.
    fn resolve_testament_source(&self, testament: &Testament) -> Result<String, String> {
        let dir = self.source_dir.clone().unwrap_or_else(|| PathBuf::from("."));

        // Build filesystem path: {source_dir}/{path segments...}/{name}.holy
        let mut file_path = dir;
        for segment in &testament.path {
            file_path = file_path.join(segment);
        }
        file_path = file_path.join(format!("{}.holy", testament.name));

        match std::fs::read_to_string(&file_path) {
            Ok(src) => return Ok(src),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                return Err(format!(
                    "testament '{}' could not be unsealed at '{}': {}",
                    testament.name, file_path.display(), e
                ));
            }
        }

        // File not found — try stdlib (only for root-level imports with no path)
        if testament.path.is_empty() {
            if let Some(src) = stdlib_source(&testament.name) {
                return Ok(src.to_string());
            }
        }

        Err(format!(
            "testament '{}' could not be unsealed at '{}'",
            testament.name, file_path.display()
        ))
    }

    // ── Declaration registration ──────────────────────────────────────────────

    fn register_top_decl(&mut self, decl: &TopDecl) {
        match decl {
            TopDecl::Salm { name, type_params, params, ret_type, body } => {
                self.salms.insert(name.clone(), SalmDef {
                    type_params: type_params.clone(),
                    params:      params.clone(),
                    ret_type:    ret_type.clone(),
                    body:        body.clone(),
                });
            }
            TopDecl::MethodSalm { name, type_params, target_type, params, ret_type, body } => {
                self.methods.insert((name.clone(), target_type.clone()), SalmDef {
                    type_params: type_params.clone(),
                    params:      params.clone(),
                    ret_type:    ret_type.clone(),
                    body:        body.clone(),
                });
            }
            TopDecl::Scripture { name, type_params, fields } => {
                self.scriptures.insert(name.clone(), ScriptureDef {
                    type_params: type_params.clone(),
                    fields:      fields.clone(),
                });
            }
            TopDecl::SinDecl { name, fields } => {
                self.sins.insert(name.clone(), fields.clone());
            }
            TopDecl::Covenant { name, variants, .. } => {
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

    // ── Upfront type validation ───────────────────────────────────────────────

    fn validate_declared_types(&self, program: &Program) -> Result<(), HolyError> {
        for decl in &program.top_decls {
            match decl {
                TopDecl::Salm { params, ret_type, type_params, .. } => {
                    for (_, ty) in params {
                        self.ensure_type_exists_with_params(ty, type_params)?;
                    }
                    self.ensure_type_exists_with_params(ret_type, type_params)?;
                }
                TopDecl::MethodSalm { target_type, params, ret_type, type_params, .. } => {
                    self.ensure_custom_type_exists(target_type)?;
                    for (_, ty) in params {
                        self.ensure_type_exists_with_params(ty, type_params)?;
                    }
                    self.ensure_type_exists_with_params(ret_type, type_params)?;
                }
                TopDecl::Scripture { name, type_params, .. } => {
                    if let Some(def) = self.scriptures.get(name) {
                        for (_, ty) in &def.fields.clone() {
                            self.ensure_type_exists_with_params(ty, type_params)?;
                        }
                    }
                }
                TopDecl::SinDecl { fields, .. } => {
                    for (_, ty) in fields {
                        self.ensure_type_exists(ty)?;
                    }
                }
                TopDecl::Covenant { variants, type_params, .. } => {
                    for v in variants {
                        for (_, ty) in &v.fields {
                            self.ensure_type_exists_with_params(ty, type_params)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

// ── Testament import filtering ────────────────────────────────────────────────

/// Returns `true` if `decl` should be imported given the `revealing` filter.
/// `None` → import everything. `Some(list)` → import only listed symbols.
/// Method salms are included whenever their target type is included (or no filter).
fn should_import(decl: &TopDecl, revealing: Option<&[String]>) -> bool {
    let Some(list) = revealing else { return true };

    match decl {
        TopDecl::Salm       { name, .. }  => list.contains(name),
        TopDecl::Scripture  { name, .. }  => list.contains(name),
        TopDecl::SinDecl    { name, .. }  => list.contains(name),
        TopDecl::Covenant   { name, .. }  => list.contains(name),
        // Include method salms whose target type is in the revealing list,
        // or whose method name is explicitly listed.
        TopDecl::MethodSalm { name, target_type, .. } => {
            list.contains(name) || list.contains(target_type)
        }
    }
}

// ── Standard library (embedded) ───────────────────────────────────────────────

/// Returns the embedded source of a known built-in stdlib module, or `None`.
fn stdlib_source(name: &str) -> Option<&'static str> {
    match name {
        "arithmos" => Some(include_str!("stdlib/arithmos.holy")),
        _ => None,
    }
}
