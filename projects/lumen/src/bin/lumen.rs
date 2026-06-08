//! `lumen` — the unified CLI. Today it has one subcommand, `serve`,
//! which runs a serving node.
//!
//! A serving node is symmetric: it answers reads from its local
//! materialized index and accepts writes by publishing them to the
//! write log (the broker). Apply happens in the background subscribe
//! loop — see `coordinator` / `wal`. Cluster topology lives in the
//! broker, not here; this binary only needs to know its bind address,
//! its log backend, and (for sharded routing) the shard count.
//!
//! ```text
//! lumen serve                          # single node, in-process log, :7373
//! lumen serve --wal nats --nats-url nats://nats:4222
//! lumen serve --host 0.0.0.0 --port 7373 --log-format json
//! ```

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use tracing_subscriber::EnvFilter;

use lumen::auth::AuthConfig;
use lumen::coordinator::WriteCoordinator;
use lumen::rdb::{LocalFsRdbStore, RdbSnapshot, RdbStore};
use lumen::storage::Engine;
use lumen::wal::{MemWal, SharedWal};
use lumen::wal_nats::NatsWal;

#[derive(Parser)]
#[command(
    name = "lumen",
    about = "lumen — search specialist (serving node + CLI)"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run a serving node (HTTP API + background apply loop).
    Serve(ServeArgs),
    /// Print lumen's machine-readable integration spec — offline, no server.
    /// Default: the OpenAPI 3 document; `--format json-schema` for the data
    /// types; `--shapes` for the query-shape cookbook; `--fields` for the
    /// field-type / analyzer catalog.
    Spec(SpecArgs),
    /// Print the agent integration playbook — offline, no server. `guide` (how
    /// to wire lumen in: mental model + declare→ingest→search→hydrate + flavor
    /// guide + non-goals), `quickstart` (copy-paste end-to-end), or `recipes`
    /// (task → ready-to-POST query bodies). Markdown by default; `--format json`
    /// for a machine-readable form.
    Llm(LlmArgs),
}

#[derive(Clone, Copy, ValueEnum)]
enum LlmTopic {
    /// The integration playbook (default).
    Guide,
    /// A copy-paste create → index → search walkthrough.
    Quickstart,
    /// Task → ready-to-POST query bodies (same source as `spec --shapes`).
    Recipes,
}

#[derive(Clone, Copy, ValueEnum)]
enum LlmFormat {
    /// Human/agent-readable Markdown (default).
    Md,
    /// Machine-readable JSON.
    Json,
}

#[derive(Parser)]
struct LlmArgs {
    /// Which part of the playbook to print.
    #[arg(value_enum, default_value_t = LlmTopic::Guide)]
    topic: LlmTopic,
    /// Output format.
    #[arg(long, value_enum, default_value_t = LlmFormat::Md)]
    format: LlmFormat,
}

#[derive(Clone, Copy, ValueEnum)]
enum WalBackend {
    /// In-process log. Single-node / dev. No external dependency.
    Embedded,
    /// NATS JetStream. Clustered: the broker owns the log + fan-out.
    Nats,
}

#[derive(Clone, Copy, ValueEnum)]
enum LogFormat {
    Pretty,
    Json,
}

#[derive(Clone, Copy, ValueEnum)]
enum SpecFormat {
    /// Full OpenAPI 3 document (default).
    Openapi,
    /// Just the component schemas (request/response data types).
    JsonSchema,
}

#[derive(Parser)]
struct SpecArgs {
    /// Schema format to emit when neither `--shapes` nor `--fields` is set.
    #[arg(long, value_enum, default_value_t = SpecFormat::Openapi)]
    format: SpecFormat,
    /// Emit the query-shape cookbook (canonical request examples) instead.
    #[arg(long)]
    shapes: bool,
    /// Emit the field-type / analyzer catalog instead.
    #[arg(long)]
    fields: bool,
}

