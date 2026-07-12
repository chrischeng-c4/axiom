// SPEC-MANAGED: apps/vat/tech-design/semantic/source/projects-vat-src-commands-mod-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Command implementations, one file per verb.
//!
//! The CLI layer ([`crate::cli`]) parses arguments and dispatches here; each
//! module owns the logic for one verb and returns an [`std::process::ExitCode`]
//! so the binary can propagate a meaningful status (notably: `vat run`
//! forwards the child's exit code).

pub mod build;
pub mod capabilities;
pub mod cluster;
pub mod compose;
pub mod diff;
pub mod doctor;
pub mod emulator;
pub mod gc;
pub mod gpu;
pub mod llm;
pub mod logs;
pub mod ls;
pub mod plan;
pub mod rm;
pub mod run;
pub mod snapshot;
pub mod state;

use anyhow::Result;

/// Print a value as JSON to stdout — pretty by default, single-line when
/// `compact`. Used wherever a verb has a machine-readable mode.
/// @spec apps/vat/tech-design/semantic/source/projects-vat-src-commands-mod-rs.md#source
pub fn print_json<T: serde::Serialize>(value: &T, compact: bool) -> Result<()> {
    let s = if compact {
        serde_json::to_string(value)?
    } else {
        serde_json::to_string_pretty(value)?
    };
    println!("{s}");
    Ok(())
}
// CODEGEN-END
// SPEC-MANAGED: apps/vat/tech-design/logic/vat-microvm-phase-3-vat-compose-limited-compose-subset-up-down-p.md#cli
// CODEGEN-BEGIN
// Module registration and dispatch live in cli.rs
// CODEGEN-END
