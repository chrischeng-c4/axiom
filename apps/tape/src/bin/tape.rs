// SPEC-MANAGED: apps/tape/tech-design/semantic/source/apps-tape-src-bin-tape-rs.md#logic
// <HANDWRITE gap="missing-generator:logic:tape-bootstrap" tracker="#768" reason="Initial Tape CLI surface before generated command wiring exists.">
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use serde_json::{json, Value};
use tape::{spec, TapeJournal};

#[derive(Parser)]
#[command(name = "tape", version, about = "tape - topic replay journal service")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Append one event envelope to a topic journal.
    Append(AppendArgs),
    /// Replay topic history by offset or timestamp.
    Replay(ReplayArgs),
    /// Manage durable consumer replay checkpoints.
    Checkpoint(CheckpointArgs),
    /// Manage named caller-driven topic pull cursors.
    Subscription(SubscriptionArgs),
    /// Serve the topic journal over HTTP (h2c + HTTP/1.1 on one port).
    Serve(ServeArgs),
    /// Print Tape's machine-readable API contract, offline.
    Spec(SpecArgs),
    /// Print agent-facing LLM topics, offline.
    Llm(LlmArgs),
    /// Self-update this binary from a published GitHub release.
    Upgrade(UpgradeArgs),
    /// Search, view, file, and comment on Tape issues.
    Issue(IssueArgs),
    /// Kubernetes artifacts split by layer: the cluster-scoped CRD, the
    /// operator control plane, and app-namespace Tape instances. Render paths
    /// are offline (they work from the binary); only `operator run` needs the
    /// `operator` build feature (#1328).
    K8s(K8sArgs),
    /// Render tape's runtime image Dockerfiles — offline, no server. Image
    /// construction is owned here (not by `k8s`) because the same artifact
    /// feeds compose, kind, and real registries (#1328).
    Dockerfile(DockerfileArgs),
    /// Write a consistent snapshot of a RUNNING node's journal to a backup
    /// destination through the shared libs/service-backup runner (#1329):
    /// fetches `GET /admin/backup` and ships the bytes to `--dest` (`file://`
    /// always; `s3://` via the lib). Needs a build with `--features backup`.
    Backup(BackupArgs),
}

#[derive(clap::Args)]
struct AppendArgs {
    /// Topic name.
    topic: String,
    /// Optional partitioning/idempotency key carried in the event envelope.
    #[arg(long)]
    key: Option<String>,
    /// JSON payload or a string payload when the value is not valid JSON.
    #[arg(long)]
    payload: String,
    /// Override event timestamp for deterministic tests/backfill.
    #[arg(long)]
    timestamp_ms: Option<u64>,
    /// Journal file. Defaults to `.tape/journal.json`.
    #[arg(long, default_value = ".tape/journal.json")]
    store: PathBuf,
}

#[derive(clap::Args)]
struct ReplayArgs {
    /// Topic name.
    topic: String,
    /// First offset to include.
    #[arg(long)]
    from_offset: Option<u64>,
    /// First event timestamp to include.
    #[arg(long)]
    from_timestamp_ms: Option<u64>,
    /// Maximum number of events to return.
    #[arg(long)]
    limit: Option<usize>,
    /// Journal file. Defaults to `.tape/journal.json`.
    #[arg(long, default_value = ".tape/journal.json")]
    store: PathBuf,
}

#[derive(clap::Args)]
struct CheckpointArgs {
    #[command(subcommand)]
    command: CheckpointCommand,
}

#[derive(Subcommand)]
enum CheckpointCommand {
    /// Read a consumer checkpoint.
    Get(CheckpointGetArgs),
    /// Advance a consumer checkpoint.
    Put(CheckpointPutArgs),
}

#[derive(clap::Args)]
struct CheckpointGetArgs {
    topic: String,
    consumer: String,
    #[arg(long, default_value = ".tape/journal.json")]
    store: PathBuf,
}

#[derive(clap::Args)]
struct CheckpointPutArgs {
    topic: String,
    consumer: String,
    #[arg(long)]
    offset: u64,
    #[arg(long, default_value = ".tape/journal.json")]
    store: PathBuf,
}

#[derive(clap::Args)]
struct SubscriptionArgs {
    #[command(subcommand)]
    command: SubscriptionCommand,
}

#[derive(Subcommand)]
enum SubscriptionCommand {
    /// Create a caller-driven pull subscription.
    Create(SubscriptionCreateArgs),
    /// List the delivery resources for one topic.
    List(SubscriptionListArgs),
    /// Show one topic delivery resource and its pull checkpoint, if any.
    Show(SubscriptionShowArgs),
    /// Read one bounded event window from a pull subscription cursor.
    Pull(SubscriptionPullArgs),
    /// Advance a pull subscription cursor after the caller processes a window.
    Ack(SubscriptionAckArgs),
    /// Delete resource metadata while preserving a matching pull checkpoint.
    Delete(SubscriptionDeleteArgs),
}

#[derive(clap::Args)]
struct SubscriptionCreateArgs {
    /// Topic that owns the subscription.
    topic: String,
    /// Subscription name, also used as the durable checkpoint consumer name.
    name: String,
    /// Journal file. Defaults to `.tape/journal.json`.
    #[arg(long, default_value = ".tape/journal.json")]
    store: PathBuf,
}

#[derive(clap::Args)]
struct SubscriptionListArgs {
    topic: String,
    #[arg(long, default_value = ".tape/journal.json")]
    store: PathBuf,
}

#[derive(clap::Args)]
struct SubscriptionShowArgs {
    topic: String,
    name: String,
    #[arg(long, default_value = ".tape/journal.json")]
    store: PathBuf,
}

#[derive(clap::Args)]
struct SubscriptionPullArgs {
    topic: String,
    name: String,
    /// Maximum events to return; defaults to the bounded pull window.
    #[arg(long)]
    limit: Option<usize>,
    #[arg(long, default_value = ".tape/journal.json")]
    store: PathBuf,
}

#[derive(clap::Args)]
struct SubscriptionAckArgs {
    topic: String,
    name: String,
    /// Next unread offset after the events this caller processed.
    #[arg(long)]
    offset: u64,
    #[arg(long, default_value = ".tape/journal.json")]
    store: PathBuf,
}

#[derive(clap::Args)]
struct SubscriptionDeleteArgs {
    topic: String,
    name: String,
    #[arg(long, default_value = ".tape/journal.json")]
    store: PathBuf,
}

