// CODEGEN-BEGIN
//! Write coordinator — the seam between the HTTP write handlers and the
//! log-driven apply loop.
//!
//! Design (see `wal` for the why): a write handler does **not** touch
//! the index. It calls [`WriteCoordinator::submit`], which:
//!
//! 1. publishes the mutation to the [`WalLog`] (the log assigns a global
//!    sequence — the total order),
//! 2. waits until **this node's** apply loop has folded the stream up to
//!    that sequence (read-your-write), and
//! 3. returns the [`ApplyOutcome`] the apply loop computed for it.
//!
//! Apply happens in exactly one place — the background loop subscribed to
//! the log — so every node converges by applying the same ordered
//! stream, and the node that received the write holds no special state.
//! For [`MemWal`](crate::wal::MemWal) the loop runs in-process and the
//! round-trip is sub-millisecond, so single-node writes feel synchronous
//! and existing tests see their writes immediately.
//!
//! Apply errors (e.g. a type mismatch caught at apply time) are routed
//! back as the original `anyhow::Error` — carrying the `StorageError` —
//! so the handler still maps them to the right HTTP status.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::{bail, Result};
use futures::{FutureExt, StreamExt};
use raft_runtime::OutcomeWindow;
use rustc_hash::FxHashMap;
use tokio::sync::{oneshot, OwnedRwLockReadGuard, OwnedRwLockWriteGuard, RwLock};

use crate::log_entry::RaftLogEntry;
use crate::storage::{ApplyOutcome, Engine};
use crate::wal::{SharedWal, WalRecord};

/// How many recent outcomes to retain, via [`OutcomeWindow`]. A publisher
/// reads its outcome within microseconds of the apply loop reaching its
/// sequence, far inside this window; outcomes for sequences no local
/// handler is waiting on (writes that originated on other nodes) age out.
const OUTCOME_WINDOW: u64 = 8192;
const APPLY_LOOP_BATCH: usize = 128;
/// Bound on `submit()`'s wait for local apply (#1486 R2). Comfortably above
/// any realistic single-batch apply latency (`APPLY_LOOP_BATCH` folds
/// synchronously in one `spawn_blocking` task), so this only fires on a
/// genuine stall — turning what would otherwise be an infinite hang into a
/// retryable 5xx.
const SUBMIT_TIMEOUT_SECS: u64 = 30;
const SUBMIT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(SUBMIT_TIMEOUT_SECS);

struct PendingApply {
    seq: u64,
    rec: WalRecord,
    aof_rec: Option<WalRecord>,
}

/// A `submit()` waiter was released without a genuine [`ApplyOutcome`]
/// (#1486 R2): either the apply loop's redelivery-dedup guard skipped the
/// waiter's sequence (already at/below `applied`), or the wait exceeded
/// [`SUBMIT_TIMEOUT`]. Both are transient/retryable, never a client
/// input error — `src/api.rs`'s `From<anyhow::Error> for ApiErr` downcasts
/// this to a `503` instead of falling through to the generic `400`
/// default, so a stranded write is loud (a 5xx) rather than silent (an
/// infinite hang, the original defect) or misleading (a 4xx).
#[derive(Debug, Clone)]
pub struct SubmitStalled(pub String);

impl std::fmt::Display for SubmitStalled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for SubmitStalled {}

/// A durable write path (local AOF append/flush/sync, a segment/RDB
/// checkpoint save, or — under the `raft-wal` feature — a raft log append)
/// hit `io::ErrorKind::StorageFull` (ENOSPC) or a wrapped equivalent (#2516).
/// Reported as a distinct, stable error so `src/api.rs`'s
/// `From<anyhow::Error> for ApiErr` maps it to `507 Insufficient Storage`
/// with the machine-readable `storage_full` code instead of falling through
/// to the generic `400` default. Every origin that produces one MUST first
/// call `Metrics::mark_storage_degraded` — this type only carries the
/// message, it does not itself flip the sticky degraded flag.
#[derive(Debug, Clone)]
pub struct StorageFullError(pub String);

impl std::fmt::Display for StorageFullError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for StorageFullError {}

/// #2516: true when `e`'s error chain contains an `io::Error` whose kind is
/// `StorageFull` (ENOSPC) — the seam every durable-write call site (AOF
/// persist, segment/RDB checkpoint save, raft log append) probes to decide
/// whether to flip the node into degraded read-only mode. Walks the full
/// `anyhow` context chain (not just the outer error) because every durable
/// write path wraps the root `std::io::Error` with `.context(...)`.
pub fn is_storage_full(e: &anyhow::Error) -> bool {
    e.chain().any(|cause| {
        cause
            .downcast_ref::<std::io::Error>()
            .is_some_and(|io_e| io_e.kind() == std::io::ErrorKind::StorageFull)
    })
}

