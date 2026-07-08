// SPEC-MANAGED: projects/tape/tech-design/semantic/source/projects-tape-src-bin-tape-bench-rs.md#logic
// <HANDWRITE gap="missing-generator:logic:tape-competitor-performance" tracker="#768" reason="Initial benchmark CLI before generated efficiency runner primitives exist.">
use anyhow::{bail, Result};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "tape-bench", version, about = "Tape local benchmark runner")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the local Tape replay benchmark and report win/loss calibration status.
    Run(RunArgs),
}

#[derive(clap::Args)]
struct RunArgs {
    /// Number of local events to append and replay.
    #[arg(long, default_value_t = 1_000)]
    events: usize,
    /// Payload body size in bytes.
    #[arg(long, default_value_t = 128)]
    payload_bytes: usize,
    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Run(args) => run(args),
    }
}

fn run(args: RunArgs) -> Result<()> {
    let report = tape::bench::run_benchmark(args.events, args.payload_bytes);
    match args.format {
        OutputFormat::Text => {
            println!(
                "events={} payload_bytes={} append_p50_us={} append_p95_us={} replay_full_us={} checkpoint_p50_us={} checkpoint_p95_us={} verdict={}",
                report.events,
                report.payload_bytes,
                report.append_p50_us,
                report.append_p95_us,
                report.replay_full_us,
                report.checkpoint_p50_us,
                report.checkpoint_p95_us,
                report.verdict
            );
            for peer in &report.peers {
                println!(
                    "peer={} replay_baseline={} status={} win_claim={} reason={}",
                    peer.peer, peer.replay_baseline, peer.status, peer.win_claim, peer.reason
                );
            }
        }
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
    }
    if let Err(error) = tape::bench::verify_report(&report) {
        bail!("{error}");
    }
    Ok(())
}
// </HANDWRITE>
