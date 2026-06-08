---
id: vat-source-projects-vat-src-commands-state-rs
summary: Source replay payload for projects/vat/src/commands/state.rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: copy-on-write-fork-and-snapshot-lifecycle
    claim: copy-on-write-fork-and-snapshot-lifecycle
    coverage: full
    rationale: "This source replay TD preserves vat's copy-on-write workspace, agent-legible state, resource isolation, and host GPU behavior."
---

# Source TD: projects/vat/src/commands/state.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/src/commands/state.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `exec` | projects/vat/src/commands/state.rs | function | pub | 15 | exec(id: String, compact: bool) -> Result<ExitCode> |
## Source
<!-- type: source lang: rust -->

`````rust
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
`````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: source
changes:
  - path: "projects/vat/src/commands/state.rs"
    action: modify
    section: source
    description: |
      Historical source replay payload retained as semantic context. Active
      codegen ownership moved to projects/vat/tech-design/semantic/vat-commands.md#schema.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-vat-src-commands-state-rs-source-replay-superseded>"
```
