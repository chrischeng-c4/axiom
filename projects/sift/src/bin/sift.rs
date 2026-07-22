// HANDWRITE-BEGIN gap="sift-service-cli" tracker="1576" reason="Implement serve, event, query, replay, spec, llm, upgrade, and issue CLI surfaces."
use std::{path::PathBuf, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use axum::extract::DefaultBodyLimit;
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;
use serde_json::Value;
use sift::{
    auth::SiftVerifier,
    collector::{
        CollectorConfig, CriMetadata, CriSourceConfig, SourceSpec, DEFAULT_BATCH_SIZE,
        DEFAULT_MAX_LINE_BYTES, DEFAULT_MAX_RETRIES,
    },
    decode_event_json,
    deploy::{DockerfileVariant, InstanceProfile},
    DurableJournal, EventQuery, ServiceState, SignalKind,
};

#[derive(Parser)]
#[command(
    name = "sift",
    version,
    about = "Sift — operational event service and CLI"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the unified h2c/HTTP1 operational-event service.
    Serve(ServeArgs),
    /// Collect axiom.service.log.v1 JSONL from a file, stdin, or Kubernetes CRI logs.
    Collect(CollectArgs),
    /// Write or import versioned events into the local raw journal.
    Event(EventArgs),
    /// Query durable raw events without starting a server.
    Query(QueryArgs),
    /// Replay durable raw events after a cursor without starting a server.
    Replay(ReplayArgs),
    /// Write a consistent Sift journal snapshot to stdout or a local file.
    Snapshot(SnapshotArgs),
    /// Restore a journal snapshot from a shared backup object URI.
    Restore(RestoreArgs),
    /// Ship a live or explicitly offline journal snapshot through the shared backup contract.
    Backup(BackupArgs),
    /// Render source or release image Dockerfiles independently of Kubernetes.
    Dockerfile(DockerfileArgs),
    /// Render cluster CRD, operator control plane, or namespaced Sift instances.
    K8s(K8sArgs),
    /// Run a command through a managed Kubernetes port-forward to Sift.
    Connect(ConnectArgs),
    /// Print Sift's API contract or generate a typed client from it.
    Spec(SpecArgs),
    /// Print offline agent-facing operational documentation.
    Llm(LlmArgs),
    /// Check or install a released Sift binary.
    Upgrade(UpgradeArgs),
    /// Search, inspect, or file Sift issues.
    Issue(IssueArgs),
}

#[derive(Args)]
struct ServeArgs {
    #[arg(long, env = "SIFT_HOST", default_value = "0.0.0.0")]
    host: String,
    #[arg(long, env = "SIFT_PORT", default_value_t = 7380)]
    port: u16,
    #[arg(long, env = "SIFT_DATA_DIR", default_value = "sift-data")]
    data_dir: PathBuf,
    #[arg(long, env = "SIFT_LOG_LEVEL", default_value = "info")]
    log_level: String,
    #[arg(long, env = "SIFT_LOG_FORMAT", value_enum, default_value_t = LogFormat::Json)]
    log_format: LogFormat,
    #[arg(long, env = "SIFT_GRACE_SECS", default_value_t = 20)]
    grace_secs: u64,
    #[arg(long, env = "SIFT_MAX_BODY_BYTES", default_value_t = 1_048_576)]
    max_body_bytes: usize,
    #[arg(long, env = "SIFT_OTLP_ENDPOINT")]
    otlp_endpoint: Option<String>,
}

#[derive(Args)]
struct CollectArgs {
    /// JSONL source path, or `-` for stdin.
    #[arg(
        long,
        conflicts_with = "cri_root",
        required_unless_present = "cri_root"
    )]
    source: Option<String>,
    /// Kubernetes node CRI pod-log root, normally /var/log/pods.
    #[arg(long, conflicts_with = "source", required_unless_present = "source")]
    cri_root: Option<PathBuf>,
    /// Stable identity used with byte offsets to derive idempotency keys.
    #[arg(long)]
    source_id: Option<String>,
    /// Sift service base URL.
    #[arg(long, env = "SIFT_URL", default_value = "http://127.0.0.1:7380")]
    endpoint: String,
    /// Optional Sift bearer token.
    #[arg(long, env = "SIFT_TOKEN")]
    token: Option<String>,
    #[arg(long, default_value = "default")]
    project: String,
    #[arg(long, default_value = "local")]
    environment: String,
    /// GCP project used for k8s_container monitored-resource identity.
    #[arg(long, env = "GCP_PROJECT_ID")]
    gcp_project: Option<String>,
    #[arg(long, env = "GKE_CLUSTER_NAME")]
    cluster: Option<String>,
    #[arg(long, env = "GKE_LOCATION")]
    location: Option<String>,
    #[arg(long, env = "NODE_NAME")]
    node: Option<String>,
    /// Durable source offset checkpoint; defaults beside the source file.
    #[arg(long)]
    checkpoint: Option<PathBuf>,
    /// Invalid-line JSONL sink; defaults beside the checkpoint.
    #[arg(long)]
    quarantine: Option<PathBuf>,
    #[arg(long, default_value_t = DEFAULT_BATCH_SIZE)]
    batch_size: usize,
    #[arg(long, default_value_t = DEFAULT_MAX_LINE_BYTES)]
    max_line_bytes: usize,
    #[arg(long, default_value_t = DEFAULT_MAX_RETRIES)]
    max_retries: usize,
    #[arg(long, default_value_t = 10)]
    request_timeout_secs: u64,
    /// Continue watching a regular file or refreshing CRI discovery after EOF.
    #[arg(long)]
    follow: bool,
    #[arg(long, default_value_t = 250)]
    follow_poll_ms: u64,
}

