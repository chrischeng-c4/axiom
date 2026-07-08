---
id: libs-compass-src-gen-registry-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/gen/registry.rs`.
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

# Standardized libs/compass/src/gen/registry.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/gen/registry.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `GeneratorRegistry` | libs/compass/src/gen/registry.rs | struct | pub | 13 | pub struct GeneratorRegistry { |
| `new` | libs/compass/src/gen/registry.rs | function | pub | 19 | pub fn new() -> Self { |
| `register` | libs/compass/src/gen/registry.rs | function | pub | 26 | pub fn register(&mut self, gen: Box<dyn CodeGenerator>) { |
| `find` | libs/compass/src/gen/registry.rs | function | pub | 31 | pub fn find(&self, spec: &serde_json::Value) -> Option<&dyn CodeGenerator> { |
| `generate` | libs/compass/src/gen/registry.rs | function | pub | 43 | pub fn generate( |
| `list` | libs/compass/src/gen/registry.rs | function | pub | 62 | pub fn list(&self) -> Vec<&str> { |
| `len` | libs/compass/src/gen/registry.rs | function | pub | 67 | pub fn len(&self) -> usize { |
| `is_empty` | libs/compass/src/gen/registry.rs | function | pub | 72 | pub fn is_empty(&self) -> bool { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Generator registry — dispatches SpecIR to the correct generator.
//!
//! Instead of ad-hoc generator selection, consumers call
//! `registry.generate(spec_ir, ctx)` and the registry finds the first
//! generator whose `can_generate()` returns `true`.
//!
//! SpecIR types now live in `sdd::generate`. This registry accepts
//! `serde_json::Value` to avoid a circular crate dependency.

use super::traits::{CodeGenerator, GenContext, GenError, GenResult, GeneratedCode};

/// Registry holding all registered [`CodeGenerator`] implementations.
pub struct GeneratorRegistry {
    generators: Vec<Box<dyn CodeGenerator>>,
}

impl GeneratorRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            generators: Vec::new(),
        }
    }

    /// Register a generator.
    pub fn register(&mut self, gen: Box<dyn CodeGenerator>) {
        self.generators.push(gen);
    }

    /// Find the first generator that can handle the given SpecIR (as JSON value).
    pub fn find(&self, spec: &serde_json::Value) -> Option<&dyn CodeGenerator> {
        self.generators
            .iter()
            .find(|g| g.can_generate(spec))
            .map(|g| g.as_ref())
    }

    /// Generate code by dispatching to the first matching generator.
    ///
    /// The `spec` parameter is a serialized SpecIR value. The registry
    /// finds the first generator whose `can_generate()` returns `true`
    /// and delegates to it.
    pub fn generate(
        &self,
        spec: &serde_json::Value,
        ctx: &GenContext,
    ) -> GenResult<Vec<GeneratedCode>> {
        let kind = spec
            .get("kind")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let gen = self.find(spec).ok_or_else(|| {
            GenError::UnsupportedFeature(format!(
                "no generator registered for SpecIR kind '{}'",
                kind
            ))
        })?;
        gen.generate(spec, ctx)
    }

    /// List names of all registered generators.
    pub fn list(&self) -> Vec<&str> {
        self.generators.iter().map(|g| g.name()).collect()
    }

    /// Number of registered generators.
    pub fn len(&self) -> usize {
        self.generators.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.generators.is_empty()
    }
}

impl Default for GeneratorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::ir::DataModelSpec;

    /// Stub generator for testing
    struct StubApiGen;

    impl CodeGenerator for StubApiGen {
        fn name(&self) -> &str {
            "stub-api"
        }

        fn can_generate(&self, spec: &serde_json::Value) -> bool {
            spec.get("kind").and_then(|v| v.as_str()) == Some("api")
        }

        fn generate(
            &self,
            _spec: &serde_json::Value,
            _ctx: &GenContext,
        ) -> GenResult<Vec<GeneratedCode>> {
            Ok(vec![GeneratedCode::new(
                "stub",
                "// generated",
                crate::gen::traits::Language::Rust,
            )])
        }

        fn generate_data_models(
            &self,
            _spec: &DataModelSpec,
            _ctx: &GenContext,
        ) -> GenResult<Vec<GeneratedCode>> {
            Err(GenError::UnsupportedFeature("use generate()".into()))
        }
    }

    #[test]
    fn test_registry_dispatch() {
        let mut registry = GeneratorRegistry::new();
        registry.register(Box::new(StubApiGen));

        let spec = serde_json::json!({
            "kind": "api",
            "schema": { "title": "Test" }
        });
        let ctx = GenContext::default();

        let result = registry.generate(&spec, &ctx);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_registry_no_match() {
        let registry = GeneratorRegistry::new();
        let spec = serde_json::json!({
            "kind": "api",
            "schema": {}
        });
        let ctx = GenContext::default();

        let result = registry.generate(&spec, &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_list() {
        let mut registry = GeneratorRegistry::new();
        registry.register(Box::new(StubApiGen));
        assert_eq!(registry.list(), vec!["stub-api"]);
        assert_eq!(registry.len(), 1);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/gen/registry.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/gen/registry.rs` captured during libs codegen standardization.
```
