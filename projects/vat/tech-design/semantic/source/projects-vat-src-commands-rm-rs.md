---
id: vat-source-projects-vat-src-commands-rm-rs
summary: Source replay payload for projects/vat/src/commands/rm.rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: copy-on-write-fork-and-snapshot-lifecycle
    claim: copy-on-write-fork-and-snapshot-lifecycle
    coverage: full
    rationale: "This source replay TD preserves vat's copy-on-write workspace, agent-legible state, resource isolation, and host GPU behavior."
---

# Source TD: projects/vat/src/commands/rm.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/src/commands/rm.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `exec` | projects/vat/src/commands/rm.rs | function | pub | 13 | exec(id: String) -> Result<ExitCode> |
## Source
<!-- type: source lang: rust -->

`````rust
//! `vat rm <id>` — delete a vat and its workspace.

use std::process::ExitCode;

use anyhow::Result;

use crate::event::{Event, EventKind};
use crate::store;

/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-commands-rm-rs.md#source
pub fn exec(id: String) -> Result<ExitCode> {
    // Best-effort: log the removal before the directory disappears, so a
    // shared events sink (future) still sees it.
    if let Ok(vat) = store::load(&id) {
        let _ = vat.log(Event::new(EventKind::Removed, format!("removing {id}")));
    }
    store::remove(&id)?;
    println!("removed {id}");
    Ok(ExitCode::SUCCESS)
}
`````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: source
changes:
  - path: "projects/vat/src/commands/rm.rs"
    action: modify
    section: source
    description: |
      Historical source replay payload retained as semantic context. Active
      codegen ownership moved to projects/vat/tech-design/semantic/vat-commands.md#schema.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-vat-src-commands-rm-rs-source-replay-superseded>"
```