#[derive(Clone, Copy, ValueEnum)]
enum LogFormat {
    Pretty,
    Json,
}

#[derive(Args)]
struct EventArgs {
    #[command(subcommand)]
    command: EventCommand,
}

#[derive(Subcommand)]
enum EventCommand {
    /// Append one OperationalEventV2 JSON document.
    Write(EventFileArgs),
    /// Import a bounded JSON array or `{ "events": [...] }` batch.
    Import(EventImportArgs),
}

#[derive(Args)]
struct EventFileArgs {
    /// JSON file containing one EventEnvelope.
    file: PathBuf,
    #[arg(long, env = "SIFT_DATA_DIR", default_value = "sift-data")]
    data_dir: PathBuf,
}

#[derive(Args)]
struct EventImportArgs {
    /// JSON file containing an event array or EventWriteRequest.
    file: PathBuf,
    #[arg(long, default_value = "default")]
    project: String,
    #[arg(long, env = "SIFT_DATA_DIR", default_value = "sift-data")]
    data_dir: PathBuf,
}

#[derive(Args)]
struct QueryArgs {
    #[arg(long, env = "SIFT_DATA_DIR", default_value = "sift-data")]
    data_dir: PathBuf,
    #[arg(long, value_enum)]
    signal: Option<SignalKind>,
    #[arg(long, default_value_t = 100)]
    limit: usize,
    #[arg(long, default_value_t = 0)]
    after: u64,
}

#[derive(Args)]
struct ReplayArgs {
    #[arg(long, env = "SIFT_DATA_DIR", default_value = "sift-data")]
    data_dir: PathBuf,
    #[arg(long, default_value_t = 0)]
    after: u64,
    #[arg(long, default_value_t = 100)]
    limit: usize,
}

#[derive(Args)]
struct SnapshotArgs {
    #[arg(long, env = "SIFT_DATA_DIR", default_value = "sift-data")]
    data_dir: PathBuf,
    /// Write the raw snapshot bytes here; omit to emit them in a JSON terminal envelope.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Args)]
struct RestoreArgs {
    #[arg(long, env = "SIFT_DATA_DIR", default_value = "sift-data")]
    data_dir: PathBuf,
    /// Source URI accepted by service-backup, for example file:///backup/sift.json.
    #[arg(long)]
    source: String,
}

#[derive(Args)]
struct BackupArgs {
    /// Running Sift base URL. Fetches the protected /admin/backup snapshot.
    #[arg(
        long,
        env = "SIFT_BACKUP_URL",
        conflicts_with = "data_dir",
        required_unless_present = "data_dir"
    )]
    url: Option<String>,
    /// Legacy offline mode: open this stopped journal directly.
    #[arg(long, conflicts_with = "url", required_unless_present = "url")]
    data_dir: Option<PathBuf>,
    /// Admin bearer token for live mode. Invalid with offline --data-dir.
    #[arg(long, env = "SIFT_BACKUP_TOKEN", requires = "url")]
    token: Option<String>,
    /// Shared backup destination URI: file://, s3://, or gs://.
    #[arg(long)]
    dest: String,
    /// Remove backup objects older than this many seconds after a successful write.
    #[arg(long)]
    retention_secs: Option<u64>,
}

#[derive(Args)]
struct DockerfileArgs {
    #[command(subcommand)]
    command: DockerfileCommand,
}

#[derive(Subcommand)]
enum DockerfileCommand {
    /// Render a source-build or release-binary Dockerfile.
    Render(DockerfileRenderArgs),
}

#[derive(Args)]
struct DockerfileRenderArgs {
    #[arg(long, value_enum, default_value_t = DockerfileVariantArg::Release)]
    variant: DockerfileVariantArg,
    #[arg(long)]
    version: Option<String>,
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Clone, Copy, ValueEnum)]
enum DockerfileVariantArg {
    Source,
    Release,
}

