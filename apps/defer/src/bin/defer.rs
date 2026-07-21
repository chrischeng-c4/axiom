// HANDWRITE-BEGIN gap="missing-generator:logic:defer-cli" tracker="#766" reason="Agent-facing Defer CLI, shared conventions, service startup, and HTTP domain client."
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand, ValueEnum};
use defer::{
    AuthConfig, CreateTask, DeferRaft, DeferScheduler, HttpDispatcher, QueueControlState,
    QueuePolicy, Target, TargetSigningKey,
};
use raft_runtime::Membership;
use serde_json::json;

#[derive(Parser)]
#[command(name = "defer", version, about = "Raft-backed delayed HTTP push queue")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the h2c + HTTP/1.1 service and committed dispatch workers.
    Serve(ServeArgs),
    /// Print or generate clients from the exact served OpenAPI contract.
    Spec(SpecArgs),
    /// Print offline agent-driving documentation.
    Llm(LlmArgs),
    /// Self-update this binary from a Defer GitHub release.
    Upgrade(UpgradeArgs),
    /// Search, inspect, or create Defer issues.
    Issue(IssueArgs),
    /// Configure, inspect, or control a remote queue.
    Queue(QueueArgs),
    /// Create, inspect, or cancel a remote delayed task.
    Task(TaskArgs),
    /// Trigger one remote committed delivery attempt.
    Dispatch(DispatchArgs),
    /// Upload a consistent live state-machine snapshot to file:// or s3://.
    Backup(BackupArgs),
    /// Render or run the layered Kubernetes API/operator/instance surface.
    K8s(K8sArgs),
    /// Render source-build or release-download image Dockerfiles.
    Dockerfile(DockerfileArgs),
}

#[derive(clap::Args)]
struct ServeArgs {
    #[arg(long, env = "DEFER_BIND", default_value = "0.0.0.0:7141")]
    bind: String,
    #[arg(long, env = "DEFER_DATA_DIR", default_value = ".defer/data")]
    data_dir: PathBuf,
    #[arg(long, env = "DEFER_PEER_SERVICE", default_value = "defer")]
    peer_service: String,
    #[arg(long, env = "DEFER_RAFT_PORT", default_value_t = 7142)]
    raft_port: u16,
    #[arg(long, env = "DEFER_GRACE_SECS", default_value_t = 10)]
    grace_secs: u64,
    /// Log output format. Kubernetes uses `json` for the shared
    /// `axiom.service.log.v1` collector contract; local development defaults
    /// to the human-readable formatter.
    #[arg(long, env = "DEFER_LOG_FORMAT", value_enum, default_value_t = LogFormat::Pretty)]
    log_format: LogFormat,
    #[arg(long, env = "DEFER_DISPATCH_TICK_MS", default_value_t = 100)]
    dispatch_tick_ms: u64,
    #[arg(long, env = "DEFER_DISPATCH_MAX_PER_QUEUE", default_value_t = 100)]
    dispatch_max_per_queue: usize,
    /// Maximum target HTTP requests in flight per process. The committed
    /// queue permits remain global across replicas.
    #[arg(long, env = "DEFER_DISPATCH_CONCURRENCY", default_value_t = 32)]
    dispatch_concurrency: usize,
    #[arg(long, env = "DEFER_TARGET_TIMEOUT_SECS", default_value_t = 30)]
    target_timeout_secs: u64,
    #[arg(long, env = "DEFER_TARGET_SIGNING_KEY_ID")]
    target_signing_key_id: Option<String>,
    #[arg(long, env = "DEFER_TARGET_SIGNING_SECRET_FILE")]
    target_signing_secret_file: Option<PathBuf>,
    #[arg(long, env = "DEFER_AUTH", default_value = "off")]
    auth: String,
    #[arg(long, env = "DEFER_TOKEN_REGISTRY_FILE")]
    token_registry_file: Option<PathBuf>,
    #[arg(long, env = "DEFER_OTLP_ENDPOINT")]
    otlp_endpoint: Option<String>,
    #[arg(long, env = "DEFER_BOOTSTRAP_SEED_URI")]
    bootstrap_seed_uri: Option<String>,
}

#[derive(Clone, Copy, ValueEnum)]
enum LogFormat {
    Pretty,
    Json,
}

#[derive(clap::Args)]
struct SpecArgs {
    #[command(subcommand)]
    gen: Option<SpecSubcommand>,
    #[arg(long, value_enum, default_value_t = SpecFormat::Openapi)]
    format: SpecFormat,
}

#[derive(Subcommand)]
enum SpecSubcommand {
    Gen(GenArgs),
}

