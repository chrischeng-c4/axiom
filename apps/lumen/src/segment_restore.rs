//! Durable, single-process segment restore.

use std::io;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use storage_durable::{CommitError, CommitFailureClass};

use crate::api::RestoreSink;
use crate::coordinator::{MutationGate, SharedAof, StorageFullError, WriteSink};
use crate::segment_rdb::SegmentRdbStore;
use crate::storage::{Engine, SnapshotV1};

#[derive(Debug)]
/// A durable restore failed before it could move `CURRENT` or live state.
pub(crate) struct RestoreNotCommitted(pub(crate) String);
impl std::fmt::Display for RestoreNotCommitted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for RestoreNotCommitted {}

/// Restore sink for embedded segment persistence.
pub struct SegmentRestoreSink {
    live_engine: Arc<Engine>,
    store: Arc<SegmentRdbStore>,
    writer: Arc<dyn WriteSink>,
    aof: SharedAof,
    gate: MutationGate,
    #[cfg(test)]
    reload_integrity_mismatch: std::sync::atomic::AtomicBool,
    #[cfg(test)]
    activation_failure: std::sync::atomic::AtomicBool,
}

impl SegmentRestoreSink {
    /// Construct a sink only for a writer that shares this process's mutation
    /// gate. Cluster writers cannot provide the atomic replacement contract.
    pub fn new(
        live_engine: Arc<Engine>,
        store: Arc<SegmentRdbStore>,
        writer: Arc<dyn WriteSink>,
        aof: SharedAof,
    ) -> Result<Self> {
        let gate = writer.mutation_gate().ok_or_else(|| {
            anyhow!("durable segment restore requires a process-local mutation gate")
        })?;
        Ok(Self {
            live_engine,
            store,
            writer,
            aof,
            gate,
            #[cfg(test)]
            reload_integrity_mismatch: std::sync::atomic::AtomicBool::new(false),
            #[cfg(test)]
            activation_failure: std::sync::atomic::AtomicBool::new(false),
        })
    }

    #[cfg(test)]
    pub(crate) fn force_reload_integrity_mismatch(&self) {
        self.reload_integrity_mismatch
            .store(true, std::sync::atomic::Ordering::Release);
    }
    #[cfg(test)]
    pub(crate) fn force_activation_failure(&self) {
        self.activation_failure
            .store(true, std::sync::atomic::Ordering::Release);
    }

    fn storage_full(error: &anyhow::Error) -> bool {
        error.chain().any(|cause| {
            cause
                .downcast_ref::<io::Error>()
                .is_some_and(|e| e.kind() == io::ErrorKind::StorageFull)
        })
    }

    fn not_committed(error: anyhow::Error) -> anyhow::Error {
        anyhow::Error::new(RestoreNotCommitted(error.to_string())).context(error)
    }

    fn mark_full(&self) {
        self.live_engine.metrics().mark_storage_degraded();
    }

    fn restart(&self) {
        self.gate.require_restart();
    }
}

