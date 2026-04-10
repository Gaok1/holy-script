use std::collections::HashMap;

use crate::{ast::HolyType, interpreter::ops::default_value};

use super::Value;

#[derive(Clone)]
pub(super) struct Binding {
    value: Value,
    /// `None` means the variable was declared without a type annotation;
    /// its type will be locked the first time it receives a value via `become`.
    ty: Option<HolyType>,
}

pub struct Env {
    global: HashMap<String, Binding>,
    locals: Vec<HashMap<String, Binding>>,
}

impl Env {
    pub fn new() -> Self {
        Env { global: HashMap::new(), locals: Vec::new() }
    }

    pub fn push(&mut self) {
        self.locals.push(HashMap::new());
    }

    pub fn pop(&mut self) {
        self.locals.pop();
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        for scope in self.locals.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v.value.clone());
            }
        }
        self.global.get(name).map(|binding| binding.value.clone())
    }

    /// Returns `None` when the variable does not exist.
    /// Returns `Some(None)` when the variable exists but has no type yet (untyped).
    /// Returns `Some(Some(ty))` when the variable exists with a concrete type.
    pub fn get_binding_state(&self, name: &str) -> Option<Option<HolyType>> {
        for scope in self.locals.iter().rev() {
            if let Some(b) = scope.get(name) {
                return Some(b.ty.clone());
            }
        }
        self.global.get(name).map(|b| b.ty.clone())
    }

    /// Convenience: returns the type if the variable exists AND is already typed.
    pub fn get_type(&self, name: &str) -> Option<HolyType> {
        self.get_binding_state(name).flatten()
    }

    /// Declare a typed variable (zero-initialised to the type's default).
    pub fn define(&mut self, name: &str, ty: HolyType, val: Option<Value>) {
        let val = val.unwrap_or(default_value(&ty));
        let binding = Binding { value: val, ty: Some(ty) };
        if let Some(scope) = self.locals.last_mut() {
            scope.insert(name.to_string(), binding);
        } else {
            self.global.insert(name.to_string(), binding);
        }
    }

    /// Declare an untyped variable (`let there be x`).
    /// Its type is `None` until the first `become` assignment locks it.
    pub fn define_untyped(&mut self, name: &str) {
        let binding = Binding { value: Value::Void, ty: None };
        if let Some(scope) = self.locals.last_mut() {
            scope.insert(name.to_string(), binding);
        } else {
            self.global.insert(name.to_string(), binding);
        }
    }

    /// Assign a new value to an already-typed variable.
    pub fn assign(&mut self, name: &str, val: Value) -> bool {
        for scope in self.locals.iter_mut().rev() {
            if let Some(binding) = scope.get_mut(name) {
                binding.value = val;
                return true;
            }
        }
        if let Some(binding) = self.global.get_mut(name) {
            binding.value = val;
            return true;
        }
        false
    }

    /// Lock the type of an untyped variable and set its first value.
    /// Called on the first `become` assignment after `let there be x`.
    pub fn lock_type(&mut self, name: &str, ty: HolyType, val: Value) -> bool {
        for scope in self.locals.iter_mut().rev() {
            if let Some(binding) = scope.get_mut(name) {
                binding.ty    = Some(ty);
                binding.value = val;
                return true;
            }
        }
        if let Some(binding) = self.global.get_mut(name) {
            binding.ty    = Some(ty);
            binding.value = val;
            return true;
        }
        false
    }

    pub fn enter_call(&mut self) -> Vec<HashMap<String, Binding>> {
        std::mem::replace(&mut self.locals, vec![HashMap::new()])
    }

    pub fn exit_call(&mut self, saved: Vec<HashMap<String, Binding>>) {
        self.locals = saved;
    }
}