struct CompletionState {
    outcomes: OutcomeWindow<Result<ApplyOutcome>>,
    waiters: FxHashMap<u64, oneshot::Sender<Result<ApplyOutcome>>>,
    /// A permit moves here immediately after publish and stays bound to the
    /// sequence until the apply loop completes it. Caller cancellation or a
    /// submit timeout only drops the waiter; it must never open the restore
    /// fence while the published record can still apply later.
    mutation_permits: FxHashMap<u64, OwnedRwLockReadGuard<()>>,
}

/// The optional local AOF the apply loop appends every applied record to (Stage
/// 2 Phase 2f-3). Wrapped in a `Mutex` because the apply loop appends from the
/// async task while the periodic checkpoint snapshotter calls `truncate_through`
/// from another task. The default writer fsyncs each successfully applied
/// record before the apply loop acknowledges its request, so an immediate
/// replacement process can recover every acknowledged write.
/// `None` on the default / non-AOF path, so `start_from` is byte-identical to
/// today.
pub type SharedAof = Arc<Mutex<crate::aof::AofWriter>>;

pub struct WriteCoordinator {
    wal: SharedWal,
    applied: AtomicU64,
    completions: Mutex<CompletionState>,
    /// Serializes one Standalone-wide replacement against ordinary writes.
    ///
    /// `submit` holds a shared permit from publish through local apply. A
    /// durable restore takes the exclusive permit, which therefore starts only
    /// after every earlier submit has completed and prevents every later submit
    /// from publishing until activation finishes. This fence is sufficient for
    /// the embedded single-process Standalone path. External producers that can
    /// publish directly to a shared WAL remain outside its contract.
    mutation_gate: Arc<RwLock<()>>,
}

impl WriteCoordinator {
    /// Spawn the apply loop and return the coordinator. The loop tails
    /// the log from the beginning and folds it into `engine`.
    pub fn start(wal: SharedWal, engine: Arc<Engine>) -> Arc<Self> {
        Self::start_from(wal, engine, 0)
    }

    /// Like [`start`](Self::start) but begins applying after `from_seq`
    /// — used when a snapshot (RDB) already seeded the engine up to that
    /// sequence.
    pub fn start_from(wal: SharedWal, engine: Arc<Engine>, from_seq: u64) -> Arc<Self> {
        // The default / non-AOF path. Delegates with no AOF, so the apply loop is
        // byte-identical to today.
        Self::start_from_inner(wal, engine, from_seq, None)
    }

    /// Like [`start_from`](Self::start_from) but also appends every APPLIED
    /// `(seq, record)` to a local AOF (Stage 2 Phase 2f-3), AFTER the apply
    /// succeeds and `applied` advances. The default / non-AOF path is unchanged —
    /// this is the only entry point that wires an AOF in.
    pub fn start_from_with_aof(
        wal: SharedWal,
        engine: Arc<Engine>,
        from_seq: u64,
        aof: SharedAof,
    ) -> Arc<Self> {
        Self::start_from_inner(wal, engine, from_seq, Some(aof))
    }

