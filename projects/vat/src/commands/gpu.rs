// SPEC-MANAGED: projects/vat/tech-design/semantic/vat-commands.md#schema
// CODEGEN-BEGIN
//! `vat gpu` — report the GPU every vat on this host can reach.
//!
//! The fastest way for an agent (or a curious human) to confirm the headline
//! claim: on Apple Silicon this prints an accessible Metal device, where the
//! same probe inside a Docker container reports nothing.

use std::process::ExitCode;

use anyhow::Result;

use crate::gpu;

/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-commands-gpu-rs.md#source
pub fn exec(json: bool) -> Result<ExitCode> {
    let info = gpu::detect();
    if json {
        crate::commands::print_json(&info, false)?;
        return Ok(ExitCode::SUCCESS);
    }
    let chip = info.chip.as_deref().unwrap_or("unknown");
    let mark = if info.accessible {
        "✓ accessible"
    } else {
        "✗ not accessible"
    };
    println!("vendor   {}", info.vendor);
    println!("chip     {chip}");
    println!("backends {}", info.backends.join(", "));
    println!("status   {mark}");
    println!("note     {}", info.note);
    Ok(ExitCode::SUCCESS)
}
// CODEGEN-END
