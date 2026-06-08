//! Background persistence thread + non-blocking command channel.
//!
//! `PersistenceHandle` owns a background OS thread that drains a bounded
//! crossbeam channel of `GraphOp` mutations and appends them to a WAL on disk.
//! It batches `fsync`s on a configurable interval and rotates WAL files when
//! they exceed `wal_max_file_size`.
//!
//! Snapshots are NOT taken automatically by this thread (would require an
//! `Arc<CtxInfEngine>` reference, creating a circular dependency). Instead,
//! the engine exposes `create_snapshot()` which calls `SnapshotWriter::create`
//! synchronously on the calling thread.
//!
//! # Observability
//!
//! [`PersistenceHandle`] exposes a running set of counters via
//! [`PersistenceHandle::stats`] returning a [`PersistenceStats`] snapshot.
//! The struct is plain `Copy` data; readers do NOT lock and there is no
//! allocation on the read path. Counters are [`AtomicU64`] (plus one
//! `usize` depth sample from `crossbeam_channel::Sender::len`) written
//! with [`Ordering::Relaxed`] — they are monitoring signals, not
//! synchronization primitives.
//!
//! Counter map (caller-thread writes `[C]`, background-thread writes `[B]`):
//!
//! - `ops_logged_total` `[C]` — successful `try_send(LogOp)` calls
//! - `ops_dropped_on_full` `[C]` — failed `try_send(LogOp)` calls (Full OR
//!   Disconnected — both count as "not admitted")
//! - `ops_appended_total` `[B]` — successful `wal_writer.append(&op)` calls
//! - `wal_bytes_written_total` `[B]` — lifetime bytes appended, accumulated
//!   via `(new_pos - prev_pos)` deltas so it survives rotation resets
//! - `wal_rotations_total` `[B]` — successful `wal_writer.rotate()` calls
//! - `flushes_requested_total` `[C]` — successful `try_send(Flush)` calls
//! - `flushes_dropped_total` `[C]` — failed `try_send(Flush)` calls
//! - `flushes_performed_total` `[B]` — successful `wal_writer.flush()` calls
//!   across all three sites (channel-triggered, auto-interval, shutdown)
//! - `last_flush_at_unix_ms` `[B]` — wall-clock ms of most recent
//!   successful flush; `0` if none
//! - `ops_buffered` — on-demand via `sender.len()` (no counter)
//! - `channel_capacity` — constant `10_000`
//!
//! The counters this module maintains are intentionally a minimal "always-on"
//! observability surface. Per-op-type breakdowns, histograms, and external
//! metric-endpoint wiring are explicitly out of scope.

use crossbeam_channel::{bounded, Receiver, Sender};
use parking_lot::Mutex;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};

use cclab_wal::{WalConfig, WalWriter};

use super::wal_ops::GraphOp;
use crate::error::{CtxInfError, Result};

/// Persistence configuration.
#[derive(Debug, Clone)]
pub struct PersistenceConfig {
    /// Directory where WAL and snapshot files live.
    pub data_dir: PathBuf,
    /// How often to fsync the WAL (milliseconds).
    pub wal_flush_interval_ms: u64,
    /// Max WAL file size before rotation (bytes).
    pub wal_max_file_size: u64,
    /// How many snapshots to keep on disk before pruning.
    pub snapshot_keep_count: usize,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("./data"),
            wal_flush_interval_ms: 100,
            wal_max_file_size: 1024 * 1024 * 1024, // 1 GiB
            snapshot_keep_count: 3,
        }
    }
}

impl PersistenceConfig {
    /// Tighter limits for tests (10ms flush, 64KiB rotation).
    pub fn for_testing(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
            wal_flush_interval_ms: 10,
            wal_max_file_size: 64 * 1024,
            snapshot_keep_count: 2,
        }
    }
}

/// Capacity of the `PersistenceHandle` command channel.
const CHANNEL_CAPACITY: usize = 10_000;