#[derive(Args)]
struct K8sArgs {
    #[command(subcommand)]
    command: K8sCommand,
}

#[derive(Subcommand)]
enum K8sCommand {
    /// Render the cluster-scoped Sift custom resource definition.
    Crd(K8sCrdArgs),
    /// Render or run the Sift controller control plane.
    Operator(K8sOperatorArgs),
    /// Render one namespaced Sift custom resource.
    Instance(K8sInstanceArgs),
    /// Render the Sift-owned node CRI collector DaemonSet.
    Collector(K8sCollectorArgs),
}

#[derive(Args)]
struct K8sCollectorArgs {
    #[command(subcommand)]
    command: K8sCollectorCommand,
}

#[derive(Subcommand)]
enum K8sCollectorCommand {
    Render(K8sCollectorRenderArgs),
}

#[derive(Args)]
struct K8sCollectorRenderArgs {
    #[arg(long, default_value = "sift-system")]
    namespace: String,
    #[arg(long, default_value = "ghcr.io/chrischeng-c4/axiom/sift:0.1.0")]
    image: String,
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Args)]
struct K8sCrdArgs {
    #[command(subcommand)]
    command: K8sCrdCommand,
}

#[derive(Subcommand)]
enum K8sCrdCommand {
    Render(K8sFileOutputArgs),
}

#[derive(Args)]
struct K8sOperatorArgs {
    #[command(subcommand)]
    command: K8sOperatorCommand,
}

#[derive(Subcommand)]
enum K8sOperatorCommand {
    /// Render service account, RBAC, and controller deployment assets.
    Render(K8sOperatorRenderArgs),
    /// Controller image entrypoint. The deployed image runs this command.
    Run,
}

#[derive(Args)]
struct K8sOperatorRenderArgs {
    #[arg(long, default_value = "sift-system")]
    namespace: String,
    #[arg(long, default_value = sift::deploy::DEFAULT_OPERATOR_IMAGE)]
    image: String,
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Args)]
struct K8sInstanceArgs {
    #[command(subcommand)]
    command: K8sInstanceCommand,
}

#[derive(Subcommand)]
enum K8sInstanceCommand {
    Render(K8sInstanceRenderArgs),
}

#[derive(Args)]
struct K8sInstanceRenderArgs {
    #[arg(long, value_enum, default_value_t = K8sInstanceProfileArg::Dev)]
    profile: K8sInstanceProfileArg,
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Clone, Copy, ValueEnum)]
enum K8sInstanceProfileArg {
    Dev,
    Staging,
    Prod,
    Template,
}

#[derive(Args)]
struct K8sFileOutputArgs {
    #[arg(long)]
    out: Option<PathBuf>,
}

/// Manage a `kubectl port-forward` around a wrapped command so callers do not
/// have to track the child process or hand-resolve the token registry secret.
#[derive(Args)]
struct ConnectArgs {
    /// kubectl context to port-forward through; omit to use the current context.
    #[arg(long)]
    context: Option<String>,
    /// Namespace of the target Service or Sift custom resource.
    #[arg(long)]
    namespace: String,
    /// Target Service name; defaults to `--cr` when a custom resource is named.
    #[arg(long)]
    service: Option<String>,
    /// Sift custom-resource name used to discover the target Service and token Secret.
    #[arg(long)]
    cr: Option<String>,
    /// Local port to forward to; omit to allocate an ephemeral port.
    #[arg(long)]
    local_port: Option<u16>,
    /// Remote Service port.
    #[arg(long, default_value_t = 7380)]
    remote_port: u16,
    /// Token-registry Secret name. Auto-discovered from `--cr` when omitted.
    #[arg(long)]
    secret: Option<String>,
    /// Explicit bearer token; otherwise one is selected from the token registry.
    #[arg(long, env = "SIFT_TOKEN")]
    token: Option<String>,
    /// Minimum role required of a token selected from the registry.
    #[arg(long, value_enum, default_value_t = TokenRole::Admin)]
    role: TokenRole,
    /// Optional resource scope used while selecting a registry token.
    #[arg(long)]
    resource: Option<String>,
    /// Command to run with `SIFT_URL` and, when available, `SIFT_TOKEN` set.
    #[arg(last = true, required = true)]
    command: Vec<String>,
}

/// Service-owned mapping to the generic `cli-std` token-role hierarchy.
#[derive(Clone, Copy, ValueEnum)]
enum TokenRole {
    Read,
    Write,
    Admin,
}

