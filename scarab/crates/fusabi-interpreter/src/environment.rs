use indexmap::IndexMap;
use crate::ast::Value;
use crate::error::{FusabiError, Result};

/// Environment for variable bindings (supports lexical scoping)
#[derive(Debug, Clone)]
pub struct Environment {
    scopes: Vec<IndexMap<String, Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            scopes: vec![IndexMap::new()],
        }
    }

    /// Push a new scope
    pub fn push_scope(&mut self) {
        self.scopes.push(IndexMap::new());
    }

    /// Pop the current scope
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Define a variable in the current scope
    pub fn define(&mut self, name: String, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }

    /// Get a variable's value (searches from innermost to outermost scope)
    pub fn get(&self, name: &str) -> Result<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone());
            }
        }
        Err(FusabiError::UndefinedVariable(name.to_string()))
    }

    /// Check if a variable exists
    pub fn contains(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|scope| scope.contains_key(name))
    }

    /// Update an existing variable (searches from innermost to outermost)
    pub fn set(&mut self, name: &str, value: Value) -> Result<()> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return Ok(());
            }
        }
        Err(FusabiError::UndefinedVariable(name.to_string()))
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