#[derive(Clone, Copy, ValueEnum)]
enum SpecFormat {
    Openapi,
    OpenapiYaml,
    Routes,
}

#[derive(clap::Args)]
struct GenArgs {
    #[arg(long, value_enum)]
    lang: GenLang,
    #[arg(long)]
    out: PathBuf,
    #[arg(long, value_enum, default_value_t = GenHttp::Fetch)]
    http: GenHttp,
}

#[derive(Clone, Copy, ValueEnum)]
enum GenLang {
    Ts,
    Py,
    Rust,
}

#[derive(Clone, Copy, ValueEnum)]
enum GenHttp {
    Fetch,
    Axios,
}

#[derive(clap::Args)]
struct LlmArgs {
    #[arg(long, default_value = "outline")]
    topic: String,
    #[arg(long, default_value = "md")]
    format: String,
}

#[derive(clap::Args)]
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

#[derive(clap::Args)]
struct IssueArgs {
    #[command(subcommand)]
    command: IssueCommand,
}

#[derive(Subcommand)]
enum IssueCommand {
    Search {
        #[arg(value_name = "QUERY", num_args = 0..)]
        query: Vec<String>,
        #[arg(long, default_value = "open")]
        state: String,
        #[arg(long, default_value_t = 20)]
        limit: u32,
    },
    View {
        number: u64,
    },
    Create {
        #[arg(short = 't', long)]
        title: Option<String>,
        #[arg(value_name = "MSG", num_args = 0..)]
        message: Vec<String>,
        #[arg(long)]
        dry_run: bool,
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

#[derive(clap::Args)]
struct QueueArgs {
    #[command(subcommand)]
    command: QueueCommand,
}

#[derive(Subcommand)]
enum QueueCommand {
    Get(RemoteQueue),
    Put(QueuePutArgs),
    Control(QueueControlArgs),
}

#[derive(clap::Args)]
struct RemoteQueue {
    #[arg(long)]
    queue: String,
    #[command(flatten)]
    remote: Remote,
}

#[derive(clap::Args)]
struct QueuePutArgs {
    #[arg(long)]
    queue: String,
    #[arg(long, default_value_t = 100)]
    max_in_flight: usize,
    #[arg(long, default_value_t = 100)]
    max_dispatch_per_tick: usize,
    #[arg(long, default_value_t = 100)]
    max_dispatches_per_second: u32,
    #[arg(long, default_value_t = 100)]
    max_burst_size: usize,
    #[arg(long, default_value_t = 30_000)]
    lease_ttl_ms: u64,
    #[arg(long, default_value_t = 1_000)]
    retry_backoff_ms: u64,
    #[command(flatten)]
    remote: Remote,
}

#[derive(clap::Args)]
struct QueueControlArgs {
    #[arg(long)]
    queue: String,
    #[arg(long, value_enum)]
    state: QueueStateArg,
    #[command(flatten)]
    remote: Remote,
}

#[derive(Clone, Copy, ValueEnum)]
enum QueueStateArg {
    Running,
    Paused,
    Disabled,
}

#[derive(clap::Args)]
struct TaskArgs {
    #[command(subcommand)]
    command: TaskCommand,
}

#[derive(Subcommand)]
enum TaskCommand {
    Create(TaskCreateArgs),
    Status(TaskRef),
    Cancel(TaskRef),
}

#[derive(clap::Args)]
struct TaskCreateArgs {
    #[arg(long)]
    queue: String,
    #[arg(long)]
    task_id: String,
    #[arg(long)]
    target_url: String,
    #[arg(long, default_value = "POST")]
    method: String,
    #[arg(long, default_value = "null")]
    payload: String,
    #[arg(long)]
    schedule_at: Option<DateTime<Utc>>,
    #[arg(long, default_value_t = 10)]
    priority: u8,
    #[arg(long, default_value_t = 3)]
    max_attempts: u32,
    #[command(flatten)]
    remote: Remote,
}

#[derive(clap::Args)]
struct TaskRef {
    #[arg(long)]
    queue: String,
    #[arg(long)]
    task_id: String,
    #[command(flatten)]
    remote: Remote,
}

#[derive(clap::Args)]
struct DispatchArgs {
    #[arg(long)]
    queue: String,
    #[command(flatten)]
    remote: Remote,
}

#[derive(clap::Args)]
struct BackupArgs {
    #[arg(long, env = "DEFER_URL", default_value = "http://127.0.0.1:7141")]
    url: String,
    #[arg(long, env = "DEFER_TOKEN")]
    token: Option<String>,
    #[arg(long)]
    dest: String,
    #[arg(long)]
    retention_secs: Option<u64>,
}

#[derive(clap::Args, Debug)]
struct K8sArgs {
    #[command(subcommand)]
    command: K8sCommand,
}

#[derive(Subcommand, Debug)]
enum K8sCommand {
    Crd(K8sCrdArgs),
    Operator(K8sOperatorArgs),
    Instance(K8sInstanceArgs),
}

#[derive(clap::Args, Debug)]
struct K8sCrdArgs {
    #[command(subcommand)]
    command: K8sCrdCommand,
}

#[derive(Subcommand, Debug)]
enum K8sCrdCommand {
    Render(OutputArgs),
}

#[derive(clap::Args, Debug)]
struct K8sOperatorArgs {
    #[command(subcommand)]
    command: Option<K8sOperatorCommand>,
}

#[derive(Subcommand, Debug)]
enum K8sOperatorCommand {
    Run,
    Render(OperatorRenderArgs),
}

#[derive(clap::Args, Debug)]
struct OperatorRenderArgs {
    #[arg(long, default_value = "defer-system")]
    namespace: String,
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(clap::Args, Debug)]
struct K8sInstanceArgs {
    #[command(subcommand)]
    command: K8sInstanceCommand,
}

#[derive(Subcommand, Debug)]
enum K8sInstanceCommand {
    Render(InstanceRenderArgs),
}

#[derive(clap::Args, Debug)]
struct InstanceRenderArgs {
    #[arg(long, value_enum, default_value_t = InstanceProfile::Dev)]
    profile: InstanceProfile,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    namespace: Option<String>,
    #[arg(long)]
    image: Option<String>,
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum InstanceProfile {
    Dev,
    Staging,
    Prod,
    Template,
}

#[derive(clap::Args, Debug)]
struct OutputArgs {
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(clap::Args, Debug)]
struct DockerfileArgs {
    #[command(subcommand)]
    command: DockerfileCommand,
}

#[derive(Subcommand, Debug)]
enum DockerfileCommand {
    Render(DockerfileRenderArgs),
}

#[derive(clap::Args, Debug)]
struct DockerfileRenderArgs {
    #[arg(long, value_enum, default_value_t = DockerfileVariant::Source)]
    variant: DockerfileVariant,
    #[arg(long)]
    version: Option<String>,
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum DockerfileVariant {
    Source,
    Release,
}

#[derive(clap::Args, Clone)]
struct Remote {
    #[arg(long, env = "DEFER_URL", default_value = "http://127.0.0.1:7141")]
    url: String,
    #[arg(long, env = "DEFER_TOKEN")]
    token: Option<String>,
}

const TOOL: cli_std::ToolInfo = cli_std::ToolInfo {
    project: "defer",
    repo: "chrischeng-c4/axiom",
    target: env!("DEFER_TARGET"),
    version: env!("CARGO_PKG_VERSION"),
    git_sha: env!("DEFER_GIT_SHA"),
    built_at: env!("DEFER_BUILT_AT"),
};

const LLM_TOPICS: &[cli_std::llm::Topic] = &[
    cli_std::llm::Topic {
        id: "workflow",
        summary: "configure a queue, create a delayed task, inspect or cancel it",
        body: "Use `defer queue put --queue jobs`, then `defer task create --queue jobs --task-id ID --target-url URL --payload JSON`. The service leases due tasks through Raft before issuing HTTP. Every terminal CLI response ends with `next: done` or a runnable next command.",
    },
    cli_std::llm::Topic {
        id: "api",
        summary: "OpenAPI, h2c routes, typed clients, probes, and metrics",
        body: "`defer spec --format openapi` is the offline twin of `/openapi.json` and `/docs`. Generate clients with `defer spec gen --lang ts|py|rust --out DIR`. The one service port supports HTTP/1.1 and h2c; `/healthz`, `/readyz`, and `/metrics` are auth exempt.",
    },
    cli_std::llm::Topic {
        id: "delivery",
        summary: "push delivery, stable idempotency, HMAC signing, retry, and DLQ",
        body: "A committed lease contains executor node and fence epoch before any HTTP effect. Targets receive `Idempotency-Key`, attempt/fence headers, and optional `x-defer-signature`. Non-2xx or transport failure commits retry/DLQ; ambiguous external effects are at-least-once and use the task-stable key for dedupe.",
    },
    cli_std::llm::Topic {
        id: "ha",
        summary: "shards, replicas, committed executor ownership, snapshot, and recovery",
        body: "`SHARD_COUNT` owns storage partitioning; `REPLICAS_PER_SHARD` owns HA. Each replica applies identical scheduler commands. `POD_NAME`, `VOTER_COUNT`, and `DEFER_PEER_SERVICE` derive topology. Durable state lives under `DEFER_DATA_DIR`; lease expiry/reclaim is itself a committed transition.",
    },
    cli_std::llm::Topic {
        id: "auth",
        summary: "shared bearer registry, queue roles, and credential rotation",
        body: "Production sets `DEFER_AUTH=required` and `DEFER_TOKEN_REGISTRY_FILE`. Queue resources use read/write/admin roles and `*` wildcard grants. The live watcher keeps the last known good registry during atomic Secret rotation. Clients send `DEFER_TOKEN` as Bearer auth.",
    },
];

#[tokio::main]
async fn main() -> Result<()> {
    peer_tls::install_default_crypto_provider();
    match Cli::parse().command {
        Command::Serve(args) => serve(args).await,
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
        Command::Queue(args) => queue(args).await,
        Command::Task(args) => task(args).await,
        Command::Dispatch(args) => dispatch(args).await,
        Command::Backup(args) => backup(args).await,
        Command::K8s(args) => k8s(args).await,
        Command::Dockerfile(args) => dockerfile(args),
    }
}

fn llm(args: LlmArgs) -> Result<()> {
    println!(
        "{}",
        cli_std::llm::render(
            TOOL.project,
            TOOL.version,
            LLM_TOPICS,
            &args.topic,
            cli_std::llm::Format::parse(&args.format),
        )?
    );
    println!("next: done");
    Ok(())
}

// <HANDWRITE gap="missing-generator:logic" tracker="#2219" reason="Own the offline OpenAPI/routes projection and exact nine-operation route twin emitted from the Defer CLI.">
fn spec(args: SpecArgs) -> Result<()> {
    let json = defer::openapi::openapi().to_pretty_json()?;
    if let Some(SpecSubcommand::Gen(args)) = args.gen {
        let lang = match args.lang {
            GenLang::Ts => cclab_openapi_codegen::Lang::Ts,
            GenLang::Py => cclab_openapi_codegen::Lang::Py,
            GenLang::Rust => cclab_openapi_codegen::Lang::Rust,
        };
        let output = cclab_openapi_codegen::generate(
            &json,
            &cclab_openapi_codegen::GenOptions {
                lang,
                spec_path: PathBuf::new(),
                out_dir: args.out.clone(),
                client_name: "createDeferClient".into(),
                http_client: match args.http {
                    GenHttp::Fetch => cclab_openapi_codegen::HttpClient::Fetch,
                    GenHttp::Axios => cclab_openapi_codegen::HttpClient::Axios,
                },
                emit_types: true,
                emit_client: true,
                emit_hooks: matches!(lang, cclab_openapi_codegen::Lang::Ts),
            },
        )?;
        std::fs::create_dir_all(&args.out)?;
        for file in output.files {
            let path = args.out.join(file.rel_path);
            std::fs::write(&path, file.contents)?;
            println!("generated {}", path.display());
        }
        println!("next: done");
        return Ok(());
    }
    match args.format {
        SpecFormat::Openapi => println!("{json}"),
        SpecFormat::OpenapiYaml => {
            println!("{}", serde_yaml::to_string(&defer::openapi::openapi())?)
        }
        SpecFormat::Routes => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "routes": [
                    "PUT /v1/queues/{queue}", "GET /v1/queues/{queue}",
                    "POST /v1/queues/{queue}/control", "POST /v1/queues/{queue}/tasks",
                    "POST /v1/queues/{queue}/tasks:batch",
                    "GET /v1/queues/{queue}/tasks/{task_id}", "DELETE /v1/queues/{queue}/tasks/{task_id}",
                    "POST /v1/queues/{queue}/dispatch", "GET /admin/backup"
                ]
            }))?
        ),
    }
    println!("next: done");
    Ok(())
}
// </HANDWRITE>

