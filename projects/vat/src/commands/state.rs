// SPEC-MANAGED: projects/vat/tech-design/semantic/vat-commands.md#schema
// CODEGEN-BEGIN
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
// CODEGEN-END
