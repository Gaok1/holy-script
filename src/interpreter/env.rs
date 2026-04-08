use std::collections::HashMap;

use crate::ast::HolyType;

use super::Value;

#[derive(Clone)]
pub(super) struct Binding {
    value: Value,
    ty: HolyType,
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

    pub fn get_type(&self, name: &str) -> Option<HolyType> {
        for scope in self.locals.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v.ty.clone());
            }
        }
        self.global.get(name).map(|binding| binding.ty.clone())
    }

    pub fn define(&mut self, name: &str, ty: HolyType, val: Value) {
        let binding = Binding { value: val, ty };
        if let Some(scope) = self.locals.last_mut() {
            scope.insert(name.to_string(), binding);
        } else {
            self.global.insert(name.to_string(), binding);
        }
    }

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

    pub fn enter_call(&mut self) -> Vec<HashMap<String, Binding>> {
        std::mem::replace(&mut self.locals, vec![HashMap::new()])
    }

    pub fn exit_call(&mut self, saved: Vec<HashMap<String, Binding>>) {
        self.locals = saved;
    }
}