/// Command sent to the persistence background thread.
enum PersistenceCommand {
    LogOp(GraphOp),
    Flush,
    /// Synchronous flush + capture of the durable WAL position.
    ///
    /// The background thread drains any LogOp commands already in the channel,
    /// fsyncs the WAL, then sends the post-fsync `wal_position()` back on
    /// `reply`. Used by `create_snapshot` so the snapshot header records a
    /// position that is guaranteed to be on disk.
    SnapshotBarrier {
        reply: Sender<Result<u64>>,
    },
    Shutdown,
}

/// Snapshot of `PersistenceHandle` running counters.
///
/// Returned by [`PersistenceHandle::stats`]. Values are consistent
/// per-field (each read uses `Ordering::Relaxed`) but the struct as a
/// whole is not a single atomic read — individual fields may reflect
/// slightly different points in time. For monitoring / SLO purposes
/// this is intentional and sufficient; do NOT treat it as a transactional
/// snapshot.
///
/// All `_total` fields are monotonic lifetime counters, never reset.
#[derive(Debug, Clone, Copy)]
pub struct PersistenceStats {
    /// Successful `try_send(LogOp)` calls (the op was admitted to the
    /// in-memory channel; the background thread may or may not have
    /// appended it yet — see `ops_appended_total`).
    pub ops_logged_total: u64,
    /// Failed `try_send(LogOp)` calls (channel Full or Disconnected).
    /// A non-zero value means back-pressure has forced silent data loss;
    /// callers that care about durability should treat this as an alert.
    pub ops_dropped_on_full: u64,
    /// Successful `wal_writer.append(&op)` calls on the background thread.
    /// Lags `ops_logged_total` by the current in-flight channel backlog.
    pub ops_appended_total: u64,
    /// Lifetime bytes written to the WAL, accumulated as
    /// `(new_pos - prev_pos)` deltas across every append. Survives
    /// rotation resets — unlike `wal_position()` which is file-relative.
    pub wal_bytes_written_total: u64,
    /// Successful `wal_writer.rotate()` calls.
    pub wal_rotations_total: u64,
    /// Successful `try_send(Flush)` calls.
    pub flushes_requested_total: u64,
    /// Failed `try_send(Flush)` calls (channel Full or Disconnected).
    pub flushes_dropped_total: u64,
    /// Successful `wal_writer.flush()` calls (all three trigger paths:
    /// channel-triggered post-append, auto-interval, shutdown drain).
    /// Distinct from `flushes_requested_total` — auto-flushes increment
    /// this but not the request counter; dropped requests increment
    /// `flushes_dropped_total` but not this counter.
    pub flushes_performed_total: u64,
    /// Wall-clock milliseconds since the UNIX epoch of the most recent
    /// successful `wal_writer.flush()`. Zero if no flush has completed
    /// yet. A large `now - last_flush_at_unix_ms` indicates stale durable
    /// state.
    pub last_flush_at_unix_ms: u64,
    /// Current number of commands sitting in the channel, sampled via
    /// `crossbeam_channel::Sender::len` (O(1)). Stale by the time the
    /// caller reads it, but useful as a back-pressure gauge.
    pub ops_buffered: usize,
    /// Capacity of the bounded command channel. Constant `10_000`;
    /// exposed so callers can compute occupancy as
    /// `ops_buffered / channel_capacity`.
    pub channel_capacity: usize,
}

/// Internal: atomic counter block shared between the caller thread and the
/// background persistence thread.
#[derive(Debug, Default)]
struct StatsInner {
    ops_logged_total: AtomicU64,
    ops_dropped_on_full: AtomicU64,
    ops_appended_total: AtomicU64,
    wal_bytes_written_total: AtomicU64,
    wal_rotations_total: AtomicU64,
    flushes_requested_total: AtomicU64,
    flushes_dropped_total: AtomicU64,
    flushes_performed_total: AtomicU64,
    last_flush_at_unix_ms: AtomicU64,
}

