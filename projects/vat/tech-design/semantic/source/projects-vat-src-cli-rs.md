---
id: vat-source-projects-vat-src-cli-rs
summary: >
  rust-source-unit TD AST payload for projects/vat/src/cli.rs.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This rust-source-unit TD preserves vat source ownership while migrating #39 off group-level source replay."
---

# Standardized projects/vat/src/cli.rs

## Overview
<!-- type: overview lang: markdown -->

Rust source-unit TD for `projects/vat/src/cli.rs`, captured during #39 vat migration onto td_ast lossless source generation.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! CLI surface.
//!
//! Verbs are deliberately few and composable, because the operator is an
//! agent, not a human juggling a dashboard. The defaults that matter for an
//! agent — JSON state, forwarded exit codes, copy-on-write disposability — are
//! the *unflagged* path. The README carries the tradeoff rationale for where
//! vat departs from Docker's human-dev ergonomics.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands;
use crate::config::ClusterBackend;
use crate::spec::{GpuRequest, Isolation};

#[derive(Parser)]
#[command(
    name = "vat",
    version = crate::VERSION,
    about = "agent-native, GPU-native dev containers (no VM: the Apple GPU just works)",
    long_about = "agent-native, GPU-native dev containers (no VM: the Apple GPU just works)\n\nRun `vat llm` for the compact agent-facing usage contract, including when to use vat.toml, how to inspect evidence, and what Docker-like assumptions do not apply."
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Create a fresh vat and run a command inside it.
    Run {
        /// Named runner(s) from vat.toml. Omit to use default_runner or the
        /// only runner; pass several to run them CONCURRENTLY against one
        /// shared workspace + service set (worst exit code wins).
        runners: Vec<String>,
        /// Clone from this host directory (default: current directory).
        #[arg(long)]
        base: Option<PathBuf>,
        /// Fork from an existing vat instead of a host directory.
        #[arg(long)]
        from: Option<String>,
        /// Optional human label for the vat.
        #[arg(long)]
        name: Option<String>,
        /// Isolation backend.
        #[arg(long, value_enum, default_value = "none")]
        isolation: Isolation,
        /// GPU expectation.
        #[arg(long, value_enum, default_value = "auto")]
        gpu: GpuRequest,
        /// Agent runner mode already emits compact JSONL. Direct mode uses this for full VatState JSON.
        #[arg(long)]
        json: bool,
        /// Direct command mode, e.g. `vat run -- python train.py`.
        #[arg(last = true, allow_hyphen_values = true, value_name = "COMMAND")]
        cmd: Vec<String>,
    },
    /// List all vats.
    Ls {
        #[arg(long)]
        json: bool,
    },
    /// Print the full agent-legible state of a vat as JSON.
    State {
        id: String,
        /// Single-line JSON instead of pretty.
        #[arg(long)]
        compact: bool,
    },
    /// Show every filesystem change vs. the vat's base.
    Diff {
        id: String,
        #[arg(long)]
        json: bool,
    },
    /// Fork a vat into a new runnable working copy.
    Fork {
        id: String,
        #[arg(long)]
        name: Option<String>,
    },
    /// Freeze a vat into an immutable snapshot.
    Snapshot {
        id: String,
        #[arg(long)]
        name: Option<String>,
    },
    /// Delete a vat and its workspace.
    Rm { id: String },
    /// Print captured logs from a vat.toml runner invocation.
    Logs { id: String, source: Option<String> },
    /// Print agent-facing docs for driving vat — offline, no network.
    Llm {
        /// Topic to print: outline (default) or guide.
        #[arg(long, default_value = "outline")]
        topic: String,
        /// Output format.
        #[arg(long, value_enum, default_value_t = LlmFormat::Md)]
        format: LlmFormat,
    },
    /// Self-update vat to the latest `vat@*` GitHub release.
    Upgrade {
        /// Report the current and latest version without changing the binary.
        #[arg(long)]
        check: bool,
        /// Install this exact version (`0.3.62` or `vat@0.3.62`) instead of the latest.
        #[arg(long)]
        version: Option<String>,
        /// Reinstall even when already on the selected version.
        #[arg(long)]
        force: bool,
        /// Skip the confirmation prompt.
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Search, view, and file vat issues on the axiom tracker.
    Issue {
        #[command(subcommand)]
        cmd: IssueCmd,
    },
    /// Report the GPU every vat on this host can reach.
    Gpu {
        #[arg(long)]
        json: bool,
    },
    /// Manage standalone local Kubernetes clusters (independent of runs).
    Cluster {
        #[command(subcommand)]
        cmd: ClusterCmd,
    },
    /// Internal: run a built-in emulator. vat spawns itself for an emulator
    /// preset service; not intended for direct human use.
    #[command(hide = true)]
    Emulator {
        #[arg(value_enum)]
        kind: EmulatorKind,
        /// host:port to bind, e.g. 127.0.0.1:8085.
        #[arg(long)]
        host_port: String,
        /// CA pem path (http-mock only).
        #[arg(long)]
        ca_path: Option<String>,
        /// Cassette dir (http-mock only).
        #[arg(long)]
        cassette_dir: Option<String>,
        /// OpenAPI spec path (openapi only).
        #[arg(long)]
        spec: Option<String>,
        /// Seed a host route (http-mock only), repeatable: `--route host=base`.
        #[arg(long)]
        route: Vec<String>,
    },
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum LlmFormat {
    Md,
    Json,
}

impl From<LlmFormat> for cli_std::llm::Format {
    fn from(format: LlmFormat) -> Self {
        match format {
            LlmFormat::Md => cli_std::llm::Format::Md,
            LlmFormat::Json => cli_std::llm::Format::Json,
        }
    }
}

#[derive(Subcommand)]
enum IssueCmd {
    /// Search vat's issues (project:vat); omit the query to list recent.
    Search {
        /// Search text (omit to list recent issues).
        #[arg(num_args = 0..)]
        query: Vec<String>,
        /// Issue state filter.
        #[arg(long, value_parser = ["open", "closed", "all"], default_value = "open")]
        state: String,
        /// Max results.
        #[arg(long, default_value_t = 20)]
        limit: u32,
    },
    /// Print a single issue by number.
    View {
        /// Issue number.
        number: u64,
    },
    /// File a structured issue (auto-tagged project:vat).
    Create {
        /// Issue title (default: derived from the message).
        #[arg(long)]
        title: Option<String>,
        /// Print the issue that would be filed without creating it.
        #[arg(long)]
        dry_run: bool,
        /// Free-text description of the problem.
        #[arg(num_args = 0..)]
        message: Vec<String>,
    },
}

/// Which built-in emulator to run.
/// @spec projects/vat/tech-design/logic/built-in-rust-emulators-pub-sub-firebase-auth.md#cli
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum EmulatorKind {
    Pubsub,
    FirebaseAuth,
    CloudTasks,
    CloudScheduler,
    CloudWorkflows,
    CloudStorage,
    HttpMock,
    Openapi,
}

/// Standalone `vat cluster` verbs. Clusters created here outlive a single run;
/// vat creates/lists/deletes them on explicit command but does not supervise
/// them.
/// @spec projects/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#cli
#[derive(Subcommand)]
enum ClusterCmd {
    /// Create a local Kubernetes cluster.
    Create {
        /// Cluster name (auto-generated when omitted).
        #[arg(long)]
        name: Option<String>,
        /// Backend to use; `auto` prefers kind → k3d → minikube.
        #[arg(long, value_enum, default_value = "auto")]
        backend: ClusterBackend,
        /// Kubernetes version for the node image (e.g. 1.30).
        #[arg(long)]
        k8s_version: Option<String>,
        /// Node count.
        #[arg(long, default_value_t = 1)]
        nodes: u32,
        #[arg(long)]
        json: bool,
    },
    /// List vat-managed clusters.
    Ls {
        #[arg(long)]
        json: bool,
    },
    /// Print the kubeconfig path (or record) for a cluster.
    Kubeconfig {
        name: String,
        #[arg(long)]
        json: bool,
    },
    /// Delete a cluster by name.
    Delete {
        name: String,
        #[arg(long)]
        json: bool,
    },
}

/// Parse argv and dispatch. Returns the process exit code (notably, `run`
/// forwards the child command's code).
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-cli-rs.md#source
pub fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Run {
            runners,
            base,
            from,
            name,
            isolation,
            gpu,
            json,
            mut cmd,
        } => {
            let target = if !cmd.is_empty() {
                let program = cmd.remove(0);
                commands::run::Target::Direct {
                    program,
                    program_args: cmd,
                }
            } else {
                commands::run::Target::Runner {
                    runner_ids: runners,
                }
            };
            commands::run::exec(commands::run::Args {
                target,
                base,
                from,
                name,
                isolation,
                gpu,
                json,
            })
        }
        Cmd::Ls { json } => commands::ls::exec(json),
        Cmd::State { id, compact } => commands::state::exec(id, compact),
        Cmd::Diff { id, json } => commands::diff::exec(id, json),
        Cmd::Fork { id, name } => commands::snapshot::fork(id, name),
        Cmd::Snapshot { id, name } => commands::snapshot::snapshot(id, name),
        Cmd::Rm { id } => commands::rm::exec(id),
        Cmd::Logs { id, source } => commands::logs::exec(id, source),
        Cmd::Llm { topic, format } => commands::llm::exec(&topic, format.into()),
        Cmd::Upgrade {
            check,
            version,
            force,
            yes,
        } => upgrade_cmd(check, version, force, yes),
        Cmd::Issue { cmd } => issue_cmd(cmd),
        Cmd::Gpu { json } => commands::gpu::exec(json),
        Cmd::Cluster { cmd } => match cmd {
            ClusterCmd::Create {
                name,
                backend,
                k8s_version,
                nodes,
                json,
            } => commands::cluster::create(name, backend, k8s_version, nodes, json),
            ClusterCmd::Ls { json } => commands::cluster::ls(json),
            ClusterCmd::Kubeconfig { name, json } => commands::cluster::kubeconfig(name, json),
            ClusterCmd::Delete { name, json } => commands::cluster::delete(name, json),
        },
        Cmd::Emulator {
            kind,
            host_port,
            ca_path,
            cassette_dir,
            spec,
            route,
        } => commands::emulator::exec(kind, host_port, ca_path, cassette_dir, spec, route),
    }
}

