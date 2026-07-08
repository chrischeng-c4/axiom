---
id: libs-compass-src-schemas-gitlab-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/schemas/gitlab.rs`.
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

# Standardized libs/compass/src/schemas/gitlab.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/schemas/gitlab.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `build_gitlab_ci_schema` | libs/compass/src/schemas/gitlab.rs | function | pub | 17 | pub(super) fn build_gitlab_ci_schema() -> Value { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Programmatic GitLab CI JSON-Schema definition.
//!
//! Defines valid top-level keys and the job structure so that `GL002`
//! (unknown keywords) can also be caught via schema validation.

use serde_json::{json, Value};

/// Build the GitLab CI config schema.
///
/// The schema validates:
/// - Top-level keys are either reserved global keywords or job definitions.
/// - Job objects have the correct keyword set.
///
/// Because `additionalProperties` + `patternProperties` is the idiomatic way
/// to allow arbitrary job names while constraining their shape, we use that
/// approach.
pub(super) fn build_gitlab_ci_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "stages": {
                "type": "array",
                "items": { "type": "string" }
            },
            "variables": { "type": "object" },
            "default": { "type": "object" },
            "include": {},
            "image": {},
            "services": { "type": "array" },
            "before_script": {},
            "after_script": {},
            "cache": {},
            "workflow": { "type": "object" },
            "pages": { "type": "object" }
        },
        "additionalProperties": {
            "anyOf": [
                job_schema(),
                // Allow non-object values for edge cases (anchors, etc.)
                { "type": ["string", "number", "boolean", "null", "array"] }
            ]
        }
    })
}

/// Schema for a single CI job object.
fn job_schema() -> Value {
    let known_keywords = vec![
        "script",
        "stage",
        "image",
        "services",
        "variables",
        "rules",
        "only",
        "except",
        "needs",
        "artifacts",
        "cache",
        "before_script",
        "after_script",
        "allow_failure",
        "when",
        "timeout",
        "retry",
        "tags",
        "environment",
        "extends",
        "trigger",
        "parallel",
        "resource_group",
        "release",
        "coverage",
        "dependencies",
        "interruptible",
        "secrets",
        "pages",
        "inherit",
        "dast_configuration",
        "id_tokens",
    ];

    // Build the properties map
    let mut props = serde_json::Map::new();
    for kw in &known_keywords {
        props.insert(kw.to_string(), json!({}));
    }

    json!({
        "type": "object",
        "properties": props,
        "additionalProperties": false
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::SchemaRegistry;

    #[test]
    fn test_gitlab_schema_compiles() {
        let schema = build_gitlab_ci_schema();
        jsonschema::Validator::new(&schema).expect("GitLab CI schema must compile");
    }

    #[test]
    fn test_valid_ci_config() {
        let registry = SchemaRegistry::new("1.30");
        let config = json!({
            "stages": ["build", "test", "deploy"],
            "variables": { "CI": "true" },
            "build_job": {
                "stage": "build",
                "script": ["make build"],
                "tags": ["docker"]
            },
            "test_job": {
                "stage": "test",
                "script": ["make test"],
                "needs": ["build_job"],
                "artifacts": { "paths": ["coverage/"] }
            }
        });
        let diags = registry.validate_gitlab_ci(&config);
        assert!(diags.is_empty(), "valid config should pass: {:?}", diags);
    }

    #[test]
    fn test_unknown_job_keyword() {
        let registry = SchemaRegistry::new("1.30");
        let config = json!({
            "stages": ["build"],
            "build_job": {
                "stage": "build",
                "script": ["make"],
                "invalid_key": true
            }
        });
        let diags = registry.validate_gitlab_ci(&config);
        assert!(
            !diags.is_empty(),
            "should detect unknown keyword 'invalid_key'"
        );
        assert!(diags
            .iter()
            .any(|d| d.message.contains("invalid_key") || d.message.contains("additional")));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/schemas/gitlab.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/schemas/gitlab.rs` captured during libs codegen standardization.
```