async fn serve(args: ServeArgs) -> Result<()> {
    let log_format = match args.log_format {
        LogFormat::Pretty => service_http::LogFormat::Pretty,
        LogFormat::Json => service_http::LogFormat::Json,
    };
    let config = service_http::HttpConfig::new(
        "127.0.0.1",
        0,
        "info",
        log_format,
        args.grace_secs,
        0,
        args.otlp_endpoint.clone(),
    );
    let identity = service_http::ServiceIdentity::new("defer", env!("CARGO_PKG_VERSION"))?;
    service_http::init_tracing_with_identity(&config, &identity)?;

    let auth = AuthConfig::resolve(
        &args.auth,
        args.token_registry_file
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned())
            .as_deref(),
        std::env::var(defer::auth::LEGACY_TOKENS_ENV)
            .ok()
            .as_deref(),
    )?;
    let admission = service_http::AdmissionConfig::from_env("DEFER")?.controller(
        "defer.read",
        "defer.write",
        "defer.admin",
    );
    if admission.is_some() {
        tracing::info!(
            "request admission enabled (DEFER_ADMISSION_*; probes and peer routes stay exempt)"
        );
    }
    let scheduler = Arc::new(Mutex::new(DeferScheduler::new()));
    let mut peer_transport = None;
    let raft = if raft_runtime::replica_mode() {
        let transport = defer::peer_tls::from_env()?
            .map(|config| defer::peer_tls::peer_transport(&config))
            .transpose()?;
        let (port, scheme) = match transport.as_ref() {
            Some(_) => (args.raft_port, "https"),
            None => (
                args.bind
                    .rsplit(':')
                    .next()
                    .context("DEFER_BIND requires a port")?
                    .parse()?,
                "http",
            ),
        };
        let topology = raft_runtime::ClusterTopology::from_env_with_scheme(
            "defer",
            &args.peer_service,
            port,
            "DEFER_PEERS",
            scheme,
        )?;
        if let Some(seed_uri) = args.bootstrap_seed_uri.as_deref() {
            let bytes = service_backup::fetch_backup_object(seed_uri)?;
            defer::raft::prepare_bootstrap_seed(&args.data_dir, topology.node_id, &bytes)?;
        }
        let raft = Arc::new(match transport.clone() {
            Some(transport) => DeferRaft::from_topology_with_peer_transport(
                scheduler,
                &args.data_dir,
                &topology,
                DeferRaft::host_config(defer::raft::SNAPSHOT_EVERY),
                transport,
            )?,
            None => DeferRaft::from_topology(
                scheduler,
                &args.data_dir,
                &topology,
                DeferRaft::host_config(defer::raft::SNAPSHOT_EVERY),
            )?,
        });
        peer_transport = transport;
        raft
    } else {
        if let Some(seed_uri) = args.bootstrap_seed_uri.as_deref() {
            let bytes = service_backup::fetch_backup_object(seed_uri)?;
            defer::raft::prepare_bootstrap_seed(&args.data_dir, 0, &bytes)?;
        }
        Arc::new(DeferRaft::spawn(
            scheduler,
            &args.data_dir.join("raft"),
            0,
            Membership {
                voters: vec![0],
                learners: vec![],
            },
            HashMap::new(),
            DeferRaft::host_config(defer::raft::SNAPSHOT_EVERY),
        )?)
    };

    let signing = match (args.target_signing_key_id, args.target_signing_secret_file) {
        (Some(key_id), Some(path)) => Some(TargetSigningKey::new(key_id, std::fs::read(path)?)?),
        (None, None) => None,
        _ => anyhow::bail!("target signing requires both key id and secret file"),
    };
    let dispatcher = HttpDispatcher::new(Duration::from_secs(args.target_timeout_secs), signing)?;
    let peer_router = peer_transport.as_ref().map(|_| raft.router());
    let state = defer::server::AppState::new(raft, dispatcher, auth);
    if let Some(path) = args.token_registry_file.as_deref() {
        std::mem::drop(service_auth::spawn_registry_file_watcher(
            state.verifier(),
            path,
        ));
    }

    let worker_state = state.clone();
    let dispatch_tick_ms = args.dispatch_tick_ms;
    let dispatch_max_per_queue = args.dispatch_max_per_queue;
    let dispatch_concurrency = args.dispatch_concurrency;
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(dispatch_tick_ms));
        while !worker_state.is_draining() {
            interval.tick().await;
            if let Err(error) = worker_state
                .dispatch_tick(dispatch_max_per_queue, dispatch_concurrency)
                .await
            {
                tracing::warn!(error = %error, "defer dispatch tick failed");
            }
        }
    });
    let listener = tokio::net::TcpListener::bind(&args.bind).await?;
    tracing::info!(
        event = "service_listening",
        addr = %listener.local_addr()?,
        "defer listening (HTTP/1.1 + HTTP/2 cleartext)"
    );
    let app = if peer_transport.is_some() {
        defer::server::router_without_raft_routes_with_admission(state.clone(), admission)
    } else {
        defer::server::router_with_admission(state.clone(), admission)
    };
    let peer_server = match (peer_transport, peer_router) {
        (Some(transport), Some(router)) => {
            let peer_bind = peer_bind_address(&args.bind, args.raft_port)?;
            let peer_listener = tokio::net::TcpListener::bind(&peer_bind).await?;
            let (shutdown, shutdown_rx) = tokio::sync::oneshot::channel();
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
    let drain_state = state.clone();
    service_http::serve(
        listener,
        app,
        service_http::shutdown_with_drain(
            move || drain_state.start_drain(),
            Duration::from_secs(args.grace_secs),
        ),
    )
    .await;
    if let Some((shutdown, serve)) = peer_server {
        let _ = shutdown.send(());
        serve.await.context("peer listener task")??;
    }
    Ok(())
}

fn peer_bind_address(bind: &str, raft_port: u16) -> Result<String> {
    let (host, _) = bind
        .rsplit_once(':')
        .with_context(|| format!("cannot derive peer bind from {bind}"))?;
    anyhow::ensure!(!host.is_empty(), "DEFER_BIND must include a host");
    Ok(format!("{host}:{raft_port}"))
}

async fn issue(args: IssueArgs) -> Result<()> {
    match args.command {
        IssueCommand::Search {
            query,
            state,
            limit,
        } => {
            cli_std::issue::search(
                &TOOL,
                cli_std::issue::SearchOptions {
                    query: (!query.is_empty()).then(|| query.join(" ")),
                    state,
                    limit,
                },
            )
            .await
        }
        IssueCommand::View { number } => cli_std::issue::view(&TOOL, number).await,
        IssueCommand::Create {
            title,
            message,
            dry_run,
            yes,
        } => {
            let message = (!message.is_empty()).then(|| message.join(" "));
            cli_std::issue::create(
                &TOOL,
                cli_std::issue::CreateOptions {
                    title: title.unwrap_or_else(|| "defer: issue report".into()),
                    message,
                    url: None,
                    repo: None,
                    label: vec!["app:defer".into()],
                    dry_run,
                    yes,
                },
            )
            .await
        }
    }
}

fn client(remote: &Remote) -> Result<reqwest::Client> {
    let mut headers = reqwest::header::HeaderMap::new();
    if let Some(token) = &remote.token {
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {token}").parse()?,
        );
    }
    Ok(reqwest::Client::builder()
        .http2_adaptive_window(true)
        .default_headers(headers)
        .build()?)
}

