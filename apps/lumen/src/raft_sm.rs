// CODEGEN-BEGIN
//! `EngineSm` — lumen's [`Engine`] as a [`raft_runtime::RaftStateMachine`].
//!
//! This is lumen's convergence onto the shared raft host (epic #524): the host
//! is the sole applier, so the `WriteCoordinator`/`WalLog` seam is no longer
//! needed for the raft path. `apply` folds a committed
//! command into the engine and records the rich [`ApplyOutcome`] in a small
//! window so the write handler can return it (read-your-write); `snapshot`/
//! `restore` bridge to the engine's RDB checkpoint (the "backup layer").
//!
//! The raft log index **is** the WAL seq (both 1-based), so `apply_raft_entry`,
//! the RDB `up_to_seq` tag, and the outcome key all share the same `Index`.

use std::io::Read;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::Result;
use raft_runtime::{
    AdmissionPermit, Index, OutcomeWindow, PreparedSnapshot, ProposalBackpressure,
    RaftStateMachine, SnapshotPreparation,
};

use crate::change_budget::AdmissionError;
use crate::coordinator::WriteSink;
use crate::log_entry::RaftLogEntry;
use crate::rdb::RdbSnapshot;
use crate::storage::{
    ApplyOutcome, Engine, RecordAdmissionError, RecordReservation, RepriceRecord,
};
use crate::wal::WalRecord;
use raft_runtime::RaftHost;

/// How many recent apply outcomes to retain for the write handler to claim,
/// via [`OutcomeWindow`].
const OUTCOME_WINDOW: u64 = 8192;

/// lumen's engine driven as a raft state machine.
pub struct EngineSm {
    engine: Arc<Engine>,
    applied: AtomicU64,
    outcomes: Mutex<OutcomeWindow<Result<ApplyOutcome>>>,
    failed: AtomicBool,
    segment_store: Option<Arc<crate::segment_rdb::SegmentRdbStore>>,
    layer_capacity_owner: Mutex<Option<crate::segment_capacity::Fallback>>,
}

impl EngineSm {
    /// Wrap `engine`, seeded at `from_seq` (the seq the engine was cold-started
    /// to, e.g. from an RDB checkpoint — `0` for a fresh engine).
    pub fn new(engine: Arc<Engine>, from_seq: u64) -> Arc<Self> {
        Self::with_snapshot_store(engine, from_seq, None)
    }

    fn with_snapshot_store(
        engine: Arc<Engine>,
        from_seq: u64,
        segment_store: Option<Arc<crate::segment_rdb::SegmentRdbStore>>,
    ) -> Arc<Self> {
        engine.capture_barrier.apply().initialize_sequence(from_seq);
        Arc::new(EngineSm {
            engine,
            applied: AtomicU64::new(from_seq),
            outcomes: Mutex::new(OutcomeWindow::new(OUTCOME_WINDOW)),
            failed: AtomicBool::new(false),
            segment_store,
            layer_capacity_owner: Mutex::new(None),
        })
    }

    /// Select the segment snapshot backend for this state machine.
    pub fn new_with_segment_store(
        engine: Arc<Engine>,
        from_seq: u64,
        store: Arc<crate::segment_rdb::SegmentRdbStore>,
    ) -> Arc<Self> {
        Self::with_snapshot_store(engine, from_seq, Some(store))
    }

    /// #2516: the wrapped engine, so [`RaftWriteSink::submit`] can flip the
    /// sticky degraded-storage gauge when the raft log append itself hits
    /// ENOSPC (before there is any committed entry to apply).
    pub fn engine(&self) -> &Arc<Engine> {
        &self.engine
    }

    /// Claim the outcome for `index` (the host's `propose` returns the index;
    /// the write handler then takes the rich outcome the local apply produced).
    pub fn take_outcome(&self, index: u64) -> Result<ApplyOutcome> {
        self.outcomes
            .lock()
            .expect("outcomes poisoned")
            .claim(index)
            .unwrap_or_else(|| Err(anyhow::anyhow!("outcome for seq {index} unavailable")))
    }
}

/// One host-owned proposal carrier. It moves by the Raft (index, term), never
/// by request identity or equal command bytes. The host retains it if a caller
/// cancels or a post-index durable append fails.
struct AdmittedRaftRecord {
    entry: RaftLogEntry,
    reservation: RecordReservation,
}

impl EngineSm {
    /// Start or reuse this state machine's independent capacity maintainer.
    /// Call only before a committed-record capacity wait; local proposal
    /// admission must still return Full without creating background work.
    fn ensure_capacity_owner(&self) -> Result<()> {
        let mut owner = self
            .layer_capacity_owner
            .lock()
            .map_err(|_| anyhow::anyhow!("capacity owner poisoned"))?;
        crate::segment_capacity::Fallback::ensure(
            &mut owner,
            &self.engine,
            self.segment_store.clone(),
        )
    }

