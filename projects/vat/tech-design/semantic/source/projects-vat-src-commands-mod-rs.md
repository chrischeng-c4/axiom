---
id: vat-source-projects-vat-src-commands-mod-rs
summary: Source replay payload for projects/vat/src/commands/mod.rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: copy-on-write-fork-and-snapshot-lifecycle
    claim: copy-on-write-fork-and-snapshot-lifecycle
    coverage: full
    rationale: "This source replay TD preserves vat's copy-on-write workspace, agent-legible state, resource isolation, and host GPU behavior."
---

# Source TD: projects/vat/src/commands/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/src/commands/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `diff` | projects/vat/src/commands/mod.rs | module | pub | 10 |  |
| `gpu` | projects/vat/src/commands/mod.rs | module | pub | 11 |  |
| `llm` | projects/vat/src/commands/mod.rs | module | pub | 12 |  |
| `logs` | projects/vat/src/commands/mod.rs | module | pub | 13 |  |
| `ls` | projects/vat/src/commands/mod.rs | module | pub | 14 |  |
| `print_json` | projects/vat/src/commands/mod.rs | function | pub | 25 | print_json(value: &T, compact: bool) -> Result<()> |
| `rm` | projects/vat/src/commands/mod.rs | module | pub | 15 |  |
| `run` | projects/vat/src/commands/mod.rs | module | pub | 16 |  |
| `snapshot` | projects/vat/src/commands/mod.rs | module | pub | 17 |  |
| `state` | projects/vat/src/commands/mod.rs | module | pub | 18 |  |
## Source
<!-- type: source lang: rust -->

`````rust
//! Command implementations, one file per verb.
//!
//! The CLI layer ([`crate::cli`]) parses arguments and dispatches here; each
//! module owns the logic for one verb and returns an [`std::process::ExitCode`]
//! so the binary can propagate a meaningful status (notably: `vat run`
//! forwards the child's exit code).

pub mod diff;
pub mod gpu;
pub mod llm;
pub mod logs;
pub mod ls;
pub mod rm;
pub mod run;
pub mod snapshot;
pub mod state;

use anyhow::Result;

/// Print a value as JSON to stdout — pretty by default, single-line when
/// `compact`. Used wherever a verb has a machine-readable mode.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-commands-mod-rs.md#source
pub fn print_json<T: serde::Serialize>(value: &T, compact: bool) -> Result<()> {
    let s = if compact {
        serde_json::to_string(value)?
    } else {
        serde_json::to_string_pretty(value)?
    };
    println!("{s}");
    Ok(())
}
`````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: source
changes:
  - path: "projects/vat/src/commands/mod.rs"
    action: modify
    section: source
    description: |
      Historical source replay payload retained as semantic context. Active
      codegen ownership moved to projects/vat/tech-design/semantic/vat-commands.md#schema.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-vat-src-commands-mod-rs-source-replay-superseded>"
```
