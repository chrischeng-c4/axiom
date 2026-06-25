//! keep server — HTTP/2 + OpenAPI front end over the sharded KV engine.
//!
//! Cloud-native: env-driven config, `/healthz` + `/readyz` probes, and
//! SIGTERM-aware graceful drain so k8s can roll pods without dropping requests.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
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
    about = "keep — cloud-native KV / claim-check store (HTTP/2 + OpenAPI)"
)]
struct Args {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

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
    let mut state = AppState::new(engine).with_body_limit(args.body_limit).with_cluster(cluster);
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
