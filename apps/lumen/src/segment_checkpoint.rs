//! Shared durable segment checkpoint path for manual requests and background work.
//! File encoding and AOF trimming happen outside the apply barrier.

use crate::storage::Engine;
use anyhow::{Context, Result};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::{mpsc, oneshot};
use tokio::time::Instant;

use crate::change_budget::{BudgetWake, ChangeBudget, Snapshot};

const WAITER_SHUTDOWN_POLL: Duration = Duration::from_millis(50);

/// Real [`crate::api::CheckpointSink`] wiring for segment-persistence mode
/// (#1389): forces the same synchronous stage-then-rename checkpoint the
/// periodic snapshotter performs (`SegmentRdbStore::save`), but synchronously
/// on demand — this is what `POST /admin/checkpoint` answers, and what the
/// reshard driver's cutover gate (`service_k8s::reshard_driver::
/// checkpoint_touched_shards`) awaits per touched shard before triggering the
/// cutover rolling restart. Also prunes + trims the AOF through the
/// checkpointed sequence, mirroring the periodic path exactly, so an
/// on-demand checkpoint leaves the AOF in the same state a periodic one
/// would (and a reshard cutover right after one doesn't leave a redundant,
/// ever-growing AOF tail).
pub struct SegmentCheckpointSink {
    pub engine: Arc<Engine>,
    pub store: Arc<crate::segment_rdb::SegmentRdbStore>,
    pub writer: Arc<dyn crate::coordinator::WriteSink>,
    pub aof: Option<crate::coordinator::SharedAof>,
}

/// Keeps checkpoint-attempt telemetry balanced across every synchronous return
/// path. It deliberately owns no checkpoint state, so it cannot change save,
/// trim, or publication ordering.
struct CheckpointAttempt<'a> {
    metrics: &'a crate::metrics::Metrics,
    failed: bool,
}

impl<'a> CheckpointAttempt<'a> {
    fn start(metrics: &'a crate::metrics::Metrics) -> Self {
        metrics.start_segment_checkpoint_attempt();
        Self {
            metrics,
            failed: false,
        }
    }

    fn mark_failed(&mut self) {
        self.failed = true;
    }
}

impl Drop for CheckpointAttempt<'_> {
    fn drop(&mut self) {
        self.metrics.finish_segment_checkpoint_attempt(self.failed);
    }
}

/// Owns one native budget waiter and one Tokio checkpoint task.
#[doc(hidden)]
pub struct SegmentCheckpointDriver {
    stop: Arc<AtomicBool>,
    waiter: Option<std::thread::JoinHandle<()>>,
    checkpoint_task: Option<tokio::task::JoinHandle<()>>,
    shutdown: Option<oneshot::Sender<()>>,
    capacity_owner: Option<crate::segment_capacity::Owner>,
}

impl Drop for SegmentCheckpointDriver {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        if let Some(task) = self.checkpoint_task.take() {
            task.abort();
        }
        if let Some(waiter) = self.waiter.take() {
            let _ = waiter.join();
        }
    }
}

impl SegmentCheckpointDriver {
    /// Stop new scheduling without waiting for an already-owned save. Shutdown
    /// cache IO subsequently joins the same store gate within its own deadline.
    #[doc(hidden)]
    pub fn request_stop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        if let Some(owner) = &mut self.capacity_owner {
            owner.stop();
        }
    }

    /// Stop scheduling and wait for a started blocking save before another
    /// driver uses the same checkpoint root. `Drop` remains emergency cleanup.
    #[doc(hidden)]
    pub async fn shutdown(&mut self) -> Result<()> {
        self.stop.store(true, Ordering::Release);
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        // Await through a mutable field borrow. Cancellation leaves the task
        // handle in self, so a later shutdown can still wait for the same save.
        let result = match self.checkpoint_task.as_mut() {
            Some(task) => task
                .await
                .map_err(|error| anyhow::anyhow!("checkpoint task failed: {error}")),
            None => Ok(()),
        };
        self.checkpoint_task.take();
        result?;
        if let Some(mut owner) = self.capacity_owner.take() {
            tokio::task::spawn_blocking(move || owner.join())
                .await
                .map_err(|error| anyhow::anyhow!("capacity shutdown task failed: {error}"))??;
        }
        // The task has now observed stop after any started blocking save. The
        // waiter exits within WAITER_SHUTDOWN_POLL, so this bounded join has no
        // await point that could lose its ownership to cancellation.
        if let Some(waiter) = self.waiter.take() {
            waiter
                .join()
                .map_err(|_| anyhow::anyhow!("checkpoint waiter panicked"))?;
        }
        Ok(())
    }
}

/// A high-water crossing schedules one immediate checkpoint. Each successful
/// publication can schedule one successor when new owner work arrived during
/// the checkpoint and remains above the trigger. A drained owner also retains
/// one successor for its next active-byte crossing, even if reservations keep
/// the process total high. Unchanged high work waits for the normal
/// post-completion period. New capacity requests can also schedule an immediate
/// attempt without waiting for the pressure to drain.
struct CheckpointSchedule {
    period: Duration,
    next_deadline: Instant,
    early_attempted: bool,
    immediate_successor: Option<u64>,
    /// The latest explicit capacity request that has been scheduled. A newer
    /// request must be allowed to bypass the pressure-epoch coalescing: the
    /// earlier checkpoint may have completed while the process remained near
    /// its hard limit.
    last_request_revision: Option<u64>,
}

// Do not re-arm on a brief dip below the 128 MiB trigger. A real drain must
// leave enough headroom before the next crossing can schedule an immediate
// checkpoint. Successful publications and new capacity requests are separate
// reasons to retry while the process remains above the threshold.
const CHECKPOINT_REARM_THRESHOLD: usize = crate::change_budget::CHECKPOINT_TRIGGER / 2;

impl CheckpointSchedule {
    fn new(period: Duration, now: Instant) -> Self {
        Self {
            period,
            next_deadline: now + period,
            early_attempted: false,
            immediate_successor: None,
            last_request_revision: None,
        }
    }

    fn should_attempt(&mut self, now: Instant, pending: Snapshot, _work_revision: u64) -> bool {
        let newer_capacity_request = pending
            .checkpoint_request_revision
            .filter(|revision| {
                self.last_request_revision
                    .is_none_or(|last| *revision > last)
            })
            .is_some_and(|revision| {
                pending.checkpoint_needed()
                    && {
                        self.last_request_revision = Some(revision);
                        true
                    }
            });
        if newer_capacity_request {
            return true;
        }
        if pending.total < CHECKPOINT_REARM_THRESHOLD {
            self.early_attempted = false;
        }
        let periodic_due = now >= self.next_deadline;
        if periodic_due {
            // A periodic attempt does not reset the pressure epoch. High work
            // alone cannot trigger another early attempt without a real drain.
            if pending.total >= CHECKPOINT_REARM_THRESHOLD {
                self.early_attempted = true;
            }
            return true;
        }
        let early = !self.early_attempted && pending.checkpoint_needed();
        if early {
            self.early_attempted = true;
        }
        early
    }

    fn completed(&mut self, now: Instant, pending: Snapshot) {
        self.immediate_successor = None;
        if pending.total >= CHECKPOINT_REARM_THRESHOLD {
            self.early_attempted = true;
        }
        self.next_deadline = now + self.period;
    }

    fn take_successor(&mut self, owner: Option<crate::change_budget::OwnerCapacityState>) -> bool {
        let Some(armed_revision) = self.immediate_successor else {
            return false;
        };
        let Some(owner) = owner.filter(|owner| owner.work_revision >= armed_revision) else {
            self.immediate_successor = None;
            return false;
        };
        if owner.active < crate::change_budget::CHECKPOINT_TRIGGER {
            // Reservations are not checkpointable. Keep the successful
            // publication's one successor until this owner's active work
            // crosses the existing trigger, rather than consuming it on a
            // below-trigger poll and waiting for the next periodic deadline.
            return false;
        }
        self.immediate_successor = None;
        true
    }

