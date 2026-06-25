//! loom binary — one binary, role-per-subcommand (#164):
//! `controller` (scheduler/state), `worker` (resident harness),
//! `run-task` (in-Job entrypoint), `job-controller` (relay → k8s Job bridge).

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "loom",
    about = "loom — DAG workflow scheduler (control plane over relay + keep)"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Scheduler + sharded, strongly-consistent DAG state; serves the client control API.
    Controller,
    /// Resident pull-loop worker harness (relay lease → keep I/O → ack).
    Worker,
    /// Single-shot in-Job task entrypoint (a k8s Job runs this).
    RunTask,
    /// relay → k8s Job bridge: lease `runner=k8s-job` tasks and create Jobs.
    JobController,
    /// Schema layer: worker-facing bidi edge over the relay work-queue (#432).
    SchemaLayer,
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Controller => loom::controller::run(),
        Command::Worker => loom::worker::run(),
        Command::RunTask => loom::runtask::run(),
        Command::JobController => loom::jobcontroller::run(),
        Command::SchemaLayer => loom::schema_layer::run(),
    }
}
