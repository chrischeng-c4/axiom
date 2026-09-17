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

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};

use anyhow::{bail, Result};
use futures::StreamExt;
use raft_runtime::OutcomeWindow;
use rustc_hash::FxHashMap;
use tokio::sync::{
    oneshot, Mutex as AsyncMutex, OwnedRwLockReadGuard, OwnedRwLockWriteGuard, RwLock,
};

use crate::change_admission::PendingChangeCapacity;
use crate::change_budget::AdmissionError;
use crate::log_entry::RaftLogEntry;
use crate::storage::{
    ApplyOutcome, Engine, RecordAdmissionError, RecordApplyGuard, RecordReservation,
    RecordTransientReservation, RepriceRecord,
};
use crate::wal::{SharedWal, WalDelivery, WalRecord};

mod committed_scalar;
use committed_scalar::MappedApply;

/// How many recent outcomes to retain, via [`OutcomeWindow`]. A publisher
/// reads its outcome within microseconds of the apply loop reaching its
/// sequence, far inside this window; outcomes for sequences no local
/// handler is waiting on (writes that originated on other nodes) age out.
const OUTCOME_WINDOW: u64 = 8192;
/// Bound on `submit()`'s wait for local apply (#1486 R2). Comfortably above
/// realistic single-record apply latency, so this only fires on a genuine
/// stall — turning what would otherwise be an infinite hang into a
/// retryable 5xx.
const SUBMIT_TIMEOUT_SECS: u64 = 30;
const SUBMIT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(SUBMIT_TIMEOUT_SECS);

struct PendingApply {
    seq: u64,
    delivery: WalDelivery,
    admitted_bytes: usize,
    reservation: Option<RecordReservation>,
    transient: Option<RecordTransientReservation>,
    enqueued_at: std::time::Instant,
}

/// A local submit makes one atomic pre-publication reservation. Its apply
/// portion remains eligible for repricing and source retention; the transient
/// portion covers transport, encode, and local AOF copies until completion.
/// Keeping both values in one map entry makes lookup and capacity relief
/// atomic with respect to the sequence.
struct LocalRecordReservation {
    apply: RecordReservation,
    transient: Option<RecordTransientReservation>,
}

impl LocalRecordReservation {
    fn bytes(&self) -> usize {
        self.apply
            .bytes()
            .checked_add(
                self.transient
                    .as_ref()
                    .map(RecordTransientReservation::bytes)
                    .unwrap_or_default(),
            )
            .expect("local reservation total overflow")
    }
}

enum PreparedLocalRecord<'a> {
    Prepared(RecordApplyGuard<'a>),
    /// The committed source needs a larger exact charge than its publication
    /// reservation.  Keep both pieces at the ordered head while a dedicated
    /// blocking apply worker waits for checkpoint relief.
    CapacityBlocked {
        entry: RaftLogEntry,
        reservation: RecordReservation,
        required: usize,
    },
    /// A committed record reached no charged apply guard. The coordinator must
    /// retain its reservation and source at the unresolved head.
    Unresolved {
        reservation: RecordReservation,
        error: anyhow::Error,
    },
}

/// Repricing happens after the WAL record has a sequence, but before its
/// capture-barrier lease. A larger valid record may grow only after the
/// ordered apply worker has retained the source and reservation at its head.
/// If that growth is full, return a blocked state so the worker can wait for
/// checkpoint relief without acquiring an apply lease or allowing later
/// records to pass.
fn prepare_local_record<'a>(
    engine: &'a Engine,
    mut entry: RaftLogEntry,
    mut reservation: RecordReservation,
) -> Result<PreparedLocalRecord<'a>> {
    loop {
        match engine.begin_admitted_record(entry, reservation) {
            Ok(guard) => return Ok(PreparedLocalRecord::Prepared(guard)),
            Err(RepriceRecord {
                entry: repriced_entry,
                reservation: mut repriced_reservation,
                required: Some(required),
                error: _,
            }) => match repriced_reservation.try_grow_to(required) {
                Ok(()) => {
                    entry = repriced_entry;
                    reservation = repriced_reservation;
                }
                Err(AdmissionError::Full { .. }) => {
                    return Ok(PreparedLocalRecord::CapacityBlocked {
                        entry: repriced_entry,
                        reservation: repriced_reservation,
                        required,
                    });
                }
                Err(grow_error) => {
                    return Ok(PreparedLocalRecord::Unresolved {
                        reservation: repriced_reservation,
                        error: anyhow::Error::new(RecordAdmissionError::Capacity(grow_error)),
                    });
                }
            },
            Err(RepriceRecord {
                reservation, error, ..
            }) => {
                return Ok(PreparedLocalRecord::Unresolved {
                    reservation,
                    error: anyhow::Error::new(error),
                });
            }
        }
    }
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

/// This process observed a durability boundary that it cannot safely resolve
/// while it keeps serving mutations. Only a restart can rebuild one exact state
/// from `CURRENT` plus the AOF and clear this latch.
#[derive(Debug, Clone)]
pub struct RestartRequired(pub String);

impl std::fmt::Display for RestartRequired {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for RestartRequired {}

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
    /// An unresolved committed head can fail before its publisher installs a
    /// waiter. Keep its typed failure separate from applied outcomes.
    unresolved: FxHashMap<u64, Result<ApplyOutcome>>,
    waiters: FxHashMap<u64, oneshot::Sender<Result<ApplyOutcome>>>,
    /// A permit moves here immediately after publish and stays bound to the
    /// sequence until the apply loop completes it. Caller cancellation or a
    /// submit timeout only drops the waiter; it must never open the restore
    /// fence while the published record can still apply later.
    mutation_permits: FxHashMap<u64, OwnedRwLockReadGuard<()>>,
}

/// One process-local mutation boundary for embedded Standalone.
///
/// Shared permits cover ordinary writes and checkpoints. Durable replacement
/// takes the exclusive permit. `restart_required` is a one-way latch for this
/// process: a successful disk-space probe must never clear a commit-uncertain
/// or AOF-gap decision.
#[derive(Clone)]
pub struct MutationGate {
    lock: Arc<RwLock<()>>,
    restart_required: Arc<AtomicBool>,
}