    /// The apply-loop spawner. Identical structure regardless of the AOF; the
    /// only AOF-specific work is the append after `complete`, conditioned on the
    /// `aof` being `Some` (the default `start_from` passes `None`).
    fn start_from_inner(
        wal: SharedWal,
        engine: Arc<Engine>,
        from_seq: u64,
        aof: Option<SharedAof>,
    ) -> Arc<Self> {
        let coord = Arc::new(Self {
            wal: wal.clone(),
            applied: AtomicU64::new(from_seq),
            completions: Mutex::new(CompletionState {
                outcomes: OutcomeWindow::new(OUTCOME_WINDOW),
                waiters: FxHashMap::default(),
                mutation_permits: FxHashMap::default(),
            }),
            mutation_gate: Arc::new(RwLock::new(())),
        });
        let loop_coord = coord.clone();
        tokio::spawn(async move {
            let mut backoff = std::time::Duration::from_millis(100);
            // Outer loop: re-subscribe from the last-applied sequence whenever
            // the stream ends or the subscribe fails. An external-log restart can tear
            // down our ephemeral subscription, so the apply loop MUST recreate
            // it and resume tailing — otherwise writes silently stop applying
            // after a broker blip. Resuming from `applied` is safe:
            // redelivery is skipped idempotently below.
            loop {
                let from = loop_coord.applied.load(Ordering::Acquire);
                let mut sub = match wal.subscribe(from).await {
                    Ok(s) => {
                        backoff = std::time::Duration::from_millis(100);
                        s
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, from, "apply loop: subscribe failed; retrying");
                        tokio::time::sleep(backoff).await;
                        backoff = (backoff * 2).min(std::time::Duration::from_secs(5));
                        continue;
                    }
                };
                while let Some(item) = sub.next().await {
                    let mut stream_ended = false;
                    match item {
                        Ok((seq, rec)) => {
                            // Idempotent under redelivery: skip anything at or
                            // below what we've already applied. Defense-in-depth
                            // (#1486): a skipped sequence never reaches `complete`,
                            // so any local waiter for it (a submit() that published
                            // this exact seq) would otherwise hang forever — release
                            // it with a distinct, retryable error instead.
                            if seq <= loop_coord.applied.load(Ordering::Acquire) {
                                loop_coord.complete_stale(seq);
                                continue;
                            }
                            let mut batch = Vec::with_capacity(APPLY_LOOP_BATCH);
                            batch.push(PendingApply {
                                seq,
                                aof_rec: aof.as_ref().map(|_| rec.clone()),
                                rec,
                            });
                            while batch.len() < APPLY_LOOP_BATCH {
                                match sub.next().now_or_never() {
                                    Some(Some(Ok((seq, rec)))) => {
                                        if seq <= loop_coord.applied.load(Ordering::Acquire) {
                                            loop_coord.complete_stale(seq);
                                            continue;
                                        }
                                        batch.push(PendingApply {
                                            seq,
                                            aof_rec: aof.as_ref().map(|_| rec.clone()),
                                            rec,
                                        });
                                    }
                                    Some(Some(Err(e))) => {
                                        tracing::warn!(
                                            error = %e,
                                            "apply loop: stream item error"
                                        );
                                    }
                                    Some(None) => {
                                        stream_ended = true;
                                        break;
                                    }
                                    None => break,
                                }
                            }
                            // apply_raft_entry is synchronous and CPU-bound (a bulk index folds
                            // thousands of items + BM25 stats). Run ready records in one blocking
                            // task to keep broker I/O moving without paying a thread handoff per WAL
                            // record.
                            let eng = engine.clone();
                            let seqs: Vec<u64> = batch.iter().map(|pending| pending.seq).collect();
                            let results = match tokio::task::spawn_blocking(move || {
                                let mut results = Vec::with_capacity(batch.len());
                                for pending in batch {
                                    let outcome = eng.apply_raft_entry(pending.rec.entry);
                                    results.push((pending.seq, outcome, pending.aof_rec));
                                }
                                results
                            })
                            .await
                            {
                                Ok(results) => results,
                                Err(e) => {
                                    let err = e.to_string();
                                    seqs.into_iter()
                                        .map(|seq| {
                                            (
                                                seq,
                                                Err(anyhow::anyhow!(
                                                    "apply batch task panicked: {err}"
                                                )),
                                                None,
                                            )
                                        })
                                        .collect()
                                }
                            };

                            for (seq, mut outcome, aof_rec) in results {
                                if let Err(e) = &outcome {
                                    tracing::warn!(seq, error = %e, "apply error (entry no-ops)");
                                }
                                // The AOF is the sole recoverable tail in the embedded segment
                                // path. `AofWriter` deliberately buffers writes, so completing
                                // the HTTP waiter before `flush` creates a restart window: a pod
                                // can be deleted after a 2xx response while the record still only
                                // lives in this process's BufWriter. Persist it through the OS
                                // before publishing `applied` or acknowledging the caller. The
                                // production writer uses `Always`, so `append` crosses the fsync
                                // boundary here. A graceful SIGTERM also performs a final sync.
                                if outcome.is_ok() {
                                    if let (Some(aof), Some(rec)) = (aof.as_ref(), aof_rec) {
                                        let persisted = {
                                            let mut writer =
                                                aof.lock().expect("aof writer poisoned");
                                            writer
                                                .append(seq, &rec)
                                                .and_then(|()| writer.flush())
                                                .and_then(|()| writer.maybe_sync())
                                        };
                                        if let Err(e) = persisted {
                                            if is_storage_full(&e) {
                                                // #2516: local disk is out of
                                                // space. Flip the sticky
                                                // degraded flag so every
                                                // subsequent mutating request
                                                // fast-fails before touching
                                                // this path again, and report
                                                // a distinct, stable error so
                                                // the caller sees 507
                                                // Insufficient Storage rather
                                                // than a generic 400.
                                                tracing::error!(
                                                    seq,
                                                    error = %e,
                                                    "AOF persist failed: ENOSPC — entering degraded read-only mode"
                                                );
                                                engine.metrics().mark_storage_degraded();
                                                outcome = Err(anyhow::Error::new(
                                                    StorageFullError(format!(
                                                        "local storage is full (ENOSPC) persisting sequence {seq}; \
                                                         node entered degraded read-only mode"
                                                    )),
                                                ));
                                            } else {
                                                tracing::warn!(
                                                    seq,
                                                    error = %e,
                                                    "AOF persist failed"
                                                );
                                                outcome = Err(e.context(
                                                    "persist applied record to local AOF",
                                                ));
                                            }
                                        }
                                    }
                                }
                                loop_coord.complete(seq, outcome);
                            }
                        }
                        Err(e) => tracing::warn!(error = %e, "apply loop: stream item error"),
                    }
                    if stream_ended {
                        break;
                    }
                }
                // Stream ended (e.g. external-log restart killed the ephemeral
                // consumer). Re-subscribe from the applied head after a short
                // pause so we don't tight-spin if the broker is flapping.
                tracing::warn!("apply loop: stream ended; re-subscribing from applied seq");
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
        });
        coord
    }