/// vat's identity + build provenance for the shared CLI-convention verbs
/// (`llm` / `upgrade` / `issue`), per CONTRIBUTING.md. Stamps come from `build.rs`.
/// @spec projects/vat/tech-design/interfaces/cli/migrate-upgrade-and-report-issue-to-the-shared-cli-std-crate.md#cli
// Used by the feature-gated upgrade/issue dispatch; unused in a lean build.
#[cfg_attr(
    not(any(feature = "self-update", feature = "issue")),
    allow(dead_code)
)]
const TOOL: cli_std::ToolInfo = cli_std::ToolInfo {
    project: "vat",
    repo: "chrischeng-c4/axiom",
    target: env!("VAT_TARGET"),
    version: env!("CARGO_PKG_VERSION"),
    git_sha: env!("VAT_GIT_SHA"),
    built_at: env!("VAT_BUILT_AT"),
};

/// `vat upgrade` → `cli_std::upgrade::run` on a tokio runtime. Without the
/// `self-update` feature the HTTP client + runtime are absent, so it bails
/// cleanly (the shipped binary includes the feature).
#[cfg(feature = "self-update")]
fn upgrade_cmd(check: bool, version: Option<String>, force: bool, yes: bool) -> Result<ExitCode> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(cli_std::upgrade::run(
        &TOOL,
        cli_std::upgrade::Options {
            check,
            tag: version,
            force,
            yes,
        },
    ))?;
    Ok(ExitCode::SUCCESS)
}