#[derive(Parser)]
struct ServeArgs {
    /// Bind address. K8s passes 0.0.0.0.
    #[arg(long, env = "LUMEN_HOST", default_value = "127.0.0.1")]
    host: String,
    /// Client API port. 7373 avoids the usual collisions (8080/9200/9000).
    #[arg(long, env = "LUMEN_PORT", default_value_t = 7373)]
    port: u16,
    /// `trace|debug|info|warn|error` (overrides via RUST_LOG still apply).
    #[arg(long, env = "LUMEN_LOG_LEVEL", default_value = "info")]
    log_level: String,
    /// Log output format.
    #[arg(long, env = "LUMEN_LOG_FORMAT", value_enum, default_value_t = LogFormat::Pretty)]
    log_format: LogFormat,
    /// Write-log backend.
    #[arg(long = "wal", env = "LUMEN_WAL", value_enum, default_value_t = WalBackend::Embedded)]
    wal: WalBackend,
    /// NATS URL (used when `--wal nats`).
    #[arg(long, env = "LUMEN_NATS_URL", default_value = "nats://localhost:4222")]
    nats_url: String,
    /// Max seconds to keep retrying the initial NATS connect before giving
    /// up. A serving node started before its broker (common during a k8s
    /// rollout) retries with backoff instead of crash-looping.
    #[arg(long, env = "LUMEN_NATS_CONNECT_TIMEOUT_SECS", default_value_t = 120)]
    nats_connect_timeout_secs: u64,
    /// Shard count for client-side routing (`crc32(collection) % N`).
    /// Install-time topology constant.
    #[arg(long, env = "SHARD_COUNT", default_value_t = 1)]
    shard_count: u32,
    /// Directory for RDB snapshots (cold-start baseline). When unset,
    /// no snapshots are taken and a node rebuilds from the full log.
    #[arg(long, env = "LUMEN_DATA_DIR")]
    data_dir: Option<String>,
    /// Seconds between RDB snapshots when `--data-dir` is set.
    #[arg(long, env = "LUMEN_SNAPSHOT_SECS", default_value_t = 300)]
    snapshot_secs: u64,
    /// Graceful drain window on SIGTERM.
    #[arg(long, env = "LUMEN_GRACE_SECS", default_value_t = 30)]
    grace_secs: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Command::Serve(args) => serve(args).await,
        Command::Spec(args) => {
            // Offline self-description: no engine, no server, no I/O beyond stdout.
            let out = if args.shapes {
                serde_json::to_string_pretty(&lumen::spec::query_shapes())?
            } else if args.fields {
                serde_json::to_string_pretty(&lumen::spec::field_catalog())?
            } else {
                match args.format {
                    SpecFormat::Openapi => lumen::spec::openapi_json(),
                    SpecFormat::JsonSchema => lumen::spec::json_schema_json(),
                }
            };
            println!("{out}");
            Ok(())
        }
        Command::Llm(args) => {
            // Offline: no engine, no server, no I/O beyond stdout.
            let md = match args.topic {
                LlmTopic::Guide => lumen::spec::llm_guide_md(),
                LlmTopic::Quickstart => lumen::spec::llm_quickstart_md(),
                LlmTopic::Recipes => lumen::spec::llm_recipes_md(),
            };
            let out = match args.format {
                LlmFormat::Md => md,
                LlmFormat::Json => match args.topic {
                    // Recipes are inherently structured → emit the canonical
                    // cookbook JSON (single source with `spec --shapes`).
                    LlmTopic::Recipes => {
                        serde_json::to_string_pretty(&lumen::spec::query_shapes())?
                    }
                    LlmTopic::Guide => serde_json::to_string_pretty(
                        &serde_json::json!({ "topic": "guide", "markdown": md }),
                    )?,
                    LlmTopic::Quickstart => serde_json::to_string_pretty(
                        &serde_json::json!({ "topic": "quickstart", "markdown": md }),
                    )?,
                },
            };
            println!("{out}");
            Ok(())
        }
    }
}

