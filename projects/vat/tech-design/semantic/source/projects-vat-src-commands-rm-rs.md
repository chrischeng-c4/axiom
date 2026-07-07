---
id: vat-source-projects-vat-src-commands-rm-rs
summary: >
  rust-source-unit TD AST payload for projects/vat/src/commands/rm.rs.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This rust-source-unit TD preserves vat source ownership while migrating #39 off group-level source replay."
---

# Standardized projects/vat/src/commands/rm.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/src/commands/rm.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `exec` | projects/vat/src/commands/rm.rs | function | pub | 13 | exec(id: String) -> Result<ExitCode> |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/commands/rm.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/vat/src/commands/rm.rs` captured during #39 vat standardization.
```