/// Handle to the background persistence thread.
///
/// Cheap to share via `Arc` — internally uses an MPSC channel.
pub struct PersistenceHandle {
    sender: Sender<PersistenceCommand>,
    /// Wrapped in `Mutex` so `PersistenceHandle` can be `Sync`
    /// (`JoinHandle` is `Send` but not `Sync`).
    thread_handle: Mutex<Option<JoinHandle<()>>>,
    config: PersistenceConfig,
    /// Byte offset within the currently-active `wal-current.log`, updated by
    /// the background thread after each append/rotate. Per-file offset — only
    /// meaningful when paired with `wal_file_timestamp`.
    wal_position: Arc<AtomicU64>,
    /// Running observability counters. See [`PersistenceStats`].
    stats: Arc<StatsInner>,
    /// Maximum unix-second timestamp among WAL files that have been rotated-out
    /// so far. `0` if no rotation has happened yet. On rotation, the background
    /// thread updates this BEFORE resetting `wal_position` to the header size
    /// of the freshly-opened `wal-current.log`, so readers that grab both
    /// values (even racily) observe a consistent-enough snapshot: the
    /// timestamp monotonically increases; the position advances within the
    /// file identified by that timestamp.
    wal_file_timestamp: Arc<AtomicU64>,
}

impl PersistenceHandle {
    /// Spawn the persistence background thread.
    pub fn new(config: PersistenceConfig) -> Result<Self> {
        info!(
            "Starting persistence background thread (data_dir: {})",
            config.data_dir.display()
        );

        let (sender, receiver) = bounded::<PersistenceCommand>(CHANNEL_CAPACITY);
        let wal_position = Arc::new(AtomicU64::new(0));
        let stats = Arc::new(StatsInner::default());
        let wal_file_timestamp = Arc::new(AtomicU64::new(0));

        let thread_config = config.clone();
        let thread_position = wal_position.clone();
        let thread_stats = stats.clone();
        let thread_timestamp = wal_file_timestamp.clone();

        let thread_handle = thread::Builder::new()
            .name("ctx-inf-persistence".to_string())
            .spawn(move || {
                if let Err(e) = Self::persistence_thread(
                    thread_config,
                    thread_position,
                    thread_stats,
                    thread_timestamp,
                    receiver,
                ) {
                    error!("Persistence thread error: {}", e);
                }
            })
            .map_err(|e| CtxInfError::Storage(format!("spawn persistence thread: {}", e)))?;

        Ok(Self {
            sender,
            thread_handle: Mutex::new(Some(thread_handle)),
            config,
            wal_position,
            stats,
            wal_file_timestamp,
        })
    }

    /// Log a mutation to the WAL (non-blocking).
    /// Drops the op with a warning if the channel is full.
    ///
    /// # Durability
    ///
    /// The send is non-blocking (`try_send`, this file ~L117) and is the
    /// sole entry point by which engine CRUD operations reach the WAL. The
    /// operation is not on disk at return — see `persistence_thread`
    /// (~L142–227) for the consuming append + fsync path. Durability is
    /// reached when a subsequent fsync executes: either the interval-driven
    /// flush (`~L202–212`), an `WalWriter::should_flush()` post-append
    /// trigger (`~L179–185`), an explicit [`Self::flush`] command (`~L188–195`),
    /// or the drain-and-fsync performed on [`Self::shutdown`] (`~L221–224`).
    ///
    /// # Ordering
    ///
    /// Operations are appended to the WAL in **send order**. The channel is
    /// an MPSC (`crossbeam_channel::bounded`, this file ~L91) consumed by a
    /// single background thread, so FIFO ordering from the sender side is
    /// preserved on the receiver side. Multiple sender threads are
    /// serialized by the channel's atomic enqueue, and the background
    /// thread's `recv_timeout` loop processes commands one at a time
    /// (`~L159–218`). On recovery, `RecoveryManager::replay_wal` reads
    /// entries in file order and applies them in the same order
    /// (`recovery.rs` ~L147–185), so the recovered state reflects the exact
    /// committed send order.
    ///
    /// # Backpressure
    ///
    /// The channel capacity is hard-coded at 10,000 commands (this file
    /// ~L91). When the channel is full (consumer fell behind), `try_send`
    /// returns `Err` and the operation is **dropped** with a `warn!` log.
    /// This is the **sole failure mode** of `log_operation` — there are no
    /// other branches that can fail. Callers who must not lose writes
    /// should either (a) throttle upstream, (b) call [`Self::flush`] before
    /// bursts to reduce queue depth, or (c) treat dropped-log warnings as a
    /// monitoring signal to scale the flush interval.
    pub fn log_operation(&self, op: GraphOp) {
        match self.sender.try_send(PersistenceCommand::LogOp(op)) {
            Ok(()) => {
                self.stats.ops_logged_total.fetch_add(1, Ordering::Relaxed);
            }
            Err(e) => {
                self.stats
                    .ops_dropped_on_full
                    .fetch_add(1, Ordering::Relaxed);
                warn!("Failed to log WAL operation: {}", e);
            }
        }
    }

