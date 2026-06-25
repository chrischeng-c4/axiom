//! keep server — HTTP/2 + OpenAPI front end over the sharded KV engine.
//!
//! Cloud-native: env-driven config, `/healthz` + `/readyz` probes, and
//! SIGTERM-aware graceful drain so k8s can roll pods without dropping requests.
//!
//! Bare `keep` (no subcommand) runs the server with the flags below; the
//! standard agent-facing commands — `keep llm`, `keep upgrade`, `keep
//! report-issue` (the CONTRIBUTING.md CLI convention, via the shared `cli-std`
//! lib) — sit alongside it. Agents start at `keep llm outline`.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use clap::{Parser, Subcommand};
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use hyper_util::server::graceful::GracefulShutdown;
use tokio::net::TcpListener;
use tower::ServiceExt;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

use keep::persistence::recovery::RecoveryManager;
use keep::persistence::{PersistenceConfig, PersistenceHandle};
use keep::{AppState, KvEngine};

#[derive(Parser, Debug)]
#[command(
    name = "keep",
    version,
    about = "keep — cloud-native KV / claim-check store (HTTP/2 + OpenAPI)"
)]
struct Cli {
    /// Standard agent-facing command. Omit it to run the server (the default).
    #[command(subcommand)]
    cmd: Option<Command>,
    /// Server flags — used when no subcommand is given (`keep <flags>`).
    #[command(flatten)]
    serve: ServeArgs,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Print agent-facing LLM topics — offline, no server. `outline` (default)
    /// maps the topics; pass a topic id for detail (`--format json` for a
    /// machine-readable form).
    Llm(LlmArgs),
    /// Self-update this binary from a published GitHub release. Resolves the
    /// running target + version, downloads the matching `keep-<target>.tar.gz`,
    /// verifies its sha256, and atomically replaces the executable. `--check`
    /// reports the available version without changing anything.
    Upgrade(UpgradeArgs),
    /// File a diagnostics-rich GitHub issue. Bundles the build version, target,
    /// git sha and OS/arch with your description, then opens an issue via
    /// `GITHUB_TOKEN` — or prints a pre-filled `issues/new` URL when no token is
    /// set. `--dry-run` previews without submitting.
    ReportIssue(ReportIssueArgs),
}

/// `keep llm` flags.
#[derive(clap::Args, Debug)]
struct LlmArgs {
    /// Topic id (`outline` lists them all).
    #[arg(default_value = "outline")]
    topic: String,
    /// Output format: `md` (default) or `json`.
    #[arg(long, default_value = "md")]
    format: String,
}

/// `keep upgrade` flags.
#[derive(clap::Args, Debug)]
struct UpgradeArgs {
    /// Report the current and latest version without modifying the binary.
    #[arg(long)]
    check: bool,
    /// Install this exact version (`0.4.3` or `keep@0.4.3`) instead of the latest.
    #[arg(long)]
    tag: Option<String>,
    /// Reinstall even when already on the selected version.
    #[arg(long)]
    force: bool,
    /// Skip the confirmation prompt.
    #[arg(short = 'y', long)]
    yes: bool,
}

/// `keep report-issue` flags.
#[derive(clap::Args, Debug)]
struct ReportIssueArgs {
    /// Issue title.
    #[arg(short = 't', long)]
    title: String,
    /// Free-text description of the problem (placed above the diagnostics block).
    #[arg(short = 'm', long)]
    message: Option<String>,
    /// Include a running node's `/version`+`/healthz` (e.g. http://localhost:7117).
    #[arg(long)]
    url: Option<String>,
    /// Target repository (`owner/name`); defaults to keep's release repo.
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
}

