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
use crate::config::{ClusterBackend, RetentionPolicy};
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
        /// Run a named production-like integration scenario from vat.toml.
        #[arg(long)]
        scenario: Option<String>,
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
        /// OCI image reference for MicroVm isolation (required when --isolation micro_vm).
        #[arg(long)]
        microvm_image: Option<String>,
        /// Agent runner mode already emits compact JSONL. Direct mode uses this for full VatState JSON.
        #[arg(long)]
        json: bool,
        /// Opaque upstream execution plan to copy into the vat and expose as VAT_PLAN_PATH.
        #[arg(long)]
        plan: Option<PathBuf>,
        /// Override vat.toml [workspace].keep for this configured run.
        #[arg(long, value_enum)]
        keep: Option<RetentionPolicy>,
        /// Direct command mode, e.g. `vat run -- python train.py`.
        #[arg(last = true, allow_hyphen_values = true, value_name = "COMMAND")]
        cmd: Vec<String>,
    },
    /// Print the configured run topology without creating a vat or starting services.
    Plan {
        /// Plan a named production-like integration scenario from vat.toml.
        #[arg(long)]
        scenario: Option<String>,
        /// Named runner(s) from vat.toml; omit to use the default selection rule.
        runners: Vec<String>,
        /// Emit machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Cheap host preflight for the configured run topology.
    Doctor {
        /// Inspect host capabilities without reading vat.toml or selecting a runner.
        #[arg(long)]
        host_only: bool,
        /// Check a named production-like integration scenario from vat.toml.
        #[arg(long)]
        scenario: Option<String>,
        /// Named runner(s) from vat.toml; omit to use the default selection rule.
        runners: Vec<String>,
        /// Emit machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Report this host's effective vat backend capabilities.
    Capabilities {
        /// Emit machine-readable JSON.
        #[arg(long)]
        json: bool,
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
    /// Garbage-collect retained vats. Dry-run by default; use --execute to delete.
    Gc {
        /// Actually delete selected vats. Omit for a dry-run report.
        #[arg(long)]
        execute: bool,
        /// Keep this many newest vats regardless of status.
        #[arg(long, default_value_t = 10)]
        keep_last: usize,
        /// Include failed runs in deletion candidates.
        #[arg(long)]
        include_failed: bool,
        /// Include snapshots in deletion candidates.
        #[arg(long)]
        include_snapshots: bool,
        /// Only select vats last updated at least this many days ago.
        #[arg(long)]
        older_than_days: Option<i64>,
        /// Measure disk size with du. Slower on large stores.
        #[arg(long)]
        measure: bool,
        /// Also compute apparent file size by walking every retained rootfs. Implies --measure.
        #[arg(long)]
        apparent: bool,
        /// Emit machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
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
    /// Build a local OCI image from a Dockerfile using the container CLI.
    Build {
        /// Path to Dockerfile (defaults to Dockerfile in context dir).
        #[arg(long)]
        file: Option<PathBuf>,
        /// Build context directory (defaults to current directory).
        #[arg(long)]
        context: Option<PathBuf>,
        /// OCI image reference to tag (defaults to <context-dir-basename>:latest).
        #[arg(long)]
        tag: Option<String>,
        /// Build argument K=V (repeatable).
        #[arg(long)]
        build_arg: Vec<String>,
        /// Emit structured JSON instead of streaming output.
        #[arg(long)]
        json: bool,
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
    /// Import and run docker-compose.yml services as vat.toml runners.
    Compose {
        #[command(subcommand)]
        cmd: ComposeCmd,
    },
    /// Install or inspect the opt-in headless `docker` command shim.
    Docker {
        #[command(subcommand)]
        cmd: DockerShimCmd,
    },
    /// Run one disposable, Docker-free local Kubernetes session over Apple Container.
    K8s {
        #[command(subcommand)]
        cmd: K8sCmd,
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
    /// Search vat's issues (app:vat); omit the query to list recent.
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
    /// File a structured issue (auto-tagged app:vat).
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

/// `vat compose` subcommands: import/up/down/ps/logs for docker-compose.yml.
#[derive(Subcommand)]
pub enum ComposeCmd {
    /// Import a docker-compose.yml as a vat.toml project.
    Import {
        /// Path to the docker-compose.yml file.
        file: PathBuf,
        /// Project name (defaults to compose file's parent directory basename).
        #[arg(long)]
        project: Option<String>,
        /// Runtime backend (auto, docker, or microvm).
        #[arg(long, value_enum, default_value_t = crate::config::ServiceRuntime::Auto)]
        runtime: crate::config::ServiceRuntime,
    },
    /// Start services from an imported compose project in foreground or detach.
    Up {
        /// Project name.
        #[arg(long)]
        project: Option<String>,
        /// Run in background.
        #[arg(long)]
        detach: bool,
    },
    /// Stop a running compose project.
    Down {
        /// Project name.
        project: String,
    },
    /// List running services in a compose project.
    Ps {
        /// Project name.
        project: String,
    },
    /// Print captured logs from a service in a compose project.
    Logs {
        /// Project name.
        project: String,
        /// Service name.
        service: String,
    },
}

/// `vat docker` manages the opt-in `docker -> vat` multicall shim. The shim
/// translates only a narrow, fail-closed Docker CLI subset to Apple Container;
/// it never creates a Docker Engine socket/API or a GUI/Desktop surface.
#[derive(Subcommand)]
enum DockerShimCmd {
    /// Create a safe `docker -> vat` symlink in an explicit directory.
    InstallShim {
        /// Directory that will contain the `docker` symlink; add it to PATH yourself.
        #[arg(long)]
        dir: PathBuf,
    },
    /// Report whether this explicit directory contains VAT's own Docker shim.
    Status {
        /// Directory that should contain the `docker` symlink.
        #[arg(long)]
        dir: PathBuf,
    },
}

/// Bounded headless Apple-Container local Kubernetes sessions. These sessions
/// deliberately do not extend `vat cluster`: the backing machine is deleted
/// at the end of the foreground command because Apple Container restart
/// semantics are not yet sufficient for durable cluster ownership.
#[derive(Subcommand)]
enum K8sCmd {
    /// Start a single disposable K3s machine for one foreground command.
    Ephemeral {
        #[command(subcommand)]
        cmd: EphemeralK8sCmd,
    },
    /// Keep one explicitly leased Apple K3s session across agent commands.
    Session {
        #[command(subcommand)]
        cmd: K8sSessionCmd,
    },
}

#[derive(Subcommand)]
enum EphemeralK8sCmd {
    /// Build VAT's embedded systemd machine image into Apple Container's image store.
    Image {
        #[command(subcommand)]
        cmd: EphemeralImageCmd,
    },
    /// Inject a private kubeconfig into one host command, then delete the machine.
    Run {
        /// Prebuilt systemd machine image. Build the default explicitly first when absent.
        #[arg(long)]
        image: Option<String>,
        /// Host command to run after the single node is Ready, e.g. `-- kubectl get nodes`.
        #[arg(
            last = true,
            allow_hyphen_values = true,
            value_name = "COMMAND",
            required = true
        )]
        command: Vec<String>,
    },
    /// Reconcile abandoned sessions recorded by an interrupted VAT process.
    Cleanup {
        /// Emit one JSON result object.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum EphemeralImageCmd {
    /// Build the default embedded-image tag required by `ephemeral run`.
    Build,
}

#[derive(Subcommand)]
enum K8sSessionCmd {
    /// Create a one-boot K3s session with private credentials and a bounded lease.
    Create {
        /// Prebuilt systemd machine image. Build the default explicitly first when absent.
        #[arg(long)]
        image: Option<String>,
        /// Lease duration: positive whole seconds, or a value such as 30m / 2h (1m through 4h).
        #[arg(long, default_value = "30m")]
        ttl: String,
    },
    /// Run one host command against a still-valid leased session.
    Exec {
        /// Emit one bounded VAT JSON result instead of replaying child stdout/stderr.
        #[arg(long, value_parser = ["json"])]
        format: Option<String>,
        /// Bound one agent command in seconds; defaults to the remaining lease and cleans its owned process group on timeout or interrupt.
        #[arg(long)]
        timeout: Option<u64>,
        /// Session id emitted by `vat k8s session create`.
        id: String,
        /// Host command to run with the private kubeconfig, e.g. `-- kubectl get nodes`.
        #[arg(
            last = true,
            allow_hyphen_values = true,
            value_name = "COMMAND",
            required = true
        )]
        command: Vec<String>,
    },
    /// Forward one active lease Service only to loopback for one host command.
    PortForward {
        #[command(subcommand)]
        cmd: K8sSessionPortForwardCmd,
    },
    /// Import a locally verified Apple Container image into this active K3s lease.
    Image {
        #[command(subcommand)]
        cmd: K8sSessionImageCmd,
    },
    /// Show a session's lease and exact Apple-machine presence without exposing credentials.
    Status {
        /// Verify the active session's owned API with its private kubeconfig.
        #[arg(long)]
        verify_api: bool,
        /// Session id emitted by `vat k8s session create`.
        id: String,
    },
    /// Delete one exact owned session and its private credentials.
    Delete {
        /// Session id emitted by `vat k8s session create`.
        id: String,
    },
    /// Reclaim expired leases and abandoned session creations; active sessions are retained.
    Cleanup {
        /// Emit one JSON result object.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum K8sSessionImageCmd {
    /// Save one local ARM64 Linux image privately, import it into K3s, then remove both archives.
    Load {
        /// Session id emitted by `vat k8s session create`.
        id: String,
        /// Locally present Apple Container image reference; arbitrary tar files are not accepted.
        image: String,
        /// Guest image platform. The current Apple K3s path is deliberately linux/arm64 only.
        #[arg(long, default_value = "linux/arm64")]
        platform: String,
    },
}

#[derive(Subcommand)]
enum K8sSessionPortForwardCmd {
    /// Forward one Service port to 127.0.0.1 for one foreground host command.
    Run {
        /// Emit one bounded VAT JSON result after confirmed tunnel cleanup.
        #[arg(long, value_parser = ["json"])]
        format: Option<String>,
        /// Session id emitted by vat k8s session create.
        id: String,
        /// Service selector, exactly service/<name>.
        resource: String,
        /// Numeric Service port to forward.
        remote_port: u16,
        /// Kubernetes namespace containing the Service.
        #[arg(long, default_value = "default")]
        namespace: String,
        /// Loopback local port. Use zero to let kubectl choose one.
        #[arg(long, default_value_t = 0)]
        local_port: u16,
        /// Host test or assertion command after --. It receives only tunnel metadata.
        #[arg(
            last = true,
            allow_hyphen_values = true,
            value_name = "COMMAND",
            required = true
        )]
        command: Vec<String>,
    },
}

/// Parse argv and dispatch. Returns the process exit code (notably, `run`
/// forwards the child command's code).
pub fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Run {
            scenario,
            runners,
            base,
            from,
            name,
            isolation,
            gpu,
            microvm_image,
            json,
            plan,
            keep,
            mut cmd,
        } => {
            if let Some(scenario_id) = scenario {
                if !cmd.is_empty() {
                    anyhow::bail!("vat run --scenario cannot be combined with direct command mode");
                }
                if !runners.is_empty() {
                    anyhow::bail!("vat run --scenario cannot be combined with runner ids");
                }
                let target = commands::run::Target::Scenario { scenario_id };
                return commands::run::exec(commands::run::Args {
                    target,
                    base,
                    from,
                    name,
                    isolation,
                    gpu,
                    microvm_image,
                    json,
                    plan,
                    keep,
                    compose_handoff: None,
                });
            }
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
                microvm_image,
                json,
                plan,
                keep,
                compose_handoff: None,
            })
        }
        Cmd::Plan {
            scenario,
            runners,
            json,
        } => commands::plan::exec(configured_target(scenario, runners)?, json),
        Cmd::Doctor {
            host_only,
            scenario,
            runners,
            json,
        } => {
            if host_only {
                if scenario.is_some() || !runners.is_empty() {
                    anyhow::bail!("`vat doctor --host-only` cannot select a scenario or runner");
                }
                commands::doctor::host_only_exec(json)
            } else {
                commands::doctor::exec(configured_target(scenario, runners)?, json)
            }
        }
        Cmd::Capabilities { json } => commands::capabilities::exec(json),
        Cmd::Ls { json } => commands::ls::exec(json),
        Cmd::State { id, compact } => commands::state::exec(id, compact),
        Cmd::Diff { id, json } => commands::diff::exec(id, json),
        Cmd::Fork { id, name } => commands::snapshot::fork(id, name),
        Cmd::Snapshot { id, name } => commands::snapshot::snapshot(id, name),
        Cmd::Rm { id } => commands::rm::exec(id),
        Cmd::Gc {
            execute,
            keep_last,
            include_failed,
            include_snapshots,
            older_than_days,
            measure,
            apparent,
            json,
        } => commands::gc::exec(commands::gc::Args {
            execute,
            keep_last,
            include_failed,
            include_snapshots,
            older_than_days,
            measure,
            apparent,
            json,
        }),
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
        Cmd::Build {
            file,
            context,
            tag,
            build_arg,
            json,
        } => commands::build::exec(commands::build::Args {
            file,
            context,
            tag,
            build_arg,
            json,
        }),
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
        Cmd::Compose { cmd } => commands::compose::exec(cmd),
        Cmd::Docker { cmd } => match cmd {
            DockerShimCmd::InstallShim { dir } => commands::docker_shim::install_shim(dir),
            DockerShimCmd::Status { dir } => commands::docker_shim::shim_status(dir),
        },
        Cmd::K8s { cmd } => match cmd {
            K8sCmd::Ephemeral { cmd } => match cmd {
                EphemeralK8sCmd::Image { cmd } => match cmd {
                    EphemeralImageCmd::Build => commands::k8s::build_default_image(),
                },
                EphemeralK8sCmd::Run { image, command } => {
                    commands::k8s::ephemeral_run(commands::k8s::EphemeralRunArgs { image, command })
                }
                EphemeralK8sCmd::Cleanup { json } => commands::k8s::cleanup_abandoned(json),
            },
            K8sCmd::Session { cmd } => match cmd {
                K8sSessionCmd::Create { image, ttl } => {
                    commands::k8s::session_create(commands::k8s::ActiveSessionCreateArgs {
                        image,
                        ttl,
                    })
                }
                K8sSessionCmd::Exec {
                    format,
                    timeout,
                    id,
                    command,
                } => commands::k8s::session_exec(id, command, format.is_some(), timeout),
                K8sSessionCmd::PortForward { cmd } => match cmd {
                    K8sSessionPortForwardCmd::Run {
                        format,
                        id,
                        resource,
                        remote_port,
                        namespace,
                        local_port,
                        command,
                    } => commands::k8s::session_port_forward(
                        commands::k8s::ActiveSessionPortForwardArgs {
                            json: format.is_some(),
                            id,
                            resource,
                            remote_port,
                            namespace,
                            local_port,
                            command,
                        },
                    ),
                },
                K8sSessionCmd::Image { cmd } => match cmd {
                    K8sSessionImageCmd::Load {
                        id,
                        image,
                        platform,
                    } => commands::k8s::session_image_load(
                        commands::k8s::ActiveSessionImageLoadArgs {
                            id,
                            image,
                            platform,
                        },
                    ),
                },
                K8sSessionCmd::Status { id, verify_api } => {
                    if verify_api {
                        commands::k8s::session_status_verify_api(id)
                    } else {
                        commands::k8s::session_status(id)
                    }
                }
                K8sSessionCmd::Delete { id } => commands::k8s::session_delete(id),
                K8sSessionCmd::Cleanup { json } => commands::k8s::session_cleanup(json),
            },
        },
    }
}

fn configured_target(
    scenario: Option<String>,
    runners: Vec<String>,
) -> Result<commands::plan::PlanTarget> {
    if let Some(scenario_id) = scenario {
        if !runners.is_empty() {
            anyhow::bail!("--scenario cannot be combined with runner ids");
        }
        Ok(commands::plan::PlanTarget::Scenario { scenario_id })
    } else {
        Ok(commands::plan::PlanTarget::Runner {
            runner_ids: runners,
        })
    }
}

/// vat's identity + build provenance for the shared CLI-convention verbs
/// (`llm` / `upgrade` / `issue`), per CONTRIBUTING.md. Stamps come from `build.rs`.
// Used by the feature-gated upgrade/issue dispatch; unused in a lean build.
#[cfg_attr(not(any(feature = "self-update", feature = "issue")), allow(dead_code))]
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
/// always scoped to the `app:vat` tracker label.
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
                        let head: String = message
                            .lines()
                            .next()
                            .unwrap_or("")
                            .chars()
                            .take(72)
                            .collect();
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
                        label: vec!["app:vat".to_string()],
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
// CODEGEN-END
// CODEGEN-BEGIN
// ComposeCmd enum defined above; dispatch in run() match statement
// CODEGEN-END
