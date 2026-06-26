// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-cli-rs.md#rust-source-unit
// CODEGEN-BEGIN
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
    /// Print the compact LLM/agent usage guide.
    Llm,
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
    /// File a diagnostics-rich GitHub issue against the axiom repo.
    #[command(name = "report-issue")]
    ReportIssue {
        /// Issue title.
        #[arg(short = 't', long)]
        title: String,
        /// Description placed above the auto-attached diagnostics block.
        #[arg(short = 'm', long)]
        message: Option<String>,
        /// Target repository (`owner/name`); defaults to vat's release repo.
        #[arg(long)]
        repo: Option<String>,
        /// Add a label (repeatable).
        #[arg(long)]
        label: Vec<String>,
        /// Assemble and print the report without submitting anything.
        #[arg(long)]
        dry_run: bool,
        /// Skip the confirmation prompt.
        #[arg(short = 'y', long)]
        yes: bool,
        /// Free-text message (used as the description when `--message` is absent).
        #[arg(trailing_var_arg = true)]
        rest: Vec<String>,
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
        /// Hermetic mode (http-mock only): block unmatched requests instead of
        /// forwarding them to the real upstream.
        #[arg(long)]
        no_forward: bool,
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
        Cmd::Llm => commands::llm::exec(),
        Cmd::Upgrade {
            check,
            version,
            force,
            yes,
        } => upgrade_cmd(check, version, force, yes),
        Cmd::ReportIssue {
            title,
            message,
            repo,
            label,
            dry_run,
            yes,
            rest,
        } => report_issue_cmd(title, message, repo, label, dry_run, yes, rest),
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
            no_forward,
        } => commands::emulator::exec(
            kind,
            host_port,
            ca_path,
            cassette_dir,
            spec,
            route,
            no_forward,
        ),
    }
}

/// vat's identity + build provenance for the shared CLI-convention verbs
/// (`upgrade` / `report-issue`), per CONTRIBUTING.md. Stamps come from `build.rs`.
/// @spec projects/vat/tech-design/interfaces/cli/migrate-upgrade-and-report-issue-to-the-shared-cli-std-crate.md#cli
// Used by the feature-gated upgrade/report-issue dispatch; unused in a lean build.
#[cfg_attr(
    not(any(feature = "self-update", feature = "report-issue")),
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

/// `vat report-issue` → `cli_std::report_issue::run` on a tokio runtime, tagging
/// the issue with `project:vat`. Without the `report-issue` feature it bails.
#[cfg(feature = "report-issue")]
#[allow(clippy::too_many_arguments)]
fn report_issue_cmd(
    title: String,
    message: Option<String>,
    repo: Option<String>,
    label: Vec<String>,
    dry_run: bool,
    yes: bool,
    rest: Vec<String>,
) -> Result<ExitCode> {
    let message = message.or_else(|| (!rest.is_empty()).then(|| rest.join(" ")));
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(cli_std::report_issue::run(
        &TOOL,
        cli_std::report_issue::Options {
            title,
            message,
            url: None,
            repo,
            label: std::iter::once("project:vat".to_string())
                .chain(label)
                .collect(),
            dry_run,
            yes,
        },
    ))?;
    Ok(ExitCode::SUCCESS)
}

#[cfg(not(feature = "report-issue"))]
#[allow(clippy::too_many_arguments)]
fn report_issue_cmd(
    _title: String,
    _message: Option<String>,
    _repo: Option<String>,
    _label: Vec<String>,
    _dry_run: bool,
    _yes: bool,
    _rest: Vec<String>,
) -> Result<ExitCode> {
    anyhow::bail!(
        "this vat build was compiled without report-issue support; rebuild with \
         default features (the published binary includes it)"
    )
}
// CODEGEN-END
