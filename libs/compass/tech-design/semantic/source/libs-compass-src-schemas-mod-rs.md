---
id: libs-compass-src-schemas-mod-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/schemas/mod.rs`.
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

# Standardized libs/compass/src/schemas/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/schemas/mod.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `frontmatter` | libs/compass/src/schemas/mod.rs | module | pub | 6 | pub mod frontmatter; |
| `SchemaRegistry` | libs/compass/src/schemas/mod.rs | struct | pub | 17 | pub struct SchemaRegistry { |
| `new` | libs/compass/src/schemas/mod.rs | function | pub | 33 | pub fn new(k8s_version: &str) -> Self { |
| `global` | libs/compass/src/schemas/mod.rs | function | pub | 59 | pub fn global() -> &'static SchemaRegistry { |
| `validate_k8s` | libs/compass/src/schemas/mod.rs | function | pub | 71 | pub fn validate_k8s(&self, value: &Value, _version: &str) -> Vec<Diagnostic> { |
| `validate_gitlab_ci` | libs/compass/src/schemas/mod.rs | function | pub | 107 | pub fn validate_gitlab_ci(&self, value: &Value) -> Vec<Diagnostic> { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Schema-based validation for Kubernetes manifests and GitLab CI configs.
//!
//! Builds JSON Schemas programmatically in Rust (no bundled JSON files)
//! and validates parsed YAML/JSON values against them using the `jsonschema` crate.

pub mod frontmatter;
mod gitlab;
mod k8s;

use crate::diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Position, Range};
use jsonschema::Validator;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Schema registry — lazily compiles and caches JSON Schema validators.
pub struct SchemaRegistry {
    /// K8s validators keyed by (kind, version). Version is stored for future
    /// differentiation; currently all versions share the same schema.
    k8s_validators: HashMap<String, Validator>,
    /// Single GitLab CI validator.
    gitlab_ci_validator: Validator,
    /// The K8s version used (stored for diagnostics).
    _k8s_version: String,
}

/// Global singleton so callers don't have to thread the registry everywhere.
static GLOBAL_REGISTRY: OnceLock<SchemaRegistry> = OnceLock::new();

impl SchemaRegistry {
    /// Build a new registry with compiled validators for all K8s resource
    /// kinds and the GitLab CI schema.
    pub fn new(k8s_version: &str) -> Self {
        let mut k8s_validators = HashMap::new();

        for (kind, schema) in k8s::build_all_schemas() {
            match Validator::new(&schema) {
                Ok(v) => {
                    k8s_validators.insert(kind, v);
                }
                Err(e) => {
                    tracing::warn!("Failed to compile K8s schema for {}: {}", kind, e);
                }
            }
        }

        let gitlab_schema = gitlab::build_gitlab_ci_schema();
        let gitlab_ci_validator =
            Validator::new(&gitlab_schema).expect("GitLab CI schema must compile");

        Self {
            k8s_validators,
            gitlab_ci_validator,
            _k8s_version: k8s_version.to_string(),
        }
    }

    /// Get (or create) the global shared registry.
    pub fn global() -> &'static SchemaRegistry {
        GLOBAL_REGISTRY.get_or_init(|| SchemaRegistry::new("1.30"))
    }

    // -----------------------------------------------------------------
    // K8s validation
    // -----------------------------------------------------------------

    /// Validate a parsed K8s manifest value.
    ///
    /// The `kind` field is extracted from the value to select the right schema.
    /// Returns diagnostics with JSON-path locations.
    pub fn validate_k8s(&self, value: &Value, _version: &str) -> Vec<Diagnostic> {
        let kind = match value.get("kind").and_then(|v| v.as_str()) {
            Some(k) => k.to_string(),
            None => return Vec::new(), // no kind — handled by K8001
        };

        let validator = match self.k8s_validators.get(&kind) {
            Some(v) => v,
            None => return Vec::new(), // unknown kind — no schema
        };

        let mut diagnostics = Vec::new();
        for error in validator.iter_errors(value) {
            let path = error.instance_path.to_string();
            let msg = if path.is_empty() {
                format!("[{}] {}", kind, error)
            } else {
                format!("[{}] {}: {}", kind, path, error)
            };

            diagnostics.push(Diagnostic::new(
                Range::new(Position::new(0, 0), Position::new(0, 1)),
                DiagnosticSeverity::Warning,
                "K8002",
                DiagnosticCategory::Logic,
                msg,
            ));
        }
        diagnostics
    }

    // -----------------------------------------------------------------
    // GitLab CI validation
    // -----------------------------------------------------------------

    /// Validate a parsed GitLab CI config value (top-level object).
    pub fn validate_gitlab_ci(&self, value: &Value) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for error in self.gitlab_ci_validator.iter_errors(value) {
            let path = error.instance_path.to_string();
            let msg = if path.is_empty() {
                format!("GitLab CI schema: {}", error)
            } else {
                format!("GitLab CI schema {}: {}", path, error)
            };

            diagnostics.push(Diagnostic::new(
                Range::new(Position::new(0, 0), Position::new(0, 1)),
                DiagnosticSeverity::Warning,
                "GL002",
                DiagnosticCategory::Logic,
                msg,
            ));
        }
        diagnostics
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/schemas/mod.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/schemas/mod.rs` captured during libs codegen standardization.
```