    fn decode_admitted(
        &self,
        command: &[u8],
        before_publication: bool,
    ) -> Result<AdmittedRaftRecord> {
        anyhow::ensure!(
            !self.failed.load(Ordering::Acquire),
            "Raft apply requires restart after an unresolved record"
        );
        let workspace = crate::wal_wire_cost::scan_workspace_bytes(command.len())?;
        // The node owns its original log command and the current apply delivery.
        // Admission also precedes the decoder's token and untagged-value buffers.
        let transport = command
            .len()
            .checked_mul(2)
            .ok_or(RecordAdmissionError::Overflow)?;
        let request = self
            .engine
            .record_ram_request_from_bound(workspace, transport);
        let mut reservation = match self.engine.try_reserve_record_ram(&request) {
            Ok(reservation) => reservation,
            Err(RecordAdmissionError::Capacity(AdmissionError::Full { .. }))
                if !before_publication =>
            {
                self.ensure_capacity_owner()?;
                self.engine.wait_reserve_record_ram(&request)?
            }
            Err(error) => return Err(error.into()),
        };
        let decoded_peak = crate::wal_wire_cost::decoded_peak_bound(command)?;
        let required = decoded_peak
            .checked_add(transport)
            .ok_or(RecordAdmissionError::Overflow)?;
        match reservation.try_grow_to(required) {
            Ok(()) => (),
            Err(AdmissionError::Full { .. }) if !before_publication => {
                self.ensure_capacity_owner()?;
                self.engine.request_pending_checkpoint();
                reservation
                    .wait_grow_to(required)
                    .map_err(RecordAdmissionError::Capacity)?;
            }
            Err(error) => return Err(RecordAdmissionError::Capacity(error).into()),
        }
        let record = WalRecord::decode(command)?;
        self.engine.price_decoded_record(
            &record.entry,
            &mut reservation,
            decoded_peak,
            transport,
            before_publication,
        )?;
        Ok(AdmittedRaftRecord {
            entry: record.entry,
            reservation,
        })
    }

    fn apply_record(&self, index: Index, mut record: AdmittedRaftRecord) -> Result<()> {
        loop {
            match self
                .engine
                .begin_admitted_record(record.entry, record.reservation)
            {
                Ok(mut guard) => {
                    let outcome = self.engine.apply_prepared_raft_entry(&mut guard);
                    let mut outcomes = self.outcomes.lock().expect("outcomes poisoned");
                    outcomes.insert(index, outcome);
                    outcomes.advance(index);
                    // A normal field validation error may follow a valid prefix.
                    // Its complete mutation and both watermarks share this lease.
                    guard.apply_lease().advance_sequence(index);
                    self.applied.store(index, Ordering::Release);
                    return Ok(());
                }
                Err(RepriceRecord {
                    entry,
                    mut reservation,
                    required: Some(required),
                    ..
                }) => {
                    match reservation.try_grow_to(required) {
                        Ok(()) => (),
                        Err(AdmissionError::Full { .. }) => {
                            self.ensure_capacity_owner()?;
                            self.engine.request_pending_checkpoint();
                            reservation
                                .wait_grow_to(required)
                                .map_err(RecordAdmissionError::Capacity)?;
                        }
                        Err(error) => return Err(RecordAdmissionError::Capacity(error).into()),
                    }
                    record = AdmittedRaftRecord { entry, reservation };
                }
                Err(reprice) => return Err(reprice.error.into()),
            }
        }
    }
}

impl RaftStateMachine for EngineSm {
    fn admit_proposal(&self, command: &[u8]) -> Result<Option<AdmissionPermit>> {
        match self.decode_admitted(command, true) {
            Ok(record) => Ok(Some(Box::new(record))),
            Err(error) => {
                if let Some(pending) = error.downcast_ref::<RecordAdmissionError>().and_then(
                    crate::change_admission::PendingChangeCapacity::from_record_prepublication,
                ) {
                    return Err(ProposalBackpressure {
                        reason: pending.to_string(),
                        retry_after_seconds: 1,
                    }
                    .into());
                }
                Err(error)
            }
        }
    }

    fn apply(&self, index: Index, command: &[u8]) -> Result<()> {
        self.apply_admitted(index, command, None)
    }