#[derive(clap::Args, Debug)]
struct ServeArgs {
    /// h2c + HTTP/1.1 listen address.
    #[arg(long, env = "TAPE_BIND", default_value = "127.0.0.1:7137")]
    bind: String,
    /// Journal file to load at boot and persist to on every mutation.
    /// Defaults to an empty in-memory journal when unset.
    #[arg(long, env = "TAPE_STORE")]
    store: Option<PathBuf>,
    /// Graceful-drain window (seconds) held after SIGTERM before the
    /// listener closes, while `/readyz` reports 503 so k8s stops routing.
    #[arg(long, env = "TAPE_GRACE_SECS", default_value_t = 10)]
    grace_secs: u64,
    /// Log output format. Kubernetes uses `json` for the shared
    /// `axiom.service.log.v1` collector contract; local development defaults
    /// to the human-readable formatter.
    #[arg(long, env = "TAPE_LOG_FORMAT", value_enum, default_value_t = LogFormat::Pretty)]
    log_format: LogFormat,
    /// OTLP gRPC endpoint for opt-in shared trace export. Unset preserves
    /// logging-only startup; requires the `otel` build feature to export.
    #[arg(long, env = "TAPE_OTLP_ENDPOINT")]
    otlp_endpoint: Option<String>,
    /// Request-auth mode for the /topics data plane: `off` (tokenless dev,
    /// the default) or `required` (bearer tokens from the registry file).
    /// Probes stay tokenless either way.
    #[arg(long, env = "TAPE_AUTH", default_value = "off")]
    auth: String,
    /// Bearer-token registry file (JSON `{token: {subject, roles}}`),
    /// mounted from a Secret in production. Required (and validated at
    /// startup) when `--auth required`.
    #[arg(long, env = "TAPE_TOKEN_REGISTRY_FILE")]
    token_registry_file: Option<PathBuf>,
    /// Durable directory for shared Raft hard state, committed log, and
    /// snapshots (#1327). Required in replica/HA mode (`REPLICAS_PER_SHARD > 1`);
    /// in single-node mode it selects `journal.json` unless `--store` is
    /// supplied explicitly.
    #[arg(long, env = "TAPE_DATA_DIR")]
    data_dir: Option<PathBuf>,
    /// Exact `file://` (or backup-enabled `s3://`) journal snapshot used only
    /// to seed a fresh replica PVC before Raft starts. Refuses non-empty
    /// `TAPE_DATA_DIR`; this is cold recovery, not a live restore endpoint.
    #[arg(long, env = "TAPE_BOOTSTRAP_SEED_URI")]
    bootstrap_seed_uri: Option<String>,
    /// Headless service name peers are resolved against in replica/HA mode
    /// (`ClusterTopology::from_env`).
    #[arg(long, env = "TAPE_PEER_SERVICE", default_value = "tape")]
    peer_service: String,
    /// Dedicated port for authenticated Raft peer RPCs when
    /// `TAPE_PEER_MTLS=on`. The public h2c port remains unchanged.
    #[arg(long, env = "TAPE_RAFT_PORT", default_value_t = 7138)]
    raft_port: u16,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum LogFormat {
    Pretty,
    Json,
}

/// `tape spec [--format ...]` or `tape spec gen ...`. Positional slots are
/// reserved for the `gen` subcommand; everything else is a flag (the CLI
/// convention).
#[derive(clap::Args)]
struct SpecArgs {
    /// Generate a typed client from the spec instead of printing it.
    #[command(subcommand)]
    gen: Option<SpecSub>,
    /// Contract format to print.
    #[arg(long, value_enum, default_value_t = SpecFormat::Openapi)]
    format: SpecFormat,
}

#[derive(Subcommand)]
enum SpecSub {
    /// Generate a typed API client (TypeScript / Python / Rust) from tape's
    /// OpenAPI document, written into `--out` (#1329).
    Gen(GenArgs),
}

#[derive(clap::Args)]
struct GenArgs {
    /// Target language for the generated client.
    #[arg(long, value_enum)]
    lang: GenLang,
    /// Output directory for the generated files.
    #[arg(long)]
    out: PathBuf,
    /// HTTP backend for the TypeScript client (ignored for py/rust).
    #[arg(long, value_enum, default_value_t = GenHttp::Fetch)]
    http: GenHttp,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum GenLang {
    /// TypeScript: types + fetch/axios client.
    Ts,
    /// Python: pydantic models + a generated HTTP client.
    Py,
    /// Rust: serde models + a reqwest client.
    Rust,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum GenHttp {
    Fetch,
    Axios,
}

#[derive(Clone, Copy, ValueEnum)]
enum SpecFormat {
    Openapi,
    OpenapiYaml,
    JsonSchema,
    Routes,
}

/// `tape backup` flags (#1329): pulls a snapshot over HTTP from a running
/// node and ships it to a destination via `libs/service-backup` (relay #1209
/// pattern).
#[derive(clap::Args)]
struct BackupArgs {
    /// Base URL of a running tape node, e.g.
    /// `http://<name>.<namespace>.svc.cluster.local:7137` (what the
    /// operator's backup CronJob passes) or `http://localhost:7137` for ad
    /// hoc use.
    #[arg(long)]
    url: String,
    /// Destination URI: `file:///path`, `s3://bucket/prefix`, or schema-only
    /// `gs://bucket/prefix` (parses, but the runner supports `file://` and
    /// `s3://` sinks today).
    #[arg(long)]
    dest: String,
    /// Bearer token for `/admin/backup` (needs `admin` on `*`). Falls back to
    /// `TAPE_BACKUP_TOKEN`; omit entirely when the node runs `--auth off`.
    #[arg(long, env = "TAPE_BACKUP_TOKEN")]
    token: Option<String>,
    /// Drop backup objects older than this many seconds after a successful
    /// put. Omit to keep everything.
    #[arg(long)]
    retention_secs: Option<u64>,
}

#[derive(clap::Args)]
struct LlmArgs {
    /// Topic: outline, workflow, api, or boundaries.
    #[arg(long, default_value = "outline")]
    topic: String,
    /// Output format: md or json.
    #[arg(long, default_value = "md")]
    format: String,
}

#[derive(clap::Args)]
struct UpgradeArgs {
    /// Report the current and latest version without modifying the binary.
    #[arg(long)]
    check: bool,
    /// Install this exact version (`0.4.3` or `tape@0.4.3`) instead of latest.
    #[arg(long = "version")]
    tag: Option<String>,
    /// Reinstall even when already on the selected version.
    #[arg(long)]
    force: bool,
    /// Skip the confirmation prompt.
    #[arg(short = 'y', long)]
    yes: bool,
}

#[derive(clap::Args)]
struct IssueArgs {
    #[command(subcommand)]
    command: IssueCommand,
}

#[derive(Subcommand)]
enum IssueCommand {
    /// Search Tape issues (`app:tape`); omit query to list recent.
    Search(IssueSearchArgs),
    /// Print one issue by number.
    View(IssueViewArgs),
    /// File a diagnostics-rich Tape issue.
    Create(IssueCreateArgs),
    /// Comment on an issue and ensure it is open.
    Comment(IssueCommentArgs),
}

#[derive(clap::Args)]
struct IssueSearchArgs {
    #[arg(value_name = "QUERY", num_args = 0..)]
    query: Vec<String>,
    #[arg(long, default_value = "open", value_parser = ["open", "closed", "all"])]
    state: String,
    #[arg(long, default_value_t = 20)]
    limit: u32,
}

#[derive(clap::Args)]
struct IssueViewArgs {
    number: u64,
}

#[derive(clap::Args)]
struct IssueCreateArgs {
    #[arg(short = 't', long)]
    title: Option<String>,
    #[arg(value_name = "MSG", num_args = 0..)]
    message: Vec<String>,
    #[arg(long)]
    url: Option<String>,
    #[arg(long)]
    repo: Option<String>,
    #[arg(long)]
    label: Vec<String>,
    #[arg(long)]
    dry_run: bool,
    #[arg(short = 'y', long)]
    yes: bool,
}

#[derive(clap::Args)]
struct IssueCommentArgs {
    number: u64,
    #[arg(value_name = "MSG", num_args = 0..)]
    message: Vec<String>,
    #[arg(long)]
    repo: Option<String>,
    #[arg(long)]
    dry_run: bool,
    #[arg(short = 'y', long)]
    yes: bool,
}

/// `tape k8s <crd|operator|instance>` — cluster artifacts split by lifecycle
/// layer (#1328).
#[derive(clap::Args, Debug)]
struct K8sArgs {
    #[command(subcommand)]
    cmd: K8sCmd,
}

#[derive(Subcommand, Debug)]
enum K8sCmd {
    /// Cluster-scoped API layer: render the Tape CRD.
    Crd(K8sCrdArgs),
    /// Operator control-plane layer: render assets or run the controller.
    Operator(K8sOperatorArgs),
    /// App-namespace declaration: render a Tape custom resource.
    Instance(K8sInstanceArgs),
}

#[derive(clap::Args, Debug)]
struct K8sCrdArgs {
    #[command(subcommand)]
    cmd: K8sCrdCmd,
}

#[derive(Subcommand, Debug)]
enum K8sCrdCmd {
    /// Render the Tape CustomResourceDefinition YAML.
    Render(K8sFileOutputArgs),
}

#[derive(clap::Args, Debug)]
struct K8sOperatorArgs {
    #[command(subcommand)]
    cmd: Option<K8sOperatorCmd>,
}

#[derive(Subcommand, Debug)]
enum K8sOperatorCmd {
    /// Container entrypoint: run the reconcile controller (needs `--features
    /// operator`). The default when no subcommand is given.
    Run,
    /// Render operator namespace/RBAC/deployment YAML.
    Render(K8sOperatorRenderArgs),
}

#[derive(clap::Args, Debug)]
struct K8sOperatorRenderArgs {
    /// Namespace that owns the operator control plane.
    #[arg(long, default_value = "tape-system")]
    namespace: String,
    /// Write to this path instead of stdout. A directory receives
    /// `operator.yaml`.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(clap::Args, Debug)]
struct K8sInstanceArgs {
    #[command(subcommand)]
    cmd: K8sInstanceCmd,
}

#[derive(Subcommand, Debug)]
enum K8sInstanceCmd {
    /// Render a namespaced `kind: Tape` custom resource.
    Render(K8sInstanceRenderArgs),
}

#[derive(clap::Args, Debug)]
struct K8sInstanceRenderArgs {
    /// Built-in instance profile.
    #[arg(long, value_enum, default_value_t = K8sInstanceProfile::Dev)]
    profile: K8sInstanceProfile,
    /// Tape CR name. HA (replicasPerShard > 1) instances must keep the
    /// default `tape` — serve derives raft peer DNS as
    /// `tape-<ordinal>.<peer-service>`.
    #[arg(long)]
    name: Option<String>,
    /// Namespace where the app-facing Tape instance lives.
    #[arg(long)]
    namespace: Option<String>,
    /// Journal image. Defaults are profile-specific.
    #[arg(long)]
    image: Option<String>,
    /// Write to this path instead of stdout. A directory receives `tape.yaml`.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum K8sInstanceProfile {
    /// Small local/kind CR: one journal pod, small disk, verbose logs.
    Dev,
    /// Pre-prod CR: prod-shaped single node, info logs, mid disk.
    Staging,
    /// Production-shape CR: 3-replica raft-HA group, large disk, auth on.
    Prod,
    /// Fill-in-the-blanks CR skeleton for app teams.
    Template,
}

#[derive(clap::Args, Debug)]
struct K8sFileOutputArgs {
    /// Write to this path instead of stdout.
    #[arg(long)]
    out: Option<PathBuf>,
}

/// `tape dockerfile <render>` — render tape's runtime image Dockerfiles.
#[derive(clap::Args, Debug)]
struct DockerfileArgs {
    #[command(subcommand)]
    cmd: DockerfileCmd,
}

#[derive(Subcommand, Debug)]
enum DockerfileCmd {
    /// Render a Dockerfile to stdout or `--out`.
    Render(DockerfileRenderArgs),
}

#[derive(clap::Args, Debug)]
struct DockerfileRenderArgs {
    /// Which runtime image contract to render.
    #[arg(long, value_enum, default_value_t = DockerfileVariant::Source)]
    variant: DockerfileVariant,
    /// Release tag used by `--variant release`; accepts `0.1.0` or
    /// `tape@0.1.0`.
    #[arg(long)]
    version: Option<String>,
    /// Write to this path instead of stdout. A directory receives `Dockerfile`
    /// or `Dockerfile.release`.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum DockerfileVariant {
    /// Build from the workspace source tree.
    Source,
    /// Fetch and verify a published `tape@<version>` release binary.
    Release,
}

const TOOL: cli_std::ToolInfo = cli_std::ToolInfo {
    project: "tape",
    repo: "chrischeng-c4/axiom",
    target: env!("TAPE_TARGET"),
    version: env!("CARGO_PKG_VERSION"),
    git_sha: env!("TAPE_GIT_SHA"),
    built_at: env!("TAPE_BUILT_AT"),
};

const LLM_TOPICS: &[cli_std::llm::Topic] = &[
    cli_std::llm::Topic {
        id: "workflow",
        summary: "topic replay model, first CLI slice, and deferred production gates",
        body: spec::llm_workflow_md(),
    },
    cli_std::llm::Topic {
        id: "api",
        summary: "append, replay, checkpoint, retention, and standard endpoint contract",
        body: spec::llm_api_md(),
    },
    cli_std::llm::Topic {
        id: "boundaries",
        summary: "Tape boundary against Relay, Loom, and Keep",
        body: spec::llm_boundaries_md(),
    },
    cli_std::llm::Topic {
        id: "operations",
        summary: "deploy artifacts — k8s crd/operator/instance render, dockerfile render",
        body: "# tape — deploying to Kubernetes\n\n\
            Deploy artifacts are offline renders; the checked-in files under `apps/tape/` \
            are the fixtures, and these commands are their in-binary form (#1328):\n\n\
            - `tape k8s crd render` — the Tape CustomResourceDefinition (tape.dev/v1alpha1).\n\
            - `tape k8s operator render [--namespace tape-system]` — operator RBAC + \
              Deployment; `tape k8s operator run` runs the reconcile controller (needs a \
              build with `--features operator`).\n\
            - `tape k8s instance render --profile dev|staging|prod|template` — a `kind: \
              Tape` CR; prod is the 3-replica raft-HA shape (the operator renders the \
              StatefulSet topology — `k8s/` base stays a single-node direct install for \
              kind/smoke).\n\
            - `tape dockerfile render --variant source|release [--version]` — the \
              from-source and published-release images.\n\n\
            HA is auto-mode raft: scale the StatefulSet and set `REPLICAS_PER_SHARD` > 1 \
            (plus `POD_NAME`, `SHARD_COUNT=1`, `VOTER_COUNT` from the downward API) and the \
            same `tape` bin runs a raft group; `--peer-service` (`TAPE_PEER_SERVICE`) names \
            the headless Service for peer DNS. No cluster env = plain single-node.\n",
    },
];

#[tokio::main]
async fn main() -> Result<()> {
    // kube-rs (operator), raft peer TLS, and online CLI paths can link
    // different rustls providers. Install the shared aws-lc-rs default before
    // any of those paths construct a TLS client or server.
    peer_tls::install_default_crypto_provider();
    let cli = Cli::parse();
    match cli.command {
        Command::Append(args) => append(args),
        Command::Replay(args) => replay(args),
        Command::Checkpoint(args) => checkpoint(args),
        Command::Subscription(args) => subscription(args),
        Command::Serve(args) => serve_main(args).await,
        Command::Spec(args) => spec(args),
        Command::Llm(args) => llm(args),
        Command::Upgrade(args) => {
            cli_std::upgrade::run(
                &TOOL,
                cli_std::upgrade::Options {
                    check: args.check,
                    tag: args.tag,
                    force: args.force,
                    yes: args.yes,
                },
            )
            .await
        }
        Command::Issue(args) => issue(args).await,
        Command::K8s(args) => k8s(args).await,
        Command::Dockerfile(args) => dockerfile(args),
        Command::Backup(args) => dispatch_backup(args).await,
    }
}

fn append(args: AppendArgs) -> Result<()> {
    let mut journal = load_journal(&args.store)?;
    let payload = parse_payload(&args.payload);
    let event = journal.append(args.topic, args.key, payload, args.timestamp_ms);
    save_journal(&args.store, &journal)?;
    println!("{}", serde_json::to_string_pretty(&event)?);
    println!(
        "next: tape replay {} --from-offset {}",
        event.topic, event.offset
    );
    Ok(())
}

fn replay(args: ReplayArgs) -> Result<()> {
    let journal = load_journal(&args.store)?;
    let events = journal.replay(
        &args.topic,
        args.from_offset,
        args.from_timestamp_ms,
        args.limit,
    );
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({ "events": events }))?
    );
    println!("next: done");
    Ok(())
}

fn checkpoint(args: CheckpointArgs) -> Result<()> {
    match args.command {
        CheckpointCommand::Get(args) => {
            let journal = load_journal(&args.store)?;
            let checkpoint = journal.checkpoint(&args.topic, &args.consumer);
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({ "checkpoint": checkpoint }))?
            );
            println!("next: done");
            Ok(())
        }
        CheckpointCommand::Put(args) => {
            let mut journal = load_journal(&args.store)?;
            let checkpoint = journal.put_checkpoint(args.topic, args.consumer, args.offset)?;
            save_journal(&args.store, &journal)?;
            println!("{}", serde_json::to_string_pretty(&checkpoint)?);
            println!("next: done");
            Ok(())
        }
    }
}

