// <HANDWRITE gap="standardize:claim-code" tracker="projects-lumen-src-bin-lumen-operator-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
//! `lumen-operator` — the lumen K8s Operator entrypoint.
//!
//! - `lumen-operator` (or `run`): start the `Lumen` CRD reconcile loop. Expects
//!   in-cluster or kubeconfig credentials and RBAC to manage the child kinds.
//! - `lumen-operator gen-crd`: print the `Lumen` CustomResourceDefinition as
//!   YAML and exit (for `kubectl apply` / checking the manifest into `k8s/`).

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "lumen-operator",
    about = "lumen K8s Operator — Lumen CRD reconcile loop"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Run the reconcile loop (default).
    Run,
    /// Print the Lumen CustomResourceDefinition as YAML and exit.
    GenCrd,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match Cli::parse().cmd.unwrap_or(Command::Run) {
        Command::GenCrd => {
            print!("{}", lumen::operator::crd_yaml());
            Ok(())
        }
        Command::Run => {
            tracing_subscriber::fmt()
                .with_env_filter(
                    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
                )
                .init();
            lumen::operator::run().await
        }
    }
}

// </HANDWRITE>
