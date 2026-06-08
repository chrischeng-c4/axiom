// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-corpus-src.md#schema
// CODEGEN-BEGIN
//! clap CLI for the jet parity fixture corpus.

use std::io::Write;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::manifest::{parse_manifest, CorpusError};
use crate::verify::verify;

const DEFAULT_MANIFEST: &str = "projects/jet/parity/data/fixtures/mui/fixtures.toml";

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-corpus-src.md#schema
#[derive(Debug, Parser)]
#[command(
    name = "jet-parity-corpus",
    about = "List, inspect, and verify the jet parity fixture corpus."
)]
pub struct FixturesCli {
    /// Path to `fixtures.toml`. Defaults to the in-repo MUI corpus.
    #[arg(long, global = true, default_value = DEFAULT_MANIFEST)]
    pub manifest: PathBuf,

    #[command(subcommand)]
    pub command: FixturesCmd,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-corpus-src.md#schema
#[derive(Debug, Subcommand)]
pub enum FixturesCmd {
    /// Print one fixture per line as `<id>\t<component>\t<jsx_path>`.
    List,
    /// Print the full manifest entry for `<id>` as TOML.
    Show {
        /// Fixture id, e.g. `mui-button-primary-v1`.
        id: String,
    },
    /// Recompute every fixture's hash and diff against the manifest.
    Verify,
}

/// Entry point for the binary. Returns the desired exit code.
///
/// @spec .aw/tech-design/projects/jet/specs/jet-parity-fixture-corpus.md#Logic
pub fn run() -> i32 {
    let cli = FixturesCli::parse();
    match dispatch(&cli, &mut std::io::stdout(), &mut std::io::stderr()) {
        Ok(code) => code,
        Err(err) => {
            let _ = writeln!(std::io::stderr(), "error: {err}");
            1
        }
    }
}

/// Inner dispatch — separated for testability.
/// @spec .aw/tech-design/projects/jet/semantic/jet-parity-corpus-src.md#schema
pub fn dispatch<W: Write, E: Write>(
    cli: &FixturesCli,
    out: &mut W,
    err: &mut E,
) -> Result<i32, CorpusError> {
    match &cli.command {
        FixturesCmd::List => {
            let manifest = parse_manifest(&cli.manifest)?;
            for entry in &manifest.fixtures {
                writeln!(out, "{}\t{}\t{}", entry.id, entry.component, entry.jsx_path)
                    .map_err(io_err(&cli.manifest))?;
            }
            Ok(0)
        }
        FixturesCmd::Show { id } => {
            let manifest = parse_manifest(&cli.manifest)?;
            let entry = manifest
                .fixtures
                .iter()
                .find(|e| &e.id == id)
                .ok_or_else(|| CorpusError::UnknownFixture { id: id.clone() })?;
            let rendered =
                toml::to_string_pretty(entry).map_err(|e| CorpusError::SchemaViolation {
                    field: "<show>".into(),
                    reason: format!("toml serialize: {e}"),
                })?;
            write!(out, "{}", rendered).map_err(io_err(&cli.manifest))?;
            Ok(0)
        }
        FixturesCmd::Verify => {
            let report = verify(&cli.manifest)?;
            if report.clean {
                writeln!(out, "ok ({} fixtures)", report.total).map_err(io_err(&cli.manifest))?;
                Ok(0)
            } else {
                for d in &report.drifted {
                    writeln!(
                        err,
                        "drift\t{}\texpected={}\tactual={}",
                        d.id, d.expected, d.actual
                    )
                    .map_err(io_err(&cli.manifest))?;
                }
                Ok(1)
            }
        }
    }
}

fn io_err(path: &std::path::Path) -> impl Fn(std::io::Error) -> CorpusError + '_ {
    move |source| CorpusError::Io {
        path: path.to_path_buf(),
        source,
    }
}
// CODEGEN-END
