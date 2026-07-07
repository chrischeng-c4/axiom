---
id: vat-source-projects-vat-src-commands-state-rs
summary: >
  rust-source-unit TD AST payload for projects/vat/src/commands/state.rs.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This rust-source-unit TD preserves vat source ownership while migrating #39 off group-level source replay."
---

# Standardized projects/vat/src/commands/state.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/src/commands/state.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `exec` | projects/vat/src/commands/state.rs | function | pub | 15 | exec(id: String, compact: bool) -> Result<ExitCode> |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! `vat state <id>` — print the full agent-legible [`VatState`] as JSON.
//!
//! This is the command an agent calls to understand a vat. Output is pretty
//! JSON by default (readable in a transcript) or single-line with `--compact`.

use std::process::ExitCode;

use anyhow::Result;

use crate::store;

/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-commands-state-rs.md#source
pub fn exec(id: String, compact: bool) -> Result<ExitCode> {
    let vat = store::load(&id)?;
    let state = vat.project()?;
    crate::commands::print_json(&state, compact)?;
    Ok(ExitCode::SUCCESS)
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/commands/state.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/vat/src/commands/state.rs` captured during #39 vat standardization.
```
