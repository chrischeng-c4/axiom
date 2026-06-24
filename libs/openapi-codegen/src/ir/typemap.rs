//! Component-schema key → generated type-name map.
//!
//! Language-neutral: a deterministic, collision-free PascalCase name per
//! component schema works as a TypeScript interface, a Python (pydantic) class,
//! and a Rust (serde) struct alike. Each emitter renders the *type expression*
//! in its own language; the *name* assignment is shared and lives here.

use crate::ir::names::{self, to_pascal};
use crate::ir::openapi::Spec;
use std::collections::BTreeMap;

/// Maps an OpenAPI component-schema key to its final, collision-free type name.
#[derive(Debug, Default)]
pub struct TypeMap {
    pub names: BTreeMap<String, String>,
}

impl TypeMap {
    /// Resolve a `#/components/schemas/<key>` reference to its type name.
    /// Non-schema or external references yield `None`.
    pub fn resolve_ref(&self, reference: &str) -> Option<String> {
        let key = reference.strip_prefix("#/components/schemas/")?;
        Some(
            self.names
                .get(key)
                .cloned()
                .unwrap_or_else(|| to_pascal(key)),
        )
    }
}

/// Assign a deterministic, collision-free type name to each component schema key.
pub fn build_type_map(spec: &Spec) -> TypeMap {
    let mut reg = names::NameRegistry::new();
    let mut map = BTreeMap::new();
    for key in spec.components.schemas.keys() {
        let name = reg.unique(&to_pascal(key));
        map.insert(key.clone(), name);
    }
    TypeMap { names: map }
}
