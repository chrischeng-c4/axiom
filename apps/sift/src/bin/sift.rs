// HANDWRITE-BEGIN gap="sift-service-cli" tracker="1576" reason="Implement serve, event, query, replay, spec, llm, upgrade, and issue CLI surfaces."
use std::{
    io::{Read, Write},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use anyhow::{Context, Result};
use axum::extract::DefaultBodyLimit;
use base64::Engine as _;
use clap::{Args, Parser, Subcommand, ValueEnum};
use prost::Message as _;
use prost14::Message as _;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use sift::{
    auth::SiftVerifier,
    collector::{
        CollectorConfig, CriMetadata, CriSourceConfig, SourceSpec, DEFAULT_BATCH_SIZE,
        DEFAULT_MAX_LINE_BYTES, DEFAULT_MAX_RETRIES,
    },
    deploy::{DockerfileVariant, InstanceProfile},
    DurableJournal, ServiceState,
};

#[derive(Parser)]
#[command(
    name = "sift",
    version,
    about = "Sift — one SRE product for logs, metrics, and traces"
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
    /// Run one versioned logs, metrics, or traces query through the Sift API.
    Query(QueryArgs),
    /// Serve the read-only Sift MCP tools.
    Mcp(McpArgs),
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
    /// Emit deterministic protocol bytes for the isolated acceptance runner.
    #[command(hide = true)]
    AcceptancePayload(AcceptancePayloadArgs),
    /// Send one valid and one invalid log through OTLP/gRPC for acceptance.
    #[command(hide = true)]
    AcceptanceGrpc(AcceptanceGrpcArgs),
}

#[derive(Args)]
struct AcceptancePayloadArgs {
    #[arg(long, value_enum)]
    kind: AcceptancePayloadKind,
    #[arg(long, default_value_t = 1)]
    items: usize,
    #[arg(long, default_value = "sift")]
    project: String,
    #[arg(long, default_value = "acceptance")]
    event_prefix: String,
    #[arg(long)]
    timestamp_unix_nano: u64,
}

#[derive(Clone, Copy, ValueEnum)]
enum AcceptancePayloadKind {
    OtlpLogsProtobuf,
    PrometheusRemoteWriteV1,
}

#[derive(Args)]
struct AcceptanceGrpcArgs {
    #[arg(long)]
    endpoint: String,
    #[arg(long, default_value = "sift")]
    project: String,
    #[arg(long)]
    token_file: Option<PathBuf>,
}

