---
id: vat-source-projects-vat-src-commands-snapshot-rs
summary: >
  rust-source-unit TD AST payload for projects/vat/src/commands/snapshot.rs.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This rust-source-unit TD preserves vat source ownership while migrating #39 off group-level source replay."
---

# Standardized projects/vat/src/commands/snapshot.rs

## Overview
<!-- type: overview lang: markdown -->

Rust source-unit TD for `projects/vat/src/commands/snapshot.rs`, captured during #39 vat migration onto td_ast lossless source generation.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! `vat snapshot <id>` and `vat fork <id>` — branch a running environment.
//!
//! Both copy-on-write clone an existing vat's rootfs into a new vat, carrying
//! lineage so the fork tree is legible:
//!
//! - **snapshot** produces a frozen label (`Status::Snapshot`) — a restore
//!   point you don't run in.
//! - **fork** produces a fresh, runnable working copy — for trying a second
//!   approach from the same starting point.
//!
//! This is the "git for a live environment" half of vat: cheap branching means
//! an agent can explore, fail, and roll back without rebuilding.

use std::process::ExitCode;

use anyhow::{Context, Result};

use crate::event::{Event, EventKind};
use crate::spec::{Base, EnvSpec};
use crate::state::Status;
use crate::{id, store};

/// Clone `parent_id` into a new vat. `freeze` marks it as a snapshot.
fn branch(parent_id: &str, name: Option<String>, freeze: bool) -> Result<store::Vat> {
    let parent = store::load(parent_id).with_context(|| format!("unknown vat {parent_id}"))?;

    let mut lineage = parent.meta.lineage.clone();
    lineage.push(parent.meta.id.clone());

    // Inherit the parent's spec but repoint base at the parent vat.
    let spec = EnvSpec {
        base: Some(Base::Vat(parent.meta.id.clone())),
        ..parent.meta.spec.clone()
    };

    let new_id = id::fresh();
    let mut child = store::create(&new_id, name, spec, Some(&parent.rootfs()), lineage)
        .context("create forked vat")?;

    let (kind, msg) = if freeze {
        child.meta.status = Status::Snapshot;
        (EventKind::Snapshot, format!("snapshot of {parent_id}"))
    } else {
        (EventKind::Fork, format!("fork of {parent_id}"))
    };
    child.save()?;
    child.log(Event::new(kind, msg))?;
    Ok(child)
}

/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-commands-snapshot-rs.md#source
pub fn snapshot(id: String, name: Option<String>) -> Result<ExitCode> {
    let child = branch(&id, name, true)?;
    println!("snapshot {} (frozen, from {id})", child.meta.id);
    Ok(ExitCode::SUCCESS)
}

/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-commands-snapshot-rs.md#source
pub fn fork(id: String, name: Option<String>) -> Result<ExitCode> {
    let child = branch(&id, name, false)?;
    println!(
        "fork {} (runnable, from {id})\n→ vat run --from {} -- <command>",
        child.meta.id, child.meta.id
    );
    Ok(ExitCode::SUCCESS)
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/commands/snapshot.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/vat/src/commands/snapshot.rs` captured during #39 vat standardization.
```