async fn print_response(response: reqwest::Response, next: &str) -> Result<()> {
    let status = response.status();
    let body = response.text().await?;
    anyhow::ensure!(status.is_success(), "Defer HTTP {status}: {body}");
    if !body.is_empty() {
        println!("{body}");
    }
    println!("next: {next}");
    Ok(())
}

async fn queue(args: QueueArgs) -> Result<()> {
    match args.command {
        QueueCommand::Get(args) => {
            let response = client(&args.remote)?
                .get(format!("{}/v1/queues/{}", args.remote.url, args.queue))
                .send()
                .await?;
            print_response(response, "done").await
        }
        QueueCommand::Put(args) => {
            let policy = QueuePolicy {
                max_in_flight: args.max_in_flight,
                max_dispatch_per_tick: args.max_dispatch_per_tick,
                max_dispatches_per_second: args.max_dispatches_per_second,
                max_burst_size: args.max_burst_size,
                lease_ttl_ms: args.lease_ttl_ms,
                retry_backoff_ms: args.retry_backoff_ms,
            };
            let response = client(&args.remote)?
                .put(format!("{}/v1/queues/{}", args.remote.url, args.queue))
                .json(&policy)
                .send()
                .await?;
            print_response(response, "done").await
        }
        QueueCommand::Control(args) => {
            let state = match args.state {
                QueueStateArg::Running => QueueControlState::Running,
                QueueStateArg::Paused => QueueControlState::Paused,
                QueueStateArg::Disabled => QueueControlState::Disabled,
            };
            let response = client(&args.remote)?
                .post(format!(
                    "{}/v1/queues/{}/control",
                    args.remote.url, args.queue
                ))
                .json(&json!({"state": state}))
                .send()
                .await?;
            print_response(response, "done").await
        }
    }
}

