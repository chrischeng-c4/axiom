---
id: libs-compass-src-type-inference-class-info-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/type_inference/class_info.rs`.
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

# Standardized libs/compass/src/type_inference/class_info.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/type_inference/class_info.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `GenericParam` | libs/compass/src/type_inference/class_info.rs | struct | pub | 9 | pub struct GenericParam { |
| `new` | libs/compass/src/type_inference/class_info.rs | function | pub | 23 | pub fn new(id: TypeVarId, name: String) -> Self { |
| `with_variance` | libs/compass/src/type_inference/class_info.rs | function | pub | 33 | pub fn with_variance(mut self, variance: Variance) -> Self { |
| `with_bound` | libs/compass/src/type_inference/class_info.rs | function | pub | 38 | pub fn with_bound(mut self, bound: Type) -> Self { |
| `with_constraints` | libs/compass/src/type_inference/class_info.rs | function | pub | 43 | pub fn with_constraints(mut self, constraints: Vec<Type>) -> Self { |
| `ClassInfo` | libs/compass/src/type_inference/class_info.rs | struct | pub | 51 | pub struct ClassInfo { |
| `is_generic` | libs/compass/src/type_inference/class_info.rs | function | pub | 85 | pub fn is_generic(&self) -> bool { |
| `arity` | libs/compass/src/type_inference/class_info.rs | function | pub | 90 | pub fn arity(&self) -> usize { |
| `get_type_param` | libs/compass/src/type_inference/class_info.rs | function | pub | 95 | pub fn get_type_param(&self, name: &str) -> Option<&GenericParam> { |
| `get_type_param_by_index` | libs/compass/src/type_inference/class_info.rs | function | pub | 100 | pub fn get_type_param_by_index(&self, index: usize) -> Option<&GenericParam> { |
| `variance_at` | libs/compass/src/type_inference/class_info.rs | function | pub | 105 | pub fn variance_at(&self, index: usize) -> Variance { |
| `add_type_param` | libs/compass/src/type_inference/class_info.rs | function | pub | 113 | pub fn add_type_param(&mut self, param: GenericParam) { |
| `get_attribute` | libs/compass/src/type_inference/class_info.rs | function | pub | 118 | pub fn get_attribute(&self, name: &str) -> Option<&Type> { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Class information for type inference

use std::collections::HashMap;

use super::ty::{Type, TypeVarId, Variance};

/// Information about a generic type parameter on a class
#[derive(Debug, Clone)]
pub struct GenericParam {
    /// TypeVar ID
    pub id: TypeVarId,
    /// Parameter name (e.g., "T", "K", "V")
    pub name: String,
    /// Variance of this type parameter
    pub variance: Variance,
    /// Optional upper bound
    pub bound: Option<Type>,
    /// Type constraints (if any)
    pub constraints: Vec<Type>,
}

impl GenericParam {
    pub fn new(id: TypeVarId, name: String) -> Self {
        Self {
            id,
            name,
            variance: Variance::Invariant,
            bound: None,
            constraints: Vec::new(),
        }
    }

    pub fn with_variance(mut self, variance: Variance) -> Self {
        self.variance = variance;
        self
    }

    pub fn with_bound(mut self, bound: Type) -> Self {
        self.bound = Some(bound);
        self
    }

    pub fn with_constraints(mut self, constraints: Vec<Type>) -> Self {
        self.constraints = constraints;
        self
    }
}

/// Information about a class definition
#[derive(Debug, Clone, Default)]
pub struct ClassInfo {
    /// Class name
    pub name: String,
    /// Base classes (may include generic instantiations like "Generic[T]")
    pub bases: Vec<String>,
    /// Generic type parameters (in declaration order)
    pub generic_params: Vec<GenericParam>,
    /// Instance attributes (name -> type)
    pub attributes: HashMap<String, Type>,
    /// Methods (name -> callable type)
    pub methods: HashMap<String, Type>,
    /// Class variables (name -> type)
    pub class_vars: HashMap<String, Type>,
    /// Whether this class is a Protocol
    pub is_protocol: bool,
    /// Whether this class is abstract
    pub is_abstract: bool,
}

impl ClassInfo {
    pub fn new(name: String) -> Self {
        Self {
            name,
            bases: Vec::new(),
            generic_params: Vec::new(),
            attributes: HashMap::new(),
            methods: HashMap::new(),
            class_vars: HashMap::new(),
            is_protocol: false,
            is_abstract: false,
        }
    }

    /// Check if this class is generic (has type parameters)
    pub fn is_generic(&self) -> bool {
        !self.generic_params.is_empty()
    }

    /// Get the number of type parameters
    pub fn arity(&self) -> usize {
        self.generic_params.len()
    }

    /// Get a type parameter by name
    pub fn get_type_param(&self, name: &str) -> Option<&GenericParam> {
        self.generic_params.iter().find(|p| p.name == name)
    }

    /// Get a type parameter by index
    pub fn get_type_param_by_index(&self, index: usize) -> Option<&GenericParam> {
        self.generic_params.get(index)
    }

    /// Get variance for a type parameter by index
    pub fn variance_at(&self, index: usize) -> Variance {
        self.generic_params
            .get(index)
            .map(|p| p.variance)
            .unwrap_or(Variance::Invariant)
    }

    /// Add a generic type parameter
    pub fn add_type_param(&mut self, param: GenericParam) {
        self.generic_params.push(param);
    }

    /// Get attribute type (checks instance attrs, then methods, then class vars)
    pub fn get_attribute(&self, name: &str) -> Option<&Type> {
        self.attributes
            .get(name)
            .or_else(|| self.methods.get(name))
            .or_else(|| self.class_vars.get(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_info_new() {
        let info = ClassInfo::new("MyClass".to_string());
        assert_eq!(info.name, "MyClass");
        assert!(!info.is_generic());
        assert_eq!(info.arity(), 0);
    }

    #[test]
    fn test_generic_class() {
        let mut info = ClassInfo::new("Container".to_string());
        let param =
            GenericParam::new(TypeVarId(0), "T".to_string()).with_variance(Variance::Covariant);
        info.add_type_param(param);

        assert!(info.is_generic());
        assert_eq!(info.arity(), 1);
        assert_eq!(info.variance_at(0), Variance::Covariant);
        assert!(info.get_type_param("T").is_some());
    }

    #[test]
    fn test_multiple_type_params() {
        let mut info = ClassInfo::new("Dict".to_string());
        info.add_type_param(GenericParam::new(TypeVarId(0), "K".to_string()));
        info.add_type_param(
            GenericParam::new(TypeVarId(1), "V".to_string()).with_variance(Variance::Covariant),
        );

        assert_eq!(info.arity(), 2);
        assert_eq!(info.variance_at(0), Variance::Invariant);
        assert_eq!(info.variance_at(1), Variance::Covariant);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/type_inference/class_info.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/type_inference/class_info.rs` captured during libs codegen standardization.
```
