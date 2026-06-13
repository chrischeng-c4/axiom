---
id: projects-guard-src-lib-rs
summary: Lossless rust-source-unit coverage for `projects/guard/src/lib.rs`.
capability_refs:
  - id: static-security-scan
    role: primary
    gap: compass-backed-diagnostic-scan
    claim: compass-backed-diagnostic-scan
    coverage: full
    rationale: "The source unit implements guard's compass-backed static security scan capability."
fill_sections: [overview, source, changes]
---

# Standardized projects/guard/src/lib.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/guard/src/lib.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `evidence` | projects/guard/src/lib.rs | module | pub | 9 |  |
| `report` | projects/guard/src/lib.rs | module | pub | 10 |  |
| `scan` | projects/guard/src/lib.rs | module | pub | 11 |  |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! guard — security posture gate for the cclab ecosystem.
//!
//! `guard` owns security policy and report semantics. It consumes `compass`
//! for AST/lint/data-flow primitives, then emits one agent-readable security
//! report per run.

pub mod evidence;
pub mod report;
pub mod scan;

pub use evidence::{EvidenceCommand, EvidenceStatus, ExternalEvidence};
pub use report::{
    Completion, Finding, GuardReport, IntegrationMap, Location, OverallStatus, Severity, Summary,
    SCHEMA_VERSION,
};
pub use scan::{default_languages, scan_path, PolicyProfile, ScanOptions};
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/guard/src/lib.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/guard/src/lib.rs` captured during guard standardization onto the codegen ladder.
```