    /// Request a WAL fsync.
    ///
    /// # Durability
    ///
    /// This is a **fire-and-forget** request. The `Flush` command is
    /// enqueued via `try_send` (this file ~L155) and the method returns
    /// immediately — it does **not** wait for the background thread to
    /// actually perform the fsync. If the channel is full the flush
    /// request itself is dropped. When the flush does eventually run, it
    /// synchronously fsyncs the WAL writer (`persistence_thread` flush
    /// branch ~L188–195).
    ///
    /// # Ordering
    ///
    /// `Flush` queues behind any already-pending `LogOp` commands, so when
    /// it executes it covers all ops enqueued before it from the same
    /// sender. It does **not** cover ops enqueued after the `Flush` is
    /// sent.
    ///
    /// # Backpressure
    ///
    /// See [`Self::log_operation`] — same channel and capacity.
    pub fn flush(&self) {
        match self.sender.try_send(PersistenceCommand::Flush) {
            Ok(()) => {
                self.stats
                    .flushes_requested_total
                    .fetch_add(1, Ordering::Relaxed);
            }
            Err(_) => {
                self.stats
                    .flushes_dropped_total
                    .fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Synchronously drain the command channel, fsync the WAL, and return the
    /// resulting durable WAL byte offset.
    ///
    /// Unlike [`Self::flush`] (fire-and-forget), this blocks until the background
    /// thread has processed every command enqueued *before* this call and has
    /// called `sync_data` on the WAL file. The returned position is the exact
    /// offset of durable bytes at the moment the fsync completed.
    ///
    /// Used by [`crate::engine::CtxInfEngine::create_snapshot`] to guarantee
    /// the snapshot header records a WAL position that is actually on disk.
    pub fn snapshot_barrier(&self) -> Result<u64> {
        let (reply_tx, reply_rx) = bounded::<Result<u64>>(1);
        self.sender
            .send(PersistenceCommand::SnapshotBarrier { reply: reply_tx })
            .map_err(|e| CtxInfError::Storage(format!("persistence thread unreachable: {}", e)))?;
        reply_rx.recv().map_err(|e| {
            CtxInfError::Storage(format!("snapshot barrier reply channel closed: {}", e))
        })?
    }

    /// Returns the current WAL byte position **within the active
    /// `wal-current.log`** (best-effort, eventually consistent).
    ///
    /// This is a per-file offset that resets to the WAL header size on every
    /// rotation. It is only meaningful when paired with
    /// [`Self::wal_file_timestamp`] — see `WalReplayStart::FromPoint` in
    /// [`crate::storage::snapshot`] for replay semantics.
    pub fn wal_position(&self) -> u64 {
        self.wal_position.load(Ordering::Relaxed)
    }

    /// Returns the maximum unix-second timestamp among WAL files that have
    /// been rotated-out so far (best-effort, eventually consistent). `0` if no
    /// rotation has happened yet.
    ///
    /// Pairs with [`Self::wal_position`] to form a rotation-safe replay
    /// starting point: "skip every WAL file with `parsed_ts <=
    /// wal_file_timestamp` entirely; apply `wal_position` as a byte-offset skip
    /// only to the first file with `parsed_ts > wal_file_timestamp`".
    pub fn wal_file_timestamp(&self) -> u64 {
        self.wal_file_timestamp.load(Ordering::Relaxed)
    }

    /// Returns the data directory.
    pub fn data_dir(&self) -> &Path {
        &self.config.data_dir
    }

    /// Returns the snapshot keep-count.
    pub fn snapshot_keep_count(&self) -> usize {
        self.config.snapshot_keep_count
    }

    /// Snapshot of running observability counters.
    ///
    /// Cheap: each field is one relaxed atomic load plus one
    /// `crossbeam_channel::Sender::len` call. Does NOT block the caller,
    /// the background thread, or any other `stats()` reader.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cclab_ctx_inf_db::{PersistenceConfig, PersistenceHandle};
    /// use cclab_ctx_inf_db::storage::GraphOp;
    /// use cclab_ctx_inf_db::{Entity, EntityType};
    ///
    /// let config = PersistenceConfig::for_testing("./data");
    /// let handle = PersistenceHandle::new(config).unwrap();
    ///
    /// for _ in 0..10 {
    ///     let e = Entity::new(EntityType::Person, "probe");
    ///     handle.log_operation(GraphOp::CreateEntity { entity: e });
    /// }
    /// handle.flush();
    ///
    /// let s = handle.stats();
    /// assert_eq!(s.ops_logged_total, 10);
    /// assert_eq!(s.channel_capacity, 10_000);
    /// // s.ops_appended_total may lag by the channel backlog.
    /// ```
    pub fn stats(&self) -> PersistenceStats {
        PersistenceStats {
            ops_logged_total: self.stats.ops_logged_total.load(Ordering::Relaxed),
            ops_dropped_on_full: self.stats.ops_dropped_on_full.load(Ordering::Relaxed),
            ops_appended_total: self.stats.ops_appended_total.load(Ordering::Relaxed),
            wal_bytes_written_total: self.stats.wal_bytes_written_total.load(Ordering::Relaxed),
            wal_rotations_total: self.stats.wal_rotations_total.load(Ordering::Relaxed),
            flushes_requested_total: self.stats.flushes_requested_total.load(Ordering::Relaxed),
            flushes_dropped_total: self.stats.flushes_dropped_total.load(Ordering::Relaxed),
            flushes_performed_total: self.stats.flushes_performed_total.load(Ordering::Relaxed),
            last_flush_at_unix_ms: self.stats.last_flush_at_unix_ms.load(Ordering::Relaxed),
            ops_buffered: self.sender.len(),
            channel_capacity: CHANNEL_CAPACITY,
        }
    }

    /// Wall-clock milliseconds since UNIX epoch. Returns `0` on pre-1970
    /// clock skew rather than panicking — stats are best-effort.
    fn now_unix_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    fn persistence_thread(
        config: PersistenceConfig,
        wal_position: Arc<AtomicU64>,
        stats: Arc<StatsInner>,
        wal_file_timestamp: Arc<AtomicU64>,
        receiver: Receiver<PersistenceCommand>,
    ) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let wal_config = WalConfig {
            flush_interval_ms: config.wal_flush_interval_ms,
            max_file_size: config.wal_max_file_size,
        };

        let mut wal_writer = WalWriter::<GraphOp>::new(&config.data_dir, wal_config)?;
        wal_position.store(wal_writer.position(), Ordering::Relaxed);

        // Initialise max-rotated-timestamp from any pre-existing rotated WAL
        // files in the data directory. Restart-after-rotation must not lose
        // the ordering information captured in filenames.
        let initial_max_ts = Self::max_rotated_wal_ts(&config.data_dir);
        wal_file_timestamp.store(initial_max_ts, Ordering::Relaxed);

        let mut last_flush = Instant::now();

        info!("Persistence thread started");

        loop {
            let timeout = Duration::from_millis(config.wal_flush_interval_ms);

            match receiver.recv_timeout(timeout) {
                Ok(PersistenceCommand::LogOp(op)) => {
                    let prev_pos = wal_writer.position();
                    if let Err(e) = wal_writer.append(&op) {
                        error!("Failed to append to WAL: {}", e);
                        continue;
                    }
                    let new_pos = wal_writer.position();
                    wal_position.store(new_pos, Ordering::Relaxed);
                    stats.ops_appended_total.fetch_add(1, Ordering::Relaxed);
                    // Position only moves forward inside a single file; rotation
                    // resets are handled in the rotate branch below.
                    let delta = new_pos.saturating_sub(prev_pos);
                    if delta > 0 {
                        stats
                            .wal_bytes_written_total
                            .fetch_add(delta, Ordering::Relaxed);
                    }

                    if wal_writer.should_rotate() {
                        debug!("WAL rotation triggered");
                        // Pre-compute the rotation timestamp using the same
                        // `SystemTime::now().as_secs()` formula the WAL writer
                        // uses. Publish it BEFORE calling `rotate()` (which
                        // resets the per-file position) so readers can never
                        // observe (old_ts, new_position=32). After rotate,
                        // reconcile against what's actually on disk — the
                        // writer may have rounded differently, but our
                        // `max_rotated_wal_ts` scan is authoritative.
                        let predicted_ts = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0);
                        let prev_max = wal_file_timestamp.load(Ordering::Relaxed);
                        wal_file_timestamp.store(predicted_ts.max(prev_max), Ordering::Relaxed);
                        if let Err(e) = wal_writer.rotate() {
                            error!("Failed to rotate WAL: {}", e);
                        } else {
                            // Re-scan on-disk to get the authoritative max ts
                            // (handles same-second collisions where the writer
                            // clobbers an existing rotated filename).
                            let on_disk_max = Self::max_rotated_wal_ts(&config.data_dir);
                            wal_file_timestamp.store(on_disk_max, Ordering::Relaxed);
                            wal_position.store(wal_writer.position(), Ordering::Relaxed);
                            stats.wal_rotations_total.fetch_add(1, Ordering::Relaxed);
                        }
                    }

                    if wal_writer.should_flush() {
                        if let Err(e) = wal_writer.flush() {
                            error!("Failed to flush WAL: {}", e);
                        } else {
                            last_flush = Instant::now();
                            stats
                                .flushes_performed_total
                                .fetch_add(1, Ordering::Relaxed);
                            stats
                                .last_flush_at_unix_ms
                                .store(Self::now_unix_ms(), Ordering::Relaxed);
                        }
                    }
                }

                Ok(PersistenceCommand::Flush) => {
                    debug!("Force flush requested");
                    if let Err(e) = wal_writer.flush() {
                        error!("Failed to flush WAL: {}", e);
                    } else {
                        last_flush = Instant::now();
                        stats
                            .flushes_performed_total
                            .fetch_add(1, Ordering::Relaxed);
                        stats
                            .last_flush_at_unix_ms
                            .store(Self::now_unix_ms(), Ordering::Relaxed);
                    }
                }

                Ok(PersistenceCommand::SnapshotBarrier { reply }) => {
                    debug!("Snapshot barrier requested");
                    // fsync the WAL first so the position we report is durable.
                    let result = match wal_writer.flush() {
                        Ok(()) => {
                            last_flush = Instant::now();
                            let pos = wal_writer.position();
                            wal_position.store(pos, Ordering::Relaxed);
                            Ok(pos)
                        }
                        Err(e) => {
                            error!("Failed to flush WAL on snapshot barrier: {}", e);
                            Err(CtxInfError::Wal(format!(
                                "snapshot barrier fsync failed: {}",
                                e
                            )))
                        }
                    };
                    // If the caller vanished (dropped the reply receiver),
                    // we silently drop the reply — the caller can't observe it.
                    let _ = reply.send(result);
                }

                Ok(PersistenceCommand::Shutdown) => {
                    info!("Shutdown signal received");
                    break;
                }

                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                    if last_flush.elapsed().as_millis() >= config.wal_flush_interval_ms as u128 {
                        if let Err(e) = wal_writer.flush() {
                            error!("Failed to flush WAL: {}", e);
                        } else {
                            last_flush = Instant::now();
                            stats
                                .flushes_performed_total
                                .fetch_add(1, Ordering::Relaxed);
                            stats
                                .last_flush_at_unix_ms
                                .store(Self::now_unix_ms(), Ordering::Relaxed);
                        }
                    }
                }

                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                    info!("Channel disconnected, shutting down");
                    break;
                }
            }
        }