// @spec apps/tape/tech-design/logic/expose-subscriptions-as-topic-delivery-resources.md#changes
fn subscription(args: SubscriptionArgs) -> Result<()> {
    match args.command {
        SubscriptionCommand::Create(args) => {
            let mut journal = load_journal(&args.store)?;
            let subscription = journal.create_subscription(args.topic, args.name)?;
            save_journal(&args.store, &journal)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({ "subscription": subscription }))?
            );
            println!(
                "next: tape subscription show {} {} --store {}",
                subscription.topic,
                subscription.name,
                args.store.display()
            );
            Ok(())
        }
        SubscriptionCommand::List(args) => {
            let journal = load_journal(&args.store)?;
            let subscriptions = journal.subscriptions(&args.topic);
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({ "subscriptions": subscriptions }))?
            );
            println!("next: done");
            Ok(())
        }
        SubscriptionCommand::Show(args) => {
            let journal = load_journal(&args.store)?;
            let subscription = journal
                .subscription(&args.topic, &args.name)
                .cloned()
                .with_context(|| {
                    format!(
                        "subscription {} does not exist for topic {}",
                        args.name, args.topic
                    )
                })?;
            let checkpoint = journal.checkpoint(&subscription.topic, &subscription.name);
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &json!({ "subscription": subscription, "checkpoint": checkpoint })
                )?
            );
            println!("next: done");
            Ok(())
        }
        SubscriptionCommand::Pull(args) => {
            let journal = load_journal(&args.store)?;
            let batch = journal.pull_subscription(&args.topic, &args.name, args.limit)?;
            let next = if batch.events.is_empty() {
                "done".to_string()
            } else {
                format!(
                    "tape subscription ack {} {} --offset {} --store {}",
                    batch.topic,
                    batch.subscription,
                    batch.next_offset,
                    args.store.display()
                )
            };
            println!("{}", serde_json::to_string_pretty(&batch)?);
            println!("next: {next}");
            Ok(())
        }
        SubscriptionCommand::Ack(args) => {
            let mut journal = load_journal(&args.store)?;
            let checkpoint = journal.ack_subscription(&args.topic, &args.name, args.offset)?;
            save_journal(&args.store, &journal)?;
            println!("{}", serde_json::to_string_pretty(&checkpoint)?);
            println!(
                "next: tape subscription pull {} {} --store {}",
                args.topic,
                args.name,
                args.store.display()
            );
            Ok(())
        }
        SubscriptionCommand::Delete(args) => {
            let mut journal = load_journal(&args.store)?;
            let subscription = journal.delete_subscription(&args.topic, &args.name)?;
            save_journal(&args.store, &journal)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({ "deleted": subscription }))?
            );
            println!(
                "next: tape subscription list {} --store {}",
                args.topic,
                args.store.display()
            );
            Ok(())
        }
    }
}