    fn apply_admitted(
        &self,
        index: Index,
        command: &[u8],
        permit: Option<AdmissionPermit>,
    ) -> Result<()> {
        if self.failed.load(Ordering::Acquire) {
            anyhow::bail!("Raft apply requires restart after an unresolved record");
        }
        let result = (|| {
            // Retained oversized fast-Index input must not enter the owned
            // decoder. Its scalar values can be projected from the pinned
            // command into private immutable files before the apply interval.
            // Local proposals still go through pre-publication admission.
            if permit.is_none() && command.len() > crate::change_budget::HARD_LIMIT / 8 {
                if let Ok(scanner) =
                    crate::wal::fast_index_scanner::FastIndexScanner::parse(command)
                {
                    if self.engine.try_apply_committed_index_with_capacity_owner(
                        &scanner,
                        index,
                        || self.ensure_capacity_owner(),
                        |apply, outcome| {
                            let mut outcomes = self.outcomes.lock().expect("outcomes poisoned");
                            outcomes.insert(index, outcome);
                            outcomes.advance(index);
                            apply.advance_sequence(index);
                            self.applied.store(index, Ordering::Release);
                        },
                    )? {
                        return Ok(());
                    }
                }
                if self
                    .engine
                    .try_apply_committed_replace_with_capacity_owner(
                        command,
                        index,
                        &mut || self.ensure_capacity_owner(),
                        |apply, outcome| {
                            let mut outcomes = self.outcomes.lock().expect("outcomes poisoned");
                            outcomes.insert(index, outcome);
                            outcomes.advance(index);
                            apply.advance_sequence(index);
                            self.applied.store(index, Ordering::Release);
                        },
                    )?
                {
                    return Ok(());
                }
            }
            let record = match permit {
                Some(permit) => *permit.downcast::<AdmittedRaftRecord>().map_err(|_| {
                    anyhow::anyhow!("Raft admission belongs to another state machine")
                })?,
                None => self.decode_admitted(command, false)?,
            };
            self.apply_record(index, record)
        })();
        if result.is_err() {
            // Preparation/IO failures are not business no-ops. Preserve the
            // source log and the old watermark, and refuse later publication.
            self.failed.store(true, Ordering::Release);
            self.engine.capture_barrier.apply().mark_uncertain();
        }
        result
    }

    fn preflight_snapshot(&self) -> Result<Option<Box<dyn SnapshotPreparation>>> {
        self.segment_store
            .as_ref()
            .map(|store| {
                store
                    .raft_snapshot_preflight(self.engine.clone())
                    .map(|preparation| Box::new(preparation) as Box<dyn SnapshotPreparation>)
            })
            .transpose()
    }

    fn snapshot(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        if let Some(preparation) = self.preflight_snapshot()? {
            return preparation
                .capture_at(self.applied_index())?
                .write_to(writer);
        }
        let capture = self
            .engine
            .capture_barrier
            .capture(self.applied_index())
            .map_err(|error| anyhow::anyhow!(error))?;
        let snapshot = self.engine.snapshot()?;
        let up_to_seq = capture.stamp().sequence;
        drop(capture);
        let bytes = RdbSnapshot {
            up_to_seq,
            snapshot,
        }
        .encode()?;
        writer.write_all(&bytes)?;
        Ok(())
    }

    fn validate_snapshot(&self, reader: &mut dyn Read) -> Result<()> {
        let mut prefix = Vec::with_capacity(8);
        reader.take(8).read_to_end(&mut prefix)?;
        let mut input = prefix.as_slice().chain(reader);
        if prefix == crate::segment_rdb::raft_archive::MAGIC {
            let store = self
                .segment_store
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("segment archive needs a segment snapshot store"))?;
            store.validate_raft_archive(&mut input)
        } else {
            let rdb = read_legacy_snapshot(&mut input)?;
            Engine::new().restore(rdb.snapshot)
        }
    }

    fn restore(&self, reader: &mut dyn Read) -> Result<()> {
        let mut prefix = Vec::with_capacity(8);
        reader.take(8).read_to_end(&mut prefix)?;
        let mut input = prefix.as_slice().chain(reader);
        let sequence = if prefix == crate::segment_rdb::raft_archive::MAGIC {
            let store = self
                .segment_store
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("segment archive needs a segment snapshot store"))?;
            store.restore_raft_archive(&self.engine, &mut input, |sequence| {
                self.applied.store(sequence, Ordering::Release)
            })?
        } else {
            let rdb = read_legacy_snapshot(&mut input)?;
            if let Some(store) = &self.segment_store {
                store.restore_legacy_raft_snapshot(&self.engine, rdb, |sequence| {
                    self.applied.store(sequence, Ordering::Release)
                })?
            } else {
                let apply = self.engine.capture_barrier.apply();
                self.engine.restore(rdb.snapshot)?;
                apply.initialize_sequence(rdb.up_to_seq);
                self.applied.store(rdb.up_to_seq, Ordering::Release);
                rdb.up_to_seq
            }
        };
        self.applied.store(sequence, Ordering::Release);
        Ok(())
    }

    fn applied_index(&self) -> Index {
        self.applied.load(Ordering::Acquire)
    }
}

impl SnapshotPreparation for crate::segment_rdb::raft_capture::SegmentRaftPreparation {
    fn capture_at(self: Box<Self>, index: Index) -> Result<Box<dyn PreparedSnapshot>> {
        Ok(Box::new((*self).capture_at(index)?))
    }
}