        info!("Performing final flush before shutdown");
        if let Err(e) = wal_writer.flush() {
            error!("Failed final flush: {}", e);
        } else {
            stats
                .flushes_performed_total
                .fetch_add(1, Ordering::Relaxed);
            stats
                .last_flush_at_unix_ms
                .store(Self::now_unix_ms(), Ordering::Relaxed);
        }
        info!("Persistence thread stopped");
        Ok(())
    }

    fn shutdown_internal(&self) -> Result<()> {
        let _ = self.sender.send(PersistenceCommand::Shutdown);
        if let Some(handle) = self.thread_handle.lock().take() {
            if let Err(e) = handle.join() {
                return Err(CtxInfError::Storage(format!(
                    "persistence thread panicked: {:?}",
                    e
                )));
            }
        }
        Ok(())
    }

    /// Scan `data_dir` for `wal-<unix-sec>.log` files and return the maximum
    /// parsed timestamp. `0` if no rotated files exist (`wal-current.log` is
    /// excluded — it has no embedded timestamp and is treated as `u64::MAX`
    /// during replay). Used during startup and after rotation to keep
    /// `wal_file_timestamp` aligned with on-disk reality.
    fn max_rotated_wal_ts(data_dir: &Path) -> u64 {
        let Ok(entries) = std::fs::read_dir(data_dir) else {
            return 0;
        };
        let mut max_ts = 0u64;
        for entry in entries.flatten() {
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            // wal-<unix-sec>.log — exclude wal-current.log explicitly.
            if let Some(stripped) = name
                .strip_prefix("wal-")
                .and_then(|s| s.strip_suffix(".log"))
            {
                if stripped == "current" {
                    continue;
                }
                if let Ok(ts) = stripped.parse::<u64>() {
                    if ts > max_ts {
                        max_ts = ts;
                    }
                }
            }
        }
        max_ts
    }

    /// Graceful shutdown — sends Shutdown, joins the thread.
    ///
    /// # Durability
    ///
    /// On return, every command enqueued before this call that was
    /// accepted by the channel has been processed, and a **final WAL
    /// fsync** has been performed. Internally: `shutdown_internal`
    /// blocking-sends `Shutdown` (this file ~L230), then joins the OS
    /// thread (~L231–239). The persistence thread reaches the `Shutdown`
    /// branch only after draining prior commands; it then falls through
    /// to the unconditional final `wal_writer.flush()` (~L221–224) before
    /// exiting. Any op that was **dropped** on a full channel prior to
    /// this call is not recovered by shutdown.
    ///
    /// # Ordering
    ///
    /// FIFO: all prior log/flush commands run before the shutdown fsync
    /// and thread exit.
    pub fn shutdown(self) -> Result<()> {
        info!("Shutting down persistence handle");
        self.shutdown_internal()
    }
}

impl Drop for PersistenceHandle {
    fn drop(&mut self) {
        // Only shut down if not already done via explicit shutdown().
        if self.thread_handle.lock().is_some() {
            warn!("PersistenceHandle dropped without explicit shutdown — forcing shutdown");
            let _ = self.shutdown_internal();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Entity, EntityType};
    use cclab_wal::find_wal_files;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn test_handle_lifecycle() {
        let temp = TempDir::new().unwrap();
        let config = PersistenceConfig::for_testing(temp.path());
        let handle = PersistenceHandle::new(config).unwrap();

        let entity = Entity::new(EntityType::Person, "Test");
        handle.log_operation(GraphOp::CreateEntity {
            entity: entity.clone(),
        });
        handle.flush();

        // Give the background thread a beat to process.
        thread::sleep(Duration::from_millis(100));

        let pos = handle.wal_position();
        assert!(pos > 0, "WAL position should advance after appending");

        handle.shutdown().unwrap();

        let wal_files = find_wal_files(temp.path()).unwrap();
        assert!(!wal_files.is_empty(), "WAL file should exist on disk");
    }
}
