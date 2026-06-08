///! Persistence handle for background WAL and snapshot management
///!
///! Provides non-blocking persistence through a background thread and channel.

use super::{PersistenceConfig, PersistenceError, Result};
use super::format::WalOp;
use super::snapshot::SnapshotWriter;
use super::wal::WalWriter;
use crate::engine::KvEngine;
use crossbeam_channel::{bounded, Sender, Receiver};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Command sent to persistence thread
enum PersistenceCommand {
    /// Write operation to WAL
    LogOp(WalOp),

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

        // Create bounded channel (buffer up to 10K operations)
        let (sender, receiver) = bounded::<PersistenceCommand>(10_000);

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

    /// Log a WAL operation (non-blocking)
    pub fn log_operation(&self, op: WalOp) {
        // Send to background thread, drop if channel is full
        // This is acceptable for task backends where slight data loss is OK
        if let Err(e) = self.sender.try_send(PersistenceCommand::LogOp(op)) {
            warn!("Failed to log WAL operation: {}", e);
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

        // Tracking for flushes
        let mut last_flush = Instant::now();

        info!("Persistence thread started");

        loop {
            // Try to receive with timeout for periodic flushes
            let timeout = Duration::from_millis(config.wal_config.flush_interval_ms);

            match receiver.recv_timeout(timeout) {
                Ok(PersistenceCommand::LogOp(op)) => {
                    // Write to WAL
                    if let Err(e) = wal_writer.append(op) {
                        error!("Failed to append to WAL: {}", e);
                        continue;
                    }

                    ops_since_snapshot += 1;

                    // Check if rotation is needed
                    if wal_writer.should_rotate() {
                        debug!("WAL rotation triggered");
                        if let Err(e) = wal_writer.rotate() {
                            error!("Failed to rotate WAL: {}", e);
                        }
                    }

                    // Check if flush is needed
                    if wal_writer.should_flush() {
                        if let Err(e) = wal_writer.flush() {
                            error!("Failed to flush WAL: {}", e);
                        } else {
                            last_flush = Instant::now();
                        }
                    }
                }

                Ok(PersistenceCommand::Flush) => {
                    debug!("Force flush requested");
                    if let Err(e) = wal_writer.flush() {
                        error!("Failed to flush WAL: {}", e);
                    } else {
                        last_flush = Instant::now();
                    }
                }

                Ok(PersistenceCommand::CreateSnapshot) => {
                    info!("Creating snapshot (manual trigger)");
                    Self::create_snapshot_internal(
                        &snapshot_writer,
                        &engine,
                        &config,
                        wal_writer.position(),
                    );
                    ops_since_snapshot = 0;
                    last_snapshot = Instant::now();
                }

                Ok(PersistenceCommand::Shutdown) => {
                    info!("Shutdown signal received");
                    break;
                }

                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                    // Periodic flush check
                    if last_flush.elapsed().as_millis() >= config.wal_config.flush_interval_ms as u128 {
                        if let Err(e) = wal_writer.flush() {
                            error!("Failed to flush WAL: {}", e);
                        } else {
                            last_flush = Instant::now();
                        }
                    }

                    // Periodic snapshot check
                    let should_snapshot =
                        ops_since_snapshot >= config.snapshot_config.ops_threshold ||
                        last_snapshot.elapsed().as_secs() >= config.snapshot_config.interval_secs;

                    if should_snapshot {
                        info!(
                            "Creating snapshot ({} ops, {} seconds since last)",
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
                }

                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                    info!("Channel disconnected, shutting down");
                    break;
                }
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
    use crate::types::{KvKey, KvValue};
    use crate::persistence::WalConfig;
    use tempfile::TempDir;
    use std::thread;
    use std::time::Duration;

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
        engine.set(&key1, KvValue::String("value1".to_string()), None).unwrap();

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
