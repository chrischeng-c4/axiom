---
id: libs-compass-src-type-inference-type-env-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/type_inference/type_env.rs`.
capability_refs:
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: multi-language-parser-and-checker-dispatch-contract
  gap: multi-language-parser-and-checker-dispatch-contract
  coverage: full
  rationale: "Multi-language parser and checker dispatch contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: agent-diagnostic-output-contract
  gap: agent-diagnostic-output-contract
  coverage: full
  rationale: "Agent diagnostic output contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: symbol-outline-and-propagated-type-query-contract
  gap: symbol-outline-and-propagated-type-query-contract
  coverage: full
  rationale: "Symbol outline and propagated type query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: semantic-search-and-graph-query-contract
  gap: semantic-search-and-graph-query-contract
  coverage: full
  rationale: "Semantic search and graph query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: structured-refactoring-contract
  gap: structured-refactoring-contract
  coverage: full
  rationale: "Structured refactoring contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: spec-parser-and-state-machine-validation-contract
  gap: spec-parser-and-state-machine-validation-contract
  coverage: full
  rationale: "Spec parser and state-machine validation contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: python-and-rust-generator-registry-contract
  gap: python-and-rust-generator-registry-contract
  coverage: full
  rationale: "Python and Rust generator registry contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: argus-daemon-protocol-and-request-handling-contract
  gap: argus-daemon-protocol-and-request-handling-contract
  coverage: full
  rationale: "Argus daemon protocol and request handling contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: watch-bridge-and-incremental-dirty-file-contract
  gap: watch-bridge-and-incremental-dirty-file-contract
  coverage: full
  rationale: "Watch bridge and incremental dirty-file contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
fill_sections: [overview, source, changes]
---

# Standardized libs/compass/src/type_inference/type_env.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/type_inference/type_env.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `TypeEnv` | libs/compass/src/type_inference/type_env.rs | struct | pub | 9 | pub struct TypeEnv { |
| `new` | libs/compass/src/type_inference/type_env.rs | function | pub | 15 | pub fn new() -> Self { |
| `push_scope` | libs/compass/src/type_inference/type_env.rs | function | pub | 22 | pub fn push_scope(&mut self) { |
| `pop_scope` | libs/compass/src/type_inference/type_env.rs | function | pub | 27 | pub fn pop_scope(&mut self) { |
| `bind` | libs/compass/src/type_inference/type_env.rs | function | pub | 34 | pub fn bind(&mut self, name: String, ty: Type) { |
| `lookup` | libs/compass/src/type_inference/type_env.rs | function | pub | 41 | pub fn lookup(&self, name: &str) -> Option<&Type> { |
| `get_all_types` | libs/compass/src/type_inference/type_env.rs | function | pub | 51 | pub fn get_all_types(&self) -> HashMap<String, Type> { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/type_inference/type_env.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/type_inference/type_env.rs` captured during libs codegen standardization.
```
