use super::format::WalOp;
use super::snapshot::SnapshotWriter;
use super::wal::WalWriter;
///! Persistence handle for background WAL and snapshot management
///!
///! Provides non-blocking persistence through a background thread and channel.
use super::{PersistenceConfig, PersistenceError, Result};
use crate::engine::KvEngine;
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tokio::sync::oneshot;
use tracing::{debug, error, info, warn};

/// Command sent to persistence thread
enum PersistenceCommand {
    /// Write operation to WAL
    LogOp(WalOp),

    /// Durability barrier: fire the sender once every op enqueued before it is
    /// fsynced. Enables durable-before-ack with group commit.
    Barrier(oneshot::Sender<()>),

    /// Force flush WAL to disk
    Flush,

    /// Create snapshot
    CreateSnapshot,

    /// Graceful shutdown
    Shutdown,
}

/// Handle to background persistence thread
pub struct PersistenceHandle {
    /// Channel sender for sending commands to background thread
    sender: Sender<PersistenceCommand>,

    /// Background thread handle
    thread_handle: Option<JoinHandle<()>>,

    /// Configuration (kept for potential future use)
    #[allow(dead_code)]
    config: PersistenceConfig,
}

impl PersistenceHandle {
    /// Create a new persistence handle with background thread
    ///
    /// This spawns a background thread that:
    /// - Receives WAL operations through a channel
    /// - Batches writes and flushes every 100ms
    /// - Rotates WAL files at size threshold
    /// - Creates periodic snapshots
    pub fn new(config: PersistenceConfig, engine: Arc<KvEngine>) -> Result<Self> {
        info!("Starting persistence background thread");

        // Unbounded so the (sync) write path never drops a WAL op. Backpressure
        // is applied at the ack layer instead: a durable write awaits its
        // barrier, so a slow disk throttles producers rather than losing data.
        // In-flight ops are bounded by the server's concurrent-request count.
        let (sender, receiver) = unbounded::<PersistenceCommand>();

        // Clone config for thread
        let thread_config = config.clone();

        // Spawn background thread
        let thread_handle = thread::Builder::new()
            .name("kv-persistence".to_string())
            .spawn(move || {
                if let Err(e) = Self::persistence_thread(thread_config, engine, receiver) {
                    error!("Persistence thread error: {}", e);
                }
            })
            .map_err(|e| PersistenceError::Io(e))?;

        Ok(Self {
            sender,
            thread_handle: Some(thread_handle),
            config,
        })
    }

    /// Log a WAL operation. Non-blocking and lossless: the unbounded channel
    /// never rejects, so the op is always queued (only fails if the writer
    /// thread is gone, i.e. during shutdown).
    pub fn log_operation(&self, op: WalOp) {
        if let Err(e) = self.sender.send(PersistenceCommand::LogOp(op)) {
            warn!("WAL writer gone, op not logged: {}", e);
        }
    }

    /// Register a durability barrier. The returned receiver fires once every WAL
    /// op enqueued *before* this call has been fsynced to disk — the basis for
    /// durable-before-ack. Returns `None` only if the writer thread is gone.
    pub fn barrier(&self) -> Option<oneshot::Receiver<()>> {
        let (tx, rx) = oneshot::channel();
        match self.sender.send(PersistenceCommand::Barrier(tx)) {
            Ok(()) => Some(rx),
            Err(_) => None,
        }
    }

    /// Force flush WAL to disk
    pub fn flush(&self) {
        let _ = self.sender.try_send(PersistenceCommand::Flush);
    }

    /// Trigger snapshot creation
    pub fn create_snapshot(&self) {
        let _ = self.sender.try_send(PersistenceCommand::CreateSnapshot);
    }

    /// Background persistence thread
    fn persistence_thread(
        config: PersistenceConfig,
        engine: Arc<KvEngine>,
        receiver: Receiver<PersistenceCommand>,
    ) -> Result<()> {
        // Initialize WAL writer
        let mut wal_writer = WalWriter::new(config.data_dir.clone(), config.wal_config.clone())?;

        // Initialize snapshot writer
        let snapshot_writer = SnapshotWriter::new(config.snapshot_config.clone());

        // Tracking for snapshots
        let mut ops_since_snapshot = 0usize;
        let mut last_snapshot = Instant::now();

        info!("Persistence thread started");

        let timeout = Duration::from_millis(config.wal_config.flush_interval_ms);

        loop {
            // --- collect one burst of commands ---------------------------------
            // Block (with a timeout for periodic maintenance) for the first
            // command, then drain everything else already queued. The whole
            // burst shares a single fsync below — that's the group commit: N
            // concurrent durable writes cost one fsync, not N.
            let mut cmds: Vec<PersistenceCommand> = Vec::new();
            match receiver.recv_timeout(timeout) {
                Ok(cmd) => cmds.push(cmd),
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {}
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                    info!("Channel disconnected, shutting down");
                    break;
                }
            }
            while let Ok(cmd) = receiver.try_recv() {
                cmds.push(cmd);
            }

            // --- apply the burst ----------------------------------------------
            let mut barriers: Vec<oneshot::Sender<()>> = Vec::new();
            let mut wrote = false;
            let mut force_flush = false;
            let mut want_snapshot = false;
            let mut stop = false;

            for cmd in cmds {
                match cmd {
                    PersistenceCommand::LogOp(op) => {
                        if let Err(e) = wal_writer.append(op) {
                            error!("Failed to append to WAL: {}", e);
                            continue;
                        }
                        ops_since_snapshot += 1;
                        wrote = true;
                    }
                    PersistenceCommand::Barrier(tx) => barriers.push(tx),
                    PersistenceCommand::Flush => force_flush = true,
                    PersistenceCommand::CreateSnapshot => want_snapshot = true,
                    PersistenceCommand::Shutdown => stop = true,
                }
            }

            // --- group commit: one fsync for the whole burst -------------------
            let has_barriers = !barriers.is_empty();
            if wrote || force_flush || has_barriers {
                if let Err(e) = wal_writer.flush() {
                    error!("Failed to flush WAL: {}", e);
                }
            }
            // Release awaiting durable writes — their ops are now on disk.
            for b in barriers {
                let _ = b.send(());
            }

            if wrote && wal_writer.should_rotate() {
                debug!("WAL rotation triggered");
                if let Err(e) = wal_writer.rotate() {
                    error!("Failed to rotate WAL: {}", e);
                }
            }

            // --- periodic maintenance -----------------------------------------
            // On an idle tick (no ops/barriers) still flush any straggler bytes.
            if !wrote && !has_barriers && !force_flush && wal_writer.should_flush() {
                if let Err(e) = wal_writer.flush() {
                    error!("Failed to flush WAL: {}", e);
                }
            }

            let should_snapshot = want_snapshot
                || ops_since_snapshot >= config.snapshot_config.ops_threshold
                || last_snapshot.elapsed().as_secs() >= config.snapshot_config.interval_secs;
            if should_snapshot && (ops_since_snapshot > 0 || want_snapshot) {
                info!(
                    "Creating snapshot ({} ops, {}s since last)",
                    ops_since_snapshot,
                    last_snapshot.elapsed().as_secs()
                );
                Self::create_snapshot_internal(
                    &snapshot_writer,
                    &engine,
                    &config,
                    wal_writer.position(),
                );
                ops_since_snapshot = 0;
                last_snapshot = Instant::now();
            }

            if stop {
                info!("Shutdown signal received");
                break;
            }
        }