impl From<TokenRole> for cli_std::connect::Role {
    fn from(role: TokenRole) -> Self {
        match role {
            TokenRole::Read => Self::Read,
            TokenRole::Write => Self::Write,
            TokenRole::Admin => Self::Admin,
        }
    }
}

#[derive(Args)]
struct SpecArgs {
    /// Generate a typed API client rather than print the OpenAPI document.
    #[command(subcommand)]
    command: Option<SpecCommand>,
    #[arg(long, default_value = "openapi-json", value_parser = ["openapi-json"])]
    format: String,
}

#[derive(Subcommand)]
enum SpecCommand {
    /// Generate a typed client from Sift's own OpenAPI document.
    Gen(GenArgs),
}

#[derive(Args)]
struct GenArgs {
    /// Target language for the generated client.
    #[arg(long, value_enum)]
    lang: GenLang,
    /// Directory that receives the generated client files.
    #[arg(long)]
    out: PathBuf,
}

#[derive(Clone, Copy, ValueEnum)]
enum GenLang {
    /// TypeScript types and fetch client.
    Ts,
    /// Python Pydantic models and HTTP/2 client.
    Py,
    /// Rust serde models and reqwest client.
    Rust,
}

#[derive(Args)]
struct LlmArgs {
    #[arg(long, default_value = "outline")]
    topic: String,
    #[arg(long, default_value = "md", value_parser = ["md", "json"])]
    format: String,
}

#[derive(Args)]
struct UpgradeArgs {
    #[arg(long)]
    check: bool,
    #[arg(long = "version")]
    tag: Option<String>,
    #[arg(long)]
    force: bool,
    #[arg(short = 'y', long)]
    yes: bool,
}

#[derive(Args)]
struct IssueArgs {
    #[command(subcommand)]
    command: IssueCommand,
}

#[derive(Subcommand)]
enum IssueCommand {
    Search(IssueSearchArgs),
    View(IssueViewArgs),
    Create(IssueCreateArgs),
    Comment(IssueCommentArgs),
}

#[derive(Args)]
struct IssueSearchArgs {
    #[arg(value_name = "QUERY", num_args = 0..)]
    query: Vec<String>,
    #[arg(long, default_value = "open", value_parser = ["open", "closed", "all"])]
    state: String,
    #[arg(long, default_value_t = 20)]
    limit: u32,
}

#[derive(Args)]
struct IssueViewArgs {
    number: u64,
}

#[derive(Args)]
struct IssueCreateArgs {
    #[arg(short = 't', long)]
    title: Option<String>,
    #[arg(value_name = "MESSAGE", num_args = 0..)]
    message: Vec<String>,
    #[arg(long)]
    dry_run: bool,
    #[arg(short = 'y', long)]
    yes: bool,
}

#[derive(Args)]
struct IssueCommentArgs {
    number: u64,
    #[arg(value_name = "MESSAGE", num_args = 0..)]
    message: Vec<String>,
    #[arg(long)]
    dry_run: bool,
    #[arg(short = 'y', long)]
    yes: bool,
}

const TOOL: cli_std::ToolInfo = cli_std::ToolInfo {
    project: "sift",
    repo: "chrischeng-c4/axiom",
    target: env!("SIFT_TARGET"),
    version: env!("CARGO_PKG_VERSION"),
    git_sha: env!("SIFT_GIT_SHA"),
    built_at: env!("SIFT_BUILT_AT"),
};

const LLM_TOPICS: &[cli_std::llm::Topic] = &[
    cli_std::llm::Topic {
        id: "ingest",
        summary: "versioned six-signal ingest and the fsync acknowledgement boundary",
        body: "# Sift ingest\n\nUse `sift collect --source <service.stdout.jsonl>` for file capture, `--source -` for stdin, or `--cri-root /var/log/pods --gcp-project <id>` for the Sift-owned Kubernetes CRI adapter. File, stdin, and CRI records feed the same checkpointed `axiom.service.log.v1` decoder/batch/retry core; `--follow` supports regular files and CRI discovery. Render the least-privilege node deployment with `sift k8s collector render`. Canonical event clients use `sift event write <file>` or `sift event import <file>`; HTTP collectors use `/v1/events:write` or OTLP `/v1/logs`, `/v1/traces`, `/v1/metrics`, and `/v1/profiles`. Accepted items have completed the shared durable append path.",
    },
    cli_std::llm::Topic {
        id: "operations",
        summary: "h2c serving, probe routes, query, replay, and local CLI use",
        body: "# Sift operations\n\nRun `sift serve --data-dir ./sift-data`. The process serves HTTP/1.1 and h2c on one port plus `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, and `/docs`. Use `sift event write|import`, `sift query`, and `sift replay` for local durable-journal inspection. Scheduled backups use `sift backup --url <service> --dest <uri>` and may supply `--token`/`SIFT_BACKUP_TOKEN`; legacy `--data-dir` backup is an explicit offline-only mode for a stopped journal and cannot be combined with `--url`.",
    },
];