/// Run the tape HTTP server: load the journal from `--store` (or start
/// empty), serve the shared service shell (standard probes merged with the
/// `/topics` data plane) over HTTP/1.1 + h2c on one port, with a
/// SIGTERM-aware graceful drain (`--grace-secs`).
async fn serve_main(args: ServeArgs) -> Result<()> {
    let log_format = match args.log_format {
        LogFormat::Pretty => service_http::LogFormat::Pretty,
        LogFormat::Json => service_http::LogFormat::Json,
    };
    let tracing_config = service_http::HttpConfig::new(
        "127.0.0.1",
        0,
        "info",
        log_format,
        args.grace_secs,
        0,
        args.otlp_endpoint.clone(),
    );
    let tracing_identity = service_http::ServiceIdentity::new("tape", env!("CARGO_PKG_VERSION"))?;
    service_http::init_tracing_with_identity(&tracing_config, &tracing_identity)?;

    // Resolve the bearer-auth contract (#1326) BEFORE anything serves: with
    // --auth required a missing/unparseable/empty registry file is a startup
    // error (nonzero exit), never a per-request 401.
    let auth = tape::auth::AuthConfig::resolve(
        &args.auth,
        args.token_registry_file
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned())
            .as_deref(),
        std::env::var(tape::auth::LEGACY_TOKENS_ENV).ok().as_deref(),
    )?;
    tracing::info!(
        required = auth.required,
        "request auth resolved (TAPE_AUTH; probes stay tokenless)"
    );
    let admission = service_http::AdmissionConfig::from_env("TAPE")?.controller(
        "tape.read",
        "tape.write",
        "tape.admin",
    );
    if admission.is_some() {
        tracing::info!(
            "request admission enabled (TAPE_ADMISSION_*; probes and peer routes stay exempt)"
        );
    }

    // The operator mounts `/data` for every StatefulSet member. In its
    // single-node topology there is no Raft state machine to own durability,
    // so use that PVC for the ordinary journal store. An explicit --store
    // still wins, and replica mode continues to keep journal durability in
    // the Raft state machine instead of a second local store.
    let replica_mode = raft_runtime::cluster::replica_mode();
    let store = resolve_journal_store(args.store.clone(), args.data_dir.as_deref(), replica_mode);
    let journal = match &store {
        Some(path) => load_journal(path)?,
        None => TapeJournal::default(),
    };
    let mut state = tape::server::AppState::with_auth(journal, store, auth);
    if let Some(path) = args.token_registry_file.as_deref() {
        // `AppState` owns this exact verifier instance, so a Secret/CSI file
        // replacement becomes visible to the live data-plane middleware
        // without restarting the Tape pod. Invalid replacements remain on the
        // shared last-known-good snapshot and emit redacted audit events.
        std::mem::drop(service_auth::spawn_registry_file_watcher(
            state.verifier(),
            path,
        ));
        tracing::info!(
            path = %path.display(),
            "watching bearer token registry for credential rotation"
        );
    }

    // Auto-mode HA (#1327): the standard downward-API quartet flips replica
    // mode (REPLICAS_PER_SHARD > 1) — no tape-specific flag. With no peer TLS
    // material the established public-port h2c topology is unchanged. A
    // complete required mTLS configuration instead makes Raft use https URLs
    // and the dedicated peer listener below.
    let mut peer_transport = None;
    let mut peer_router = None;
    if args.bootstrap_seed_uri.is_some() {
        anyhow::ensure!(
            replica_mode,
            "--bootstrap-seed-uri (TAPE_BOOTSTRAP_SEED_URI) requires replica/HA mode"
        );
        anyhow::ensure!(
            args.store.is_none(),
            "--bootstrap-seed-uri cannot be combined with --store; seed only a fresh replica PVC"
        );
    }
    if replica_mode {
        let transport = tape::peer_tls::from_env()?
            .map(|config| tape::peer_tls::peer_transport(&config))
            .transpose()
            .context("raft: build shared required-mTLS peer transport")?;
        let (peer_port, peer_scheme) = match transport.as_ref() {
            Some(_) => (args.raft_port, "https"),
            None => (
                args.bind
                    .rsplit(':')
                    .next()
                    .and_then(|port| port.parse::<u16>().ok())
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "cannot derive the h2c raft peer port from --bind {}",
                            args.bind
                        )
                    })?,
                "http",
            ),
        };
        let topo = raft_runtime::ClusterTopology::from_env_with_scheme(
            "tape",
            &args.peer_service,
            peer_port,
            "TAPE_PEERS",
            peer_scheme,
        )?;
        let data_dir = args.data_dir.clone().ok_or_else(|| {
            anyhow::anyhow!("replica/HA mode requires a durable --data-dir (TAPE_DATA_DIR)")
        })?;
        anyhow::ensure!(
            !data_dir.as_os_str().is_empty(),
            "replica/HA mode requires a durable --data-dir (TAPE_DATA_DIR)"
        );
        if let Some(seed_uri) = args.bootstrap_seed_uri.as_deref() {
            let bytes = service_backup::fetch_backup_object(seed_uri)
                .with_context(|| format!("read bootstrap seed {seed_uri}"))?;
            tape::raft::prepare_bootstrap_seed(&data_dir, topo.node_id, &bytes)?;
            tracing::info!(
                seed_uri,
                bytes = bytes.len(),
                "bootstrap seed prepared before raft catch-up"
            );
        }
        let raft = std::sync::Arc::new(match transport.clone() {
            Some(transport) => tape::raft::TapeRaft::from_topology_with_peer_transport(
                state.journal_handle(),
                &data_dir,
                &topo,
                tape::raft::TapeRaft::host_config(tape::raft::SNAPSHOT_EVERY),
                transport,
            )?,
            None => tape::raft::TapeRaft::from_topology(
                state.journal_handle(),
                &data_dir,
                &topo,
                tape::raft::TapeRaft::host_config(tape::raft::SNAPSHOT_EVERY),
            )?,
        });
        tracing::info!(
            node_id = topo.node_id,
            replicas = topo.replicas_per_shard,
            voters = topo.membership.voters.len(),
            peer_scheme,
            peer_port,
            "raft: replica/HA mode — append/checkpoint-put replicate"
        );
        if let Some(transport) = transport {
            peer_router = Some(raft.router());
            peer_transport = Some(transport);
        }
        state.set_raft(raft);
    }

    let app = if peer_transport.is_some() {
        tape::server::router_without_raft_routes_with_admission(state.clone(), admission)
    } else {
        tape::server::router_with_admission(state.clone(), admission)
    };

    let listener = tokio::net::TcpListener::bind(&args.bind).await?;
    tracing::info!(
        addr = %listener.local_addr()?,
        "tape listening (HTTP/1.1 + HTTP/2 cleartext)"
    );

    let peer_server = match (peer_transport, peer_router) {
        (Some(transport), Some(router)) => {
            let peer_bind = peer_bind_address(&args.bind, args.raft_port)?;
            let peer_listener = tokio::net::TcpListener::bind(&peer_bind)
                .await
                .with_context(|| format!("bind authenticated raft peer listener {peer_bind}"))?;
            let (shutdown, shutdown_rx) = tokio::sync::oneshot::channel();
            tracing::info!(
                addr = %peer_bind,
                tls_generation = transport.generation(),
                "tape raft peer mTLS listening"
            );
            let serve = tokio::spawn(async move {
                transport
                    .serve(peer_listener, router, async move {
                        let _ = shutdown_rx.await;
                    })
                    .await
            });
            Some((shutdown, serve))
        }
        (None, None) => None,
        _ => unreachable!("peer transport and router are configured together"),
    };

    let grace = Duration::from_secs(args.grace_secs);
    let drain_state = state.clone();
    service_http::serve(
        listener,
        app,
        service_http::shutdown_with_drain(move || drain_state.start_drain(), grace),
    )
    .await;
    if let Some((shutdown, serve)) = peer_server {
        let _ = shutdown.send(());
        serve.await.context("raft peer listener task panicked")??;
    }
    Ok(())
}