    fn completed_success(
        &mut self,
        now: Instant,
        after: Snapshot,
        owner_before: Option<crate::change_budget::OwnerCapacityState>,
        owner_after: Option<crate::change_budget::OwnerCapacityState>,
    ) {
        self.next_deadline = now + self.period;
        self.early_attempted = after.total >= CHECKPOINT_REARM_THRESHOLD;
        self.immediate_successor = match (owner_before, owner_after) {
            (Some(_), Some(after)) if after.active < crate::change_budget::CHECKPOINT_TRIGGER => {
                Some(after.work_revision)
            }
            (Some(before), Some(after))
                if after.active > 0
                    && after.work_revision > before.work_revision
                    && (after.active >= crate::change_budget::CHECKPOINT_TRIGGER
                        || after
                            .checkpoint_request_revision
                            .is_some_and(|revision| revision > before.work_revision)) =>
            {
                Some(after.work_revision)
            }
            _ => None,
        };
        if after.total < crate::change_budget::CHECKPOINT_TRIGGER {
            self.early_attempted = false;
        }
    }
}

impl SegmentCheckpointSink {
    fn complete_periodic_checkpoint(
        schedule: &mut CheckpointSchedule,
        after: Snapshot,
        owner_before: Option<crate::change_budget::OwnerCapacityState>,
        owner_after: Option<crate::change_budget::OwnerCapacityState>,
    ) {
        schedule.completed_success(Instant::now(), after, owner_before, owner_after);
    }

    /// Standalone shutdown-only preparation. The caller owns its absolute
    /// timeout. A detached native thread cannot make Tokio runtime shutdown
    /// wait forever for optional graph IO; incomplete files remain cache misses.
    #[doc(hidden)]
    pub async fn save_shutdown_graph_cache(self: Arc<Self>) -> Result<usize> {
        let gate = self.writer.mutation_gate().context("shutdown cache requires a writer fence")?;
        let permit = gate.exclusive().await?;
        let (send, receive) = oneshot::channel();
        std::thread::Builder::new()
            .name("lumen-hnsw-shutdown-cache".into())
            .spawn(move || {
                let _permit = permit;
                let result = (|| {
                    if !self.engine.has_hnsw_graphs()? && !self.store.has_hnsw_graph_cache() {
                        return Ok(0);
                    }
                    if let Some(aof) = &self.aof {
                        // The exclusive writer fence includes every completed
                        // append. Preserve that durable tail instead of spending
                        // Docker's stop window rewriting the full checkpoint.
                        aof.lock()
                            .map_err(|_| anyhow::anyhow!("aof writer poisoned"))?
                            .sync()?;
                    } else {
                        self.checkpoint_sync(&self.store)?;
                    }
                    self.store.save_hnsw_graph_caches(&self.engine)
                })();
                let _ = send.send(result);
            })
            .context("start optional shutdown cache worker")?;
        receive.await.context("shutdown cache worker stopped")?
    }

    async fn checkpoint_with_fence(
        &self,
        fence: Option<crate::segment_capacity::PublicationFence>,
    ) -> Result<bool> {
        let sink_engine = self.engine.clone();
        let sink_store = self.store.clone();
        let sink_writer = self.writer.clone();
        let sink_aof = self.aof.clone();
        tokio::task::spawn_blocking(move || {
            let sink = Arc::new(SegmentCheckpointSink {
                engine: sink_engine,
                store: sink_store,
                writer: sink_writer,
                aof: sink_aof,
            });
            // Capacity refusal can happen while a manual checkpoint is frozen.
            // Register this sink first so relief uses the same root, rather than
            // retaining another full capture in a temporary spill store. Keep
            // the owner in the blocking task even if the async caller cancels.
            let _manual_owner = if fence.is_none() {
                crate::segment_capacity::Owner::start(sink.clone(), false)?
            } else {
                None
            };
            // Manual publication and its background merge may outlive this
            // scoped owner. Only a persistent driver's supplied fence applies
            // to them; a later manual request must not invalidate that merge.
            let store = match fence {
                Some(fence) => sink.store.as_ref().clone().with_publication_fence(fence),
                None => sink.store.as_ref().clone(),
            };
            sink.checkpoint_sync(&store)
        })
        .await
        .context("checkpoint task panicked")??;
        Ok(true)
    }

    pub(crate) fn checkpoint_sync(
        &self,
        store: &crate::segment_rdb::SegmentRdbStore,
    ) -> Result<()> {
        let mut attempt = CheckpointAttempt::start(self.engine.metrics());
        let result = (|| {
            if self
                .writer
                .mutation_gate()
                .is_some_and(|gate| gate.is_restart_required())
            {
                return Err(anyhow::Error::new(crate::coordinator::RestartRequired(
                    "checkpoint refused: restart required".into(),
                )));
            }
            let sequence = store.save_with_sequence(&self.engine, self.writer.applied_seq())?;
            store.prune(3)?;
            match store.disk_bytes() {
                Ok(bytes) => self.engine.metrics().set_segment_disk_bytes(bytes),
                Err(error) => tracing::warn!(%error, "segment disk metric unavailable after prune"),
            }
            if let Some(aof) = &self.aof {
                #[cfg(unix)]
                let trim = (|| {
                    let mut plan = {
                        let mut writer = aof
                            .lock()
                            .map_err(|_| anyhow::anyhow!("aof writer poisoned"))?;
                        writer.begin_trim(sequence)?
                    };
                    plan.copy_stable_prefix()?;
                    let mut writer = aof
                        .lock()
                        .map_err(|_| anyhow::anyhow!("aof writer poisoned"))?;
                    writer.finish_trim(plan)
                })();
                #[cfg(not(unix))]
                let trim = aof
                    .lock()
                    .map_err(|_| anyhow::anyhow!("aof writer poisoned"))
                    .and_then(|mut writer| writer.truncate_through(sequence));
                if let Err(error) = trim {
                    tracing::warn!(%error, "AOF trim after checkpoint failed");
                }
            }
            Ok(())
        })();
        if result.is_err() {
            attempt.mark_failed();
        }
        result
    }

    /// Start the periodic form of checkpoint_now. The budget only supplies
    /// wake hints; the same sink performs manual and periodic saves.
    #[doc(hidden)]
    pub fn spawn_periodic_driver(self: Arc<Self>, period: Duration) -> SegmentCheckpointDriver {
        self.spawn_driver_with_budget(period, ChangeBudget::process_shared())
    }

    fn spawn_driver_with_budget(
        self: Arc<Self>,
        period: Duration,
        budget: ChangeBudget,
    ) -> SegmentCheckpointDriver {
        self.spawn_driver_with_owner_kind(period, budget, true)
    }

    fn spawn_driver_with_owner_kind(
        self: Arc<Self>,
        period: Duration,
        budget: ChangeBudget,
        configured: bool,
    ) -> SegmentCheckpointDriver {
        let capacity_owner = crate::segment_capacity::Owner::start(self.clone(), configured)
            .expect("start native layer capacity owner");
        let periodic_fence = capacity_owner.as_ref().map(|owner| owner.fence());
        if capacity_owner.is_none() && !configured {
            return SegmentCheckpointDriver {
                stop: Arc::new(AtomicBool::new(false)),
                waiter: None,
                checkpoint_task: None,
                shutdown: None,
                capacity_owner: None,
            };
        }
        let wake = budget.checkpoint_wake();
        let (notices, mut notices_rx) = mpsc::channel(1);
        let stop = Arc::new(AtomicBool::new(false));
        let observed = wake.epoch();
        let waiter = spawn_budget_waiter(wake, notices, stop.clone(), observed);
        let (shutdown, mut shutdown_rx) = oneshot::channel();
        let task_stop = stop.clone();
        let checkpoint_task = tokio::spawn(async move {
            let mut schedule = CheckpointSchedule::new(period, Instant::now());
            loop {
                if task_stop.load(Ordering::Acquire) {
                    return;
                }
                // This pre-sleep sample makes initially high work visible even
                // if it predates creation of the waiter.
                let now = Instant::now();
                let pending = budget.snapshot();
                let work_revision = pending.work_revision;
                let owner_before = self.engine.capacity_owner_state();
                let successor_attempt = schedule.take_successor(owner_before);
                let ordinary_attempt = schedule.should_attempt(now, pending, work_revision);
                if successor_attempt || ordinary_attempt {
                    match self.checkpoint_with_fence(periodic_fence.clone()).await {
                        Ok(_) => {
                            let after = budget.snapshot();
                            let owner_after = self.engine.capacity_owner_state();
                            Self::complete_periodic_checkpoint(
                                &mut schedule,
                                after,
                                owner_before,
                                owner_after,
                            );
                        }
                        Err(error) => {
                            if crate::coordinator::is_storage_full(&error) {
                                self.engine.metrics().mark_storage_degraded();
                            }
                            tracing::warn!(error = %format!("{error:#}"), "periodic segment checkpoint failed");
                            // Failed checkpoints retain the normal period
                            // backoff and leave the request pending.
                            schedule.completed(Instant::now(), pending);
                        }
                    }
                    while notices_rx.try_recv().is_ok() {}
                    continue;
                }
                tokio::select! {
                    _ = tokio::time::sleep_until(schedule.next_deadline) => {}
                    _ = &mut shutdown_rx => return,
                    notice = notices_rx.recv() => {
                        if notice.is_none() {
                            return;
                        }
                    }
                }
            }
        });
        SegmentCheckpointDriver {
            stop,
            waiter: Some(waiter),
            checkpoint_task: Some(checkpoint_task),
            shutdown: Some(shutdown),
            capacity_owner,
        }
    }

}