#[tokio::main]
async fn main() -> Result<()> {
    // The operator and online CLI pull both ring and aws-lc-rs through the
    // shared transport stack. Pick the ecosystem-wide provider before any
    // TLS client (notably kube) initializes it.
    peer_tls::install_default_crypto_provider();
    match Cli::parse().command {
        Command::Serve(args) => serve(args).await,
        Command::Collect(args) => collect(args).await,
        Command::Event(args) => append_event(args),
        Command::Query(args) => query(args),
        Command::Replay(args) => replay(args),
        Command::Snapshot(args) => snapshot(args),
        Command::Restore(args) => restore(args),
        Command::Backup(args) => backup(args).await,
        Command::Dockerfile(args) => dockerfile(args),
        Command::K8s(args) => k8s(args).await,
        Command::Connect(args) => connect(args).await,
        Command::Spec(args) => match args.command {
            Some(SpecCommand::Gen(args)) => spec_gen(args),
            None => {
                let _ = args.format;
                print_json_terminal(serde_json::json!({
                    "openapi": serde_json::from_str::<Value>(&sift::openapi_json()?)?
                }))
            }
        },
        Command::Llm(args) => {
            let output = cli_std::llm::render(
                "sift",
                env!("CARGO_PKG_VERSION"),
                LLM_TOPICS,
                &args.topic,
                cli_std::llm::Format::parse(&args.format),
            )?;
            if args.format == "json" {
                print_json_text_terminal(&output)
            } else {
                println!("{output}");
                println!("next: done");
                Ok(())
            }
        }
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
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn installs_rustls_provider_before_kubernetes_cli_initializes_tls() {
        peer_tls::install_default_crypto_provider();
        assert!(rustls::crypto::CryptoProvider::get_default().is_some());
    }
}

// <HANDWRITE gap="missing-generator:logic" tracker="1873" reason="Implement the Sift-owned local structured-stdout collector CLI seam.">
async fn collect(args: CollectArgs) -> Result<()> {
    let (source, default_source_id, default_checkpoint) = if let Some(root) = args.cri_root {
        let canonical = std::fs::canonicalize(&root)
            .with_context(|| format!("resolve CRI root {}", root.display()))?;
        let gcp_project = args
            .gcp_project
            .context("--gcp-project or GCP_PROJECT_ID is required with --cri-root")?;
        let source_id = args
            .node
            .as_deref()
            .map(|node| format!("cri-node:{node}"))
            .unwrap_or_else(|| format!("cri-root:{}", canonical.display()));
        (
            SourceSpec::Cri(CriSourceConfig {
                root: canonical,
                metadata: CriMetadata {
                    gcp_project,
                    cluster: args.cluster,
                    location: args.location,
                    node: args.node,
                },
            }),
            source_id,
            PathBuf::from("sift-cri.checkpoint.json"),
        )
    } else if args.source.as_deref() == Some("-") {
        (
            SourceSpec::Stdin,
            "stdin".to_string(),
            PathBuf::from("sift-stdin.checkpoint.json"),
        )
    } else {
        let source_arg = args.source.context("--source or --cri-root is required")?;
        let path = PathBuf::from(&source_arg);
        let canonical = std::fs::canonicalize(&path)
            .with_context(|| format!("resolve collector source {}", path.display()))?;
        let source_id = format!("file:{}", canonical.display());
        let checkpoint = PathBuf::from(format!("{}.sift-checkpoint.json", path.display()));
        (SourceSpec::File(path), source_id, checkpoint)
    };
    let checkpoint_path = args.checkpoint.unwrap_or(default_checkpoint);
    let quarantine_path = args
        .quarantine
        .unwrap_or_else(|| PathBuf::from(format!("{}.rejected.jsonl", checkpoint_path.display())));
    let summary = sift::collector::run_collector(CollectorConfig {
        source,
        source_id: args.source_id.unwrap_or(default_source_id),
        endpoint: args.endpoint,
        token: args.token,
        project: args.project,
        environment: args.environment,
        checkpoint_path,
        quarantine_path,
        batch_size: args.batch_size,
        max_line_bytes: args.max_line_bytes,
        max_retries: args.max_retries,
        request_timeout: Duration::from_secs(args.request_timeout_secs),
        initial_backoff: Duration::from_millis(50),
        follow: args.follow,
        follow_poll_interval: Duration::from_millis(args.follow_poll_ms),
    })
    .await?;
    print_json_terminal(summary)
}
// </HANDWRITE>

async fn serve(args: ServeArgs) -> Result<()> {
    let format = match args.log_format {
        LogFormat::Pretty => service_http::LogFormat::Pretty,
        LogFormat::Json => service_http::LogFormat::Json,
    };
    let config = service_http::HttpConfig::new(
        args.host,
        args.port,
        args.log_level,
        format,
        args.grace_secs,
        args.max_body_bytes,
        args.otlp_endpoint,
    );
    service_http::init_tracing(&config)?;

    let state = Arc::new(ServiceState::open(&args.data_dir)?);
    let projection_worker = state.start_projection_worker();
    let verifier = Arc::new(SiftVerifier::from_env()?);
    let data_plane = sift::protected_router(state.clone(), verifier);
    let data_plane = match state.raft_router() {
        Some(raft_routes) => data_plane.merge(raft_routes),
        None => data_plane,
    }
    .layer(DefaultBodyLimit::max(config.body_limit_bytes));
    let app =
        service_http::standard_probe_routes(state.clone(), Some(state.clone()), sift::openapi)
            .merge(data_plane)
            .layer(service_http::trace_layer());
    let listener = tokio::net::TcpListener::bind(config.bind_addr())
        .await
        .context("bind Sift service listener")?;
    tracing::info!(address = %config.bind_addr(), "sift serving HTTP/1.1 and h2c");
    let grace = Duration::from_secs(config.grace_secs);
    service_http::serve(
        listener,
        app,
        service_http::shutdown_with_drain(move || state.start_drain(), grace),
    )
    .await;
    projection_worker.stop().await;
    Ok(())
}

fn append_event(args: EventArgs) -> Result<()> {
    match args.command {
        EventCommand::Write(args) => {
            let source = std::fs::read_to_string(&args.file)
                .with_context(|| format!("read event file {}", args.file.display()))?;
            let event =
                decode_event_json(source.as_bytes()).context("parse operational event JSON")?;
            let result = DurableJournal::open(&args.data_dir)?.append(event)?;
            print_json_terminal(result)
        }
        EventCommand::Import(args) => import_events(args),
    }
}

fn import_events(args: EventImportArgs) -> Result<()> {
    let source = std::fs::read(&args.file)
        .with_context(|| format!("read event batch {}", args.file.display()))?;
    let value: Value = serde_json::from_slice(&source).context("parse event batch JSON")?;
    let values = match value {
        Value::Array(values) => values,
        Value::Object(mut object) => object
            .remove("events")
            .and_then(|value| value.as_array().cloned())
            .context("event import object must contain an events array")?,
        _ => anyhow::bail!("event import expects a JSON array or object with events"),
    };
    let journal = DurableJournal::open(&args.data_dir)?;
    let mut results = Vec::with_capacity(values.len());
    for (index, value) in values.into_iter().enumerate() {
        let event_id = sift::ingest::batch::event_id_hint(&value);
        match sift::ingest::batch::decode_item(value, &args.project)
            .and_then(|event| journal.append(event))
        {
            Ok(result) => results.push(sift::ingest::BatchItemResult::accepted(
                index,
                result.event_id,
                result.cursor,
                result.duplicate,
            )),
            Err(error) => results.push(sift::ingest::BatchItemResult::rejected(
                index,
                event_id,
                "invalid_event",
                error.to_string(),
                false,
            )),
        }
    }
    print_json_terminal(sift::ingest::EventWriteResponse::from_results(results))
}

fn query(args: QueryArgs) -> Result<()> {
    let rows = DurableJournal::open(&args.data_dir)?.query(EventQuery {
        signal: args.signal,
        after: args.after,
        limit: args.limit,
    })?;
    print_json_terminal(rows)
}

fn replay(args: ReplayArgs) -> Result<()> {
    let rows = DurableJournal::open(&args.data_dir)?.replay(args.after, args.limit)?;
    print_json_terminal(rows)
}

fn snapshot(args: SnapshotArgs) -> Result<()> {
    let bytes = DurableJournal::open(&args.data_dir)?.snapshot_bytes()?;
    if let Some(path) = args.out {
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("create snapshot output directory {}", parent.display())
            })?;
        }
        std::fs::write(&path, &bytes)
            .with_context(|| format!("write Sift snapshot {}", path.display()))?;
        return print_json_terminal(serde_json::json!({
            "path": path,
            "bytes": bytes.len(),
        }));
    }
    print_json_terminal(serde_json::from_slice::<Value>(&bytes)?)
}