// <HANDWRITE gap="missing-generator:serve-peer-transport" tracker="#1805" reason="serve-peer-transport section in tape.rs is hand-written pending codegen support">
/// Resolve the local journal path for a serving process. Replica mode owns
/// durability through Raft; only a single-node process derives a store from
/// its mounted data directory.
fn resolve_journal_store(
    explicit_store: Option<PathBuf>,
    data_dir: Option<&std::path::Path>,
    replica_mode: bool,
) -> Option<PathBuf> {
    explicit_store.or_else(|| {
        if replica_mode {
            None
        } else {
            data_dir.map(|dir| dir.join("journal.json"))
        }
    })
}

/// Reuse the public listener's host portion for the dedicated peer port.
/// This preserves `0.0.0.0`, hostname, and bracketed IPv6 bindings without
/// allowing a Raft port to silently replace the public data-plane port.
fn peer_bind_address(bind: &str, raft_port: u16) -> Result<String> {
    let (host, _) = bind.rsplit_once(':').ok_or_else(|| {
        anyhow::anyhow!("cannot derive authenticated raft bind address from --bind {bind}")
    })?;
    anyhow::ensure!(
        !host.is_empty(),
        "--bind must include a host before its port"
    );
    Ok(format!("{host}:{raft_port}"))
}
// </HANDWRITE>