async fn serve(args: ServeArgs) -> Result<()> {
    init_tracing(&args.log_level, args.log_format);

    let engine = Arc::new(Engine::new());

    // GPU probe is informational — log it so operators know whether
    // `wgpu-brute-force` vector fields will hit the GPU or fall back.
    let _ = lumen::gpu_probe::probe_and_log();

    // Select the write log.
    let wal: SharedWal = match args.wal {
        WalBackend::Embedded => {
            tracing::info!("wal=embedded (in-process; single-node)");
            Arc::new(MemWal::new())
        }
        WalBackend::Nats => {
            tracing::info!(url = %args.nats_url, "wal=nats (JetStream)");
            Arc::new(
                connect_nats_with_retry(&args.nats_url, args.nats_connect_timeout_secs)
                    .await
                    .context("connect NATS write log")?,
            )
        }
    };

    // RDB bootstrap: load the latest snapshot (if any) so we tail from
    // its sequence instead of replaying the whole log.
    let rdb_store = match &args.data_dir {
        Some(dir) => Some(Arc::new(
            LocalFsRdbStore::new(dir).context("open RDB store")?,
        )),
        None => None,
    };
    let start_seq = if let Some(store) = &rdb_store {
        match store.load_latest().await? {
            Some(rdb) => {
                let seq = rdb.up_to_seq;
                rdb.restore_into(&engine).context("restore RDB")?;
                tracing::info!(up_to_seq = seq, "restored RDB baseline");
                seq
            }
            None => 0,
        }
    } else {
        0
    };

    let writer = WriteCoordinator::start_from(wal, engine.clone(), start_seq);

    let auth = Arc::new(AuthConfig::from_env()?);
    if auth.required {
        tracing::info!(tokens = auth.tokens.len(), "auth required");
    } else {
        tracing::warn!("auth=off — set LUMEN_AUTH=required for production");
    }

    let state = lumen::api::AppState::with_components(engine.clone(), auth, writer.clone());
    let app = lumen::api::router(state);

    // Periodic RDB snapshotter.
    if let Some(store) = rdb_store {
        let snap_engine = engine.clone();
        let snap_writer = writer.clone();
        let period = Duration::from_secs(args.snapshot_secs.max(1));
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(period);
            ticker.tick().await; // skip immediate fire
            loop {
                ticker.tick().await;
                let seq = snap_writer.applied_seq();
                match RdbSnapshot::capture(&snap_engine, seq) {
                    Ok(rdb) => {
                        if let Err(e) = store.save(&rdb).await {
                            tracing::warn!(error = %e, "RDB snapshot save failed");
                        } else {
                            tracing::info!(up_to_seq = seq, "RDB snapshot written");
                            let _ = store.prune(3).await;
                        }
                    }
                    Err(e) => tracing::warn!(error = %e, "RDB capture failed"),
                }
            }
        });
    }

    let bind = format!("{}:{}", args.host, args.port);
    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .with_context(|| format!("bind {bind}"))?;
    tracing::info!(addr = %bind, shard_count = args.shard_count, "lumen serve listening");

    let grace = Duration::from_secs(args.grace_secs);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(engine.clone(), grace))
        .await?;
    Ok(())
}

/// Connect to NATS, retrying the initial connect with exponential backoff
/// (capped at 5s/attempt) until `timeout_secs` elapses. Once connected,
/// `async-nats` auto-reconnects on its own, so only the initial connect needs
/// this — it stops a serving node from crash-looping when it starts before
/// the broker (e.g. mid-rollout). The last error is returned on timeout.
async fn connect_nats_with_retry(url: &str, timeout_secs: u64) -> Result<NatsWal> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(timeout_secs);
    let mut backoff = Duration::from_millis(250);
    let mut attempt = 0u32;
    loop {
        attempt += 1;
        match NatsWal::connect(url).await {
            Ok(wal) => {
                if attempt > 1 {
                    tracing::info!(attempt, "connected to NATS write log");
                }
                return Ok(wal);
            }
            Err(e) => {
                let now = tokio::time::Instant::now();
                if now >= deadline {
                    return Err(e).with_context(|| {
                        format!("NATS unreachable after {timeout_secs}s ({attempt} attempts)")
                    });
                }
                let sleep_for = backoff.min(deadline.saturating_duration_since(now));
                tracing::warn!(
                    attempt,
                    retry_in_ms = sleep_for.as_millis() as u64,
                    error = %e,
                    "NATS connect failed; retrying"
                );
                tokio::time::sleep(sleep_for).await;
                backoff = (backoff * 2).min(Duration::from_secs(5));
            }
        }
    }
}

async fn shutdown_signal(engine: Arc<Engine>, grace: Duration) {
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
        _ = ctrl_c  => tracing::info!("received SIGINT"),
        _ = sigterm => tracing::info!("received SIGTERM"),
    }
    engine.start_drain();
    tracing::info!(grace_secs = grace.as_secs(), "draining — readyz=503");
    tokio::time::sleep(grace).await;
    tracing::info!("grace expired — shutting down");
}

fn init_tracing(level: &str, format: LogFormat) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("info,lumen={level}")));
    match format {
        LogFormat::Pretty => tracing_subscriber::fmt().with_env_filter(filter).init(),
        LogFormat::Json => tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .init(),
    }
}