impl PreparedSnapshot for crate::segment_rdb::raft_capture::SegmentRaftCapture {
    fn write_to(self: Box<Self>, writer: &mut dyn std::io::Write) -> Result<()> {
        let generation = (*self).publish()?;
        crate::segment_rdb::raft_archive::write_archive(
            generation.path(),
            generation.sequence(),
            writer,
        )
    }
}

fn read_legacy_snapshot(reader: &mut dyn Read) -> Result<RdbSnapshot> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    let header: [u8; 4] = bytes
        .get(..4)
        .ok_or_else(|| anyhow::anyhow!("truncated legacy RDB header"))?
        .try_into()?;
    let expanded = u32::from_le_bytes(header) as usize;
    // Every LZ4 length extension consumes a byte for at most 255 output bytes.
    // Reject an impossible claim before the legacy decoder allocates its output.
    anyhow::ensure!(
        expanded <= bytes.len().saturating_sub(4).saturating_mul(255),
        "legacy RDB expanded length exceeds its compressed input bound"
    );
    RdbSnapshot::decode(&bytes)
}

/// The [`WriteSink`] for `--wal raft`: a write proposes through the shared
/// [`RaftHost`] (which handles leader-redirect + read-your-write), and the rich
/// [`ApplyOutcome`] is claimed from the local [`EngineSm`] apply (the host
/// applies on every node, so a follower has its own outcome).
pub struct RaftWriteSink {
    host: Arc<RaftHost>,
    sm: Arc<EngineSm>,
}

impl RaftWriteSink {
    pub fn new(host: Arc<RaftHost>, sm: Arc<EngineSm>) -> Self {
        Self { host, sm }
    }

    /// Count exactly the terminal local pre-publication capacity refusal. The
    /// host has not appended a command when this returns an error.
    fn prepublication_backpressure(
        &self,
        pending: crate::change_admission::PendingChangeCapacity,
    ) -> anyhow::Error {
        self.sm.engine().metrics().incr_segment_backpressure();
        anyhow::Error::new(pending)
    }
}

