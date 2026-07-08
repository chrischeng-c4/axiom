---
id: libs-cli-std-src-report-issue-rs
summary: Lossless rust-source-unit coverage for `libs/cli-std/src/report_issue.rs`.
capability_refs:
  - id: standard-agent-cli-commands
    role: primary
    claim: standard-agent-cli-commands-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Cli Std library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/cli-std/src/report_issue.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/cli-std/src/report_issue.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `run` | libs/cli-std/src/report_issue.rs | re-export | pub | 10 | pub use crate::issue::{create as run, CreateOptions as Options}; |
| `Options` | libs/cli-std/src/report_issue.rs | re-export | pub | 10 | pub use crate::issue::{create as run, CreateOptions as Options}; |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Deprecated compatibility alias of [`crate::issue`].
//!
//! The `<tool> report-issue` command is being replaced by the `<tool> issue
//! <verb>` group (`search` / `view` / `create`), so the logic now lives in
//! [`crate::issue`]. This shim keeps tools that have not yet migrated their CLI
//! surface (keep / loom / lumen) building unchanged — they call
//! `cli_std::report_issue::run(&tool, Options { .. })`, which forwards to
//! [`crate::issue::create`]. Drop this module once those tools adopt `issue`.

pub use crate::issue::{create as run, CreateOptions as Options};
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/cli-std/src/report_issue.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/cli-std/src/report_issue.rs` captured during libs codegen standardization.
```
