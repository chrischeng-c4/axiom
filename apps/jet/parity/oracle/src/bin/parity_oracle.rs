// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-oracle-src-bin.md#schema
// CODEGEN-BEGIN

//! `parity-oracle` CLI — `cargo run -p jet-parity-oracle -- run --fixture <name>`.
//!
//! @spec parity-dom-reference-runner.md#Changes (Cargo.toml — binary target)

use clap::{Parser, Subcommand};
use jet_parity_oracle::{run_fixture, BrowserKind, MatrixEntry, RunnerConfig, RunnerError};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "parity-oracle",
    version,
    about = "Headless DOM reference runner (jet #2139)"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run a fixture and emit its 5-channel artifact bundle.
    Run {
        #[arg(long)]
        fixture: PathBuf,
        #[arg(long, default_value = "1.0")]
        dpr: f32,
        #[arg(long, default_value = "artifacts")]
        artifact_root: PathBuf,
        #[arg(long, default_value = "fixtures/__shell__/index.html")]
        shell: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), RunnerError> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Run {
            fixture,
            dpr,
            artifact_root,
            shell,
        } => {
            let config = RunnerConfig {
                artifact_root,
                shell_html: shell,
                ..RunnerConfig::default()
            };
            let matrix = MatrixEntry {
                browser: BrowserKind::Chromium,
                dpr,
            };
            let bundle = run_fixture(&config, &fixture, matrix).await?;
            println!("artifact bundle written to {}", bundle.root_dir.display());
            Ok(())
        }
    }
}
// CODEGEN-END