#[derive(clap::Args, Debug)]
struct ServeArgs {
    /// Bind host. k8s passes 0.0.0.0.
    #[arg(long, env = "KEEP_HOST", default_value = "127.0.0.1")]
    host: String,
    /// HTTP port (HTTP/1.1 + HTTP/2 cleartext on the same port).
    #[arg(long, env = "KEEP_PORT", default_value_t = 7117)]
    port: u16,
    /// Engine shard count (multi-core scaling).
    #[arg(long, env = "KEEP_SHARDS", default_value_t = 256)]
    shards: usize,
    /// `trace|debug|info|warn|error` (RUST_LOG overrides this).
    #[arg(long, env = "KEEP_LOG_LEVEL", default_value = "info")]
    log_level: String,
    /// Data directory for WAL + snapshots (the disk tier).
    #[arg(long, env = "KEEP_DATA_DIR", default_value = "./data")]
    data_dir: PathBuf,
    /// Run in-memory only (no durability).
    #[arg(long, env = "KEEP_DISABLE_PERSISTENCE", default_value_t = false)]
    disable_persistence: bool,
    /// WAL fsync interval (ms).
    #[arg(long, env = "KEEP_FSYNC_MS", default_value_t = 100)]
    fsync_interval_ms: u64,
    /// Snapshot interval (s).
    #[arg(long, env = "KEEP_SNAPSHOT_SECS", default_value_t = 300)]
    snapshot_interval_secs: u64,
    /// Snapshot trigger threshold (op count).
    #[arg(long, env = "KEEP_SNAPSHOT_OPS", default_value_t = 100_000)]
    snapshot_ops_threshold: usize,
    /// Max request body size (bytes) — bounds claim-check blob writes.
    #[arg(long, env = "KEEP_BODY_LIMIT", default_value_t = keep::http::DEFAULT_BODY_LIMIT)]
    body_limit: usize,
    /// Graceful drain window on SIGTERM (s).
    #[arg(long, env = "KEEP_GRACE_SECS", default_value_t = 30)]
    grace_secs: u64,
    /// This node's ordinal in the cluster (k8s StatefulSet pod index).
    #[arg(long, env = "KEEP_NODE_ID", default_value_t = 0)]
    node_id: usize,
    /// Total nodes in the cluster.
    #[arg(long, env = "KEEP_NODE_COUNT", default_value_t = 1)]
    node_count: usize,
    /// Virtual shard count for client-side routing (>= node_count).
    #[arg(long, env = "KEEP_SHARD_COUNT", default_value_t = 1)]
    shard_count: u32,
    /// Peer base URLs (comma-separated), index = node ordinal.
    #[arg(long, env = "KEEP_PEERS", value_delimiter = ',')]
    peers: Vec<String>,
}

/// This binary's identity + build provenance for the standard CLI ops
/// (`upgrade` / `report-issue`), per the CONTRIBUTING.md CLI convention.
const TOOL: cli_std::ToolInfo = cli_std::ToolInfo {
    project: "keep",
    repo: "chrischeng-c4/axiom",
    target: env!("KEEP_TARGET"),
    version: env!("CARGO_PKG_VERSION"),
    git_sha: env!("KEEP_GIT_SHA"),
    built_at: env!("KEEP_BUILT_AT"),
};