    /// Defense-in-depth (#1486): release any waiter stranded on a sequence
    /// the apply loop's redelivery-dedup guard is about to skip (already at
    /// or below `applied`). Unlike [`complete`](Self::complete), this is
    /// NOT reporting a real apply outcome — the dedup guard means this
    /// sequence's record was never folded into the engine on this pass, so
    /// there is no genuine `ApplyOutcome` to hand back. A distinct,
    /// explicitly-retryable error lets the caller's error message (and any
    /// HTTP status mapping) tell this apart from a real apply failure, and
    /// — critically — completes the waiter at all, instead of leaving
    /// `submit()`'s `rx.await` hanging forever. Deliberately does not touch
    /// `applied` or the outcomes window: `seq` is already accounted for by
    /// the watermark, so there is nothing further to record.
    fn complete_stale(&self, seq: u64) {
        let (waiter, mutation_permit) = {
            let mut m = self.completions.lock().expect("completions poisoned");
            (m.waiters.remove(&seq), m.mutation_permits.remove(&seq))
        };
        if let Some(tx) = waiter {
            let _ = tx.send(Err(anyhow::Error::new(SubmitStalled(format!(
                "sequence {seq} arrived at or below the applied watermark (stale redelivery \
                or a sequence-domain mismatch); the write was not applied on this pass — retry"
            )))));
        }
        drop(mutation_permit);
    }

    fn complete(&self, seq: u64, outcome: Result<ApplyOutcome>) {
        let mut direct = None;
        let mutation_permit;
        {
            let mut m = self.completions.lock().expect("completions poisoned");
            mutation_permit = m.mutation_permits.remove(&seq);
            if let Some(tx) = m.waiters.remove(&seq) {
                direct = Some((tx, outcome));
            } else {
                m.outcomes.insert(seq, outcome);
            }
            // Prune everything older than the retention window.
            m.outcomes.advance(seq);
            // Publish the new applied head while the completion lock still
            // hides the outcome from register_waiter. Otherwise an apply that
            // wins the publish/register race can expose its outcome, let the
            // caller-owned permit drop, and open the restore fence before this
            // watermark advances.
            self.applied.store(seq, Ordering::Release);
        }
        // The restore fence may open only after the applied watermark above
        // describes this completed record. The caller future may already have
        // been cancelled or timed out; the permit is sequence-owned here.
        drop(mutation_permit);
        if let Some((tx, outcome)) = direct {
            let _ = tx.send(outcome);
        }
    }

    fn register_waiter(
        &self,
        seq: u64,
        mutation_permit: OwnedRwLockReadGuard<()>,
    ) -> Result<oneshot::Receiver<Result<ApplyOutcome>>> {
        let mut m = self.completions.lock().expect("completions poisoned");
        if let Some(result) = m.outcomes.claim(seq) {
            let (tx, rx) = oneshot::channel();
            let _ = tx.send(result);
            return Ok(rx);
        }
        if seq <= self.applied.load(Ordering::Acquire) {
            let (tx, rx) = oneshot::channel();
            let _ = tx.send(Err(anyhow::Error::new(SubmitStalled(format!(
                "sequence {seq} completed as a stale redelivery before its waiter registered; \
                 the write was not applied on this pass — retry"
            )))));
            return Ok(rx);
        }
        let (tx, rx) = oneshot::channel();
        if m.waiters.contains_key(&seq) {
            bail!("duplicate waiter for sequence {seq}");
        }
        if m.mutation_permits.contains_key(&seq) {
            bail!("duplicate mutation permit for sequence {seq}");
        }
        m.waiters.insert(seq, tx);
        m.mutation_permits.insert(seq, mutation_permit);
        Ok(rx)
    }

