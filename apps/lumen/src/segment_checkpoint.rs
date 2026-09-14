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

/// A completed attempt suppresses only the work revision it sampled. A later
/// direct Engine mutation rearms even when the writer WAL sequence is unchanged.
struct CheckpointSchedule {
    period: Duration,
    next_deadline: Instant,
    last_completed_work_revision: Option<u64>,
}

impl CheckpointSchedule {
    fn new(period: Duration, now: Instant) -> Self {
        Self {
            period,
            next_deadline: now + period,
            last_completed_work_revision: None,
        }
    }

    fn should_attempt(&self, now: Instant, pending: Snapshot, work_revision: u64) -> bool {
        now >= self.next_deadline
            || (pending.checkpoint_needed()
                && self
                    .last_completed_work_revision
                    .is_none_or(|completed| work_revision > completed))
            || (pending
                .checkpoint_request_revision
                .is_some_and(|requested| requested <= work_revision)
                && self
                    .last_completed_work_revision
                    .is_none_or(|completed| work_revision > completed))
    }

    fn completed(&mut self, now: Instant, work_revision: u64) {
        self.last_completed_work_revision = Some(work_revision);
        self.next_deadline = now + self.period;
    }
}

impl SegmentCheckpointSink {
    async fn checkpoint_with_fence(
        &self,
        fence: Option<crate::segment_capacity::PublicationFence>,
    ) -> Result<bool> {
        let sink_engine = self.engine.clone();
        let sink_store = self.store.clone();
        let sink_writer = self.writer.clone();
        let sink_aof = self.aof.clone();
        tokio::task::spawn_blocking(move || {
            let sink = SegmentCheckpointSink {
                engine: sink_engine,
                store: sink_store,
                writer: sink_writer,
                aof: sink_aof,
            };
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
                if schedule.should_attempt(now, pending, work_revision) {
                    if let Err(error) = self.checkpoint_with_fence(periodic_fence.clone()).await {
                        if crate::coordinator::is_storage_full(&error) {
                            self.engine.metrics().mark_storage_degraded();
                        }
                        tracing::warn!(error = %format!("{error:#}"), "periodic segment checkpoint failed");
                    }
                    // Reset on both success and failure. New work that was
                    // applied during the attempt has a greater revision and
                    // gets an immediate next-loop attempt; unchanged high
                    // work waits one full period.
                    schedule.completed(Instant::now(), work_revision);
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
            .spawn_driver_with_budget(Duration::from_secs(3600), budget);
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
        .expect(
            "last store/engine owner must reclaim its private root within the shared deadline",
        );
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
        assert!(
            budget.snapshot().frozen >= frozen_before_failure,
            "failed save must retain the captured charge"
        );
        admitted_keyword(&engine, "newer", "newer");
        assert!(sink.checkpoint_now().await.unwrap());
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
        let mut driver = sink.spawn_driver_with_budget(Duration::from_secs(3600), budget);
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
        let release = ReleaseWrite(hold.clone());
        hold.armed.store(true, Ordering::Release);
        let sink = Arc::new(SegmentCheckpointSink {
            engine: engine.clone(),
            store: store.clone(),
            writer: Arc::new(EngineWatermarkSink::new(engine.clone())),
            aof: None,
        });
        let mut driver = sink.spawn_driver_with_budget(Duration::from_secs(3600), budget);
        engine.request_pending_checkpoint();
        tokio::task::spawn_blocking(move || entered_rx.recv_timeout(Duration::from_secs(5)))
            .await
            .unwrap()
            .expect("checkpoint must reach the real write pause");
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
        let schedule = CheckpointSchedule::new(Duration::from_secs(30), now);
        assert!(schedule.should_attempt(
            now,
            snapshot(crate::change_budget::CHECKPOINT_TRIGGER, 1, None),
            1
        ));
    }

    #[test]
    fn same_wal_sequence_direct_change_rearms_after_an_attempt() {
        let now = Instant::now();
        let mut schedule = CheckpointSchedule::new(Duration::from_secs(30), now);
        let high = snapshot(crate::change_budget::CHECKPOINT_TRIGGER, 1, None);
        assert!(schedule.should_attempt(now, high, 1));
        schedule.completed(now + Duration::from_secs(2), 1);
        // Fixture WAL sequence stays 7. The direct change has work revision 2.
        assert!(schedule.should_attempt(
            now + Duration::from_secs(2),
            snapshot(crate::change_budget::CHECKPOINT_TRIGGER, 2, None),
            2,
        ));
    }

    #[test]
    fn unchanged_high_failure_waits_for_completion_relative_period() {
        let now = Instant::now();
        let mut schedule = CheckpointSchedule::new(Duration::from_secs(10), now);
        let high = snapshot(crate::change_budget::CHECKPOINT_TRIGGER, 1, None);
        schedule.completed(now + Duration::from_secs(3), 1);
        assert!(!schedule.should_attempt(now + Duration::from_secs(12), high, 1));
        assert!(schedule.should_attempt(now + Duration::from_secs(13), high, 1));
    }

    #[test]
    fn blocked_admission_request_runs_below_soft_limit_once_per_work_revision() {
        let now = Instant::now();
        let mut schedule = CheckpointSchedule::new(Duration::from_secs(30), now);
        let requested = snapshot(100 * 1024 * 1024, 7, Some(7));
        assert!(schedule.should_attempt(now, requested, 7));
        schedule.completed(now + Duration::from_secs(1), 7);
        assert!(!schedule.should_attempt(now + Duration::from_secs(1), requested, 7));
        assert!(schedule.should_attempt(
            now + Duration::from_secs(1),
            snapshot(100 * 1024 * 1024, 8, Some(7)),
            8,
        ));
    }

    #[test]
    fn no_work_has_no_checkpoint_request_attempt() {
        let now = Instant::now();
        let schedule = CheckpointSchedule::new(Duration::from_secs(30), now);
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
    async fn budget_wake_runs_real_checkpoint_again_with_unchanged_wal_sequence() {
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
        wait_for_checkpoint_count(&engine, 2).await;
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
