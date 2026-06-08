// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/main.md#source
// CODEGEN-BEGIN
//! Agentic Workflow CLI.
//!
//! Standalone binary entry point. Delegates to the `agentic_workflow` library for
//! the `Commands` enum and `run_command` dispatch.

use agentic_workflow::cli::{run_command, Commands};
use anyhow::Context;
use clap::Parser;

#[derive(Parser)]
#[command(
    name = "aw",
    version = env!("AW_BUILD_VERSION"),
    about = "Agentic Workflow — spec-governed workflow engine and CLI"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let rt = tokio::runtime::Runtime::new().context("Failed to create tokio runtime")?;
    rt.block_on(run_command(cli.command))?;
    Ok(())
}

// CODEGEN-END
