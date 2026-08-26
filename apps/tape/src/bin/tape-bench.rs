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
    /// WI #3052 AC1: drive the real WAL commit coordinator over real HTTP at
    /// varying connection counts and report the durable throughput scaling
    /// ratio (highest sampled connection count vs. the lowest).
    Durable(DurableArgs),
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

#[derive(clap::Args)]
struct DurableArgs {
    /// Number of sequential append requests each connection issues.
    #[arg(long, default_value_t = 200)]
    events_per_connection: usize,
    /// Payload body size in bytes.
    #[arg(long, default_value_t = 128)]
    payload_bytes: usize,
    /// Connection counts to sample, e.g. `--connections 1,4,16`.
    #[arg(long, value_delimiter = ',', default_value = "1,4,16")]
    connections: Vec<usize>,
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
        Command::Durable(args) => durable(args),
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

fn durable(args: DurableArgs) -> Result<()> {
    if args.connections.is_empty() {
        bail!("--connections requires at least one connection count");
    }
    let report = tape::bench::run_durable_benchmark(
        args.events_per_connection,
        args.payload_bytes,
        &args.connections,
    );
    match args.format {
        OutputFormat::Text => {
            println!(
                "payload_bytes={} scaling_ratio={:.2}x",
                report.payload_bytes, report.scaling_ratio
            );
            for sample in &report.samples {
                println!(
                    "connections={} events={} elapsed_us={} ops_per_sec={:.2}",
                    sample.connections, sample.events, sample.elapsed_us, sample.ops_per_sec
                );
            }
        }
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
    }
    Ok(())
}
// </HANDWRITE>