#[async_trait::async_trait]
impl WriteSink for RaftWriteSink {
    async fn submit(&self, entry: RaftLogEntry) -> Result<ApplyOutcome> {
        let record = WalRecord::new(entry);
        let raw = Engine::record_owned_bytes(&record.entry).map_err(|error| match error {
            RecordAdmissionError::Overflow => self.prepublication_backpressure(
                crate::change_admission::PendingChangeCapacity::Overflow,
            ),
            other => anyhow::Error::new(other),
        })?;
        let extra = raw.checked_mul(2).ok_or_else(|| {
            self.prepublication_backpressure(
                crate::change_admission::PendingChangeCapacity::Overflow,
            )
        })?;
        let request = self.sm.engine.record_ram_request_from_bound(raw, extra);
        // This origin reservation covers the decoded request and its encoder.
        // Leader admission is separate and follows the command into the host.
        let _origin = self
            .sm
            .engine
            .try_reserve_record_ram(&request)
            .map_err(|error| {
                match crate::change_admission::PendingChangeCapacity::from_record_prepublication(
                    &error,
                ) {
                    Some(pending) => self.prepublication_backpressure(pending),
                    None => anyhow::Error::new(error),
                }
            })?;
        let command = record.encode()?;
        drop(record);
        let index = match self.host.propose(command).await {
            Ok(index) => index,
            Err(e) => {
                if let Some(backpressure) = e.downcast_ref::<ProposalBackpressure>() {
                    return Err(self.prepublication_backpressure(
                        crate::change_admission::PendingChangeCapacity::Raft {
                            reason: backpressure.reason.clone(),
                        },
                    ));
                }
                // #2516: the raft log append is itself a durable write path
                // (named explicitly alongside AOF/segment/snapshot writes) —
                // an ENOSPC here (surfaced through `propose`'s error chain the
                // same way an AOF ENOSPC is) must enter the same sticky
                // degraded read-only mode, not just propagate a generic error.
                if crate::coordinator::is_storage_full(&e) {
                    tracing::error!(
                        error = %e,
                        "raft log append hit ENOSPC — entering degraded read-only mode"
                    );
                    self.sm.engine().metrics().mark_storage_degraded();
                    return Err(anyhow::Error::new(crate::coordinator::StorageFullError(
                        "local storage is full (ENOSPC) appending to the raft log; node \
                         entered degraded read-only mode"
                            .to_string(),
                    )));
                }
                return Err(e);
            }
        };
        self.sm.take_outcome(index)
    }
    fn applied_seq(&self) -> u64 {
        self.sm.applied_index()
    }
    fn restart_required(&self) -> bool {
        self.sm.failed.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log_entry::RaftLogEntry;
    use crate::types::{
        BatchUnindexDocsRequest, CreateCollectionRequest, FieldSpec, FieldType, FieldValue,
        IndexItem, IndexRequest,
    };
    use raft_runtime::{HostConfig, Membership, RaftHost, RaftStore};
    use std::collections::{BTreeMap, HashMap};

    mod admission_contract {
        use super::*;
        // Draft bytes for apps/lumen/src/raft_sm.rs's existing `#[cfg(test)] mod tests`.
        // The host status is observed through its public router after apply cleanup.

        use crate::change_admission::PendingChangeCapacity;
        use crate::change_budget::{ChangeBudget, HARD_LIMIT};
        use std::sync::{mpsc, Arc, Condvar, Mutex};
        use std::time::{Duration, Instant};
        use tower::ServiceExt;

        const TEST_LIMIT: Duration = Duration::from_secs(2);

        fn keyword_field() -> FieldSpec {
            FieldSpec {
                field_type: FieldType::Keyword,
                analyzer: None,
                multi: None,
                dim: None,
                metric: None,
                backend: None,
                quantize: None,
            }
        }

        fn keyword_schema() -> CreateCollectionRequest {
            CreateCollectionRequest {
                fields: BTreeMap::from([("email".to_string(), keyword_field())]),
            }
        }

        fn index_entry(id: &str) -> RaftLogEntry {
            RaftLogEntry::Index {
                collection_id: "docs".into(),
                req: IndexRequest {
                    items: vec![IndexItem {
                        external_id: id.into(),
                        field: "email".into(),
                        value: FieldValue::String(format!("{id}@example.test")),
                        version: None,
                    }],
                    request_id: None,
                },
            }
        }

        fn single_node(engine: Arc<Engine>) -> (tempfile::TempDir, Arc<RaftHost>, Arc<EngineSm>) {
            let dir = tempfile::tempdir().unwrap();
            let sm = EngineSm::new(engine, 0);
            let host = Arc::new(RaftHost::spawn(
                0,
                Membership {
                    voters: vec![0],
                    learners: vec![],
                },
                HashMap::new(),
                RaftStore::open(
                    dir.path().to_str().unwrap(),
                    0,
                    raft_runtime::FsyncPolicy::Os,
                )
                .unwrap(),
                sm.clone() as Arc<dyn RaftStateMachine>,
                HostConfig::default(),
            ));
            (dir, host, sm)
        }

        async fn raft_last_index(host: &RaftHost) -> u64 {
            let response = host
                .router()
                .oneshot(
                    axum::http::Request::builder()
                        .uri("/raftz")
                        .body(axum::body::Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), axum::http::StatusCode::OK);
            let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()["last_index"]
                .as_u64()
                .unwrap()
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn raft_full_local_admission_refuses_before_append_or_apply() {
            let budget = ChangeBudget::with_hard_limit(HARD_LIMIT);
            let engine = Arc::new(Engine::with_change_budget(budget.clone()));
            engine.create_collection("docs", keyword_schema()).unwrap();
            let used = budget.snapshot().total;
            let blocker = budget.owner().try_reserve(HARD_LIMIT - used).unwrap();
            let (_dir, host, sm) = single_node(engine.clone());
            let sink = RaftWriteSink::new(host.clone(), sm.clone());
            let before = engine.metrics().segment_backpressure_total.get();

            let mut submit = tokio::spawn(async move { sink.submit(index_entry("full")).await });
            let initial = tokio::time::timeout(TEST_LIMIT, &mut submit).await;
            let finished_before_release = initial.is_ok();

            // Always clear the synthetic Full state before inspecting status or joining
            // a delayed task. A correct pre-publication refusal has already returned.
            drop(blocker);
            let after_release = if finished_before_release {
                None
            } else {
                let joined = tokio::time::timeout(TEST_LIMIT, &mut submit).await;
                if joined.is_err() {
                    submit.abort();
                    let _ = tokio::time::timeout(TEST_LIMIT, &mut submit).await;
                }
                Some(joined)
            };
            let applied = sm.applied_index();
            // The submit task has returned or has been bounded and aborted. Query the
            // public router only now; never query it while a gated callback holds the
            // host node lock.
            let appended = raft_last_index(host.as_ref()).await;
            host.shutdown().await.unwrap();

            assert!(
                finished_before_release,
                "pre-publication Full must return before capacity is released"
            );
            assert!(after_release.is_none());
            let error = initial.unwrap().unwrap().unwrap_err();
            let is_capacity = error.downcast_ref::<PendingChangeCapacity>().is_some();
            let response =
                axum::response::IntoResponse::into_response(crate::api::ApiErr::from(error));
            assert!(is_capacity);
            assert_eq!(response.status(), axum::http::StatusCode::TOO_MANY_REQUESTS);
            assert_eq!(response.headers()[axum::http::header::RETRY_AFTER], "1");
            assert_eq!(
                engine.metrics().segment_backpressure_total.get(),
                before + 1,
                "one final raft pre-publication refusal increments once"
            );
            assert_eq!(appended, 0, "429 refusal must precede Raft append");
            assert_eq!(applied, 0, "429 refusal must precede state-machine apply");
            assert_eq!(budget.snapshot().reserved, 0);
        }

        fn wait_for_capacity_waiter(budget: &ChangeBudget) -> bool {
            let deadline = Instant::now() + TEST_LIMIT;
            while Instant::now() < deadline {
                if budget.has_capacity_waiters() {
                    return true;
                }
                std::thread::yield_now();
            }
            false
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn committed_raft_head_waits_without_holding_capture_barrier() {
            let budget = ChangeBudget::with_hard_limit(HARD_LIMIT);
            let engine = Arc::new(Engine::with_change_budget(budget.clone()));
            engine.create_collection("docs", keyword_schema()).unwrap();
            // The generic committed wait below must be reserved-only. Otherwise
            // the new owner can correctly publish this schema while the test is
            // checking its capture-barrier behavior.
            let schema_dir = tempfile::tempdir().unwrap();
            let schema_store = crate::segment_rdb::SegmentRdbStore::new(schema_dir.path()).unwrap();
            schema_store.save(&engine, 0).unwrap();
            assert_eq!(
                budget.snapshot().active,
                0,
                "schema checkpoint must freeze fixture work"
            );
            let used = budget.snapshot().total;
            let blocker = budget.owner().try_reserve(HARD_LIMIT - used).unwrap();
            let sm = EngineSm::new(engine.clone(), 0);
            let command = WalRecord::new(index_entry("committed")).encode().unwrap();

            let applying = {
                let sm = sm.clone();
                tokio::task::spawn_blocking(move || sm.apply(1, &command))
            };
            let waiter_seen = wait_for_capacity_waiter(&budget);
            let owner_started = engine.layer_maintenance.owner().is_some();

            // This probe is the lease assertion. It must complete before capacity is
            // released; a wait entered after CaptureBarrier::apply would block it.
            let (capture_tx, capture_rx) = mpsc::channel();
            let probe_engine = engine.clone();
            let probe = std::thread::spawn(move || {
                let stamp = probe_engine
                    .capture_barrier
                    .capture(0)
                    .map(|lease| lease.stamp().sequence);
                let _ = capture_tx.send(stamp);
            });
            let capture_before_release = capture_rx.recv_timeout(TEST_LIMIT);
            let applied_before_release = sm.applied_index();

            // Release and join before assertions. A correct implementation must wake,
            // consume the original committed command, and finish the blocking worker.
            drop(blocker);
            let apply_result = tokio::time::timeout(TEST_LIMIT, applying).await;
            probe.join().unwrap();
            let applied_after_release = sm.applied_index();
            let capture_cut = engine.capture_barrier.capture(0).unwrap().stamp().sequence;

            assert!(
                waiter_seen,
                "committed source must register a capacity wait"
            );
            assert!(
                owner_started,
                "committed Raft admission must start its independent capacity owner before waiting"
            );
            assert_eq!(
                capture_before_release.unwrap().unwrap(),
                0,
                "capture must complete while committed admission waits"
            );
            assert_eq!(applied_before_release, 0);
            assert!(apply_result.unwrap().unwrap().is_ok());
            assert_eq!(applied_after_release, 1);
            assert_eq!(
                capture_cut, 1,
                "record state and watermark share one interval"
            );
            assert!(budget.high_water_bytes() <= HARD_LIMIT);
        }

        #[derive(Default)]
        struct ApplyGate {
            state: Mutex<(bool, bool)>, // (entered, released)
            changed: Condvar,
        }
        impl ApplyGate {
            fn block_apply(&self) {
                let mut state = self.state.lock().unwrap();
                state.0 = true;
                self.changed.notify_all();
                while !state.1 {
                    state = self.changed.wait(state).unwrap();
                }
            }
            fn wait_entered(&self) -> bool {
                let deadline = Instant::now() + TEST_LIMIT;
                let mut state = self.state.lock().unwrap();
                while !state.0 {
                    let remaining = deadline.saturating_duration_since(Instant::now());
                    if remaining.is_zero() {
                        return false;
                    }
                    let (next, timed) = self.changed.wait_timeout(state, remaining).unwrap();
                    state = next;
                    if timed.timed_out() {
                        return state.0;
                    }
                }
                true
            }
            fn release(&self) {
                let mut state = self.state.lock().unwrap();
                state.1 = true;
                self.changed.notify_all();
            }
        }

        /// A real Raft state-machine wrapper. It stops only after the host has invoked
        /// apply, which is after the host allocated/appended the index. It does not
        /// manufacture an apply result or poll host state while the node lock is held.
        struct GatedSm {
            inner: Arc<EngineSm>,
            gate: Arc<ApplyGate>,
        }
        impl RaftStateMachine for GatedSm {
            fn admit_proposal(&self, command: &[u8]) -> anyhow::Result<Option<AdmissionPermit>> {
                self.inner.admit_proposal(command)
            }
            fn apply_admitted(
                &self,
                index: Index,
                command: &[u8],
                permit: Option<AdmissionPermit>,
            ) -> anyhow::Result<()> {
                self.gate.block_apply();
                self.inner.apply_admitted(index, command, permit)
            }
            fn apply(&self, index: Index, command: &[u8]) -> anyhow::Result<()> {
                self.apply_admitted(index, command, None)
            }
            fn snapshot(&self, writer: &mut dyn std::io::Write) -> anyhow::Result<()> {
                self.inner.snapshot(writer)
            }
            fn restore(&self, reader: &mut dyn std::io::Read) -> anyhow::Result<()> {
                self.inner.restore(reader)
            }
            fn applied_index(&self) -> Index {
                self.inner.applied_index()
            }
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn cancelled_raft_submit_keeps_reservation_through_actual_apply() {
            let budget = ChangeBudget::with_hard_limit(HARD_LIMIT);
            let engine = Arc::new(Engine::with_change_budget(budget.clone()));
            engine.create_collection("docs", keyword_schema()).unwrap();
            let baseline = budget.snapshot().total;
            let inner = EngineSm::new(engine.clone(), 0);
            let gate = Arc::new(ApplyGate::default());
            let dir = tempfile::tempdir().unwrap();
            let host = Arc::new(RaftHost::spawn(
                0,
                Membership {
                    voters: vec![0],
                    learners: vec![],
                },
                HashMap::new(),
                RaftStore::open(
                    dir.path().to_str().unwrap(),
                    0,
                    raft_runtime::FsyncPolicy::Os,
                )
                .unwrap(),
                Arc::new(GatedSm {
                    inner: inner.clone(),
                    gate: gate.clone(),
                }),
                HostConfig::default(),
            ));
            let sink = Arc::new(RaftWriteSink::new(host.clone(), inner.clone()));

            let mut submit = tokio::spawn({
                let sink = sink.clone();
                async move { sink.submit(index_entry("cancelled")).await }
            });
            let entered = gate.wait_entered();
            let reserved_before_cancel = budget.snapshot().reserved;
            submit.abort();
            let reserved_after_cancel = budget.snapshot().reserved;

            // Keep the gate closed while the caller is cancelled. Then release it and
            // bounded-join both the caller and the real host apply before assertions.
            gate.release();
            let submit_joined = tokio::time::timeout(TEST_LIMIT, &mut submit).await;
            let applied = tokio::time::timeout(TEST_LIMIT, async {
                while inner.applied_index() != 1 {
                    tokio::task::yield_now().await;
                }
            })
            .await;
            let reserved_after_apply = budget.snapshot().reserved;
            host.shutdown().await.unwrap();

            assert!(
                entered,
                "gate proves append reached the real host apply callback"
            );
            assert!(
                reserved_before_cancel > 0,
                "caller cancellation must not release post-append reservation"
            );
            assert!(
                reserved_after_cancel > 0,
                "reservation must survive cancellation while the real host apply is gated"
            );
            assert!(
                submit_joined.is_ok(),
                "cancelled caller task must finish after gate cleanup"
            );
            assert!(applied.is_ok());
            assert_eq!(engine.stats("docs").unwrap().documents_indexed, 1);
            assert_eq!(reserved_after_apply, 0);
            assert!(budget.snapshot().active >= baseline);
        }

        // Required commands after root places these bytes in raft_sm.rs:
        // cargo test -p lumen --lib raft_full_local_admission_refuses_before_append_or_apply
        // cargo test -p lumen --lib committed_raft_head_waits_without_holding_capture_barrier
        // cargo test -p lumen --lib cancelled_raft_submit_keeps_reservation_through_actual_apply
        // cargo test -p lumen --lib
        // cargo test -p lumen
    }

    fn number_field() -> FieldSpec {
        FieldSpec {
            field_type: FieldType::Number,
            analyzer: None,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        }
    }

    /// lumen's real `Engine`, driven through the shared `RaftHost`, applies
    /// committed commands and returns the rich `ApplyOutcome` (read-your-write).
    #[tokio::test]
    async fn engine_applies_through_the_shared_host() {
        let tmp = std::env::temp_dir().join(format!("lumen-enginesm-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&tmp);
        let engine = Arc::new(Engine::new());
        let sm = EngineSm::new(engine.clone(), 0);
        let host = RaftHost::spawn(
            0,
            Membership {
                voters: vec![0],
                learners: vec![],
            },
            HashMap::new(),
            RaftStore::open(tmp.to_str().unwrap(), 0, raft_runtime::FsyncPolicy::Os).unwrap(),
            sm.clone() as Arc<dyn RaftStateMachine>,
            HostConfig::default(),
        );

        // create a collection through consensus → rich Created outcome.
        let mut fields = BTreeMap::new();
        fields.insert("n".to_string(), number_field());
        let cmd = WalRecord::new(RaftLogEntry::CreateCollection {
            collection_id: "docs".into(),
            req: CreateCollectionRequest { fields },
        })
        .encode()
        .unwrap();
        let idx = host.propose(cmd).await.unwrap();
        assert_eq!(idx, 1);
        assert!(matches!(sm.take_outcome(1), Ok(ApplyOutcome::Created(_))));

        // index a doc → rich Indexed outcome, applied to the real engine.
        let cmd = WalRecord::new(RaftLogEntry::Index {
            collection_id: "docs".into(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: "d1".into(),
                    field: "n".into(),
                    value: FieldValue::Number(42.0),
                    version: None,
                }],
                request_id: None,
            },
        })
        .encode()
        .unwrap();
        let idx = host.propose(cmd).await.unwrap();
        assert_eq!(idx, 2);
        match sm.take_outcome(2) {
            Ok(ApplyOutcome::Indexed(r)) => assert_eq!(r.indexed, 1),
            other => panic!("expected Indexed, got {other:?}"),
        }
        // RYW: the engine reflects the applied write immediately.
        assert_eq!(sm.applied_index(), 2);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn truncate_control_record_applies_and_survives_a_raft_snapshot() {
        let engine = Arc::new(Engine::new());
        let sm = EngineSm::new(engine.clone(), 0);
        let mut fields = BTreeMap::new();
        fields.insert("n".to_string(), number_field());
        for (index, entry) in [
            RaftLogEntry::CreateCollection {
                collection_id: "docs".into(),
                req: CreateCollectionRequest { fields },
            },
            RaftLogEntry::Index {
                collection_id: "docs".into(),
                req: IndexRequest {
                    items: vec![IndexItem {
                        external_id: "old".into(),
                        field: "n".into(),
                        value: FieldValue::Number(42.0),
                        version: None,
                    }],
                    request_id: None,
                },
            },
            RaftLogEntry::TruncateDocs {
                collection_id: "docs".into(),
            },
        ]
        .into_iter()
        .enumerate()
        {
            sm.apply((index + 1) as u64, &WalRecord::new(entry).encode().unwrap())
                .unwrap();
        }
        assert!(matches!(
            sm.take_outcome(3),
            Ok(ApplyOutcome::DocsTruncated)
        ));
        assert_eq!(engine.stats("docs").unwrap().documents_indexed, 0);

        let mut bytes = Vec::new();
        sm.snapshot(&mut bytes).unwrap();
        let restored_engine = Arc::new(Engine::new());
        let restored = EngineSm::new(restored_engine.clone(), 0);
        restored.restore(&mut bytes.as_slice()).unwrap();
        assert_eq!(restored.applied_index(), 3);
        assert_eq!(restored_engine.stats("docs").unwrap().documents_indexed, 0);
    }

    #[test]
    fn unindex_control_record_applies_and_survives_a_raft_snapshot() {
        let engine = Arc::new(Engine::new());
        let sm = EngineSm::new(engine.clone(), 0);
        let mut fields = BTreeMap::new();
        fields.insert("n".to_string(), number_field());
        for (index, entry) in [
            RaftLogEntry::CreateCollection {
                collection_id: "docs".into(),
                req: CreateCollectionRequest { fields },
            },
            RaftLogEntry::Index {
                collection_id: "docs".into(),
                req: IndexRequest {
                    items: vec![
                        IndexItem {
                            external_id: "remove".into(),
                            field: "n".into(),
                            value: FieldValue::Number(1.0),
                            version: None,
                        },
                        IndexItem {
                            external_id: "keep".into(),
                            field: "n".into(),
                            value: FieldValue::Number(2.0),
                            version: None,
                        },
                    ],
                    request_id: None,
                },
            },
            RaftLogEntry::UnindexDocs {
                collection_id: "docs".into(),
                req: BatchUnindexDocsRequest {
                    external_ids: vec!["remove".into()],
                },
            },
        ]
        .into_iter()
        .enumerate()
        {
            sm.apply((index + 1) as u64, &WalRecord::new(entry).encode().unwrap())
                .unwrap();
        }
        assert!(matches!(
            sm.take_outcome(3),
            Ok(ApplyOutcome::DocsUnindexed)
        ));
        assert_eq!(engine.stats("docs").unwrap().documents_indexed, 1);

        let mut bytes = Vec::new();
        sm.snapshot(&mut bytes).unwrap();
        let restored_engine = Arc::new(Engine::new());
        let restored = EngineSm::new(restored_engine.clone(), 0);
        restored.restore(&mut bytes.as_slice()).unwrap();
        assert_eq!(restored.applied_index(), 3);
        assert_eq!(restored_engine.stats("docs").unwrap().documents_indexed, 1);
    }
}
// CODEGEN-END
