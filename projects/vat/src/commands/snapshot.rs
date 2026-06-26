// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-commands-snapshot-rs.md#rust-source-unit
// CODEGEN-BEGIN
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
// CODEGEN-END
// SPEC-MANAGED: projects/vat/tech-design/logic/vat-td-ast-promote-remaining-grouped-source-units.md#rust-source-unit
// CODEGEN-BEGIN

// CODEGEN-END