async fn task(args: TaskArgs) -> Result<()> {
    match args.command {
        TaskCommand::Create(args) => {
            let task = CreateTask {
                task_id: args.task_id,
                target: Target {
                    url: args.target_url,
                    method: args.method,
                    headers: Default::default(),
                },
                payload: serde_json::from_str(&args.payload)
                    .unwrap_or_else(|_| json!(args.payload)),
                schedule_at: args.schedule_at.unwrap_or_else(Utc::now),
                priority: args.priority,
                max_attempts: args.max_attempts,
            };
            let response = client(&args.remote)?
                .post(format!(
                    "{}/v1/queues/{}/tasks",
                    args.remote.url, args.queue
                ))
                .json(&task)
                .send()
                .await?;
            print_response(response, "done").await
        }
        TaskCommand::Status(args) => {
            let response = client(&args.remote)?
                .get(format!(
                    "{}/v1/queues/{}/tasks/{}",
                    args.remote.url, args.queue, args.task_id
                ))
                .send()
                .await?;
            print_response(response, "done").await
        }
        TaskCommand::Cancel(args) => {
            let response = client(&args.remote)?
                .delete(format!(
                    "{}/v1/queues/{}/tasks/{}",
                    args.remote.url, args.queue, args.task_id
                ))
                .send()
                .await?;
            print_response(response, "done").await
        }
    }
}