fn spawn_budget_waiter(
    wake: Arc<BudgetWake>,
    notices: mpsc::Sender<()>,
    stop: Arc<AtomicBool>,
    mut observed: u64,
) -> std::thread::JoinHandle<()> {
    // Subscribe before the async task can sample pending work. Starting the
    // thread later must not swallow a change between that sample and sleep.
    std::thread::spawn(move || {
        while !stop.load(Ordering::Acquire) {
            let _ = wake.wait_for_change_timeout(observed, WAITER_SHUTDOWN_POLL);
            let current = wake.epoch();
            if current == observed {
                continue;
            }
            observed = current;
            match notices.try_send(()) {
                Ok(()) | Err(TrySendError::Full(_)) => {}
                Err(TrySendError::Closed(_)) => return,
            }
        }
    })
}

#[async_trait::async_trait]
impl crate::api::CheckpointSink for SegmentCheckpointSink {
    async fn checkpoint_now(&self) -> Result<bool> {
        self.checkpoint_with_fence(None).await
    }
}

/// A read-only `WriteSink` for bootstrap pending-change spill checkpoints.
///
/// It samples the Engine capture barrier because the coordinator may not exist
/// before AOF or Raft replay. `SegmentRdbStore` captures again during save, so
/// this fallback can never fabricate a later sequence.
#[doc(hidden)]
pub struct EngineWatermarkSink {
    engine: Arc<Engine>,
}

impl EngineWatermarkSink {
    #[doc(hidden)]
    pub fn new(engine: Arc<Engine>) -> Self {
        Self { engine }
    }
}

#[async_trait::async_trait]
impl crate::coordinator::WriteSink for EngineWatermarkSink {
    async fn submit(
        &self,
        _: crate::log_entry::RaftLogEntry,
    ) -> Result<crate::storage::ApplyOutcome> {
        anyhow::bail!("bootstrap checkpoint watermark is not a write sink")
    }

    fn applied_seq(&self) -> u64 {
        self.engine
            .capture_barrier
            .capture(0)
            .map(|lease| lease.stamp().sequence)
            .unwrap_or(0)
    }
}

static PENDING_SPILL_NONCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// A process-private root. It stays alive for the Engine lifetime because live
/// mmap readers and checkpoint lineage can still name files below it.
struct SpillDirectory {
    path: std::path::PathBuf,
}

impl SpillDirectory {
    fn create() -> Result<Self> {
        let parent = std::env::temp_dir();
        let process = std::process::id();
        for _ in 0..128 {
            let nonce = PENDING_SPILL_NONCE.fetch_add(1, Ordering::Relaxed);
            let path = parent.join(format!("lumen-pending-spill-{process}-{nonce}"));
            let mut builder = std::fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            match builder.create(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error).context("create pending spill root"),
            }
        }
        anyhow::bail!("could not allocate a unique pending spill root")
    }
}

pub(crate) fn temporary_spill_store() -> Result<Arc<crate::segment_rdb::SegmentRdbStore>> {
    let root = Arc::new(SpillDirectory::create()?);
    Ok(Arc::new(
        crate::segment_rdb::SegmentRdbStore::new(&root.path)?.with_root_guard(root),
    ))
}

impl Drop for SpillDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// Owns a private spill root plus an early budget driver. It never becomes the
/// public checkpoint sink and is never read on a later process start.
#[doc(hidden)]
pub struct PendingChangeSpill {
    sink: Arc<SegmentCheckpointSink>,
    driver: Option<SegmentCheckpointDriver>,
    temporary_root: Option<Arc<SpillDirectory>>,
}

impl PendingChangeSpill {
    #[doc(hidden)]
    pub fn temporary(engine: Arc<Engine>, period: Duration) -> Result<Self> {
        let root = Arc::new(SpillDirectory::create()?);
        let store = Arc::new(
            crate::segment_rdb::SegmentRdbStore::new(&root.path)?.with_root_guard(root.clone()),
        );
        Ok(Self::start(engine, store, period, Some(root)))
    }

    /// Uses the configured segment root during startup replay. AOF is omitted,
    /// so this bootstrap path cannot trim the replay source.
    #[doc(hidden)]
    pub fn configured_replay(
        engine: Arc<Engine>,
        store: Arc<crate::segment_rdb::SegmentRdbStore>,
        period: Duration,
    ) -> Self {
        Self::start(engine, store, period, None)
    }

    fn start(
        engine: Arc<Engine>,
        store: Arc<crate::segment_rdb::SegmentRdbStore>,
        period: Duration,
        temporary_root: Option<Arc<SpillDirectory>>,
    ) -> Self {
        let sink = Arc::new(SegmentCheckpointSink {
            writer: Arc::new(EngineWatermarkSink::new(engine.clone())),
            engine,
            store,
            aof: None,
        });
        let driver = sink.clone().spawn_driver_with_owner_kind(
            period,
            ChangeBudget::process_shared(),
            temporary_root.is_none(),
        );
        Self {
            sink,
            driver: Some(driver),
            temporary_root,
        }
    }

    /// The caller must await this before starting another driver for the same
    /// configured root. It keeps the private root alive.
    #[doc(hidden)]
    pub async fn stop_bootstrap(&mut self) -> Result<()> {
        if let Some(driver) = &mut self.driver {
            driver.shutdown().await?;
        }
        self.driver.take();
        Ok(())
    }