impl Default for MutationGate {
    fn default() -> Self {
        Self {
            lock: Arc::new(RwLock::new(())),
            restart_required: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl MutationGate {
    /// Acquire a shared mutation/checkpoint permit unless the process already
    /// requires restart. The second check closes the race with a latch set while
    /// this caller waited behind an exclusive replacement.
    pub async fn shared(&self) -> Result<OwnedRwLockReadGuard<()>> {
        self.ensure_serving()?;
        let permit = self.lock.clone().read_owned().await;
        self.ensure_serving()?;
        Ok(permit)
    }

    /// Acquire the exclusive replacement permit unless a prior durability
    /// failure already requires restart.
    pub async fn exclusive(&self) -> Result<OwnedRwLockWriteGuard<()>> {
        self.ensure_serving()?;
        let permit = self.lock.clone().write_owned().await;
        self.ensure_serving()?;
        Ok(permit)
    }

    /// Permanently reject new mutations for this process.
    pub fn require_restart(&self) {
        self.restart_required.store(true, Ordering::Release);
    }

    pub fn is_restart_required(&self) -> bool {
        self.restart_required.load(Ordering::Acquire)
    }

    fn ensure_serving(&self) -> Result<()> {
        if self.is_restart_required() {
            return Err(anyhow::Error::new(RestartRequired(
                "durability state is uncertain; restart this Lumen process before retrying any mutation"
                    .to_string(),
            )));
        }
        Ok(())
    }
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
    engine: Arc<Engine>,
    /// A local submit holds this lock from WAL publication through installing
    /// its sequence-keyed reservation. The apply loop takes and removes the
    /// reservation before it enters `spawn_blocking`; therefore a subscriber
    /// that observes a record while `publish` is still returning cannot apply
    /// it through the unadmitted path.
    local_reservations: AsyncMutex<FxHashMap<u64, LocalRecordReservation>>,
    has_local_aof: bool,
    /// A refused local request can need source relief without an apply waiter.
    capacity_relief_requested: AtomicBool,
    layer_capacity_owner: Mutex<Option<crate::segment_capacity::Fallback>>,
    applied: AtomicU64,
    completions: Mutex<CompletionState>,
    /// A failed committed head can still retain its WAL source. Keep any
    /// reservation that the engine did not adopt until process exit.
    failed_head_reservations: Mutex<Vec<RecordReservation>>,
    /// A failed local head must keep its transport/AOF reservation too. This
    /// stays separate from the apply reservation because it never owns source
    /// retention or Engine state.
    failed_head_transient_reservations: Mutex<Vec<RecordTransientReservation>>,
    failed_head_retentions: Mutex<FxHashMap<u64, crate::change_budget::SourceRetention>>,
    /// A non-owning self reference lets a cancelled request leave a publisher
    /// task alive without retaining the coordinator forever.
    self_weak: Weak<WriteCoordinator>,
    /// Serializes one Standalone-wide replacement against ordinary writes.
    ///
    /// `submit` holds a shared permit from publish through local apply. A
    /// durable restore takes the exclusive permit, which therefore starts only
    /// after every earlier submit has completed and prevents every later submit
    /// from publishing until activation finishes. This fence is sufficient for
    /// the embedded single-process Standalone path. External producers that can
    /// publish directly to a shared WAL remain outside its contract.
    mutation_gate: MutationGate,
}

impl WriteCoordinator {
    /// Start or reuse the native checkpoint owner before this coordinator waits
    /// on capacity for a committed record. This holds no apply lease.
    fn ensure_capacity_owner(&self) -> Result<()> {
        let mut owner = self
            .layer_capacity_owner
            .lock()
            .map_err(|_| anyhow::anyhow!("capacity owner poisoned"))?;
        crate::segment_capacity::Fallback::ensure(&mut owner, &self.engine, None)
    }

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
        engine.capture_barrier.apply().initialize_sequence(from_seq);
        let coord = Arc::new_cyclic(|self_weak| Self {
            wal: wal.clone(),
            engine: engine.clone(),
            local_reservations: AsyncMutex::new(FxHashMap::default()),
            has_local_aof: aof.is_some(),
            capacity_relief_requested: AtomicBool::new(false),
            layer_capacity_owner: Mutex::new(None),
            applied: AtomicU64::new(from_seq),
            completions: Mutex::new(CompletionState {
                outcomes: OutcomeWindow::new(OUTCOME_WINDOW),
                unresolved: FxHashMap::default(),
                waiters: FxHashMap::default(),
                mutation_permits: FxHashMap::default(),
            }),
            failed_head_reservations: Mutex::new(Vec::new()),
            failed_head_transient_reservations: Mutex::new(Vec::new()),
            failed_head_retentions: Mutex::new(FxHashMap::default()),
            self_weak: self_weak.clone(),
            mutation_gate: MutationGate::default(),
        });
        Self::start_capacity_relief(&coord);
        if let Some(aof) = aof.clone() {
            Self::start_aof_sync(&coord, aof);
        }
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
                let mut sub = match wal.subscribe_admitted(from).await {
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
                // A retry reservation covers the next decoded working copy.
                // Its source subscription is installed before the old one drops.
                let mut replay_reservation: Option<(u64, RecordReservation)> = None;
                while let Some(item) = sub.next().await {
                    match item {
                        Ok((seq, mut delivery)) => {
                            // Idempotent under redelivery: skip anything at or
                            // below what we've already applied. Defense-in-depth
                            // (#1486): a skipped sequence never reaches `complete`,
                            // so any local waiter for it (a submit() that published
                            // this exact seq) would otherwise hang forever — release
                            // it with a distinct, retryable error instead.
                            let local_reservation = loop_coord.take_local_reservation(seq).await;
                            let (mut reservation, mut transient) = if let Some((
                                expected,
                                reservation,
                            )) = replay_reservation.take()
                            {
                                if seq != expected || local_reservation.is_some() {
                                    // Losing the pinned head is a broken WAL contract. Never
                                    // acknowledge a later sequence as if that head had applied.
                                    engine.capture_barrier.apply().mark_uncertain();
                                    loop_coord.mutation_gate.require_restart();
                                    tracing::error!(seq, expected, "WAL replay did not return its capacity-blocked head; restart required");
                                    return;
                                }
                                (Some(reservation), None)
                            } else {
                                match local_reservation {
                                    Some(local) => (Some(local.apply), local.transient),
                                    None => (None, None),
                                }
                            };
                            if seq <= loop_coord.applied.load(Ordering::Acquire) {
                                loop_coord.complete_stale(seq);
                                continue;
                            }
                            if reservation.is_none() {
                                match loop_coord
                                    .try_apply_mapped_delivery(seq, delivery, aof.clone())
                                    .await
                                {
                                    Ok(MappedApply::Fallback(retained)) => delivery = retained,
                                    Ok(MappedApply::Applied) => {
                                        loop_coord
                                            .failed_head_retentions
                                            .lock()
                                            .expect("failed-head retentions poisoned")
                                            .remove(&seq);
                                        continue;
                                    }
                                    Ok(MappedApply::Unresolved) => {
                                        // Advancing this subscription would release the source
                                        // whose AOF completion is uncertain. Retain it at head.
                                        futures::future::pending::<()>().await;
                                        unreachable!("unresolved mapped source remains pinned");
                                    }
                                    Err(error) => {
                                        engine.capture_barrier.apply().mark_uncertain();
                                        loop_coord.mutation_gate.require_restart();
                                        loop_coord.fail_unresolved(seq, Err(anyhow::Error::new(
                                            RestartRequired(format!("committed mapped WAL apply failed: {error}; source retained"))
                                        )), None);
                                        futures::future::pending::<()>().await;
                                        unreachable!("failed mapped source remains pinned");
                                    }
                                }
                                // Foreign delivery has no local publication reservation. Own
                                // its raw working copy first; the apply preparation below must
                                // acquire the rest before cloning AOF or normalized values.
                                let request = delivery.decoded_owned_bytes().ok().and_then(|raw| {
                                    delivery
                                        .extra_delivery_bytes(aof.is_some())
                                        .ok()
                                        .map(|extra| {
                                            engine.record_ram_request_from_bound(raw, extra)
                                        })
                                });
                                if let Some(request) = request {
                                    match engine.try_reserve_record_ram(&request) {
                                        Ok(admitted) => reservation = Some(admitted),
                                        Err(RecordAdmissionError::Capacity(AdmissionError::Full { .. })) => {
                                            // The current MemWal delivery is not acknowledged
                                            // until another poll. Pin the unchanged applied cut
                                            // before dropping that subscription and working copy.
                                            let replay = loop {
                                                match wal.subscribe_admitted(loop_coord.applied.load(Ordering::Acquire)).await {
                                                    Ok(replay) => break replay,
                                                    Err(error) => {
                                                        tracing::warn!(seq, %error, "could not pin committed WAL head for capacity retry");
                                                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                                                    }
                                                }
                                            };
                                            drop(delivery);
                                            sub = replay;
                                            if let Err(error) = loop_coord.ensure_capacity_owner() {
                                                engine.capture_barrier.apply().mark_uncertain();
                                                loop_coord.mutation_gate.require_restart();
                                                loop_coord.fail_unresolved(
                                                    seq,
                                                    Err(anyhow::Error::new(RestartRequired(format!(
                                                        "could not start committed WAL capacity maintenance: {error}; source retained"
                                                    )))),
                                                    None,
                                                );
                                                futures::future::pending::<()>().await;
                                                unreachable!("failed committed head remains pinned");
                                            }
                                            let eng = engine.clone();
                                            match tokio::task::spawn_blocking(move || eng.wait_reserve_record_ram(&request)).await {
                                                Ok(Ok(admitted)) => replay_reservation = Some((seq, admitted)),
                                                result => {
                                                    engine.capture_barrier.apply().mark_uncertain();
                                                    loop_coord.mutation_gate.require_restart();
                                                    tracing::error!(seq, error = ?result.map(|admission| admission.map(|_| ())), "committed WAL admission wait failed; restart required");
                                                    return;
                                                }
                                            }
                                            continue;
                                        }
                                        // Oversized raw records still need the streaming source
                                        // adapter. This branch retains the established dispatch
                                        // until that distinct preparation route is installed.
                                        Err(error) => tracing::warn!(seq, ?error, "committed WAL raw record requires streaming preparation"),
                                    }
                                }
                            }
                            let Some(admitted_bytes) = reservation.as_ref().map(|apply| {
                                apply
                                    .bytes()
                                    .checked_add(
                                        transient
                                            .as_ref()
                                            .map(RecordTransientReservation::bytes)
                                            .unwrap_or_default(),
                                    )
                                    .expect("local admission total overflow")
                            }) else {
                                // A source larger than the process budget needs a streaming
                                // apply representation. Never decode it with invented credit.
                                engine.capture_barrier.apply().mark_uncertain();
                                loop_coord.mutation_gate.require_restart();
                                loop_coord.fail_unresolved(
                                    seq,
                                    Err(anyhow::Error::new(RestartRequired(
                                        "committed WAL record has no bounded delivery admission; source retained".into(),
                                    ))),
                                    reservation,
                                );
                                // Keep the subscription at the unresolved source. A next poll
                                // would release its bytes and permit a successor to apply.
                                futures::future::pending::<()>().await;
                                unreachable!(
                                    "the unresolved failed head must retain its subscription"
                                )
                            };
                            let mut pending = PendingApply {
                                seq,
                                delivery,
                                admitted_bytes,
                                reservation,
                                transient,
                                enqueued_at: std::time::Instant::now(),
                            };
                            if let Some(retention) = pending
                                .reservation
                                .as_mut()
                                .map(RecordReservation::source_retention)
                            {
                                let retained = pending.delivery.retain_source(retention.clone());
                                loop_coord
                                    .failed_head_retentions
                                    .lock()
                                    .expect("failed-head retentions poisoned")
                                    .insert(seq, retention);
                                if let Err(error) = retained {
                                    engine.capture_barrier.apply().mark_uncertain();
                                    loop_coord.mutation_gate.require_restart();
                                    loop_coord.fail_unresolved(
                                        seq,
                                        Err(anyhow::Error::new(RestartRequired(format!(
                                            "cannot retain committed WAL source {seq}: {error}"
                                        )))),
                                        pending.reservation.take(),
                                    );
                                    loop_coord.retain_failed_transient(pending.transient.take());
                                    futures::future::pending::<()>().await;
                                }
                            }
                            // A local record is admitted before publish and then applied one at a
                            // time. Do not prefetch another decoded record while this record owns
                            // capacity through its AOF and watermark boundary.
                            let eng = engine.clone();
                            let applying_coord = loop_coord.clone();
                            let local_aof = aof.clone();
                            let applied = tokio::task::spawn_blocking(move || {
                                // Keep this outside the unwind boundary. A panic after a
                                // locally admitted WAL record must retain both halves at the
                                // unresolved head, even though the transient half has no
                                // source-retention bridge of its own.
                                let mut transient = pending.transient.take();
                                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                let aof = local_aof;
                                let seq = pending.seq;
                                // The transient half is already admitted. It must remain alive
                                // across source decode, reprice waits, AOF clone/encode, append,
                                // and flush. It is never passed to Engine preparation.
                                let rec = match pending.delivery.read(pending.admitted_bytes) {
                                    Ok(rec) => rec,
                                    Err(error) => {
                                        eng.capture_barrier.apply().mark_uncertain();
                                        applying_coord.mutation_gate.require_restart();
                                        applying_coord.fail_unresolved(
                                            seq,
                                            Err(anyhow::Error::new(RestartRequired(format!(
                                                "committed WAL source read failed: {error}; restart required"
                                            )))),
                                            pending.reservation,
                                        );
                                        applying_coord.retain_failed_transient(transient.take());
                                        return false;
                                    }
                                };
                                let apply_kind = crate::metrics::ApplyKind::from_entry(&rec.entry);
                                eng.metrics().observe_coordinator_stage(
                                    apply_kind,
                                    crate::metrics::CoordinatorStage::PublishToApplyStart,
                                    pending.enqueued_at.elapsed(),
                                );
                                let version = rec.version;
                                // Persist the recoverable WAL tail before preparing the
                                // record.  Preparation acquires the CaptureBarrier apply
                                // lease; waiting for the AOF mutex while holding that lease
                                // can deadlock a checkpoint that is trying to capture the
                                // engine while another writer holds the AOF mutex.
                                let aof_rec = aof.as_ref().map(|_| WalRecord {
                                    version,
                                    entry: rec.entry.clone(),
                                });
                                if !applying_coord.mutation_gate.is_restart_required() {
                                    if let (Some(aof), Some(rec)) = (aof.as_ref(), aof_rec.as_ref()) {
                                        let persisted = {
                                            let mut writer =
                                                aof.lock().expect("aof writer poisoned");
                                            writer
                                                .append(seq, rec)
                                                .and_then(|()| writer.flush())
                                        };
                                        if let Err(e) = persisted {
                                            applying_coord.mutation_gate.require_restart();
                                            if is_storage_full(&e) {
                                                tracing::error!(
                                                    seq,
                                                    error = %e,
                                                    "AOF persist failed: ENOSPC — entering degraded read-only mode"
                                                );
                                                eng.metrics().mark_storage_degraded();
                                                applying_coord.fail_unresolved(
                                                    seq,
                                                    Err(anyhow::Error::new(StorageFullError(
                                                        format!(
                                                            "local storage is full (ENOSPC) persisting sequence {seq}; node entered degraded read-only mode"
                                                        ),
                                                    ))),
                                                    pending.reservation.take(),
                                                );
                                            } else {
                                                tracing::error!(
                                                    seq,
                                                    error = %e,
                                                    "AOF persist failed; restart required"
                                                );
                                                applying_coord.fail_unresolved(
                                                    seq,
                                                    Err(anyhow::Error::new(RestartRequired(
                                                        format!(
                                                            "could not persist local AOF sequence {seq}: {e}; restart this Lumen process"
                                                        ),
                                                    ))),
                                                    pending.reservation.take(),
                                                );
                                            }
                                            applying_coord.retain_failed_transient(transient.take());
                                            return false;
                                        }
                                    }
                                }
                                // A prepared guard owns the apply lease and its retained charge
                                // until after AOF persistence and watermark advancement below.
                                let mut prepared = None;
                                // #4326: per-record apply cost, measured across the whole
                                // prepare+apply span below (including any reprice/
                                // wait_grow_to retry inside `prepare_local_record`) so a
                                // 500k-doc perf probe can read per-record apply cost from
                                // `GET /metrics` — see `Metrics::observe_coordinator_apply`.
                                let apply_started_at = std::time::Instant::now();
                                let mut outcome = match pending.reservation {
                                    Some(reservation) => {
                                        // A post-WAL exact reprice may need capacity that was
                                        // unavailable at publication time.  Keep this ordered
                                        // head in the existing dedicated blocking worker.  No
                                        // apply lease is held while the reservation waits, and
                                        // the subscription cannot advance to a later record.
                                        let mut entry = rec.entry;
                                        let mut reservation = reservation;
                                        let prepared_local = loop {
                                            match prepare_local_record(
                                                eng.as_ref(),
                                                entry,
                                                reservation,
                                            ) {
                                                Ok(PreparedLocalRecord::CapacityBlocked {
                                                    entry: blocked_entry,
                                                    reservation: mut blocked_reservation,
                                                    required,
                                                }) => {
                                                    if let Err(error) =
                                                        applying_coord.ensure_capacity_owner()
                                                    {
                                                        break Ok(PreparedLocalRecord::Unresolved {
                                                            reservation: blocked_reservation,
                                                            error,
                                                        });
                                                    }
                                                    eng.request_pending_checkpoint();
                                                    match blocked_reservation.wait_grow_to(required) {
                                                        Ok(()) => {
                                                            entry = blocked_entry;
                                                            reservation = blocked_reservation;
                                                        }
                                                        Err(error) => {
                                                            break Ok(PreparedLocalRecord::Unresolved {
                                                                reservation: blocked_reservation,
                                                                error: anyhow::Error::new(
                                                                    RecordAdmissionError::Capacity(
                                                                        error,
                                                                    ),
                                                                ),
                                                            });
                                                        }
                                                    }
                                                }
                                                other => break other,
                                            }
                                        };
                                        match prepared_local {
                                        Ok(PreparedLocalRecord::Prepared(mut guard)) => {
                                            // This copy is made only after the full normalized
                                            // and transport reservation became a retained charge.
                                            let prepared_entry =
                                                guard.entry().expect("unapplied prepared entry");
                                            let apply_kind =
                                                crate::metrics::ApplyKind::from_entry(prepared_entry);
                                            let apply_items =
                                                crate::metrics::apply_item_count(prepared_entry);
                                            let outcome = eng.apply_prepared_raft_entry(&mut guard);
                                            eng.metrics().observe_coordinator_apply(
                                                apply_kind,
                                                apply_items,
                                                apply_started_at.elapsed(),
                                            );
                                            prepared = Some(guard);
                                            outcome
                                        }
                                        Ok(PreparedLocalRecord::Unresolved {
                                            reservation,
                                            error,
                                        }) => {
                                            // No apply guard proves the full normalized record is
                                            // charged. Retain this committed source and stop at its
                                            // prefix instead of using generic uncharged apply.
                                            eng.capture_barrier.apply().mark_uncertain();
                                            applying_coord.mutation_gate.require_restart();
                                            applying_coord.fail_unresolved(
                                                seq,
                                                Err(anyhow::Error::new(RestartRequired(format!(
                                                "committed WAL record has no charged apply guard: {error}; source retained"
                                                )))),
                                                Some(reservation),
                                            );
                                            applying_coord
                                                .retain_failed_transient(transient.take());
                                            return false;
                                        }
                                        Ok(PreparedLocalRecord::CapacityBlocked { .. }) => {
                                            unreachable!("capacity-blocked local record must wait or resolve")
                                        }
                                        Err(error) => {
                                            eng.capture_barrier.apply().mark_uncertain();
                                            applying_coord.mutation_gate.require_restart();
                                            applying_coord.fail_unresolved(
                                                seq,
                                                Err(anyhow::Error::new(RestartRequired(format!(
                                                "committed WAL preparation returned unexpectedly: {error}; source retained"
                                                )))),
                                                None,
                                            );
                                            applying_coord
                                                .retain_failed_transient(transient.take());
                                            return false;
                                        }
                                        }
                                    }
                                    None => {
                                        // Admission reaches this branch only when no bounded
                                        // committed representation exists. Do not let it escape
                                        // through an uncharged generic apply.
                                        eng.capture_barrier.apply().mark_uncertain();
                                        applying_coord.mutation_gate.require_restart();
                                        applying_coord.fail_unresolved(
                                            seq,
                                            Err(anyhow::Error::new(RestartRequired(
                                                "committed WAL record has no charged apply guard; source retained"
                                                    .into(),
                                            ))),
                                            None,
                                        );
                                        applying_coord.retain_failed_transient(transient.take());
                                        return false;
                                    }
                                };
                                let apply = prepared.as_ref().map(|guard| guard.apply_lease());
                                // A preceding AOF gap makes every later record
                                // unrecoverable, including one whose Engine
                                // application reports a normal validation
                                // error.  Do not append or acknowledge it.
                                if applying_coord.mutation_gate.is_restart_required() {
                                    outcome = Err(anyhow::Error::new(RestartRequired(format!(
                                        "sequence {seq} applied after an earlier local AOF gap; restart this Lumen process"
                                    ))));
                                }
                                if let Err(e) = &outcome {
                                    tracing::warn!(seq, error = %e, "apply error (entry no-ops)");
                                }
                                apply
                                    .expect("completed committed record has a charged apply guard")
                                    .advance_sequence(seq);
                                eng.metrics().observe_coordinator_stage(
                                    apply_kind,
                                    crate::metrics::CoordinatorStage::ApplyToWaiter,
                                    apply_started_at.elapsed(),
                                );
                                // Release both halves before publishing the applied watermark.
                                // A cancelled local submit has no waiter to observe completion,
                                // so `applied_seq` is its only completion signal. Publishing it
                                // first would let callers observe an applied record while its
                                // reservation is still charged on the budget.
                                drop(apply);
                                drop(prepared.take());
                                drop(transient.take());
                                applying_coord.complete(seq, outcome);
                                true
                                }));
                                (result, transient)
                            }).await;
                            match applied {
                                Ok((Ok(true), transient)) => {
                                    debug_assert!(
                                        transient.is_none(),
                                        "completed local apply must release its transient reservation"
                                    );
                                    loop_coord
                                        .failed_head_retentions
                                        .lock()
                                        .expect("failed-head retentions poisoned")
                                        .remove(&seq);
                                }
                                Ok((Ok(false), transient)) => {
                                    loop_coord.retain_failed_transient(transient);
                                    // Keep this subscription at its failed head. Polling again
                                    // would acknowledge source bytes and permit a successor.
                                    futures::future::pending::<()>().await;
                                }
                                Ok((Err(_), transient)) => {
                                    // The lease also latches uncertainty while unwinding, before
                                    // another checkpoint can enter the failed apply interval.
                                    engine.capture_barrier.apply().mark_uncertain();
                                    loop_coord.mutation_gate.require_restart();
                                    loop_coord.retain_failed_transient(transient);
                                    loop_coord.fail_unresolved(
                                        seq,
                                        Err(anyhow::Error::new(RestartRequired(format!(
                                            "apply task panicked; restart required"
                                        )))),
                                        None,
                                    );
                                    // Retain the subscription floor after a panic too. The source
                                    // might still own the committed head's bytes.
                                    futures::future::pending::<()>().await;
                                }
                                Err(error) => {
                                    // A JoinError means the worker did not return its transient
                                    // half (for example, runtime shutdown). The retained source
                                    // still pins its apply half; require restart and preserve the
                                    // subscription floor rather than claiming completion.
                                    engine.capture_barrier.apply().mark_uncertain();
                                    loop_coord.mutation_gate.require_restart();
                                    loop_coord.fail_unresolved(
                                        seq,
                                        Err(anyhow::Error::new(RestartRequired(format!(
                                            "apply task stopped: {error}; restart required"
                                        )))),
                                        None,
                                    );
                                    futures::future::pending::<()>().await;
                                }
                            }
                        }
                        Err(e) => tracing::warn!(error = %e, "apply loop: stream item error"),
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

    /// Capacity can be held by later publications or by already applied
    /// sources pinned by a slow reader. This task needs neither an apply lease
    /// nor MutationGate. Source charges remain until native replacement proves
    /// that the original payload has left RAM.
    ///
    /// `capacity_relief_requested` (a pre-publication door refusal, e.g. an
    /// HTTP 429) always drives the applied-source staging scan below — it
    /// releases sources of records already folded into the engine and needs
    /// no waiter. It alone must never steal a *pending* local reservation
    /// from `local_reservations`: that ledger holds already-admitted records
    /// simply waiting their turn to apply, not spare capacity. Stealing one
    /// forces its apply to re-reserve from scratch and can park it in
    /// `wait_reserve_record_ram` until checkpoint frees bytes. The ledger
    /// steal below runs only when `has_capacity_waiters()` reports a real
    /// committed apply or replay blocked on capacity.
    fn start_capacity_relief(coord: &Arc<Self>) {
        let weak = Arc::downgrade(coord);
        let mut staged_through = coord.applied_seq();
        tokio::spawn(async move {
            let mut requested_through = staged_through;
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(25)).await;
                let Some(coord) = weak.upgrade() else {
                    return;
                };
                let requested = coord
                    .capacity_relief_requested
                    .swap(false, Ordering::AcqRel);
                let waiting = coord.engine.has_capacity_waiters();
                if requested || waiting {
                    requested_through = requested_through.max(coord.applied_seq());
                }
                // Scan each applied sequence once, in bounded groups. Even a
                // refused request with no waiting apply must finish its group.
                // Checkpoint may have retired the journal while native readers
                // still retain these sources and their independent charges.
                for _ in 0..64 {
                    if staged_through >= requested_through {
                        break;
                    }
                    let seq = staged_through + 1;
                    match coord.wal.stage_source(seq).await {
                        Ok(None) => staged_through = seq,
                        Ok(Some(proof)) if proof.sequence() == seq => staged_through = seq,
                        Ok(Some(_)) => {
                            tracing::error!(
                                seq,
                                "WAL source returned an offload proof for another sequence"
                            );
                            break;
                        }
                        Err(error) => {
                            tracing::warn!(seq, %error, "applied WAL source staging failed; retaining source charge");
                            break;
                        }
                    }
                }
                if !waiting {
                    // A door refusal (`requested`) with no committed apply
                    // blocked on capacity has already been served above: the
                    // applied-source staging scan advanced `staged_through`.
                    // Stealing a pending ledger reservation here would only
                    // rob an already-admitted, not-yet-applied local record
                    // that is simply waiting its turn — forcing its apply to
                    // re-reserve from scratch and park in
                    // `wait_reserve_record_ram` for no genuine waiter.
                    continue;
                }
                let mut ledger = coord.local_reservations.lock().await;
                // Select without copying the pending ledger into a second buffer.
                // One attempt per tick lets the waiting head take released capacity.
                if let Some(seq) = ledger
                    .iter()
                    .max_by_key(|(&seq, reservation)| (reservation.bytes(), std::cmp::Reverse(seq)))
                    .map(|(&seq, _)| seq)
                {
                    match coord.wal.stage_source(seq).await {
                        Ok(Some(proof)) if proof.sequence() == seq => {
                            // No apply worker can take this reservation while the ledger
                            // is locked. Its decoded delivery and normalized data do not
                            // exist yet. The source payload is now gone as well.
                            drop(ledger.remove(&seq));
                        }
                        Ok(None) => {}
                        Ok(Some(_)) => {
                            tracing::error!(
                                seq,
                                "WAL source returned an offload proof for another sequence"
                            );
                        }
                        Err(error) => {
                            tracing::warn!(seq, %error, "WAL source staging failed; retaining pending reservation");
                        }
                    }
                }
            }
        });
    }

    fn start_aof_sync(coord: &Arc<Self>, aof: SharedAof) {
        let weak = Arc::downgrade(coord);
        let engine = coord.engine.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                if weak.upgrade().is_none() {
                    return;
                }
                // Never wait for the synchronous AOF mutex on a Tokio worker.
                // A test or a foreground append may intentionally hold it while
                // unrelated collections must continue serving queries.
                let plan = match tokio::task::spawn_blocking({
                    let aof = aof.clone();
                    move || {
                        let mut writer = match aof.try_lock() {
                            Ok(writer) => writer,
                            Err(std::sync::TryLockError::WouldBlock) => return Ok(None),
                            Err(std::sync::TryLockError::Poisoned(_)) => {
                                return Err(anyhow::anyhow!("AOF writer poisoned"));
                            }
                        };
                        writer.begin_sync()
                    }
                })
                .await
                {
                    Ok(Ok(plan)) => plan,
                    Ok(Err(error)) => {
                        tracing::error!(%error, "AOF sync preparation failed");
                        engine.capture_barrier.apply().mark_uncertain();
                        if let Some(coord) = weak.upgrade() {
                            coord.mutation_gate.require_restart();
                        }
                        return;
                    }
                    Err(error) => {
                        tracing::error!(%error, "AOF sync preparation task failed");
                        engine.capture_barrier.apply().mark_uncertain();
                        if let Some(coord) = weak.upgrade() {
                            coord.mutation_gate.require_restart();
                        }
                        return;
                    }
                };
                let Some(plan) = plan else { continue };
                let sync_result = tokio::task::spawn_blocking(move || {
                    let mut plan = plan;
                    let result = plan.sync_off_lock();
                    (plan, result)
                })
                .await;
                let result = match sync_result {
                    Ok((plan, result)) => match tokio::task::spawn_blocking({
                        let aof = aof.clone();
                        move || {
                            let mut writer = aof
                                .lock()
                                .map_err(|_| anyhow::anyhow!("AOF writer poisoned"))?;
                            result.and_then(|()| writer.complete_sync(plan))
                        }
                    })
                    .await
                    {
                        Ok(result) => result,
                        Err(error) => {
                            Err(anyhow::anyhow!("AOF sync completion task failed: {error}"))
                        }
                    },
                    Err(error) => Err(anyhow::anyhow!("AOF sync task failed: {error}")),
                };
                if let Err(error) = result {
                    tracing::error!(%error, "AOF sync failed; restart required");
                    engine.capture_barrier.apply().mark_uncertain();
                    if let Some(coord) = weak.upgrade() {
                        coord.mutation_gate.require_restart();
                    }
                    return;
                }
            }
        });
    }

    async fn take_local_reservation(&self, seq: u64) -> Option<LocalRecordReservation> {
        self.local_reservations.lock().await.remove(&seq)
    }

    /// `try_reserve_record` prices the submitted raw record. Without an AOF,
    /// two extras cover the retained WAL record and an encode `Vec` with up to
    /// two raw bounds of capacity: three total raw bounds. With an AOF, four
    /// extras cover the retained WAL record, delivered record, AOF clone, and
    /// encode `Vec` up to two raw bounds: five total raw bounds.
    /// An unrepresentable bound is refused before the WAL assigns a sequence.
    ///
    /// `Ok(None)` means "publish with no reservation at all" — the WAL
    /// delivery loop's post-publication path then has nothing to grow from
    /// and waits on capacity with no bound. `try_reserve_record` never
    /// returns that gap for a domain pricing failure (unresolved
    /// schema/context maps to at least a minimal raw+transport reservation,
    /// or a capacity refusal here); this `None` arm stays only for an
    /// admission error `PendingChangeCapacity` does not classify at all
    /// (e.g. `RecordAdmissionError::WrongEngine`), which cannot occur for a
    /// same-Engine local submit today.
    fn try_admit_local_record(
        &self,
        entry: &RaftLogEntry,
    ) -> Result<Option<LocalRecordReservation>> {
        let raw = match Engine::record_owned_bytes(entry) {
            Ok(raw) => raw,
            Err(RecordAdmissionError::Overflow) => {
                return Err(self.prepublication_backpressure(PendingChangeCapacity::Overflow));
            }
            Err(error) => return Err(error.into()),
        };
        let copies = if self.has_local_aof { 4 } else { 2 };
        let Some(extra_owned) = raw.checked_mul(copies) else {
            return Err(self.prepublication_backpressure(PendingChangeCapacity::Overflow));
        };
        match self.engine.try_reserve_record(entry, extra_owned) {
            Ok(reservation) => {
                let (apply, transient) = reservation.split_transport();
                Ok(Some(LocalRecordReservation { apply, transient }))
            }
            // Unknown schema/context preparation is likewise not a reason to
            // discard a valid record; apply returns its original domain outcome.
            Err(error) => {
                if matches!(
                    error,
                    RecordAdmissionError::Capacity(AdmissionError::Full { .. })
                ) {
                    // A local pre-publication refusal does not create a
                    // committed capacity waiter. Start the same independent
                    // checkpoint owner used by committed/replayed records,
                    // otherwise request_pending_checkpoint() only signals a
                    // driver that may not exist and repeated 429s can make
                    // no progress through active or frozen changes.
                    if let Err(owner_error) = self.ensure_capacity_owner() {
                        tracing::warn!(
                            error = %owner_error,
                            "could not start local capacity maintenance"
                        );
                    }
                    self.engine.request_pending_checkpoint();
                    self.capacity_relief_requested
                        .store(true, Ordering::Release);
                }
                match PendingChangeCapacity::from_record_prepublication(&error) {
                    Some(pending) => Err(self.prepublication_backpressure(pending)),
                    None => Ok(None),
                }
            }
        }
    }

    /// Count only a final local refusal before this process publishes a WAL
    /// record. Committed replay and ordinary domain errors do not pass here.
    fn prepublication_backpressure(&self, pending: PendingChangeCapacity) -> anyhow::Error {
        self.engine.metrics().incr_segment_backpressure();
        anyhow::Error::new(pending)
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

    /// Notify the caller that a committed head is unresolved without claiming
    /// it was applied. The sequence permit stays closed and the watermark
    /// stays at the last known-good prefix.
    fn fail_unresolved(
        &self,
        seq: u64,
        outcome: Result<ApplyOutcome>,
        reservation: Option<RecordReservation>,
    ) {
        if let Some(reservation) = reservation {
            self.failed_head_reservations
                .lock()
                .expect("failed-head reservations poisoned")
                .push(reservation);
        }
        let mut completions = self.completions.lock().expect("completions poisoned");
        if let Some(waiter) = completions.waiters.remove(&seq) {
            let _ = waiter.send(outcome);
        } else {
            completions.unresolved.insert(seq, outcome);
        }
    }

    /// Keep the local transient half until the uncertain process exits. Its
    /// source is not retained here; the matching apply half owns that bridge.
    fn retain_failed_transient(&self, transient: Option<RecordTransientReservation>) {
        if let Some(transient) = transient {
            self.failed_head_transient_reservations
                .lock()
                .expect("failed-head transient reservations poisoned")
                .push(transient);
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
        if let Some(result) = m.unresolved.remove(&seq) {
            if m.mutation_permits.contains_key(&seq) {
                bail!("duplicate mutation permit for sequence {seq}");
            }
            let (tx, rx) = oneshot::channel();
            // The failed head still owns the shared permit. Install it before
            // exposing the typed error to a caller that may immediately start
            // a restore attempt.
            m.mutation_permits.insert(seq, mutation_permit);
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
        // Full local admission is a retryable refusal before this record can
        // consume a WAL sequence. Oversized and context-dependent records
        // preserve the old path until root wires durable preparation.
        let kind = crate::metrics::ApplyKind::from_entry(&entry);
        let admission_started_at = std::time::Instant::now();
        let reservation = self.try_admit_local_record(&entry)?;
        // Keep the shared permit through publish AND local apply. An exclusive
        // restore fence can therefore observe one exact applied/WAL boundary:
        // no earlier submit remains in flight and no later submit has obtained
        // a sequence yet.
        let mutation_permit = self.mutation_gate.shared().await?;
        self.engine.metrics().observe_coordinator_stage(
            kind,
            crate::metrics::CoordinatorStage::AdmissionToMutationGate,
            admission_started_at.elapsed(),
        );
        let (published_tx, published_rx) = oneshot::channel();
        let publisher = self
            .self_weak
            .upgrade()
            .ok_or_else(|| anyhow::anyhow!("write coordinator stopped before publish"))?;
        let wal = self.wal.clone();
        tokio::spawn(async move {
            // This task owns the shared permit from before WAL publication. A
            // caller may cancel after the WAL accepts its record, but it cannot
            // release the restore fence before sequence ownership is installed.
            let result = if let Some(reservation) = reservation {
                // The subscriber cannot take this ledger while publication is
                // still returning, so it sees the reservation for this exact
                // sequence before it can start local apply.
                let mut ledger = publisher.local_reservations.lock().await;
                match wal.publish(WalRecord::new(entry)).await {
                    Ok(seq) => {
                        if ledger.insert(seq, reservation).is_some() {
                            Err(anyhow::anyhow!(
                                "duplicate local reservation for sequence {seq}"
                            ))
                        } else {
                            drop(ledger);
                            publisher
                                .register_waiter(seq, mutation_permit)
                                .map(|receiver| (seq, receiver))
                        }
                    }
                    Err(error) => Err(error),
                }
            } else {
                match wal.publish(WalRecord::new(entry)).await {
                    Ok(seq) => publisher
                        .register_waiter(seq, mutation_permit)
                        .map(|receiver| (seq, receiver)),
                    Err(error) => Err(error),
                }
            };
            let _ = published_tx.send(result);
        });
        let (seq, rx) = match published_rx.await {
            Ok(Ok(pair)) => pair,
            Ok(Err(error)) => return Err(error),
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "publish task stopped before registering a waiter"
                ))
            }
        };
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
    pub async fn fence_mutations(&self) -> Result<OwnedRwLockWriteGuard<()>> {
        self.mutation_gate.exclusive().await
    }

    /// Keep an ordinary checkpoint from crossing an exclusive restore.
    pub async fn checkpoint_permit(&self) -> Result<OwnedRwLockReadGuard<()>> {
        self.mutation_gate.shared().await
    }

    /// Clone the process-local gate for components that must join the same
    /// checkpoint/restore boundary.
    pub fn mutation_gate(&self) -> MutationGate {
        self.mutation_gate.clone()
    }

    pub fn require_restart(&self) {
        self.mutation_gate.require_restart();
    }

    pub fn is_restart_required(&self) -> bool {
        self.mutation_gate.is_restart_required()
    }
}

