//! Type environment for name-to-type mappings

use std::collections::HashMap;

use super::ty::Type;

/// Type environment mapping names to types
#[derive(Debug, Clone, Default)]
pub struct TypeEnv {
    /// Stack of scopes, innermost last
    scopes: Vec<HashMap<String, Type>>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    /// Push a new scope
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pop the innermost scope
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Bind a name to a type in the current scope
    pub fn bind(&mut self, name: String, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, ty);
        }
    }

    /// Look up a name, searching from innermost to outermost scope
    pub fn lookup(&self, name: &str) -> Option<&Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }

    /// Get all types from all scopes (later scopes override earlier ones)
    pub fn get_all_types(&self) -> HashMap<String, Type> {
        let mut types = HashMap::new();
        for scope in &self.scopes {
            for (name, ty) in scope {
                types.insert(name.clone(), ty.clone());
            }
        }
        types
    }
}
