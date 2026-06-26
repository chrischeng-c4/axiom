// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-commands-diff-rs.md#rust-source-unit
// CODEGEN-BEGIN
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
// CODEGEN-END
// SPEC-MANAGED: projects/vat/tech-design/logic/vat-td-ast-promote-remaining-grouped-source-units.md#rust-source-unit
// CODEGEN-BEGIN

// CODEGEN-END
