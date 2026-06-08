---
number: 803
title: "feat(lens): bundle JSON schemas for K8s and GitLab CI validation"
state: open
labels: [enhancement, P2, crate:lens]
group: "lint-and-dispatch"
---

# #803 — feat(lens): bundle JSON schemas for K8s and GitLab CI validation

## Context

Spec defined bundled JSON schemas for offline K8s/GitLab CI validation (R8, R9). Currently checkers use source-line heuristics only. Schema validation would catch structural errors that heuristics miss.

## Scope

### Kubernetes
- Bundle K8s API schemas for 1.28, 1.29, 1.30
- `include_bytes!()` for offline use
- `--k8s-version` flag to select schema version
- Validate manifest structure against schema (apiVersion + kind → correct spec)
- `jsonschema` crate already in Cargo.toml

### GitLab CI
- Bundle official GitLab CI JSON schema
- Validate top-level structure, job keywords, variable syntax

## Architecture

```rust
// schemas/mod.rs
pub struct SchemaRegistry {
    k8s_schemas: HashMap<String, serde_json::Value>,  // "1.28", "1.29", "1.30"
    gitlab_ci_schema: serde_json::Value,
}

impl SchemaRegistry {
    pub fn new() -> Self {
        // Load from include_bytes!
    }
    pub fn validate_k8s(&self, value: &Value, version: &str) -> Vec<Diagnostic>;
    pub fn validate_gitlab_ci(&self, value: &Value) -> Vec<Diagnostic>;
}
```

## Files to create
- CREATE `crates/cclab-lens/src/schemas/mod.rs`
- CREATE `crates/cclab-lens/src/schemas/k8s_1_28.json` (or compressed)
- CREATE `crates/cclab-lens/src/schemas/k8s_1_29.json`
- CREATE `crates/cclab-lens/src/schemas/k8s_1_30.json`
- CREATE `crates/cclab-lens/src/schemas/gitlab_ci.json`
- MODIFY K8s and GitLab CI checkers to use schema validation