#[derive(Args)]
struct ServeArgs {
    /// Internal Sift deployment role. All roles use the same product binary.
    #[arg(long, value_enum, default_value = "all")]
    role: RunRole,
    #[arg(long, env = "SIFT_HOST", default_value = "0.0.0.0")]
    host: String,
    #[arg(long, env = "SIFT_PORT", default_value_t = 7380)]
    port: u16,
    /// OTLP/gRPC listener port. It defaults to 4317 for the normal HTTP port.
    #[arg(long, env = "SIFT_GRPC_PORT")]
    grpc_port: Option<u16>,
    #[arg(long, env = "SIFT_DATA_DIR", default_value = sift::storage::DEFAULT_DATA_DIR)]
    data_dir: PathBuf,
    /// Development-only temporary storage. Production roles refuse this flag.
    #[arg(long, conflicts_with = "data_dir")]
    ephemeral: bool,
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
    #[arg(long, env = "SIFT_TOKEN", conflicts_with = "token_file")]
    token: Option<String>,
    /// Rotating projected ServiceAccount token. The file is read for every request.
    #[arg(long, env = "SIFT_TOKEN_FILE", conflicts_with = "token")]
    token_file: Option<PathBuf>,
    /// Required audience in a projected ServiceAccount token.
    #[arg(long, env = "SIFT_TOKEN_AUDIENCE", default_value = "sift.axiom.dev")]
    token_audience: String,
    /// Persistent Sift root used for agent checkpoints and rejected records.
    #[arg(long, env = "SIFT_DATA_DIR", default_value = sift::storage::DEFAULT_DATA_DIR)]
    data_dir: PathBuf,
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
    /// Durable source offset checkpoint; defaults under DATA_DIR/agent.
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

#[derive(Clone, Copy, Eq, PartialEq, ValueEnum)]
enum RunRole {
    All,
    Agent,
    Gateway,
    Query,
    Store,
    Control,
    Operator,
}

impl From<RunRole> for sift::storage::StorageRole {
    fn from(role: RunRole) -> Self {
        match role {
            RunRole::All => Self::All,
            RunRole::Agent => Self::Agent,
            RunRole::Gateway => Self::Gateway,
            RunRole::Query => Self::Query,
            RunRole::Store => Self::Store,
            RunRole::Control => Self::Control,
            RunRole::Operator => Self::Operator,
        }
    }
}

#[derive(Args)]
struct QueryArgs {
    /// QueryRequestV1 JSON file, or `-` for stdin.
    #[arg(value_name = "REQUEST")]
    request: String,
    /// Sift service base URL.
    #[arg(long, env = "SIFT_URL", default_value = "http://127.0.0.1:7380")]
    endpoint: String,
    /// Optional Sift bearer token.
    #[arg(long, env = "SIFT_TOKEN")]
    token: Option<String>,
    #[arg(long, default_value_t = 30)]
    request_timeout_secs: u64,
}

#[derive(Args)]
struct McpArgs {
    #[command(subcommand)]
    command: McpCommand,
}

#[derive(Subcommand)]
enum McpCommand {
    /// Serve Sift tools over standard input and output.
    Serve(McpServeArgs),
}

#[derive(Args)]
struct McpServeArgs {
    /// Use the MCP standard-input and standard-output transport.
    #[arg(long, default_value_t = true, action = clap::ArgAction::SetTrue)]
    stdio: bool,
    /// Sift service base URL used by the MCP tools.
    #[arg(long, env = "SIFT_URL", default_value = "http://127.0.0.1:7380")]
    endpoint: String,
    /// Optional Sift bearer token.
    #[arg(long, env = "SIFT_TOKEN")]
    token: Option<String>,
}

#[derive(Args)]
struct SnapshotArgs {
    #[arg(long, env = "SIFT_DATA_DIR", default_value = sift::storage::DEFAULT_DATA_DIR)]
    data_dir: PathBuf,
    /// Write the raw snapshot bytes here; omit to emit them in a JSON terminal envelope.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Args)]
struct RestoreArgs {
    #[arg(long, env = "SIFT_DATA_DIR", default_value = sift::storage::DEFAULT_DATA_DIR)]
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
    #[arg(
        long,
        env = "SIFT_BACKUP_TOKEN",
        requires = "url",
        conflicts_with = "token_file"
    )]
    token: Option<String>,
    /// Rotating projected ServiceAccount token for live mode.
    #[arg(
        long,
        env = "SIFT_TOKEN_FILE",
        requires = "url",
        conflicts_with = "token"
    )]
    token_file: Option<PathBuf>,
    #[arg(long, env = "SIFT_TOKEN_AUDIENCE", default_value = "sift.axiom.dev")]
    token_audience: String,
    /// Project checked by Kubernetes SubjectAccessReview for live backup.
    #[arg(long, env = "SIFT_PROJECT", default_value = "*")]
    project: String,
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
    #[arg(long, default_value = sift::deploy::DEFAULT_OPERATOR_IMAGE)]
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
        summary: "OTLP logs, metrics, traces, and durable acknowledgement",
        body: "# Sift ingest\n\nUse OTLP/HTTP `/v1/logs`, `/v1/metrics`, and `/v1/traces`, or OTLP/gRPC on port 4317. Metrics clients can also use Prometheus Remote Write 1.0 at `/prometheus/api/v1/write`. Use `sift collect --source <service.stdout.jsonl>` for file capture, `--source -` for stdin, or `--cri-root /var/log/pods --gcp-project <id>` for Kubernetes CRI logs. Collector checkpoints live under `/var/lib/sift/agent` by default. Accepted items have completed the durable Sift append path.",
    },
    cli_std::llm::Topic {
        id: "operations",
        summary: "persistent roles, unified query, MCP, and backup",
        body: "# Sift operations\n\nRun `sift serve` with a writable `/var/lib/sift`, or set `--data-dir`. The process serves HTTP/1.1 and h2c plus OTLP/gRPC. Use `sift query <request.json> --endpoint <url>` for the versioned logs, metrics, or traces query API. Use `sift mcp serve --stdio --endpoint <url>` for the same read-only capabilities through MCP. Scheduled backups use `sift backup --url <service> --dest <uri>` and may supply `--token` or `SIFT_BACKUP_TOKEN`.",
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
        Command::Query(args) => query(args).await,
        Command::Mcp(args) => mcp(args).await,
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
        Command::AcceptancePayload(args) => acceptance_payload(args),
        Command::AcceptanceGrpc(args) => acceptance_grpc(args).await,
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
    let (source, default_source_id) = if let Some(root) = args.cri_root {
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
        )
    } else if args.source.as_deref() == Some("-") {
        (SourceSpec::Stdin, "stdin".to_string())
    } else {
        let source_arg = args.source.context("--source or --cri-root is required")?;
        let path = PathBuf::from(&source_arg);
        let canonical = std::fs::canonicalize(&path)
            .with_context(|| format!("resolve collector source {}", path.display()))?;
        let source_id = format!("file:{}", canonical.display());
        (SourceSpec::File(path), source_id)
    };
    let source_id = args.source_id.unwrap_or(default_source_id);
    let checkpoint_path = args
        .checkpoint
        .unwrap_or_else(|| agent_checkpoint_path(&args.data_dir, &source_id));
    let quarantine_path = args
        .quarantine
        .unwrap_or_else(|| PathBuf::from(format!("{}.rejected.jsonl", checkpoint_path.display())));
    let summary = sift::collector::run_collector(CollectorConfig {
        source,
        source_id,
        endpoint: args.endpoint,
        token: args.token,
        token_file: args.token_file,
        token_audience: args.token_audience,
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

fn agent_checkpoint_path(data_dir: &std::path::Path, source_id: &str) -> PathBuf {
    let digest = Sha256::digest(source_id.as_bytes());
    data_dir
        .join("agent")
        .join(format!("{}.checkpoint.json", hex::encode(&digest[..16])))
}
// </HANDWRITE>

fn acceptance_payload(args: AcceptancePayloadArgs) -> Result<()> {
    if args.items == 0 || args.items > 1_000 {
        anyhow::bail!("--items must be between 1 and 1000");
    }
    if args.project.trim().is_empty() {
        anyhow::bail!("--project must not be empty");
    }
    if args.event_prefix.trim().is_empty() {
        anyhow::bail!("--event-prefix must not be empty");
    }
    let bytes = match args.kind {
        AcceptancePayloadKind::OtlpLogsProtobuf => {
            use sift::ingest::otlp::wire::{
                any_value, AnyValue, ExportLogsServiceRequest, KeyValue, LogRecord, Resource,
                ResourceLogs, ScopeLogs,
            };

            let mut log_records = Vec::with_capacity(args.items);
            for index in 0..args.items {
                let timestamp = args
                    .timestamp_unix_nano
                    .checked_add(index as u64)
                    .context("OTLP fixture timestamp overflow")?;
                log_records.push(LogRecord {
                    time_unix_nano: timestamp,
                    observed_time_unix_nano: timestamp,
                    severity_number: 9,
                    severity_text: "INFO".into(),
                    body: Some(AnyValue {
                        value: Some(any_value::Value::StringValue(format!(
                            "Sift acceptance log {index}"
                        ))),
                    }),
                    attributes: vec![KeyValue {
                        key: "sift.event_id".into(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::StringValue(format!(
                                "{}-{index}",
                                args.event_prefix
                            ))),
                        }),
                        ..Default::default()
                    }],
                    dropped_attributes_count: 0,
                    flags: 0,
                    trace_id: Vec::new(),
                    span_id: Vec::new(),
                    event_name: String::new(),
                });
            }
            ExportLogsServiceRequest {
                resource_logs: vec![ResourceLogs {
                    resource: Some(Resource {
                        attributes: vec![KeyValue {
                            key: "service.name".into(),
                            value: Some(AnyValue {
                                value: Some(any_value::Value::StringValue(
                                    "sift-acceptance".into(),
                                )),
                            }),
                            ..Default::default()
                        }],
                        dropped_attributes_count: 0,
                        entity_refs: Vec::new(),
                    }),
                    scope_logs: vec![ScopeLogs {
                        scope: None,
                        log_records,
                        schema_url: String::new(),
                    }],
                    schema_url: String::new(),
                }],
            }
            .encode_to_vec()
        }
        AcceptancePayloadKind::PrometheusRemoteWriteV1 => {
            use sift::prometheus::remote::{Label, Sample, TimeSeries, WriteRequest};

            let base_millis = i64::try_from(args.timestamp_unix_nano / 1_000_000)
                .context("Prometheus fixture timestamp exceeds i64 milliseconds")?;
            let samples = (0..args.items)
                .map(|index| {
                    let timestamp = base_millis
                        .checked_add(index as i64)
                        .context("Prometheus fixture timestamp overflow")?;
                    Ok(Sample {
                        value: index as f64,
                        timestamp,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            let request = WriteRequest {
                timeseries: vec![TimeSeries {
                    labels: vec![
                        Label {
                            name: "__name__".into(),
                            value: "sift_acceptance_total".into(),
                        },
                        Label {
                            name: "environment".into(),
                            value: "acceptance".into(),
                        },
                        Label {
                            name: "fixture".into(),
                            value: args.event_prefix,
                        },
                        Label {
                            name: "project".into(),
                            value: args.project,
                        },
                    ],
                    samples,
                    exemplars: Vec::new(),
                }],
                metadata: Vec::new(),
            };
            metrics_remote_write::encode_snappy(&request.encode_to_vec())
                .context("compress Prometheus Remote Write fixture")?
        }
    };
    std::io::stdout()
        .lock()
        .write_all(&bytes)
        .context("write acceptance protocol bytes")
}

async fn acceptance_grpc(args: AcceptanceGrpcArgs) -> Result<()> {
    use opentelemetry_proto::tonic::{
        collector::logs::v1::{logs_service_client::LogsServiceClient, ExportLogsServiceRequest},
        common::v1::{any_value, AnyValue, KeyValue},
        logs::v1::{LogRecord, ResourceLogs, ScopeLogs},
        resource::v1::Resource,
    };
    use tonic::{codec::CompressionEncoding, Request};

    if args.project.trim().is_empty() {
        anyhow::bail!("--project must not be empty");
    }
    let timestamp = u64::try_from(chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default())
        .context("current time precedes Unix epoch")?;
    let valid = LogRecord {
        time_unix_nano: timestamp,
        observed_time_unix_nano: timestamp,
        severity_text: "INFO".into(),
        body: Some(AnyValue {
            value: Some(any_value::Value::StringValue(
                "Sift OTLP/gRPC acceptance".into(),
            )),
        }),
        attributes: vec![KeyValue {
            key: "sift.event_id".into(),
            value: Some(AnyValue {
                value: Some(any_value::Value::StringValue(format!(
                    "grpc-acceptance-{timestamp}"
                ))),
            }),
            ..Default::default()
        }],
        ..Default::default()
    };
    let invalid = LogRecord {
        time_unix_nano: timestamp.saturating_add(1),
        observed_time_unix_nano: timestamp.saturating_add(1),
        ..Default::default()
    };
    let request = ExportLogsServiceRequest {
        resource_logs: vec![ResourceLogs {
            resource: Some(Resource {
                attributes: vec![KeyValue {
                    key: "service.name".into(),
                    value: Some(AnyValue {
                        value: Some(any_value::Value::StringValue("sift-acceptance".into())),
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            scope_logs: vec![ScopeLogs {
                scope: None,
                log_records: vec![valid, invalid],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }],
    };
    let mut client = LogsServiceClient::connect(args.endpoint)
        .await
        .context("connect to Sift OTLP/gRPC")?
        .send_compressed(CompressionEncoding::Gzip)
        .accept_compressed(CompressionEncoding::Gzip);
    let mut request = Request::new(request);
    request.metadata_mut().insert(
        "x-sift-project",
        args.project.parse().context("encode project metadata")?,
    );
    if let Some(path) = args.token_file {
        let token = std::fs::read_to_string(&path)
            .with_context(|| format!("read projected token {}", path.display()))?;
        let authorization = format!("Bearer {}", token.trim());
        request.metadata_mut().insert(
            "authorization",
            authorization
                .parse()
                .context("encode authorization metadata")?,
        );
    }
    let response = client
        .export(request)
        .await
        .context("export OTLP/gRPC logs")?
        .into_inner();
    let rejected = response
        .partial_success
        .as_ref()
        .map_or(0, |partial| partial.rejected_log_records);
    if rejected != 1 {
        anyhow::bail!("OTLP/gRPC acceptance expected one rejected log, got {rejected}");
    }
    print_json_terminal(serde_json::json!({
        "signal":"logs",
        "accepted":1,
        "rejected":rejected,
        "compression":"gzip"
    }))
}

async fn serve(args: ServeArgs) -> Result<()> {
    if args.ephemeral && (args.role != RunRole::All || production_environment()) {
        anyhow::bail!(
            "--ephemeral is forbidden for production Sift roles; use writable persistent storage"
        );
    }
    let ephemeral_root = args
        .ephemeral
        .then(|| tempfile::Builder::new().prefix("sift-ephemeral-").tempdir())
        .transpose()
        .context("create explicit ephemeral Sift root")?;
    let data_dir = ephemeral_root
        .as_ref()
        .map(|root| root.path())
        .unwrap_or(args.data_dir.as_path());
    let format = match args.log_format {
        LogFormat::Pretty => service_http::LogFormat::Pretty,
        LogFormat::Json => service_http::LogFormat::Json,
    };
    let config = service_http::HttpConfig::new(
        args.host.clone(),
        args.port,
        args.log_level,
        format,
        args.grace_secs,
        args.max_body_bytes,
        args.otlp_endpoint,
    );
    service_http::init_tracing(&config)?;

    if matches!(args.role, RunRole::All | RunRole::Store) {
        if let Ok(manifest_uri) = std::env::var("SIFT_BOOTSTRAP_ARCHIVE_MANIFEST_URI") {
            if !manifest_uri.trim().is_empty() {
                match sift::storage::archive::bootstrap_gcs_if_needed(&manifest_uri, data_dir)? {
                    Some(manifest) => tracing::info!(
                        source_manifest = manifest_uri,
                        source_cluster_id = manifest.source_cluster_id,
                        event_count = manifest.event_count,
                        "Sift fresh-volume archive bootstrap completed"
                    ),
                    None => tracing::info!(
                        source_manifest = manifest_uri,
                        "Sift archive bootstrap already completed; reusing restored volume"
                    ),
                }
            }
        }
    }

    let state = Arc::new(ServiceState::open_with_role(data_dir, args.role.into())?);
    let grace = Duration::from_secs(config.grace_secs.max(1));
    let reserve = Duration::from_secs((config.grace_secs / 10).min(5));
    let supervisor = server_lifecycle::TaskSupervisor::new(grace, reserve)?;
    let drain_state = state.clone();
    supervisor.register_hook(
        server_lifecycle::HookStage::AdmissionStop,
        "sift-ingest-admission",
        move |_| {
            let drain_state = drain_state.clone();
            async move {
                drain_state.start_drain();
                Ok(())
            }
        },
    )?;

    let projection_worker = Arc::new(tokio::sync::Mutex::new(Some(
        state.start_projection_worker(),
    )));
    let archive_worker = if matches!(args.role, RunRole::All | RunRole::Store) {
        let destination = std::env::var("SIFT_ARCHIVE_DESTINATION")
            .ok()
            .filter(|value| !value.trim().is_empty());
        let interval = std::env::var("SIFT_ARCHIVE_INTERVAL_SECS")
            .unwrap_or_else(|_| "60".to_string())
            .parse::<u64>()
            .context("SIFT_ARCHIVE_INTERVAL_SECS must be a positive integer")?;
        if interval == 0 {
            anyhow::bail!("SIFT_ARCHIVE_INTERVAL_SECS must be greater than zero");
        }
        Some(match destination {
            Some(destination) => {
                state.start_archive_worker(destination, Duration::from_secs(interval))
            }
            None => state.start_local_archive_worker(Duration::from_secs(interval)),
        })
    } else {
        None
    };
    let archive_worker = Arc::new(tokio::sync::Mutex::new(archive_worker));
    let drain_batches = state.clone();
    supervisor.register_hook(
        server_lifecycle::HookStage::DomainQuiesce,
        "sift-ingest-batches",
        move |_| {
            let drain_batches = drain_batches.clone();
            async move {
                drain_batches
                    .finish_drain()
                    .await
                    .map_err(|error| error.to_string())
            }
        },
    )?;
    let stop_archive = archive_worker.clone();
    supervisor.register_hook(
        server_lifecycle::HookStage::BackgroundStop,
        "sift-lifecycle-worker",
        move |_| {
            let stop_archive = stop_archive.clone();
            async move {
                if let Some(worker) = stop_archive.lock().await.take() {
                    worker.stop().await;
                }
                Ok(())
            }
        },
    )?;
    let flush_projections = projection_worker.clone();
    supervisor.register_hook(
        server_lifecycle::HookStage::FinalFlush,
        "sift-projections",
        move |_| {
            let flush_projections = flush_projections.clone();
            async move {
                if let Some(worker) = flush_projections.lock().await.take() {
                    worker.stop().await;
                }
                Ok(())
            }
        },
    )?;
    let verifier = Arc::new(SiftVerifier::from_env().await?);
    if let Some((transport, peer_port, raft_router)) = state.peer_server() {
        let listener = tokio::net::TcpListener::bind((args.host.as_str(), peer_port))
            .await
            .context("bind Sift peer mTLS listener")?;
        let address = listener
            .local_addr()
            .context("read Sift peer mTLS listener address")?;
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        tracing::info!(%address, "sift serving mutually authenticated Raft peers");
        let task = tokio::spawn(async move {
            transport
                .serve(listener, raft_router, async {
                    let _ = shutdown_rx.await;
                })
                .await
        });
        supervisor.register_oneshot_task(
            server_lifecycle::HookStage::TransportDrain,
            "sift-raft-peer",
            shutdown_tx,
            task,
        )?;
    }
    if matches!(args.role, RunRole::All | RunRole::Gateway | RunRole::Store) {
        let grpc_port = args
            .grpc_port
            .unwrap_or(if args.port == 7380 { 4317 } else { 0 });
        let listener = tokio::net::TcpListener::bind((args.host.as_str(), grpc_port))
            .await
            .context("bind Sift OTLP/gRPC listener")?;
        let address = listener
            .local_addr()
            .context("read OTLP/gRPC listener address")?;
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let grpc_state = state.clone();
        let grpc_verifier = verifier.clone();
        let grpc_store = (args.role == RunRole::Gateway).then(|| {
            std::env::var("SIFT_STORE_GRPC_ENDPOINT")
                .unwrap_or_else(|_| "http://127.0.0.1:4317".to_string())
        });
        let maximum_message_bytes = config.body_limit_bytes;
        tracing::info!(%address, "sift serving OTLP/gRPC");
        let task = tokio::spawn(async move {
            if let Some(store) = grpc_store {
                sift::grpc::serve_proxy(
                    listener,
                    &store,
                    grpc_verifier,
                    maximum_message_bytes,
                    async {
                        let _ = shutdown_rx.await;
                    },
                )
                .await
            } else {
                sift::grpc::serve(listener, grpc_state, grpc_verifier, async {
                    let _ = shutdown_rx.await;
                })
                .await
            }
        });
        supervisor.register_oneshot_task(
            server_lifecycle::HookStage::TransportDrain,
            "sift-otlp-grpc",
            shutdown_tx,
            task,
        )?;
    }
    let internal_endpoint = local_http_endpoint(&args.host, args.port);
    let data_plane = match args.role {
        RunRole::Gateway => {
            let store = std::env::var("SIFT_STORE_ENDPOINT")
                .unwrap_or_else(|_| "http://127.0.0.1:7380".to_string());
            let query = std::env::var("SIFT_QUERY_ENDPOINT").unwrap_or_else(|_| store.clone());
            sift::proxy::gateway_router(&store, &query, config.body_limit_bytes)?
                .merge(sift::mcp::http_router(&internal_endpoint)?)
                .layer(axum::middleware::from_fn_with_state(
                    verifier,
                    sift::auth::auth_middleware,
                ))
        }
        RunRole::Query => {
            let store = std::env::var("SIFT_STORE_ENDPOINT")
                .unwrap_or_else(|_| "http://127.0.0.1:7380".to_string());
            sift::query_role_router(state.clone(), &store, config.body_limit_bytes)?.layer(
                axum::middleware::from_fn_with_state(verifier, sift::auth::auth_middleware),
            )
        }
        _ => sift::protected_router_with_mcp(state.clone(), verifier, &internal_endpoint)?,
    }
    .layer(DefaultBodyLimit::max(config.body_limit_bytes));
    let app =
        service_http::standard_probe_routes(state.clone(), Some(state.clone()), sift::openapi)
            .merge(data_plane)
            .layer(service_http::trace_layer())
            // Per-request Server-Timing response attribution, composed at
            // the same outermost position as trace_layer() above (#2490).
            .layer(axum::middleware::from_fn(
                service_http::server_timing_middleware,
            ));
    let listener = tokio::net::TcpListener::bind(config.bind_addr())
        .await
        .context("bind Sift service listener")?;
    tracing::info!(address = %config.bind_addr(), "sift serving HTTP/1.1 and h2c");
    let lifecycle = supervisor.lifecycle();
    let signal_supervisor = supervisor.clone();
    let signal_task = tokio::spawn(async move {
        service_http::wait_shutdown_signal().await;
        signal_supervisor
            .shutdown("signal", "Sift shutdown signal received")
            .await
    });
    let http_report = service_http::serve_with_lifecycle(
        listener,
        app,
        service_http::HttpServerOptions::default(),
        lifecycle,
    )
    .await;
    let shutdown_report = signal_task.await.context("join Sift shutdown supervisor")?;
    let failures = shutdown_report
        .outcomes
        .iter()
        .filter(|outcome| outcome.status != server_lifecycle::HookStatus::Completed)
        .map(|outcome| format!("{}: {:?}", outcome.name, outcome.status))
        .collect::<Vec<_>>();
    if !failures.is_empty() {
        anyhow::bail!(
            "Sift shutdown did not complete cleanly: {}",
            failures.join(", ")
        );
    }
    tracing::info!(
        accepted = http_report.accepted,
        completed = http_report.completed,
        failed = http_report.failed,
        timed_out = http_report.timed_out,
        "Sift shared HTTP runtime stopped"
    );
    Ok(())
}

fn production_environment() -> bool {
    std::env::var("SIFT_PRODUCTION").ok().is_some_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "on"
        )
    }) || std::env::var("SIFT_ENVIRONMENT").ok().is_some_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "prod" | "production"
        )
    })
}

fn local_http_endpoint(host: &str, port: u16) -> String {
    let host = match host {
        "0.0.0.0" | "::" | "[::]" => "127.0.0.1".to_string(),
        host if host.starts_with('[') => host.to_string(),
        host if host.contains(':') => format!("[{host}]"),
        host => host.to_string(),
    };
    format!("http://{host}:{port}")
}

async fn query(args: QueryArgs) -> Result<()> {
    let source = read_json_input(&args.request)?;
    let request: sift::api::QueryRequestV1 =
        serde_json::from_slice(&source).context("parse QueryRequestV1 JSON")?;
    request.validate().context("validate QueryRequestV1")?;
    let response = sift::mcp::SiftApiClient::new(
        &args.endpoint,
        args.token,
        Duration::from_secs(args.request_timeout_secs),
    )?
    .query(&request)
    .await?;
    print_json_terminal(response)
}

fn read_json_input(source: &str) -> Result<Vec<u8>> {
    if source == "-" {
        let mut bytes = Vec::new();
        std::io::stdin()
            .read_to_end(&mut bytes)
            .context("read JSON from stdin")?;
        return Ok(bytes);
    }
    std::fs::read(source).with_context(|| format!("read JSON file {source}"))
}

async fn mcp(args: McpArgs) -> Result<()> {
    match args.command {
        McpCommand::Serve(args) => {
            if !args.stdio {
                anyhow::bail!("only --stdio is supported by `sift mcp serve`");
            }
            sift::mcp::serve_stdio(args.endpoint, args.token).await
        }
    }
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
    print_json_terminal(serde_json::json!({
        "format": "sift-snapshot-v2",
        "encoding": "base64",
        "bytes": bytes.len(),
        "snapshot_base64": base64::engine::general_purpose::STANDARD.encode(&bytes),
    }))
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
            sift::backup::backup_live_journal_authenticated(
                url,
                args.token.as_deref(),
                args.token_file.as_deref(),
                &args.token_audience,
                &args.project,
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
                "docker build -f apps/sift/Dockerfile -t sift:dev .",
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
    use openapi_codegen::{generate, GenOptions, HttpClient, Lang};

    let lang = match args.lang {
        GenLang::Ts => Lang::Ts,
        GenLang::Py => Lang::Py,
        GenLang::Rust => Lang::Rust,
    };
    let output = generate(
        &sift::openapi_json()?,
        &GenOptions {
            lang,
            target: None,
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