fn spec(args: SpecArgs) -> Result<()> {
    // `spec gen` writes a typed client; everything else prints to stdout.
    if let Some(SpecSub::Gen(gen)) = args.gen {
        return spec_gen(gen);
    }
    let out = match args.format {
        SpecFormat::Openapi => spec::openapi_json(),
        SpecFormat::OpenapiYaml => spec::openapi_yaml(),
        SpecFormat::JsonSchema => spec::json_schema_json(),
        SpecFormat::Routes => spec::routes_json(),
    };
    println!("{out}");
    Ok(())
}

/// `tape spec gen` — generate a typed client from tape's own OpenAPI
/// document (offline; no server) via the shared `libs/openapi-codegen`,
/// written into `--out`. One codegen path, no external tool (relay #1209
/// pattern).
fn spec_gen(args: GenArgs) -> Result<()> {
    use cclab_openapi_codegen::{generate, GenOptions, HttpClient, Lang};
    let lang = match args.lang {
        GenLang::Ts => Lang::Ts,
        GenLang::Py => Lang::Py,
        GenLang::Rust => Lang::Rust,
    };
    let opts = GenOptions {
        lang,
        spec_path: PathBuf::new(),
        out_dir: args.out.clone(),
        client_name: "createClient".to_string(),
        http_client: match args.http {
            GenHttp::Fetch => HttpClient::Fetch,
            GenHttp::Axios => HttpClient::Axios,
        },
        emit_types: true,
        emit_client: true,
        // TanStack Query hooks are a TypeScript-only concern.
        emit_hooks: matches!(lang, Lang::Ts),
    };
    let output = generate(&spec::openapi_json(), &opts)?;
    std::fs::create_dir_all(&args.out)?;
    for file in &output.files {
        let path = args.out.join(&file.rel_path);
        std::fs::write(&path, &file.contents)?;
        println!("generated {}", path.display());
    }
    Ok(())
}

/// `tape backup` (#1329): fetch `{url}/admin/backup` and ship the bytes to
/// `--dest` via `libs/service-backup`, printing the resulting
/// `BackupRunResult` as JSON. This is what the operator's optional backup
/// CronJob invokes on a schedule; it works equally ad hoc.
#[cfg(feature = "backup")]
async fn dispatch_backup(args: BackupArgs) -> Result<()> {
    let dest = service_backup::BackupDestination::from_uri(&args.dest)?;
    let retention = match args.retention_secs {
        Some(secs) => service_backup::RetentionPolicy::max_age_seconds(secs),
        None => service_backup::RetentionPolicy::default(),
    };
    let result =
        tape::backup::run_backup(&args.url, args.token.as_deref(), &dest, &retention).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#[cfg(not(feature = "backup"))]
async fn dispatch_backup(_args: BackupArgs) -> Result<()> {
    anyhow::bail!(
        "this tape build was compiled without backup support; rebuild with \
         `--features backup` (the published image includes it)"
    )
}

fn llm(args: LlmArgs) -> Result<()> {
    let out = cli_std::llm::render(
        TOOL.project,
        TOOL.version,
        LLM_TOPICS,
        &args.topic,
        cli_std::llm::Format::parse(&args.format),
    )?;
    println!("{out}");
    Ok(())
}

async fn issue(args: IssueArgs) -> Result<()> {
    match args.command {
        IssueCommand::Search(args) => {
            let query = (!args.query.is_empty()).then(|| args.query.join(" "));
            cli_std::issue::search(
                &TOOL,
                cli_std::issue::SearchOptions {
                    query,
                    state: args.state,
                    limit: args.limit,
                },
            )
            .await
        }
        IssueCommand::View(args) => cli_std::issue::view(&TOOL, args.number).await,
        IssueCommand::Create(args) => {
            let message = (!args.message.is_empty()).then(|| args.message.join(" "));
            let title = args.title.unwrap_or_else(|| {
                message
                    .as_deref()
                    .and_then(|msg| msg.lines().next())
                    .map(|head| format!("tape: {}", head.chars().take(72).collect::<String>()))
                    .unwrap_or_else(|| "tape: issue report".to_string())
            });
            cli_std::issue::create(
                &TOOL,
                cli_std::issue::CreateOptions {
                    title,
                    message,
                    url: args.url,
                    repo: args.repo,
                    label: std::iter::once("app:tape".to_string())
                        .chain(args.label)
                        .collect(),
                    dry_run: args.dry_run,
                    yes: args.yes,
                },
            )
            .await
        }
        IssueCommand::Comment(args) => {
            let message = (!args.message.is_empty()).then(|| args.message.join(" "));
            cli_std::issue::comment(
                &TOOL,
                cli_std::issue::CommentOptions {
                    number: args.number,
                    message,
                    repo: args.repo,
                    dry_run: args.dry_run,
                    yes: args.yes,
                },
            )
            .await
        }
    }
}

fn load_journal(path: &Path) -> Result<TapeJournal> {
    if !path.exists() {
        return Ok(TapeJournal::default());
    }
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))
}

fn save_journal(path: &Path, journal: &TapeJournal) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let bytes = serde_json::to_vec_pretty(journal)?;
    fs::write(path, bytes).with_context(|| format!("write {}", path.display()))
}

fn parse_payload(input: &str) -> Value {
    serde_json::from_str(input).unwrap_or_else(|_| Value::String(input.to_string()))
}

/// `tape k8s` — cluster artifacts split by lifecycle layer. Only `operator
/// run` needs kube-rs at runtime; the render paths are offline and work from
/// the binary (the generated CRD is embedded, the operator manifests are
/// string-templated, the instance CRs are profile-templated) (#1328).
async fn k8s(args: K8sArgs) -> Result<()> {
    match args.cmd {
        K8sCmd::Crd(a) => match a.cmd {
            K8sCrdCmd::Render(a) => write_or_print(a.out.as_deref(), "crd.yaml", &crd_yaml()),
        },
        K8sCmd::Operator(a) => match a.cmd.unwrap_or(K8sOperatorCmd::Run) {
            K8sOperatorCmd::Run => run_operator().await,
            K8sOperatorCmd::Render(a) => {
                let yaml = render_operator_yaml(&a.namespace);
                write_or_print(a.out.as_deref(), "operator.yaml", &yaml)
            }
        },
        K8sCmd::Instance(a) => match a.cmd {
            K8sInstanceCmd::Render(a) => {
                let yaml = render_instance_yaml(&a);
                write_or_print(a.out.as_deref(), "tape.yaml", &yaml)
            }
        },
    }
}