/// keep's agent-facing `llm` topics — the single in-code source of truth.
const TOPICS: &[cli_std::llm::Topic] = &[
    cli_std::llm::Topic {
        id: "http",
        summary: "the HTTP/2 + OpenAPI surface (KV, batch, scan, locks, collections, probes)",
        body: "# keep — HTTP/2 API surface\n\n\
            One port speaks HTTP/1.1 and HTTP/2 cleartext (h2c, prior-knowledge). JSON \
            values, or raw `application/octet-stream` blobs on the value path.\n\n\
            - `GET|PUT|DELETE|HEAD /v1/kv/{key}` — scalar value (`?ttl_ms=` on PUT).\n\
            - `POST /v1/kv/{key}/incr|cas|setnx` — atomic integer / compare-and-swap.\n\
            - `POST /v1/kv:mset|kv:mget|kv:mdel` — batch.\n\
            - `GET /v1/kv?prefix=&limit=` — prefix scan.\n\
            - `POST|DELETE /v1/locks/{name}` — owner+TTL advisory locks.\n\
            - `/v1/hashes /v1/sets /v1/zsets /v1/lists` — collections.\n\
            - `/healthz /readyz /metrics /openapi.json /docs` — probes, metrics, OpenAPI.\n\n\
            The full document: `GET /openapi.json` (served by the binary).\n",
    },
    cli_std::llm::Topic {
        id: "claim-check",
        summary: "the relay/loom worker data plane — inputs/results by id, namespaces, tokens",
        body: "# keep — claim-check data plane\n\n\
            keep is loom/relay's result store: a worker GETs its input and PUTs its result \
            by message id; the producer mirrors it.\n\n\
            - `GET|PUT /v1/inputs/{id}` — job input payload.\n\
            - `GET|PUT /v1/results/{id}` — job result payload.\n\n\
            Bytes-first (octet-stream), durable before the write is acked.\n\n\
            **Scoped tokens** (#445/#446): set `KEEP_TOKEN_SECRET` to require an HMAC \
            `Authorization: Bearer <token>` scoped to the bare input/result key on worker ops.\n\n\
            **Namespaces** (#464): send `X-Keep-Namespace` (loom sends `LOOM_NAMESPACE`) and \
            keep stores at `{ns}::{kind}:{id}` — applied after the token check, so token scope \
            stays the bare key. Absent header ⇒ bare key (single-tenant back-compat).\n",
    },
    cli_std::llm::Topic {
        id: "operate",
        summary: "run / configure / deploy — flags, env vars, persistence, k8s probes",
        body: "# keep — operating the server\n\n\
            Bare `keep` runs the server (env-driven; flags override). Key knobs:\n\n\
            - `--host/--port` (`KEEP_HOST` / `KEEP_PORT`, default `127.0.0.1:7117`).\n\
            - `--shards` (`KEEP_SHARDS`) — engine shard count for multi-core scaling.\n\
            - `--data-dir` (`KEEP_DATA_DIR`) — WAL + snapshot disk tier; cold start recovers it.\n\
            - `--disable-persistence` — in-memory only.\n\
            - `--node-id/--node-count/--shard-count/--peers` — cluster topology.\n\
            - `KEEP_TOKEN_SECRET` — enable claim-check token enforcement.\n\n\
            Cloud-native: `/healthz` + `/readyz` probes and a SIGTERM-aware graceful drain \
            (`--grace-secs`) so k8s rolls pods without dropping requests.\n",
    },
];

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        // Default (no subcommand): run the server.
        None => serve_main(cli.serve).await,
        Some(cmd) => dispatch(cmd).await,
    }
}

async fn dispatch(cmd: Command) -> Result<()> {
    match cmd {
        Command::Llm(args) => {
            // Offline: no engine, no server, no I/O beyond stdout.
            let out = cli_std::llm::render(
                TOOL.project,
                TOOL.version,
                TOPICS,
                &args.topic,
                cli_std::llm::Format::parse(&args.format),
            )?;
            println!("{out}");
            Ok(())
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
        Command::ReportIssue(args) => {
            cli_std::report_issue::run(
                &TOOL,
                cli_std::report_issue::Options {
                    title: args.title,
                    message: args.message,
                    url: args.url,
                    repo: args.repo,
                    label: args.label,
                    dry_run: args.dry_run,
                    yes: args.yes,
                },
            )
            .await
        }
    }
}

/// Run the keep server (the default, no-subcommand path).
async fn serve_main(args: ServeArgs) -> Result<()> {
    // RUST_LOG wins; otherwise fall back to --log-level.
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&args.log_level));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    info!(%addr, shards = args.shards, "starting keep");

    // Engine + persistence: recover from disk on a cold start.
    let (engine, persistence) = if args.disable_persistence {
        info!("persistence disabled — in-memory only");
        (Arc::new(KvEngine::with_shards(args.shards)), None)
    } else {
        info!(data_dir = %args.data_dir.display(), "recovering engine");
        let (recovered, stats) = RecoveryManager::recover(&args.data_dir, args.shards)?;
        info!(
            snapshot_entries = stats.snapshot_entries,
            wal_replayed = stats.wal_entries_replayed,
            corrupted = stats.corrupted_entries,
            "recovery complete"
        );
        let config = PersistenceConfig::new(&args.data_dir)
            .with_fsync_interval_ms(args.fsync_interval_ms)
            .with_snapshot_interval_secs(args.snapshot_interval_secs)
            .with_snapshot_ops_threshold(args.snapshot_ops_threshold);
        let engine = Arc::new(recovered);
        let persistence = Arc::new(PersistenceHandle::new(config, engine.clone())?);
        engine.enable_persistence(persistence.clone());
        (engine, Some(persistence))
    };

    let cluster = keep::ClusterConfig::new(
        args.node_id,
        args.node_count,
        args.shard_count,
        args.peers.clone(),
    );
    info!(
        node_id = cluster.node_id,
        node_count = cluster.node_count,
        shard_count = cluster.shard_count,
        owned_shards = cluster.owned_shards().len(),
        "cluster topology"
    );
    let mut state = AppState::new(engine)
        .with_body_limit(args.body_limit)
        .with_cluster(cluster);
    // Scoped claim-check tokens (#446): enforce when KEEP_TOKEN_SECRET is set.
    if let Ok(secret) = std::env::var("KEEP_TOKEN_SECRET") {
        if !secret.is_empty() {
            state = state.with_token_secret(secret.into_bytes());
            tracing::info!("claim-check token enforcement ON");
        }
    }
    let app = keep::router(state.clone());

    let listener = TcpListener::bind(addr).await?;
    info!(%addr, "listening (HTTP/1.1 + HTTP/2 cleartext)");

    let grace = Duration::from_secs(args.grace_secs);
    serve(listener, app, shutdown_signal(state.clone(), grace)).await;

    // Post-drain: flush WAL/snapshot to disk so the result store is durable.
    if let Some(p) = persistence {
        info!("flushing persistence");
        match Arc::try_unwrap(p) {
            Ok(p) => {
                if let Err(e) = p.shutdown() {
                    warn!(error = %e, "persistence shutdown error");
                }
            }
            Err(_) => warn!("persistence still shared — relying on Drop"),
        }
    }
    info!("shutdown complete");
    Ok(())
}