fn restore(args: RestoreArgs) -> Result<()> {
    let journal = DurableJournal::open(&args.data_dir)?;
    sift::backup::restore_journal(&journal, &args.source)?;
    print_json_terminal(serde_json::json!({
        "status": "restored",
        "source": args.source,
    }))
}

async fn backup(args: BackupArgs) -> Result<()> {
    let result = match (args.url.as_deref(), args.data_dir.as_deref()) {
        (Some(url), None) => {
            sift::backup::backup_live_journal(
                url,
                args.token.as_deref(),
                &args.dest,
                args.retention_secs,
            )
            .await?
        }
        (None, Some(data_dir)) => {
            let journal = DurableJournal::open(data_dir)?;
            sift::backup::backup_journal(&journal, &args.dest, args.retention_secs)?
        }
        _ => anyhow::bail!(
            "choose exactly one backup source: live --url or legacy offline --data-dir"
        ),
    };
    print_json_terminal(result)
}

fn dockerfile(args: DockerfileArgs) -> Result<()> {
    match args.command {
        DockerfileCommand::Render(args) => {
            let variant = match args.variant {
                DockerfileVariantArg::Source => DockerfileVariant::Source,
                DockerfileVariantArg::Release => DockerfileVariant::Release,
            };
            let body = sift::deploy::dockerfile(variant, args.version.as_deref())?;
            let file_name = match variant {
                DockerfileVariant::Source => "Dockerfile",
                DockerfileVariant::Release => "Dockerfile.release",
            };
            write_artifact(
                args.out.as_deref(),
                file_name,
                &body,
                "docker build -f projects/sift/Dockerfile -t sift:dev .",
            )
        }
    }
}