    #[doc(hidden)]
    pub fn store(&self) -> &Arc<crate::segment_rdb::SegmentRdbStore> {
        &self.sink.store
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aof::AofWriter;
    use crate::api::CheckpointSink;
    use crate::coordinator::WriteSink;
    use crate::log_entry::RaftLogEntry;
    use crate::types::{
        CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
    };
    use crate::wal::WalRecord;
    use std::collections::BTreeMap;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Mutex;

    #[test]
    fn periodic_success_bookkeeping_does_not_consume_capacity_request() {
        let budget = crate::change_budget::ChangeBudget::with_hard_limit(
            crate::change_budget::CHECKPOINT_TRIGGER * 2,
        );
        let engine = Arc::new(Engine::with_change_budget(budget.clone()));
        engine
            .create_collection(
                "docs",
                serde_json::from_value(serde_json::json!({
                    "fields": {"kw":{"type":"keyword"}}
                }))
                .unwrap(),
            )
            .unwrap();
        engine
            .index(
                "docs",
                serde_json::from_value(serde_json::json!({
                    "items":[{"external_id":"one","field":"kw","value":"retained"}]
                }))
                .unwrap(),
            )
            .unwrap();
        let _held = budget
            .owner()
            .try_reserve(crate::change_budget::CHECKPOINT_TRIGGER)
            .unwrap()
            .commit_retained()
            .unwrap();
        engine.request_pending_checkpoint();
        let request_revision = engine
            .capacity_owner_state()
            .and_then(|state| state.checkpoint_request_revision)
            .expect("capacity request revision");
        let root = tempfile::tempdir().unwrap();
        let store = crate::segment_rdb::SegmentRdbStore::new(root.path()).unwrap();
        let sink = SegmentCheckpointSink {
            engine: engine.clone(),
            store: Arc::new(store),
            writer: Arc::new(EngineWatermarkSink::new(engine.clone())),
            aof: None,
        };
        sink.checkpoint_sync(&sink.store).unwrap();
        let mut schedule = CheckpointSchedule::new(Duration::from_secs(1), Instant::now());
        let before = engine.capacity_owner_state();
        SegmentCheckpointSink::complete_periodic_checkpoint(
            &mut schedule,
            budget.snapshot(),
            before,
            engine.capacity_owner_state(),
        );
        assert!(engine
            .capacity_owner_state()
            .and_then(|state| state.checkpoint_request_revision)
            .is_some_and(|revision| revision == request_revision));
    }

    #[tokio::test]
    async fn shutdown_graph_cache_preserves_current_and_durable_aof_tail() {
        let root = tempfile::tempdir().unwrap();
        let engine = Arc::new(Engine::new());
        let store = Arc::new(crate::segment_rdb::SegmentRdbStore::new(root.path()).unwrap());
        let tail = root.path().join("aof.log");
        let aof = Arc::new(Mutex::new(AofWriter::open(&tail).unwrap()));
        let writer = crate::coordinator::WriteCoordinator::start_from_with_aof(
            Arc::new(crate::wal::MemWal::new()), engine.clone(), 0, aof.clone(),
        );
        writer.submit(RaftLogEntry::CreateCollection {
            collection_id: "v".into(),
            req: serde_json::from_value(serde_json::json!({
                "fields": {"v": {"type": "vector", "dim": 3, "metric": "l2", "backend": "hnsw-cpu"}}
            })).unwrap(),
        }).await.unwrap();
        let sink = Arc::new(SegmentCheckpointSink {
            engine, store, writer: writer.clone(), aof: Some(aof.clone()),
        });
        sink.checkpoint_now().await.unwrap();
        let current = std::fs::read(root.path().join("CURRENT")).unwrap();
        writer.submit(RaftLogEntry::Index {
            collection_id: "v".into(),
            req: serde_json::from_value(serde_json::json!({"items": [{
                "external_id": "tail", "field": "v", "value": [1.0, 2.0, 3.0]
            }]})).unwrap(),
        }).await.unwrap();
        aof.lock().unwrap().sync().unwrap();
        let before = std::fs::read(&tail).unwrap();
        assert!(!before.is_empty());
        assert_eq!(sink.save_shutdown_graph_cache().await.unwrap(), 1);
        assert_eq!(std::fs::read(root.path().join("CURRENT")).unwrap(), current,
            "shutdown cache must not rewrite CURRENT when a durable AOF exists");
        assert_eq!(std::fs::read(tail).unwrap(), before, "cache must not trim the authoritative tail");
    }

    #[tokio::test]
    async fn shutdown_cache_wait_for_inflight_writes_can_be_cancelled() {
        let root = tempfile::tempdir().unwrap();
        let engine = Arc::new(Engine::new());
        let writer = crate::coordinator::WriteCoordinator::start(
            Arc::new(crate::wal::MemWal::new()), engine.clone(),
        );
        let gate = writer.mutation_gate();
        let in_flight = gate.shared().await.unwrap();
        let sink = Arc::new(SegmentCheckpointSink {
            engine,
            store: Arc::new(crate::segment_rdb::SegmentRdbStore::new(root.path()).unwrap()),
            writer, aof: None,
        });
        assert!(tokio::time::timeout(Duration::from_millis(25), sink.clone().save_shutdown_graph_cache()).await.is_err());
        assert!(!root.path().join("hnsw-graph-cache").exists());
        drop(in_flight);
        assert_eq!(tokio::time::timeout(Duration::from_secs(2), sink.save_shutdown_graph_cache()).await.unwrap().unwrap(), 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dropping_private_spill_keeps_root_while_blocking_save_owns_store() {
        struct HoldWrite {
            armed: AtomicBool,
            entered: std::sync::mpsc::Sender<()>,
            released: (Mutex<bool>, std::sync::Condvar),
        }
        impl storage_durable::FailureInjector for HoldWrite {
            fn check(&self, point: &storage_durable::FailurePoint) -> std::io::Result<()> {
                if point.step == storage_durable::CommitStep::SyncFile
                    && self.armed.swap(false, Ordering::AcqRel)
                {
                    self.entered.send(()).unwrap();
                    let (lock, wake) = &self.released;
                    let mut released = lock.lock().unwrap();
                    while !*released {
                        released = wake.wait(released).unwrap();
                    }
                }
                Ok(())
            }
        }
        struct ReleaseWrite(Arc<HoldWrite>);
        impl Drop for ReleaseWrite {
            fn drop(&mut self) {
                *self.0.released.0.lock().unwrap() = true;
                self.0.released.1.notify_all();
            }
        }

        let budget = ChangeBudget::with_hard_limit(8 * 1024 * 1024);
        let engine = Arc::new(Engine::with_change_budget(budget.clone()));
        engine
            .create_collection("captured", spill_keyword_schema())
            .unwrap();
        admitted_keyword(&engine, "kept", "kept");
        let (entered_tx, entered_rx) = std::sync::mpsc::channel();
        let hold = Arc::new(HoldWrite {
            armed: AtomicBool::new(true),
            entered: entered_tx,
            released: (Mutex::new(false), std::sync::Condvar::new()),
        });
        let release = ReleaseWrite(hold.clone());
        let root = Arc::new(SpillDirectory::create().unwrap());
        let root_path = root.path.clone();
        let sink = {
            let store = Arc::new(
                crate::segment_rdb::SegmentRdbStore::new_with_failure_injector(&root.path, hold)
                    .unwrap()
                    .with_root_guard(root.clone()),
            );
            Arc::new(SegmentCheckpointSink {
                engine: engine.clone(),
                store,
                writer: Arc::new(EngineWatermarkSink::new(engine.clone())),
                aof: None,
            })
        };
        let running_store = Arc::downgrade(&sink.store);
        let driver = sink
            .clone()
            // This test verifies shutdown ownership after a write starts. A
            // zero period starts the periodic path immediately; request
            // revisions intentionally do not bypass the high-water schedule.
            .spawn_driver_with_budget(Duration::ZERO, budget);
        let spill = PendingChangeSpill {
            sink,
            driver: Some(driver),
            temporary_root: Some(root),
        };
        engine.request_pending_checkpoint();
        tokio::task::spawn_blocking(move || entered_rx.recv_timeout(Duration::from_secs(5)))
            .await
            .unwrap()
            .expect("save must reach the injected blocking write");

        drop(spill);
        assert!(
            root_path.exists(),
            "the blocking store clone pins its private root"
        );
        // The final store Arc's strong count reaches zero before its destructor
        // necessarily finishes dropping `root_guard` on the blocking worker.
        // One deadline covers both that drain and final root reclamation.
        let completion_deadline = Instant::now() + Duration::from_secs(5);
        drop(release);
        tokio::time::timeout_at(completion_deadline, async {
            while running_store.strong_count() > 0 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("aborted driver must let the started blocking save finish and release its store");
        assert!(contains_keyword(&engine, "kept"));
        assert!(
            root_path.join("CURRENT").is_file(),
            "blocking save must finish durable publication after driver drop"
        );
        assert!(
            root_path.exists(),
            "the live engine pins installed reader files"
        );
        drop(engine);
        tokio::time::timeout_at(completion_deadline, async {
            while root_path.exists() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("last store/engine owner must reclaim its private root within the shared deadline");
        assert!(
            !root_path.exists(),
            "last store/engine owner releases only its root"
        );
    }

    struct FailOnce(Mutex<Option<storage_durable::CommitStep>>);

    impl storage_durable::FailureInjector for FailOnce {
        fn check(&self, point: &storage_durable::FailurePoint) -> std::io::Result<()> {
            let mut armed = self.0.lock().unwrap();
            if *armed == Some(point.step) {
                *armed = None;
                return Err(std::io::Error::other("injected pending spill failure"));
            }
            Ok(())
        }
    }

    struct Watermark(AtomicU64);
    #[async_trait::async_trait]
    impl WriteSink for Watermark {
        async fn submit(
            &self,
            _: crate::log_entry::RaftLogEntry,
        ) -> Result<crate::storage::ApplyOutcome> {
            anyhow::bail!("checkpoint test has no publisher")
        }
        fn applied_seq(&self) -> u64 {
            self.0.load(Ordering::SeqCst)
        }
    }

    fn spill_keyword_schema() -> CreateCollectionRequest {
        let mut fields = BTreeMap::new();
        fields.insert(
            "email".to_owned(),
            FieldSpec {
                field_type: FieldType::Keyword,
                analyzer: None,
                multi: None,
                dim: None,
                metric: None,
                backend: None,
                quantize: None,
            },
        );
        CreateCollectionRequest { fields }
    }

    fn admitted_keyword(engine: &Engine, external_id: &str, value: &str) {
        let entry = RaftLogEntry::Index {
            collection_id: "captured".to_owned(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: external_id.to_owned(),
                    field: "email".to_owned(),
                    value: FieldValue::String(value.to_owned()),
                    version: None,
                }],
                request_id: None,
            },
        };
        let reservation = engine.try_reserve_record(&entry, 0).unwrap();
        let mut guard = engine
            .begin_admitted_record(entry, reservation)
            .unwrap_or_else(|failure| panic!("admit fixture: {:?}", failure.error));
        engine.apply_prepared_raft_entry(&mut guard).unwrap();
    }

    fn contains_keyword(engine: &Engine, value: &str) -> bool {
        let request = serde_json::from_value(serde_json::json!({
            "query": {"term": {"field": "email", "value": value}},
            "limit": 10
        }))
        .unwrap();
        !engine.search("captured", request).unwrap().hits.is_empty()
    }

    #[test]
    fn bootstrap_watermark_reads_engine_capture_sequence() {
        let engine = Arc::new(Engine::new());
        let apply = engine.capture_barrier.apply();
        apply.initialize_sequence(41);
        drop(apply);
        assert_eq!(EngineWatermarkSink::new(engine).applied_seq(), 41);
    }

    #[tokio::test]
    async fn private_spill_root_outlives_handle_store_and_live_cold_engines() {
        let engine = Arc::new(Engine::new());
        engine
            .create_collection("captured", spill_keyword_schema())
            .unwrap();
        admitted_keyword(&engine, "kept", "kept");
        let mut spill =
            PendingChangeSpill::temporary(engine.clone(), Duration::from_secs(3600)).unwrap();
        let root = spill.temporary_root.as_ref().unwrap().path.clone();
        assert!(spill.sink.checkpoint_now().await.unwrap());
        let escaped = spill.store().clone();
        let (cold, _) = escaped.load_latest().unwrap().unwrap();
        spill.stop_bootstrap().await.unwrap();
        drop(spill);
        assert!(root.exists(), "escaped store must retain the private root");
        assert!(contains_keyword(&engine, "kept"));
        assert!(contains_keyword(&cold, "kept"));
        drop(escaped);
        assert!(root.exists(), "live engines must retain their reader paths");
        drop(engine);
        assert!(
            root.exists(),
            "cold engine must retain its installed root guard"
        );
        drop(cold);
        assert!(
            !root.exists(),
            "last reader/store drop must reclaim private root"
        );
    }

    #[tokio::test]
    async fn failed_bootstrap_spill_retries_the_frozen_cut() {
        let root = tempfile::tempdir().unwrap();
        let budget = ChangeBudget::with_hard_limit(8 * 1024 * 1024);
        let engine = Arc::new(Engine::with_change_budget(budget.clone()));
        engine
            .create_collection("captured", spill_keyword_schema())
            .unwrap();
        admitted_keyword(&engine, "old", "old");
        let baseline = Arc::new(crate::segment_rdb::SegmentRdbStore::new(root.path()).unwrap());
        baseline.save_with_sequence(&engine, 1).unwrap();
        let apply = engine.capture_barrier.apply();
        apply.initialize_sequence(1);
        drop(apply);
        admitted_keyword(&engine, "captured", "captured");
        let frozen_before_failure = budget.snapshot().active;
        assert!(
            frozen_before_failure > 0,
            "admitted captured row must own pending bytes"
        );
        let store = Arc::new(
            crate::segment_rdb::SegmentRdbStore::new_with_failure_injector(
                root.path(),
                Arc::new(FailOnce(Mutex::new(Some(
                    storage_durable::CommitStep::SyncFile,
                )))),
            )
            .unwrap(),
        );
        let sink = SegmentCheckpointSink {
            engine: engine.clone(),
            store: store.clone(),
            writer: Arc::new(EngineWatermarkSink::new(engine.clone())),
            aof: None,
        };
        assert!(sink.checkpoint_now().await.is_err());
        assert_eq!(engine.metrics().segment_checkpoint_started_total.get(), 1);
        assert_eq!(engine.metrics().segment_checkpoint_failed_total.get(), 1);
        assert_eq!(engine.metrics().segment_checkpoint_in_flight.get(), 0);
        assert!(
            budget.snapshot().frozen >= frozen_before_failure,
            "failed save must retain the captured charge"
        );
        admitted_keyword(&engine, "newer", "newer");
        assert!(sink.checkpoint_now().await.unwrap());
        assert_eq!(engine.metrics().segment_checkpoint_started_total.get(), 2);
        assert_eq!(engine.metrics().segment_checkpoint_failed_total.get(), 1);
        assert_eq!(engine.metrics().segment_checkpoint_in_flight.get(), 0);
        let (replayed, sequence) = sink.store.load_latest().unwrap().unwrap();
        assert_eq!(sequence, 1);
        assert!(contains_keyword(&replayed, "captured"));
        assert!(!contains_keyword(&replayed, "newer"));
        assert!(
            budget.snapshot().active > 0,
            "newer post-failure row remains charged"
        );
    }

    #[tokio::test]
    async fn bootstrap_driver_high_budget_uses_capture_sequence_without_coordinator() {
        let budget = ChangeBudget::with_hard_limit(crate::change_budget::CHECKPOINT_TRIGGER + 1);
        let engine = Arc::new(Engine::with_change_budget(budget.clone()));
        let apply = engine.capture_barrier.apply();
        apply.initialize_sequence(23);
        drop(apply);
        let root = tempfile::tempdir().unwrap();
        let store = Arc::new(crate::segment_rdb::SegmentRdbStore::new(root.path()).unwrap());
        let sink = Arc::new(SegmentCheckpointSink {
            engine: engine.clone(),
            store: store.clone(),
            writer: Arc::new(EngineWatermarkSink::new(engine.clone())),
            aof: None,
        });
        let owner = budget.owner();
        let _charge = owner
            .try_reserve(crate::change_budget::CHECKPOINT_TRIGGER)
            .unwrap()
            .commit_retained()
            .unwrap();
        // Start the write through the periodic path immediately. Capacity
        // request revisions remain wake hints and do not bypass scheduling.
        let mut driver = sink.spawn_driver_with_budget(Duration::ZERO, budget);
        wait_for_checkpoint_count(&engine, 1).await;
        assert_eq!(
            store.load_current_generation().unwrap().unwrap().sequence,
            23
        );
        driver.shutdown().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn graceful_shutdown_waits_for_the_started_checkpoint_write() {
        struct HoldWrite {
            armed: AtomicBool,
            entered: std::sync::mpsc::Sender<()>,
            released: (Mutex<bool>, std::sync::Condvar),
        }
        impl storage_durable::FailureInjector for HoldWrite {
            fn check(&self, point: &storage_durable::FailurePoint) -> std::io::Result<()> {
                if point.step == storage_durable::CommitStep::SyncFile
                    && self.armed.swap(false, Ordering::AcqRel)
                {
                    self.entered.send(()).unwrap();
                    let (lock, wake) = &self.released;
                    let mut released = lock.lock().unwrap();
                    while !*released {
                        released = wake.wait(released).unwrap();
                    }
                }
                Ok(())
            }
        }
        struct ReleaseWrite(Arc<HoldWrite>);
        impl Drop for ReleaseWrite {
            fn drop(&mut self) {
                *self.0.released.0.lock().unwrap() = true;
                self.0.released.1.notify_all();
            }
        }
        let root = tempfile::tempdir().unwrap();
        let budget = ChangeBudget::with_hard_limit(8 * 1024 * 1024);
        let engine = Arc::new(Engine::with_change_budget(budget.clone()));
        engine
            .create_collection("captured", spill_keyword_schema())
            .unwrap();
        admitted_keyword(&engine, "kept", "kept");
        let (entered_tx, entered_rx) = std::sync::mpsc::channel();
        let hold = Arc::new(HoldWrite {
            armed: AtomicBool::new(false),
            entered: entered_tx,
            released: (Mutex::new(false), std::sync::Condvar::new()),
        });
        let store = Arc::new(
            crate::segment_rdb::SegmentRdbStore::new_with_failure_injector(
                root.path(),
                hold.clone(),
            )
            .unwrap(),
        );
        // Establish a durable predecessor first, then create a real changed
        // checkpoint payload. This makes the injected SyncFile pause belong
        // to an actual SegmentRdbStore write.
        store.save(&engine, 1).unwrap();
        admitted_keyword(&engine, "after-base", "after-base");
        let successor_apply = engine.capture_barrier.apply();
        successor_apply.initialize_sequence(2);
        drop(successor_apply);
        let release = ReleaseWrite(hold.clone());
        hold.armed.store(true, Ordering::Release);
        let sink = Arc::new(SegmentCheckpointSink {
            engine: engine.clone(),
            store: store.clone(),
            writer: Arc::new(EngineWatermarkSink::new(engine.clone())),
            aof: None,
        });
        // Force the periodic checkpoint path to observe the real write. The
        // request remains a wake hint; the zero period makes the first
        // scheduled attempt deterministic for this ownership test.
        let mut driver = sink.spawn_driver_with_budget(Duration::ZERO, budget);
        engine.request_pending_checkpoint();
        tokio::task::spawn_blocking(move || entered_rx.recv_timeout(Duration::from_secs(5)))
            .await
            .unwrap()
            .expect("checkpoint must reach the real write pause");
        assert_eq!(engine.metrics().segment_checkpoint_started_total.get(), 1);
        assert_eq!(engine.metrics().segment_checkpoint_failed_total.get(), 0);
        assert_eq!(engine.metrics().segment_checkpoint_in_flight.get(), 1);
        let stop = driver.stop.clone();
        let (finished_tx, mut finished_rx) = oneshot::channel();
        let stopping = tokio::spawn(async move {
            let result = driver.shutdown().await;
            let _ = finished_tx.send(result);
        });
        tokio::time::timeout(Duration::from_secs(5), async {
            while !stop.load(Ordering::Acquire) {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("shutdown must request stop");
        assert!(
            tokio::time::timeout(Duration::from_millis(200), &mut finished_rx)
                .await
                .is_err(),
            "graceful shutdown must not finish while its checkpoint write is paused"
        );
        drop(release);
        tokio::time::timeout(Duration::from_secs(5), finished_rx)
            .await
            .expect("shutdown must finish after write release")
            .unwrap()
            .unwrap();
        stopping.await.unwrap();
        assert_eq!(engine.metrics().segment_checkpoint_started_total.get(), 1);
        assert_eq!(engine.metrics().segment_checkpoint_failed_total.get(), 0);
        assert_eq!(engine.metrics().segment_checkpoint_in_flight.get(), 0);
        let (cold, _) = store
            .load_latest()
            .unwrap()
            .expect("started checkpoint must finish");
        assert!(contains_keyword(&cold, "kept"));
    }

    #[tokio::test]
    async fn configured_replay_spill_checkpoints_but_never_trims_aof_tail() {
        let root = tempfile::tempdir().unwrap();
        let tail = root.path().join("aof.log");
        let mut aof = AofWriter::open(&tail).unwrap();
        aof.append(
            7,
            &WalRecord::new(RaftLogEntry::DropCollection {
                collection_id: "sentinel".to_owned(),
                force: false,
            }),
        )
        .unwrap();
        aof.sync().unwrap();
        let before = std::fs::read(&tail).unwrap();
        let engine = Arc::new(Engine::new());
        let apply = engine.capture_barrier.apply();
        apply.initialize_sequence(7);
        drop(apply);
        let store = Arc::new(crate::segment_rdb::SegmentRdbStore::new(root.path()).unwrap());
        let mut spill =
            PendingChangeSpill::configured_replay(engine, store.clone(), Duration::from_secs(3600));
        assert!(spill.temporary_root.is_none());
        assert!(spill.sink.aof.is_none());
        assert!(spill.sink.checkpoint_now().await.unwrap());
        assert_eq!(
            store.load_current_generation().unwrap().unwrap().sequence,
            7
        );
        assert_eq!(
            std::fs::read(&tail).unwrap(),
            before,
            "bootstrap save must not trim replay tail"
        );
        spill.stop_bootstrap().await.unwrap();
        assert!(root.path().exists());
    }

    #[tokio::test]
    async fn checkpoint_sink_disk_gauge_matches_post_prune_files() {
        let root = tempfile::tempdir().unwrap();
        let engine = Arc::new(Engine::new());
        engine
            .create_collection(
                "u",
                serde_json::from_value(serde_json::json!({
                    "fields": { "email": { "type": "keyword" } }
                }))
                .unwrap(),
            )
            .unwrap();
        let writer = Arc::new(Watermark(AtomicU64::new(0)));
        let store = Arc::new(crate::segment_rdb::SegmentRdbStore::new(root.path()).unwrap());
        let sink = SegmentCheckpointSink {
            engine: engine.clone(),
            store: store.clone(),
            writer: writer.clone(),
            aof: None,
        };
        for sequence in 1..=6 {
            writer.0.store(sequence, Ordering::SeqCst);
            assert!(sink.checkpoint_now().await.unwrap());
        }
        let generations = std::fs::read_dir(root.path())
            .unwrap()
            .map(Result::unwrap)
            .filter(|entry| entry.file_name().to_string_lossy().starts_with("gen-"))
            .count();
        assert_eq!(
            generations, 3,
            "fixture must actually prune retained generations"
        );
        assert_eq!(
            engine.metrics().segment_disk_bytes.get(),
            store.disk_bytes().unwrap(),
            "checkpoint sink must refresh disk usage after pruning"
        );
    }

    fn snapshot(
        total: usize,
        work_revision: u64,
        checkpoint_request_revision: Option<u64>,
    ) -> Snapshot {
        Snapshot {
            reserved: 0,
            active: total,
            frozen: 0,
            total,
            work_revision,
            checkpoint_request_revision,
        }
    }

    #[test]
    fn initial_high_work_runs_before_the_first_deadline() {
        let now = Instant::now();
        let mut schedule = CheckpointSchedule::new(Duration::from_secs(30), now);
        assert!(schedule.should_attempt(
            now,
            snapshot(crate::change_budget::CHECKPOINT_TRIGGER, 1, None),
            1
        ));
    }

    #[test]
    fn high_work_does_not_recheckpoint_for_every_new_revision() {
        let now = Instant::now();
        let mut schedule = CheckpointSchedule::new(Duration::from_secs(30), now);
        let high = snapshot(crate::change_budget::CHECKPOINT_TRIGGER, 1, None);
        assert!(schedule.should_attempt(now, high, 1));
        schedule.completed(now + Duration::from_secs(2), high);
        // New work remains covered by the completed high-water checkpoint.
        assert!(!schedule.should_attempt(
            now + Duration::from_secs(2),
            snapshot(crate::change_budget::CHECKPOINT_TRIGGER, 2, None),
            2,
        ));
        assert!(!schedule.should_attempt(now + Duration::from_secs(2), snapshot(0, 2, None), 2,));
        assert!(schedule.should_attempt(
            now + Duration::from_secs(2),
            snapshot(crate::change_budget::CHECKPOINT_TRIGGER, 3, None),
            3,
        ));
    }

    #[test]
    fn transient_dip_below_trigger_does_not_rearm_high_work() {
        let now = Instant::now();
        let mut schedule = CheckpointSchedule::new(Duration::from_secs(30), now);
        let high = snapshot(crate::change_budget::CHECKPOINT_TRIGGER, 1, None);
        assert!(schedule.should_attempt(now, high, 1));
        schedule.completed(now + Duration::from_secs(1), high);
        assert!(!schedule.should_attempt(
            now + Duration::from_secs(1),
            snapshot(CHECKPOINT_REARM_THRESHOLD + 1, 2, None),
            2,
        ));
        assert!(!schedule.should_attempt(
            now + Duration::from_secs(1),
            snapshot(crate::change_budget::CHECKPOINT_TRIGGER, 3, None),
            3,
        ));
        assert!(!schedule.should_attempt(
            now + Duration::from_secs(1),
            snapshot(CHECKPOINT_REARM_THRESHOLD - 1, 4, None),
            4,
        ));
        assert!(schedule.should_attempt(now + Duration::from_secs(1), high, 5,));
    }

    #[test]
    fn newer_request_revision_bypasses_high_water_period_once() {
        let now = Instant::now();
        let mut schedule = CheckpointSchedule::new(Duration::from_secs(30), now);
        let first = snapshot(crate::change_budget::CHECKPOINT_TRIGGER, 10, Some(10));
        assert!(schedule.should_attempt(now, first, 10));
        schedule.completed(now + Duration::from_secs(1), first);

        // A newer blocked-admission request proves that the prior checkpoint
        // did not create enough headroom. It must trigger one more attempt
        // without waiting for the normal period.
        assert!(schedule.should_attempt(
            now + Duration::from_secs(1),
            snapshot(crate::change_budget::CHECKPOINT_TRIGGER, 11, Some(11)),
            11,
        ));
        schedule.completed(
            now + Duration::from_secs(2),
            snapshot(crate::change_budget::CHECKPOINT_TRIGGER, 11, Some(11)),
        );
        assert!(!schedule.should_attempt(
            now + Duration::from_secs(31),
            snapshot(crate::change_budget::CHECKPOINT_TRIGGER, 12, Some(11)),
            12,
        ));
        schedule.completed(
            now + Duration::from_secs(32),
            snapshot(crate::change_budget::CHECKPOINT_TRIGGER, 12, Some(11)),
        );
        assert!(!schedule.should_attempt(
            now + Duration::from_secs(32),
            snapshot(crate::change_budget::CHECKPOINT_TRIGGER, 13, Some(11)),
            13,
        ));

        // A real drain re-arms immediate capacity relief for the next crossing.
        assert!(!schedule.should_attempt(
            now + Duration::from_secs(2),
            snapshot(CHECKPOINT_REARM_THRESHOLD - 1, 13, None),
            13,
        ));
        assert!(schedule.should_attempt(
            now + Duration::from_secs(2),
            snapshot(crate::change_budget::CHECKPOINT_TRIGGER, 14, Some(14)),
            14,
        ));
    }

    #[test]
    fn successful_high_checkpoint_immediately_retries_new_publishable_work() {
        let now = Instant::now();
        let mut schedule = CheckpointSchedule::new(Duration::from_secs(3600), now);
        let before = snapshot(crate::change_budget::CHECKPOINT_TRIGGER, 10, Some(10));
        assert!(schedule.should_attempt(now, before, 10));
        schedule.completed_success(
            now + Duration::from_secs(1),
            snapshot(crate::change_budget::CHECKPOINT_TRIGGER + 1, 11, Some(11)),
            Some(crate::change_budget::OwnerCapacityState {
                active: 0,
                frozen: 0,
                work_revision: 10,
                checkpoint_request_revision: None,
            }),
            Some(crate::change_budget::OwnerCapacityState {
                active: crate::change_budget::CHECKPOINT_TRIGGER + 1,
                frozen: 0,
                work_revision: 11,
                checkpoint_request_revision: Some(11),
            }),
        );
        let successor_owner = Some(crate::change_budget::OwnerCapacityState {
            active: crate::change_budget::CHECKPOINT_TRIGGER + 1,
            frozen: 0,
            work_revision: 11,
            checkpoint_request_revision: Some(11),
        });
        assert!(schedule.take_successor(successor_owner));
        assert!(!schedule.take_successor(successor_owner));
    }

    #[test]
    fn successful_successors_keep_new_high_work_runnable_without_a_refusal() {
        let now = Instant::now();
        let mut schedule = CheckpointSchedule::new(Duration::from_secs(3600), now);
        let high = crate::change_budget::CHECKPOINT_TRIGGER + 1;
        assert!(schedule.should_attempt(now, snapshot(high, 10, None), 10));

        for revision in 11..=13 {
            let before = crate::change_budget::OwnerCapacityState {
                active: high,
                frozen: 0,
                work_revision: revision - 1,
                checkpoint_request_revision: None,
            };
            let after = crate::change_budget::OwnerCapacityState {
                work_revision: revision,
                ..before
            };
            schedule.completed_success(
                now + Duration::from_secs(revision - 10),
                snapshot(high, revision, None),
                Some(before),
                Some(after),
            );
            assert!(
                schedule.take_successor(Some(after)),
                "new high work after publication {revision} must not wait for a 429 to request relief"
            );
            assert!(
                !schedule.take_successor(Some(after)),
                "one publication must arm only one successor"
            );
        }

        // A high process total is not enough: unchanged owner work must never
        // form a busy loop, even after a chain of successful publications.
        let unchanged = crate::change_budget::OwnerCapacityState {
            active: high,
            frozen: 0,
            work_revision: 13,
            checkpoint_request_revision: None,
        };
        schedule.completed_success(
            now + Duration::from_secs(4),
            snapshot(high, 13, None),
            Some(unchanged),
            Some(unchanged),
        );
        assert!(!schedule.take_successor(Some(unchanged)));
        assert!(!schedule.should_attempt(
            now + Duration::from_secs(4),
            snapshot(high, 13, None),
            13,
        ));
    }

    #[test]
    fn successful_checkpoint_below_trigger_rearms_next_crossing() {
        let now = Instant::now();
        let mut schedule = CheckpointSchedule::new(Duration::from_secs(3600), now);
        let before = snapshot(crate::change_budget::CHECKPOINT_TRIGGER, 10, Some(10));
        assert!(schedule.should_attempt(now, before, 10));
        schedule.completed_success(
            now + Duration::from_secs(1),
            snapshot(crate::change_budget::CHECKPOINT_TRIGGER - 1, 10, Some(10)),
            Some(crate::change_budget::OwnerCapacityState {
                active: 0,
                frozen: 0,
                work_revision: 10,
                checkpoint_request_revision: None,
            }),
            Some(crate::change_budget::OwnerCapacityState {
                active: 0,
                frozen: 0,
                work_revision: 10,
                checkpoint_request_revision: Some(10),
            }),
        );
        assert!(schedule.should_attempt(
            now + Duration::from_secs(1),
            snapshot(crate::change_budget::CHECKPOINT_TRIGGER, 11, Some(11)),
            11,
        ));
    }

    #[test]
    fn new_owner_work_crossing_after_publication_is_not_hidden_by_reservations() {
        let now = Instant::now();
        let trigger = crate::change_budget::CHECKPOINT_TRIGGER;
        let mut schedule = CheckpointSchedule::new(Duration::from_secs(3600), now);
        assert!(schedule.should_attempt(now, snapshot(trigger, 10, None), 10));
        let before = crate::change_budget::OwnerCapacityState {
            active: trigger,
            frozen: 0,
            work_revision: 10,
            checkpoint_request_revision: None,
        };
        let after = crate::change_budget::OwnerCapacityState {
            active: trigger * 3 / 4,
            work_revision: 11,
            ..before
        };
        // A successful publication left newer owner work below the trigger.
        // In-flight request reservations keep the process total above it.
        let pending = Snapshot {
            reserved: trigger / 2,
            active: after.active,
            frozen: 0,
            total: trigger * 5 / 4,
            work_revision: 11,
            checkpoint_request_revision: None,
        };
        schedule.completed_success(now, pending, Some(before), Some(after));
        assert!(
            !schedule.take_successor(Some(after)),
            "reserved work is not a capture target"
        );
        assert!(
            !schedule.should_attempt(now, pending, 11),
            "do not publish just for reservations"
        );

        let crossed = crate::change_budget::OwnerCapacityState {
            active: trigger,
            work_revision: 12,
            ..after
        };
        assert!(
            schedule.take_successor(Some(crossed)),
            "new publishable owner work must cross the unchanged trigger without waiting for a refusal or the normal period"
        );
        assert!(
            !schedule.take_successor(Some(crossed)),
            "one publication permits only one successor"
        );
    }

    #[test]
    fn reservations_after_a_complete_owner_drain_do_not_hide_new_work() {
        let now = Instant::now();
        let trigger = crate::change_budget::CHECKPOINT_TRIGGER;
        let mut schedule = CheckpointSchedule::new(Duration::from_secs(3600), now);
        let before = crate::change_budget::OwnerCapacityState {
            active: trigger,
            frozen: 0,
            work_revision: 10,
            checkpoint_request_revision: None,
        };
        let drained = crate::change_budget::OwnerCapacityState {
            active: 0,
            ..before
        };
        let reserved = Snapshot {
            reserved: trigger,
            active: 0,
            frozen: 0,
            total: trigger,
            work_revision: 10,
            checkpoint_request_revision: None,
        };
        schedule.completed_success(now, reserved, Some(before), Some(drained));
        for _ in 0..3 {
            assert!(!schedule.take_successor(Some(drained)));
            assert!(!schedule.should_attempt(now, reserved, 10));
        }
        let new_work = crate::change_budget::OwnerCapacityState {
            active: trigger,
            work_revision: 11,
            ..drained
        };
        assert!(
            schedule.take_successor(Some(new_work)),
            "the owner's next crossing must remain observable after a complete drain"
        );
        assert!(!schedule.take_successor(Some(new_work)));
    }

    #[test]
    fn failed_publication_cancels_a_deferred_successor() {
        let now = Instant::now();
        let trigger = crate::change_budget::CHECKPOINT_TRIGGER;
        let mut schedule = CheckpointSchedule::new(Duration::from_secs(30), now);
        schedule.immediate_successor = Some(10);
        schedule.completed(now, snapshot(trigger, 11, None));
        let owner = crate::change_budget::OwnerCapacityState {
            active: trigger,
            frozen: 0,
            work_revision: 11,
            checkpoint_request_revision: None,
        };
        assert!(
            !schedule.take_successor(Some(owner)),
            "a failed attempt must retain its existing periodic backoff"
        );
        assert!(!schedule.should_attempt(now, snapshot(trigger, 11, None), 11));
        assert!(schedule.should_attempt(
            now + Duration::from_secs(30),
            snapshot(trigger, 11, None),
            11
        ));
    }

    #[test]
    fn missing_or_replaced_owner_cancels_a_deferred_successor() {
        let now = Instant::now();
        let trigger = crate::change_budget::CHECKPOINT_TRIGGER;
        for owner in [
            None,
            Some(crate::change_budget::OwnerCapacityState {
                active: trigger,
                frozen: 0,
                work_revision: 9,
                checkpoint_request_revision: None,
            }),
        ] {
            let mut schedule = CheckpointSchedule::new(Duration::from_secs(30), now);
            schedule.immediate_successor = Some(10);
            assert!(!schedule.take_successor(owner));
            assert_eq!(schedule.immediate_successor, None);
        }
    }

    #[test]
    fn unchanged_high_failure_waits_for_completion_relative_period() {
        let now = Instant::now();
        let mut schedule = CheckpointSchedule::new(Duration::from_secs(10), now);
        let high = snapshot(crate::change_budget::CHECKPOINT_TRIGGER, 1, None);
        schedule.completed(now + Duration::from_secs(3), high);
        assert!(!schedule.should_attempt(now + Duration::from_secs(12), high, 1));
        assert!(schedule.should_attempt(now + Duration::from_secs(13), high, 1));
    }

    #[test]
    fn blocked_admission_request_is_consumed_once() {
        let now = Instant::now();
        let mut schedule = CheckpointSchedule::new(Duration::from_secs(30), now);
        let requested = snapshot(100 * 1024 * 1024, 7, Some(7));
        assert!(!schedule.should_attempt(now, requested, 7));
        schedule.completed(now + Duration::from_secs(1), requested);
        assert!(!schedule.should_attempt(now + Duration::from_secs(1), requested, 7));
        assert!(!schedule.should_attempt(
            now + Duration::from_secs(1),
            snapshot(100 * 1024 * 1024, 8, Some(7)),
            8,
        ));
    }

    #[test]
    fn no_work_has_no_checkpoint_request_attempt() {
        let now = Instant::now();
        let mut schedule = CheckpointSchedule::new(Duration::from_secs(30), now);
        assert!(!schedule.should_attempt(now, snapshot(0, 0, None), 0));
    }

    async fn wait_for_checkpoint_count(engine: &Engine, expected: u64) {
        tokio::time::timeout(Duration::from_secs(3), async {
            while engine.metrics().segment_checkpoint_completed_total.get() < expected {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("actual checkpoint driver made no progress");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn budget_wake_does_not_repeat_high_checkpoint_for_each_small_change() {
        let root = tempfile::tempdir().unwrap();
        let engine = Arc::new(Engine::new());
        let store = Arc::new(crate::segment_rdb::SegmentRdbStore::new(root.path()).unwrap());
        let writer = Arc::new(Watermark(AtomicU64::new(7)));
        let sink = Arc::new(SegmentCheckpointSink {
            engine: engine.clone(),
            store: store.clone(),
            writer: writer.clone(),
            aof: None,
        });
        // This unit drives the accounting input directly. Runtime Engine
        // admission has its own HTTP contract; no large payload is allocated here.
        let budget = ChangeBudget::new();
        let owner = budget.owner();
        owner
            .try_reserve(crate::change_budget::CHECKPOINT_TRIGGER)
            .unwrap()
            .commit()
            .unwrap();
        let driver = sink.spawn_driver_with_budget(Duration::from_secs(3600), budget);
        wait_for_checkpoint_count(&engine, 1).await;
        assert_eq!(
            store.load_current_generation().unwrap().unwrap().sequence,
            7
        );
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert_eq!(
            engine.metrics().segment_checkpoint_completed_total.get(),
            1,
            "unchanged high work must not spin checkpoints"
        );
        owner.try_reserve(1).unwrap().commit().unwrap();
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert_eq!(
            engine.metrics().segment_checkpoint_completed_total.get(),
            1,
            "small work under the same high-water window must not recheckpoint"
        );
        assert_eq!(writer.applied_seq(), 7);
        assert_eq!(
            store.load_current_generation().unwrap().unwrap().sequence,
            7
        );
        drop(driver);
        owner.retire();
    }

    #[test]
    fn waiter_observes_change_between_subscription_and_thread_start() {
        let budget = ChangeBudget::new();
        let wake = budget.checkpoint_wake();
        let observed = wake.epoch();
        let owner = budget.owner();
        let _reservation = owner.try_reserve(1).unwrap();
        let (sender, mut receiver) = mpsc::channel(1);
        let stop = Arc::new(AtomicBool::new(false));
        let waiter = spawn_budget_waiter(wake, sender, stop.clone(), observed);
        let deadline = std::time::Instant::now() + Duration::from_secs(1);
        let got_notice = loop {
            if receiver.try_recv().is_ok() {
                break true;
            }
            if std::time::Instant::now() >= deadline {
                break false;
            }
            std::thread::yield_now();
        };
        stop.store(true, Ordering::Release);
        waiter.join().unwrap();
        assert!(
            got_notice,
            "a change after subscription must not be swallowed by late thread start"
        );
    }
}