#[cfg(feature = "operator")]
async fn run_operator() -> Result<()> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
    tape::operator::run().await
}

#[cfg(not(feature = "operator"))]
async fn run_operator() -> Result<()> {
    anyhow::bail!(
        "this tape build was compiled without operator support; rebuild with \
         `--features operator` (the published image includes it)"
    )
}

#[cfg(feature = "operator")]
fn crd_yaml() -> String {
    tape::operator::crd_yaml()
}

#[cfg(not(feature = "operator"))]
fn crd_yaml() -> String {
    cli_std::artifact::ensure_trailing_newline(include_str!("../../k8s/operator/crd.yaml"))
}

/// Render the operator control-plane manifests (RBAC + Deployment) with the
/// namespace substituted, from the checked-in fixtures.
fn render_operator_yaml(namespace: &str) -> String {
    let mut out = String::new();
    out.push_str(&cli_std::artifact::replace_kubernetes_namespace(
        include_str!("../../k8s/operator/rbac.yaml"),
        "tape-system",
        namespace,
    ));
    out.push_str("\n---\n");
    out.push_str(&cli_std::artifact::replace_kubernetes_namespace(
        include_str!("../../k8s/operator/deployment.yaml"),
        "tape-system",
        namespace,
    ));
    cli_std::artifact::ensure_trailing_newline(&out)
}

/// Render a `kind: Tape` custom resource for the selected profile.
fn render_instance_yaml(args: &K8sInstanceRenderArgs) -> String {
    let default_version = env!("CARGO_PKG_VERSION");
    let (default_name, default_namespace, default_image, body) = match args.profile {
        K8sInstanceProfile::Dev => (
            "tape",
            "default",
            "tape:latest".to_string(),
            InstanceBody::Dev,
        ),
        K8sInstanceProfile::Staging => (
            "tape",
            "staging",
            format!("tape:{default_version}"),
            InstanceBody::Staging,
        ),
        K8sInstanceProfile::Prod => (
            "tape",
            "production",
            format!("registry.example.com/tape:{default_version}"),
            InstanceBody::Prod,
        ),
        K8sInstanceProfile::Template => (
            "tape",
            "REPLACE_ME__APP_NAMESPACE",
            "REPLACE_ME__REGISTRY/tape:REPLACE_ME__IMAGE_TAG".to_string(),
            InstanceBody::Template,
        ),
    };
    let name = args.name.as_deref().unwrap_or(default_name);
    let namespace = args.namespace.as_deref().unwrap_or(default_namespace);
    let image = args.image.as_deref().unwrap_or(&default_image);

    let mut yaml = format!(
        "apiVersion: tape.dev/v1alpha1\nkind: Tape\nmetadata:\n  name: {name}\n  namespace: {namespace}\nspec:\n  image: {image}\n"
    );
    match body {
        InstanceBody::Dev => {
            yaml.push_str(
                "  replicasPerShard: 1\n  voterCount: 1\n  logLevel: debug\n  storage: 1Gi\n  resources:\n    cpu: \"1\"\n    memory: 4Gi\n",
            );
        }
        InstanceBody::Staging => {
            yaml.push_str(
                "  replicasPerShard: 1\n  voterCount: 1\n  logLevel: info\n  storage: 20Gi\n  resources:\n    cpu: \"1\"\n    memory: 4Gi\n",
            );
        }
        InstanceBody::Prod => {
            yaml.push_str(
                "  imagePullPolicy: Always\n  replicasPerShard: 3\n  voterCount: 3\n  logLevel: info\n  storage: 100Gi\n  graceSecs: 30\n  auth: required\n  tokensSecret: tape-token-registry\n  resources:\n    cpu: \"1\"\n    memory: 4Gi\n",
            );
        }
        InstanceBody::Template => {
            yaml.push_str(
                "  imagePullPolicy: IfNotPresent\n  replicasPerShard: REPLACE_ME__REPLICAS_PER_SHARD\n  voterCount: REPLACE_ME__VOTER_COUNT\n  storage: 10Gi\n  resources:\n    cpu: \"1\"\n    memory: 4Gi\n",
            );
        }
    }
    cli_std::artifact::ensure_trailing_newline(&yaml)
}

enum InstanceBody {
    Dev,
    Staging,
    Prod,
    Template,
}

/// `tape dockerfile render` — render tape's runtime image Dockerfiles. The
/// checked-in Dockerfiles are the fixtures; the CLI is their in-binary form
/// (marker stripping + `tape@version` substitution), so `render` stays the
/// source of truth (relay #1208 pattern).
fn dockerfile(args: DockerfileArgs) -> Result<()> {
    match args.cmd {
        DockerfileCmd::Render(a) => {
            let (file_name, body) = match a.variant {
                DockerfileVariant::Source => ("Dockerfile", render_source_dockerfile()),
                DockerfileVariant::Release => (
                    "Dockerfile.release",
                    render_release_dockerfile(a.version.as_deref()),
                ),
            };
            write_or_print(a.out.as_deref(), file_name, &body)
        }
    }
}

fn render_source_dockerfile() -> String {
    cli_std::artifact::strip_source_ownership_markers(include_str!("../../Dockerfile"))
}