/// The write seam the API binds to: submit a log entry, get its applied outcome,
/// and report the applied head. Implemented by [`WriteCoordinator`] (the WAL-seam
/// path for embedded WAL) and by `RaftWriteSink` (the raft-runtime path).
#[async_trait::async_trait]
pub trait WriteSink: Send + Sync {
    async fn submit(&self, entry: RaftLogEntry) -> Result<ApplyOutcome>;
    fn applied_seq(&self) -> u64;
    fn restart_required(&self) -> bool {
        false
    }
    fn mutation_gate(&self) -> Option<MutationGate> {
        None
    }
}

#[async_trait::async_trait]
impl WriteSink for WriteCoordinator {
    async fn submit(&self, entry: RaftLogEntry) -> Result<ApplyOutcome> {
        WriteCoordinator::submit(self, entry).await
    }
    fn applied_seq(&self) -> u64 {
        WriteCoordinator::applied_seq(self)
    }
    fn restart_required(&self) -> bool {
        WriteCoordinator::is_restart_required(self)
    }
    fn mutation_gate(&self) -> Option<MutationGate> {
        Some(WriteCoordinator::mutation_gate(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change_admission::PendingChangeCapacity;
    use crate::change_budget::ChangeBudget;
    use crate::types::{
        CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
    };
    use crate::wal::{MemWal, WalLog, WalStream};
    use std::collections::BTreeMap as Map;
    use std::sync::atomic::AtomicU8;
    use tokio::sync::Notify;

    #[derive(Clone)]
    struct ControlledWal {
        record: Arc<Mutex<Option<WalRecord>>>,
        latest: Arc<AtomicU64>,
        published: Arc<Notify>,
        observed_publish: Arc<Notify>,
        delivered: Arc<Notify>,
        release_publish: Arc<Notify>,
        release_delivery: Arc<Notify>,
        mode: Arc<AtomicU8>,
    }

    impl ControlledWal {
        const PAUSE_PUBLISH: u8 = 1;
        const PAUSE_DELIVERY: u8 = 2;

        fn paused(mode: u8) -> Arc<Self> {
            Arc::new(Self {
                record: Arc::new(Mutex::new(None)),
                latest: Arc::new(AtomicU64::new(0)),
                published: Arc::new(Notify::new()),
                observed_publish: Arc::new(Notify::new()),
                delivered: Arc::new(Notify::new()),
                release_publish: Arc::new(Notify::new()),
                release_delivery: Arc::new(Notify::new()),
                mode: Arc::new(AtomicU8::new(mode)),
            })
        }
    }

    #[async_trait::async_trait]
    impl WalLog for ControlledWal {
        async fn publish(&self, record: WalRecord) -> Result<u64> {
            *self.record.lock().unwrap() = Some(record);
            self.latest.store(1, Ordering::Release);
            self.published.notify_one();
            self.observed_publish.notify_one();
            if self.mode.load(Ordering::Acquire) & Self::PAUSE_PUBLISH != 0 {
                self.release_publish.notified().await;
            }
            Ok(1)
        }

        async fn subscribe(&self, _from_seq: u64) -> Result<WalStream> {
            let wal = self.clone();
            Ok(Box::pin(futures::stream::unfold(false, move |delivered| {
                let wal = wal.clone();
                async move {
                    if delivered {
                        return futures::future::pending().await;
                    }
                    wal.published.notified().await;
                    if wal.mode.load(Ordering::Acquire) & Self::PAUSE_DELIVERY != 0 {
                        wal.release_delivery.notified().await;
                    }
                    let record = wal.record.lock().unwrap().clone()?;
                    wal.delivered.notify_one();
                    Some((Ok((1, record)), true))
                }
            })))
        }

        async fn latest_seq(&self) -> Result<u64> {
            Ok(self.latest.load(Ordering::Acquire))
        }
    }

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

    fn admitted_index_entry() -> RaftLogEntry {
        RaftLogEntry::Index {
            collection_id: "u".into(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: "u1".into(),
                    field: "email".into(),
                    value: FieldValue::String("v".into()),
                    version: None,
                }],
                request_id: None,
            },
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn future_local_reservation_cannot_starve_earlier_external_record() {
        struct DeferredWal {
            inner: MemWal,
            released: Arc<std::sync::atomic::AtomicBool>,
            changed: Arc<tokio::sync::Notify>,
        }
        #[async_trait::async_trait]
        impl WalLog for DeferredWal {
            async fn publish(&self, record: WalRecord) -> Result<u64> {
                self.inner.publish(record).await
            }
            async fn latest_seq(&self) -> Result<u64> {
                self.inner.latest_seq().await
            }
            async fn stage_source(&self, seq: u64) -> Result<Option<crate::wal::WalSourceRelease>> {
                self.inner.stage_source(seq).await
            }
            async fn subscribe_admitted(
                &self,
                from: u64,
            ) -> Result<crate::wal::WalAdmissionStream> {
                let stream = self.inner.subscribe_admitted(from).await?;
                let state = (stream, self.released.clone(), self.changed.clone());
                Ok(Box::pin(futures::stream::unfold(
                    state,
                    |(mut stream, released, changed)| async move {
                        loop {
                            let wake = changed.notified();
                            if released.load(Ordering::Acquire) {
                                break;
                            }
                            wake.await;
                        }
                        stream
                            .next()
                            .await
                            .map(|record| (record, (stream, released, changed)))
                    },
                )))
            }
            async fn subscribe(&self, from: u64) -> Result<WalStream> {
                let stream = self.inner.subscribe(from).await?;
                let state = (stream, self.released.clone(), self.changed.clone());
                Ok(Box::pin(futures::stream::unfold(
                    state,
                    |(mut stream, released, changed)| async move {
                        loop {
                            let wake = changed.notified();
                            if released.load(Ordering::Acquire) {
                                break;
                            }
                            wake.await;
                        }
                        stream
                            .next()
                            .await
                            .map(|record| (record, (stream, released, changed)))
                    },
                )))
            }
        }
        let entry = |id: &str| RaftLogEntry::Index {
            collection_id: "u".into(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: id.into(),
                    field: "email".into(),
                    value: FieldValue::String(format!("{id}-{}", "x".repeat(8192))),
                    version: None,
                }],
                request_id: None,
            },
        };
        let external = entry("external");
        let local = entry("local");
        let calibration = Engine::with_change_budget(ChangeBudget::with_hard_limit(1024 * 1024));
        calibration
            .create_collection("u", keyword_schema())
            .unwrap();
        let local_raw = Engine::record_owned_bytes(&local).unwrap();
        let local_price = calibration
            .try_reserve_record(&local, local_raw * 2)
            .unwrap()
            .bytes();
        let external_raw = Engine::record_owned_bytes(&external).unwrap() * 3;
        let external_price = calibration
            .try_reserve_record(&external, external_raw / 3 * 2)
            .unwrap()
            .bytes();
        let hard = local_price + external_raw - 1;
        assert!(external_price < hard && local_price < hard);
        let budget = ChangeBudget::with_hard_limit(hard);
        let engine = Arc::new(Engine::with_change_budget(budget.clone()));
        engine.create_collection("u", keyword_schema()).unwrap();
        let wal = Arc::new(DeferredWal {
            inner: MemWal::new(),
            released: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            changed: Arc::new(tokio::sync::Notify::new()),
        });
        assert_eq!(wal.publish(WalRecord::new(external)).await.unwrap(), 1);
        let coord = WriteCoordinator::start(wal.clone(), engine.clone());
        let mut submitted = tokio::spawn({
            let coord = coord.clone();
            async move { coord.submit(local).await }
        });
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                if wal.latest_seq().await.unwrap() >= 2 || submitted.is_finished() {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("the local publication attempt must finish before delivery starts");
        let committed_local = wal.latest_seq().await.unwrap() == 2;
        assert_eq!(coord.applied_seq(), 0);

        let directory = tempfile::tempdir().unwrap();
        let store = Arc::new(crate::segment_rdb::SegmentRdbStore::new(directory.path()).unwrap());
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let checkpoint = tokio::spawn({
            let stop = stop.clone();
            let engine = engine.clone();
            let coord = coord.clone();
            let store = store.clone();
            async move {
                while !stop.load(Ordering::Acquire) {
                    let engine = engine.clone();
                    let store = store.clone();
                    let seq = coord.applied_seq();
                    tokio::task::spawn_blocking(move || store.save(&engine, seq))
                        .await
                        .unwrap()
                        .unwrap();
                    tokio::time::sleep(std::time::Duration::from_millis(25)).await;
                }
            }
        });
        wal.released.store(true, Ordering::Release);
        wal.changed.notify_waiters();
        let mut delivered_outcome = None;
        let result = tokio::time::timeout(std::time::Duration::from_secs(3), async {
            delivered_outcome = Some((&mut submitted).await);
            let expected = if committed_local { 2 } else { 1 };
            while coord.applied_seq() < expected {
                tokio::task::yield_now().await;
            }
        })
        .await;
        let completed_without_reservation_discard = result.is_ok();
        let outcome = match result {
            Ok(()) => delivered_outcome.take().expect("local result returned"),
            Err(_) => {
                // Teardown only: unblock a broken implementation so the Tokio
                // runtime does not wait forever for its native capacity waiter.
                // The acceptance boolean was captured before this intervention.
                drop(coord.local_reservations.lock().await.remove(&2));
                match delivered_outcome.take() {
                    Some(outcome) => outcome,
                    None => tokio::time::timeout(std::time::Duration::from_secs(3), &mut submitted)
                        .await
                        .expect("failure teardown must release the stalled reservation"),
                }
            }
        };
        stop.store(true, Ordering::Release);
        checkpoint.await.unwrap();
        if committed_local {
            outcome.unwrap().unwrap();
            assert_eq!(coord.applied_seq(), 2);
            assert_eq!(engine.stats("u").unwrap().documents_indexed, 2);
        } else {
            let error = outcome.unwrap().unwrap_err();
            assert!(error.downcast_ref::<PendingChangeCapacity>().is_some());
            assert_eq!(coord.applied_seq(), 1);
            assert_eq!(engine.stats("u").unwrap().documents_indexed, 1);
        }
        assert!(budget.high_water_bytes() <= hard);
        assert!(completed_without_reservation_discard,
            "future local reservation blocked the earlier external record despite a running checkpoint worker");
    }

    async fn wait_for_reserved(budget: &ChangeBudget) {
        tokio::time::timeout(std::time::Duration::from_secs(1), async {
            while budget.snapshot().reserved == 0 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("local published record must retain its reservation");
    }

    #[tokio::test]
    async fn no_guard_reprice_retains_charge_without_mutating_state_or_watermark() {
        let budget = ChangeBudget::with_hard_limit(1024 * 1024);
        let first = Engine::with_change_budget(budget.clone());
        let second = Arc::new(Engine::with_change_budget(budget));
        first.create_collection("u", keyword_schema()).unwrap();
        second.create_collection("u", keyword_schema()).unwrap();
        let coord = WriteCoordinator::start(Arc::new(MemWal::new()), second.clone());
        let entry = admitted_index_entry();
        let reservation = first.try_reserve_record(&entry, 0).unwrap();
        let bytes = reservation.bytes();

        let result = prepare_local_record(second.as_ref(), entry, reservation)
            .expect("wrong-engine reprice must return its original reservation");
        match result {
            PreparedLocalRecord::Prepared(_) => {
                panic!("wrong-engine reprice cannot acquire an apply guard")
            }
            PreparedLocalRecord::Unresolved { reservation, error } => {
                assert_eq!(
                    reservation.bytes(),
                    bytes,
                    "no-guard failure must retain reservation bytes"
                );
                assert!(
                    matches!(
                        error.downcast_ref::<RecordAdmissionError>(),
                        Some(RecordAdmissionError::WrongEngine)
                    ),
                    "wrong-engine no-guard failure must keep its typed admission error: {error:#}"
                );
            }
            PreparedLocalRecord::CapacityBlocked { .. } => {
                panic!("wrong-engine reprice must fail before capacity blocking")
            }
        }
        assert_eq!(
            second.stats("u").unwrap().documents_indexed,
            0,
            "preparation failure must not mutate collection state"
        );
        assert_eq!(
            coord.applied_seq(),
            0,
            "preparation failure must not advance the coordinator watermark"
        );
    }

    #[test]
    fn fitting_published_reprice_does_not_start_capacity_owner() {
        let budget = ChangeBudget::with_hard_limit(1024 * 1024);
        let active = Engine::with_change_budget(budget.clone());
        active.create_collection("u", keyword_schema()).unwrap();
        let entry = admitted_index_entry();

        // Build real stale state. The reservation predates a restore, so the
        // next begin must reprice it; the 1 MiB budget leaves room for growth.
        let initial = active.try_reserve_record(&entry, 0).unwrap();
        let initial = match prepare_local_record(&active, entry.clone(), initial).unwrap() {
            PreparedLocalRecord::Prepared(mut guard) => {
                active.apply_prepared_raft_entry(&mut guard).unwrap();
                drop(guard);
                active.freeze_checkpoint_collections(None).unwrap()
            }
            PreparedLocalRecord::Unresolved { .. } => {
                panic!("initial fitting record must be prepared")
            }
            PreparedLocalRecord::CapacityBlocked { .. } => {
                panic!("initial fitting record must not block")
            }
        };
        let replacement = Engine::with_change_budget(budget.clone());
        replacement
            .create_collection("u", keyword_schema())
            .unwrap();
        let reservation = active.try_reserve_record(&entry, 0).unwrap();
        active.restore(replacement.snapshot().unwrap()).unwrap();
        let before = budget.snapshot().total;
        let prepared = prepare_local_record(&active, entry, reservation)
            .expect("stale fitting reprice must grow without fallback maintenance");
        assert!(
            budget.snapshot().total > before,
            "fixture must exercise a real fitting reprice growth"
        );
        match prepared {
            PreparedLocalRecord::Prepared(mut guard) => {
                active.apply_prepared_raft_entry(&mut guard).unwrap();
                drop(guard);
            }
            PreparedLocalRecord::Unresolved { .. } => {
                panic!("fitting reprice must not use legacy fallback")
            }
            PreparedLocalRecord::CapacityBlocked { .. } => {
                panic!("fitting reprice must not block")
            }
        }
        drop(initial);
    }

    #[test]
    fn full_published_reprice_returns_capacity_blocked_without_waiting() {
        let limit = 1024 * 1024;
        let budget = ChangeBudget::with_hard_limit(limit);
        let active = Engine::with_change_budget(budget.clone());
        active.create_collection("u", keyword_schema()).unwrap();
        let entry = admitted_index_entry();

        let initial = active.try_reserve_record(&entry, 0).unwrap();
        let initial = match prepare_local_record(&active, entry.clone(), initial).unwrap() {
            PreparedLocalRecord::Prepared(mut guard) => {
                active.apply_prepared_raft_entry(&mut guard).unwrap();
                drop(guard);
                active.freeze_checkpoint_collections(None).unwrap()
            }
            PreparedLocalRecord::Unresolved { .. } => {
                panic!("initial record must be prepared")
            }
            PreparedLocalRecord::CapacityBlocked { .. } => {
                panic!("initial record must not block")
            }
        };
        let replacement = Engine::with_change_budget(budget.clone());
        replacement
            .create_collection("u", keyword_schema())
            .unwrap();
        let reservation = active.try_reserve_record(&entry, 0).unwrap();
        let reserved_bytes = reservation.bytes();
        active.restore(replacement.snapshot().unwrap()).unwrap();

        let filler_owner = budget.owner();
        let remaining = limit - budget.snapshot().total;
        let _filler = filler_owner.try_reserve(remaining).unwrap();
        let result = prepare_local_record(&active, entry, reservation).unwrap();
        match result {
            PreparedLocalRecord::Prepared(_) => {
                panic!("a full post-WAL reprice must not acquire an apply guard")
            }
            PreparedLocalRecord::CapacityBlocked {
                reservation,
                required,
                ..
            } => {
                assert_eq!(reservation.bytes(), reserved_bytes);
                assert!(required > reservation.bytes());
                assert!(
                    matches!(
                        active.try_reserve_record(&admitted_index_entry(), 0),
                        Err(RecordAdmissionError::Capacity(AdmissionError::Full { .. }))
                    ),
                    "a later record must not pass the unresolved retained head"
                );
            }
            PreparedLocalRecord::Unresolved { .. } => {
                panic!("capacity Full must retain a retryable blocked state")
            }
        }
        drop(initial);
    }

    #[test]
    fn full_published_reprice_waits_outside_apply_lease_then_applies_after_relief() {
        let limit = 1024 * 1024;
        let budget = ChangeBudget::with_hard_limit(limit);
        let active = Engine::with_change_budget(budget.clone());
        active.create_collection("u", keyword_schema()).unwrap();
        let entry = admitted_index_entry();

        let initial = active.try_reserve_record(&entry, 0).unwrap();
        let initial = match prepare_local_record(&active, entry.clone(), initial).unwrap() {
            PreparedLocalRecord::Prepared(mut guard) => {
                active.apply_prepared_raft_entry(&mut guard).unwrap();
                drop(guard);
                active.freeze_checkpoint_collections(None).unwrap()
            }
            _ => panic!("initial record must be prepared"),
        };
        let replacement = Engine::with_change_budget(budget.clone());
        replacement
            .create_collection("u", keyword_schema())
            .unwrap();
        let reservation = active.try_reserve_record(&entry, 0).unwrap();
        active.restore(replacement.snapshot().unwrap()).unwrap();
        let filler_owner = budget.owner();
        let filler = filler_owner
            .try_reserve(limit - budget.snapshot().total)
            .unwrap();
        let blocked = prepare_local_record(&active, entry, reservation).unwrap();
        let PreparedLocalRecord::CapacityBlocked {
            entry,
            mut reservation,
            required,
        } = blocked
        else {
            panic!("full reprice must retain a blocked head")
        };

        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let waiter = std::thread::spawn(move || {
            started_tx.send(()).unwrap();
            reservation
                .wait_grow_to(required)
                .map(|()| (entry, reservation))
        });
        started_rx.recv().unwrap();
        for _ in 0..10_000 {
            if budget.has_capacity_waiters() {
                break;
            }
            std::thread::yield_now();
        }
        assert!(
            budget.has_capacity_waiters(),
            "the blocked committed head must wait on the budget without an apply lease"
        );
        assert!(matches!(
            active.try_reserve_record(&admitted_index_entry(), 0),
            Err(RecordAdmissionError::Capacity(AdmissionError::Full { .. }))
        ));

        drop(filler);
        let (entry, reservation) = waiter.join().unwrap().unwrap();
        let prepared = prepare_local_record(&active, entry, reservation).unwrap();
        match prepared {
            PreparedLocalRecord::Prepared(mut guard) => {
                active.apply_prepared_raft_entry(&mut guard).unwrap();
            }
            _ => panic!("capacity relief must let the retained head acquire its apply guard"),
        }
        assert_eq!(active.stats("u").unwrap().documents_indexed, 1);
        drop(initial);
    }

    #[tokio::test]
    async fn full_local_admission_refuses_before_wal_publication() {
        let entry = admitted_index_entry();
        let calibration_budget = ChangeBudget::with_hard_limit(1024 * 1024);
        let calibration = Engine::with_change_budget(calibration_budget);
        calibration
            .create_collection("u", keyword_schema())
            .unwrap();
        let base = calibration.try_reserve_record(&entry, 0).unwrap().bytes();
        let raw = Engine::record_owned_bytes(&entry).unwrap();
        let requested = calibration
            .try_reserve_record(&entry, raw.checked_mul(2).unwrap())
            .unwrap()
            .bytes();
        assert!(requested > base);

        let budget = ChangeBudget::with_hard_limit(requested.checked_mul(2).unwrap());
        let engine = Arc::new(Engine::with_change_budget(budget.clone()));
        engine.create_collection("u", keyword_schema()).unwrap();
        let remaining = requested.checked_mul(2).unwrap() - budget.snapshot().total;
        let _held = engine.try_reserve_record(&entry, remaining - base).unwrap();
        let wal = Arc::new(MemWal::new());
        let coord = WriteCoordinator::start(wal.clone(), engine);
        let before = coord.engine.metrics().segment_backpressure_total.get();

        let error = coord.submit(entry).await.unwrap_err();
        assert!(error.downcast_ref::<PendingChangeCapacity>().is_some());
        assert_eq!(
            coord.engine.metrics().segment_backpressure_total.get(),
            before + 1,
            "one final pre-publication refusal increments once"
        );
        assert_eq!(wal.latest_seq().await.unwrap(), 0);
        assert_eq!(coord.applied_seq(), 0);
    }

    #[tokio::test]
    async fn local_success_and_domain_error_do_not_count_segment_backpressure() {
        let budget = ChangeBudget::with_hard_limit(1024 * 1024);
        let engine = Arc::new(Engine::with_change_budget(budget));
        engine.create_collection("u", keyword_schema()).unwrap();
        let coord = WriteCoordinator::start(Arc::new(MemWal::new()), engine.clone());
        let before = engine.metrics().segment_backpressure_total.get();

        coord.submit(admitted_index_entry()).await.unwrap();
        let mut invalid = admitted_index_entry();
        let RaftLogEntry::Index { req, .. } = &mut invalid else {
            unreachable!("the admitted fixture is an Index request")
        };
        req.items[0].field = "missing-field".to_owned();
        let error = coord.submit(invalid).await.unwrap_err();

        assert!(error.downcast_ref::<PendingChangeCapacity>().is_none());
        assert_eq!(
            engine.metrics().segment_backpressure_total.get(),
            before,
            "successful and domain-error applies are not pre-publication capacity refusals"
        );
    }

    #[tokio::test]
    async fn local_admission_composite_drops_both_reservation_halves() {
        let budget = ChangeBudget::with_hard_limit(1024 * 1024);
        let engine = Arc::new(Engine::with_change_budget(budget.clone()));
        engine.create_collection("u", keyword_schema()).unwrap();
        let coord = WriteCoordinator::start(Arc::new(MemWal::new()), engine);
        let baseline = budget.snapshot().total;

        let local = coord
            .try_admit_local_record(&admitted_index_entry())
            .unwrap()
            .expect("bounded local record must reserve before WAL publication");
        assert!(local.transient.is_some());
        assert_eq!(budget.snapshot().reserved, local.bytes());
        drop(local);
        assert_eq!(
            budget.snapshot().total,
            baseline,
            "dropping a capacity-relief map entry must release apply and transient halves"
        );
    }

    #[tokio::test]
    async fn local_capacity_refusal_starts_checkpoint_owner_without_committed_waiter() {
        let budget = ChangeBudget::with_hard_limit(1024 * 1024);
        let engine = Arc::new(Engine::with_change_budget(budget.clone()));
        engine.create_collection("u", keyword_schema()).unwrap();
        let coord = WriteCoordinator::start(Arc::new(MemWal::new()), engine);
        let filler_owner = budget.owner();
        let _filler = filler_owner
            .try_reserve(1024 * 1024 - budget.snapshot().total)
            .unwrap();

        let error = match coord.try_admit_local_record(&admitted_index_entry()) {
            Ok(_) => panic!("full local admission must be refused"),
            Err(error) => error,
        };
        assert!(
            error.downcast_ref::<PendingChangeCapacity>().is_some(),
            "a full local admission remains a typed retryable refusal"
        );
        assert!(
            coord
                .layer_capacity_owner
                .lock()
                .expect("capacity owner poisoned")
                .is_some(),
            "a local refusal must create independent checkpoint maintenance"
        );
    }

    #[tokio::test]
    async fn cancelled_local_submit_keeps_its_reservation_until_apply_finishes() {
        let budget = ChangeBudget::with_hard_limit(1024 * 1024);
        let engine = Arc::new(Engine::with_change_budget(budget.clone()));
        engine.create_collection("u", keyword_schema()).unwrap();
        let wal = ControlledWal::paused(ControlledWal::PAUSE_DELIVERY);
        let coord = WriteCoordinator::start(wal.clone(), engine);
        let submit = tokio::spawn({
            let coord = coord.clone();
            async move { coord.submit(admitted_index_entry()).await }
        });

        tokio::time::timeout(
            std::time::Duration::from_secs(1),
            wal.observed_publish.notified(),
        )
        .await
        .expect("submit must publish after pre-admission");
        wait_for_reserved(&budget).await;
        {
            let ledger = coord.local_reservations.lock().await;
            let local = ledger
                .get(&1)
                .expect("cancelled local submit must retain its composite reservation");
            assert!(
                local.transient.is_some(),
                "local admission must retain transport/AOF ownership with apply ownership"
            );
            assert_eq!(
                local.bytes(),
                budget.snapshot().reserved,
                "the sequence map must retain both reservation halves"
            );
        }
        submit.abort();
        assert!(submit.await.unwrap_err().is_cancelled());
        assert!(budget.snapshot().reserved > 0);
        assert_eq!(coord.applied_seq(), 0);

        wal.release_delivery.notify_one();
        tokio::time::timeout(std::time::Duration::from_secs(1), async {
            while coord.applied_seq() != 1 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("apply must consume the cancelled caller's sequence reservation");
        assert_eq!(budget.snapshot().reserved, 0);
        assert!(budget.snapshot().active > 0);
    }

    #[tokio::test]
    async fn publication_lookup_race_cannot_apply_before_reservation_installation() {
        let budget = ChangeBudget::with_hard_limit(1024 * 1024);
        let engine = Arc::new(Engine::with_change_budget(budget));
        engine.create_collection("u", keyword_schema()).unwrap();
        let wal = ControlledWal::paused(ControlledWal::PAUSE_PUBLISH);
        let coord = WriteCoordinator::start(wal.clone(), engine);
        let submit = tokio::spawn({
            let coord = coord.clone();
            async move { coord.submit(admitted_index_entry()).await }
        });

        tokio::time::timeout(std::time::Duration::from_secs(1), wal.delivered.notified())
            .await
            .expect("subscriber must observe the record while publish is paused");
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        assert_eq!(
            coord.applied_seq(),
            0,
            "apply must wait for the sequence reservation installed after publish returns"
        );

        wal.release_publish.notify_one();
        assert!(matches!(
            submit.await.unwrap().unwrap(),
            ApplyOutcome::Indexed(_)
        ));
        assert_eq!(coord.applied_seq(), 1);
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

    /// #4326: `lumen_coordinator_apply_seconds`/`lumen_coordinator_apply_items_total`
    /// must observe exactly once per admitted local `submit`, under the
    /// `index` kind with `items_total` equal to the submitted doc count —
    /// the per-record apply cost a 500k-doc perf probe reads from
    /// `GET /metrics`.
    #[tokio::test]
    async fn submit_of_index_record_observes_coordinator_apply_histogram() {
        let engine = Arc::new(Engine::new());
        let wal = Arc::new(MemWal::new());
        let coord = WriteCoordinator::start(wal, engine.clone());

        coord
            .submit(RaftLogEntry::CreateCollection {
                collection_id: "u".into(),
                req: keyword_schema(),
            })
            .await
            .unwrap();

        assert_eq!(
            engine
                .metrics()
                .render()
                .matches("lumen_coordinator_apply_seconds_count{kind=\"index\"} 0")
                .count(),
            1,
            "index kind must still emit its zero row before any index submit"
        );

        coord
            .submit(RaftLogEntry::Index {
                collection_id: "u".into(),
                req: IndexRequest {
                    items: vec![
                        IndexItem {
                            external_id: "u1".into(),
                            field: "email".into(),
                            value: FieldValue::String("a@x.com".into()),
                            version: None,
                        },
                        IndexItem {
                            external_id: "u2".into(),
                            field: "email".into(),
                            value: FieldValue::String("b@x.com".into()),
                            version: None,
                        },
                    ],
                    request_id: None,
                },
            })
            .await
            .unwrap();

        let out = engine.metrics().render();
        assert!(
            out.contains("lumen_coordinator_apply_seconds_count{kind=\"index\"} 1"),
            "expected exactly one observed index apply in:\n{out}"
        );
        assert!(
            out.contains("lumen_coordinator_apply_items_total{kind=\"index\"} 2"),
            "expected items_total to equal the submitted doc count in:\n{out}"
        );
        for stage in [
            "admission_to_mutation_gate",
            "publish_to_apply_start",
            "apply_to_waiter",
        ] {
            assert!(
                out.contains(&format!(
                    "lumen_coordinator_stage_seconds_count{{kind=\"index\",stage=\"{stage}\"}} 1"
                )),
                "expected one {stage} observation in:\n{out}"
            );
        }
    }

    #[tokio::test]
    async fn exclusive_mutation_fence_blocks_publish_until_released() {
        let engine = Arc::new(Engine::new());
        let wal = Arc::new(MemWal::new());
        let coord = WriteCoordinator::start(wal, engine);
        let fence = coord.fence_mutations().await.unwrap();

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
        let permit = coord.mutation_gate.shared().await.unwrap();
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
            .expect("the fence must open after the sequence completes")
            .expect("the process must not require restart");
        assert_eq!(coord.applied_seq(), 1);
    }

    #[tokio::test]
    async fn restart_required_rejects_mutations_and_never_clears_itself() {
        let engine = Arc::new(Engine::new());
        let wal = Arc::new(MemWal::new());
        let coord = WriteCoordinator::start(wal, engine);

        coord.require_restart();
        assert!(coord.is_restart_required());
        assert!(coord.mutation_gate().is_restart_required());

        let error = coord
            .submit(RaftLogEntry::CreateCollection {
                collection_id: "u".into(),
                req: keyword_schema(),
            })
            .await
            .expect_err("restart latch must reject new writes");
        assert!(error.downcast_ref::<RestartRequired>().is_some(), "{error}");
        assert!(coord.fence_mutations().await.is_err());
        assert!(coord.checkpoint_permit().await.is_err());
        assert!(coord.is_restart_required());
        assert_eq!(coord.applied_seq(), 0, "rejected write must not publish");
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
        let aof = Arc::new(Mutex::new(
            crate::aof::AofWriter::open_with_policy(&aof_path, crate::aof::FsyncPolicy::EverySec)
                .unwrap(),
        ));
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

    #[tokio::test]
    async fn partial_error_record_is_persisted_and_replays_its_earlier_mutation() {
        let dir = tempfile::tempdir().unwrap();
        let aof_path = dir.path().join("aof.log");
        let aof = Arc::new(Mutex::new(crate::aof::AofWriter::open(&aof_path).unwrap()));
        let engine = Arc::new(Engine::new());
        let coord =
            WriteCoordinator::start_from_with_aof(Arc::new(MemWal::new()), engine.clone(), 0, aof);
        coord
            .submit(RaftLogEntry::CreateCollection {
                collection_id: "u".into(),
                req: keyword_schema(),
            })
            .await
            .unwrap();
        let error = coord
            .submit(RaftLogEntry::Index {
                collection_id: "u".into(),
                req: IndexRequest {
                    items: vec![
                        IndexItem {
                            external_id: "u1".into(),
                            field: "email".into(),
                            value: FieldValue::String("ok".into()),
                            version: None,
                        },
                        IndexItem {
                            external_id: "u2".into(),
                            field: "missing".into(),
                            value: FieldValue::String("bad".into()),
                            version: None,
                        },
                    ],
                    request_id: None,
                },
            })
            .await
            .unwrap_err();
        assert!(error
            .downcast_ref::<crate::storage::StorageError>()
            .is_some());
        assert_eq!(engine.stats("u").unwrap().documents_indexed, 1);
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

    #[tokio::test]
    async fn partial_error_aof_gap_is_uncertain_and_blocks_later_append() {
        let dir = tempfile::tempdir().unwrap();
        let aof_path = dir.path().join("aof.log");
        let aof = Arc::new(Mutex::new(crate::aof::AofWriter::open(&aof_path).unwrap()));
        let engine = Arc::new(Engine::new());
        let coord = WriteCoordinator::start_from_with_aof(
            Arc::new(MemWal::new()),
            engine.clone(),
            0,
            aof.clone(),
        );
        coord
            .submit(RaftLogEntry::CreateCollection {
                collection_id: "u".into(),
                req: keyword_schema(),
            })
            .await
            .unwrap();
        aof.lock().unwrap().set_inject_storage_full(true);
        let error = coord
            .submit(RaftLogEntry::Index {
                collection_id: "u".into(),
                req: IndexRequest {
                    items: vec![
                        IndexItem {
                            external_id: "u1".into(),
                            field: "email".into(),
                            value: FieldValue::String("kept-live".into()),
                            version: None,
                        },
                        IndexItem {
                            external_id: "u2".into(),
                            field: "missing".into(),
                            value: FieldValue::String("invalid".into()),
                            version: None,
                        },
                    ],
                    request_id: None,
                },
            })
            .await
            .unwrap_err();
        assert!(
            error.downcast_ref::<StorageFullError>().is_some(),
            "{error}"
        );
        // The AOF failure occurs before preparation and apply.
        assert_eq!(engine.stats("u").unwrap().documents_indexed, 0);
        assert!(coord.is_restart_required());
        aof.lock().unwrap().set_inject_storage_full(false);

        let later = coord
            .submit(RaftLogEntry::Index {
                collection_id: "u".into(),
                req: IndexRequest {
                    items: vec![IndexItem {
                        external_id: "u3".into(),
                        field: "email".into(),
                        value: FieldValue::String("must-not-append".into()),
                        version: None,
                    }],
                    request_id: None,
                },
            })
            .await
            .unwrap_err();
        assert!(later.downcast_ref::<RestartRequired>().is_some(), "{later}");
        let mut persisted = Vec::new();
        crate::aof::AofReader::replay(&aof_path, 0, |seq, _| persisted.push(seq)).unwrap();
        assert_eq!(persisted, vec![1]);
    }

    /// #2516: prove the REAL ENOSPC detection/classification/metrics path
    /// end to end, through the actual production write path — not a
    /// parallel fake. Uses `AofWriter::set_inject_storage_full` (the
    /// `#[cfg(test)]` fault-injection seam on the real `AofWriter::append`,
    /// scoped to this test's own writer instance so parallel test threads
    /// never cross-contaminate) so the apply loop's genuine
    /// AOF-persist-failure branch runs.
    #[tokio::test]
    async fn aof_enospc_marks_degraded_and_requires_restart() {
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
        assert!(
            coord.is_restart_required(),
            "an applied mutation with no AOF record cannot be repaired in-process"
        );

        // A successful disk-space probe may clear the ENOSPC gauge. It must not
        // clear the independent durability-gap latch.
        engine.metrics().clear_storage_degraded();
        let error = coord
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
            .expect_err("a process with an AOF gap must reject later mutations");
        assert!(error.downcast_ref::<RestartRequired>().is_some(), "{error}");
        assert!(!engine.metrics().is_storage_degraded());
        assert!(coord.is_restart_required());
    }

    #[tokio::test]
    async fn aof_gap_rejects_every_later_applied_record() {
        let dir = tempfile::tempdir().unwrap();
        let aof_path = dir.path().join("aof.log");
        let mut writer = crate::aof::AofWriter::open(&aof_path).unwrap();
        writer.inject_failure_once(std::io::ErrorKind::Other);
        let aof = Arc::new(Mutex::new(writer));

        let wal = Arc::new(MemWal::new());
        for collection_id in ["first", "second"] {
            wal.publish(WalRecord::new(RaftLogEntry::CreateCollection {
                collection_id: collection_id.into(),
                req: keyword_schema(),
            }))
            .await
            .unwrap();
        }

        let engine = Arc::new(Engine::new());
        let coord = WriteCoordinator::start_from_with_aof(wal, engine.clone(), 0, aof);
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while !coord.is_restart_required() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("the unresolved AOF head must require restart");

        assert!(coord.is_restart_required());
        assert_eq!(coord.applied_seq(), 0);
        // The unresolved AOF head is rejected before apply.
        assert!(engine.list_collections().unwrap().is_empty());
        let mut persisted = Vec::new();
        crate::aof::AofReader::replay(&aof_path, 0, |seq, _| persisted.push(seq)).unwrap();
        assert!(
            persisted.is_empty(),
            "the unresolved head and every later record must stay outside the AOF"
        );
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
        assert_eq!(coord.applied_seq(), 1);
        assert!(matches!(
            coord
                .submit(RaftLogEntry::CreateCollection {
                    collection_id: "u".into(),
                    req: keyword_schema(),
                })
                .await
                .unwrap(),
            ApplyOutcome::Created(_)
        ));
        assert_eq!(coord.applied_seq(), 2);
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

    #[tokio::test]
    async fn full_raw_delivery_replays_the_same_head_before_later_records() {
        use crate::change_budget::ChangeBudget;
        use crate::wal::{WalLog, WalStream};

        struct ObservedWal {
            inner: MemWal,
            subscriptions: AtomicU64,
            subscribed: tokio::sync::Notify,
        }
        #[async_trait::async_trait]
        impl WalLog for ObservedWal {
            async fn publish(&self, record: WalRecord) -> Result<u64> {
                self.inner.publish(record).await
            }
            async fn subscribe(&self, from: u64) -> Result<WalStream> {
                let stream = self.inner.subscribe(from).await?;
                self.subscriptions.fetch_add(1, Ordering::Release);
                self.subscribed.notify_one();
                Ok(stream)
            }
            async fn latest_seq(&self) -> Result<u64> {
                self.inner.latest_seq().await
            }
        }

        let budget = ChangeBudget::with_hard_limit(1024 * 1024);
        let blocking_owner = budget.owner();
        let engine = Arc::new(Engine::with_change_budget(budget.clone()));
        engine.create_collection("u", keyword_schema()).unwrap();
        // Keep this fixture reserved-only. A fallback owner may validly publish
        // real active schema work, which would make a synthetic Full assertion
        // race with maintenance rather than exercise the pinned raw head.
        let schema_dir = tempfile::tempdir().unwrap();
        let schema_store = crate::segment_rdb::SegmentRdbStore::new(schema_dir.path()).unwrap();
        schema_store.save(&engine, 0).unwrap();
        assert_eq!(
            budget.snapshot().active,
            0,
            "schema checkpoint must freeze fixture work"
        );
        let blocking = blocking_owner
            .try_reserve(1024 * 1024 - budget.snapshot().total)
            .unwrap();
        let wal = Arc::new(ObservedWal {
            inner: MemWal::new(),
            subscriptions: AtomicU64::new(0),
            subscribed: tokio::sync::Notify::new(),
        });
        let record = |id: &str| {
            WalRecord::new(RaftLogEntry::Index {
                collection_id: "u".into(),
                req: IndexRequest {
                    items: vec![IndexItem {
                        external_id: id.into(),
                        field: "email".into(),
                        value: FieldValue::String(format!("{id}@example.test")),
                        version: None,
                    }],
                    request_id: None,
                },
            })
        };
        assert_eq!(wal.publish(record("first")).await.unwrap(), 1);
        let dir = tempfile::tempdir().unwrap();
        let aof_path = dir.path().join("aof.log");
        let aof = Arc::new(Mutex::new(crate::aof::AofWriter::open(&aof_path).unwrap()));
        let coord = WriteCoordinator::start_from_with_aof(wal.clone(), engine.clone(), 0, aof);
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while wal.subscriptions.load(Ordering::Acquire) < 2 {
                wal.subscribed.notified().await;
            }
        })
        .await
        .expect("full raw admission must pin a replay subscription");
        assert!(
            coord
                .layer_capacity_owner
                .lock()
                .expect("capacity owner poisoned")
                .is_some(),
            "committed native raw admission must start independent capacity maintenance before waiting",
        );
        assert_eq!(wal.publish(record("second")).await.unwrap(), 2);
        assert_eq!(coord.applied_seq(), 0, "no unowned working copy may apply");
        assert_eq!(engine.stats("u").unwrap().documents_indexed, 0);
        assert_eq!(budget.snapshot().total, 1024 * 1024);
        // The raw delivery was destroyed before the blocking reserve. Dropping
        // the real competing reservation supplies space without an apply lease.
        drop(blocking);
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while coord.applied_seq() < 2 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("the original head and its successor must both apply");
        assert!(!coord.is_restart_required());
        assert_eq!(engine.stats("u").unwrap().documents_indexed, 2);
        let mut persisted = Vec::new();
        crate::aof::AofReader::replay(&aof_path, 0, |seq, rec| {
            let RaftLogEntry::Index { req, .. } = rec.entry else {
                panic!("unexpected AOF operation")
            };
            persisted.push((seq, req.items[0].external_id.clone()));
        })
        .unwrap();
        assert_eq!(persisted, vec![(1, "first".into()), (2, "second".into())]);
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
        let permit = coord.mutation_gate.shared().await.unwrap();
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

    #[tokio::test]
    async fn unresolved_head_result_survives_waiter_registration_race() {
        let coord = WriteCoordinator::start(Arc::new(MemWal::new()), Arc::new(Engine::new()));
        coord.fail_unresolved(
            1,
            Err(anyhow::Error::new(RestartRequired(
                "committed source is unresolved".into(),
            ))),
            None,
        );

        let permit = coord.mutation_gate.shared().await.unwrap();
        let receiver = coord
            .register_waiter(1, permit)
            .expect("late waiter must receive the unresolved head result");
        let error = tokio::time::timeout(std::time::Duration::from_secs(2), receiver)
            .await
            .expect("unresolved head result must not wait for submit timeout")
            .expect("unresolved head result must be sent")
            .expect_err("unresolved head cannot report success");
        assert!(error.downcast_ref::<RestartRequired>().is_some(), "{error}");
        assert_eq!(coord.applied_seq(), 0);
    }

    #[tokio::test]
    async fn unbounded_committed_delivery_latches_at_the_last_applied_head() {
        let engine = Arc::new(Engine::with_change_budget(ChangeBudget::with_hard_limit(1)));
        let wal = Arc::new(MemWal::new());
        for collection_id in ["first", "second"] {
            wal.publish(WalRecord::new(RaftLogEntry::CreateCollection {
                collection_id: collection_id.into(),
                req: keyword_schema(),
            }))
            .await
            .unwrap();
        }

        let coord = WriteCoordinator::start(wal, engine.clone());
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while !coord.is_restart_required() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("an unbounded committed delivery must require restart");

        assert_eq!(coord.applied_seq(), 0);
        assert!(engine.list_collections().unwrap().is_empty());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn capacity_relief_stages_applied_sources_pinned_by_a_slow_subscriber() {
        check_slow_subscriber_capacity_relief(true).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn capacity_refusal_stages_applied_sources_without_a_waiting_apply() {
        check_slow_subscriber_capacity_relief(false).await;
    }

    async fn check_slow_subscriber_capacity_relief(wait_for_capacity: bool) {
        const LIMIT: usize = 4 * 1024 * 1024;
        let budget = ChangeBudget::with_hard_limit(LIMIT);
        let engine = Arc::new(Engine::with_change_budget(budget.clone()));
        let wal = Arc::new(MemWal::new());
        let _slow = wal.subscribe_admitted(0).await.unwrap();
        let coord = WriteCoordinator::start(wal.clone(), engine.clone());
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
                        external_id: "retained".into(),
                        field: "email".into(),
                        value: FieldValue::String("x".repeat(32 * 1024)),
                        version: None,
                    }],
                    request_id: None,
                },
            })
            .await
            .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let store = crate::segment_rdb::SegmentRdbStore::new(dir.path()).unwrap();
        store.save(&engine, coord.applied_seq()).unwrap();
        let pinned = budget.snapshot().total;
        assert!(
            pinned >= 32 * 1024,
            "checkpoint must retain the slow subscriber's raw source charge"
        );
        let filler_owner = budget.owner();
        let filler = filler_owner.try_reserve(LIMIT - pinned).unwrap();
        if wait_for_capacity {
            let waiting_owner = budget.owner();
            let mut waiting = tokio::task::spawn_blocking(move || waiting_owner.wait_reserve(1));
            let progressed =
                tokio::time::timeout(std::time::Duration::from_secs(5), &mut waiting).await;
            let staged_without_releasing_filler = progressed.is_ok();
            // Always join the blocking waiter, including the failed progress case.
            drop(filler);
            if let Ok(result) = progressed {
                drop(result.unwrap().unwrap());
            } else {
                drop(waiting.await.unwrap().unwrap());
            }
            assert!(
                staged_without_releasing_filler,
                "capacity relief must stage an applied source held by a slow subscriber"
            );
        } else {
            let error = coord
                .submit(RaftLogEntry::CreateCollection {
                    collection_id: "refused".into(),
                    req: keyword_schema(),
                })
                .await
                .unwrap_err();
            assert!(error.downcast_ref::<PendingChangeCapacity>().is_some());
            assert_eq!(coord.applied_seq(), 2);
            assert!(!budget.has_capacity_waiters());
            let relieved = tokio::time::timeout(std::time::Duration::from_secs(5), async {
                while budget.snapshot().total == LIMIT {
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
            })
            .await;
            drop(filler);
            assert!(
                relieved.is_ok(),
                "pre-publication capacity refusal must stage pinned sources without a waiting apply"
            );
            assert!(engine.stats("refused").is_err());
        }
        assert_eq!(engine.stats("u").unwrap().documents_indexed, 1);
        assert!(!coord.is_restart_required());
    }

    /// A door refusal (HTTP 429) sets `capacity_relief_requested` with no
    /// committed apply blocked on capacity. That alone must not let the
    /// relief task steal the ledger reservation of a different, already
    /// admitted local record that is simply waiting its turn to apply.
    /// Stealing it forces the apply loop to re-reserve from scratch and can
    /// park it in `wait_reserve_record_ram` until checkpoint frees bytes —
    /// exactly the > 5s perf-run stall this test guards against.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn capacity_relief_leaves_a_fresh_local_reservation_when_no_apply_is_waiting() {
        struct DeferredAdmittedWal {
            inner: MemWal,
            released: Arc<std::sync::atomic::AtomicBool>,
            changed: Arc<tokio::sync::Notify>,
        }
        #[async_trait::async_trait]
        impl WalLog for DeferredAdmittedWal {
            async fn publish(&self, record: WalRecord) -> Result<u64> {
                self.inner.publish(record).await
            }
            async fn latest_seq(&self) -> Result<u64> {
                self.inner.latest_seq().await
            }
            async fn stage_source(&self, seq: u64) -> Result<Option<crate::wal::WalSourceRelease>> {
                self.inner.stage_source(seq).await
            }
            async fn subscribe(&self, from: u64) -> Result<WalStream> {
                self.inner.subscribe(from).await
            }
            async fn subscribe_admitted(
                &self,
                from: u64,
            ) -> Result<crate::wal::WalAdmissionStream> {
                let stream = self.inner.subscribe_admitted(from).await?;
                let state = (stream, self.released.clone(), self.changed.clone());
                Ok(Box::pin(futures::stream::unfold(
                    state,
                    |(mut stream, released, changed)| async move {
                        loop {
                            let wake = changed.notified();
                            if released.load(Ordering::Acquire) {
                                break;
                            }
                            wake.await;
                        }
                        stream
                            .next()
                            .await
                            .map(|record| (record, (stream, released, changed)))
                    },
                )))
            }
        }

        let budget = ChangeBudget::with_hard_limit(1024 * 1024);
        let engine = Arc::new(Engine::with_change_budget(budget.clone()));
        engine.create_collection("u", keyword_schema()).unwrap();
        let wal = Arc::new(DeferredAdmittedWal {
            inner: MemWal::new(),
            released: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            changed: Arc::new(tokio::sync::Notify::new()),
        });
        let coord = WriteCoordinator::start(wal.clone(), engine.clone());

        let entry = RaftLogEntry::Index {
            collection_id: "u".into(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: "admitted".into(),
                    field: "email".into(),
                    value: FieldValue::String("x".repeat(4096)),
                    version: None,
                }],
                request_id: None,
            },
        };
        let mut submitted = tokio::spawn({
            let coord = coord.clone();
            async move { coord.submit(entry).await }
        });

        // Wait until the door-admitted record's reservation lands in the
        // pending ledger, while the apply loop stays deferred and cannot
        // take it yet.
        let seq = tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                if let Some(seq) = coord.local_reservations.lock().await.keys().next().copied() {
                    return seq;
                }
                assert!(
                    !submitted.is_finished(),
                    "submit finished before installing its reservation"
                );
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("the admitted local reservation must appear in the pending ledger");

        // Simulate a door refusal that requested relief with no committed
        // apply blocked on capacity.
        coord
            .capacity_relief_requested
            .store(true, Ordering::Release);
        assert!(!engine.has_capacity_waiters());

        // Give the relief task (25ms tick) several chances to (mis)fire.
        tokio::time::sleep(std::time::Duration::from_millis(120)).await;

        assert!(
            coord.local_reservations.lock().await.contains_key(&seq),
            "a door refusal alone must not steal an already-admitted, unapplied local reservation"
        );

        // Release the deferred apply loop and confirm the record still
        // applies cleanly, without going through a capacity wait.
        wal.released.store(true, Ordering::Release);
        wal.changed.notify_waiters();
        let outcome = tokio::time::timeout(std::time::Duration::from_secs(5), &mut submitted)
            .await
            .expect("submit must complete once the apply loop is released")
            .expect("submit task must not panic")
            .expect("submit must apply successfully");
        assert!(matches!(outcome, ApplyOutcome::Indexed(_)));
        assert_eq!(engine.stats("u").unwrap().documents_indexed, 1);
    }
}
// CODEGEN-END