async fn k8s(args: K8sArgs) -> Result<()> {
    match args.command {
        K8sCommand::Crd(args) => match args.command {
            K8sCrdCommand::Render(args) => write_artifact(
                args.out.as_deref(),
                "sift-crd.yaml",
                &sift::deploy::crd_yaml(),
                "kubectl apply -f -",
            ),
        },
        K8sCommand::Operator(args) => match args.command {
            K8sOperatorCommand::Render(args) => write_artifact(
                args.out.as_deref(),
                "sift-operator.yaml",
                &sift::deploy::operator_yaml_with_image(&args.namespace, &args.image)?,
                "kubectl apply -f -",
            ),
            K8sOperatorCommand::Run => operator_run().await,
        },
        K8sCommand::Instance(args) => match args.command {
            K8sInstanceCommand::Render(args) => {
                let profile = match args.profile {
                    K8sInstanceProfileArg::Dev => InstanceProfile::Dev,
                    K8sInstanceProfileArg::Staging => InstanceProfile::Staging,
                    K8sInstanceProfileArg::Prod => InstanceProfile::Prod,
                    K8sInstanceProfileArg::Template => InstanceProfile::Template,
                };
                write_artifact(
                    args.out.as_deref(),
                    "sift.yaml",
                    &sift::deploy::instance_yaml(profile),
                    "kubectl apply -f -",
                )
            }
        },
        K8sCommand::Collector(args) => match args.command {
            K8sCollectorCommand::Render(args) => write_artifact(
                args.out.as_deref(),
                "sift-collector.yaml",
                &sift::deploy::collector_yaml(&args.namespace, &args.image)?,
                "kubectl apply -f -",
            ),
        },
    }
}

async fn operator_run() -> Result<()> {
    sift::operator::run().await
}

/// Shared k8s-native connection lifecycle: resolve the Service and optional
/// token registry, wait for a ready local port-forward, run the wrapped
/// command, then let `ChildGuard` terminate and reap kubectl on every exit.
async fn connect(args: ConnectArgs) -> Result<()> {
    let service = args
        .service
        .clone()
        .or_else(|| args.cr.clone())
        .context("--service or --cr is required")?;
    let secret = match args.secret.clone() {
        Some(secret) => Some(secret),
        None => match &args.cr {
            Some(cr) => cli_std::connect::resolve_cr_tokens_secret(
                args.context.as_deref(),
                &args.namespace,
                "sift",
                cr,
            )?,
            None => None,
        },
    };
    let local_port = args
        .local_port
        .map(Ok)
        .unwrap_or_else(cli_std::connect::free_local_port)?;

    let mut forward = std::process::Command::new("kubectl");
    if let Some(context) = &args.context {
        forward.args(["--context", context]);
    }
    forward.args([
        "port-forward",
        "-n",
        &args.namespace,
        &format!("svc/{service}"),
        &format!("{local_port}:{}", args.remote_port),
    ]);
    forward.stdout(std::process::Stdio::null());
    forward.stderr(std::process::Stdio::null());
    let _forward =
        cli_std::connect::ChildGuard::spawn(&mut forward).context("start kubectl port-forward")?;
    cli_std::connect::wait_for_local_port_ready(local_port, Duration::from_secs(30))?;

    let token = cli_std::connect::resolve_token(
        args.token.as_deref(),
        args.context.as_deref(),
        Some(&args.namespace),
        secret.as_deref(),
        args.role.into(),
        args.resource.as_deref(),
    )?;
    let (program, rest) = args
        .command
        .split_first()
        .context("wrapped command is empty")?;
    let mut command = std::process::Command::new(program);
    command.args(rest);
    command.env("SIFT_URL", format!("http://127.0.0.1:{local_port}"));
    if let Some(token) = token {
        command.env("SIFT_TOKEN", token);
    }
    let status = command.status().context("run wrapped command")?;
    if !status.success() {
        anyhow::bail!("wrapped command exited with {status}");
    }
    Ok(())
}