async fn dispatch(args: DispatchArgs) -> Result<()> {
    let response = client(&args.remote)?
        .post(format!(
            "{}/v1/queues/{}/dispatch",
            args.remote.url, args.queue
        ))
        .send()
        .await?;
    print_response(response, "done").await
}

async fn backup(args: BackupArgs) -> Result<()> {
    let destination = service_backup::BackupDestination::from_uri(&args.dest)?;
    let retention = match args.retention_secs {
        Some(seconds) => service_backup::RetentionPolicy::max_age_seconds(seconds),
        None => service_backup::RetentionPolicy::default(),
    };
    let result = service_backup::run_admin_snapshot_backup(
        &args.url,
        args.token.as_deref(),
        &destination,
        &retention,
    )
    .await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    println!("next: done");
    Ok(())
}

async fn k8s(args: K8sArgs) -> Result<()> {
    match args.command {
        K8sCommand::Crd(args) => match args.command {
            K8sCrdCommand::Render(args) => {
                write_or_print(args.out.as_deref(), "crd.yaml", &crd_yaml())
            }
        },
        K8sCommand::Operator(args) => match args.command.unwrap_or(K8sOperatorCommand::Run) {
            K8sOperatorCommand::Run => run_operator().await,
            K8sOperatorCommand::Render(args) => write_or_print(
                args.out.as_deref(),
                "operator.yaml",
                &operator_yaml(&args.namespace),
            ),
        },
        K8sCommand::Instance(args) => match args.command {
            K8sInstanceCommand::Render(args) => {
                write_or_print(args.out.as_deref(), "defer.yaml", &instance_yaml(&args))
            }
        },
    }
}