#[async_trait]
impl RestoreSink for SegmentRestoreSink {
    async fn restore(&self, snapshot: SnapshotV1) -> Result<()> {
        // Decode and validate all user data before entering the exclusive gate.
        let candidate = tokio::task::spawn_blocking(move || {
            let candidate = Arc::new(Engine::new());
            candidate.restore(snapshot).map(|()| candidate)
        })
        .await
        .map_err(|join| Self::not_committed(anyhow!("candidate restore task failed: {join}")))??;

        let _exclusive = self.gate.exclusive().await?;
        let watermark = self.writer.applied_seq();

        let aof = Arc::clone(&self.aof);
        let sync = tokio::task::spawn_blocking(move || {
            let mut aof = aof.lock().map_err(|_| anyhow!("AOF mutex poisoned"))?;
            aof.sync_strict()
        })
        .await
        .map_err(|join| Self::not_committed(anyhow!("strict AOF sync task failed: {join}")))?;
        if let Err(error) = sync {
            if Self::storage_full(&error) {
                self.mark_full();
                return Err(anyhow::Error::new(StorageFullError(error.to_string())).context(error));
            }
            return Err(Self::not_committed(error));
        }

        let store = Arc::clone(&self.store);
        let candidate_for_save = Arc::clone(&candidate);
        let save = tokio::task::spawn_blocking(move || {
            store.save_required(&candidate_for_save, watermark)
        })
        .await;
        let committed_name = match save {
            Ok(Ok(name)) => name,
            Ok(Err(error)) => {
                let full = Self::storage_full(&error);
                if full {
                    self.mark_full();
                }
                if let Some(commit) = error
                    .chain()
                    .find_map(|cause| cause.downcast_ref::<CommitError>())
                {
                    if commit.class() == CommitFailureClass::CommitUncertain {
                        self.restart();
                        return Err(anyhow::Error::new(crate::coordinator::RestartRequired(
                            "segment restore commit outcome is uncertain; restart required".into(),
                        ))
                        .context(error));
                    }
                }
                if full {
                    return Err(
                        anyhow::Error::new(StorageFullError(error.to_string())).context(error)
                    );
                }
                return Err(Self::not_committed(error));
            }
            Err(join) => {
                self.restart();
                return Err(anyhow::Error::new(crate::coordinator::RestartRequired(
                    "segment restore save task panicked; restart required".into(),
                ))
                .context(anyhow!("save task failed: {join}")));
            }
        };

        let store = Arc::clone(&self.store);
        let loaded =
            match tokio::task::spawn_blocking(move || store.load_current_generation()).await {
                Ok(loaded) => loaded,
                Err(join) => {
                    self.restart();
                    return Err(anyhow::Error::new(crate::coordinator::RestartRequired(
                        "segment restore reload task panicked; restart required".into(),
                    ))
                    .context(anyhow!("CURRENT reload task failed: {join}")));
                }
            };
        let loaded = match loaded {
            Ok(Some(loaded)) => loaded,
            Ok(None) => {
                self.restart();
                return Err(anyhow::Error::new(crate::coordinator::RestartRequired(
                    "segment restore committed without a CURRENT generation".into(),
                )));
            }
            Err(error) => {
                self.restart();
                return Err(anyhow::Error::new(crate::coordinator::RestartRequired(
                    "segment restore could not reload CURRENT".into(),
                ))
                .context(error));
            }
        };
        #[cfg(test)]
        let forced_mismatch = self
            .reload_integrity_mismatch
            .swap(false, std::sync::atomic::Ordering::AcqRel);
        #[cfg(not(test))]
        let forced_mismatch = false;
        if forced_mismatch || loaded.name != committed_name || loaded.sequence != watermark {
            self.restart();
            return Err(anyhow::Error::new(crate::coordinator::RestartRequired(
                "segment restore CURRENT integrity check failed; restart required".into(),
            )));
        }
        let fresh = match Arc::try_unwrap(loaded.engine) {
            Ok(engine) => engine,
            Err(_) => {
                self.restart();
                return Err(anyhow::Error::new(crate::coordinator::RestartRequired(
                    "segment restore reload retained unexpected engine references".into(),
                )));
            }
        };
        #[cfg(test)]
        let activation_failure = self
            .activation_failure
            .swap(false, std::sync::atomic::Ordering::AcqRel);
        #[cfg(not(test))]
        let activation_failure = false;
        if activation_failure {
            self.restart();
            return Err(anyhow::Error::new(crate::coordinator::RestartRequired(
                "segment restore live activation failed; restart required".into(),
            )));
        }
        if let Err(error) = self.live_engine.activate_replacement(fresh) {
            self.restart();
            return Err(anyhow::Error::new(crate::coordinator::RestartRequired(
                "segment restore live activation failed; restart required".into(),
            ))
            .context(error));
        }

        let aof = Arc::clone(&self.aof);
        let store = Arc::clone(&self.store);
        tokio::task::spawn_blocking(move || {
            if let Err(error) = aof
                .lock()
                .map_err(|_| anyhow!("AOF mutex poisoned"))
                .and_then(|mut aof| aof.truncate_through(watermark))
            {
                tracing::warn!(error = %error, "durable restore AOF trim failed");
            }
            if let Err(error) = store.prune(3) {
                tracing::warn!(error = %error, "durable restore generation prune failed");
            }
        })
        .await
        .unwrap_or_else(
            |error| tracing::warn!(error = %error, "durable restore cleanup task failed"),
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinator::{RestartRequired, WriteCoordinator};
    use crate::log_entry::RaftLogEntry;
    use crate::types::CreateCollectionRequest;
    use crate::wal::{MemWal, SharedWal};
    use std::path::Path;
    use std::sync::Mutex;
    use storage_durable::{CommitStep, FailureInjector, FailurePoint};

    const WATERMARK: u64 = 7;

    #[derive(Clone, Copy)]
    enum InjectedAction {
        Error(io::ErrorKind),
        Panic,
    }

    #[derive(Default)]
    struct FailAt(Mutex<Option<(CommitStep, InjectedAction)>>);

    impl FailAt {
        fn arm(&self, step: CommitStep, kind: io::ErrorKind) {
            *self.0.lock().unwrap() = Some((step, InjectedAction::Error(kind)));
        }

        fn panic_at(&self, step: CommitStep) {
            *self.0.lock().unwrap() = Some((step, InjectedAction::Panic));
        }
    }

    impl FailureInjector for FailAt {
        fn check(&self, point: &FailurePoint) -> io::Result<()> {
            let action = self
                .0
                .lock()
                .unwrap()
                .as_ref()
                .filter(|(step, _)| *step == point.step)
                .map(|(_, action)| *action);
            match action {
                Some(InjectedAction::Error(kind)) => Err(io::Error::from(kind)),
                Some(InjectedAction::Panic) => panic!("injected save panic"),
                None => Ok(()),
            }
        }
    }

    struct Fixture {
        dir: tempfile::TempDir,
        live: Arc<Engine>,
        store: Arc<SegmentRdbStore>,
        writer: Arc<WriteCoordinator>,
        aof: SharedAof,
        old_name: storage_durable::GenerationName,
        old_current: Vec<u8>,
    }

    fn schema() -> CreateCollectionRequest {
        serde_json::from_value(serde_json::json!({
            "fields": { "value": { "type": "keyword" } }
        }))
        .expect("valid keyword schema")
    }

    fn engine_with(collection: &str) -> Arc<Engine> {
        let engine = Arc::new(Engine::new());
        engine
            .create_collection(collection, schema())
            .expect("create fixture collection");
        engine
    }

    fn replacement_snapshot() -> SnapshotV1 {
        engine_with("restored")
            .snapshot()
            .expect("replacement snapshot")
    }

    fn assert_collections(engine: &Engine, expected: &[&str]) {
        let mut actual = engine.list_collections().expect("list collections");
        actual.sort();
        let mut expected = expected
            .iter()
            .map(|name| (*name).to_string())
            .collect::<Vec<_>>();
        expected.sort();
        assert_eq!(actual, expected);
    }

    fn current_bytes(root: &Path) -> Vec<u8> {
        std::fs::read(root.join("CURRENT")).expect("read CURRENT")
    }

    fn setup(injector: Option<Arc<dyn FailureInjector>>) -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(match injector {
            Some(i) => SegmentRdbStore::new_with_failure_injector(dir.path(), i).unwrap(),
            None => SegmentRdbStore::new(dir.path()).unwrap(),
        });
        let live = engine_with("old");
        let old_name = store
            .save_required(&live, WATERMARK)
            .expect("seed old CURRENT");
        let old_current = current_bytes(dir.path());
        let aof = Arc::new(Mutex::new(
            crate::aof::AofWriter::open(dir.path().join("aof")).unwrap(),
        ));
        let wal: SharedWal = Arc::new(MemWal::starting_at(WATERMARK));
        let writer = WriteCoordinator::start_from_with_aof(
            wal,
            Arc::clone(&live),
            WATERMARK,
            Arc::clone(&aof),
        );
        Fixture {
            dir,
            live,
            store,
            writer,
            aof,
            old_name,
            old_current,
        }
    }

    #[tokio::test]
    async fn success_commits_exact_watermark_and_next_write_advances() {
        let fixture = setup(None);
        let sink = SegmentRestoreSink::new(
            Arc::clone(&fixture.live),
            Arc::clone(&fixture.store),
            fixture.writer.clone(),
            fixture.aof.clone(),
        )
        .unwrap();

        sink.restore(replacement_snapshot()).await.unwrap();

        assert_collections(fixture.live.as_ref(), &["restored"]);
        let loaded = fixture
            .store
            .load_current_generation()
            .unwrap()
            .expect("restored CURRENT");
        assert_eq!(loaded.sequence, WATERMARK);
        assert_ne!(loaded.name, fixture.old_name);
        assert_collections(loaded.engine.as_ref(), &["restored"]);

        fixture
            .writer
            .submit(RaftLogEntry::CreateCollection {
                collection_id: "after".to_string(),
                req: schema(),
            })
            .await
            .expect("first post-restore write");
        assert_eq!(fixture.writer.applied_seq(), WATERMARK + 1);
        assert_collections(fixture.live.as_ref(), &["after", "restored"]);
    }

    #[tokio::test]
    async fn invalid_snapshot_does_not_wait_on_gate_or_change_state() {
        let fixture = setup(None);
        let sink = SegmentRestoreSink::new(
            fixture.live.clone(),
            fixture.store.clone(),
            fixture.writer,
            fixture.aof,
        )
        .unwrap();
        let gate = sink.gate.clone();
        let _held = gate.shared().await.unwrap();
        let invalid = SnapshotV1 {
            version: 999,
            collections: Default::default(),
        };

        tokio::time::timeout(std::time::Duration::from_secs(1), sink.restore(invalid))
            .await
            .expect("invalid snapshot must fail before waiting for exclusive gate")
            .expect_err("invalid snapshot must fail");
        assert_eq!(current_bytes(fixture.dir.path()), fixture.old_current);
        assert_collections(fixture.live.as_ref(), &["old"]);
        assert!(!sink.gate.is_restart_required());
    }

    #[tokio::test]
    async fn precommit_preserves_old_current_and_live_state() {
        let injector = Arc::new(FailAt::default());
        let fixture = setup(Some(injector.clone()));
        injector.arm(CommitStep::WriteCurrentTemp, io::ErrorKind::Other);
        let sink = SegmentRestoreSink::new(
            fixture.live.clone(),
            fixture.store,
            fixture.writer,
            fixture.aof,
        )
        .unwrap();

        let err = sink.restore(replacement_snapshot()).await.unwrap_err();
        assert!(err
            .chain()
            .any(|e| e.downcast_ref::<RestoreNotCommitted>().is_some()));
        assert_eq!(current_bytes(fixture.dir.path()), fixture.old_current);
        assert_collections(fixture.live.as_ref(), &["old"]);
        assert!(!sink.gate.is_restart_required());
        assert!(!fixture.live.metrics().is_storage_degraded());
    }

    #[tokio::test]
    async fn precommit_storage_full_is_degraded_without_moving_current() {
        let injector = Arc::new(FailAt::default());
        let fixture = setup(Some(injector.clone()));
        injector.arm(CommitStep::WriteCurrentTemp, io::ErrorKind::StorageFull);
        let sink = SegmentRestoreSink::new(
            fixture.live.clone(),
            fixture.store,
            fixture.writer,
            fixture.aof,
        )
        .unwrap();

        let err = sink.restore(replacement_snapshot()).await.unwrap_err();
        assert!(err
            .chain()
            .any(|e| e.downcast_ref::<StorageFullError>().is_some()));
        assert_eq!(current_bytes(fixture.dir.path()), fixture.old_current);
        assert_collections(fixture.live.as_ref(), &["old"]);
        assert!(!sink.gate.is_restart_required());
        assert!(fixture.live.metrics().is_storage_degraded());
    }

    #[tokio::test]
    async fn commit_uncertain_latches_and_fresh_store_follows_new_current() {
        let injector = Arc::new(FailAt::default());
        let fixture = setup(Some(injector.clone()));
        injector.arm(CommitStep::SyncRootAfterCurrent, io::ErrorKind::Other);
        let sink = SegmentRestoreSink::new(
            fixture.live.clone(),
            fixture.store,
            fixture.writer,
            fixture.aof,
        )
        .unwrap();

        let err = sink.restore(replacement_snapshot()).await.unwrap_err();
        assert!(err
            .chain()
            .any(|e| e.downcast_ref::<RestartRequired>().is_some()));
        assert!(sink.gate.is_restart_required());
        assert_collections(fixture.live.as_ref(), &["old"]);

        let fresh = SegmentRdbStore::new(fixture.dir.path()).unwrap();
        let loaded = fresh
            .load_current_generation()
            .unwrap()
            .expect("new CURRENT after uncertain final sync");
        assert_eq!(loaded.sequence, WATERMARK);
        assert_ne!(loaded.name, fixture.old_name);
        assert_collections(loaded.engine.as_ref(), &["restored"]);
    }

    #[tokio::test]
    async fn reload_integrity_failure_latches_and_preserves_old_live_state() {
        let fixture = setup(None);
        let sink = SegmentRestoreSink::new(
            fixture.live.clone(),
            fixture.store,
            fixture.writer,
            fixture.aof,
        )
        .unwrap();
        sink.force_reload_integrity_mismatch();
        let err = sink.restore(replacement_snapshot()).await.unwrap_err();
        assert!(err
            .chain()
            .any(|e| e.downcast_ref::<RestartRequired>().is_some()));
        assert!(sink.gate.is_restart_required());
        assert_collections(fixture.live.as_ref(), &["old"]);

        let fresh = SegmentRdbStore::new(fixture.dir.path()).unwrap();
        let loaded = fresh.load_current_generation().unwrap().unwrap();
        assert_collections(loaded.engine.as_ref(), &["restored"]);
    }

    #[tokio::test]
    async fn save_task_panic_latches_and_preserves_old_live_state() {
        let injector = Arc::new(FailAt::default());
        let fixture = setup(Some(injector.clone()));
        injector.panic_at(CommitStep::WriteCurrentTemp);
        let sink = SegmentRestoreSink::new(
            fixture.live.clone(),
            fixture.store,
            fixture.writer,
            fixture.aof,
        )
        .unwrap();

        let err = sink.restore(replacement_snapshot()).await.unwrap_err();
        assert!(err
            .chain()
            .any(|e| e.downcast_ref::<RestartRequired>().is_some()));
        assert!(sink.gate.is_restart_required());
        assert_eq!(current_bytes(fixture.dir.path()), fixture.old_current);
        assert_collections(fixture.live.as_ref(), &["old"]);
    }

    #[tokio::test]
    async fn activation_failure_latches_with_new_current_and_old_live_state() {
        let fixture = setup(None);
        let sink = SegmentRestoreSink::new(
            fixture.live.clone(),
            fixture.store,
            fixture.writer,
            fixture.aof,
        )
        .unwrap();
        sink.force_activation_failure();
        let err = sink.restore(replacement_snapshot()).await.unwrap_err();
        assert!(err
            .chain()
            .any(|e| e.downcast_ref::<RestartRequired>().is_some()));
        assert!(sink.gate.is_restart_required());
        assert_collections(fixture.live.as_ref(), &["old"]);

        let fresh = SegmentRdbStore::new(fixture.dir.path()).unwrap();
        let loaded = fresh.load_current_generation().unwrap().unwrap();
        assert_collections(loaded.engine.as_ref(), &["restored"]);
    }
}