fn write_artifact(
    out: Option<&std::path::Path>,
    file_name: &str,
    body: &str,
    next: &str,
) -> Result<()> {
    match out {
        None => {
            print!("{body}");
            println!("next: {next}");
        }
        Some(out) => {
            let path = if out.is_dir() {
                std::fs::create_dir_all(out)
                    .with_context(|| format!("create artifact directory {}", out.display()))?;
                out.join(file_name)
            } else {
                if let Some(parent) = out.parent().filter(|parent| !parent.as_os_str().is_empty()) {
                    std::fs::create_dir_all(parent).with_context(|| {
                        format!("create artifact output directory {}", parent.display())
                    })?;
                }
                out.to_path_buf()
            };
            std::fs::write(&path, body)
                .with_context(|| format!("write deployment artifact {}", path.display()))?;
            println!("wrote {}", path.display());
            println!("next: kubectl apply -f {}", path.display());
        }
    }
    Ok(())
}

fn spec_gen(args: GenArgs) -> Result<()> {
    use cclab_openapi_codegen::{generate, GenOptions, HttpClient, Lang};

    let lang = match args.lang {
        GenLang::Ts => Lang::Ts,
        GenLang::Py => Lang::Py,
        GenLang::Rust => Lang::Rust,
    };
    let output = generate(
        &sift::openapi_json()?,
        &GenOptions {
            lang,
            spec_path: PathBuf::new(),
            out_dir: args.out.clone(),
            client_name: "createClient".to_string(),
            http_client: HttpClient::Fetch,
            emit_types: true,
            emit_client: true,
            emit_hooks: matches!(lang, Lang::Ts),
        },
    )?;
    std::fs::create_dir_all(&args.out)
        .with_context(|| format!("create generated client directory {}", args.out.display()))?;
    for file in output.files {
        let path = args.out.join(file.rel_path);
        std::fs::write(&path, file.contents)
            .with_context(|| format!("write generated client file {}", path.display()))?;
        println!("generated {}", path.display());
    }
    let entrypoint = match lang {
        Lang::Ts => "index.ts",
        Lang::Py => "__init__.py",
        Lang::Rust => "mod.rs",
    };
    println!("next: {}", args.out.join(entrypoint).display());
    Ok(())
}

fn print_json_text_terminal(text: &str) -> Result<()> {
    let value: Value = serde_json::from_str(text).context("parse machine-readable CLI output")?;
    print_json_terminal(value)
}

fn print_json_terminal(value: impl Serialize) -> Result<()> {
    let value = serde_json::to_value(value)?;
    let output = match value {
        Value::Object(mut object) => {
            object.insert("next".to_string(), Value::String("done".to_string()));
            Value::Object(object)
        }
        value => serde_json::json!({ "result": value, "next": "done" }),
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

async fn issue(args: IssueArgs) -> Result<()> {
    match args.command {
        IssueCommand::Search(args) => {
            cli_std::issue::search(
                &TOOL,
                cli_std::issue::SearchOptions {
                    query: (!args.query.is_empty()).then(|| args.query.join(" ")),
                    state: args.state,
                    limit: args.limit,
                },
            )
            .await
        }
        IssueCommand::View(args) => cli_std::issue::view(&TOOL, args.number).await,
        IssueCommand::Create(args) => {
            let message = (!args.message.is_empty()).then(|| args.message.join(" "));
            let title = args
                .title
                .unwrap_or_else(|| "sift: issue report".to_string());
            cli_std::issue::create(
                &TOOL,
                cli_std::issue::CreateOptions {
                    title,
                    message,
                    label: vec!["app:sift".to_string()],
                    dry_run: args.dry_run,
                    yes: args.yes,
                    ..Default::default()
                },
            )
            .await
        }
        IssueCommand::Comment(args) => {
            cli_std::issue::comment(
                &TOOL,
                cli_std::issue::CommentOptions {
                    number: args.number,
                    message: (!args.message.is_empty()).then(|| args.message.join(" ")),
                    dry_run: args.dry_run,
                    yes: args.yes,
                    ..Default::default()
                },
            )
            .await
        }
    }
}
// HANDWRITE-END