#[cfg(feature = "operator")]
async fn run_operator() -> Result<()> {
    defer::operator::run().await
}

#[cfg(not(feature = "operator"))]
async fn run_operator() -> Result<()> {
    anyhow::bail!("operator runtime requires a build with `--features operator`")
}

#[cfg(feature = "operator")]
fn crd_yaml() -> String {
    defer::operator::crd_yaml()
}

#[cfg(not(feature = "operator"))]
fn crd_yaml() -> String {
    cli_std::artifact::ensure_trailing_newline(include_str!("../../k8s/operator/crd.yaml"))
}

fn operator_yaml(namespace: &str) -> String {
    let yaml = format!(
        "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: {namespace}\n---\napiVersion: v1\nkind: ServiceAccount\nmetadata:\n  name: defer-operator\n  namespace: {namespace}\n---\napiVersion: rbac.authorization.k8s.io/v1\nkind: ClusterRole\nmetadata:\n  name: defer-operator\nrules:\n  - apiGroups: [\"defer.dev\"]\n    resources: [\"defers\", \"defers/status\", \"defers/finalizers\"]\n    verbs: [\"get\", \"list\", \"watch\", \"create\", \"update\", \"patch\", \"delete\"]\n  - apiGroups: [\"apps\", \"batch\", \"policy\", \"coordination.k8s.io\"]\n    resources: [\"statefulsets\", \"cronjobs\", \"poddisruptionbudgets\", \"leases\"]\n    verbs: [\"get\", \"list\", \"watch\", \"create\", \"update\", \"patch\", \"delete\"]\n  - apiGroups: [\"\"]\n    resources: [\"services\", \"serviceaccounts\"]\n    verbs: [\"get\", \"list\", \"watch\", \"create\", \"update\", \"patch\", \"delete\"]\n---\napiVersion: rbac.authorization.k8s.io/v1\nkind: ClusterRoleBinding\nmetadata:\n  name: defer-operator\nroleRef:\n  apiGroup: rbac.authorization.k8s.io\n  kind: ClusterRole\n  name: defer-operator\nsubjects:\n  - kind: ServiceAccount\n    name: defer-operator\n    namespace: {namespace}\n---\napiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: defer-operator\n  namespace: {namespace}\nspec:\n  replicas: 2\n  selector:\n    matchLabels:\n      app.kubernetes.io/name: defer-operator\n  template:\n    metadata:\n      labels:\n        app.kubernetes.io/name: defer-operator\n    spec:\n      serviceAccountName: defer-operator\n      containers:\n        - name: operator\n          image: defer:{}\n          command: [\"defer\", \"k8s\", \"operator\", \"run\"]\n          env:\n            - name: POD_NAME\n              valueFrom: {{fieldRef: {{fieldPath: metadata.name}}}}\n            - name: POD_NAMESPACE\n              valueFrom: {{fieldRef: {{fieldPath: metadata.namespace}}}}\n          resources:\n            requests: {{cpu: 100m, memory: 128Mi}}\n            limits: {{cpu: 500m, memory: 256Mi}}\n          securityContext:\n            allowPrivilegeEscalation: false\n            readOnlyRootFilesystem: true\n            capabilities: {{drop: [\"ALL\"]}}\n",
        env!("CARGO_PKG_VERSION")
    );
    cli_std::artifact::ensure_trailing_newline(&yaml)
}