#[cfg(not(feature = "self-update"))]
fn upgrade_cmd(
    _check: bool,
    _version: Option<String>,
    _force: bool,
    _yes: bool,
) -> Result<ExitCode> {
    anyhow::bail!(
        "this vat build was compiled without self-update support; rebuild with \
         default features (the published binary includes it)"
    )
}

/// `vat issue <search|view|create>` → `cli_std::issue` on a tokio runtime,
/// always scoped to the `project:vat` tracker label.
#[cfg(feature = "issue")]
fn issue_cmd(cmd: IssueCmd) -> Result<ExitCode> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(async {
        match cmd {
            IssueCmd::Search {
                query,
                state,
                limit,
            } => {
                let query = (!query.is_empty()).then(|| query.join(" "));
                cli_std::issue::search(
                    &TOOL,
                    cli_std::issue::SearchOptions {
                        query,
                        state,
                        limit,
                    },
                )
                .await
            }
            IssueCmd::View { number } => cli_std::issue::view(&TOOL, number).await,
            IssueCmd::Create {
                title,
                dry_run,
                message,
            } => {
                let message = (!message.is_empty()).then(|| message.join(" "));
                let title = title.unwrap_or_else(|| {
                    if let Some(message) = message.as_deref().filter(|m| !m.trim().is_empty()) {
                        let head: String =
                            message.lines().next().unwrap_or("").chars().take(72).collect();
                        format!("vat: {head}")
                    } else {
                        "vat: issue report".to_string()
                    }
                });
                cli_std::issue::create(
                    &TOOL,
                    cli_std::issue::CreateOptions {
                        title,
                        message,
                        url: None,
                        repo: None,
                        label: vec!["project:vat".to_string()],
                        dry_run,
                        yes: true,
                    },
                )
                .await
            }
        }
    })?;
    Ok(ExitCode::SUCCESS)
}

#[cfg(not(feature = "issue"))]
fn issue_cmd(_cmd: IssueCmd) -> Result<ExitCode> {
    anyhow::bail!(
        "this vat build was compiled without issue support; rebuild with \
         default features (the published binary includes it)"
    )
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/cli.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/vat/src/cli.rs` captured during #39 vat standardization.
```
