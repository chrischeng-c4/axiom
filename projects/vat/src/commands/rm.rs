// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-commands-rm-rs.md#rust-source-unit
// CODEGEN-BEGIN
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
// CODEGEN-END
// SPEC-MANAGED: projects/vat/tech-design/logic/vat-td-ast-promote-remaining-grouped-source-units.md#rust-source-unit
// CODEGEN-BEGIN

// CODEGEN-END
