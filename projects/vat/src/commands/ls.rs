// SPEC-MANAGED: projects/vat/tech-design/semantic/vat-commands.md#schema
// CODEGEN-BEGIN
//! `vat ls` — list all vats with a one-line status each.
//!
//! Human mode prints a compact table; `--json` emits an array of projected
//! [`VatState`] documents for an agent to consume in one read.

use std::process::ExitCode;

use anyhow::Result;

use crate::state::Status;
use crate::store;

/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-commands-ls-rs.md#source
pub fn exec(json: bool) -> Result<ExitCode> {
    let mut vats = store::list()?;
    // Newest first.
    vats.sort_by(|a, b| b.meta.created_at.cmp(&a.meta.created_at));

    if json {
        let states: Vec<_> = vats
            .iter()
            .map(|v| v.project())
            .collect::<Result<Vec<_>>>()?;
        crate::commands::print_json(&states, false)?;
        return Ok(ExitCode::SUCCESS);
    }

    if vats.is_empty() {
        println!("no vats (try: vat run -- <command>)");
        return Ok(ExitCode::SUCCESS);
    }

    println!(
        "{:<14} {:<12} {:>7} {:<20} NAME",
        "ID", "STATUS", "CHANGES", "CREATED"
    );
    for v in &vats {
        let changes = v
            .changes()
            .map(|c| c.oneline())
            .unwrap_or_else(|_| "?".into());
        let created = v.meta.created_at.format("%Y-%m-%d %H:%M:%S");
        println!(
            "{:<14} {:<12} {:>7} {:<20} {}",
            v.meta.id,
            status_label(&v.meta.status),
            changes,
            created,
            v.meta.name.as_deref().unwrap_or("")
        );
    }
    Ok(ExitCode::SUCCESS)
}

fn status_label(s: &Status) -> String {
    match s {
        Status::Created => "created".into(),
        Status::Running => "running".into(),
        Status::Exited { code } => format!("exited:{code}"),
        Status::Snapshot => "snapshot".into(),
    }
}
// CODEGEN-END
