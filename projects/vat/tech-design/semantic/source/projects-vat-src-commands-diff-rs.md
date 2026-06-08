---
id: vat-source-projects-vat-src-commands-diff-rs
summary: Source replay payload for projects/vat/src/commands/diff.rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: copy-on-write-fork-and-snapshot-lifecycle
    claim: copy-on-write-fork-and-snapshot-lifecycle
    coverage: full
    rationale: "This source replay TD preserves vat's copy-on-write workspace, agent-legible state, resource isolation, and host GPU behavior."
---

# Source TD: projects/vat/src/commands/diff.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/src/commands/diff.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `exec` | projects/vat/src/commands/diff.rs | function | pub | 15 | exec(id: String, json: bool) -> Result<ExitCode> |
## Source
<!-- type: source lang: rust -->

`````rust
//! `vat diff <id>` — full filesystem changes vs. the vat's base.
//!
//! Where `vat state` shows a bounded sample, `diff` shows every changed path.
//! `--json` emits the complete [`ChangeSet`].

use std::process::ExitCode;

use anyhow::Result;

use crate::store;

/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-commands-diff-rs.md#source
pub fn exec(id: String, json: bool) -> Result<ExitCode> {
    let vat = store::load(&id)?;
    let changes = vat.changes()?;

    if json {
        crate::commands::print_json(&changes, false)?;
        return Ok(ExitCode::SUCCESS);
    }

    if changes.is_empty() {
        println!("{id}: no changes vs base");
        return Ok(ExitCode::SUCCESS);
    }
    for p in &changes.added {
        println!("A  {p}");
    }
    for p in &changes.modified {
        println!("M  {p}");
    }
    for p in &changes.deleted {
        println!("D  {p}");
    }
    println!("{}", changes.oneline());
    Ok(ExitCode::SUCCESS)
}
`````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: source
changes:
  - path: "projects/vat/src/commands/diff.rs"
    action: modify
    section: source
    description: |
      Historical source replay payload retained as semantic context. Active
      codegen ownership moved to projects/vat/tech-design/semantic/vat-commands.md#schema.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-vat-src-commands-diff-rs-source-replay-superseded>"
```
