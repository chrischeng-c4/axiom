---
id: vat-source-projects-vat-src-commands-mod-rs
summary: >
  rust-source-unit TD AST payload for projects/vat/src/commands/mod.rs.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This rust-source-unit TD preserves vat source ownership while migrating #39 off group-level source replay."
---

# Standardized projects/vat/src/commands/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/src/commands/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `cluster` | projects/vat/src/commands/mod.rs | module | pub | 10 |  |
| `diff` | projects/vat/src/commands/mod.rs | module | pub | 11 |  |
| `emulator` | projects/vat/src/commands/mod.rs | module | pub | 12 |  |
| `gpu` | projects/vat/src/commands/mod.rs | module | pub | 13 |  |
| `llm` | projects/vat/src/commands/mod.rs | module | pub | 14 |  |
| `logs` | projects/vat/src/commands/mod.rs | module | pub | 15 |  |
| `ls` | projects/vat/src/commands/mod.rs | module | pub | 16 |  |
| `print_json` | projects/vat/src/commands/mod.rs | function | pub | 27 | print_json(value: &T, compact: bool) -> Result<()> |
| `rm` | projects/vat/src/commands/mod.rs | module | pub | 17 |  |
| `run` | projects/vat/src/commands/mod.rs | module | pub | 18 |  |
| `snapshot` | projects/vat/src/commands/mod.rs | module | pub | 19 |  |
| `state` | projects/vat/src/commands/mod.rs | module | pub | 20 |  |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Command implementations, one file per verb.
//!
//! The CLI layer ([`crate::cli`]) parses arguments and dispatches here; each
//! module owns the logic for one verb and returns an [`std::process::ExitCode`]
//! so the binary can propagate a meaningful status (notably: `vat run`
//! forwards the child's exit code).

pub mod cluster;
pub mod diff;
pub mod emulator;
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/commands/mod.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/vat/src/commands/mod.rs` captured during #39 vat standardization.
```
