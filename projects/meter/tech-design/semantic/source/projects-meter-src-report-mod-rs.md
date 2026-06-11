---
id: projects-meter-src-report-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: json-default-report-envelope-and-findings
    claim: json-default-report-envelope-and-findings
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/src/report/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/report/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `builder` | projects/meter/src/report/mod.rs | module | pub | 15 |  |
| `emit` | projects/meter/src/report/mod.rs | module | pub | 16 |  |
| `env` | projects/meter/src/report/mod.rs | module | pub | 17 |  |
| `envelope` | projects/meter/src/report/mod.rs | module | pub | 18 |  |
| `finding` | projects/meter/src/report/mod.rs | module | pub | 19 |  |
| `persist` | projects/meter/src/report/mod.rs | module | pub | 20 |  |
| `producer` | projects/meter/src/report/mod.rs | module | pub | 21 |  |
| `schema` | projects/meter/src/report/mod.rs | module | pub | 22 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/meter/src/report/mod.rs -->
````rust
//! Agent-first report layer — the center of the `meter` agent surface.
//!
//! Every `meter` verb funnels its result through [`ReportBuilder::finalize`] into a
//! single self-describing [`MeterReport`], which [`emit`] prints as exactly one JSON
//! document on stdout (diagnostics go to stderr). Populator verbs persist their
//! report to `.meter/last-report.json` via [`persist::write_last_report`] so the
//! read-only `report`/`state` verb can re-project with zero engine work.
//!
//! This module is ALWAYS compiled (not feature-gated): it is a pure data +
//! serialization layer with no spawn/IO side effects beyond `emit`/`persist`,
//! and the `meter` crate stays a clean rlib for its mamba + pgkit consumers.

pub mod builder;
pub mod emit;
pub mod env;
pub mod envelope;
pub mod finding;
pub mod persist;
pub mod producer;
pub mod schema;

// Public surface re-exports.
pub use builder::ReportBuilder;
pub use emit::{diag, emit, render};
pub use envelope::{
    Completion, EnvBlock, FindingsSummary, MeterReport, OverallStatus, RunnerRecord, SCHEMA_VERSION,
};
pub use finding::{finding_id, Finding, Invoke, Kind, Location, Severity};
pub use persist::{read_last_report, write_last_report};
pub use producer::IntoFindings;
pub use schema::{catalog, json_schema};
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/report/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/report/mod.rs` captured during meter full-codegen standardization.
```