        // Final flush before shutdown
        info!("Performing final flush before shutdown");
        if let Err(e) = wal_writer.flush() {
            error!("Failed final flush: {}", e);
        }

        info!("Persistence thread stopped");
        Ok(())
    }

    /// Helper to create snapshot (called from persistence thread)
    fn create_snapshot_internal(
        snapshot_writer: &SnapshotWriter,
        engine: &Arc<KvEngine>,
        config: &PersistenceConfig,
        wal_position: u64,
    ) {
        match snapshot_writer.create_snapshot(engine.as_ref(), &config.data_dir, wal_position) {
            Ok(path) => {
                info!("Snapshot created: {}", path.display());
            }
            Err(e) => {
                error!("Failed to create snapshot: {}", e);
            }
        }
    }

    /// Internal shutdown implementation
    fn shutdown_internal(&mut self) -> Result<()> {
        info!("Shutting down persistence handle");

        // Send shutdown signal
        let _ = self.sender.send(PersistenceCommand::Shutdown);

        // Wait for thread to finish
        if let Some(handle) = self.thread_handle.take() {
            if let Err(e) = handle.join() {
                error!("Failed to join persistence thread: {:?}", e);
                return Err(PersistenceError::DataDirectory(format!(
                    "Persistence thread panic: {:?}",
                    e
                )));
            }
        }

        Ok(())
    }

    /// Graceful shutdown of persistence thread (consumes self)
    pub fn shutdown(mut self) -> Result<()> {
        self.shutdown_internal()
    }
}

impl Drop for PersistenceHandle {
    fn drop(&mut self) {
        if self.thread_handle.is_some() {
            warn!("PersistenceHandle dropped without explicit shutdown, forcing shutdown");
            let _ = self.shutdown_internal();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::WalConfig;
    use crate::types::{KvKey, KvValue};
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn test_persistence_handle_basic() {
        let temp_dir = TempDir::new().unwrap();

        let config = PersistenceConfig {
            data_dir: temp_dir.path().to_path_buf(),
            wal_config: WalConfig {
                flush_interval_ms: 50, // Faster for testing
                max_file_size: 1024 * 1024,
            },
            ..Default::default()
        };

        let engine = Arc::new(KvEngine::new());
        let handle = PersistenceHandle::new(config, engine.clone()).unwrap();

        // Log some operations
        handle.log_operation(WalOp::Set {
            key: "test".to_string(),
            value: KvValue::String("value".to_string()),
            ttl: None,
        });

        handle.log_operation(WalOp::Delete {
            key: "test2".to_string(),
        });

        // Force flush
        handle.flush();

        // Give it time to process
        thread::sleep(Duration::from_millis(200));

        // Shutdown
        let _ = handle.shutdown();

        // Verify WAL file was created
        let wal_files = super::super::wal::find_wal_files(temp_dir.path()).unwrap();
        assert!(!wal_files.is_empty(), "WAL file should be created");
    }

    #[test]
    fn test_persistence_handle_snapshot() {
        let temp_dir = TempDir::new().unwrap();

        let config = PersistenceConfig {
            data_dir: temp_dir.path().to_path_buf(),
            wal_config: WalConfig::default(),
            ..Default::default()
        };

        let engine = Arc::new(KvEngine::new());

        // Add some data to engine
        let key1 = KvKey::new("key1").unwrap();
        engine
            .set(&key1, KvValue::String("value1".to_string()), None)
            .unwrap();

        let handle = PersistenceHandle::new(config, engine.clone()).unwrap();

        // Trigger snapshot creation
        handle.create_snapshot();

        // Give it time to create snapshot
        thread::sleep(Duration::from_millis(500));

        let _ = handle.shutdown();

        // Verify snapshot was created
        let snapshots = super::super::snapshot::find_snapshot_files(temp_dir.path()).unwrap();
        assert!(!snapshots.is_empty(), "Snapshot should be created");
    }
}