fn instance_yaml(args: &InstanceRenderArgs) -> String {
    let version = env!("CARGO_PKG_VERSION");
    let (namespace, image, body) = match args.profile {
        InstanceProfile::Dev => ("default", "defer:latest".to_string(), "  replicasPerShard: 1\n  voterCount: 1\n  storage: 1Gi\n  logLevel: debug\n"),
        InstanceProfile::Staging => ("staging", format!("defer:{version}"), "  replicasPerShard: 1\n  voterCount: 1\n  storage: 20Gi\n  logLevel: info\n"),
        InstanceProfile::Prod => ("production", format!("registry.example.com/defer:{version}"), "  imagePullPolicy: Always\n  replicasPerShard: 3\n  voterCount: 3\n  storage: 100Gi\n  graceSecs: 30\n  auth: required\n  tokensSecret: defer-token-registry\n  targetSigningSecret: defer-target-signing\n  targetSigningKeyId: active\n  peerTlsSecret: defer-peer-tls\n  backup:\n    schedule: \"0 */6 * * *\"\n    destination: s3://REPLACE_ME/defer\n    retentionSecs: 604800\n    adminTokenSecret: defer-backup-admin\n"),
        InstanceProfile::Template => ("REPLACE_ME__APP_NAMESPACE", "REPLACE_ME__REGISTRY/defer:REPLACE_ME__TAG".into(), "  replicasPerShard: REPLACE_ME__REPLICAS\n  voterCount: REPLACE_ME__VOTERS\n  storage: 10Gi\n"),
    };
    let name = args.name.as_deref().unwrap_or("defer");
    let namespace = args.namespace.as_deref().unwrap_or(namespace);
    let image = args.image.as_deref().unwrap_or(&image);
    cli_std::artifact::ensure_trailing_newline(&format!(
        "apiVersion: defer.dev/v1alpha1\nkind: Defer\nmetadata:\n  name: {name}\n  namespace: {namespace}\nspec:\n  image: {image}\n{body}  resources:\n    cpu: \"1\"\n    memory: 4Gi\n"
    ))
}

fn dockerfile(args: DockerfileArgs) -> Result<()> {
    match args.command {
        DockerfileCommand::Render(args) => {
            let (name, body) = match args.variant {
                DockerfileVariant::Source => (
                    "Dockerfile",
                    cli_std::artifact::strip_source_ownership_markers(include_str!(
                        "../../Dockerfile"
                    )),
                ),
                DockerfileVariant::Release => {
                    let tag = cli_std::artifact::release_tag(
                        "defer",
                        args.version.as_deref(),
                        env!("CARGO_PKG_VERSION"),
                    );
                    let template = cli_std::artifact::strip_source_ownership_markers(include_str!(
                        "../../Dockerfile.release"
                    ));
                    let rendered = template
                        .lines()
                        .map(|line| {
                            if line.starts_with("ARG DEFER_VERSION=") {
                                format!("ARG DEFER_VERSION={tag}")
                            } else {
                                line.to_owned()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    (
                        "Dockerfile.release",
                        cli_std::artifact::ensure_trailing_newline(&rendered),
                    )
                }
            };
            write_or_print(args.out.as_deref(), name, &body)
        }
    }
}

fn write_or_print(out: Option<&Path>, default_file: &str, body: &str) -> Result<()> {
    cli_std::artifact::write_or_print(out, default_file, body)?;
    println!("next: done");
    Ok(())
}
// HANDWRITE-END