/// Accept loop serving HTTP/1.1 and HTTP/2 cleartext (h2c prior-knowledge) on
/// one socket via hyper-util's auto builder, with connection-level graceful
/// shutdown. `shutdown` resolves after SIGTERM + the drain window.
async fn serve(
    listener: TcpListener,
    app: axum::Router,
    shutdown: impl std::future::Future<Output = ()>,
) {
    let mut builder = auto::Builder::new(TokioExecutor::new());
    // Clients open ~CPU-core connections and multiplex thousands of streams
    // over each (that's the HTTP/2 best practice — see examples/bench_compare).
    // Lift the concurrent-stream ceiling so a high-concurrency client isn't
    // throttled/starved per connection (the default ~200 caused stream
    // starvation + hangs at few-connections/high-concurrency). Flow-control
    // windows are left at hyper defaults: on a low-RTT link the workload is
    // CPU-bound (frame + JSON), not window-bound, so enlarging them is a WAN-only
    // tuning with no local benefit.
    builder.http2().max_concurrent_streams(4096);
    let graceful = GracefulShutdown::new();
    let mut shutdown = std::pin::pin!(shutdown);

    loop {
        tokio::select! {
            accept = listener.accept() => {
                let (stream, _peer) = match accept {
                    Ok(s) => s,
                    Err(e) => {
                        warn!(error = %e, "accept failed");
                        continue;
                    }
                };
                let io = TokioIo::new(stream);
                let app = app.clone();
                // axum's Router is Service<Request<Incoming>>; oneshot drives one request.
                let svc = service_fn(move |req| app.clone().oneshot(req));
                let conn = builder.serve_connection_with_upgrades(io, svc);
                let conn = graceful.watch(conn.into_owned());
                tokio::spawn(async move {
                    if let Err(e) = conn.await {
                        tracing::debug!(error = %e, "connection closed with error");
                    }
                });
            }
            _ = &mut shutdown => {
                info!("no longer accepting connections");
                break;
            }
        }
    }
    drop(listener);

    // Bound the in-flight wait so a stuck client can't block process exit.
    tokio::select! {
        _ = graceful.shutdown() => info!("all connections drained"),
        _ = tokio::time::sleep(Duration::from_secs(5)) => warn!("drain timeout — forcing shutdown"),
    }
}

/// Resolve when SIGINT or SIGTERM arrives, flip `/readyz` to 503, then hold the
/// grace window so k8s stops routing before the listener closes.
async fn shutdown_signal(state: AppState, grace: Duration) {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let sigterm = async {
        use tokio::signal::unix::{signal, SignalKind};
        if let Ok(mut s) = signal(SignalKind::terminate()) {
            s.recv().await;
        }
    };
    #[cfg(not(unix))]
    let sigterm = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("received SIGINT"),
        _ = sigterm => info!("received SIGTERM"),
    }
    state.start_drain();
    info!(grace_secs = grace.as_secs(), "draining — readyz=503");
    tokio::time::sleep(grace).await;
    info!("grace expired");
}