    /// Publish `entry`, wait for local apply, and return its outcome.
    ///
    /// Bounded by [`SUBMIT_TIMEOUT`] (#1486 R2, defense-in-depth): a stray
    /// sequence-domain mismatch (the class R1 fixes) or any other apply-loop
    /// stall must surface as a retryable 5xx to the caller, never an
    /// unbounded hang that leaks a server task per request.
    pub async fn submit(&self, entry: RaftLogEntry) -> Result<ApplyOutcome> {
        // Keep the shared permit through publish AND local apply. An exclusive
        // restore fence can therefore observe one exact applied/WAL boundary:
        // no earlier submit remains in flight and no later submit has obtained
        // a sequence yet.
        let mutation_permit = self.mutation_gate.clone().read_owned().await;
        let seq = self.wal.publish(WalRecord::new(entry)).await?;
        let rx = self.register_waiter(seq, mutation_permit)?;
        match tokio::time::timeout(SUBMIT_TIMEOUT, rx).await {
            Ok(Ok(outcome)) => outcome,
            Ok(Err(_)) => Err(anyhow::anyhow!(
                "apply loop stopped before sequence {seq} was applied"
            )),
            Err(_) => {
                // The waiter entry may still be sitting in `completions.waiters`
                // (a very-late `complete`/`complete_stale` will just find no live
                // receiver and drop the result) — nothing to clean up here beyond
                // returning the bounded error.
                Err(anyhow::Error::new(SubmitStalled(format!(
                    "timed out after {SUBMIT_TIMEOUT_SECS}s waiting for sequence {seq} to apply"
                ))))
            }
        }
    }

    /// Highest sequence this node has applied.
    pub fn applied_seq(&self) -> u64 {
        self.applied.load(Ordering::Acquire)
    }

    /// Fence every [`Self::submit`] call in this process.
    ///
    /// The returned owned guard keeps the fence closed until it is dropped.
    /// Tokio's fair write-preferring queue also prevents a stream of new
    /// submits from starving a waiting restore.
    pub async fn fence_mutations(&self) -> OwnedRwLockWriteGuard<()> {
        self.mutation_gate.clone().write_owned().await
    }
}

/// The write seam the API binds to: submit a log entry, get its applied outcome,
/// and report the applied head. Implemented by [`WriteCoordinator`] (the WAL-seam
/// path for embedded/nats) and by `RaftWriteSink` (the raft-runtime path).
#[async_trait::async_trait]
pub trait WriteSink: Send + Sync {
    async fn submit(&self, entry: RaftLogEntry) -> Result<ApplyOutcome>;
    fn applied_seq(&self) -> u64;
}

#[async_trait::async_trait]
impl WriteSink for WriteCoordinator {
    async fn submit(&self, entry: RaftLogEntry) -> Result<ApplyOutcome> {
        WriteCoordinator::submit(self, entry).await
    }
    fn applied_seq(&self) -> u64 {
        WriteCoordinator::applied_seq(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
    };
    use crate::wal::MemWal;
    use std::collections::BTreeMap as Map;

    fn keyword_schema() -> CreateCollectionRequest {
        let mut fields = Map::new();
        fields.insert(
            "email".to_string(),
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

    #[tokio::test]
    async fn submit_creates_then_indexes_and_outcome_is_routed_back() {
        let engine = Arc::new(Engine::new());
        let wal = Arc::new(MemWal::new());
        let coord = WriteCoordinator::start(wal, engine.clone());

        let created = coord
            .submit(RaftLogEntry::CreateCollection {
                collection_id: "u".into(),
                req: keyword_schema(),
            })
            .await
            .unwrap();
        match created {
            ApplyOutcome::Created(r) => {
                assert_eq!(r.version, 1);
                assert_eq!(r.fields_count, 1);
            }
            other => panic!("expected Created, got {other:?}"),
        }

        let indexed = coord
            .submit(RaftLogEntry::Index {
                collection_id: "u".into(),
                req: IndexRequest {
                    items: vec![IndexItem {
                        external_id: "u1".into(),
                        field: "email".into(),
                        value: FieldValue::String("a@x.com".into()),
                        version: None,
                    }],
                    request_id: None,
                },
            })
            .await
            .unwrap();
        match indexed {
            ApplyOutcome::Indexed(r) => assert_eq!(r.indexed, 1),
            other => panic!("expected Indexed, got {other:?}"),
        }

        // The write is visible via a direct engine read (read-your-write).
        assert_eq!(engine.stats("u").unwrap().documents_indexed, 1);
    }

    #[tokio::test]
    async fn exclusive_mutation_fence_blocks_publish_until_released() {
        let engine = Arc::new(Engine::new());
        let wal = Arc::new(MemWal::new());
        let coord = WriteCoordinator::start(wal, engine);
        let fence = coord.fence_mutations().await;

        let mut pending = {
            let coord = coord.clone();
            tokio::spawn(async move {
                coord
                    .submit(RaftLogEntry::CreateCollection {
                        collection_id: "u".into(),
                        req: keyword_schema(),
                    })
                    .await
            })
        };

        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(50), &mut pending)
                .await
                .is_err(),
            "a submit must remain blocked while the exclusive fence is held"
        );
        assert_eq!(coord.applied_seq(), 0, "blocked submit must not publish");

        drop(fence);
        let outcome = tokio::time::timeout(std::time::Duration::from_secs(5), pending)
            .await
            .expect("submit must resume after the fence is released")
            .expect("submit task must not panic")
            .expect("submit must succeed");
        assert!(matches!(outcome, ApplyOutcome::Created(_)));
        assert_eq!(coord.applied_seq(), 1);
    }

    #[tokio::test]
    async fn cancelled_submit_keeps_fence_closed_until_sequence_completes() {
        let engine = Arc::new(Engine::new());
        let wal = Arc::new(MemWal::new());
        let coord = WriteCoordinator::start(wal, engine);

        // Model the exact post-publish state directly: the caller owns a
        // waiter, while the completion table owns the mutation permit for the
        // published sequence. Cancelling the caller drops only the receiver.
        let permit = coord.mutation_gate.clone().read_owned().await;
        let waiter = coord.register_waiter(1, permit).unwrap();
        drop(waiter);

        let mut fence = Box::pin(coord.fence_mutations());
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(50), &mut fence)
                .await
                .is_err(),
            "caller cancellation must not open the fence before apply completion"
        );

