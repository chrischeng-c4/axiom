//! KV Store Server
//!
//! High-performance TCP server for the cclab KV store.

use clap::Parser;
use cclab_kv::persistence::{PersistenceConfig, PersistenceHandle};
use cclab_kv::{KvEngine, KvServer};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::signal;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(name = "kv-server")]
#[command(about = "High-performance KV store TCP server")]
struct Args {
    /// Address to bind to
    #[arg(short, long, default_value = "127.0.0.1:7117")]
    bind: SocketAddr,

    /// Number of shards for the KV engine
    #[arg(short, long, default_value = "256")]
    shards: usize,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: Level,

    /// Data directory for persistence (WAL and snapshots)
    #[arg(long, default_value = "./data")]
    data_dir: PathBuf,

    /// Disable persistence (in-memory only)
    #[arg(long, default_value = "false")]
    disable_persistence: bool,

    /// WAL fsync interval in milliseconds
    #[arg(long, default_value = "100")]
    fsync_interval_ms: u64,

    /// Snapshot creation interval in seconds
    #[arg(long, default_value = "300")]
    snapshot_interval_secs: u64,

    /// Snapshot creation threshold (operations count)
    #[arg(long, default_value = "100000")]
    snapshot_ops_threshold: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Setup logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(args.log_level)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting KV server on {}", args.bind);
    info!("Shards: {}", args.shards);

    // Create or recover engine (returns Arc for sharing with server and persistence)
    let (engine_arc, persistence_handle) = if args.disable_persistence {
        info!("Persistence disabled - running in-memory only");
        (Arc::new(KvEngine::with_shards(args.shards)), None)
    } else {
        info!("Persistence enabled - data directory: {}", args.data_dir.display());

        // Attempt recovery
        let (recovered_engine, stats) =
            cclab_kv::persistence::recovery::RecoveryManager::recover(
                &args.data_dir,
                args.shards,
            )?;

        if stats.snapshot_loaded {
            info!(
                "Recovered from snapshot: {} entries in {:?}",
                stats.snapshot_entries,
                stats.recovery_duration
            );
        }

        if stats.wal_entries_replayed > 0 {
            info!(
                "Replayed WAL: {} entries in {:?}",
                stats.wal_entries_replayed,
                stats.recovery_duration
            );
        }

        if stats.corrupted_entries > 0 {
            warn!(
                "Skipped {} corrupted entries during recovery",
                stats.corrupted_entries
            );
        }

        if !stats.snapshot_loaded && stats.wal_entries_replayed == 0 {
            info!("No previous data found - starting fresh");
        }

        // Setup persistence
        let config = PersistenceConfig::new(&args.data_dir)
            .with_fsync_interval_ms(args.fsync_interval_ms)
            .with_snapshot_interval_secs(args.snapshot_interval_secs)
            .with_snapshot_ops_threshold(args.snapshot_ops_threshold);

        // Wrap recovered engine in Arc for sharing between persistence and server
        let engine_arc = Arc::new(recovered_engine);

        // Create persistence handle with the Arc (clones the Arc, not the engine)
        let persistence = Arc::new(PersistenceHandle::new(config, engine_arc.clone())?);

        // Enable persistence on the engine (uses interior mutability)
        engine_arc.enable_persistence(persistence.clone());

        info!(
            "Persistence configured: fsync={}ms, snapshot={}s or {} ops",
            args.fsync_interval_ms,
            args.snapshot_interval_secs,
            args.snapshot_ops_threshold
        );

        (engine_arc, Some(persistence))
    };

    // Create server with engine (shares the Arc)
    let server = KvServer::with_engine(engine_arc);

    // Setup graceful shutdown
    let server_task = tokio::spawn(async move {
        if let Err(e) = server.run(args.bind).await {
            eprintln!("Server error: {}", e);
        }
    });

    // Wait for shutdown signal
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Shutdown signal received");
        }
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        }
    }

    // Graceful shutdown
    if let Some(persistence_arc) = persistence_handle {
        info!("Flushing persistence...");
        // Try to unwrap Arc to get ownership for shutdown
        match Arc::try_unwrap(persistence_arc) {
            Ok(persistence) => {
                if let Err(e) = persistence.shutdown() {
                    warn!("Error during persistence shutdown: {}", e);
                } else {
                    info!("Persistence shutdown complete");
                }
            }
            Err(_) => {
                warn!("Could not get exclusive access to persistence handle for shutdown");
                // The Drop implementation will handle cleanup
            }
        }
    }

    server_task.abort();
    info!("Server shutdown complete");

    Ok(())
}
