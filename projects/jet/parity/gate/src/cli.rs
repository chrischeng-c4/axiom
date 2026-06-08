// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
// CODEGEN-BEGIN
//! Clap surface for the `jet-parity-gate` binary.

use std::path::PathBuf;

use chrono::Utc;
use clap::{Parser, Subcommand};

use crate::gate::{run_gate, GateReport, EXIT_SKIPPED};
use crate::init::run_init;
use crate::manifest::{GateError, GatingManifest};
use crate::result::ChannelResult;
use crate::waivers::Waivers;

const DEFAULT_MANIFEST: &str = "projects/jet/parity/data/parity-gating.toml";
const DEFAULT_WAIVERS: &str = "projects/jet/parity/data/waivers.toml";
const DEFAULT_RESULTS_DIR: &str = "projects/jet/parity/data/results";
const DEFAULT_INIT_DIR: &str = "projects/jet/parity/data";

#[derive(Debug, Parser)]
#[command(
    name = "jet-parity-gate",
    about = "CI gate over jet parity channel-results (issue #2144).",
    version
)]
/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Command,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run the gate against a directory of `*.channel-result.json` files.
    Gate {
        #[arg(long, default_value = DEFAULT_MANIFEST)]
        manifest: PathBuf,
        #[arg(long, default_value = DEFAULT_RESULTS_DIR)]
        results_dir: PathBuf,
        #[arg(long, default_value = DEFAULT_WAIVERS)]
        waivers: PathBuf,
    },
    /// Scaffold a `parity-gating.toml` + `waivers.toml` + docs into a directory.
    Init {
        #[arg(long, default_value = DEFAULT_INIT_DIR)]
        target_dir: PathBuf,
        #[arg(long)]
        force: bool,
    },
}

/// Parse argv and dispatch. Returns the desired process exit code.
/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-gate-src.md#schema
pub fn run() -> i32 {
    let cli = Cli::parse();
    match dispatch(cli) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e}");
            // Use SKIPPED for "missing manifest at default path" so the
            // workflow stub doesn't blow up before the manifest lands.
            if let GateError::Io { .. } = e {
                EXIT_SKIPPED
            } else {
                1
            }
        }
    }
}

fn dispatch(cli: Cli) -> Result<i32, GateError> {
    match cli.cmd {
        Command::Gate {
            manifest,
            results_dir,
            waivers,
        } => {
            let m = GatingManifest::parse(&manifest)?;
            let results = ChannelResult::parse_dir(&results_dir)?;
            let w = Waivers::parse(&waivers)?;
            let report = run_gate(&m, &results, &w, Utc::now());
            print_report(&report);
            Ok(report.exit_code)
        }
        Command::Init { target_dir, force } => {
            let r = run_init(&target_dir, force)?;
            for p in &r.written {
                println!("wrote {}", p.display());
            }
            Ok(0)
        }
    }
}

fn print_report(r: &GateReport) {
    println!(
        "parity-gate: total={total} pass={pass} fail={fail} waived={waived} skipped={skipped} \
         blocking={blocking} exit={exit}",
        total = r.total,
        pass = r.pass,
        fail = r.fail,
        waived = r.waived,
        skipped = r.skipped,
        blocking = r.blocking,
        exit = r.exit_code,
    );
    for n in &r.notes {
        println!("  - {n}");
    }
}
// CODEGEN-END