fn render_release_dockerfile(version: Option<&str>) -> String {
    let tag = cli_std::artifact::release_tag("tape", version, env!("CARGO_PKG_VERSION"));
    let version = tag.trim_start_matches("tape@");
    let template =
        cli_std::artifact::strip_source_ownership_markers(include_str!("../../Dockerfile.release"));
    let mut out = String::new();
    for line in template.lines() {
        if line.starts_with("#   docker build -f apps/tape/Dockerfile.release -t tape:") {
            out.push_str(&format!(
                "#   docker build -f apps/tape/Dockerfile.release -t tape:{version} \\"
            ));
        } else if line.starts_with("#     --build-arg TAPE_VERSION=") {
            out.push_str(&format!("#     --build-arg TAPE_VERSION={tag} ."));
        } else if line.starts_with("ARG TAPE_VERSION=") {
            out.push_str(&format!("ARG TAPE_VERSION={tag}"));
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

/// Write `body` to `out` (a file, or `default_file` inside a directory) or
/// print it to stdout.
fn write_or_print(out: Option<&Path>, default_file: &str, body: &str) -> Result<()> {
    cli_std::artifact::write_or_print(out, default_file, body)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_parse_surface() {
        Cli::command().debug_assert();

        // #1325: `serve` gains --bind/--store/--grace-secs, with env fallback
        // and a 10s default grace window; existing commands keep parsing.
        // #1326: `serve` also gains --auth/--token-registry-file, defaulting
        // to tokenless (`off`).
        let cli = Cli::try_parse_from(["tape", "serve"]).unwrap();
        let Command::Serve(args) = cli.command else {
            panic!("expected Serve");
        };
        assert_eq!(args.bind, "127.0.0.1:7137");
        assert!(args.store.is_none());
        assert_eq!(args.grace_secs, 10);
        assert_eq!(args.auth, "off");
        assert!(args.token_registry_file.is_none());

        let cli = Cli::try_parse_from([
            "tape",
            "serve",
            "--bind",
            "0.0.0.0:9000",
            "--store",
            "/tmp/journal.json",
            "--grace-secs",
            "3",
            "--auth",
            "required",
            "--token-registry-file",
            "/tmp/tape-token-registry.json",
        ])
        .unwrap();
        let Command::Serve(args) = cli.command else {
            panic!("expected Serve");
        };
        assert_eq!(args.bind, "0.0.0.0:9000");
        assert_eq!(args.store, Some(PathBuf::from("/tmp/journal.json")));
        assert_eq!(args.grace_secs, 3);
        assert_eq!(args.auth, "required");
        assert_eq!(
            args.token_registry_file,
            Some(PathBuf::from("/tmp/tape-token-registry.json"))
        );

        let cli =
            Cli::try_parse_from(["tape", "append", "orders", "--payload", "{\"n\":1}"]).unwrap();
        assert!(matches!(cli.command, Command::Append(_)));
    }

    #[test]
    fn single_node_data_dir_resolves_to_the_pvc_journal_store() {
        let data_dir = PathBuf::from("/data");
        assert_eq!(
            resolve_journal_store(None, Some(&data_dir), false),
            Some(PathBuf::from("/data/journal.json"))
        );
        assert_eq!(
            resolve_journal_store(
                Some(PathBuf::from("/tmp/override.json")),
                Some(&data_dir),
                false,
            ),
            Some(PathBuf::from("/tmp/override.json")),
            "an explicit --store must keep precedence over TAPE_DATA_DIR"
        );
        assert_eq!(
            resolve_journal_store(None, Some(&data_dir), true),
            None,
            "replica mode persists through Raft rather than a parallel journal file"
        );
    }

    /// #1328: `tape k8s <crd|operator|instance>` parses with the expected
    /// subcommands and flags.
    #[test]
    fn k8s_verbs_parse() {
        let cli = Cli::try_parse_from(["tape", "k8s", "crd", "render"]).expect("crd render");
        assert!(matches!(
            cli.command,
            Command::K8s(K8sArgs {
                cmd: K8sCmd::Crd(K8sCrdArgs {
                    cmd: K8sCrdCmd::Render(_),
                }),
            })
        ));

        let cli = Cli::try_parse_from([
            "tape",
            "k8s",
            "instance",
            "render",
            "--profile",
            "prod",
            "--namespace",
            "production",
        ])
        .expect("instance render");
        match cli.command {
            Command::K8s(K8sArgs {
                cmd:
                    K8sCmd::Instance(K8sInstanceArgs {
                        cmd: K8sInstanceCmd::Render(a),
                    }),
            }) => {
                assert!(matches!(a.profile, K8sInstanceProfile::Prod));
                assert_eq!(a.namespace.as_deref(), Some("production"));
            }
            _ => panic!("expected k8s instance render"),
        }

        // `operator` with no subcommand defaults to `run`.
        let cli = Cli::try_parse_from(["tape", "k8s", "operator"]).expect("operator default");
        match cli.command {
            Command::K8s(K8sArgs {
                cmd: K8sCmd::Operator(K8sOperatorArgs { cmd }),
            }) => assert!(cmd.is_none()),
            _ => panic!("expected k8s operator"),
        }
    }

    /// #1329: `tape backup` parses its flags (url/dest/token/retention-secs).
    #[test]
    fn backup_verb_parses() {
        let cli = Cli::try_parse_from([
            "tape",
            "backup",
            "--url",
            "http://localhost:7137",
            "--dest",
            "file:///tmp/backups",
            "--token",
            "s3cr3t",
            "--retention-secs",
            "3600",
        ])
        .expect("backup should parse");
        match cli.command {
            Command::Backup(a) => {
                assert_eq!(a.url, "http://localhost:7137");
                assert_eq!(a.dest, "file:///tmp/backups");
                assert_eq!(a.token.as_deref(), Some("s3cr3t"));
                assert_eq!(a.retention_secs, Some(3600));
            }
            _ => panic!("expected backup"),
        }
    }

    /// #1329: `tape spec gen` parses its flags and generates non-empty client
    /// output for each supported language from tape's own OpenAPI document.
    #[test]
    fn spec_gen_verbs_parse_and_generate() {
        let cli = Cli::try_parse_from(["tape", "spec", "gen", "--lang", "ts", "--out", "/tmp/x"])
            .expect("spec gen should parse");
        match cli.command {
            Command::Spec(SpecArgs {
                gen: Some(SpecSub::Gen(a)),
                ..
            }) => {
                assert!(matches!(a.lang, GenLang::Ts));
                assert_eq!(a.out, PathBuf::from("/tmp/x"));
            }
            _ => panic!("expected spec gen"),
        }

        for lang in [GenLang::Ts, GenLang::Py, GenLang::Rust] {
            let opts = cclab_openapi_codegen::GenOptions {
                lang: match lang {
                    GenLang::Ts => cclab_openapi_codegen::Lang::Ts,
                    GenLang::Py => cclab_openapi_codegen::Lang::Py,
                    GenLang::Rust => cclab_openapi_codegen::Lang::Rust,
                },
                spec_path: PathBuf::new(),
                out_dir: PathBuf::new(),
                client_name: "createClient".to_string(),
                http_client: cclab_openapi_codegen::HttpClient::Fetch,
                emit_types: true,
                emit_client: true,
                emit_hooks: matches!(lang, GenLang::Ts),
            };
            let out = cclab_openapi_codegen::generate(&spec::openapi_json(), &opts)
                .expect("spec gen should succeed for tape's own OpenAPI document");
            assert!(
                !out.files.is_empty(),
                "{lang:?} should emit at least one file"
            );
        }
    }

    /// #1328: `tape dockerfile render` parses with variant/version/out flags,
    /// and the shared tag helper converges bare/prefixed tags.
    #[test]
    fn dockerfile_verbs_parse() {
        let cli = Cli::try_parse_from([
            "tape",
            "dockerfile",
            "render",
            "--variant",
            "release",
            "--version",
            "1.2.3",
        ])
        .expect("dockerfile render should parse");
        match cli.command {
            Command::Dockerfile(DockerfileArgs {
                cmd: DockerfileCmd::Render(a),
            }) => {
                assert!(matches!(a.variant, DockerfileVariant::Release));
                assert_eq!(a.version.as_deref(), Some("1.2.3"));
            }
            _ => panic!("expected dockerfile render"),
        }
        assert_eq!(
            cli_std::artifact::release_tag("tape", Some("1.2.3"), env!("CARGO_PKG_VERSION")),
            "tape@1.2.3"
        );
        assert_eq!(
            cli_std::artifact::release_tag("tape", Some("tape@1.2.3"), env!("CARGO_PKG_VERSION")),
            "tape@1.2.3"
        );
        assert_eq!(
            cli_std::artifact::release_tag("tape", None, env!("CARGO_PKG_VERSION")),
            format!("tape@{}", env!("CARGO_PKG_VERSION"))
        );
    }
}
// </HANDWRITE>