        coord.complete(1, Err(anyhow::anyhow!("synthetic apply failure")));
        let _guard = tokio::time::timeout(std::time::Duration::from_secs(5), fence)
            .await
            .expect("the fence must open after the sequence completes");
        assert_eq!(coord.applied_seq(), 1);
    }

    /// The embedded segment path may be replaced immediately after its write
    /// endpoint returns. Its AOF must therefore be readable *at acknowledgement
    /// time*, rather than waiting for writer drop.
    /// This is the exact local half of the single-replica pod-restart contract:
    /// fresh engine -> replay local AOF -> collection and indexed document.
    #[tokio::test]
    async fn embedded_aof_is_replayable_when_submit_acknowledges_a_write() {
        let dir = tempfile::tempdir().unwrap();
        let aof_path = dir.path().join("aof.log");
        let aof = Arc::new(Mutex::new(crate::aof::AofWriter::open(&aof_path).unwrap()));
        let engine = Arc::new(Engine::new());
        let wal = Arc::new(MemWal::new());
        let coord = WriteCoordinator::start_from_with_aof(wal, engine, 0, aof);

        coord
            .submit(RaftLogEntry::CreateCollection {
                collection_id: "u".into(),
                req: keyword_schema(),
            })
            .await
            .unwrap();
        coord
            .submit(RaftLogEntry::Index {
                collection_id: "u".into(),
                req: IndexRequest {
                    items: vec![IndexItem {
                        external_id: "u1".into(),
                        field: "email".into(),
                        value: FieldValue::String("u1@example.test".into()),
                        version: None,
                    }],
                    request_id: None,
                },
            })
            .await
            .unwrap();

        // Do not flush or sync the test's writer: a returned write itself is
        // the contract boundary. Before the fix this reader saw an empty AOF
        // because both frames remained in the process-local BufWriter.
        let mut seqs = Vec::new();
        crate::aof::AofReader::replay(&aof_path, 0, |seq, _| seqs.push(seq)).unwrap();
        assert_eq!(seqs, vec![1, 2]);

        let restarted = Arc::new(Engine::new());
        assert_eq!(
            crate::aof::replay_aof_into(&restarted, &aof_path, 0).unwrap(),
            2
        );
        assert_eq!(restarted.stats("u").unwrap().documents_indexed, 1);
    }

    /// #2516: prove the REAL ENOSPC detection/classification/metrics path
    /// end to end, through the actual production write path — not a
    /// parallel fake. Uses `AofWriter::set_inject_storage_full` (the
    /// `#[cfg(test)]` fault-injection seam on the real `AofWriter::append`,
    /// scoped to this test's own writer instance so parallel test threads
    /// never cross-contaminate) so the apply loop's genuine
    /// AOF-persist-failure branch runs.
    #[tokio::test]
    async fn aof_enospc_returns_storage_full_error_and_marks_degraded() {
        let dir = tempfile::tempdir().unwrap();
        let aof_path = dir.path().join("aof.log");
        let aof = Arc::new(Mutex::new(crate::aof::AofWriter::open(&aof_path).unwrap()));
        let engine = Arc::new(Engine::new());
        let wal = Arc::new(MemWal::new());
        let coord = WriteCoordinator::start_from_with_aof(wal, engine.clone(), 0, aof.clone());

        // A normal write before the disk fills must succeed and must not
        // touch the degraded flag.
        coord
            .submit(RaftLogEntry::CreateCollection {
                collection_id: "u".into(),
                req: keyword_schema(),
            })
            .await
            .unwrap();
        assert!(!engine.metrics().is_storage_degraded());

        // Arm the fault injection: the next AofWriter::append hits a
        // synthetic ENOSPC, exercising the real coordinator apply-loop
        // branch that classifies it and flips the sticky flag.
        aof.lock().unwrap().set_inject_storage_full(true);
        let err = coord
            .submit(RaftLogEntry::Index {
                collection_id: "u".into(),
                req: IndexRequest {
                    items: vec![IndexItem {
                        external_id: "u1".into(),
                        field: "email".into(),
                        value: FieldValue::String("a@x.com".into()),
                        version: None,
                    }],
                    request_id: None,
                },
            })
            .await
            .unwrap_err();
        aof.lock().unwrap().set_inject_storage_full(false);

        assert!(
            err.downcast_ref::<StorageFullError>().is_some(),
            "expected StorageFullError, got: {err}"
        );
        assert!(
            engine.metrics().is_storage_degraded(),
            "ENOSPC on the AOF write path must flip the sticky degraded gauge"
        );
        assert_eq!(engine.metrics().storage_full_errors_total.get(), 1);

        // Probe/clear (what the periodic re-probe does once space returns):
        // writes must resume once the flag is cleared.
        engine.metrics().clear_storage_degraded();
        let indexed = coord
            .submit(RaftLogEntry::Index {
                collection_id: "u".into(),
                req: IndexRequest {
                    items: vec![IndexItem {
                        external_id: "u2".into(),
                        field: "email".into(),
                        value: FieldValue::String("b@x.com".into()),
                        version: None,
                    }],
                    request_id: None,
                },
            })
            .await
            .unwrap();
        assert!(matches!(indexed, ApplyOutcome::Indexed(r) if r.indexed == 1));
        assert!(!engine.metrics().is_storage_degraded());
    }

    #[tokio::test]
    async fn submit_propagates_apply_error_with_type() {
        use crate::storage::StorageError;
        let engine = Arc::new(Engine::new());
        let wal = Arc::new(MemWal::new());
        let coord = WriteCoordinator::start(wal, engine.clone());

        // Index into a collection that doesn't exist → CollectionNotFound,
        // and the error must survive routing (downcast still works).
        let err = coord
            .submit(RaftLogEntry::Index {
                collection_id: "ghost".into(),
                req: IndexRequest {
                    items: vec![IndexItem {
                        external_id: "x".into(),
                        field: "email".into(),
                        value: FieldValue::String("a@x.com".into()),
                        version: None,
                    }],
                    request_id: None,
                },
            })
            .await
            .unwrap_err();
        assert!(
            err.downcast_ref::<StorageError>()
                .map(|e| matches!(e, StorageError::CollectionNotFound(_)))
                .unwrap_or(false),
            "StorageError must survive coordinator routing, got: {err}"
        );
    }

    /// #1486 AC1/AC2: an engine "restored" to a non-zero watermark (mirrors
    /// `serve()`'s `MemWal::starting_at(start_seq)` + `start_from(engine,
    /// start_seq)` pairing, whatever the restore source — segment checkpoint,
    /// AOF-tail replay, or CBOR RDB) accepts its first subsequent write
    /// immediately (no waiter leak) and that write is durable + reflected in
    /// stats/metrics, not stranded behind a stale watermark.
    #[tokio::test]
    async fn restore_seeds_wal_above_watermark_first_write_completes_promptly() {
        let engine = Arc::new(Engine::new());
        // Pre-restore state: schema already present (as a real checkpoint
        // restore would leave it), engine otherwise fresh.
        engine.create_collection("u", keyword_schema()).unwrap();

        const RESTORED_WATERMARK: u64 = 5;
        let wal = Arc::new(MemWal::starting_at(RESTORED_WATERMARK));
        let coord = WriteCoordinator::start_from(wal, engine.clone(), RESTORED_WATERMARK);
        assert_eq!(coord.applied_seq(), RESTORED_WATERMARK);

        let outcome = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            coord.submit(RaftLogEntry::Index {
                collection_id: "u".into(),
                req: IndexRequest {
                    items: vec![IndexItem {
                        external_id: "post-restore-1".into(),
                        field: "email".into(),
                        value: FieldValue::String("fresh@x.com".into()),
                        version: None,
                    }],
                    request_id: None,
                },
            }),
        )
        .await
        .expect("first post-restore write must complete promptly, not hang (#1486)")
        .expect("first post-restore write must succeed");

        match outcome {
            ApplyOutcome::Indexed(r) => assert_eq!(r.indexed, 1),
            other => panic!("expected Indexed, got {other:?}"),
        }

        // Durable + reflected in read-your-write state.
        let stats = engine.stats("u").unwrap();
        assert_eq!(
            stats.documents_indexed, 1,
            "the fresh post-restore doc must be counted"
        );
        assert!(
            stats.last_indexed_at.is_some(),
            "last_indexed_at must advance for a genuinely-applied post-restore write"
        );

        // Searchable: the apply loop actually folded the write into the
        // engine (not silently dropped by the dedup guard).
        assert!(
            engine.metrics().index_writes_total.get() >= 1,
            "lumen_index_writes_total must advance for a genuinely-applied post-restore write"
        );
        assert!(
            engine.metrics().index_bytes_total.get() > 0,
            "lumen_index_bytes_total must advance for a genuinely-applied post-restore write"
        );

        // The WAL's own sequence domain is strictly above the restored
        // watermark, and the coordinator's applied head advanced past it.
        assert!(coord.applied_seq() > RESTORED_WATERMARK);
    }

    /// #1486 documents the defect class R1 fixes: pairing a non-zero
    /// `start_from` watermark with an UNSEEDED `MemWal::new()` (base 0) — the
    /// pre-fix `serve()` wiring. The stale sequence must fail promptly without
    /// applying. `MemWal::starting_at` remains required for the first write to
    /// succeed after a restore.
    #[tokio::test]
    async fn unseeded_wal_after_restore_fails_without_applying() {
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", keyword_schema()).unwrap();

        const RESTORED_WATERMARK: u64 = 5;
        // The bug: base-0 WAL paired with a watermark seeded from a restore.
        let wal = Arc::new(MemWal::new());
        let coord = WriteCoordinator::start_from(wal, engine.clone(), RESTORED_WATERMARK);

        let error = tokio::time::timeout(
            std::time::Duration::from_millis(500),
            coord.submit(RaftLogEntry::Index {
                collection_id: "u".into(),
                req: IndexRequest {
                    items: vec![IndexItem {
                        external_id: "post-restore-1".into(),
                        field: "email".into(),
                        value: FieldValue::String("fresh@x.com".into()),
                        version: None,
                    }],
                    request_id: None,
                },
            }),
        )
        .await
        .expect("stale sequence must fail promptly")
        .expect_err("an unseeded WAL must not report a successful write");
        assert!(
            error.downcast_ref::<SubmitStalled>().is_some(),
            "an unseeded WAL must report the stale sequence as SubmitStalled: {error}"
        );
        // Never actually applied — the read side agrees with the error.
        assert_eq!(engine.stats("u").unwrap().documents_indexed, 0);
    }

    /// #1486 R2: `submit()` is bounded by `SUBMIT_TIMEOUT`, so even a
    /// completely stalled apply (nothing ever calls `complete`) surfaces as
    /// a distinct, retryable `SubmitStalled` error rather than an infinite
    /// hang. Exercises `complete_stale` too: the dedup guard's stale-skip
    /// path releases a waiter with `SubmitStalled`, not a plain hang.
    #[tokio::test]
    async fn dedup_guard_completes_stranded_waiter_as_submit_stalled() {
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", keyword_schema()).unwrap();
        let wal = Arc::new(MemWal::new());
        let coord = WriteCoordinator::start(wal, engine.clone());

        // Register a waiter directly for a sequence at/below `applied`
        // (0 at start) — exactly what the apply loop's dedup guard would
        // see on a stale-redelivery, and route it through the same
        // `complete_stale` the guard calls.
        let permit = coord.mutation_gate.clone().read_owned().await;
        let rx = coord
            .register_waiter(0, permit)
            .expect("register waiter for seq 0");
        coord.complete_stale(0);
        let outcome = tokio::time::timeout(std::time::Duration::from_secs(2), rx)
            .await
            .expect("complete_stale must resolve the waiter promptly, not hang")
            .expect("oneshot must not be dropped without a send");
        let err = outcome.expect_err("a dedup-skipped sequence must not report a fake success");
        assert!(
            err.downcast_ref::<SubmitStalled>().is_some(),
            "expected SubmitStalled, got: {err}"
        );
    }
}
// CODEGEN-END
