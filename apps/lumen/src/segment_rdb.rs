// CODEGEN-BEGIN
//! Durable segment-checkpoint generations.
//!
//! A checkpoint stores one directory per collection. Each collection contains
//! mmap segments and `_schema.json`. New checkpoints also contain one
//! top-level `_generation.json` manifest. The shared `storage-durable`
//! generation store fsyncs the complete tree and changes `CURRENT` atomically.
//! `CURRENT` is the only source of truth. A complete but unpointed directory is
//! never selected during restart.
//!
//! New directory names are `gen-<seq>-rev-<revision>`. A same-sequence save
//! creates a new immutable revision. A background save below the active
//! sequence is a no-op. Exact 0.4.28 `gen-<seq>` directories remain readable:
//! on the first 0.4.29 restart, Lumen validates the exact highest legacy
//! directory and then writes `CURRENT` once. It never falls back from a corrupt
//! highest legacy directory.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use storage_durable::{
    CurrentGenerationStaging, CurrentReadErrorKind, CurrentTarget, FailureInjector, GenerationName,
    GenerationStore, NoFailures, StagedGeneration,
};

use crate::capture_barrier::CaptureStamp;
use crate::storage::{Engine, FrozenCheckpoint, RecoveryPhase, RecoveryProfile};

#[path = "segment_background_merge.rs"]
mod background;
#[path = "segment_compaction.rs"]
mod compaction;
#[path = "segment_merge_rebase.rs"]
mod merge_rebase;
#[cfg(feature = "raft-wal")]
#[path = "segment_raft_archive.rs"]
pub(crate) mod raft_archive;
#[cfg(feature = "raft-wal")]
#[path = "segment_raft_capture.rs"]
pub(crate) mod raft_capture;
#[path = "segment_save_gate.rs"]
mod save_gate;
#[path = "segment_telemetry.rs"]
mod telemetry;
use save_gate::SaveGate;
use telemetry::MeasuredCapture;

const GENERATION_MANIFEST_FILE: &str = "_generation.json";
const GENERATION_MANIFEST_SCHEMA_VERSION: u32 = 2;
const GENERATION_MANIFEST_V2: u32 = 2;
const GENERATION_MANIFEST_V3: u32 = 3;
const FLAT_PAYLOAD_DIR: &str = "payload";
const CHECKPOINT_SCHEMA_FILE: &str = "_schema.json";
const CURRENT_FILE: &str = "CURRENT";
const CURRENT_TEMP_FILE: &str = "CURRENT.tmp";
const AOF_FILE: &str = "aof.log";
const AOF_COMPACT_TEMP_FILE: &str = "aof.log.compact.tmp";
const HNSW_GRAPH_CACHE_DIR: &str = "hnsw-graph-cache";
// The shipped image pre-populates its declared volume with this inert regular
// file so Docker recognizes a non-empty data directory. It carries no Lumen
// state and is part of a semantically new root.
const CONTAINER_VOLUME_SEED_FILE: &str = ".lumen-volume-seed";
// ext-family filesystems create this direct child at the root of a fresh
// volume. It is safe only when it remains an empty real directory.
const EXT_FILESYSTEM_METADATA_DIR: &str = "lost+found";

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MergePhase {
    BeforeEncode,
    BeforePublish,
    AfterPublish,
}
#[doc(hidden)]
pub trait MergeObserver: Send + Sync {
    fn observe(&self, phase: MergePhase) -> std::io::Result<()>;
}
struct NoMergeObserver;
impl MergeObserver for NoMergeObserver {
    fn observe(&self, _: MergePhase) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SegmentGenerationManifest {
    schema_version: u32,
    checkpoint_sequence: u64,
    revision: u64,
    previous: Option<String>,
    next_collection_generation: u64,
    collections: Vec<CollectionCatalog>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyV1GenerationManifest {
    schema_version: u32,
    sequence: u64,
    revision: u64,
    previous: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CollectionCatalog {
    collection_id: String,
    collection_generation: u64,
    schema_version: u32,
    data_version: u64,
    schema: serde_json::Value,
    segments: Vec<SegmentReference>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SegmentReference {
    role: SegmentRole,
    field: Option<String>,
    ordinal: u32,
    kind: SegmentKind,
    format: SegmentFormat,
    path: String,
    local_rows: Option<LocalRowsReference>,
    #[serde(default)]
    applied_seq: Option<u64>,
    #[serde(default)]
    payload_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SegmentRole {
    Field,
    CollectionEids,
    VectorEids,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SegmentKind {
    Base,
    Delta,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum SegmentFormat {
    #[serde(rename = "lseg-v1")]
    LsegV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalRowsReference {
    format: String,
    path: String,
    count: u32,
}

#[derive(Debug, Clone)]
struct GenerationRecord {
    name: GenerationName,
    path: PathBuf,
    sequence: u64,
    revision: u64,
    legacy: bool,
    previous: Option<GenerationName>,
}

/// The durable checkpoint decision made before Lumen starts accepting work.
///
/// This is intentionally about the checkpoint root only. The binary logs the
/// separate AOF replay decision after it applies the checkpoint baseline.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SegmentStartupDecision {
    InitializedEmptyRoot,
    RecoveredUncommittedEmpty,
    RestoredCurrentEmpty,
    RestoredCurrentGeneration,
    AdoptedLegacy0428,
}

impl SegmentStartupDecision {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InitializedEmptyRoot => "initialized_empty_root",
            Self::RecoveredUncommittedEmpty => "recovered_uncommitted_empty",
            Self::RestoredCurrentEmpty => "restored_current_empty",
            Self::RestoredCurrentGeneration => "restored_current_generation",
            Self::AdoptedLegacy0428 => "adopted_legacy_0428",
        }
    }
}

/// The exact successful checkpoint-root state selected during cold start.
#[derive(Clone, Debug)]
pub struct SegmentStartupOutcome {
    pub decision: SegmentStartupDecision,
    pub checkpoint_sequence: Option<u64>,
    pub generation: Option<GenerationName>,
    pub recovered_legacy_aside: bool,
    pub staging_cleaned: usize,
}

#[derive(Clone, Copy, Debug)]
enum StartupBootstrap {
    ExistingCurrent {
        staging_cleaned: usize,
    },
    InitializedEmpty {
        recovered_uncommitted: bool,
        staging_cleaned: usize,
    },
    Legacy {
        recovered_legacy_aside: bool,
        staging_cleaned: usize,
    },
}

#[derive(Debug, Default)]
struct RootInventory {
    non_seed_entries: usize,
    revision_generations: Vec<String>,
    has_aof_log: bool,
    has_aof_compact_temp: bool,
    has_graph_cache: bool,
}

/// The exact generation selected by `CURRENT`, reopened into a fresh engine.
#[derive(Clone)]
pub struct LoadedSegmentGeneration {
    pub name: GenerationName,
    pub sequence: u64,
    pub engine: Arc<Engine>,
}

impl GenerationRecord {
    fn order_key(&self) -> (u8, u64) {
        if self.legacy {
            (0, self.sequence)
        } else {
            (1, self.revision)
        }
    }
}

pub(crate) type CheckpointRootGuard = Arc<dyn Send + Sync>;

/// Filesystem-backed segment checkpoints selected through one durable
/// `CURRENT` pointer.
///
/// Clones and independently opened handles for the same canonical root share
/// `save_gate` inside one process. The gate covers preparation, abandoned-stage
/// cleanup, activation, reopen, and prune. As required by `GenerationStore`, one
/// process must own all mutations for a root; cross-process writers are not
/// supported.
#[derive(Clone)]
pub struct SegmentRdbStore {
    root: PathBuf,
    save_gate: Arc<SaveGate>,
    generations: GenerationStore,
    bootstrap: StartupBootstrap,
    // Clones share proof of the last verified immutable manifest. Unknown
    // generations require payload validation before the hard-link fast path.
    verified_catalog: Arc<Mutex<Option<(GenerationName, Vec<u8>)>>>,
    pending_frozen: Arc<Mutex<Option<PendingFrozenCheckpoint>>>,
    merge_observer: Arc<dyn MergeObserver>,
    background: Arc<background::RootWork>,
    root_guard: Option<CheckpointRootGuard>,
    publication_fence: Option<crate::segment_capacity::PublicationFence>,
    recovery_profile: RecoveryProfile,
    recovery_timings: Arc<Mutex<RecoveryTimings>>,
}

/// Aggregate timings for the pre-bind portion of durable recovery.
///
/// Keep this intentionally scalar-only. The profile must never retain or emit
/// paths, collection IDs, field names, document IDs, or document values.
#[derive(Default)]
struct RecoveryTimings {
    layout_validation_ms: u64,
    manifest_decode_ms: u64,
    base_decode_ms: u64,
    delta_decode_ms: u64,
    reopen_ms: u64,
    identity_hydration_ms: u64,
    vector_finish_ms: u64,
}

/// Keeps one immutable generation present while an external snapshot writer
/// reads it.  The generation may stop being `CURRENT`; pruning still observes
/// this root-local reference until the pin drops.
pub(crate) struct SegmentArchivePin {
    background: Arc<background::RootWork>,
    _root_guard: Option<CheckpointRootGuard>,
    name: String,
    path: PathBuf,
    sequence: u64,
}

impl SegmentArchivePin {
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn sequence(&self) -> u64 {
        self.sequence
    }
}

impl Drop for SegmentArchivePin {
    fn drop(&mut self) {
        self.background.unpin(&self.name);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PendingPredecessor {
    name: Option<String>,
    sequence: Option<u64>,
    revision: Option<u64>,
    legacy: Option<bool>,
}

impl PendingPredecessor {
    fn from_current(current: Option<&GenerationRecord>) -> Self {
        Self {
            name: current.map(|record| record.name.as_str().to_owned()),
            sequence: current.map(|record| record.sequence),
            revision: current.map(|record| record.revision),
            legacy: current.map(|record| record.legacy),
        }
    }
}

/// Ordinary saves never move a cut backward. A Raft restore replaces product
/// state from the authoritative durable Raft snapshot before replaying its log.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SaveIntent {
    Ordinary,
    ExactRaft,
    RaftRestore,
}

enum SaveAttempt {
    Complete(GenerationName),
    CapacityWait(u64),
    FreshCapture,
}

/// Identifies one diagnostic checkpoint attempt. This context is trace-only
/// and never participates in save selection or ownership.
#[derive(Clone, Copy)]
pub(crate) struct CheckpointDiagnosticContext {
    origin: &'static str,
    attempt_id: Option<u64>,
    started: Instant,
    #[cfg(test)]
    capture: Option<DiagnosticCaptureToken>,
}

/// A copyable test address. The registry owns the sender, so production
/// checkpoint contexts remain small and copyable across blocking tasks.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct DiagnosticCaptureToken(u64);

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NumericCanonicalEvent {
    Phase {
        attempt_id: u64,
        phase: &'static str,
        pass: u8,
        reused: bool,
        frozen_bytes: u64,
    },
    Selected {
        attempt_id: u64,
        reason: &'static str,
        source: &'static str,
        pending_total: usize,
    },
    Refusal {
        revision: u64,
        present: bool,
        requested: usize,
        used: usize,
        hard_limit: usize,
    },
}

#[cfg(test)]
type CaptureSenders = Mutex<HashMap<DiagnosticCaptureToken, std::sync::mpsc::SyncSender<NumericCanonicalEvent>>>;

#[cfg(test)]
fn capture_senders() -> &'static CaptureSenders {
    static SENDERS: OnceLock<CaptureSenders> = OnceLock::new();
    SENDERS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(test)]
pub(crate) fn send_numeric_event(token: DiagnosticCaptureToken, event: NumericCanonicalEvent) {
    // Never hold the registry lock while delivering. A full or closed test
    // channel must not change the checkpoint or refusal path.
    let sender = capture_senders()
        .lock()
        .ok()
        .and_then(|senders| senders.get(&token).cloned());
    if let Some(sender) = sender {
        let _ = sender.try_send(event);
    }
}

#[cfg(test)]
pub(crate) struct DiagnosticCapture {
    token: DiagnosticCaptureToken,
    receiver: std::sync::mpsc::Receiver<NumericCanonicalEvent>,
}

#[cfg(test)]
impl DiagnosticCapture {
    pub(crate) fn new() -> Self {
        static NEXT_TOKEN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let token = DiagnosticCaptureToken(NEXT_TOKEN
            .fetch_update(
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
                |next| next.checked_add(1),
            )
            .expect("diagnostic capture tokens exhausted"));
        let (sender, receiver) = std::sync::mpsc::sync_channel(64);
        capture_senders().lock().unwrap().insert(token, sender);
        Self { token, receiver }
    }

    pub(crate) fn token(&self) -> DiagnosticCaptureToken { self.token }

    pub(crate) fn drain(&self) -> Vec<NumericCanonicalEvent> {
        self.receiver.try_iter().collect()
    }
}

#[cfg(test)]
impl Drop for DiagnosticCapture {
    fn drop(&mut self) {
        if let Ok(mut senders) = capture_senders().lock() {
            senders.remove(&self.token);
        }
    }
}

impl CheckpointDiagnosticContext {
    pub(crate) fn new(origin: &'static str, attempt_id: Option<u64>) -> Self {
        Self {
            origin,
            attempt_id,
            started: Instant::now(),
            #[cfg(test)]
            capture: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_capture(mut self, token: DiagnosticCaptureToken) -> Self {
        self.capture = Some(token);
        self
    }

    #[cfg(test)]
    fn capture_event(self, event: NumericCanonicalEvent) {
        if let Some(token) = self.capture {
            send_numeric_event(token, event);
        }
    }

    fn attempt_id(self) -> Option<u64> {
        self.attempt_id
    }

    /// Emit one bounded, machine-readable lifecycle phase. A missing attempt
    /// id is intentionally silent, so ordinary saves keep their existing path.
    pub(crate) fn trace_phase(self, phase: &'static str) {
        let Some(checkpoint_attempt_id) = self.attempt_id() else {
            return;
        };
        #[cfg(test)]
        self.capture_event(NumericCanonicalEvent::Phase { attempt_id: checkpoint_attempt_id, phase, pass: 0, reused: false, frozen_bytes: 0 });
        tracing::info!(
            event = "segment_checkpoint_diagnostic_phase",
            phase,
            checkpoint_attempt_id,
            checkpoint_origin = self.origin,
            elapsed_ns = duration_ns(self.started.elapsed()),
            "segment checkpoint diagnostic phase"
        );
    }

    /// Record the scheduler branch that actually selected this checkpoint.
    /// This has no wake-lag field because a coalesced channel notice cannot
    /// provide a precise wake timestamp.
    pub(crate) fn trace_scheduler_selected(
        self,
        reason: &'static str,
        resample_source: &'static str,
        pending: crate::change_budget::Snapshot,
    ) {
        let Some(checkpoint_attempt_id) = self.attempt_id() else {
            return;
        };
        #[cfg(test)]
        self.capture_event(NumericCanonicalEvent::Selected { attempt_id: checkpoint_attempt_id, reason, source: resample_source, pending_total: pending.total });
        tracing::info!(
            event = "segment_checkpoint_diagnostic_phase",
            phase = "scheduler_selected",
            checkpoint_attempt_id,
            checkpoint_origin = self.origin,
            scheduler_reason = reason,
            resample_source,
            elapsed_ns = duration_ns(self.started.elapsed()),
            pending_total_bytes = pending.total,
            pending_reserved_bytes = pending.reserved,
            pending_active_bytes = pending.active,
            pending_frozen_bytes = pending.frozen,
            checkpoint_trigger_bytes = crate::change_budget::CHECKPOINT_TRIGGER,
            "segment checkpoint diagnostic phase"
        );
    }

    pub(crate) fn trace_freeze_completed(
        self,
        checkpoint_pass: u8,
        frozen_cut_reused: bool,
        frozen_cut_bytes: u64,
    ) {
        let Some(checkpoint_attempt_id) = self.attempt_id() else {
            return;
        };
        #[cfg(test)]
        self.capture_event(NumericCanonicalEvent::Phase { attempt_id: checkpoint_attempt_id, phase: "freeze_completed", pass: checkpoint_pass, reused: frozen_cut_reused, frozen_bytes: frozen_cut_bytes });
        tracing::info!(
            event = "segment_checkpoint_diagnostic_phase",
            phase = "freeze_completed",
            checkpoint_attempt_id,
            checkpoint_origin = self.origin,
            checkpoint_pass,
            frozen_cut_reused,
            frozen_cut_bytes,
            elapsed_ns = duration_ns(self.started.elapsed()),
            "segment checkpoint diagnostic phase"
        );
    }

    pub(crate) fn trace_publish_completed(self, checkpoint_pass: u8) {
        let Some(checkpoint_attempt_id) = self.attempt_id() else {
            return;
        };
        #[cfg(test)]
        self.capture_event(NumericCanonicalEvent::Phase { attempt_id: checkpoint_attempt_id, phase: "publish_completed", pass: checkpoint_pass, reused: false, frozen_bytes: 0 });
        tracing::info!(
            event = "segment_checkpoint_diagnostic_phase",
            phase = "publish_completed",
            checkpoint_attempt_id,
            checkpoint_origin = self.origin,
            checkpoint_pass,
            elapsed_ns = duration_ns(self.started.elapsed()),
            "segment checkpoint diagnostic phase"
        );
    }

    pub(crate) fn trace_terminal(self, result: &Result<()>) {
        let Some(checkpoint_attempt_id) = self.attempt_id() else {
            return;
        };
        #[cfg(test)]
        self.capture_event(NumericCanonicalEvent::Phase { attempt_id: checkpoint_attempt_id, phase: "terminal", pass: 0, reused: false, frozen_bytes: 0 });
        tracing::info!(
            event = "segment_checkpoint_diagnostic_phase",
            phase = "terminal",
            checkpoint_attempt_id,
            checkpoint_origin = self.origin,
            elapsed_ns = duration_ns(self.started.elapsed()),
            terminal_result = if result.is_ok() { "ok" } else { "error" },
            "segment checkpoint diagnostic phase"
        );
    }
}

/// Timing that starts before a root save permit is requested. A checkpoint
/// always takes this permit before it freezes a cut, so capture-to-gate wait is
/// structurally zero and is emitted as such in the diagnostic event.
#[derive(Clone, Copy)]
struct SaveGateTrace {
    wait_ns: u64,
    acquired_at: Instant,
}

impl SaveGateTrace {
    fn acquire(gate: &Arc<SaveGate>) -> (save_gate::SavePermit, Self) {
        let started = Instant::now();
        let permit = gate.lock_owned();
        (
            permit,
            Self {
                wait_ns: duration_ns(started.elapsed()),
                acquired_at: Instant::now(),
            },
        )
    }
}

fn duration_ns(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

pub(crate) fn checkpoint_diagnostic_enabled() -> bool {
    std::env::var("LUMEN_PERF_DIAGNOSTIC").as_deref() == Ok("1")
}

fn trace_save_gate_acquired(context: Option<CheckpointDiagnosticContext>, trace: SaveGateTrace) {
    // Gate timing remains in the one bounded completion record. It is not a
    // lifecycle phase, so one checkpoint never emits duplicate phase paths.
    let _ = (context, trace);
}

fn trace_capacity_wait_begin(context: Option<CheckpointDiagnosticContext>, revision: u64) {
    let _ = (context, revision);
}

fn trace_capacity_wait_end(
    context: Option<CheckpointDiagnosticContext>,
    revision: u64,
    duration_ns: Option<u64>,
    result: std::result::Result<&background::CapacityWait, &anyhow::Error>,
) {
    let _ = (context, revision, duration_ns, result);
}

fn trace_durable_save_end(
    context: Option<CheckpointDiagnosticContext>,
    sequence: u64,
    revision: u64,
    duration_ns: u64,
) {
    let _ = (context, sequence, revision, duration_ns);
}

#[derive(Clone, Copy)]
enum StagingSelection {
    Generic,
    CurrentIfDurable,
    BackgroundScratch,
}

enum GenerationStaging {
    Generic(StagedGeneration),
    Current(CurrentGenerationStaging),
}

impl GenerationStaging {
    fn generation(&self) -> &GenerationName {
        match self {
            Self::Generic(staged) => staged.generation(),
            Self::Current(staged) => staged.generation(),
        }
    }

    fn path(&self) -> &Path {
        match self {
            Self::Generic(staged) => staged.path(),
            Self::Current(staged) => staged.path(),
        }
    }

    fn commit_with_publication_guard<G>(
        self,
        store: &GenerationStore,
        acquire: impl FnOnce() -> std::io::Result<G>,
    ) -> Result<GenerationName, storage_durable::CommitError> {
        match self {
            Self::Generic(staged) => store.commit_with_publication_guard(staged, acquire),
            Self::Current(staged) => {
                store.commit_from_current_with_publication_guard(staged, acquire)
            }
        }
    }
}

struct PendingFrozenCheckpoint {
    engine: Weak<Engine>,
    stamp: CaptureStamp,
    sequence: u64,
    predecessor: PendingPredecessor,
    frozen: FrozenCheckpoint,
    detached_capture_ns: u64,
    _layer_window: crate::segment_capacity::FrozenWindow,
}

/// Restores detached payload ownership to the shared slot if any fallible
/// pre-publication step returns. `disarm` is valid only after durable publish
/// plus live Engine binding have both succeeded.
struct PendingFrozenLease {
    slot: Arc<Mutex<Option<PendingFrozenCheckpoint>>>,
    pending: Option<PendingFrozenCheckpoint>,
}

impl PendingFrozenLease {
    fn pending(&self) -> &PendingFrozenCheckpoint {
        self.pending
            .as_ref()
            .expect("pending frozen lease is armed")
    }

    fn disarm(&mut self) {
        self.pending.take();
    }
}

impl Drop for PendingFrozenLease {
    fn drop(&mut self) {
        let Some(pending) = self.pending.take() else {
            return;
        };
        let mut slot = self
            .slot
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(
            slot.is_none(),
            "save lock must serialize pending frozen ownership"
        );
        *slot = Some(pending);
    }
}

impl fmt::Debug for SegmentRdbStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SegmentRdbStore")
            .field("root", &self.root)
            .finish_non_exhaustive()
    }
}

impl SegmentRdbStore {
    /// Pin an already published immutable generation for a detached archive
    /// reader.  Callers hold the save permit while creating this pin, so prune
    /// cannot observe the generation between publication and protection.
    pub(crate) fn pin_published_generation(
        &self,
        name: &GenerationName,
    ) -> Result<SegmentArchivePin> {
        let record = self.record_for_name(name.clone())?;
        self.background.pin(name.as_str().to_owned());
        Ok(SegmentArchivePin {
            background: self.background.clone(),
            _root_guard: self.root_guard.clone(),
            name: name.as_str().to_owned(),
            path: record.path,
            sequence: record.sequence,
        })
    }

    pub(crate) fn with_root_guard(mut self, guard: CheckpointRootGuard) -> Self {
        self.root_guard = Some(guard);
        self
    }

    pub(crate) fn with_publication_fence(
        mut self,
        fence: crate::segment_capacity::PublicationFence,
    ) -> Self {
        self.publication_fence = Some(fence);
        self
    }

    pub(crate) fn has_publication_fence(&self) -> bool {
        self.publication_fence.is_some()
    }

    pub(crate) fn request_capacity_merge(&self, engine: &Arc<Engine>) -> Result<u64> {
        self.request_merge_for_capacity_retry(engine, None, None)
    }

    /// Wait only for the capacity request's next root publication or an idle
    /// queue. A later unrelated root job does not delay a new checkpoint.
    pub(crate) fn wait_for_capacity_merge_progress(
        &self,
        revision: u64,
        timeout: Duration,
    ) -> Result<()> {
        self.background
            .wait_for_capacity_progress_after(revision, Instant::now() + timeout)
            .map(|_| ())
    }

    pub(crate) fn has_current_generation(&self) -> Result<bool> {
        Ok(self.current_record()?.is_some())
    }

    fn retain_root_for(&self, engine: &Engine) {
        if let Some(guard) = &self.root_guard {
            engine.retain_checkpoint_root(guard.clone());
        }
    }

    fn reset_recovery_profile(&self) {
        self.recovery_profile.reset();
        if self.recovery_profile.enabled() {
            *self
                .recovery_timings
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = RecoveryTimings::default();
        }
    }

    fn add_recovery_timing(&self, update: impl FnOnce(&mut RecoveryTimings)) {
        if self.recovery_profile.enabled() {
            update(
                &mut self
                    .recovery_timings
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()),
            );
        }
    }

    fn emit_recovery_profile(&self) {
        let Some(profile) = self.recovery_profile.snapshot() else {
            return;
        };
        let timings = self
            .recovery_timings
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        tracing::info!(
            layout_validation_ms = timings.layout_validation_ms,
            manifest_decode_ms = timings.manifest_decode_ms,
            base_decode_ms = timings.base_decode_ms,
            delta_decode_ms = timings.delta_decode_ms,
            reopen_ms = timings.reopen_ms,
            collection_open_count = profile.collection_open_count,
            collection_open_total_ms = profile.collection_open_total_ms,
            collection_open_max_ms = profile.collection_open_max_ms,
            vector_flat_open_count = profile.vector_flat_open_count,
            vector_flat_open_ms = profile.vector_flat_open_ms,
            vector_hnsw_open_count = profile.vector_hnsw_open_count,
            vector_hnsw_open_ms = profile.vector_hnsw_open_ms,
            coverage_rebuild_ms = profile.coverage_rebuild_ms,
            identity_hydration_ms = timings.identity_hydration_ms,
            vector_finish_ms = timings.vector_finish_ms,
            "segment durable recovery profile"
        );
    }

    /// Open or create the checkpoint root.
    ///
    /// A genuinely empty root receives the explicit empty `CURRENT` sentinel.
    /// A root with exact 0.4.28 legacy generations remains uninitialized until
    /// [`Self::reopen_into_with_outcome`] validates and adopts the highest one.
    /// A non-empty root with an unknown layout fails before this method writes
    /// `CURRENT` or removes any entry.
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        Self::open_with_injector_and_observer(root, Arc::new(NoFailures), Arc::new(NoMergeObserver))
    }

    fn open_with_injector(
        root: impl Into<PathBuf>,
        injector: Arc<dyn FailureInjector>,
    ) -> Result<Self> {
        Self::open_with_injector_and_observer(root, injector, Arc::new(NoMergeObserver))
    }

    fn open_with_injector_and_observer(
        root: impl Into<PathBuf>,
        injector: Arc<dyn FailureInjector>,
        observer: Arc<dyn MergeObserver>,
    ) -> Result<Self> {
        let root = root.into();
        std::fs::create_dir_all(&root)
            .with_context(|| format!("create segment-checkpoint dir {}", root.display()))?;
        let generations = GenerationStore::open_with_injector(&root, injector)
            .with_context(|| format!("open generation store {}", root.display()))?;
        let root = std::fs::canonicalize(&root)
            .with_context(|| format!("canonicalize checkpoint root {}", root.display()))?;
        let store = Self {
            save_gate: shared_save_gate(&root)?,
            pending_frozen: shared_pending_frozen(&root)?,
            merge_observer: observer,
            background: background::shared(&root)?,
            root_guard: None,
            publication_fence: None,
            root,
            generations,
            bootstrap: StartupBootstrap::ExistingCurrent { staging_cleaned: 0 },
            verified_catalog: Arc::new(Mutex::new(None)),
            recovery_profile: RecoveryProfile::from_env(),
            recovery_timings: Arc::new(Mutex::new(RecoveryTimings::default())),
        };
        let _guard = store.save_gate.lock_owned();
        let bootstrap = store.prepare_startup_root()?;
        drop(_guard);
        Ok(Self { bootstrap, ..store })
    }

    #[doc(hidden)]
    pub fn with_merge_observer(
        root: impl Into<PathBuf>,
        observer: Arc<dyn MergeObserver>,
    ) -> Result<Self> {
        Self::open_with_injector_and_observer(root, Arc::new(NoFailures), observer)
    }

    /// Drive deterministic durability and merge interleavings in integration tests.
    #[doc(hidden)]
    pub fn with_failure_injector_and_merge_observer(
        root: impl Into<PathBuf>,
        injector: Arc<dyn FailureInjector>,
        observer: Arc<dyn MergeObserver>,
    ) -> Result<Self> {
        Self::open_with_injector_and_observer(root, injector, observer)
    }

    /// Open a store with deterministic filesystem failures for restore tests.
    #[cfg(test)]
    pub(crate) fn new_with_failure_injector(
        root: impl Into<PathBuf>,
        injector: Arc<dyn FailureInjector>,
    ) -> Result<Self> {
        Self::open_with_injector(root, injector)
    }

    /// Checkpoint `engine` through a new immutable generation.
    ///
    /// Same-sequence saves remain meaningful because reshard operations can
    /// change state without advancing `applied_seq`. A lower sequence can only
    /// be a stale background caller, so it returns without moving `CURRENT`.
    pub fn save(&self, engine: &Arc<Engine>, up_to_seq: u64) -> Result<()> {
        self.save_inner(engine, up_to_seq, false, StagingSelection::CurrentIfDurable)
            .map(|_| ())
    }

    /// Return the durable cut selected by this save. A caller may have sampled
    /// its watermark before an in-flight apply record finished.
    pub fn save_with_sequence(&self, engine: &Arc<Engine>, up_to_seq: u64) -> Result<u64> {
        let name = self.save_inner(engine, up_to_seq, false, StagingSelection::CurrentIfDurable)?;
        parse_revision_name(name.as_str())
            .map(|(sequence, _)| sequence)
            .or_else(|| parse_legacy_name(name.as_str()))
            .ok_or_else(|| anyhow!("saved checkpoint has an invalid generation name"))
    }

    /// Save with one trace-only scheduler label. The label does not affect
    /// checkpoint selection, bytes, locking, publication, or acknowledgement.
    pub(crate) fn save_with_sequence_diagnostic(
        &self,
        engine: &Arc<Engine>,
        up_to_seq: u64,
        origin: &'static str,
        attempt_id: Option<u64>,
    ) -> Result<u64> {
        let trace_context = checkpoint_diagnostic_enabled()
            .then(|| CheckpointDiagnosticContext::new(origin, attempt_id));
        self.save_with_sequence_diagnostic_context(engine, up_to_seq, trace_context)
    }

    pub(crate) fn save_with_sequence_diagnostic_context(
        &self,
        engine: &Arc<Engine>,
        up_to_seq: u64,
        trace_context: Option<CheckpointDiagnosticContext>,
    ) -> Result<u64> {
        let name = self.save_inner_traced(
            engine,
            up_to_seq,
            false,
            StagingSelection::CurrentIfDurable,
            trace_context,
        )?;
        parse_revision_name(name.as_str())
            .map(|(sequence, _)| sequence)
            .or_else(|| parse_legacy_name(name.as_str()))
            .ok_or_else(|| anyhow!("saved checkpoint has an invalid generation name"))
    }

    /// Save a generation for a restore operation.
    ///
    /// Unlike [`Self::save`], this never silently ignores a stale sequence and
    /// always creates a new revision, including when the sequence is unchanged.
    pub fn save_required(&self, engine: &Arc<Engine>, up_to_seq: u64) -> Result<GenerationName> {
        self.save_inner(engine, up_to_seq, true, StagingSelection::Generic)
    }

    fn save_inner(
        &self,
        engine: &Arc<Engine>,
        up_to_seq: u64,
        required: bool,
        selection: StagingSelection,
    ) -> Result<GenerationName> {
        self.save_inner_traced(engine, up_to_seq, required, selection, None)
    }

    fn save_inner_traced(
        &self,
        engine: &Arc<Engine>,
        up_to_seq: u64,
        required: bool,
        selection: StagingSelection,
        trace_context: Option<CheckpointDiagnosticContext>,
    ) -> Result<GenerationName> {
        let (permit, gate_trace) = if trace_context.is_some() {
            let (permit, trace) = SaveGateTrace::acquire(&self.save_gate);
            trace_save_gate_acquired(trace_context, trace);
            (permit, Some(trace))
        } else {
            (self.save_gate.lock_owned(), None)
        };
        self.save_inner_permitted_selected(
            engine,
            up_to_seq,
            required,
            permit,
            SaveIntent::Ordinary,
            None,
            selection,
            trace_context,
            gate_trace,
        )
    }

    pub(super) fn save_inner_permitted(
        &self,
        engine: &Arc<Engine>,
        up_to_seq: u64,
        required: bool,
        permit: save_gate::SavePermit,
        intent: SaveIntent,
        archive_pin: Option<&mut Option<SegmentArchivePin>>,
    ) -> Result<GenerationName> {
        self.save_inner_permitted_selected(
            engine,
            up_to_seq,
            required,
            permit,
            intent,
            archive_pin,
            StagingSelection::Generic,
            None,
            None,
        )
    }

    fn save_inner_permitted_selected(
        &self,
        engine: &Arc<Engine>,
        up_to_seq: u64,
        required: bool,
        permit: save_gate::SavePermit,
        intent: SaveIntent,
        mut archive_pin: Option<&mut Option<SegmentArchivePin>>,
        selection: StagingSelection,
        trace_context: Option<CheckpointDiagnosticContext>,
        gate_trace: Option<SaveGateTrace>,
    ) -> Result<GenerationName> {
        let mut capacity_deadline = None;
        let mut permit = Some(permit);
        let mut gate_trace = gate_trace;
        let mut idle_revision = None;
        let mut checkpoint_pass = 1u8;
        loop {
            let guard = permit
                .take()
                .expect("each checkpoint attempt must own the save permit");
            let attempt_gate_trace = gate_trace.take();
            match self.save_inner_permitted_attempt(
                engine,
                up_to_seq,
                required,
                guard,
                intent,
                archive_pin.as_deref_mut(),
                idle_revision,
                capacity_deadline,
                selection,
                trace_context,
                attempt_gate_trace,
                checkpoint_pass,
            )? {
                SaveAttempt::Complete(name) => return Ok(name),
                SaveAttempt::FreshCapture => {
                    // The pending cut was published. Its fresh successor can
                    // request work, while retaining any existing wait deadline.
                    idle_revision = None;
                    checkpoint_pass = 2;
                    if trace_context.is_some() {
                        let (next_permit, next_trace) = SaveGateTrace::acquire(&self.save_gate);
                        trace_save_gate_acquired(trace_context, next_trace);
                        permit = Some(next_permit);
                        gate_trace = Some(next_trace);
                    } else {
                        permit = Some(self.save_gate.lock_owned());
                    }
                }
                SaveAttempt::CapacityWait(revision) => {
                    let deadline = *capacity_deadline
                        .get_or_insert_with(|| Instant::now() + Duration::from_secs(60));
                    let capacity_wait_started = trace_context.map(|_| Instant::now());
                    trace_capacity_wait_begin(trace_context, revision);
                    let capacity_wait = self
                        .background
                        .wait_for_capacity_progress_after(revision, deadline);
                    trace_capacity_wait_end(
                        trace_context,
                        revision,
                        capacity_wait_started.map(|started| duration_ns(started.elapsed())),
                        capacity_wait.as_ref(),
                    );
                    match capacity_wait? {
                        background::CapacityWait::Published => idle_revision = None,
                        background::CapacityWait::Idle => idle_revision = Some(revision),
                    }
                    if trace_context.is_some() {
                        let (next_permit, next_trace) = SaveGateTrace::acquire(&self.save_gate);
                        trace_save_gate_acquired(trace_context, next_trace);
                        permit = Some(next_permit);
                        gate_trace = Some(next_trace);
                    } else {
                        permit = Some(self.save_gate.lock_owned());
                    }
                }
            }
        }
    }

    fn save_inner_permitted_attempt(
        &self,
        engine: &Arc<Engine>,
        up_to_seq: u64,
        required: bool,
        _guard: save_gate::SavePermit,
        intent: SaveIntent,
        archive_pin: Option<&mut Option<SegmentArchivePin>>,
        idle_revision: Option<u64>,
        capacity_deadline: Option<Instant>,
        selection: StagingSelection,
        trace_context: Option<CheckpointDiagnosticContext>,
        gate_trace: Option<SaveGateTrace>,
        checkpoint_pass: u8,
    ) -> Result<SaveAttempt> {
        let requested_sequence = up_to_seq;
        let started = std::time::Instant::now();
        let capture_hold_ns = std::sync::atomic::AtomicU64::new(0);
        self.inventory_root()?;
        self.sweep_abandoned_staging()?;

        let current = self.current_record()?;
        let prior_manifest = current
            .as_ref()
            .filter(|record| !record.legacy)
            .map(|record| read_generation_manifest(&record.path))
            .transpose()?;
        if let Some((record, manifest)) = current.as_ref().zip(prior_manifest.as_ref()) {
            self.verify_predecessor_catalog(record, manifest)?;
        }
        let floor = prior_manifest
            .as_ref()
            .map_or(1, |manifest| manifest.next_collection_generation);
        let predecessor = PendingPredecessor::from_current(current.as_ref());
        let (mut pending, capture_stamp, up_to_seq, retried_pending, frozen_cut_bytes) =
            if let Some(pending) = self
                .take_matching_pending(engine, &predecessor)
                .with_context(|| "take matching pending checkpoint".to_owned())?
            {
                let stamp = pending.pending().stamp;
                let sequence = pending.pending().sequence;
                (pending, stamp, sequence, true, 0)
            } else {
                if intent == SaveIntent::ExactRaft {
                    bail!("exact Raft checkpoint lost its captured epoch; cannot recapture");
                }
                let capture_lease = engine
                    .capture_barrier
                    .capture(up_to_seq)
                    .map_err(|error| anyhow!(error))?;
                let capture_lease = MeasuredCapture::new(capture_lease, &capture_hold_ns);
                // Check while apply is excluded, before taking any journal ownership.
                // The worker cannot advance a predecessor while frozen work waits.
                if let Some(manifest) = &prior_manifest {
                    if self.needs_delta_capacity(
                        engine,
                        manifest,
                        &current.as_ref().expect("manifest has CURRENT").path,
                    )? {
                        let revision = self.request_merge_for_capacity_retry(
                            engine,
                            idle_revision,
                            capacity_deadline,
                        )?;
                        drop(capture_lease);
                        drop(_guard);
                        return Ok(SaveAttempt::CapacityWait(revision));
                    }
                }
                let capture_stamp = capture_lease.stamp();
                // Callers may label prepared/imported snapshots with an explicit cut.
                // Never label live data below a record completed while capture waited.
                let sequence = up_to_seq.max(capture_stamp.sequence);
                if let Some(current) = &current {
                    if sequence < current.sequence && intent != SaveIntent::RaftRestore {
                        if required {
                            bail!(
                                "required segment generation sequence {sequence} is below CURRENT sequence {}",
                                current.sequence
                            );
                        }
                        return Ok(SaveAttempt::Complete(current.name.clone()));
                    }
                }
                engine.prepare_checkpoint_namespace(&self.root, floor)?;
                let frozen_before = trace_context
                    .and_then(|_| engine.capacity_owner_state().map(|state| state.frozen));
                let frozen = engine.freeze_checkpoint_collections(
                    current.as_ref().map(|record| record.path.as_path()),
                )?;
                let frozen_cut_bytes = frozen_before
                    .zip(
                        trace_context
                            .and_then(|_| engine.capacity_owner_state().map(|state| state.frozen)),
                    )
                    .map_or(0, |(before, after)| after.saturating_sub(before));
                let layer_window = engine.layer_maintenance.freeze();
                drop(capture_lease);
                (
                    PendingFrozenLease {
                        slot: self.pending_frozen.clone(),
                        pending: Some(PendingFrozenCheckpoint {
                            engine: Arc::downgrade(engine),
                            stamp: capture_stamp,
                            sequence,
                            predecessor,
                            frozen,
                            detached_capture_ns: 0,
                            _layer_window: layer_window,
                        }),
                    },
                    capture_stamp,
                    sequence,
                    false,
                    u64::try_from(frozen_cut_bytes).unwrap_or(u64::MAX),
                )
            };
        if let Some(context) = trace_context {
            context.trace_freeze_completed(
                checkpoint_pass,
                retried_pending,
                frozen_cut_bytes,
            );
        }
        capture_hold_ns.fetch_add(
            pending.pending().detached_capture_ns,
            std::sync::atomic::Ordering::Relaxed,
        );
        if let Some(current) = &current {
            if up_to_seq < current.sequence && intent != SaveIntent::RaftRestore {
                if required {
                    bail!(
                        "required segment generation sequence {up_to_seq} is below CURRENT sequence {}",
                        current.sequence
                    );
                }
                return Ok(SaveAttempt::Complete(current.name.clone()));
            }
        }
        let staging_selection = if current.as_ref().is_some_and(|record| !record.legacy) {
            selection
        } else {
            StagingSelection::Generic
        };
        let (revision, mut staged) =
            self.begin_next_generation_selected(up_to_seq, staging_selection)?;
        let staging_path = staged.path().to_path_buf();
        // A v3 predecessor stores payloads under `payload/`, while the
        // capture writer consumes collection-directory paths.  Materialize
        // temporary hard-link aliases only for this migration input.  The
        // aliases are removed before the v2 generation is validated and
        // published, so the new writer remains collection-directory based.
        let compatibility_tree = current
            .as_ref()
            .zip(prior_manifest.as_ref())
            .filter(|(_, manifest)| manifest.schema_version == GENERATION_MANIFEST_V3)
            .map(|(record, manifest)| materialize_flat_reopen_tree(&record.path, manifest))
            .transpose()?;
        let frozen_result = pending.pending().frozen.write(&staging_path, up_to_seq);
        if let Some(tree) = compatibility_tree {
            for path in tree {
                let _ = std::fs::remove_dir_all(path);
            }
        }
        let mut capture = match frozen_result {
            Ok(capture) => capture,
            Err(error) => {
                let _ = std::fs::remove_dir_all(&staging_path);
                return Err(error).context("write frozen checkpoint collections");
            }
        };

        let previous = current
            .as_ref()
            .filter(|record| record.sequence <= up_to_seq)
            .map(|record| record.name.clone());
        let mut collections = match catalog_collections(&staging_path, up_to_seq) {
            Ok(collections) => collections,
            Err(error) => {
                let _ = std::fs::remove_dir_all(&staging_path);
                return Err(error).context("catalog staged segment checkpoint");
            }
        };
        for collection in &mut collections {
            let identity = capture
                .collections
                .get(&collection.collection_id)
                .ok_or_else(|| anyhow!("staged collection missing capture identity"))?;
            collection.collection_generation = identity.generation;
            collection.data_version = identity.data_version;
            if capture.reused.contains(&collection.collection_id) {
                if let Some(prior) = prior_manifest.as_ref().and_then(|manifest| {
                    manifest
                        .collections
                        .iter()
                        .find(|old| old.collection_id == collection.collection_id)
                }) {
                    if prior.collection_generation != identity.generation
                        || prior.schema_version != identity.schema_version
                        || prior.schema != collection.schema
                    {
                        bail!("reused collection catalog identity changed");
                    }
                    if prior
                        .segments
                        .iter()
                        .any(|segment| segment.payload_sha256.is_none())
                    {
                        bail!("v2 predecessor is missing payload checksum");
                    }
                    collection.segments = prior.segments.clone();
                }
            }
            for segment in &mut collection.segments {
                if matches!(segment.kind, SegmentKind::Base) && segment.payload_sha256.is_none() {
                    segment.payload_sha256 =
                        Some(base_payload_sha256(&staging_path.join(&segment.path))?);
                }
            }
            if let Some(fields) = capture.field_deltas.get(&collection.collection_id) {
                write_field_deltas(&staging_path, up_to_seq, collection, fields).with_context(
                    || {
                        format!(
                            "write v2 field deltas collection={} staging={}",
                            collection.collection_id,
                            staging_path.display()
                        )
                    },
                )?;
            }
        }
        // Open fresh layers before compaction replaces the last input's path.
        // The live index must first acknowledge the exact captured delta, then
        // replace its immutable inputs with the compacted view.
        prepare_live_delta_readers(&staging_path, &collections, &mut capture)?;
        // Construct and validate every scalar catalog replacement while the
        // staged files are still private. CURRENT must never expose a catalog
        // whose live view has not retained its post-capture private suffix.
        engine.prepare_scalar_checkpoint_publications(&mut capture)?;
        let checkpoint_payload_bytes = telemetry::new_file_bytes(
            &staging_path,
            current.as_ref().map(|record| record.path.as_path()),
        )?;
        let manifest = SegmentGenerationManifest {
            schema_version: GENERATION_MANIFEST_SCHEMA_VERSION,
            checkpoint_sequence: up_to_seq,
            revision,
            previous: previous.as_ref().map(|name| name.as_str().to_owned()),
            next_collection_generation: capture.next_generation,
            collections,
        };
        if let Err(error) = write_generation_manifest(&staging_path, &manifest) {
            let _ = std::fs::remove_dir_all(&staging_path);
            return Err(error);
        }
        let written_bytes = checkpoint_payload_bytes
            .checked_add(telemetry::manifest_bytes(&staging_path)?)
            .ok_or_else(|| anyhow!("checkpoint byte count overflow"))?;

        let staged_record = GenerationRecord {
            name: staged.generation().clone(),
            path: staging_path.clone(),
            sequence: up_to_seq,
            revision,
            legacy: false,
            previous,
        };
        // Validate the catalog and physical segment envelopes. Reopening an
        // Engine here would reconstruct every interner and HNSW graph merely
        // to publish unchanged hard links. Actual cold-open validation belongs
        // to the recovery path, before it installs data into a caller engine.
        let inherited = current
            .as_ref()
            .zip(prior_manifest.as_ref())
            .map(|(record, manifest)| (record.path.as_path(), manifest));
        if let Err(error) = validate_generation_layout_with_prior(&staged_record, inherited) {
            let _ = std::fs::remove_dir_all(&staging_path);
            return Err(error).context("validate staged segment generation");
        }
        let (pending_bytes, pending_layers) =
            telemetry::pending_deltas(&staging_path, &manifest.collections)?;
        if let GenerationStaging::Current(current_stage) = &mut staged {
            register_checkpoint_inherited_files(
                current_stage,
                &manifest.collections,
                prior_manifest.as_ref(),
                &capture,
            )?;
        }
        // Retain owner exclusion through durable publication AND live binding.
        // A replacement owner never observes a half-installed catalog.
        let publish_started = trace_context.map(|_| Instant::now());
        let mut owner_publication = None;
        let commit = staged.commit_with_publication_guard(&self.generations, || {
            owner_publication = self
                .publication_fence
                .as_ref()
                .map(|fence| fence.acquire())
                .transpose()
                .map_err(std::io::Error::other)?;
            let publication = engine
                .capture_barrier
                .capture(up_to_seq)
                .map_err(std::io::Error::other)?;
            let publication = MeasuredCapture::new(publication, &capture_hold_ns);
            let pin = publication
                .publication_pin(capture_stamp)
                .map_err(std::io::Error::other)?;
            drop(publication);
            Ok(pin)
        });
        if let Err(error) = commit {
            if error.class() == storage_durable::CommitFailureClass::CommitUncertain {
                engine.capture_barrier.apply().mark_uncertain();
                // A durable pointer may have advanced. Keep the actual payload
                // for restart diagnosis; the uncertainty latch makes every
                // in-process retry refuse it rather than reporting success.
            }
            return Err(anyhow::Error::new(error)).with_context(|| {
                format!("activate segment generation seq {up_to_seq} revision {revision}")
            });
        }
        if let Some(context) = trace_context {
            context.trace_publish_completed(checkpoint_pass);
        }
        let publish_ns = publish_started.map(|started| duration_ns(started.elapsed()));
        *self
            .verified_catalog
            .lock()
            .unwrap_or_else(|p| p.into_inner()) =
            Some((staged_record.name.clone(), serde_json::to_vec(&manifest)?));
        // Publication is durable. Bind only collections whose captured tuple
        // still matches; concurrent mutations retain their dirty state.
        let acknowledge_started = trace_context.map(|_| Instant::now());
        let binding = engine
            .capture_barrier
            .capture(up_to_seq)
            .map_err(anyhow::Error::msg)?;
        let binding = MeasuredCapture::new(binding, &capture_hold_ns);
        binding
            .validate_publish(capture_stamp)
            .map_err(anyhow::Error::msg)?;
        self.retain_root_for(engine);
        engine
            .bind_checkpoint_origins(&self.root.join(staged_record.name.as_str()), &mut capture)
            .with_context(|| {
                format!(
                    "bind v2 checkpoint origins generation={}",
                    self.root.join(staged_record.name.as_str()).display()
                )
            })?;
        engine.acknowledge_record_charges(&capture)?;
        drop(binding);
        let acknowledge_ns = acknowledge_started.map(|started| duration_ns(started.elapsed()));
        engine.metrics().observe_segment_checkpoint(
            written_bytes,
            started.elapsed(),
            std::time::Duration::from_nanos(
                capture_hold_ns.load(std::sync::atomic::Ordering::Relaxed),
            ),
        );
        engine
            .metrics()
            .set_segment_pending_delta(pending_bytes, pending_layers);
        match self.disk_bytes() {
            Ok(bytes) => engine.metrics().set_segment_disk_bytes(bytes),
            Err(error) => {
                tracing::warn!(%error, "segment disk metric unavailable after durable checkpoint")
            }
        }
        let name = staged_record.name.clone();
        if let Some(slot) = archive_pin {
            *slot = Some(self.pin_published_generation(&name)?);
        }
        let follow_with_fresh_capture = intent == SaveIntent::Ordinary
            && retried_pending
            && (required || requested_sequence > up_to_seq);
        pending.disarm();
        drop(owner_publication);
        drop(_guard);
        if background::needs_merge(&manifest) {
            if let Err(error) = self.request_merge(engine) {
                tracing::warn!(%error, "could not request segment merge after durable checkpoint");
            }
        }
        trace_durable_save_end(
            trace_context,
            up_to_seq,
            revision,
            duration_ns(started.elapsed()),
        );
        if let Some(trace_context) = trace_context {
            let gate_trace = gate_trace.expect("diagnostic checkpoints time the save gate");
            let save_gate_hold_ns = duration_ns(gate_trace.acquired_at.elapsed());
            let capacity_request_revision = engine
                .capacity_owner_state()
                .and_then(|state| state.checkpoint_request_revision);
            let root_merge = self.background.trace_state();
            tracing::info!(
                event = "segment_checkpoint_diagnostic",
                checkpoint_origin = trace_context.origin,
                checkpoint_sequence = up_to_seq,
                checkpoint_revision = revision,
                checkpoint_pass,
                frozen_cut_bytes,
                frozen_cut_reused = retried_pending,
                // The root gate is deliberately acquired before the capture
                // barrier. Keep this explicit zero in the trace so operators
                // do not infer that a frozen cut waits behind another save.
                capture_to_save_gate_wait_ns = 0u64,
                save_gate_wait_ns = gate_trace.wait_ns,
                save_gate_hold_ns,
                publish_ns = publish_ns.expect("diagnostic checkpoints time publication"),
                acknowledge_ns =
                    acknowledge_ns.expect("diagnostic checkpoints time acknowledgement"),
                capacity_request_pending = capacity_request_revision.is_some(),
                capacity_request_revision = capacity_request_revision.unwrap_or_default(),
                root_merge_queued = root_merge.queued,
                root_merge_running = root_merge.running,
                root_merge_requested = root_merge.requested,
                root_merge_published_revision = root_merge.published_revision,
                "segment checkpoint diagnostic"
            );
        }
        if follow_with_fresh_capture {
            return Ok(SaveAttempt::FreshCapture);
        }
        Ok(SaveAttempt::Complete(name))
    }

    /// Allocated generation bytes, counted once per file identity across hard links.
    pub fn disk_bytes(&self) -> Result<u64> {
        telemetry::generation_disk_bytes(&self.root)
    }

    /// Optional acceleration only. The caller holds the mutation fence; the
    /// save gate keeps physical publication and cache IO serialized.
    pub(crate) fn save_hnsw_graph_caches(&self, engine: &Engine) -> Result<usize> {
        let _guard = self.save_gate.lock_owned();
        engine.save_hnsw_graph_caches(&self.root.join(HNSW_GRAPH_CACHE_DIR))
    }

    pub(crate) fn has_hnsw_graph_cache(&self) -> bool {
        std::fs::symlink_metadata(self.root.join(HNSW_GRAPH_CACHE_DIR))
            .is_ok_and(|metadata| metadata.is_dir() && !metadata.file_type().is_symlink())
    }

    /// Finish standalone recovery only after the authoritative AOF tail has
    /// been applied. The optional graph must match those final vector contents.
    #[doc(hidden)]
    pub fn finish_aof_graph_restore(&self, engine: &Engine) -> Result<()> {
        let _guard = self.save_gate.lock_owned();
        let started = self.recovery_profile.enabled().then(Instant::now);
        self.recovery_profile
            .phase_start(RecoveryPhase::CheckpointHnswGraph, || {
                engine.finish_checkpoint_vectors_with_graph_cache(Some(
                    &self.root.join(HNSW_GRAPH_CACHE_DIR),
                ))
            })?;
        if let Some(started) = started {
            self.add_recovery_timing(|timings| {
                timings.vector_finish_ms = started.elapsed().as_millis() as u64;
            });
        }
        self.emit_recovery_profile();
        Ok(())
    }

    fn verify_predecessor_catalog(
        &self,
        record: &GenerationRecord,
        manifest: &SegmentGenerationManifest,
    ) -> Result<()> {
        let encoded = serde_json::to_vec(manifest)?;
        let mut verified = self
            .verified_catalog
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        if let Some((name, bytes)) = verified.as_ref() {
            if name == &record.name {
                if bytes != &encoded {
                    bail!("CURRENT manifest changed after verification");
                }
                return Ok(());
            }
        }
        // This runs under the store serialization lock, before CaptureBarrier.
        // It validates files directly and never builds another Engine.
        validate_generation_layout(record).context("verify predecessor generation")?;
        if serde_json::to_vec(&read_generation_manifest(&record.path)?)? != encoded {
            bail!("CURRENT manifest changed during verification");
        }
        *verified = Some((record.name.clone(), encoded));
        Ok(())
    }

    fn take_matching_pending(
        &self,
        engine: &Arc<Engine>,
        predecessor: &PendingPredecessor,
    ) -> Result<Option<PendingFrozenLease>> {
        let mut slot = self
            .pending_frozen
            .lock()
            .map_err(|_| anyhow!("pending frozen checkpoint lock poisoned"))?;
        let Some(pending) = slot.take() else {
            return Ok(None);
        };
        let Some(owner) = pending.engine.upgrade() else {
            return Ok(None);
        };
        if owner.capture_barrier.is_uncertain() {
            *slot = Some(pending);
            bail!("checkpoint retry refused: durability is uncertain; restart required");
        }
        if !Arc::ptr_eq(&owner, engine) {
            if owner.capture_barrier.epoch() != pending.stamp.epoch {
                // A restore candidate replaced the old owner's epoch. Its
                // detached cut is stale, so release it and let the candidate
                // capture its own state.
                return Ok(None);
            }
            *slot = Some(pending);
            bail!("checkpoint root has pending frozen work for another Engine");
        }
        if owner.capture_barrier.epoch() != pending.stamp.epoch {
            return Ok(None);
        }
        if &pending.predecessor != predecessor {
            // Another durable generation became CURRENT. This cut cannot be
            // rebased without risking a downgrade, so discard only this stale
            // detached work and let this caller capture from the new CURRENT.
            return Ok(None);
        }
        let mut lease = PendingFrozenLease {
            slot: self.pending_frozen.clone(),
            pending: Some(pending),
        };
        drop(slot);
        let validation = engine
            .capture_barrier
            .capture(lease.pending().sequence)
            .map_err(anyhow::Error::msg)?;
        if validation.validate_publish(lease.pending().stamp).is_err() {
            // Restore replaced the Engine epoch. The actual old payload cannot
            // publish into replacement state and is intentionally released.
            lease.disarm();
            return Ok(None);
        }
        drop(validation);
        Ok(Some(lease))
    }

    #[cfg(test)]
    fn pending_frozen_identity(&self) -> Option<usize> {
        self.pending_frozen
            .lock()
            .ok()?
            .as_ref()
            .map(|pending| std::ptr::from_ref(&pending.frozen) as usize)
    }

    /// Reopen the exact active checkpoint into a fresh engine.
    pub fn load_latest(&self) -> Result<Option<(Arc<Engine>, u64)>> {
        let engine = Arc::new(Engine::new());
        match self.reopen_into(&engine)? {
            Some(seq) => Ok(Some((engine, seq))),
            None => Ok(None),
        }
    }

    /// Load exactly the generation named by `CURRENT`.
    ///
    /// This method never performs legacy adoption and never searches for a
    /// higher, unpointed generation.
    pub fn load_current_generation(&self) -> Result<Option<LoadedSegmentGeneration>> {
        let _guard = self.save_gate.lock_owned();
        let CurrentTarget::Generation(name) = self
            .generations
            .read_current()
            .map_err(|error| anyhow::Error::new(error).context("read CURRENT"))?
        else {
            return Ok(None);
        };
        let record = self.record_for_name(name.clone())?;
        let engine = Arc::new(Engine::new());
        self.reopen_record(&engine, &record)?;
        Ok(Some(LoadedSegmentGeneration {
            name,
            sequence: record.sequence,
            engine,
        }))
    }

    /// Reopen the cold-start checkpoint and return the decision that selected
    /// it. A missing `CURRENT` can adopt only an exact 0.4.28 generation that
    /// the open-time inventory already accepted. It never selects an unpointed
    /// revision or falls back from a corrupt highest legacy generation.
    pub fn reopen_into_with_outcome(&self, engine: &Arc<Engine>) -> Result<SegmentStartupOutcome> {
        self.reopen_into_with_graph_policy(engine, false)
    }

    /// Standalone startup only. A true second result requires calling
    /// `finish_aof_graph_restore` after AOF replay and before serving queries.
    /// Ordinary checkpoint readers and Raft keep the eager restore path.
    #[doc(hidden)]
    pub fn reopen_for_aof_replay(
        &self,
        engine: &Arc<Engine>,
    ) -> Result<(SegmentStartupOutcome, bool)> {
        let deferred = self.has_hnsw_graph_cache();
        Ok((
            self.reopen_into_with_graph_policy(engine, deferred)?,
            deferred,
        ))
    }

    fn reopen_into_with_graph_policy(
        &self,
        engine: &Arc<Engine>,
        defer_until_aof: bool,
    ) -> Result<SegmentStartupOutcome> {
        let _guard = self.save_gate.lock_owned();
        match self.generations.read_current() {
            Ok(CurrentTarget::Empty) => match self.bootstrap {
                StartupBootstrap::InitializedEmpty {
                    recovered_uncommitted,
                    staging_cleaned,
                } => Ok(SegmentStartupOutcome {
                    decision: if recovered_uncommitted {
                        SegmentStartupDecision::RecoveredUncommittedEmpty
                    } else {
                        SegmentStartupDecision::InitializedEmptyRoot
                    },
                    checkpoint_sequence: None,
                    generation: None,
                    recovered_legacy_aside: false,
                    staging_cleaned,
                }),
                StartupBootstrap::ExistingCurrent { staging_cleaned } => {
                    Ok(SegmentStartupOutcome {
                        decision: SegmentStartupDecision::RestoredCurrentEmpty,
                        checkpoint_sequence: None,
                        generation: None,
                        recovered_legacy_aside: false,
                        staging_cleaned,
                    })
                }
                StartupBootstrap::Legacy { .. } => {
                    bail!("CURRENT became empty before legacy segment generation adoption")
                }
            },
            Ok(CurrentTarget::Generation(name)) => {
                let record = self.record_for_name(name)?;
                let seq = self.reopen_record_with_graph_policy(engine, &record, defer_until_aof)?;
                let staging_cleaned = match self.bootstrap {
                    StartupBootstrap::ExistingCurrent { staging_cleaned } => staging_cleaned,
                    StartupBootstrap::InitializedEmpty {
                        staging_cleaned, ..
                    }
                    | StartupBootstrap::Legacy {
                        staging_cleaned, ..
                    } => staging_cleaned,
                };
                Ok(SegmentStartupOutcome {
                    decision: SegmentStartupDecision::RestoredCurrentGeneration,
                    checkpoint_sequence: Some(seq),
                    generation: Some(record.name),
                    recovered_legacy_aside: false,
                    staging_cleaned,
                })
            }
            Err(error) if error.kind == CurrentReadErrorKind::Missing => {
                let StartupBootstrap::Legacy {
                    recovered_legacy_aside,
                    staging_cleaned,
                } = self.bootstrap
                else {
                    bail!("CURRENT disappeared after segment root initialization");
                };
                let inventory = self.inventory_root()?;
                if let Some(name) = inventory.revision_generations.first() {
                    bail!(
                        "CURRENT is missing but root contains unpointed revision generation `{name}`; refusing to select or initialize it"
                    );
                }
                let Some(record) = self.legacy_records()?.into_iter().next_back() else {
                    bail!("CURRENT is missing and no exact 0.4.28 generation can be adopted");
                };
                let seq = self.reopen_record_with_graph_policy(engine, &record, defer_until_aof)?;
                self.generations
                    .adopt_legacy(record.name.clone())
                    .map_err(anyhow::Error::new)
                    .with_context(|| format!("adopt legacy segment generation {}", record.name))?;
                Ok(SegmentStartupOutcome {
                    decision: SegmentStartupDecision::AdoptedLegacy0428,
                    checkpoint_sequence: Some(seq),
                    generation: Some(record.name),
                    recovered_legacy_aside,
                    staging_cleaned,
                })
            }
            Err(error) => Err(anyhow::Error::new(error).context("read CURRENT")),
        }
    }

    /// Reopen the cold-start checkpoint without exposing the startup decision.
    pub fn reopen_into(&self, engine: &Arc<Engine>) -> Result<Option<u64>> {
        Ok(self.reopen_into_with_outcome(engine)?.checkpoint_sequence)
    }

    /// Retain the active generation and up to `keep - 1` prior generations.
    ///
    /// `keep=0` still retains the active generation. Complete revisions newer
    /// than `CURRENT` are failed pre-commit attempts and are removed. The method
    /// never removes the directory named by `CURRENT`. If a prior prune already
    /// removed the predecessor named by an immutable manifest, this call keeps
    /// any unlinked older directories whose lineage it can no longer prove.
    pub fn prune(&self, keep: usize) -> Result<usize> {
        let _guard = self.save_gate.lock_owned();
        self.inventory_root()?;
        self.sweep_abandoned_staging()?;
        let (history, truncated) = self.active_history()?;
        let all = self.generation_entries()?;

        let retain: BTreeSet<_> = history
            .iter()
            .take(keep.max(1))
            .map(|record| record.name.as_str().to_owned())
            .collect();
        let active_chain: BTreeSet<_> = history
            .iter()
            .map(|record| record.name.as_str().to_owned())
            .collect();
        let current = history.first();

        let mut removed = 0usize;
        for (name, path) in all {
            if retain.contains(&name) {
                continue;
            }
            if self.background.protects(&name) {
                if active_chain.contains(&name) {
                    self.background.defer_reclaim(&name);
                }
                continue;
            }
            if truncated
                && !active_chain.contains(&name)
                && !self.background.known_retired(&name)
                && !current.is_some_and(|current| definitely_unpointed_after(&name, current))
            {
                continue;
            }
            std::fs::remove_dir_all(&path)
                .with_context(|| format!("remove segment generation {}", path.display()))?;
            self.background.reclaimed(&name);
            removed += 1;
        }
        if removed > 0 {
            sync_directory(&self.root).context("fsync checkpoint root after prune")?;
        }
        Ok(removed)
    }

    /// Activated predecessor-chain sequences, ascending and de-duplicated.
    pub fn generation_seqs(&self) -> Result<Vec<u64>> {
        Ok(self
            .active_history()?
            .0
            .into_iter()
            .map(|record| record.sequence)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect())
    }

    /// Inspect every direct child before any root mutation. The segment root is
    /// Lumen-owned, but a wrong mount or an older unsupported layout must still
    /// fail loudly instead of being converted into a fresh empty store.
    fn inventory_root(&self) -> Result<RootInventory> {
        let mut inventory = RootInventory::default();
        let mut children = Vec::new();
        let mut violations = Vec::new();
        let mut entries = std::fs::read_dir(&self.root)
            .with_context(|| format!("read checkpoint root {}", self.root.display()))?
            .collect::<std::io::Result<Vec<_>>>()?;
        entries.sort_by_key(|entry| entry.file_name());

        for entry in entries {
            let path = entry.path();
            let metadata = std::fs::symlink_metadata(&path)
                .with_context(|| format!("inspect checkpoint root entry {}", path.display()))?;
            let kind = root_entry_kind(&metadata);
            let raw = match entry.file_name().into_string() {
                Ok(raw) => raw,
                Err(name) => {
                    let display = format!("{name:?}");
                    children.push(format!("{display} ({kind})"));
                    violations.push(format!("checkpoint root has non-UTF-8 entry {display}"));
                    continue;
                }
            };
            children.push(format!("{raw} ({kind})"));
            // This optional cache cannot authorize empty initialization. With
            // CURRENT present, a malformed cache remains ignorable. Readers
            // and writers independently refuse cache symlinks.
            if raw == HNSW_GRAPH_CACHE_DIR {
                inventory.has_graph_cache = true;
                continue;
            }
            let regular_file = !metadata.file_type().is_symlink() && metadata.is_file();
            let real_directory = !metadata.file_type().is_symlink() && metadata.is_dir();

            if raw != CONTAINER_VOLUME_SEED_FILE && raw != EXT_FILESYSTEM_METADATA_DIR {
                inventory.non_seed_entries += 1;
            }

            if raw == EXT_FILESYSTEM_METADATA_DIR {
                if !real_directory {
                    violations.push(format!(
                        "{EXT_FILESYSTEM_METADATA_DIR} must be a real empty directory: {}",
                        path.display()
                    ));
                } else {
                    match std::fs::read_dir(&path) {
                        Ok(mut contents) => match contents.next() {
                            None => {}
                            Some(Ok(_)) => violations.push(format!(
                                "{EXT_FILESYSTEM_METADATA_DIR} must be empty: {}",
                                path.display()
                            )),
                            Some(Err(error)) => violations.push(format!(
                                "cannot verify {EXT_FILESYSTEM_METADATA_DIR} is empty at {}: {error}",
                                path.display()
                            )),
                        },
                        Err(error) => violations.push(format!(
                            "cannot inspect {EXT_FILESYSTEM_METADATA_DIR} at {}: {error}",
                            path.display()
                        )),
                    }
                }
                continue;
            }

            if matches!(
                raw.as_str(),
                CURRENT_FILE
                    | CURRENT_TEMP_FILE
                    | AOF_FILE
                    | AOF_COMPACT_TEMP_FILE
                    | CONTAINER_VOLUME_SEED_FILE
            ) {
                if !regular_file {
                    violations.push(format!(
                        "checkpoint root entry must be a regular file: {}",
                        path.display()
                    ));
                } else if raw == AOF_FILE {
                    inventory.has_aof_log = true;
                } else if raw == AOF_COMPACT_TEMP_FILE {
                    inventory.has_aof_compact_temp = true;
                }
                continue;
            }
            if parse_legacy_name(&raw).is_some() {
                if !real_directory {
                    violations.push(format!(
                        "legacy checkpoint must be a real directory: {}",
                        path.display()
                    ));
                }
                continue;
            }
            if parse_revision_name(&raw).is_some() {
                if !real_directory {
                    violations.push(format!(
                        "segment generation must be a real directory: {}",
                        path.display()
                    ));
                } else {
                    inventory.revision_generations.push(raw);
                }
                continue;
            }
            if is_legacy_aside_name(&raw) {
                if !real_directory {
                    violations.push(format!(
                        "legacy aside must be a real directory: {}",
                        path.display()
                    ));
                }
                continue;
            }
            if is_known_staging_name(&raw) {
                if !real_directory {
                    violations.push(format!(
                        "checkpoint staging must be a real directory: {}",
                        path.display()
                    ));
                }
                continue;
            }
            violations.push(format!(
                "unrecognized non-empty segment checkpoint root entry `{raw}` at {}",
                path.display()
            ));
        }

        if inventory.has_aof_compact_temp && !inventory.has_aof_log {
            violations.push(format!(
                "{AOF_COMPACT_TEMP_FILE} requires regular {AOF_FILE} beside it"
            ));
        }
        if !violations.is_empty() {
            bail!(
                "invalid segment checkpoint root inventory [{}]; refusing to initialize CURRENT: {}",
                children.join(", "),
                violations.join("; ")
            );
        }
        Ok(inventory)
    }

    /// Establish the one permitted missing-`CURRENT` state before a caller can
    /// save or reopen. This runs under `save_gate` and mutates only after the
    /// full direct-child inventory has accepted the root.
    fn prepare_startup_root(&self) -> Result<StartupBootstrap> {
        let inventory = self.inventory_root()?;
        match self.generations.read_current() {
            Ok(_) => Ok(StartupBootstrap::ExistingCurrent {
                staging_cleaned: self.sweep_abandoned_staging()?,
            }),
            Err(error) if error.kind == CurrentReadErrorKind::Missing => {
                if let Some(name) = inventory.revision_generations.first() {
                    bail!(
                        "CURRENT is missing but root contains unpointed revision generation `{name}`; refusing to select or initialize it"
                    );
                }
                if inventory.has_graph_cache {
                    bail!(
                        "CURRENT is missing beside an optional HNSW graph cache; refusing initialization or cleanup without durable authority"
                    );
                }
                let recovered_legacy_aside = self.reconcile_legacy_asides()?;
                let staging_cleaned = self.sweep_abandoned_staging()?;
                if !self.legacy_records()?.is_empty() {
                    return Ok(StartupBootstrap::Legacy {
                        recovered_legacy_aside,
                        staging_cleaned,
                    });
                }
                self.generations
                    .initialize_empty()
                    .map_err(anyhow::Error::new)
                    .context("initialize empty segment generation store")?;
                Ok(StartupBootstrap::InitializedEmpty {
                    recovered_uncommitted: inventory.non_seed_entries > 0,
                    staging_cleaned,
                })
            }
            Err(error) => Err(anyhow::Error::new(error).context("read CURRENT")),
        }
    }

    fn current_record(&self) -> Result<Option<GenerationRecord>> {
        match self.generations.read_current() {
            Ok(CurrentTarget::Empty) => Ok(None),
            Ok(CurrentTarget::Generation(name)) => self.record_for_name(name).map(Some),
            Err(error) => Err(anyhow::Error::new(error).context("read CURRENT")),
        }
    }

    /// Return the exact activated predecessor chain, newest first. A missing
    /// predecessor is an allowed retention boundary because prune never mutates
    /// an immutable manifest merely to truncate its link.
    fn active_history(&self) -> Result<(Vec<GenerationRecord>, bool)> {
        let Some(mut record) = self.current_record()? else {
            return Ok((Vec::new(), false));
        };
        let mut visited = BTreeSet::new();
        let mut history = Vec::new();
        let mut truncated = false;

        loop {
            let name = record.name.as_str().to_owned();
            if !visited.insert(name.clone()) {
                bail!("segment generation predecessor cycle at `{name}`");
            }
            let next = if let Some(previous) = &record.previous {
                if let Some(previous_record) = self.record_if_present(previous.clone())? {
                    if previous_record.sequence > record.sequence
                        || previous_record.order_key() >= record.order_key()
                    {
                        bail!(
                            "generation {} has non-predecessor link {}",
                            record.name,
                            previous
                        );
                    }
                    Some(previous_record)
                } else {
                    truncated = true;
                    None
                }
            } else if record.legacy {
                self.legacy_records()?
                    .into_iter()
                    .filter(|candidate| candidate.legacy && candidate.sequence < record.sequence)
                    .max_by_key(|candidate| candidate.sequence)
            } else {
                None
            };
            history.push(record);
            let Some(next) = next else {
                break;
            };
            record = next;
        }
        Ok((history, truncated))
    }

    fn begin_next_generation(&self, sequence: u64) -> Result<(u64, StagedGeneration)> {
        let (revision, staged) =
            self.begin_next_generation_selected(sequence, StagingSelection::Generic)?;
        match staged {
            GenerationStaging::Generic(staged) => Ok((revision, staged)),
            GenerationStaging::Current(_) => {
                unreachable!("generic staging selection returned current-derived stage")
            }
        }
    }

    pub(super) fn begin_background_merge_stage(&self, sequence: u64) -> Result<StagedGeneration> {
        let (_, staged) =
            self.begin_next_generation_selected(sequence, StagingSelection::BackgroundScratch)?;
        match staged {
            GenerationStaging::Generic(staged) => Ok(staged),
            GenerationStaging::Current(_) => {
                unreachable!("background scratch selection returned current-derived stage")
            }
        }
    }

    pub(super) fn begin_next_generation_selected(
        &self,
        sequence: u64,
        selection: StagingSelection,
    ) -> Result<(u64, GenerationStaging)> {
        let mut revision = match selection {
            // A background merge scratch tree is private and never becomes a
            // published generation. Keep it outside the published revision
            // sequence so the later atomic publication can use the next
            // revision without colliding with its own scratch directory.
            StagingSelection::BackgroundScratch => 0,
            StagingSelection::Generic | StagingSelection::CurrentIfDurable => self
                .generation_entries()?
                .into_iter()
                .filter_map(|(name, _)| parse_revision_name(&name).map(|(_, revision)| revision))
                .max()
                .unwrap_or(0)
                .checked_add(1)
                .ok_or_else(|| anyhow!("segment generation revision exhausted"))?,
        };

        loop {
            let name = GenerationName::parse(format!("gen-{sequence}-rev-{revision}"))
                .map_err(anyhow::Error::new)
                .context("build segment generation name")?;
            let staged = match selection {
                StagingSelection::Generic => self
                    .generations
                    .begin(name.clone())
                    .map(GenerationStaging::Generic),
                StagingSelection::CurrentIfDurable => {
                    #[cfg(unix)]
                    {
                        match self.generations.begin_from_current_if_durable(name.clone()) {
                            Ok(Some(staged)) => Ok(GenerationStaging::Current(staged)),
                            Ok(None) => self
                                .generations
                                .begin(name.clone())
                                .map(GenerationStaging::Generic),
                            Err(error) => Err(error),
                        }
                    }
                    #[cfg(not(unix))]
                    {
                        self.generations
                            .begin(name.clone())
                            .map(GenerationStaging::Generic)
                    }
                }
                StagingSelection::BackgroundScratch => self
                    .generations
                    .begin(name.clone())
                    .map(GenerationStaging::Generic),
            };
            match staged {
                Ok(staged) => return Ok((revision, staged)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    revision = revision
                        .checked_add(1)
                        .ok_or_else(|| anyhow!("segment generation revision exhausted"))?;
                }
                Err(error) => return Err(error).context("create segment generation staging"),
            }
        }
    }

    fn record_for_name(&self, name: GenerationName) -> Result<GenerationRecord> {
        let path = self.generations.generation_path(&name);
        let metadata = std::fs::symlink_metadata(&path)
            .with_context(|| format!("inspect segment generation {}", path.display()))?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            bail!(
                "segment generation must be a real directory: {}",
                path.display()
            );
        }
        if let Some(sequence) = parse_legacy_name(name.as_str()) {
            if path.join(GENERATION_MANIFEST_FILE).exists() {
                bail!(
                    "legacy generation {} unexpectedly contains {}",
                    name,
                    GENERATION_MANIFEST_FILE
                );
            }
            return Ok(GenerationRecord {
                name,
                path,
                sequence,
                revision: 0,
                legacy: true,
                previous: None,
            });
        }

        let (sequence, revision) = parse_revision_name(name.as_str())
            .ok_or_else(|| anyhow!("CURRENT names an unsupported generation `{name}`"))?;
        let manifest = read_generation_manifest(&path)?;
        if manifest.schema_version != 1
            && !matches!(
                manifest.schema_version,
                GENERATION_MANIFEST_V2 | GENERATION_MANIFEST_V3
            )
        {
            bail!(
                "generation {} has unsupported manifest schema {}",
                name,
                manifest.schema_version
            );
        }
        if manifest.checkpoint_sequence != sequence || manifest.revision != revision {
            bail!(
                "generation {} manifest does not match its directory name",
                name
            );
        }
        let previous = match manifest.previous {
            Some(raw) => {
                if !is_supported_generation_name(&raw) {
                    bail!("generation {name} has unsupported predecessor `{raw}`");
                }
                if !is_older_predecessor(&raw, sequence, revision) {
                    bail!("generation {name} has non-predecessor link `{raw}`");
                }
                Some(
                    GenerationName::parse(raw)
                        .map_err(anyhow::Error::new)
                        .context("parse previous segment generation")?,
                )
            }
            None => None,
        };
        if previous.as_ref() == Some(&name) {
            bail!("generation {name} points to itself as predecessor");
        }
        Ok(GenerationRecord {
            name,
            path,
            sequence,
            revision,
            legacy: false,
            previous,
        })
    }

    fn record_if_present(&self, name: GenerationName) -> Result<Option<GenerationRecord>> {
        let path = self.generations.generation_path(&name);
        match std::fs::symlink_metadata(&path) {
            Ok(_) => self.record_for_name(name).map(Some),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => {
                Err(error).with_context(|| format!("inspect segment generation {}", path.display()))
            }
        }
    }

    fn generation_entries(&self) -> Result<Vec<(String, PathBuf)>> {
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(&self.root)
            .with_context(|| format!("read checkpoint root {}", self.root.display()))?
        {
            let entry = entry?;
            let Some(raw) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if parse_legacy_name(&raw).is_none() && parse_revision_name(&raw).is_none() {
                continue;
            }
            let path = entry.path();
            let metadata = std::fs::symlink_metadata(&path)
                .with_context(|| format!("inspect segment generation {}", path.display()))?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                bail!(
                    "segment generation must be a real directory: {}",
                    path.display()
                );
            }
            entries.push((raw, path));
        }
        entries.sort_by(|left, right| left.0.cmp(&right.0));
        Ok(entries)
    }

    fn legacy_records(&self) -> Result<Vec<GenerationRecord>> {
        let mut records = Vec::new();
        for (raw, _) in self.generation_entries()? {
            if parse_legacy_name(&raw).is_none() {
                continue;
            }
            let name = GenerationName::parse(raw)
                .map_err(anyhow::Error::new)
                .context("parse legacy segment generation")?;
            records.push(self.record_for_name(name)?);
        }
        records.sort_by_key(|record| record.sequence);
        Ok(records)
    }

    fn reopen_record(&self, engine: &Arc<Engine>, record: &GenerationRecord) -> Result<u64> {
        self.reopen_record_with_graph_policy(engine, record, false)
    }

    fn reopen_record_with_graph_policy(
        &self,
        engine: &Arc<Engine>,
        record: &GenerationRecord,
        defer_until_aof: bool,
    ) -> Result<u64> {
        self.reset_recovery_profile();
        let layout_started = self.recovery_profile.enabled().then(Instant::now);
        let collections = self
            .recovery_profile
            .phase_start(RecoveryPhase::LayoutValidation, || {
                validate_generation_layout(record)
            })?;
        if let Some(started) = layout_started {
            self.add_recovery_timing(|timings| {
                timings.layout_validation_ms = started.elapsed().as_millis() as u64;
            });
        }
        let replacement = Engine::new();
        self.reopen_once_with_graph_policy(&replacement, record, collections, defer_until_aof)?;
        self.retain_root_for(&replacement);
        // Decode and build backends once, before touching the caller. Activation
        // is one apply interval and invalidates captures from its former epoch.
        self.background
            .pin_loaded_engine(record.name.as_str().to_owned(), &replacement)?;
        engine.activate_replacement(replacement)?;
        if !defer_until_aof {
            self.emit_recovery_profile();
        }
        Ok(record.sequence)
    }

    /// Validate a candidate without installing it, for legacy adoption.
    fn validate_record(&self, record: &GenerationRecord) -> Result<usize> {
        let collections = validate_generation_layout(record)?;
        let verifier = Engine::new();
        self.reopen_once(&verifier, record, collections)?;
        Ok(collections)
    }

    fn reopen_once(
        &self,
        engine: &Engine,
        record: &GenerationRecord,
        collections: usize,
    ) -> Result<()> {
        self.reopen_once_with_graph_policy(engine, record, collections, false)
    }

    fn reopen_once_with_graph_policy(
        &self,
        engine: &Engine,
        record: &GenerationRecord,
        collections: usize,
        defer_until_aof: bool,
    ) -> Result<()> {
        let manifest_started = self.recovery_profile.enabled().then(Instant::now);
        let manifest = if record.legacy {
            None
        } else {
            Some(
                self.recovery_profile
                    .phase_start(RecoveryPhase::ManifestDecode, || {
                        read_generation_manifest(&record.path)
                    })?,
            )
        };
        if let Some(started) = manifest_started {
            self.add_recovery_timing(|timings| {
                timings.manifest_decode_ms = started.elapsed().as_millis() as u64;
            });
        }
        let graph_cache = self.root.join(HNSW_GRAPH_CACHE_DIR);
        let graph_cache = std::fs::symlink_metadata(&graph_cache)
            .ok()
            .filter(|metadata| metadata.is_dir() && !metadata.file_type().is_symlink())
            .map(|_| graph_cache);
        let defer_hnsw = defer_until_aof
            || manifest.as_ref().is_some_and(|manifest| {
                (graph_cache.is_some()
                    && matches!(
                        manifest.schema_version,
                        GENERATION_MANIFEST_V2 | GENERATION_MANIFEST_V3
                    ))
                    || manifest.collections.iter().any(|collection| {
                        collection.segments.iter().any(|segment| {
                            (matches!(segment.kind, SegmentKind::Delta)
                                || segment.local_rows.is_some())
                                && segment
                                    .field
                                    .as_ref()
                                    .and_then(|field| collection.schema.get(field))
                                    .and_then(|spec| spec.get("type"))
                                    .and_then(serde_json::Value::as_str)
                                    == Some("vector")
                        })
                    })
            });
        let base_decode_started = self.recovery_profile.enabled().then(Instant::now);
        let mut mapped_bases = BTreeMap::<String, BTreeMap<String, Vec<String>>>::new();
        self.recovery_profile
            .phase_start(RecoveryPhase::BaseRowsDecode, || -> Result<()> {
                for collection in manifest.iter().flat_map(|manifest| &manifest.collections) {
                    for segment in &collection.segments {
                        if !matches!(segment.kind, SegmentKind::Base) {
                            continue;
                        }
                        if let Some(local) = &segment.local_rows {
                            let rows = crate::segment::decode_sparse_local_rows(
                                &record.path.join(&local.path),
                                local.count,
                            )?;
                            let ids = (0..local.count)
                                .map(|row| {
                                    rows.external_id(row)
                                        .map(str::to_owned)
                                        .ok_or_else(|| anyhow!("mapped base row is missing"))
                                })
                                .collect::<Result<_>>()?;
                            mapped_bases
                                .entry(collection.collection_id.clone())
                                .or_default()
                                .insert(
                                    segment
                                        .field
                                        .clone()
                                        .ok_or_else(|| anyhow!("mapped base has no field"))?,
                                    ids,
                                );
                        }
                    }
                }
                Ok(())
            })?;
        if let Some(started) = base_decode_started {
            self.add_recovery_timing(|timings| {
                timings.base_decode_ms = started.elapsed().as_millis() as u64;
            });
        }
        let compatibility_tree =
            self.recovery_profile
                .phase_start(RecoveryPhase::FlatCompatibilityTree, || {
                    manifest
                        .as_ref()
                        .filter(|manifest| manifest.schema_version == GENERATION_MANIFEST_V3)
                        .map(|manifest| materialize_flat_reopen_tree(&record.path, manifest))
                        .transpose()
                })?;
        let reopen_started = self.recovery_profile.enabled().then(Instant::now);
        let reopened = self
            .recovery_profile
            .phase_start(RecoveryPhase::EngineReopen, || {
                engine
                    .reopen_from_segment_dir_with_base_rows(
                        &record.path,
                        defer_hnsw,
                        &mapped_bases,
                        self.recovery_profile
                            .enabled()
                            .then_some(&self.recovery_profile),
                    )
                    .with_context(|| format!("reopen checkpoint {}", record.path.display()))
            });
        if let Some(tree) = compatibility_tree {
            for path in tree {
                let _ = std::fs::remove_dir_all(path);
            }
        }
        let reopened = reopened?;
        if let Some(manifest) = manifest.as_ref().filter(|m| {
            matches!(
                m.schema_version,
                GENERATION_MANIFEST_V2 | GENERATION_MANIFEST_V3
            )
        }) {
            let capture = crate::storage::CheckpointCapture {
                prepared: BTreeMap::new(),
                prepared_deltas: BTreeMap::new(),
                prepared_compactions: BTreeMap::new(),
                scalar_cuts: BTreeMap::new(),
                scalar_publications: BTreeMap::new(),
                scalar_retire: BTreeMap::new(),
                live_delta_inputs: BTreeMap::new(),
                live_base_inputs: BTreeMap::new(),
                collections: manifest
                    .collections
                    .iter()
                    .map(|c| {
                        (
                            c.collection_id.clone(),
                            crate::storage::CheckpointCollectionIdentity {
                                generation: c.collection_generation,
                                data_version: c.data_version,
                                schema_version: c.schema_version,
                            },
                        )
                    })
                    .collect(),
                next_generation: manifest.next_collection_generation,
                field_dirty: BTreeMap::new(),
                frozen_changes: BTreeMap::new(),
                reused: BTreeSet::new(),
                initial_sparse: BTreeSet::new(),
                field_deltas: Arc::new(BTreeMap::new()),
                record_cut: None,
            };
            let delta_decode_started = self.recovery_profile.enabled().then(Instant::now);
            self.recovery_profile.phase_start(
                RecoveryPhase::DeltaDecodeApply,
                || -> Result<()> {
                    for collection in &manifest.collections {
                        for segment in &collection.segments {
                            if matches!(segment.kind, SegmentKind::Delta) {
                                let local = segment
                                    .local_rows
                                    .as_ref()
                                    .ok_or_else(|| anyhow!("delta has no row map"))?;
                                let ids = crate::segment::decode_sparse_local_rows(
                                    &record.path.join(&local.path),
                                    local.count,
                                )?;
                                let reader =
                                    std::sync::Arc::new(crate::segment::SegmentReader::open(
                                        &record.path.join(&segment.path),
                                    )?);
                                let field = segment
                                    .field
                                    .as_deref()
                                    .ok_or_else(|| anyhow!("delta has no field"))?;
                                let spec: crate::types::FieldSpec = serde_json::from_value(
                                    collection
                                        .schema
                                        .get(field)
                                        .cloned()
                                        .ok_or_else(|| anyhow!("delta field missing"))?,
                                )?;
                                let external_ids = (0..local.count)
                                    .map(|row| {
                                        ids.external_id(row)
                                            .map(str::to_owned)
                                            .ok_or_else(|| anyhow!("missing local row"))
                                    })
                                    .collect::<Result<Vec<_>>>()?;
                                if spec.field_type == crate::types::FieldType::Vector
                                    && spec.vector_spec()?.is_some_and(|vector| {
                                        vector.backend != crate::types::VectorBackend::FlatCpu
                                    })
                                {
                                    let values = read_delta_values(&reader, &spec)?;
                                    let rows = external_ids.into_iter().zip(values).collect();
                                    engine.apply_checkpoint_delta(
                                        &collection.collection_id,
                                        field,
                                        rows,
                                    )?;
                                } else {
                                    engine.attach_checkpoint_delta_reader(
                                        &collection.collection_id,
                                        field,
                                        reader,
                                        external_ids,
                                    )?;
                                }
                            }
                        }
                    }
                    Ok(())
                },
            )?;
            if let Some(started) = delta_decode_started {
                self.add_recovery_timing(|timings| {
                    timings.delta_decode_ms = started.elapsed().as_millis() as u64;
                });
            }
            if defer_hnsw && !defer_until_aof {
                let vector_finish_started = self.recovery_profile.enabled().then(Instant::now);
                self.recovery_profile.phase_start(
                    RecoveryPhase::CheckpointHnswGraph,
                    || -> Result<()> {
                        if let Some(cache) = graph_cache.as_deref() {
                            engine.finish_checkpoint_vectors_with_graph_cache(Some(cache))?;
                        } else {
                            engine.finish_checkpoint_vectors()?;
                        }
                        Ok(())
                    },
                )?;
                if let Some(started) = vector_finish_started {
                    self.add_recovery_timing(|timings| {
                        timings.vector_finish_ms = started.elapsed().as_millis() as u64;
                    });
                }
            }
            let identity_hydration_started = self.recovery_profile.enabled().then(Instant::now);
            self.recovery_profile
                .phase_start(RecoveryPhase::IdentityHydration, || {
                    engine.hydrate_checkpoint_identities(&record.path, &capture)
                })?;
            if let Some(started) = identity_hydration_started {
                self.add_recovery_timing(|timings| {
                    timings.identity_hydration_ms = started.elapsed().as_millis() as u64;
                });
            }
            if let Some(started) = reopen_started {
                self.add_recovery_timing(|timings| {
                    timings.reopen_ms = started.elapsed().as_millis() as u64;
                });
            }
            return Ok(());
        }
        if collections == 0 {
            if reopened != 0 {
                bail!(
                    "empty generation {} reopened with unexpected sequence {reopened}",
                    record.name
                );
            }
        } else if reopened != record.sequence {
            bail!(
                "generation {} expected sequence {} but reopened {reopened}",
                record.name,
                record.sequence
            );
        }
        engine.bind_legacy_checkpoint_origin(&record.path)?;
        if let Some(started) = reopen_started {
            self.add_recovery_timing(|timings| {
                timings.reopen_ms = started.elapsed().as_millis() as u64;
            });
        }
        Ok(())
    }

    fn sweep_abandoned_staging(&self) -> Result<usize> {
        let mut removed = 0usize;
        for entry in std::fs::read_dir(&self.root)
            .with_context(|| format!("read checkpoint root {}", self.root.display()))?
        {
            let entry = entry?;
            let Some(raw) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if !is_known_staging_name(&raw) {
                continue;
            }
            if self.background.protects(&raw) {
                continue;
            }
            let path = entry.path();
            let metadata = std::fs::symlink_metadata(&path)
                .with_context(|| format!("inspect checkpoint staging {}", path.display()))?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                bail!(
                    "checkpoint staging must be a real directory: {}",
                    path.display()
                );
            }
            std::fs::remove_dir_all(&path)
                .with_context(|| format!("remove abandoned staging {}", path.display()))?;
            removed += 1;
        }
        if removed > 0 {
            sync_directory(&self.root).context("fsync root after staging cleanup")?;
        }
        Ok(removed)
    }

    fn reconcile_legacy_asides(&self) -> Result<bool> {
        let mut asides = Vec::new();
        for entry in std::fs::read_dir(&self.root)
            .with_context(|| format!("read checkpoint root {}", self.root.display()))?
        {
            let entry = entry?;
            let Some(raw) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            let Some(sequence) = parse_legacy_aside_name(&raw) else {
                continue;
            };
            let path = entry.path();
            let metadata = std::fs::symlink_metadata(&path)
                .with_context(|| format!("inspect legacy aside {}", path.display()))?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                bail!("legacy aside must be a real directory: {}", path.display());
            }
            asides.push((sequence, path));
        }
        asides.sort_by_key(|(sequence, _)| *sequence);
        let mut changed = false;
        for (sequence, aside) in asides {
            let committed = self.root.join(format!("gen-{sequence}"));
            match std::fs::symlink_metadata(&committed) {
                Ok(metadata) if !metadata.file_type().is_symlink() && metadata.is_dir() => {
                    std::fs::remove_dir_all(&aside).with_context(|| {
                        format!("remove stale legacy aside {}", aside.display())
                    })?;
                }
                Ok(_) => {
                    bail!(
                        "legacy checkpoint target must be a real directory: {}",
                        committed.display()
                    );
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    std::fs::rename(&aside, &committed).with_context(|| {
                        format!(
                            "restore legacy checkpoint {} -> {}",
                            aside.display(),
                            committed.display()
                        )
                    })?;
                }
                Err(error) => {
                    return Err(error).with_context(|| {
                        format!("inspect legacy checkpoint {}", committed.display())
                    });
                }
            }
            changed = true;
        }
        if changed {
            sync_directory(&self.root).context("fsync root after legacy aside recovery")?;
        }
        Ok(changed)
    }
}

fn parse_canonical_u64(raw: &str) -> Option<u64> {
    let value = raw.parse::<u64>().ok()?;
    (value.to_string() == raw).then_some(value)
}

fn parse_legacy_name(raw: &str) -> Option<u64> {
    parse_canonical_u64(raw.strip_prefix("gen-")?)
}

fn parse_legacy_aside_name(raw: &str) -> Option<u64> {
    raw.strip_prefix("gen-")?
        .strip_suffix(".old")
        .and_then(parse_canonical_u64)
}

fn root_entry_kind(metadata: &std::fs::Metadata) -> &'static str {
    if metadata.file_type().is_symlink() {
        "symlink"
    } else if metadata.is_file() {
        "regular file"
    } else if metadata.is_dir() {
        "directory"
    } else {
        "special file"
    }
}

fn is_legacy_aside_name(raw: &str) -> bool {
    parse_legacy_aside_name(raw).is_some()
}

fn is_known_staging_name(raw: &str) -> bool {
    raw.strip_prefix(".gen-")
        .and_then(|name| name.strip_suffix(".tmp"))
        .and_then(parse_canonical_u64)
        .is_some()
        || raw
            .strip_prefix(".stage-")
            .and_then(parse_revision_name)
            .is_some()
}

fn parse_revision_name(raw: &str) -> Option<(u64, u64)> {
    let rest = raw.strip_prefix("gen-")?;
    let (sequence, revision) = rest.rsplit_once("-rev-")?;
    Some((
        parse_canonical_u64(sequence)?,
        parse_canonical_u64(revision)?,
    ))
}

fn is_supported_generation_name(raw: &str) -> bool {
    parse_legacy_name(raw).is_some() || parse_revision_name(raw).is_some()
}

fn is_older_predecessor(raw: &str, current_sequence: u64, current_revision: u64) -> bool {
    if let Some(sequence) = parse_legacy_name(raw) {
        sequence <= current_sequence
    } else if let Some((sequence, revision)) = parse_revision_name(raw) {
        sequence <= current_sequence && revision < current_revision
    } else {
        false
    }
}

fn definitely_unpointed_after(raw: &str, current: &GenerationRecord) -> bool {
    if let Some(sequence) = parse_legacy_name(raw) {
        sequence > current.sequence
    } else if let Some((sequence, revision)) = parse_revision_name(raw) {
        current.legacy || sequence > current.sequence || revision >= current.revision
    } else {
        false
    }
}

/// Build the v2 catalog from the staged, self-contained checkpoint tree.  The
/// catalog never points at a predecessor: every referenced byte is below the
/// generation being published, which keeps retained checkpoints independently
/// reopenable after their predecessors are pruned.
fn catalog_collections(root: &Path, checkpoint_sequence: u64) -> Result<Vec<CollectionCatalog>> {
    let mut collections = Vec::new();
    for entry in std::fs::read_dir(root)
        .with_context(|| format!("read staged checkpoint {}", root.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.file_name().and_then(|name| name.to_str()) == Some(GENERATION_MANIFEST_FILE) {
            continue;
        }
        let metadata = std::fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            bail!(
                "catalogued collection must be a real directory: {}",
                path.display()
            );
        }
        let collection_id = crate::storage::collection_name_from_dir(&path)
            .ok_or_else(|| anyhow!("undecodable checkpoint subdir {}", path.display()))?;
        let collection_dir = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("non-utf8 checkpoint subdir {}", path.display()))?
            .to_owned();
        let schema_path = path.join(CHECKPOINT_SCHEMA_FILE);
        let schema: serde_json::Value = serde_json::from_slice(
            &std::fs::read(&schema_path)
                .with_context(|| format!("read checkpoint schema {}", schema_path.display()))?,
        )
        .with_context(|| format!("decode checkpoint schema {}", schema_path.display()))?;
        let schema_version = schema
            .get("version")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| {
                anyhow!(
                    "checkpoint schema has no version: {}",
                    schema_path.display()
                )
            })?
            .try_into()
            .context("checkpoint schema version exceeds u32")?;
        let layout = crate::storage::CheckpointLayout::from_sidecar(&schema)?;
        let fields = schema
            .get("fields")
            .cloned()
            .ok_or_else(|| anyhow!("checkpoint schema has no fields: {}", schema_path.display()))?;
        // Derive roles from schema: a keyword called `tag.eids` is not a vector sidecar.
        let specs: BTreeMap<String, crate::types::FieldSpec> =
            serde_json::from_value(fields.clone())?;
        let mut segments = vec![SegmentReference {
            role: SegmentRole::CollectionEids,
            field: None,
            ordinal: 0,
            kind: SegmentKind::Base,
            format: SegmentFormat::LsegV1,
            path: format!("{collection_dir}/_collection.lmeta.lseg"),
            local_rows: None,
            applied_seq: None,
            payload_sha256: None,
        }];
        for (name, spec) in specs {
            let stem = layout.field_stem(&name);
            segments.push(SegmentReference {
                role: SegmentRole::Field,
                field: Some(name.clone()),
                ordinal: 0,
                kind: SegmentKind::Base,
                format: SegmentFormat::LsegV1,
                path: format!("{collection_dir}/{stem}.lseg"),
                local_rows: None,
                applied_seq: None,
                payload_sha256: None,
            });
            if spec.field_type == crate::types::FieldType::Vector {
                segments.push(SegmentReference {
                    role: SegmentRole::VectorEids,
                    field: Some(name.clone()),
                    ordinal: 0,
                    kind: SegmentKind::Base,
                    format: SegmentFormat::LsegV1,
                    path: format!("{collection_dir}/{stem}.eids.lseg"),
                    local_rows: None,
                    applied_seq: None,
                    payload_sha256: None,
                });
            }
        }
        collections.push(CollectionCatalog {
            collection_id,
            // First v2 publication assigns durable ids. Subsequent reuse-aware
            // saves replace this provisional value with the inherited id.
            collection_generation: 0,
            schema_version,
            data_version: checkpoint_sequence,
            schema: fields,
            segments,
        });
    }
    collections.sort_by(|left, right| left.collection_id.cmp(&right.collection_id));
    for (index, collection) in collections.iter_mut().enumerate() {
        collection.collection_generation = (index as u64) + 1;
    }
    Ok(collections)
}

fn validate_catalog_references(root: &Path, manifest: &SegmentGenerationManifest) -> Result<()> {
    validate_catalog_references_with_prior(root, manifest, None)
}

fn validate_catalog_references_with_prior(
    root: &Path,
    manifest: &SegmentGenerationManifest,
    prior: PriorCatalog<'_>,
) -> Result<()> {
    if manifest.schema_version == GENERATION_MANIFEST_V3 {
        return validate_flat_catalog_references(root, manifest, prior);
    }
    let mut paths = BTreeSet::new();
    let mut catalog_ids = BTreeSet::new();
    let mut generations = BTreeSet::new();
    if manifest.next_collection_generation == 0 {
        bail!("invalid collection generation allocator");
    }
    let mut disk_ids = BTreeSet::new();
    for entry in std::fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            disk_ids.insert(
                crate::storage::collection_name_from_dir(&path)
                    .ok_or_else(|| anyhow!("undecodable checkpoint subdir {}", path.display()))?,
            );
        }
    }
    for collection in &manifest.collections {
        if !catalog_ids.insert(collection.collection_id.clone()) {
            bail!("duplicate catalog collection");
        }
        if collection.collection_generation == 0
            || collection.collection_generation >= manifest.next_collection_generation
            || !generations.insert(collection.collection_generation)
        {
            bail!("invalid or duplicate collection generation");
        }
        let collection_dir = collection_checkpoint_dir_name(&collection.collection_id);
        let disk_dir = root.join(&collection_dir);
        let sidecar: serde_json::Value =
            serde_json::from_slice(&std::fs::read(disk_dir.join(CHECKPOINT_SCHEMA_FILE))?)?;
        let layout = crate::storage::CheckpointLayout::from_sidecar(&sidecar)?;
        if sidecar.get("fields") != Some(&collection.schema) {
            bail!("catalog schema does not match checkpoint schema");
        }
        if sidecar.get("version").and_then(serde_json::Value::as_u64)
            != Some(collection.schema_version as u64)
        {
            bail!("catalog schema version does not match checkpoint schema");
        }
        let fields: BTreeMap<String, crate::types::FieldSpec> =
            serde_json::from_value(collection.schema.clone()).context("decode catalog schema")?;
        let mut expected = BTreeSet::from([format!("{collection_dir}/_collection.lmeta.lseg")]);
        for (name, spec) in &fields {
            let stem = layout.field_stem(name);
            expected.insert(format!("{collection_dir}/{stem}.lseg"));
            if spec.field_type == crate::types::FieldType::Vector {
                expected.insert(format!("{collection_dir}/{stem}.eids.lseg"));
            }
        }
        let mut present = BTreeSet::new();
        let mut ordinals = BTreeMap::<String, u32>::new();
        let mut delta_counts = BTreeMap::<String, usize>::new();
        for segment in &collection.segments {
            if matches!(segment.kind, SegmentKind::Delta) {
                let field = segment
                    .field
                    .clone()
                    .ok_or_else(|| anyhow!("delta must name a field"))?;
                let count = delta_counts.entry(field).or_default();
                *count += 1;
                if *count > 16 {
                    bail!("field exceeds sixteen delta segments");
                }
                validate_field_delta(
                    root,
                    manifest.checkpoint_sequence,
                    collection,
                    segment,
                    &fields,
                    &mut paths,
                    &mut ordinals,
                    prior,
                )?;
                continue;
            }
            if segment.ordinal != 0 {
                bail!("unsupported base segment layout");
            }
            match segment.role {
                SegmentRole::CollectionEids if segment.field.is_some() => {
                    bail!("collection_eids segment must not name a field")
                }
                SegmentRole::Field | SegmentRole::VectorEids if segment.field.is_none() => {
                    bail!("field segment must name a field")
                }
                _ => {}
            }
            let expected_path = match segment.role {
                SegmentRole::CollectionEids => format!("{collection_dir}/_collection.lmeta.lseg"),
                SegmentRole::Field | SegmentRole::VectorEids => {
                    let name = segment.field.as_ref().unwrap();
                    let stem = layout.field_stem(name);
                    let spec = fields
                        .get(name)
                        .ok_or_else(|| anyhow!("catalog segment field is absent from schema"))?;
                    if matches!(segment.role, SegmentRole::VectorEids) {
                        if spec.field_type != crate::types::FieldType::Vector {
                            bail!("vector_eids segment must name a vector field");
                        }
                        format!("{collection_dir}/{stem}.eids.lseg")
                    } else {
                        format!("{collection_dir}/{stem}.lseg")
                    }
                }
            };
            let relative = Path::new(&segment.path);
            if relative.is_absolute()
                || segment
                    .path
                    .split('/')
                    .any(|part| matches!(part, "" | "." | ".."))
            {
                bail!(
                    "segment reference path escapes generation: {}",
                    segment.path
                );
            }
            if segment.path != expected_path {
                bail!("segment reference does not match its collection and field");
            }
            present.insert(segment.path.clone());
            if !paths.insert(segment.path.clone()) {
                bail!("duplicate segment reference: {}", segment.path);
            }
            let target = root.join(relative);
            let metadata = std::fs::symlink_metadata(&target)
                .map_err(|_| anyhow!("catalogued segment is missing: {}", target.display()))?;
            if metadata.file_type().is_symlink() {
                bail!(
                    "catalogued segment must not be a symlink: {}",
                    target.display()
                );
            }
            if !metadata.is_file() {
                bail!(
                    "catalogued segment must be a regular file: {}",
                    target.display()
                );
            }
            let expected_checksum = segment
                .payload_sha256
                .as_deref()
                .ok_or_else(|| anyhow!("v2 base is missing payload_sha256"))?;
            if let Some(local) = &segment.local_rows {
                if !matches!(segment.role, SegmentRole::Field)
                    || local.path
                        != format!(
                            "{}.rows.cbor",
                            segment
                                .path
                                .strip_suffix(".lseg")
                                .ok_or_else(|| anyhow!("mapped base path has no suffix"))?
                        )
                    || local.format != "lumen-local-eids-cbor-v1"
                {
                    bail!("unsupported mapped base row layout");
                }
                if !paths.insert(local.path.clone()) {
                    bail!("duplicate mapped base row reference");
                }
                let rows_path = root.join(&local.path);
                let metadata = std::fs::symlink_metadata(&rows_path)?;
                if metadata.file_type().is_symlink() || !metadata.is_file() {
                    bail!("mapped base rows must be a regular file");
                }
                let reader = crate::segment::SegmentReader::open(&target)?;
                if reader.n_docs() != local.count
                    || reader.applied_seq()
                        != segment
                            .applied_seq
                            .ok_or_else(|| anyhow!("mapped base has no sequence"))?
                    || reader.applied_seq() > manifest.checkpoint_sequence
                {
                    bail!("mapped base row count or sequence differs from catalog");
                }
                if !is_inherited_delta(root, collection, segment, prior)? {
                    let rows = crate::segment::decode_sparse_local_rows(&rows_path, local.count)?;
                    if delta_payload_sha256(&target, &rows_path)? != expected_checksum {
                        bail!("mapped base checksum differs from catalog");
                    }
                    let field = segment.field.as_deref().expect("mapped field validated");
                    if fields[field].field_type == crate::types::FieldType::Vector {
                        let stem = layout.field_stem(field);
                        let ids = crate::segment::SegmentReader::open(
                            &disk_dir.join(format!("{stem}.eids.lseg")),
                        )?
                        .eids_all()
                        .ok_or_else(|| anyhow!("mapped vector EID sidecar is torn"))?;
                        if ids.len() != local.count as usize
                            || ids.iter().enumerate().any(|(row, eid)| {
                                rows.external_id(row as u32) != Some(eid.as_str())
                            })
                        {
                            bail!("mapped vector row map differs from EID sidecar");
                        }
                    }
                }
            } else if !is_inherited_delta(root, collection, segment, prior)?
                && base_payload_sha256(&target)? != expected_checksum
            {
                bail!("base payload checksum does not match catalog");
            }
        }
        if present != expected {
            bail!("catalog is missing a required base segment");
        }
    }
    if catalog_ids != disk_ids {
        bail!("catalog collection set does not match checkpoint");
    }
    Ok(())
}

fn validate_flat_catalog_references(
    root: &Path,
    manifest: &SegmentGenerationManifest,
    prior: PriorCatalog<'_>,
) -> Result<()> {
    let mut paths = BTreeSet::new();
    let mut generations = BTreeSet::new();
    for collection in &manifest.collections {
        if collection.collection_generation == 0
            || !generations.insert(collection.collection_generation)
        {
            bail!("invalid or duplicate collection generation");
        }
        let schema = serde_json::from_slice::<serde_json::Value>(&std::fs::read(
            collection_schema_path(root, collection),
        )?)?;
        if schema.get("fields") != Some(&collection.schema) {
            bail!("flat catalog schema does not match checkpoint schema");
        }
        let mut delta_counts = BTreeMap::<String, usize>::new();
        for segment in &collection.segments {
            if matches!(segment.kind, SegmentKind::Delta) {
                let field = segment
                    .field
                    .as_ref()
                    .ok_or_else(|| anyhow!("delta must name a field"))?;
                let count = delta_counts.entry(field.clone()).or_default();
                *count += 1;
                if *count > 16 {
                    bail!("field exceeds sixteen delta segments");
                }
            }
            if Path::new(&segment.path).is_absolute()
                || segment
                    .path
                    .split('/')
                    .any(|part| matches!(part, "" | "." | ".."))
            {
                bail!("flat segment path escapes generation");
            }
            if !paths.insert(segment.path.clone()) {
                bail!("duplicate flat segment reference: {}", segment.path);
            }
            let target = root.join(&segment.path);
            let metadata = std::fs::symlink_metadata(&target).map_err(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    anyhow!("catalogued segment is missing: {}", target.display())
                } else {
                    anyhow::Error::new(error)
                        .context(format!("inspect flat segment {}", target.display()))
                }
            })?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("flat segment is not a regular file: {}", target.display());
            }
            if let Some(local) = &segment.local_rows {
                if !paths.insert(local.path.clone()) {
                    bail!("duplicate flat row reference: {}", local.path);
                }
                let rows = root.join(&local.path);
                if !std::fs::symlink_metadata(&rows)
                    .with_context(|| format!("inspect flat row map {}", rows.display()))?
                    .is_file()
                {
                    bail!("flat row map is not a regular file");
                }
                if !is_inherited_delta(root, collection, segment, prior)?
                    && delta_payload_sha256(&target, &rows)?
                        != segment.payload_sha256.as_deref().unwrap_or_default()
                {
                    bail!("flat payload checksum does not match catalog");
                }
            } else if !is_inherited_delta(root, collection, segment, prior)?
                && base_payload_sha256(&target)?
                    != segment.payload_sha256.as_deref().unwrap_or_default()
            {
                bail!("flat base checksum does not match catalog");
            }
        }
    }
    Ok(())
}

fn prepare_live_delta_readers(
    root: &Path,
    collections: &[CollectionCatalog],
    capture: &mut crate::storage::CheckpointCapture,
) -> Result<()> {
    for collection in collections {
        let Some(fields) = capture.field_deltas.get(&collection.collection_id) else {
            continue;
        };
        let specs: BTreeMap<String, crate::types::FieldSpec> =
            serde_json::from_value(collection.schema.clone())?;
        for field in fields.keys() {
            let spec = specs
                .get(field)
                .ok_or_else(|| anyhow!("captured delta field is absent from schema"))?;
            if matches!(spec.field_type, crate::types::FieldType::Vector)
                && spec
                    .vector_spec()?
                    .is_some_and(|vector| vector.backend != crate::types::VectorBackend::FlatCpu)
            {
                continue;
            }
            let segment = collection
                .segments
                .iter()
                .filter(|segment| {
                    matches!(segment.kind, SegmentKind::Delta)
                        && segment.field.as_deref() == Some(field.as_str())
                })
                .max_by_key(|segment| segment.ordinal)
                .ok_or_else(|| anyhow!("captured delta has no catalog segment"))?;
            let local = segment
                .local_rows
                .as_ref()
                .ok_or_else(|| anyhow!("delta has no row map"))?;
            let rows =
                crate::segment::decode_sparse_local_rows(&root.join(&local.path), local.count)?;
            let external_ids = (0..local.count)
                .map(|row| {
                    rows.external_id(row)
                        .map(str::to_owned)
                        .ok_or_else(|| anyhow!("delta row map is incomplete"))
                })
                .collect::<Result<Vec<_>>>()?;
            let reader = std::sync::Arc::new(crate::segment::SegmentReader::open(
                &root.join(&segment.path),
            )?);
            capture
                .live_delta_inputs
                .entry(collection.collection_id.clone())
                .or_default()
                .entry(field.clone())
                .or_default()
                .push(reader.clone());
            capture
                .prepared_deltas
                .entry(collection.collection_id.clone())
                .or_default()
                .push(crate::storage::PreparedCheckpointDelta {
                    field: field.clone(),
                    reader,
                    external_ids,
                });
        }
    }
    Ok(())
}

fn read_delta_values(
    reader: &crate::segment::SegmentReader,
    spec: &crate::types::FieldSpec,
) -> Result<Vec<Option<crate::storage::CheckpointValue>>> {
    use crate::storage::CheckpointValue;
    use crate::types::FieldType;
    if spec.field_type == FieldType::Text {
        let mut values: Vec<_> = (0..reader.n_docs())
            .map(|row| {
                reader.text_is_present(row).then(|| CheckpointValue::Text {
                    doc_len: reader.text_doc_len(row),
                    tokens: BTreeMap::new(),
                })
            })
            .collect();
        let tokens = reader
            .text_tokens_all()
            .ok_or_else(|| anyhow!("invalid text delta postings"))?;
        for (token, rows, tfs) in tokens {
            if rows.len() != tfs.len() {
                bail!("text delta posting length mismatch");
            }
            for (row, tf) in rows.into_iter().zip(tfs) {
                let Some(Some(CheckpointValue::Text { tokens, .. })) = values.get_mut(row as usize)
                else {
                    bail!("text delta posting names absent row");
                };
                if tf == 0 || tokens.insert(token.clone(), tf).is_some() {
                    bail!("invalid text delta frequency");
                }
            }
        }
        let mut count = 0;
        let mut total = 0;
        for value in values.iter().flatten() {
            let CheckpointValue::Text { doc_len, tokens } = value else {
                unreachable!()
            };
            if tokens.values().map(|tf| *tf as u64).sum::<u64>() != *doc_len as u64 {
                bail!("text delta lengths and postings disagree");
            }
            count += 1;
            total += *doc_len as u64;
        }
        if count != reader.text_doc_count() || total != reader.text_total_doc_len() {
            bail!("text delta corpus statistics disagree");
        }
        return Ok(values);
    }
    if spec.field_type == FieldType::Vector {
        let dim = spec
            .dim
            .ok_or_else(|| anyhow!("vector delta dimension missing"))? as usize;
        if reader.vectors_slice(dim).is_none() {
            bail!("invalid vector delta geometry");
        }
    }
    (0..reader.n_docs())
        .map(|row| {
            Ok(match spec.field_type {
                FieldType::Keyword => reader.keyword_at(row).map(CheckpointValue::Keyword),
                FieldType::Number => reader.number_at(row).map(CheckpointValue::Number),
                FieldType::Set => reader.set_at(row).map(CheckpointValue::Set),
                FieldType::Hash => reader.hash_at(row).map(CheckpointValue::Hash),
                FieldType::Vector => reader
                    .vector_at(
                        row,
                        spec.dim
                            .ok_or_else(|| anyhow!("vector delta dimension missing"))?
                            as usize,
                    )
                    .map(|value| CheckpointValue::Vector(value.to_vec())),
                _ => bail!("unsupported delta field type"),
            })
        })
        .collect()
}

fn delta_path_prefix(collection: &str, field: &str, ordinal: u32) -> String {
    format!(
        "{}/__delta/{}/{ordinal}",
        collection_checkpoint_dir_name(collection),
        collection_checkpoint_dir_name(field)
    )
}

fn write_field_deltas(
    root: &Path,
    sequence: u64,
    collection: &mut CollectionCatalog,
    fields: &crate::storage::CheckpointDeltas,
) -> Result<()> {
    for (field, rows) in fields {
        if rows.is_empty() {
            continue;
        }
        let ordinal = collection
            .segments
            .iter()
            .filter(|segment| {
                segment.field.as_deref() == Some(field.as_str())
                    && matches!(segment.role, SegmentRole::Field)
            })
            .map(|segment| segment.ordinal)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(|| anyhow!("segment ordinal exhausted"))?;
        let field_dir = collection_checkpoint_dir_name(field);
        let prefix = delta_path_prefix(&collection.collection_id, field, ordinal);
        let segment_path = format!("{prefix}.lseg");
        let rows_path = format!("{prefix}.rows.cbor");
        std::fs::create_dir_all(root.join(&segment_path).parent().unwrap())?;
        let ids: Vec<_> = rows.iter().map(|(id, _)| id.clone()).collect();
        use crate::storage::CheckpointValue;
        let specs: BTreeMap<String, crate::types::FieldSpec> =
            serde_json::from_value(collection.schema.clone())?;
        let spec = specs
            .get(field)
            .ok_or_else(|| anyhow!("delta field absent from schema"))?;
        let staged_scalar = rows.iter().any(|(_, value)| {
            matches!(value.as_deref(), Some(CheckpointValue::StagedScalar { .. }))
        });
        if staged_scalar
            && matches!(
                spec.field_type,
                crate::types::FieldType::Keyword
                    | crate::types::FieldType::Number
                    | crate::types::FieldType::Set
            )
        {
            crate::storage::write_scalar_checkpoint_rows(
                &root.join(&segment_path),
                sequence,
                spec.field_type,
                rows,
            )?;
        } else {
            match spec.field_type {
                crate::types::FieldType::Keyword => {
                    let values: Vec<_> = rows
                        .iter()
                        .map(|(_, value)| match value.as_deref() {
                            Some(CheckpointValue::Keyword(value)) => Ok(Some(value.as_str())),
                            None => Ok(None),
                            _ => bail!("keyword delta value mismatch"),
                        })
                        .collect::<Result<_>>()?;
                    let mut postings: BTreeMap<String, roaring::RoaringBitmap> = BTreeMap::new();
                    for (row, value) in values.iter().enumerate() {
                        if let Some(value) = value {
                            postings
                                .entry((*value).to_owned())
                                .or_default()
                                .insert(u32::try_from(row)?);
                        }
                    }
                    crate::segment::write_keyword_segment(
                        &root.join(&segment_path),
                        sequence,
                        &values,
                        &postings,
                    )?;
                }
                crate::types::FieldType::Number => {
                    let values: Vec<_> = rows
                        .iter()
                        .map(|(_, value)| match value.as_deref() {
                            Some(CheckpointValue::Number(value)) => Ok(Some(*value)),
                            None => Ok(None),
                            _ => bail!("number delta value mismatch"),
                        })
                        .collect::<Result<_>>()?;
                    crate::segment::write_number_segment(
                        &root.join(&segment_path),
                        sequence,
                        &values,
                    )?;
                }
                crate::types::FieldType::Hash => {
                    let values: Vec<_> = rows
                        .iter()
                        .map(|(_, value)| match value.as_deref() {
                            Some(CheckpointValue::Hash(value)) => Ok(Some(*value)),
                            None => Ok(None),
                            _ => bail!("hash delta value mismatch"),
                        })
                        .collect::<Result<_>>()?;
                    crate::segment::write_hash_segment(
                        &root.join(&segment_path),
                        sequence,
                        &values,
                    )?;
                }
                crate::types::FieldType::Set => {
                    let values: Vec<_> = rows
                        .iter()
                        .map(|(_, value)| match value.as_deref() {
                            Some(CheckpointValue::Set(value)) => Ok(Some(value.as_slice())),
                            None => Ok(None),
                            _ => bail!("set delta value mismatch"),
                        })
                        .collect::<Result<_>>()?;
                    let mut postings: BTreeMap<String, roaring::RoaringBitmap> = BTreeMap::new();
                    for (row, values) in values.iter().enumerate() {
                        if let Some(values) = values {
                            for value in *values {
                                postings
                                    .entry(value.clone())
                                    .or_default()
                                    .insert(u32::try_from(row)?);
                            }
                        }
                    }
                    crate::segment::write_set_segment(
                        &root.join(&segment_path),
                        sequence,
                        &values,
                        &postings,
                    )?;
                }
                crate::types::FieldType::Text => {
                    crate::storage::write_text_checkpoint_rows(
                        &root.join(&segment_path),
                        sequence,
                        rows,
                    )?;
                }
                crate::types::FieldType::Vector => {
                    let values: Vec<_> = rows
                        .iter()
                        .map(|(_, value)| match value.as_deref() {
                            Some(CheckpointValue::Vector(value)) => Ok(Some(value.as_slice())),
                            Some(CheckpointValue::StagedVector(value)) => {
                                Ok(Some(value.as_f32_slice()))
                            }
                            None => Ok(None),
                            _ => bail!("vector delta value mismatch"),
                        })
                        .collect::<Result<_>>()?;
                    let dim = spec
                        .dim
                        .ok_or_else(|| anyhow!("vector delta dimension missing"))?
                        as usize;
                    crate::segment::write_vector_segment(
                        &root.join(&segment_path),
                        sequence,
                        dim,
                        &values,
                    )?;
                }
                _ => bail!("unsupported delta field"),
            }
        }
        crate::segment::encode_sparse_local_rows(&root.join(&rows_path), &ids)?;
        let payload_sha256 =
            delta_payload_sha256(&root.join(&segment_path), &root.join(&rows_path))?;
        collection.segments.push(SegmentReference {
            role: SegmentRole::Field,
            field: Some(field.clone()),
            ordinal,
            kind: SegmentKind::Delta,
            format: SegmentFormat::LsegV1,
            path: segment_path,
            local_rows: Some(LocalRowsReference {
                format: "lumen-local-eids-cbor-v1".to_owned(),
                path: rows_path,
                count: u32::try_from(rows.len())?,
            }),
            applied_seq: Some(sequence),
            payload_sha256: Some(payload_sha256),
        });
    }
    Ok(())
}

/// One field a merge job will compact: an eligible delta stack in the
/// catalog, with the base segment it may fold in.
///
/// Selection reads only file sizes, so it runs against the read-only source
/// generation before the job stages anything. That is what lets the job link
/// exactly one collection into its scratch stage: the collection it is about
/// to compact is known before the first hard link is made. Every candidate a
/// single call to `select_staged_delta_window` returns shares that same
/// `collection_index`/`collection_id`.
pub(super) struct StagedMergeCandidate {
    collection_index: usize,
    /// The collection whose directory the compaction reads and writes. It is
    /// the only directory a job's scratch stage needs.
    pub(super) collection_id: String,
    field: String,
    inputs: Vec<SegmentReference>,
    base: SegmentReference,
    includes_base: bool,
}

/// Select one field-level merge from `collections`, reading segment sizes from
/// `root`.
///
/// The scheduler first chooses the collection with the deepest eligible delta
/// stack. Ties are deterministic: catalog order wins. Every field tied at
/// that collection depth is then returned in field-name order. Shallower
/// eligible fields remain for a later scheduling pass. Unless a selected
/// field's complete delta stack has reached the base size, that field
/// contributes exactly the adjacent pair with the smallest combined on-disk
/// size.
/// Once the delta stack reaches the base size, the base and all captured
/// deltas are folded together so the base can be replaced.
pub(super) fn select_staged_delta_window(
    root: &Path,
    collections: &[CollectionCatalog],
) -> Result<Vec<StagedMergeCandidate>> {
    let mut selected_collection: Option<(usize, usize)> = None;
    for (collection_index, collection) in collections.iter().enumerate() {
        let mut by_field = BTreeMap::<String, Vec<&SegmentReference>>::new();
        for segment in &collection.segments {
            if matches!(segment.role, SegmentRole::Field)
                && matches!(segment.kind, SegmentKind::Delta)
            {
                by_field
                    .entry(segment.field.clone().expect("field delta has field"))
                    .or_default()
                    .push(segment);
            }
        }
        let deepest = by_field
            .values()
            .filter(|deltas| deltas.len() >= 4)
            .map(Vec::len)
            .max()
            .unwrap_or(0);
        if deepest > 0
            && selected_collection
                .as_ref()
                .is_none_or(|(_, selected_depth)| deepest > *selected_depth)
        {
            selected_collection = Some((collection_index, deepest));
        }
    }
    let Some((collection_index, selected_depth)) = selected_collection else {
        return Ok(Vec::new());
    };
    let collection = &collections[collection_index];
    // A small durable-cut cohort should share the whole-root publication
    // cost. Expand its field bound only when the complete job stays inside
    // the existing byte budget. Larger pair cohorts retain their old bound.
    const MAX_FIELDS_PER_MERGE_JOB: usize = 8;
    const MAX_BYTE_BOUNDED_FIELDS_PER_MERGE_JOB: usize = 16;
    const MAX_LOGICAL_READ_BYTES_PER_MERGE_JOB: u64 = 24 * 1024 * 1024;
    let mut candidates = Vec::new();
    let mut by_field = BTreeMap::<String, Vec<&SegmentReference>>::new();
    for segment in &collection.segments {
        if matches!(segment.role, SegmentRole::Field) && matches!(segment.kind, SegmentKind::Delta)
        {
            by_field
                .entry(segment.field.clone().expect("field delta has field"))
                .or_default()
                .push(segment);
        }
    }
    for (field, deltas) in by_field {
        if deltas.len() < 4 || deltas.len() != selected_depth {
            continue;
        }
        let delta_bytes = deltas.iter().try_fold(0u64, |total, segment| {
            total
                .checked_add(segment_reference_bytes(root, segment)?)
                .ok_or_else(|| anyhow!("delta byte count overflow"))
        })?;
        let base = collection
            .segments
            .iter()
            .find(|segment| {
                matches!(segment.role, SegmentRole::Field)
                    && matches!(segment.kind, SegmentKind::Base)
                    && segment.field.as_deref() == Some(field.as_str())
            })
            .ok_or_else(|| anyhow!("delta field has no base segment"))?;
        let mut base_bytes = segment_reference_bytes(root, base)?;
        if let Some(sidecar) = collection.segments.iter().find(|segment| {
            matches!(segment.role, SegmentRole::VectorEids) && segment.field == base.field
        }) {
            base_bytes = base_bytes
                .checked_add(segment_reference_bytes(root, sidecar)?)
                .ok_or_else(|| anyhow!("base byte count overflow"))?;
        }
        let includes_base = delta_bytes >= base_bytes;
        let mut full_stack_bytes = delta_bytes;
        if includes_base {
            full_stack_bytes = full_stack_bytes
                .checked_add(base_bytes)
                .ok_or_else(|| anyhow!("merge input byte count overflow"))?;
        }
        let (inputs, merge_bytes) = if includes_base {
            (
                deltas.iter().map(|segment| (*segment).clone()).collect(),
                full_stack_bytes,
            )
        } else {
            let mut smallest: Option<(u64, usize)> = None;
            for (index, pair) in deltas.windows(2).enumerate() {
                let pair_bytes = segment_reference_bytes(root, pair[0])?
                    .checked_add(segment_reference_bytes(root, pair[1])?)
                    .ok_or_else(|| anyhow!("adjacent delta byte count overflow"))?;
                if smallest.is_none_or(|(best_bytes, _)| pair_bytes < best_bytes) {
                    smallest = Some((pair_bytes, index));
                }
            }
            let (pair_bytes, index) =
                smallest.ok_or_else(|| anyhow!("delta field has no adjacent pair"))?;
            (
                deltas[index..index + 2]
                    .iter()
                    .map(|segment| (*segment).clone())
                    .collect(),
                pair_bytes,
            )
        };
        candidates.push((
            deltas.len(),
            field.clone(),
            merge_bytes,
            StagedMergeCandidate {
                collection_index,
                collection_id: collection.collection_id.clone(),
                field,
                inputs,
                base: base.clone(),
                includes_base,
            },
        ));
    }
    candidates.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
    let mut selected = Vec::new();
    let mut total = 0u64;
    for (_depth, _field, estimate, candidate) in candidates {
        let same_pair_cohort = !candidate.includes_base
            && selected.iter().all(|selected: &StagedMergeCandidate| {
                !selected.includes_base && selected.inputs.len() == 2
            });
        let within_byte_budget = total
            .checked_add(estimate)
            .is_some_and(|next| next <= MAX_LOGICAL_READ_BYTES_PER_MERGE_JOB);
        if selected.is_empty()
            || (selected.len() < MAX_BYTE_BOUNDED_FIELDS_PER_MERGE_JOB && within_byte_budget)
            || (selected.len() < MAX_FIELDS_PER_MERGE_JOB
                && same_pair_cohort
                && estimate <= MAX_LOGICAL_READ_BYTES_PER_MERGE_JOB)
        {
            total = total
                .checked_add(estimate)
                .ok_or_else(|| anyhow!("merge input byte count overflow"))?;
            selected.push(candidate);
        } else {
            break;
        }
    }
    Ok(selected)
}

/// Select the next bounded window for a field already admitted to one merge
/// job. Unlike the initial scheduler selector, this helper also accepts two
/// or three remaining deltas so one scratch job can drain its chosen fields
/// without widening collection priority.
pub(super) fn select_staged_field_window(
    root: &Path,
    collection: &CollectionCatalog,
    collection_index: usize,
    field: &str,
) -> Result<Option<StagedMergeCandidate>> {
    let deltas: Vec<&SegmentReference> = collection
        .segments
        .iter()
        .filter(|segment| {
            segment.role == SegmentRole::Field
                && segment.kind == SegmentKind::Delta
                && segment.field.as_deref() == Some(field)
        })
        .collect();
    if deltas.len() < 2 {
        return Ok(None);
    }
    let base = collection
        .segments
        .iter()
        .find(|segment| {
            segment.role == SegmentRole::Field
                && segment.kind == SegmentKind::Base
                && segment.field.as_deref() == Some(field)
        })
        .ok_or_else(|| anyhow!("delta field has no base segment"))?;
    let delta_bytes = deltas.iter().try_fold(0u64, |total, segment| {
        total
            .checked_add(segment_reference_bytes(root, segment)?)
            .ok_or_else(|| anyhow!("delta byte count overflow"))
    })?;
    let mut base_bytes = segment_reference_bytes(root, base)?;
    if let Some(sidecar) = collection
        .segments
        .iter()
        .find(|segment| segment.role == SegmentRole::VectorEids && segment.field == base.field)
    {
        base_bytes = base_bytes
            .checked_add(segment_reference_bytes(root, sidecar)?)
            .ok_or_else(|| anyhow!("base byte count overflow"))?;
    }
    let includes_base = delta_bytes >= base_bytes;
    let inputs = if includes_base {
        deltas.iter().map(|segment| (*segment).clone()).collect()
    } else {
        let mut smallest: Option<(u64, usize)> = None;
        for (index, pair) in deltas.windows(2).enumerate() {
            let pair_bytes = segment_reference_bytes(root, pair[0])?
                .checked_add(segment_reference_bytes(root, pair[1])?)
                .ok_or_else(|| anyhow!("adjacent delta byte count overflow"))?;
            if smallest.is_none_or(|(best_bytes, _)| pair_bytes < best_bytes) {
                smallest = Some((pair_bytes, index));
            }
        }
        let (_, index) = smallest.ok_or_else(|| anyhow!("delta field has no adjacent pair"))?;
        deltas[index..index + 2]
            .iter()
            .map(|segment| (*segment).clone())
            .collect()
    };
    Ok(Some(StagedMergeCandidate {
        collection_index,
        collection_id: collection.collection_id.clone(),
        field: field.to_owned(),
        inputs,
        base: base.clone(),
        includes_base,
    }))
}

/// Fold every selected candidate's layers into one compacted output each,
/// inside the staged generation already prepared for this job. Every
/// candidate shares the same collection, so each fold reads and writes only
/// that collection's directory; the publication side folds each output as a
/// sequence of rebases, and keeping outputs in candidate order leaves the
/// identity check per output unchanged.
fn compact_staged_delta_windows(
    root: &Path,
    sequence: u64,
    collections: &mut [CollectionCatalog],
    capture: &mut crate::storage::CheckpointCapture,
    scratch_delta_readers: &mut BTreeMap<String, Vec<Arc<crate::segment::SegmentReader>>>,
    observer: &dyn MergeObserver,
    candidates: Vec<StagedMergeCandidate>,
) -> Result<Vec<compaction::CompactedField>> {
    encode_staged_candidates_in_order(candidates, observer, |candidate| {
        let collection = collections[candidate.collection_index].clone();
        let selected = if candidate.includes_base {
            let mut selected = vec![candidate.base.clone()];
            selected.extend(candidate.inputs.iter().cloned());
            selected
        } else {
            candidate.inputs.clone()
        };
        let output = compaction::write_compacted_field(
            root,
            sequence,
            &collection,
            &candidate.field,
            &selected,
            candidate.includes_base,
        )?;
        let StagedMergeCandidate {
            collection_index,
            collection_id: _,
            field,
            inputs,
            base,
            includes_base,
        } = candidate;
        finish_compacted_field(
            root,
            &mut collections[collection_index],
            field,
            inputs,
            base,
            includes_base,
            capture,
            scratch_delta_readers,
            output,
        )
    })
}

/// Notify and encode one candidate before moving to the next one. The merge
/// worker is already serialized, so this deliberately has no nested pool.
fn encode_staged_candidates_in_order<T, F>(
    candidates: Vec<StagedMergeCandidate>,
    observer: &dyn MergeObserver,
    mut encode: F,
) -> Result<Vec<T>>
where
    F: FnMut(StagedMergeCandidate) -> Result<T>,
{
    let mut outputs = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        observer.observe(MergePhase::BeforeEncode)?;
        outputs.push(encode(candidate)?);
    }
    Ok(outputs)
}

#[allow(clippy::too_many_arguments)]
fn finish_compacted_field(
    root: &Path,
    collection: &mut CollectionCatalog,
    field: String,
    inputs: Vec<SegmentReference>,
    base: SegmentReference,
    includes_base: bool,
    capture: &mut crate::storage::CheckpointCapture,
    scratch_delta_readers: &mut BTreeMap<String, Vec<Arc<crate::segment::SegmentReader>>>,
    output: compaction::CompactedField,
) -> Result<compaction::CompactedField> {
    let selected = if includes_base {
        let mut selected = vec![base];
        selected.extend(inputs.iter().cloned());
        selected
    } else {
        inputs.clone()
    };
    let specs: BTreeMap<String, crate::types::FieldSpec> =
        serde_json::from_value(collection.schema.clone())?;
    let is_hnsw = specs
        .get(&field)
        .ok_or_else(|| anyhow!("compacted field has no schema"))?
        .vector_spec()?
        .is_some_and(|spec| spec.backend != crate::types::VectorBackend::FlatCpu);
    let live_inputs = if is_hnsw {
        Vec::new()
    } else {
        let readers = scratch_delta_readers
            .get(&field)
            .cloned()
            .ok_or_else(|| anyhow!("compaction has no captured live input identity"))?;
        let catalog_deltas: Vec<_> = collection
            .segments
            .iter()
            .filter(|segment| {
                matches!(segment.role, SegmentRole::Field)
                    && matches!(segment.kind, SegmentKind::Delta)
                    && segment.field.as_deref() == Some(field.as_str())
            })
            .collect();
        if readers.len() != catalog_deltas.len() {
            bail!("compaction catalog differs from captured live layer count");
        }
        let mut selected_readers = Vec::with_capacity(inputs.len());
        for input in &inputs {
            let position = catalog_deltas
                .iter()
                .position(|segment| *segment == input)
                .ok_or_else(|| anyhow!("compaction input is absent from captured catalog"))?;
            selected_readers.push(
                readers
                    .get(position)
                    .cloned()
                    .ok_or_else(|| anyhow!("compaction input has no captured reader"))?,
            );
        }
        selected_readers
    };
    let live_base = if includes_base && !is_hnsw {
        Some(
            capture
                .live_base_inputs
                .get(&collection.collection_id)
                .and_then(|fields| fields.get(&field))
                .cloned()
                .ok_or_else(|| anyhow!("compaction has no captured live base identity"))?,
        )
    } else {
        None
    };
    if !is_hnsw {
        let local = output
            .output
            .local_rows
            .as_ref()
            .ok_or_else(|| anyhow!("compacted delta has no row map"))?;
        let rows = crate::segment::decode_sparse_local_rows(&root.join(&local.path), local.count)?;
        let external_ids = (0..local.count)
            .map(|row| {
                rows.external_id(row)
                    .map(str::to_owned)
                    .ok_or_else(|| anyhow!("compacted row map is incomplete"))
            })
            .collect::<Result<_>>()?;
        let reader = std::sync::Arc::new(crate::segment::SegmentReader::open(
            &root.join(&output.output.path),
        )?);
        let staged_inputs = scratch_delta_readers
            .get_mut(&field)
            .expect("staged live input identity validated");
        if includes_base {
            staged_inputs.clear();
            capture
                .live_base_inputs
                .entry(collection.collection_id.clone())
                .or_default()
                .insert(field.clone(), reader.clone());
        } else {
            let start = staged_inputs
                .windows(live_inputs.len())
                .position(|window| {
                    window
                        .iter()
                        .zip(&live_inputs)
                        .all(|(actual, expected)| std::sync::Arc::ptr_eq(actual, expected))
                })
                .ok_or_else(|| anyhow!("staged compaction input identity changed"))?;
            staged_inputs.splice(start..start + live_inputs.len(), [reader.clone()]);
        }
        capture
            .live_delta_inputs
            .entry(collection.collection_id.clone())
            .or_default()
            .insert(field.clone(), staged_inputs.clone());
        capture
            .prepared_compactions
            .entry(collection.collection_id.clone())
            .or_default()
            .push(crate::storage::PreparedCheckpointCompaction {
                field: field.clone(),
                base: live_base,
                inputs: live_inputs,
                reader,
                external_ids,
                scalar: None,
            });
    }
    replace_compacted_delta_references(collection, &output)?;
    if let Some(sidecar) = &output.vector_eids {
        let old = collection
            .segments
            .iter_mut()
            .find(|segment| {
                matches!(segment.role, SegmentRole::VectorEids) && segment.field == sidecar.field
            })
            .ok_or_else(|| anyhow!("compacted vector base has no catalog sidecar"))?;
        *old = sidecar.clone();
    }
    for input in &selected {
        if input.path != output.output.path {
            std::fs::remove_file(root.join(&input.path))?;
        }
        if let Some(rows) = &input.local_rows {
            if output
                .output
                .local_rows
                .as_ref()
                .is_none_or(|output_rows| output_rows.path != rows.path)
            {
                std::fs::remove_file(root.join(&rows.path))?;
            }
        }
    }
    Ok(output)
}

fn segment_reference_bytes(root: &Path, segment: &SegmentReference) -> Result<u64> {
    std::iter::once(&segment.path)
        .chain(segment.local_rows.iter().map(|rows| &rows.path))
        .try_fold(0u64, |total, path| {
            total
                .checked_add(std::fs::metadata(root.join(path))?.len())
                .ok_or_else(|| anyhow!("segment byte count overflow"))
        })
}

fn replace_compacted_delta_references(
    collection: &mut CollectionCatalog,
    compacted: &compaction::CompactedField,
) -> Result<()> {
    let last = compacted
        .inputs
        .last()
        .ok_or_else(|| anyhow!("compaction has no inputs"))?;
    let mut replaced = Vec::with_capacity(collection.segments.len());
    for segment in collection.segments.drain(..) {
        if compacted
            .inputs
            .iter()
            .any(|input| serde_json::to_value(input).ok() == serde_json::to_value(&segment).ok())
        {
            if serde_json::to_value(&segment)? == serde_json::to_value(last)? {
                replaced.push(compacted.output.clone());
            }
            continue;
        }
        replaced.push(segment);
    }
    collection.segments = replaced;
    Ok(())
}

fn validate_field_delta(
    root: &Path,
    checkpoint_sequence: u64,
    collection: &CollectionCatalog,
    segment: &SegmentReference,
    fields: &BTreeMap<String, crate::types::FieldSpec>,
    paths: &mut BTreeSet<String>,
    ordinals: &mut BTreeMap<String, u32>,
    prior: PriorCatalog<'_>,
) -> Result<()> {
    let field = segment
        .field
        .as_deref()
        .ok_or_else(|| anyhow!("delta must name a field"))?;
    if !matches!(segment.role, SegmentRole::Field)
        || !fields.get(field).is_some_and(|spec| {
            matches!(
                spec.field_type,
                crate::types::FieldType::Keyword
                    | crate::types::FieldType::Number
                    | crate::types::FieldType::Set
                    | crate::types::FieldType::Hash
                    | crate::types::FieldType::Text
                    | crate::types::FieldType::Vector
            )
        })
    {
        bail!("unsupported delta field type or role");
    }
    let ordinal = ordinals.entry(field.to_owned()).or_default();
    if segment.ordinal <= *ordinal {
        bail!("delta ordinals must be strictly increasing and ordered");
    }
    *ordinal = segment.ordinal;
    let local = segment
        .local_rows
        .as_ref()
        .ok_or_else(|| anyhow!("delta must include local rows"))?;
    let (expected_segment_path, expected_rows_path) = if flat_layout(collection) {
        let field_dir = collection_checkpoint_dir_name(field);
        (
            flat_payload_name(
                &collection.collection_id,
                Path::new(&format!("__delta/{field_dir}/{}.lseg", segment.ordinal)),
            ),
            flat_payload_name(
                &collection.collection_id,
                Path::new(&format!(
                    "__delta/{field_dir}/{}.rows.cbor",
                    segment.ordinal
                )),
            ),
        )
    } else {
        let prefix = delta_path_prefix(&collection.collection_id, field, segment.ordinal);
        (format!("{prefix}.lseg"), format!("{prefix}.rows.cbor"))
    };
    if segment.path != expected_segment_path
        || local.path != expected_rows_path
        || local.format != "lumen-local-eids-cbor-v1"
        || local.count == 0
    {
        bail!("invalid delta path or local row format");
    }
    for path in [&segment.path, &local.path] {
        if !paths.insert(path.clone()) {
            bail!("duplicate delta reference");
        }
        let metadata = std::fs::symlink_metadata(root.join(path))?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            bail!("delta reference must be a regular file");
        }
    }
    let reader = crate::segment::SegmentReader::open(&root.join(&segment.path))?;
    if reader.n_docs() != local.count
        || reader.applied_seq()
            != segment
                .applied_seq
                .ok_or_else(|| anyhow!("v2 delta is missing applied_seq"))?
        || reader.applied_seq() > checkpoint_sequence
    {
        bail!("delta row count or sequence does not match catalog");
    }
    let expected = segment
        .payload_sha256
        .as_deref()
        .ok_or_else(|| anyhow!("v2 delta is missing payload_sha256"))?;
    if !is_inherited_delta(root, collection, segment, prior)? {
        crate::segment::decode_sparse_local_rows(&root.join(&local.path), local.count)?;
        if delta_payload_sha256(&root.join(&segment.path), &root.join(&local.path))? != expected {
            bail!("delta payload checksum does not match catalog");
        }
    }
    Ok(())
}

/// Reuse validation only for the exact prior reference and the same immutable
/// hard-linked files. Cold recovery never supplies a predecessor.
fn is_inherited_delta(
    root: &Path,
    collection: &CollectionCatalog,
    segment: &SegmentReference,
    prior: PriorCatalog<'_>,
) -> Result<bool> {
    let Some((old_root, manifest)) = prior else {
        return Ok(false);
    };
    let Some(old) = manifest.collections.iter().find(|old| {
        old.collection_id == collection.collection_id
            && old.collection_generation == collection.collection_generation
            && old.schema_version == collection.schema_version
            && old.schema == collection.schema
    }) else {
        return Ok(false);
    };
    let reference = serde_json::to_value(segment)?;
    let mut matched = false;
    for old_segment in &old.segments {
        if serde_json::to_value(old_segment)? == reference {
            matched = true;
            break;
        }
    }
    if !matched {
        return Ok(false);
    }
    for relative in
        std::iter::once(&segment.path).chain(segment.local_rows.iter().map(|rows| &rows.path))
    {
        let old_file = std::fs::symlink_metadata(old_root.join(relative))?;
        let new_file = std::fs::symlink_metadata(root.join(relative))?;
        if !old_file.is_file() || !new_file.is_file() {
            return Ok(false);
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if old_file.dev() != new_file.dev() || old_file.ino() != new_file.ino() {
                return Ok(false);
            }
        }
        #[cfg(not(unix))]
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn base_payload_sha256(path: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(b"lumen.base.payload-sha256.v1\0");
    hash_delta_component(&mut hasher, b"segment", path)?;
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

/// Hash the two immutable delta files as separate, length-framed components.
/// The framing prevents a payload/row-map boundary ambiguity. Input is read
/// in fixed-size chunks, and the observed count must equal the metadata length.
fn delta_payload_sha256(segment: &Path, rows: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(b"lumen.delta.payload-sha256.v1\0");
    for (tag, path) in [(b"segment".as_slice(), segment), (b"rows".as_slice(), rows)] {
        hash_delta_component(&mut hasher, tag, path)?;
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

#[cfg(test)]
thread_local! { static CHECKSUM_BYTES: std::cell::Cell<u64> = const { std::cell::Cell::new(0) }; }

fn hash_delta_component(hasher: &mut Sha256, tag: &[u8], path: &Path) -> Result<()> {
    let expected = std::fs::metadata(path)
        .with_context(|| format!("inspect delta payload component {}", path.display()))?
        .len();
    hasher.update((tag.len() as u64).to_be_bytes());
    hasher.update(tag);
    hasher.update(expected.to_be_bytes());
    let mut file = File::open(path)
        .with_context(|| format!("open delta payload component {}", path.display()))?;
    let mut observed = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .with_context(|| format!("read delta payload component {}", path.display()))?;
        if count == 0 {
            break;
        }
        observed = observed
            .checked_add(count as u64)
            .ok_or_else(|| anyhow!("delta payload length overflow"))?;
        if observed > expected {
            bail!(
                "delta payload component changed while hashing: {}",
                path.display()
            );
        }
        #[cfg(test)]
        CHECKSUM_BYTES.with(|bytes| bytes.set(bytes.get() + count as u64));
        hasher.update(&buffer[..count]);
    }
    if observed != expected {
        bail!(
            "delta payload component changed while hashing: {}",
            path.display()
        );
    }
    Ok(())
}

fn collection_checkpoint_dir_name(name: &str) -> String {
    name.bytes().map(|byte| format!("{byte:02x}")).collect()
}

fn flat_payload_name(collection: &str, relative: &Path) -> String {
    let mut value = collection
        .bytes()
        .chain(std::iter::once(b'_'))
        .collect::<Vec<_>>();
    for byte in relative.to_string_lossy().bytes() {
        value.extend(format!("{byte:02x}").bytes());
    }
    format!(
        "{}/{}",
        FLAT_PAYLOAD_DIR,
        String::from_utf8(value).expect("ascii")
    )
}

pub(super) fn flat_layout(collection: &CollectionCatalog) -> bool {
    collection
        .segments
        .iter()
        .any(|segment| segment.path.starts_with("payload/"))
}

pub(super) fn collection_schema_path(root: &Path, collection: &CollectionCatalog) -> PathBuf {
    if flat_layout(collection) {
        root.join(flat_payload_name(
            &collection.collection_id,
            Path::new(CHECKPOINT_SCHEMA_FILE),
        ))
    } else {
        root.join(collection_checkpoint_dir_name(&collection.collection_id))
            .join(CHECKPOINT_SCHEMA_FILE)
    }
}

pub(super) fn collection_output_path(
    root: &Path,
    collection: &CollectionCatalog,
    relative: &str,
) -> PathBuf {
    if flat_layout(collection) {
        root.join(flat_payload_name(
            &collection.collection_id,
            Path::new(relative),
        ))
    } else {
        root.join(collection_checkpoint_dir_name(&collection.collection_id))
            .join(relative)
    }
}

pub(super) fn collection_output_relative(collection: &CollectionCatalog, relative: &str) -> String {
    if flat_layout(collection) {
        flat_payload_name(&collection.collection_id, Path::new(relative))
    } else {
        format!(
            "{}/{}",
            collection_checkpoint_dir_name(&collection.collection_id),
            relative
        )
    }
}

fn materialize_flat_reopen_tree(
    root: &Path,
    manifest: &SegmentGenerationManifest,
) -> Result<Vec<PathBuf>> {
    let mut created = Vec::new();
    for collection in &manifest.collections {
        let dir = root.join(collection_checkpoint_dir_name(&collection.collection_id));
        std::fs::create_dir_all(&dir)?;
        created.push(dir.clone());
        let schema = collection_schema_path(root, collection);
        let schema_alias = dir.join(CHECKPOINT_SCHEMA_FILE);
        std::fs::hard_link(&schema, &schema_alias).with_context(|| {
            format!(
                "link v3 collection schema {} -> {}",
                schema.display(),
                schema_alias.display()
            )
        })?;
        let sidecar: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&schema_alias).with_context(|| {
                format!("read materialized schema {}", schema_alias.display())
            })?)?;
        let layout = crate::storage::CheckpointLayout::from_sidecar(&sidecar)?;
        for segment in &collection.segments {
            let relative = if matches!(segment.kind, SegmentKind::Delta) {
                let field = segment
                    .field
                    .as_deref()
                    .ok_or_else(|| anyhow!("flat delta has no field"))?;
                format!(
                    "{}/__delta/{}/{}.lseg",
                    collection_checkpoint_dir_name(&collection.collection_id),
                    collection_checkpoint_dir_name(field),
                    segment.ordinal
                )
            } else {
                match segment.role {
                    SegmentRole::CollectionEids => format!(
                        "{}/_collection.lmeta.lseg",
                        collection_checkpoint_dir_name(&collection.collection_id)
                    ),
                    SegmentRole::Field => format!(
                        "{}/{}.lseg",
                        collection_checkpoint_dir_name(&collection.collection_id),
                        layout.field_stem(
                            segment
                                .field
                                .as_deref()
                                .ok_or_else(|| anyhow!("flat field has no name"))?
                        )
                    ),
                    SegmentRole::VectorEids => format!(
                        "{}/{}.eids.lseg",
                        collection_checkpoint_dir_name(&collection.collection_id),
                        layout.field_stem(
                            segment
                                .field
                                .as_deref()
                                .ok_or_else(|| anyhow!("flat vector field has no name"))?
                        )
                    ),
                }
            };
            let destination = root.join(&relative);
            std::fs::create_dir_all(destination.parent().unwrap())?;
            let source = root.join(&segment.path);
            std::fs::hard_link(&source, &destination).with_context(|| {
                format!(
                    "link v3 segment {} -> {}",
                    source.display(),
                    destination.display()
                )
            })?;
            if let Some(rows) = &segment.local_rows {
                let row_relative = format!(
                    "{}.rows.cbor",
                    relative.strip_suffix(".lseg").unwrap_or(&relative)
                );
                let row_destination = root.join(row_relative);
                let rows_source = root.join(&rows.path);
                std::fs::hard_link(&rows_source, &row_destination).with_context(|| {
                    format!(
                        "link v3 local rows {} -> {}",
                        rows_source.display(),
                        row_destination.display()
                    )
                })?;
            }
        }
    }
    Ok(created)
}

fn flatten_checkpoint_payload(root: &Path, collections: &mut [CollectionCatalog]) -> Result<()> {
    let payload = root.join(FLAT_PAYLOAD_DIR);
    std::fs::create_dir_all(&payload)?;
    for collection in collections {
        let source = root.join(collection_checkpoint_dir_name(&collection.collection_id));
        let mut files = Vec::new();
        let mut pending = vec![source.clone()];
        while let Some(dir) = pending.pop() {
            for entry in std::fs::read_dir(&dir)? {
                let entry = entry?;
                let path = entry.path();
                let metadata = std::fs::symlink_metadata(&path)?;
                if metadata.is_dir() {
                    pending.push(path);
                } else if metadata.is_file() {
                    files.push(path);
                } else {
                    bail!(
                        "checkpoint contains unsupported payload entry: {}",
                        path.display()
                    );
                }
            }
        }
        let mut moved = BTreeMap::new();
        for file in files {
            let relative = file.strip_prefix(&source)?;
            let old = format!(
                "{}/{}",
                collection_checkpoint_dir_name(&collection.collection_id),
                relative.to_string_lossy()
            );
            let target_rel = flat_payload_name(&collection.collection_id, relative);
            let target = root.join(&target_rel);
            std::fs::rename(&file, &target)?;
            moved.insert(old, target_rel);
        }
        for segment in &mut collection.segments {
            if let Some(path) = moved.get(&segment.path) {
                segment.path = path.clone();
            }
            if let Some(rows) = &mut segment.local_rows {
                if let Some(path) = moved.get(&rows.path) {
                    rows.path = path.clone();
                }
            }
        }
        let old_schema = format!(
            "{}/{}",
            collection_checkpoint_dir_name(&collection.collection_id),
            CHECKPOINT_SCHEMA_FILE
        );
        if !moved.contains_key(&old_schema) {
            bail!("collection schema was not staged");
        }
        std::fs::remove_dir_all(source)?;
    }
    Ok(())
}

fn write_generation_manifest(path: &Path, manifest: &SegmentGenerationManifest) -> Result<()> {
    let file = std::fs::File::create(path.join(GENERATION_MANIFEST_FILE))
        .with_context(|| format!("create generation manifest under {}", path.display()))?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, manifest).context("encode generation manifest")?;
    writer.write_all(b"\n")?;
    writer
        .flush()
        .with_context(|| format!("write generation manifest under {}", path.display()))
}

fn read_generation_manifest(path: &Path) -> Result<SegmentGenerationManifest> {
    let manifest_path = path.join(GENERATION_MANIFEST_FILE);
    let metadata = std::fs::symlink_metadata(&manifest_path)
        .with_context(|| format!("inspect generation manifest {}", manifest_path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!(
            "generation manifest must be a regular file: {}",
            manifest_path.display()
        );
    }
    // Probe only the discriminator. Serde skips the collection tree in this
    // pass; the second pass decodes directly into the strict typed catalog.
    // A complete v2 catalog scales with collections and layers, unlike v1's
    // fixed-size envelope. Do not impose v1's byte limit on that catalog or
    // keep both a raw file buffer and an untyped JSON tree in memory.
    #[derive(Deserialize)]
    struct VersionProbe {
        schema_version: Option<u32>,
    }
    let file = std::fs::File::open(&manifest_path)
        .with_context(|| format!("open generation manifest {}", manifest_path.display()))?;
    let mut reader = BufReader::new(file);
    let probe: VersionProbe = serde_json::from_reader(&mut reader)
        .with_context(|| format!("decode generation manifest {}", manifest_path.display()))?;
    reader.seek(SeekFrom::Start(0))?;
    match probe.schema_version {
        Some(1) => {
            if metadata.len() > 4096 {
                bail!(
                    "generation manifest is too large: {}",
                    manifest_path.display()
                );
            }
            let legacy: LegacyV1GenerationManifest =
                serde_json::from_reader(reader).with_context(|| {
                    format!("decode v1 generation manifest {}", manifest_path.display())
                })?;
            if legacy.schema_version != 1 {
                bail!("generation manifest version changed while reading");
            }
            Ok(SegmentGenerationManifest {
                schema_version: legacy.schema_version,
                checkpoint_sequence: legacy.sequence,
                revision: legacy.revision,
                previous: legacy.previous,
                next_collection_generation: 1,
                collections: Vec::new(),
            })
        }
        Some(2) => {
            let manifest: SegmentGenerationManifest = serde_json::from_reader(reader)
                .with_context(|| {
                    format!("decode v2 generation manifest {}", manifest_path.display())
                })?;
            if manifest.schema_version != 2 {
                bail!("generation manifest version changed while reading");
            }
            Ok(manifest)
        }
        Some(3) => {
            let manifest: SegmentGenerationManifest = serde_json::from_reader(reader)
                .with_context(|| {
                    format!("decode v3 generation manifest {}", manifest_path.display())
                })?;
            if manifest.schema_version != 3 {
                bail!("unknown generation layout for schema version 3");
            }
            Ok(manifest)
        }
        Some(version) => bail!("unknown manifest schema version {version}"),
        None => bail!("generation manifest has no schema_version"),
    }
}

type PriorCatalog<'a> = Option<(&'a Path, &'a SegmentGenerationManifest)>;

fn exact_prior_segment_reference(
    collection: &CollectionCatalog,
    segment: &SegmentReference,
    prior: Option<&SegmentGenerationManifest>,
) -> bool {
    prior
        .and_then(|prior| {
            prior.collections.iter().find(|old| {
                old.collection_id == collection.collection_id
                    && old.collection_generation == collection.collection_generation
                    && old.schema_version == collection.schema_version
                    && old.schema == collection.schema
            })
        })
        .is_some_and(|old| {
            old.segments
                .iter()
                .any(|old_segment| old_segment == segment)
        })
}

fn register_checkpoint_inherited_files(
    staged: &mut CurrentGenerationStaging,
    collections: &[CollectionCatalog],
    prior: Option<&SegmentGenerationManifest>,
    capture: &crate::storage::CheckpointCapture,
) -> Result<()> {
    for collection in collections {
        let same_collection = prior.is_some_and(|prior| {
            prior.collections.iter().any(|old| {
                old.collection_id == collection.collection_id
                    && old.collection_generation == collection.collection_generation
                    && old.schema_version == collection.schema_version
                    && old.schema == collection.schema
            })
        });
        if same_collection && capture.reused.contains(&collection.collection_id) {
            let relative = collection_output_relative(collection, CHECKPOINT_SCHEMA_FILE);
            staged
                .inherit_current_file(&relative)
                .with_context(|| format!("inherit CURRENT file {}", relative))?;
        }
        for segment in &collection.segments {
            if exact_prior_segment_reference(collection, segment, prior) {
                staged
                    .inherit_current_file(&segment.path)
                    .with_context(|| format!("inherit CURRENT file {}", segment.path))?;
                if let Some(rows) = &segment.local_rows {
                    staged
                        .inherit_current_file(&rows.path)
                        .with_context(|| format!("inherit CURRENT file {}", rows.path))?;
                }
            }
        }
    }
    Ok(())
}

/// Worker bound for the whole-generation filesystem passes.
///
/// The hard-link and stat passes over a generation are filesystem-metadata
/// bound, not CPU bound: measured on this project's storage, link throughput
/// over a 182-collection / 8k-file generation peaks near four concurrent
/// workers (3.3s serial, 1.7s at four) and degrades again past it (2.5s at
/// six), so an unbounded pool is slower than this bound, not faster.
const GENERATION_LINK_WORKERS: usize = 4;

/// Worker bound for the validation pass over a staged generation. It is not
/// the link bound: validating a collection reads and decodes its checkpoint
/// schema and opens every base segment it declares, so it keeps more than four
/// workers busy where a pure link pass does not.
const GENERATION_VALIDATE_WORKERS: usize = 8;

/// Apply `task` to every item on a bounded worker set, returning the results in
/// input order.
///
/// Order is the contract: callers fold the results in input order, so the
/// first error reported is the same one a sequential pass would have reported,
/// whichever worker happened to observe it first.
pub(super) fn map_generation_pass<T: Sync, R: Send>(
    items: &[T],
    workers: usize,
    task: impl Fn(&T) -> Result<R> + Sync,
) -> Vec<Result<R>> {
    if items.len() < 2 {
        return items.iter().map(&task).collect();
    }
    let next = std::sync::atomic::AtomicUsize::new(0);
    let slots: Vec<Mutex<Option<Result<R>>>> = items.iter().map(|_| Mutex::new(None)).collect();
    let workers = workers.max(1).min(items.len());
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let Some(item) = items.get(index) else {
                    return;
                };
                let result = task(item);
                *slots[index].lock().unwrap_or_else(|p| p.into_inner()) = Some(result);
            });
        }
    });
    slots
        .into_iter()
        .map(|slot| {
            slot.into_inner()
                .unwrap_or_else(|p| p.into_inner())
                .expect("every item is assigned to exactly one worker")
        })
        .collect()
}

/// Validate one direct entry of a generation directory: either its manifest
/// file, or one collection subtree with its checkpoint schema and every base
/// segment the schema declares.
///
/// Returns whether the entry was a collection. Entries are disjoint subtrees,
/// which is what lets [`validate_generation_layout_with_prior`] run this pass
/// on a bounded worker set without weakening any check it makes.
fn validate_generation_entry(
    path: &Path,
    record: &GenerationRecord,
    references: &BTreeMap<&str, &SegmentReference>,
    v2: bool,
    flat: bool,
) -> Result<bool> {
    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        bail!("generation contains a symlink: {}", path.display());
    }
    if path.file_name().and_then(|name| name.to_str()) == Some(GENERATION_MANIFEST_FILE) {
        if record.legacy || !metadata.is_file() {
            bail!("invalid generation manifest entry: {}", path.display());
        }
        return Ok(false);
    }
    if !metadata.is_dir() {
        bail!("unexpected generation entry: {}", path.display());
    }
    validate_real_tree(path)?;
    if flat && path.file_name().and_then(|name| name.to_str()) == Some(FLAT_PAYLOAD_DIR) {
        return Ok(false);
    }

    let schema_path = path.join(CHECKPOINT_SCHEMA_FILE);
    let schema_metadata = std::fs::symlink_metadata(&schema_path)
        .with_context(|| format!("inspect checkpoint schema {}", schema_path.display()))?;
    if schema_metadata.file_type().is_symlink() || !schema_metadata.is_file() {
        bail!(
            "checkpoint schema must be a regular file: {}",
            schema_path.display()
        );
    }
    let schema: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&schema_path)
            .with_context(|| format!("read checkpoint schema {}", schema_path.display()))?,
    )
    .with_context(|| format!("decode checkpoint schema {}", schema_path.display()))?;
    let applied_seq = schema
        .get("applied_seq")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| {
            anyhow!(
                "checkpoint schema has no applied_seq: {}",
                schema_path.display()
            )
        })?;
    if (v2 && applied_seq > record.sequence) || (!v2 && applied_seq != record.sequence) {
        bail!(
            "checkpoint schema {} has sequence {applied_seq}, expected {}",
            schema_path.display(),
            record.sequence
        );
    }
    if v2 {
        let layout = crate::storage::CheckpointLayout::from_sidecar(&schema)?;
        let fields: BTreeMap<String, crate::types::FieldSpec> = serde_json::from_value(
            schema
                .get("fields")
                .cloned()
                .ok_or_else(|| anyhow!("checkpoint schema has no fields"))?,
        )?;
        let mut bases = vec![(path.join("_collection.lmeta.lseg"), false)];
        for (name, spec) in fields {
            let stem = layout.field_stem(&name);
            let vector = spec.field_type == crate::types::FieldType::Vector;
            bases.push((path.join(format!("{stem}.lseg")), vector));
            if vector {
                bases.push((path.join(format!("{stem}.eids.lseg")), false));
            }
        }
        for (file, vector) in bases {
            let segment = crate::segment::SegmentReader::open(&file)
                .with_context(|| format!("catalogued segment is missing: {}", file.display()))?;
            // Only shipped raw-layout vectors used row count as watermark.
            let legacy_vector_header = layout == crate::storage::CheckpointLayout::Legacy
                && vector
                && segment.applied_seq() == segment.n_docs() as u64;
            let relative = file
                .strip_prefix(&record.path)?
                .to_str()
                .ok_or_else(|| anyhow!("non-UTF8 base path"))?;
            let reference = *references
                .get(relative)
                .ok_or_else(|| anyhow!("base segment has no catalog reference"))?;
            // Original v2 bases inherit their collection sidecar cut.
            // Compacted bases record their own, possibly newer, cut.
            let catalog_sequence = reference.applied_seq.unwrap_or(applied_seq);
            if !legacy_vector_header
                && (segment.applied_seq() != catalog_sequence || catalog_sequence > record.sequence)
            {
                bail!("segment sequence does not match checkpoint catalog");
            }
        }
    }
    Ok(true)
}

fn validate_generation_layout(record: &GenerationRecord) -> Result<usize> {
    validate_generation_layout_with_prior(record, None)
}

fn validate_generation_layout_with_prior(
    record: &GenerationRecord,
    prior: PriorCatalog<'_>,
) -> Result<usize> {
    let manifest = if record.legacy {
        None
    } else {
        Some(read_generation_manifest(&record.path)?)
    };
    let v2 = manifest.as_ref().is_some_and(|manifest| {
        matches!(
            manifest.schema_version,
            GENERATION_MANIFEST_V2 | GENERATION_MANIFEST_V3
        )
    });
    let flat = manifest
        .as_ref()
        .is_some_and(|manifest| manifest.schema_version == GENERATION_MANIFEST_V3);
    // Validate names before using the catalog to resolve a physical base.
    // A malformed reference must not be mistaken for an omitted base.
    if v2 {
        for reference in manifest
            .as_ref()
            .into_iter()
            .flat_map(|manifest| &manifest.collections)
            .flat_map(|collection| &collection.segments)
        {
            if Path::new(&reference.path).is_absolute()
                || reference
                    .path
                    .split('/')
                    .any(|part| matches!(part, "" | "." | ".."))
            {
                bail!(
                    "segment reference path escapes generation: {}",
                    reference.path
                );
            }
        }
    }
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(&record.path)
        .with_context(|| format!("read generation {}", record.path.display()))?
    {
        entries.push(entry?.path());
    }
    entries.sort();

    // One index over the whole catalog: resolving each declared base by a
    // linear scan of every collection's segments made this pass quadratic in
    // the number of collections.
    let mut references: BTreeMap<&str, &SegmentReference> = BTreeMap::new();
    for reference in manifest
        .as_ref()
        .into_iter()
        .flat_map(|manifest| &manifest.collections)
        .flat_map(|collection| &collection.segments)
    {
        references
            .entry(reference.path.as_str())
            .or_insert(reference);
    }
    let mut collections = 0usize;
    for outcome in map_generation_pass(&entries, GENERATION_VALIDATE_WORKERS, |path| {
        validate_generation_entry(path, record, &references, v2, flat)
    }) {
        if outcome? {
            collections += 1;
        }
    }

    if !record.legacy {
        if let Some(previous) = &record.previous {
            if !is_older_predecessor(previous.as_str(), record.sequence, record.revision) {
                bail!(
                    "generation {} has non-predecessor link {}",
                    record.name,
                    previous
                );
            }
        }
        let manifest = read_generation_manifest(&record.path)?;
        let expected_previous = record.previous.as_ref().map(GenerationName::as_str);
        if (manifest.schema_version != 1
            && !matches!(
                manifest.schema_version,
                GENERATION_MANIFEST_V2 | GENERATION_MANIFEST_V3
            ))
            || manifest.checkpoint_sequence != record.sequence
            || manifest.revision != record.revision
            || manifest.previous.as_deref() != expected_previous
        {
            bail!(
                "generation {} manifest does not match its validated record",
                record.name
            );
        }
        if matches!(
            manifest.schema_version,
            GENERATION_MANIFEST_V2 | GENERATION_MANIFEST_V3
        ) {
            validate_catalog_references_with_prior(&record.path, &manifest, prior)?;
        }
    }
    Ok(collections)
}

fn validate_real_tree(root: &Path) -> Result<()> {
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let metadata = std::fs::symlink_metadata(&directory)
            .with_context(|| format!("inspect checkpoint path {}", directory.display()))?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            bail!(
                "checkpoint directory must be a real directory: {}",
                directory.display()
            );
        }

        let mut entries = std::fs::read_dir(&directory)
            .with_context(|| format!("read checkpoint directory {}", directory.display()))?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<std::io::Result<Vec<_>>>()?;
        entries.sort();
        for path in entries {
            let metadata = std::fs::symlink_metadata(&path)
                .with_context(|| format!("inspect checkpoint path {}", path.display()))?;
            if metadata.file_type().is_symlink() {
                bail!("checkpoint contains a symlink: {}", path.display());
            }
            if metadata.is_dir() {
                pending.push(path);
            } else if !metadata.is_file() {
                bail!(
                    "checkpoint contains a special filesystem entry: {}",
                    path.display()
                );
            }
        }
    }
    Ok(())
}

fn shared_save_gate(root: &Path) -> Result<Arc<SaveGate>> {
    static ROOT_LOCKS: OnceLock<Mutex<HashMap<PathBuf, Weak<SaveGate>>>> = OnceLock::new();

    let registry = ROOT_LOCKS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut registry = registry
        .lock()
        .map_err(|_| anyhow!("segment root-lock registry poisoned"))?;
    registry.retain(|_, lock| lock.strong_count() > 0);
    if let Some(lock) = registry.get(root).and_then(Weak::upgrade) {
        return Ok(lock);
    }

    let gate = Arc::new(SaveGate::default());
    registry.insert(root.to_path_buf(), Arc::downgrade(&gate));
    Ok(gate)
}

fn shared_pending_frozen(root: &Path) -> Result<Arc<Mutex<Option<PendingFrozenCheckpoint>>>> {
    static ROOT_PENDING: OnceLock<
        Mutex<HashMap<PathBuf, Weak<Mutex<Option<PendingFrozenCheckpoint>>>>>,
    > = OnceLock::new();
    let registry = ROOT_PENDING.get_or_init(|| Mutex::new(HashMap::new()));
    let mut registry = registry
        .lock()
        .map_err(|_| anyhow!("segment pending-frozen registry poisoned"))?;
    registry.retain(|_, pending| pending.strong_count() > 0);
    if let Some(pending) = registry.get(root).and_then(Weak::upgrade) {
        return Ok(pending);
    }
    let pending = Arc::new(Mutex::new(None));
    registry.insert(root.to_path_buf(), Arc::downgrade(&pending));
    Ok(pending)
}

fn sync_directory(path: &Path) -> std::io::Result<()> {
    OpenOptions::new().read(true).open(path)?.sync_all()
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    #[test]
    fn graph_cache_without_current_cannot_initialize_or_clean_a_root() {
        for kind in 0..3 {
            let root = tempfile::tempdir().unwrap();
            let outside = tempfile::tempdir().unwrap();
            let cache = root.path().join(super::HNSW_GRAPH_CACHE_DIR);
            match kind {
                0 => std::fs::create_dir(&cache).unwrap(),
                1 => std::fs::write(&cache, b"damaged cache").unwrap(),
                _ => std::os::unix::fs::symlink(outside.path(), &cache).unwrap(),
            }
            let partial = root.path().join(".gen-7.tmp");
            std::fs::create_dir(&partial).unwrap();
            std::fs::write(partial.join("sentinel"), b"keep until authority is known").unwrap();
            let result = super::SegmentRdbStore::new(root.path());
            assert!(
                result.is_err(),
                "a cache cannot authorize empty initialization without CURRENT (kind {kind})"
            );
            assert!(!root.path().join("CURRENT").exists());
            assert_eq!(
                std::fs::read(partial.join("sentinel")).unwrap(),
                b"keep until authority is known"
            );
            assert_eq!(std::fs::read_dir(outside.path()).unwrap().count(), 0);
        }
    }

    #[test]
    fn graph_cache_cleanup_does_not_remove_unrelated_root_entries() {
        let root = tempfile::tempdir().unwrap();
        let store = super::SegmentRdbStore::new(root.path()).unwrap();
        let cache = root.path().join(super::HNSW_GRAPH_CACHE_DIR);
        let obsolete = cache.join("a".repeat(64));
        std::fs::create_dir_all(&obsolete).unwrap();
        std::fs::write(obsolete.join("graph.hnsw.graph"), b"obsolete cache").unwrap();
        let retained = cache.join("unknown-entry");
        std::fs::create_dir(&retained).unwrap();
        std::fs::write(retained.join("sentinel"), b"keep").unwrap();
        assert_eq!(
            store
                .save_hnsw_graph_caches(&crate::storage::Engine::new())
                .unwrap(),
            0
        );
        assert!(!obsolete.exists());
        assert_eq!(std::fs::read(retained.join("sentinel")).unwrap(), b"keep");
        assert!(root.path().join("CURRENT").is_file());
    }

    use super::*;

    struct DiagnosticEnvironment {
        _lock: std::sync::MutexGuard<'static, ()>,
        previous: Option<std::ffi::OsString>,
    }

    impl DiagnosticEnvironment {
        fn set(enabled: bool) -> Self {
            static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
            let lock = LOCK
                .get_or_init(|| Mutex::new(()))
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let previous = std::env::var_os("LUMEN_PERF_DIAGNOSTIC");
            if enabled {
                std::env::set_var("LUMEN_PERF_DIAGNOSTIC", "1");
            } else {
                std::env::remove_var("LUMEN_PERF_DIAGNOSTIC");
            }
            Self {
                _lock: lock,
                previous,
            }
        }
    }

    impl Drop for DiagnosticEnvironment {
        fn drop(&mut self) {
            if let Some(previous) = self.previous.take() {
                std::env::set_var("LUMEN_PERF_DIAGNOSTIC", previous);
            } else {
                std::env::remove_var("LUMEN_PERF_DIAGNOSTIC");
            }
        }
    }

    #[derive(Clone, Default)]
    struct DiagnosticTraceWriter(Arc<Mutex<Vec<u8>>>);

    struct DiagnosticTraceWriterGuard(Arc<Mutex<Vec<u8>>>);

    impl std::io::Write for DiagnosticTraceWriterGuard {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl<'writer> tracing_subscriber::fmt::MakeWriter<'writer> for DiagnosticTraceWriter {
        type Writer = DiagnosticTraceWriterGuard;

        fn make_writer(&'writer self) -> Self::Writer {
            DiagnosticTraceWriterGuard(self.0.clone())
        }
    }

    impl DiagnosticTraceWriter {
        fn records(&self) -> Vec<serde_json::Value> {
            String::from_utf8(self.0.lock().unwrap().clone())
                .unwrap()
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect()
        }
    }

    fn cohort_catalog(root: &Path, fields: &[(&str, usize, usize)]) -> CollectionCatalog {
        let mut segments = Vec::new();
        for (field, depth, bytes) in fields {
            let base_path = format!("{field}.base.lseg");
            std::fs::write(root.join(&base_path), vec![b'b'; 1_000_000]).unwrap();
            segments.push(SegmentReference {
                role: SegmentRole::Field,
                field: Some((*field).to_owned()),
                ordinal: 0,
                kind: SegmentKind::Base,
                format: SegmentFormat::LsegV1,
                path: base_path,
                local_rows: None,
                applied_seq: None,
                payload_sha256: None,
            });
            for ordinal in 1..=*depth {
                let path = format!("{field}.{ordinal}.delta.lseg");
                std::fs::write(root.join(&path), vec![b'd'; *bytes]).unwrap();
                segments.push(SegmentReference {
                    role: SegmentRole::Field,
                    field: Some((*field).to_owned()),
                    ordinal: ordinal as u32,
                    kind: SegmentKind::Delta,
                    format: SegmentFormat::LsegV1,
                    path,
                    local_rows: None,
                    applied_seq: None,
                    payload_sha256: None,
                });
            }
        }
        CollectionCatalog {
            collection_id: "cohort".to_owned(),
            collection_generation: 1,
            schema_version: 1,
            data_version: 1,
            schema: serde_json::json!({}),
            segments,
        }
    }

    #[test]
    fn selector_keeps_small_fourteen_field_cohort_in_one_bounded_job() {
        // Fourteen fields in the durable workload share one checkpoint cut.
        // Splitting a small cohort repeats the whole-root publication work,
        // even when all selected inputs fit inside the existing byte budget.
        for (count, delta_bytes, expected) in [
            (14, 1, 14),
            (17, 1, 16),
            (14, 1024 * 1024, 12),
            (9, 2 * 1024 * 1024, 8),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let names: Vec<_> = (0..count).map(|index| format!("f{index:02}")).collect();
            let fields: Vec<_> = names
                .iter()
                .map(|name| (name.as_str(), 4, delta_bytes))
                .collect();
            let catalog = cohort_catalog(dir.path(), &fields);
            // Sparse base files keep these fixtures delta-only without
            // allocating or reading a large corpus. Selection uses file sizes.
            for name in &names {
                OpenOptions::new()
                    .write(true)
                    .open(dir.path().join(format!("{name}.base.lseg")))
                    .unwrap()
                    .set_len(64 * 1024 * 1024)
                    .unwrap();
            }
            let selected = select_staged_delta_window(dir.path(), &[catalog]).unwrap();
            assert_eq!(
                selected.len(),
                expected,
                "count={count}, delta_bytes={delta_bytes}: small cohorts share one publication; large cohorts retain their old bound"
            );
            assert_eq!(
                selected
                    .iter()
                    .map(|candidate| candidate.field.as_str())
                    .collect::<Vec<_>>(),
                names[..expected]
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
                "field ordering stays deterministic"
            );
            assert!(selected
                .iter()
                .all(|candidate| { !candidate.includes_base && candidate.inputs.len() == 2 }));
            if selected.len() > 8 {
                assert!(
                    selected.len() * 2 * delta_bytes <= 24 * 1024 * 1024,
                    "an expanded cohort must fit the unchanged total byte budget"
                );
            }
        }
    }

    #[test]
    fn selector_bounds_cohort_by_fields_bytes_and_priority() {
        let dir = tempfile::tempdir().unwrap();
        let tiny = cohort_catalog(
            dir.path(),
            &[
                ("g", 4, 1),
                ("a", 4, 1),
                ("f", 4, 1),
                ("b", 4, 1),
                ("e", 4, 1),
                ("c", 4, 1),
                ("d", 4, 1),
            ],
        );
        let selected = select_staged_delta_window(dir.path(), &[tiny]).unwrap();
        assert_eq!(selected.len(), 7);
        assert_eq!(
            selected
                .iter()
                .map(|candidate| candidate.field.as_str())
                .collect::<Vec<_>>(),
            vec!["a", "b", "c", "d", "e", "f", "g"]
        );

        let dir = tempfile::tempdir().unwrap();
        let ten = cohort_catalog(
            dir.path(),
            &[
                ("a", 4, 2_500_000),
                ("b", 4, 2_500_000),
                ("c", 4, 2_500_000),
            ],
        );
        assert_eq!(
            select_staged_delta_window(dir.path(), &[ten])
                .unwrap()
                .len(),
            2
        );

        let dir = tempfile::tempdir().unwrap();
        let mixed = cohort_catalog(
            dir.path(),
            &[("z", 6, 1), ("a", 7, 1), ("b", 7, 1), ("c", 3, 1)],
        );
        let selected = select_staged_delta_window(dir.path(), &[mixed]).unwrap();
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].field, "a");
        assert_eq!(selected[1].field, "b");

        let dir = tempfile::tempdir().unwrap();
        let first_large = cohort_catalog(dir.path(), &[("a", 4, 6_250_000), ("b", 4, 1)]);
        let selected = select_staged_delta_window(dir.path(), &[first_large]).unwrap();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].field, "a");
    }

    struct BlockingBeforeEncode {
        state: Mutex<(usize, bool)>,
        wake: std::sync::Condvar,
    }

    impl MergeObserver for BlockingBeforeEncode {
        fn observe(&self, phase: MergePhase) -> std::io::Result<()> {
            assert_eq!(phase, MergePhase::BeforeEncode);
            let mut state = self.state.lock().unwrap();
            state.0 += 1;
            self.wake.notify_all();
            if state.0 == 1 {
                while !state.1 {
                    state = self.wake.wait(state).unwrap();
                }
            }
            Ok(())
        }
    }

    fn test_merge_candidate(field: &str) -> StagedMergeCandidate {
        let segment = SegmentReference {
            role: SegmentRole::Field,
            field: Some(field.to_owned()),
            ordinal: 1,
            kind: SegmentKind::Delta,
            format: SegmentFormat::LsegV1,
            path: format!("{field}.delta.lseg"),
            local_rows: None,
            applied_seq: Some(1),
            payload_sha256: None,
        };
        StagedMergeCandidate {
            collection_index: 0,
            collection_id: "test".to_owned(),
            field: field.to_owned(),
            inputs: vec![segment.clone()],
            base: SegmentReference {
                kind: SegmentKind::Base,
                path: format!("{field}.base.lseg"),
                ..segment
            },
            includes_base: false,
        }
    }

    #[test]
    fn sequential_candidate_encoding_waits_before_starting_the_next_candidate() {
        let observer = Arc::new(BlockingBeforeEncode {
            state: Mutex::new((0, false)),
            wake: std::sync::Condvar::new(),
        });
        let encoded = Arc::new(Mutex::new(Vec::new()));
        let candidates = vec![test_merge_candidate("alpha"), test_merge_candidate("beta")];
        let worker_observer = Arc::clone(&observer);
        let worker_encoded = Arc::clone(&encoded);
        let worker = std::thread::spawn(move || {
            encode_staged_candidates_in_order(candidates, worker_observer.as_ref(), |candidate| {
                let field = candidate.field;
                worker_encoded.lock().unwrap().push(field.clone());
                Ok(field)
            })
        });

        let mut state = observer.state.lock().unwrap();
        while state.0 == 0 {
            state = observer.wake.wait(state).unwrap();
        }
        assert_eq!(state.0, 1);
        assert!(encoded.lock().unwrap().is_empty());
        state.1 = true;
        observer.wake.notify_all();
        drop(state);

        assert_eq!(worker.join().unwrap().unwrap(), vec!["alpha", "beta"]);
        assert_eq!(encoded.lock().unwrap().as_slice(), ["alpha", "beta"]);
    }

    #[test]
    fn first_ordinary_save_publishes_from_an_empty_current() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "first", "value");

        store.save(&engine, 1).unwrap();

        assert!(matches!(
            store.generations.read_current().unwrap(),
            CurrentTarget::Generation(_)
        ));
    }

    #[test]
    fn flat_layout_writes_and_reopens_multiple_collections() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        engine.create_collection("v", kw_schema()).unwrap();
        index_kw_in(&engine, "u", "u1", "one");
        index_kw_in(&engine, "v", "v1", "two");
        store.save(&engine, 1).unwrap();
        let (cold, _) = store.load_latest().unwrap().unwrap();
        assert_eq!(cold.list_collections().unwrap(), vec!["u", "v"]);
    }

    #[test]
    fn flat_layout_second_checkpoint_and_cold_restart_completes() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        engine.create_collection("v", kw_schema()).unwrap();
        index_kw_in(&engine, "u", "u1", "one");
        index_kw_in(&engine, "v", "v1", "two");
        store.save(&engine, 1).unwrap();
        index_kw_in(&engine, "u", "u2", "three");
        store.save(&engine, 2).unwrap();
        let (cold, sequence) = store.load_latest().unwrap().unwrap();
        assert_eq!(sequence, 2);
        assert_eq!(cold.list_collections().unwrap(), vec!["u", "v"]);
    }

    #[test]
    fn flat_layout_rejects_more_than_sixteen_deltas_per_field() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw_in(&engine, "u", "u1", "one");
        store.save(&engine, 1).unwrap();
        index_kw_in(&engine, "u", "u2", "two");
        store.save(&engine, 2).unwrap();

        let generation = current_generation(&store);
        let generation_path = dir.path().join(generation.as_str());
        let mut manifest = read_generation_manifest(&generation_path).unwrap();
        let collection = manifest
            .collections
            .iter_mut()
            .find(|collection| collection.collection_id == "u")
            .unwrap();
        let template = collection
            .segments
            .iter()
            .find(|segment| matches!(segment.kind, SegmentKind::Delta))
            .cloned()
            .expect("second save must retain a delta for the cap fixture");
        let template_rows = template.local_rows.clone().unwrap();
        let template_reader =
            crate::segment::SegmentReader::open(&generation_path.join(&template.path)).unwrap();
        assert_eq!(template_reader.n_docs(), template_rows.count);
        assert_eq!(template_reader.applied_seq(), template.applied_seq.unwrap());
        let template_applied_seq = template.applied_seq.unwrap();
        let template_row_count = template_rows.count;
        assert!(template_applied_seq <= manifest.checkpoint_sequence);
        let existing_delta_count = collection
            .segments
            .iter()
            .filter(|segment| matches!(segment.kind, SegmentKind::Delta))
            .count();
        let max_ordinal = collection
            .segments
            .iter()
            .filter(|segment| matches!(segment.kind, SegmentKind::Delta))
            .map(|segment| segment.ordinal)
            .max()
            .unwrap();
        let template_segment_parent = Path::new(&template.path)
            .parent()
            .expect("v2 delta must have a directory parent")
            .to_owned();
        let template_rows_parent = Path::new(&template_rows.path)
            .parent()
            .expect("v2 local rows must have a directory parent")
            .to_owned();
        for ordinal in (max_ordinal + 1)..=(max_ordinal + (17 - existing_delta_count) as u32) {
            let mut delta = template.clone();
            delta.ordinal = ordinal;
            delta.path = template_segment_parent
                .join(format!("{ordinal}.lseg"))
                .to_string_lossy()
                .into_owned();
            let mut rows = template_rows.clone();
            rows.path = template_rows_parent
                .join(format!("{ordinal}.rows.cbor"))
                .to_string_lossy()
                .into_owned();
            delta.applied_seq = Some(template_applied_seq);
            rows.count = template_row_count;
            delta.local_rows = Some(rows.clone());
            std::fs::create_dir_all(generation_path.join(&delta.path).parent().unwrap()).unwrap();
            std::fs::create_dir_all(generation_path.join(&rows.path).parent().unwrap()).unwrap();
            let delta_path = generation_path.join(&delta.path);
            if !delta_path.exists() {
                std::fs::hard_link(generation_path.join(&template.path), &delta_path).unwrap();
            }
            let rows_path = generation_path.join(&rows.path);
            if !rows_path.exists() {
                std::fs::hard_link(generation_path.join(&template_rows.path), &rows_path).unwrap();
            }
            collection.segments.push(delta);
        }
        assert_eq!(
            collection
                .segments
                .iter()
                .filter(|segment| matches!(segment.kind, SegmentKind::Delta))
                .count(),
            17
        );
        let error = validate_catalog_references(&generation_path, &manifest).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("field exceeds sixteen delta segments"),
            "unexpected v3 delta-cap error: {error:#}"
        );
    }

    struct FailInheritedPayload {
        relative_path: PathBuf,
        hits: std::sync::atomic::AtomicUsize,
    }

    impl FailureInjector for FailInheritedPayload {
        fn check(&self, point: &storage_durable::FailurePoint) -> std::io::Result<()> {
            if point.step == storage_durable::CommitStep::SyncFile
                && point.relative_path == self.relative_path
            {
                self.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                return Err(std::io::Error::other("retained payload must be synced"));
            }
            Ok(())
        }
    }

    #[test]
    fn required_save_syncs_a_retained_payload_that_ordinary_save_skips() {
        let dir = tempfile::tempdir().unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "first", "value");
        let initial = SegmentRdbStore::new(dir.path()).unwrap();
        initial.save(&engine, 1).unwrap();
        let initial_name = current_generation(&initial);
        let manifest = read_generation_manifest(&dir.path().join(initial_name.as_str())).unwrap();
        let retained = PathBuf::from(manifest.collections[0].segments[0].path.clone());
        let injector = Arc::new(FailInheritedPayload {
            relative_path: retained,
            hits: std::sync::atomic::AtomicUsize::new(0),
        });
        let store =
            SegmentRdbStore::new_with_failure_injector(dir.path(), injector.clone()).unwrap();

        index_kw(&engine, "second", "value-two");
        store.save(&engine, 2).unwrap();
        assert_ne!(current_generation(&store), initial_name);
        assert_eq!(
            store.load_current_generation().unwrap().unwrap().sequence,
            2
        );
        assert_eq!(
            injector.hits.load(std::sync::atomic::Ordering::Relaxed),
            0,
            "ordinary current-derived save must skip a registered retained payload"
        );
        assert!(
            store.save_required(&engine, 3).is_err(),
            "restore/import save must keep generic full-file durability"
        );
        assert_eq!(
            injector.hits.load(std::sync::atomic::Ordering::Relaxed),
            1,
            "required generic save must sync the retained payload"
        );
    }

    struct FailOnce(Mutex<Option<storage_durable::CommitStep>>);

    impl FailureInjector for FailOnce {
        fn check(&self, point: &storage_durable::FailurePoint) -> std::io::Result<()> {
            let mut armed = self.0.lock().unwrap();
            if *armed == Some(point.step) {
                *armed = None;
                return Err(std::io::Error::other("one pending-checkpoint failure"));
            }
            Ok(())
        }
    }

    fn pending_retry_preserves_cut_after_failure(step: storage_durable::CommitStep) {
        let dir = tempfile::tempdir().unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "doc", "old");
        SegmentRdbStore::new(dir.path())
            .unwrap()
            .save(&engine, 1)
            .unwrap();

        index_kw(&engine, "doc", "captured");
        let store = SegmentRdbStore::new_with_failure_injector(
            dir.path(),
            Arc::new(FailOnce(Mutex::new(Some(step)))),
        )
        .unwrap();
        assert!(store.save(&engine, 2).is_err());
        index_kw(&engine, "doc", "newer");
        store.save(&engine, 2).unwrap();
        let (replayed, sequence) = store.load_latest().unwrap().unwrap();
        assert_eq!(sequence, 2);
        assert!(has_keyword(&replayed, "captured"));
        assert!(!has_keyword(&replayed, "newer"));

        store.save(&engine, 3).unwrap();
        let (later, sequence) = store.load_latest().unwrap().unwrap();
        assert_eq!(sequence, 3);
        assert!(has_keyword(&later, "newer"));
    }

    #[test]
    fn pending_frozen_syncfile_retry_keeps_captured_payload() {
        pending_retry_preserves_cut_after_failure(storage_durable::CommitStep::SyncFile);
    }

    #[test]
    fn pending_frozen_renamecurrent_retry_keeps_captured_payload() {
        pending_retry_preserves_cut_after_failure(storage_durable::CommitStep::RenameCurrent);
    }

    #[test]
    fn pending_retry_then_requested_newer_cut_publishes_in_order() {
        let dir = tempfile::tempdir().unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "doc", "old");
        SegmentRdbStore::new(dir.path())
            .unwrap()
            .save(&engine, 1)
            .unwrap();
        index_kw(&engine, "doc", "captured");
        let store = SegmentRdbStore::new_with_failure_injector(
            dir.path(),
            Arc::new(FailOnce(Mutex::new(Some(
                storage_durable::CommitStep::SyncFile,
            )))),
        )
        .unwrap();
        assert!(store.save(&engine, 2).is_err());
        let retained = store
            .pending_frozen_identity()
            .expect("actual frozen payload retained");
        index_kw(&engine, "doc", "newer");
        assert_eq!(store.save_with_sequence(&engine, 3).unwrap(), 3);
        assert!(store.pending_frozen_identity().is_none());
        assert!(store
            .generation_entries()
            .unwrap()
            .iter()
            .any(|(name, _)| parse_revision_name(name.as_str()).is_some_and(|(seq, _)| seq == 2)));
        assert_ne!(retained, 0);
        let (latest, sequence) = store.load_latest().unwrap().unwrap();
        assert_eq!(sequence, 3);
        assert!(has_keyword(&latest, "newer"));
    }

    #[test]
    fn diagnostic_reused_cut_then_fresh_successor_uses_ordered_passes() {
        use tracing_subscriber::prelude::*;

        let _environment = DiagnosticEnvironment::set(true);
        let writer = DiagnosticTraceWriter::default();
        let subscriber = tracing_subscriber::registry().with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_ansi(false)
                .with_writer(writer.clone()),
        );
        let _guard = tracing::subscriber::set_default(subscriber);
        let dir = tempfile::tempdir().unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "doc", "old");
        SegmentRdbStore::new(dir.path())
            .unwrap()
            .save(&engine, 1)
            .unwrap();
        index_kw(&engine, "doc", "captured");
        let store = SegmentRdbStore::new_with_failure_injector(
            dir.path(),
            Arc::new(FailOnce(Mutex::new(Some(
                storage_durable::CommitStep::SyncFile,
            )))),
        )
        .unwrap();
        assert!(store.save(&engine, 2).is_err());
        index_kw(&engine, "doc", "newer");
        let context = CheckpointDiagnosticContext::new("periodic", Some(81));
        assert_eq!(
            store
                .save_with_sequence_diagnostic_context(&engine, 3, Some(context))
                .unwrap(),
            3
        );
        drop(_guard);

        let phases: Vec<_> = writer
            .records()
            .into_iter()
            .filter(|record| record["fields"]["event"] == "segment_checkpoint_diagnostic_phase")
            .filter(|record| {
                matches!(
                    record["fields"]["phase"].as_str(),
                    Some("freeze_completed" | "publish_completed")
                )
            })
            .collect();
        assert_eq!(
            phases
                .iter()
                .map(|record| record["fields"]["phase"].as_str().unwrap())
                .collect::<Vec<_>>(),
            [
                "freeze_completed",
                "publish_completed",
                "freeze_completed",
                "publish_completed",
            ]
        );
        assert_eq!(phases[0]["fields"]["checkpoint_pass"], 1);
        assert_eq!(phases[0]["fields"]["frozen_cut_reused"], true);
        assert_eq!(phases[0]["fields"]["frozen_cut_bytes"], 0);
        assert_eq!(phases[1]["fields"]["checkpoint_pass"], 1);
        assert_eq!(phases[2]["fields"]["checkpoint_pass"], 2);
        assert_eq!(phases[2]["fields"]["frozen_cut_reused"], false);
        assert!(phases[2]["fields"]["frozen_cut_bytes"].as_u64().is_some());
        assert_eq!(phases[3]["fields"]["checkpoint_pass"], 2);
    }

    #[test]
    fn restored_epoch_discards_old_pending_before_foreign_engine_can_save() {
        let dir = tempfile::tempdir().unwrap();
        let old = Arc::new(Engine::new());
        old.create_collection("u", kw_schema()).unwrap();
        index_kw(&old, "doc", "old");
        SegmentRdbStore::new(dir.path())
            .unwrap()
            .save(&old, 1)
            .unwrap();
        index_kw(&old, "doc", "captured");
        let store = SegmentRdbStore::new_with_failure_injector(
            dir.path(),
            Arc::new(FailOnce(Mutex::new(Some(
                storage_durable::CommitStep::SyncFile,
            )))),
        )
        .unwrap();
        assert!(store.save(&old, 2).is_err());
        let retained = store
            .pending_frozen_identity()
            .expect("pending frozen payload");

        let replacement = Engine::new();
        replacement.create_collection("u", kw_schema()).unwrap();
        index_kw(&replacement, "doc", "restored");
        old.restore(replacement.snapshot().unwrap()).unwrap();
        let foreign = Arc::new(Engine::new());
        foreign.create_collection("u", kw_schema()).unwrap();
        index_kw(&foreign, "doc", "foreign");
        store.save(&foreign, 3).unwrap();
        assert!(store.pending_frozen_identity().is_none());
        assert_ne!(retained, 0);
        let (latest, _) = store.load_latest().unwrap().unwrap();
        assert!(has_keyword(&latest, "foreign"));
        assert!(!has_keyword(&latest, "captured"));
    }

    #[test]
    fn v2_delta_rejects_missing_integrity_fields() {
        let (_dir, _store, _engine, root, manifest) = two_keyword_deltas();
        let mut missing = manifest.clone();
        let delta = missing.collections[0]
            .segments
            .iter_mut()
            .filter(|segment| matches!(segment.kind, SegmentKind::Delta))
            .max_by_key(|segment| segment.ordinal)
            .unwrap();
        delta.applied_seq = None;
        delta.payload_sha256 = None;
        assert!(validate_catalog_references(&root, &missing).is_err());
    }

    #[test]
    fn unchanged_checkpoint_links_deltas_without_reading_their_payload_again() {
        let (_dir, store, engine, _root, _manifest) = two_keyword_deltas();
        CHECKSUM_BYTES.with(|bytes| bytes.set(0));
        store.save_required(&engine, 64).unwrap();
        assert_eq!(
            CHECKSUM_BYTES.with(|bytes| bytes.get()),
            0,
            "unchanged checkpoint reread inherited delta payload"
        );
        store.load_latest().unwrap().unwrap();
        assert!(
            CHECKSUM_BYTES.with(|bytes| bytes.get()) > 0,
            "cold recovery must still verify every inherited payload"
        );
    }

    #[test]
    fn v2_base_rejects_validly_decodable_payload_change() {
        let (_dir, _store, _engine, root, manifest) = two_keyword_deltas();
        let base = manifest.collections[0]
            .segments
            .iter()
            .find(|segment| {
                matches!(segment.kind, SegmentKind::Base)
                    && matches!(segment.role, SegmentRole::Field)
                    && segment.field.as_deref() == Some("email")
            })
            .unwrap();
        let path = root.join(&base.path);
        let mut bytes = std::fs::read(&path).unwrap();
        assert_eq!(bytes[4104] & 1, 1);
        bytes[4104] ^= 1;
        std::fs::write(&path, bytes).unwrap();
        let reader = crate::segment::SegmentReader::open(&path).unwrap();
        assert_eq!(reader.n_docs(), 1);
        assert_eq!(reader.keyword_at(0), None);
        assert!(
            validate_catalog_references(&root, &manifest).is_err(),
            "v2 base payload corruption must not become silent data loss"
        );
    }

    #[test]
    fn changed_verified_predecessor_manifest_cannot_be_inherited() {
        let (_dir, store, engine, root, mut manifest) = two_keyword_deltas();
        let before = std::fs::read(store.root.join("CURRENT")).unwrap();
        manifest.collections[0]
            .segments
            .iter_mut()
            .find(|segment| matches!(segment.kind, SegmentKind::Delta))
            .unwrap()
            .payload_sha256 = Some("0".repeat(64));
        write_generation_manifest(&root, &manifest).unwrap();
        assert!(
            store.save_required(&engine, 64).is_err(),
            "changed predecessor checksum must not be inherited without verification"
        );
        assert_eq!(std::fs::read(store.root.join("CURRENT")).unwrap(), before);
    }

    #[test]
    fn new_store_verifies_predecessor_before_inheriting_payloads() {
        let (dir, _store, engine, root, mut manifest) = two_keyword_deltas();
        manifest.collections[0]
            .segments
            .iter_mut()
            .find(|segment| matches!(segment.kind, SegmentKind::Delta))
            .unwrap()
            .payload_sha256 = Some("0".repeat(64));
        write_generation_manifest(&root, &manifest).unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let before = std::fs::read(store.root.join("CURRENT")).unwrap();
        assert!(
            store.save_required(&engine, 64).is_err(),
            "new store must validate predecessor before trusting inherited checksums"
        );
        assert_eq!(std::fs::read(store.root.join("CURRENT")).unwrap(), before);
    }

    fn two_keyword_deltas() -> (
        tempfile::TempDir,
        SegmentRdbStore,
        Arc<Engine>,
        std::path::PathBuf,
        SegmentGenerationManifest,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "first@x.com");
        // Keep a populated legacy base for the base-payload corruption tests.
        // Fresh first checkpoints now store these rows in their first delta.
        engine.restore(engine.snapshot().unwrap()).unwrap();
        store.save_required(&engine, 61).unwrap();
        index_kw(&engine, "u1", "second@x.com");
        store.save_required(&engine, 62).unwrap();
        index_kw(&engine, "u1", "third@x.com");
        let generation = store.save_required(&engine, 63).unwrap();
        let root = dir.path().join(generation.as_str());
        let manifest = read_generation_manifest(&root).unwrap();
        (dir, store, engine, root, manifest)
    }

    #[test]
    fn v2_delta_rejects_valid_older_payload_and_row_map_substitution() {
        let (_dir, _store, _engine, root, manifest) = two_keyword_deltas();
        let deltas: Vec<_> = manifest.collections[0]
            .segments
            .iter()
            .filter(|segment| matches!(segment.kind, SegmentKind::Delta))
            .collect();
        assert_eq!(deltas.len(), 2);
        let older = deltas[0];
        let newer = deltas[1];

        // The older pair is a fully valid segment and row map for the same
        // external ID.  The old `<= checkpoint_sequence` check accepted it as
        // the newer ordinal and cold replay silently restored "second".
        std::fs::copy(root.join(&older.path), root.join(&newer.path)).unwrap();
        std::fs::copy(
            root.join(&older.local_rows.as_ref().unwrap().path),
            root.join(&newer.local_rows.as_ref().unwrap().path),
        )
        .unwrap();
        assert!(validate_catalog_references(&root, &manifest).is_err());
    }

    #[test]
    fn v2_delta_rejects_validly_decodable_payload_bit_flip() {
        let (_dir, _store, _engine, root, manifest) = two_keyword_deltas();
        let newer = manifest.collections[0]
            .segments
            .iter()
            .filter(|segment| matches!(segment.kind, SegmentKind::Delta))
            .max_by_key(|segment| segment.ordinal)
            .unwrap();
        let payload_path = root.join(&newer.path);
        let mut payload = std::fs::read(&payload_path).unwrap();
        // One keyword row has its u32 dictionary ID at 4096 and its
        // 8-aligned present bitset at 4104. These bytes are outside the
        // header/directory CRC. Flip presence while keeping the file decodable.
        assert_eq!(payload[4104] & 1, 1);
        payload[4104] ^= 1;
        std::fs::write(&payload_path, payload).unwrap();
        let reader = crate::segment::SegmentReader::open(&payload_path)
            .expect("structural checks accept a changed present bit");
        assert_eq!(reader.n_docs(), 1);
        assert_eq!(reader.applied_seq(), 63);
        assert_eq!(reader.keyword_at(0), None);
        assert!(validate_catalog_references(&root, &manifest).is_err());
    }

    use crate::types::{
        CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
        QueryNode, SearchRequest, TermQuery,
    };
    use std::collections::BTreeMap;

    fn kw_schema() -> CreateCollectionRequest {
        let mut fields = BTreeMap::new();
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

    fn index_kw(e: &Engine, eid: &str, v: &str) {
        index_kw_in(e, "u", eid, v);
    }

    fn index_kw_in(e: &Engine, collection: &str, eid: &str, v: &str) {
        e.index(
            collection,
            IndexRequest {
                items: vec![IndexItem {
                    external_id: eid.into(),
                    field: "email".into(),
                    value: FieldValue::String(v.into()),
                    version: None,
                }],
                request_id: None,
            },
        )
        .unwrap();
    }

    fn has_keyword(engine: &Engine, value: &str) -> bool {
        !engine
            .search(
                "u",
                SearchRequest {
                    query: QueryNode::Term(TermQuery {
                        field: "email".into(),
                        value: FieldValue::String(value.into()),
                    }),
                    limit: 10,
                    offset: 0,
                    cursor: None,
                    routing_key: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
            .unwrap()
            .hits
            .is_empty()
    }

    fn current_generation(store: &SegmentRdbStore) -> GenerationName {
        match store.generations.read_current().unwrap() {
            CurrentTarget::Generation(name) => name,
            CurrentTarget::Empty => panic!("expected an active generation"),
        }
    }

    #[test]
    fn background_scratch_does_not_consume_published_revision() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "one", "base");
        store.save(&engine, 1).unwrap();

        let scratch = store.begin_background_merge_stage(1).unwrap();
        assert_eq!(scratch.generation().as_str(), "gen-1-rev-0");
        let (revision, publication) = store
            .begin_next_generation_selected(1, StagingSelection::CurrentIfDurable)
            .unwrap();
        assert_eq!(revision, 2);

        drop(publication);
        drop(scratch);
    }

    #[test]
    fn background_worker_drains_compaction_requests_for_every_ready_field() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        for name in ["u", "v"] {
            engine.create_collection(name, kw_schema()).unwrap();
            index_kw_in(&engine, name, "one", "base");
        }
        store.save(&engine, 1).unwrap();
        for round in 1..=4 {
            for name in ["u", "v"] {
                index_kw_in(&engine, name, "one", &format!("round-{round}"));
            }
            store.save(&engine, round + 1).unwrap();
        }
        store
            .wait_for_merges(std::time::Duration::from_secs(10))
            .unwrap();
        // The bounded pairwise scheduler publishes one collection per job.
        // Request the next job after the first publication so the tied
        // collection receives its own pairwise compaction turn.
        store.request_merge(&engine).unwrap();
        store
            .wait_for_merges(std::time::Duration::from_secs(10))
            .unwrap();
        let manifest =
            read_generation_manifest(&dir.path().join(current_generation(&store).as_str()))
                .unwrap();
        for collection in &manifest.collections {
            let deltas = collection
                .segments
                .iter()
                .filter(|segment| matches!(segment.kind, SegmentKind::Delta))
                .count();
            assert!(
                deltas < 4,
                "every ready field must receive compaction work before the worker becomes idle: {} still has {deltas}",
                collection.collection_id
            );
        }
    }

    fn install_unpointed_generation(
        store: &SegmentRdbStore,
        engine: &Arc<Engine>,
        sequence: u64,
        previous: Option<&GenerationName>,
    ) -> GenerationName {
        let (revision, staged) = store.begin_next_generation(sequence).unwrap();
        let staging_path = staged.path().to_path_buf();
        engine.flush_to_segments(&staging_path, sequence).unwrap();
        write_generation_manifest(
            &staging_path,
            &SegmentGenerationManifest {
                schema_version: GENERATION_MANIFEST_SCHEMA_VERSION,
                checkpoint_sequence: sequence,
                revision,
                previous: previous.map(|name| name.as_str().to_owned()),
                next_collection_generation: 1,
                collections: Vec::new(),
            },
        )
        .unwrap();
        let name = staged.generation().clone();
        let target = store.generations.generation_path(&name);
        drop(staged);
        std::fs::rename(staging_path, &target).unwrap();
        sync_directory(&store.root).unwrap();
        name
    }

    #[test]
    fn loaded_checkpoint_files_stay_until_its_last_reader_is_dropped() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "one", "base");
        store.save(&engine, 1).unwrap();
        let loaded = store.load_current_generation().unwrap().unwrap();
        let protected = dir.path().join(loaded.name.as_str());
        // The loaded Engine can outlive every handle used to open its store.
        drop(store);
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        for round in 1..=4 {
            index_kw(&engine, "one", &format!("round-{round}"));
            store.save(&engine, round + 1).unwrap();
        }
        store
            .wait_for_merges(std::time::Duration::from_secs(10))
            .unwrap();
        store.prune(1).unwrap();
        assert!(
            protected.exists(),
            "a live cold reader must pin its checkpoint files"
        );
        assert!(has_keyword(&loaded.engine, "base"));
        drop(loaded);
        store.prune(1).unwrap();
        assert!(
            !protected.exists(),
            "a proved old generation can be reclaimed once its reader is gone"
        );
    }

    fn first_segment_file(root: &Path) -> PathBuf {
        let mut pending = vec![root.to_path_buf()];
        while let Some(directory) = pending.pop() {
            let mut entries = std::fs::read_dir(directory)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect::<Vec<_>>();
            entries.sort();
            for path in entries {
                if path.is_dir() {
                    pending.push(path);
                } else if path.extension().and_then(|extension| extension.to_str()) == Some("lseg")
                {
                    return path;
                }
            }
        }
        panic!("expected a segment file under {}", root.display());
    }

    #[test]
    fn merge_checkpoint_telemetry_separates_transient_delta_and_merged_base() {
        let actual_dir = tempfile::tempdir().unwrap();
        let control_dir = tempfile::tempdir().unwrap();
        let actual = SegmentRdbStore::new(actual_dir.path()).unwrap();
        let control = SegmentRdbStore::new(control_dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        let reference = Arc::new(Engine::new());
        for (store, state) in [(&actual, &engine), (&control, &reference)] {
            state.create_collection("u", kw_schema()).unwrap();
            for (id, value) in [
                ("one", "base-one"),
                ("two", "base-two"),
                ("three", "base-three"),
            ] {
                index_kw(state, id, value);
            }
            // Both controls need the same populated base before four measured
            // updates. This also exercises upgrade from snapshot state.
            state.restore(state.snapshot().unwrap()).unwrap();
            store.save(state, 1).unwrap();
        }
        // A single-delta control writes the same final record at the same cut.
        // Its physical payload supplies an oracle independent of the counter.
        index_kw(&reference, "one", "round-4");
        control.save(&reference, 5).unwrap();
        let control_path = control_dir
            .path()
            .join(current_generation(&control).as_str());
        let control_manifest = read_generation_manifest(&control_path).unwrap();
        let delta = control_manifest.collections[0]
            .segments
            .iter()
            .find(|segment| matches!(segment.kind, SegmentKind::Delta))
            .unwrap();
        let fresh_delta_bytes = std::fs::metadata(control_path.join(&delta.path))
            .unwrap()
            .len()
            + std::fs::metadata(control_path.join(&delta.local_rows.as_ref().unwrap().path))
                .unwrap()
                .len();
        for round in 1..=3 {
            index_kw(&engine, "one", &format!("round-{round}"));
            actual.save(&engine, round + 1).unwrap();
        }
        let checkpoint_before = engine.metrics().segment_checkpoint_bytes_total.get();
        let merge_before = engine.metrics().segment_merge_write_bytes_total.get();
        let merge_count_before = engine.metrics().segment_merge_completed_total.get();
        index_kw(&engine, "one", "round-4");
        let checkpoint_name = actual
            .save_inner(&engine, 5, false, StagingSelection::CurrentIfDurable)
            .unwrap();
        actual
            .wait_for_merges(std::time::Duration::from_secs(10))
            .unwrap();
        let current = actual_dir.path().join(current_generation(&actual).as_str());
        let manifest = read_generation_manifest(&current).unwrap();
        let base = manifest.collections[0]
            .segments
            .iter()
            .find(|segment| matches!(segment.role, SegmentRole::Field))
            .unwrap();
        assert!(matches!(base.kind, SegmentKind::Base));
        let merged_bytes = std::fs::metadata(current.join(&base.path)).unwrap().len()
            + std::fs::metadata(current.join(&base.local_rows.as_ref().unwrap().path))
                .unwrap()
                .len();
        assert_ne!(
            fresh_delta_bytes, merged_bytes,
            "oracle must distinguish fresh input from merged output"
        );
        let manifest_bytes = std::fs::metadata(
            actual_dir
                .path()
                .join(checkpoint_name.as_str())
                .join(GENERATION_MANIFEST_FILE),
        )
        .unwrap()
        .len();
        assert_eq!(
            engine.metrics().segment_checkpoint_bytes_total.get() - checkpoint_before,
            fresh_delta_bytes + manifest_bytes,
            "checkpoint counter must include its fresh delta and its own published manifest"
        );
        assert_eq!(
            engine.metrics().segment_merge_write_bytes_total.get() - merge_before,
            merged_bytes,
            "merge counter must contain the merged output only"
        );
        assert_eq!(
            engine.metrics().segment_merge_completed_total.get() - merge_count_before,
            1
        );
    }

    fn checkpoint_diagnostic_records(
        enabled: bool,
        origin: &'static str,
        attempt_id: Option<u64>,
    ) -> Vec<serde_json::Value> {
        use tracing_subscriber::prelude::*;

        let _environment = DiagnosticEnvironment::set(enabled);
        let writer = DiagnosticTraceWriter::default();
        let subscriber = tracing_subscriber::registry().with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_ansi(false)
                .with_writer(writer.clone()),
        );
        let _guard = tracing::subscriber::set_default(subscriber);
        let dir = tempfile::tempdir().unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "one", "trace-value");
        let store = SegmentRdbStore::new(dir.path()).unwrap();

        assert_eq!(
            store
                .save_with_sequence_diagnostic(&engine, 1, origin, attempt_id)
                .unwrap(),
            1
        );
        drop(_guard);
        writer.records()
    }

    #[test]
    fn checkpoint_diagnostic_trace_is_machine_readable() {
        let record = checkpoint_diagnostic_records(true, "periodic", None)
            .into_iter()
            .find(|record| record["fields"]["event"] == "segment_checkpoint_diagnostic")
            .expect("durable checkpoint must emit its diagnostic trace");
        let fields = record["fields"]
            .as_object()
            .expect("diagnostic trace fields must be JSON object");
        assert_eq!(fields["checkpoint_origin"], "periodic");
        assert_eq!(fields["checkpoint_sequence"], 1);
        assert_eq!(fields["capture_to_save_gate_wait_ns"], 0);
        for name in [
            "frozen_cut_bytes",
            "save_gate_wait_ns",
            "save_gate_hold_ns",
            "publish_ns",
            "acknowledge_ns",
            "capacity_request_pending",
            "capacity_request_revision",
            "root_merge_queued",
            "root_merge_running",
            "root_merge_requested",
            "root_merge_published_revision",
        ] {
            assert!(fields.contains_key(name), "missing diagnostic field {name}");
        }
    }

    #[test]
    fn numeric_capture_tokens_isolate_parallel_subscribers_and_expire_on_drop() {
        let first = DiagnosticCapture::new();
        let second = DiagnosticCapture::new();
        assert_ne!(first.token(), second.token());
        let event = NumericCanonicalEvent::Phase {
            attempt_id: 7, phase: "terminal", pass: 0, reused: false, frozen_bytes: 0,
        };
        send_numeric_event(first.token(), event);
        assert_eq!(first.drain(), [event]);
        assert!(second.drain().is_empty());
        let old = first.token();
        drop(first);
        send_numeric_event(old, event);
        send_numeric_event(second.token(), event);
        assert_eq!(second.drain(), [event]);
    }

    #[test]
    fn checkpoint_diagnostic_trace_is_absent_without_exact_environment_flag() {
        let records = checkpoint_diagnostic_records(false, "periodic", None);
        assert!(
            !records
                .iter()
                .any(|record| record["fields"]["event"] == "segment_checkpoint_diagnostic"),
            "ordinary checkpoints must not emit a diagnostic trace"
        );
    }

    #[test]
    fn manual_checkpoint_diagnostic_phases_keep_one_attempt_id() {
        let records = checkpoint_diagnostic_records(true, "manual", Some(41));
        let phases: Vec<_> = records
            .iter()
            .filter(|record| record["fields"]["event"] == "segment_checkpoint_diagnostic_phase")
            .collect();
        assert!(phases
            .iter()
            .any(|record| record["fields"]["phase"] == "freeze_completed"));
        assert!(phases
            .iter()
            .any(|record| record["fields"]["phase"] == "publish_completed"));
        assert!(phases
            .iter()
            .all(|record| record["fields"]["checkpoint_attempt_id"] == 41));
    }

    #[test]
    fn periodic_checkpoint_lifecycle_phases_are_ordered_and_share_one_attempt_id() {
        use tracing_subscriber::prelude::*;

        let _environment = DiagnosticEnvironment::set(true);
        let writer = DiagnosticTraceWriter::default();
        let subscriber = tracing_subscriber::registry().with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_ansi(false)
                .with_writer(writer.clone()),
        );
        let _guard = tracing::subscriber::set_default(subscriber);
        let context = CheckpointDiagnosticContext::new("periodic", Some(73));
        let pending = crate::change_budget::Snapshot {
            reserved: 11,
            active: 22,
            frozen: 33,
            total: crate::change_budget::CHECKPOINT_TRIGGER,
            work_revision: 1,
            checkpoint_request_revision: None,
        };
        context.trace_scheduler_selected("threshold", "initial_sample", pending);
        context.trace_phase("checkpoint_started");
        let dir = tempfile::tempdir().unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "one", "trace-value");
        SegmentRdbStore::new(dir.path())
            .unwrap()
            .save_with_sequence_diagnostic_context(&engine, 1, Some(context))
            .unwrap();
        let terminal: Result<()> = Ok(());
        context.trace_terminal(&terminal);
        drop(_guard);

        let phases: Vec<_> = writer
            .records()
            .into_iter()
            .filter(|record| record["fields"]["event"] == "segment_checkpoint_diagnostic_phase")
            .collect();
        assert_eq!(
            phases
                .iter()
                .map(|record| record["fields"]["phase"].as_str().unwrap())
                .collect::<Vec<_>>(),
            [
                "scheduler_selected",
                "checkpoint_started",
                "freeze_completed",
                "publish_completed",
                "terminal",
            ]
        );
        assert!(phases
            .iter()
            .all(|record| record["fields"]["checkpoint_attempt_id"] == 73));
        assert!(phases.windows(2).all(|pair| {
            pair[0]["fields"]["elapsed_ns"].as_u64() <= pair[1]["fields"]["elapsed_ns"].as_u64()
        }));
        let selected = &phases[0]["fields"];
        assert_eq!(
            selected["pending_total_bytes"],
            crate::change_budget::CHECKPOINT_TRIGGER
        );
        assert_eq!(selected["pending_reserved_bytes"], 11);
        assert_eq!(selected["pending_active_bytes"], 22);
        assert_eq!(selected["pending_frozen_bytes"], 33);
        assert_eq!(selected["scheduler_reason"], "threshold");
        assert_eq!(selected["resample_source"], "initial_sample");
        assert!(selected.get("wake_lag_ns").is_none());
        let frozen = &phases[2]["fields"];
        assert_eq!(frozen["checkpoint_pass"], 1);
        assert_eq!(frozen["frozen_cut_reused"], false);
        assert!(frozen["frozen_cut_bytes"].as_u64().is_some());
    }

    #[test]
    fn checkpoint_telemetry_counts_only_durable_publications_and_new_file_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "first", "value");
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        assert_eq!(engine.metrics().segment_checkpoint_completed_total.get(), 0);
        store.save(&engine, 1).unwrap();
        assert_eq!(
            engine.metrics().segment_checkpoint_completed_total.get(),
            1,
            "real durable checkpoint must produce one telemetry completion"
        );
        let first_bytes = engine.metrics().segment_checkpoint_bytes_total.get();
        assert!(first_bytes > 0);
        assert!(engine.metrics().segment_disk_bytes.get() >= first_bytes);
        assert_eq!(
            engine.metrics().segment_merge_completed_total.get(),
            0,
            "checkpoint without a merge cannot manufacture merge evidence"
        );
        store.save(&engine, 2).unwrap();
        let linked_bytes = engine.metrics().segment_checkpoint_bytes_total.get() - first_bytes;
        assert!(
            linked_bytes > 0 && linked_bytes < first_bytes,
            "an unchanged checkpoint writes its manifest, not every linked payload again"
        );
        struct FailAt(storage_durable::CommitStep);
        impl FailureInjector for FailAt {
            fn check(&self, point: &storage_durable::FailurePoint) -> std::io::Result<()> {
                if point.step == self.0 {
                    return Err(std::io::Error::other("telemetry injected failure"));
                }
                Ok(())
            }
        }
        let failing = SegmentRdbStore::new_with_failure_injector(
            dir.path(),
            Arc::new(FailAt(storage_durable::CommitStep::SyncFile)),
        )
        .unwrap();
        index_kw(&engine, "second", "new");
        assert!(failing.save(&engine, 3).is_err());
        assert_eq!(
            engine.metrics().segment_checkpoint_completed_total.get(),
            2,
            "failed checkpoint cannot manufacture durable completion evidence"
        );
    }

    #[test]
    fn save_then_load_round_trips_at_seq() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();

        let src = Arc::new(Engine::new());
        src.create_collection("u", kw_schema()).unwrap();
        index_kw(&src, "u1", "a@x.com");
        store.save(&src, 42).unwrap();

        let (eng, seq) = store.load_latest().unwrap().expect("a checkpoint");
        assert_eq!(seq, 42);
        assert_eq!(eng.stats("u").unwrap().documents_indexed, 1);
    }

    #[test]
    fn reopen_replaces_the_complete_previous_collection_set() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let source = Arc::new(Engine::new());
        source.create_collection("u", kw_schema()).unwrap();
        store.save(&source, 10).unwrap();
        let target = Arc::new(Engine::new());
        target.create_collection("obsolete", kw_schema()).unwrap();
        assert_eq!(store.reopen_into(&target).unwrap(), Some(10));
        assert_eq!(target.list_collections().unwrap(), vec!["u"]);
    }

    #[test]
    fn adopts_exact_0428_generation_once_and_writes_exact_current() {
        let dir = tempfile::tempdir().unwrap();
        let legacy = dir.path().join("gen-42");
        std::fs::create_dir(&legacy).unwrap();
        let source = Arc::new(Engine::new());
        source.create_collection("u", kw_schema()).unwrap();
        index_kw(&source, "u1", "a@x.com");
        source.flush_to_segments(&legacy, 42).unwrap();

        let store = SegmentRdbStore::new(dir.path()).unwrap();
        assert!(!dir.path().join("CURRENT").exists());
        let loaded = Arc::new(Engine::new());
        let outcome = store.reopen_into_with_outcome(&loaded).unwrap();
        assert_eq!(outcome.decision, SegmentStartupDecision::AdoptedLegacy0428);
        assert_eq!(outcome.checkpoint_sequence, Some(42));
        assert_eq!(
            outcome.generation.as_ref().map(GenerationName::as_str),
            Some("gen-42")
        );
        assert_eq!(loaded.stats("u").unwrap().documents_indexed, 1);
        assert_eq!(
            std::fs::read(dir.path().join("CURRENT")).unwrap(),
            b"generation:gen-42\n"
        );

        let restarted = SegmentRdbStore::new(dir.path()).unwrap();
        assert_eq!(restarted.load_latest().unwrap().unwrap().1, 42);
    }

    #[test]
    fn adopts_empty_0428_generation_without_losing_sequence() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("gen-42")).unwrap();

        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let (_, sequence) = store.load_latest().unwrap().unwrap();
        assert_eq!(sequence, 42);
        assert_eq!(
            std::fs::read(dir.path().join("CURRENT")).unwrap(),
            b"generation:gen-42\n"
        );
    }

    #[test]
    fn corrupt_highest_legacy_generation_blocks_fallback_and_adoption() {
        let dir = tempfile::tempdir().unwrap();
        let valid = dir.path().join("gen-42");
        std::fs::create_dir(&valid).unwrap();
        let source = Arc::new(Engine::new());
        source.create_collection("u", kw_schema()).unwrap();
        index_kw(&source, "u1", "a@x.com");
        source.flush_to_segments(&valid, 42).unwrap();
        let corrupt = dir.path().join("gen-99");
        std::fs::create_dir(&corrupt).unwrap();
        std::fs::write(corrupt.join("not-a-collection"), b"corrupt").unwrap();

        let store = SegmentRdbStore::new(dir.path()).unwrap();
        assert!(store.load_latest().is_err());
        assert!(!dir.path().join("CURRENT").exists());
    }

    #[cfg(unix)]
    #[test]
    fn legacy_nested_symlink_blocks_adoption() {
        use std::os::unix::fs::symlink;

        let dir = tempfile::tempdir().unwrap();
        let legacy = dir.path().join("gen-42");
        std::fs::create_dir(&legacy).unwrap();
        let source = Arc::new(Engine::new());
        source.create_collection("u", kw_schema()).unwrap();
        index_kw(&source, "u1", "a@x.com");
        source.flush_to_segments(&legacy, 42).unwrap();
        let outside = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(outside.path(), b"outside").unwrap();
        let collection = std::fs::read_dir(&legacy)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| path.is_dir())
            .unwrap();
        symlink(outside.path(), collection.join("nested-link")).unwrap();

        let store = SegmentRdbStore::new(dir.path()).unwrap();
        assert!(store.load_latest().is_err());
        assert!(!dir.path().join("CURRENT").exists());
    }

    #[cfg(unix)]
    #[test]
    fn parseable_legacy_aside_symlink_blocks_empty_initialization() {
        use std::os::unix::fs::symlink;

        let dir = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        symlink(target.path(), dir.path().join("gen-7.old")).unwrap();

        assert!(SegmentRdbStore::new(dir.path()).is_err());
        assert!(!dir.path().join("CURRENT").exists());
    }

    #[test]
    fn empty_new_generation_preserves_sequence_after_restart() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        store.save(&Arc::new(Engine::new()), 42).unwrap();

        let restarted = SegmentRdbStore::new(dir.path()).unwrap();
        let (_, sequence) = restarted.load_latest().unwrap().unwrap();
        assert_eq!(sequence, 42);
    }

    #[test]
    fn lower_sequence_save_is_a_noop() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");
        store.save(&engine, 9).unwrap();
        let current_before = std::fs::read(dir.path().join("CURRENT")).unwrap();

        index_kw(&engine, "u2", "b@x.com");
        store.save(&engine, 8).unwrap();

        assert_eq!(
            std::fs::read(dir.path().join("CURRENT")).unwrap(),
            current_before
        );
        let (loaded, sequence) = store.load_latest().unwrap().unwrap();
        assert_eq!(sequence, 9);
        assert_eq!(loaded.stats("u").unwrap().documents_indexed, 1);
    }

    #[test]
    fn required_lower_sequence_rejects_without_changing_current() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        store.save(&engine, 9).unwrap();
        let before = std::fs::read(dir.path().join("CURRENT")).unwrap();

        let error = store.save_required(&engine, 8).unwrap_err();
        assert!(error.to_string().contains("below CURRENT sequence 9"));
        assert_eq!(std::fs::read(dir.path().join("CURRENT")).unwrap(), before);
    }

    #[test]
    fn required_same_sequence_creates_distinct_revision_and_exact_loader_matches() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");
        let first = store.save_required(&engine, 9).unwrap();
        index_kw(&engine, "u2", "b@x.com");
        let second = store.save_required(&engine, 9).unwrap();
        assert_ne!(first, second);

        let loaded = store.load_current_generation().unwrap().unwrap();
        assert_eq!(loaded.name, second);
        assert_eq!(loaded.sequence, 9);
        assert_eq!(loaded.engine.stats("u").unwrap().documents_indexed, 2);
    }

    #[test]
    fn exact_loader_never_selects_unpointed_higher_generation() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");
        let current = store.save_required(&engine, 9).unwrap();
        let unpointed = install_unpointed_generation(&store, &engine, 99, Some(&current));

        let loaded = store.load_current_generation().unwrap().unwrap();
        assert_eq!(loaded.name, current);
        assert_eq!(loaded.sequence, 9);
        assert_ne!(loaded.name, unpointed);
    }

    #[test]
    fn exact_loader_never_adopts_a_legacy_generation_when_current_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let legacy = dir.path().join("gen-42");
        std::fs::create_dir(&legacy).unwrap();
        let source = Arc::new(Engine::new());
        source.create_collection("u", kw_schema()).unwrap();
        index_kw(&source, "u1", "a@x.com");
        source.flush_to_segments(&legacy, 42).unwrap();

        let store = SegmentRdbStore::new(dir.path()).unwrap();
        assert!(!dir.path().join("CURRENT").exists());
        assert!(store.load_current_generation().is_err());
        assert!(
            !dir.path().join("CURRENT").exists(),
            "exact restore reload must not perform the 0.4.28 startup adoption"
        );
    }

    #[test]
    fn injected_store_commit_is_deterministic() {
        #[derive(Default)]
        struct FailRenameCurrent(Mutex<Vec<storage_durable::FailurePoint>>);

        impl FailureInjector for FailRenameCurrent {
            fn check(&self, point: &storage_durable::FailurePoint) -> std::io::Result<()> {
                self.0.lock().unwrap().push(point.clone());
                if point.step == storage_durable::CommitStep::RenameCurrent {
                    return Err(std::io::Error::other("injected rename failure"));
                }
                Ok(())
            }
        }

        let dir = tempfile::tempdir().unwrap();
        SegmentRdbStore::new(dir.path()).unwrap();
        let injector = Arc::new(FailRenameCurrent::default());
        let store =
            SegmentRdbStore::new_with_failure_injector(dir.path(), injector.clone()).unwrap();
        let error = store
            .save_required(&Arc::new(Engine::new()), 1)
            .unwrap_err();
        assert!(error.to_string().contains("activate segment generation"));
        assert!(matches!(store.load_current_generation().unwrap(), None));
        assert!(injector
            .0
            .lock()
            .unwrap()
            .iter()
            .any(|point| point.step == storage_durable::CommitStep::RenameCurrent));
    }

    #[test]
    fn load_latest_picks_highest_seq() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let e = Arc::new(Engine::new());
        e.create_collection("u", kw_schema()).unwrap();
        index_kw(&e, "u1", "a@x.com");
        for seq in [10u64, 5, 99, 50] {
            store.save(&e, seq).unwrap();
        }
        assert_eq!(store.load_latest().unwrap().unwrap().1, 99);
    }

    #[test]
    fn prune_keeps_newest() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let e = Arc::new(Engine::new());
        e.create_collection("u", kw_schema()).unwrap();
        index_kw(&e, "u1", "a@x.com");
        for seq in 1..=5u64 {
            store.save(&e, seq).unwrap();
        }
        let removed = store.prune(2).unwrap();
        assert_eq!(removed, 3);
        assert_eq!(store.generation_seqs().unwrap(), vec![4, 5]);
        assert_eq!(store.load_latest().unwrap().unwrap().1, 5);
    }

    #[test]
    fn torn_staging_dir_is_ignored_and_swept() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let e = Arc::new(Engine::new());
        e.create_collection("u", kw_schema()).unwrap();
        index_kw(&e, "u1", "a@x.com");
        store.save(&e, 7).unwrap();

        // Simulate a crash mid-stage: a leftover `.gen-<seq>.tmp` dir.
        std::fs::create_dir_all(dir.path().join(".gen-9.tmp")).unwrap();
        // load_latest still returns the good committed generation, not the torn one.
        assert_eq!(store.load_latest().unwrap().unwrap().1, 7);
        // A subsequent save sweeps the torn staging dir.
        store.save(&e, 8).unwrap();
        assert!(!dir.path().join(".gen-9.tmp").exists());
        assert_eq!(store.load_latest().unwrap().unwrap().1, 8);
    }

    #[test]
    fn abandoned_durable_staging_is_swept_on_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let _first = SegmentRdbStore::new(dir.path()).unwrap();
        let staging = dir.path().join(".stage-gen-9-rev-99");
        std::fs::create_dir(&staging).unwrap();
        std::fs::write(staging.join("partial"), b"partial").unwrap();

        let _reopened = SegmentRdbStore::new(dir.path()).unwrap();
        assert!(!staging.exists());
    }

    #[test]
    fn unrelated_legacy_like_directory_is_not_swept() {
        let dir = tempfile::tempdir().unwrap();
        let unrelated = dir.path().join(".gen-user.tmp");
        std::fs::create_dir(&unrelated).unwrap();
        std::fs::write(unrelated.join("owned-by-user"), b"keep").unwrap();

        assert!(SegmentRdbStore::new(dir.path()).is_err());
        assert!(
            !dir.path().join("CURRENT").exists(),
            "an unrecognized non-empty root must not become an empty store"
        );
        assert!(unrelated.is_dir());
        assert_eq!(
            std::fs::read(unrelated.join("owned-by-user")).unwrap(),
            b"keep"
        );
    }

    #[test]
    fn mixed_legacy_and_unknown_root_fails_before_legacy_adoption() {
        let dir = tempfile::tempdir().unwrap();
        let legacy = dir.path().join("gen-42");
        std::fs::create_dir(&legacy).unwrap();
        let source = Arc::new(Engine::new());
        source.create_collection("u", kw_schema()).unwrap();
        index_kw(&source, "u1", "a@x.com");
        source.flush_to_segments(&legacy, 42).unwrap();
        std::fs::create_dir(dir.path().join("foreign-layout")).unwrap();

        assert!(SegmentRdbStore::new(dir.path()).is_err());
        assert!(
            !dir.path().join("CURRENT").exists(),
            "unknown content must block legacy adoption before it writes CURRENT"
        );
        assert!(legacy.is_dir());
        assert!(dir.path().join("foreign-layout").is_dir());
    }

    #[test]
    fn unknown_entry_beside_valid_current_fails_before_any_cleanup() {
        let dir = tempfile::tempdir().unwrap();
        let first = SegmentRdbStore::new(dir.path()).unwrap();
        let current = std::fs::read(dir.path().join("CURRENT")).unwrap();
        drop(first);
        let foreign = dir.path().join("foreign-layout");
        std::fs::create_dir(&foreign).unwrap();

        assert!(SegmentRdbStore::new(dir.path()).is_err());
        assert_eq!(std::fs::read(dir.path().join("CURRENT")).unwrap(), current);
        assert!(foreign.is_dir());
    }

    #[test]
    fn unpointed_revision_without_current_has_a_specific_fail_closed_error() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("gen-7-rev-1")).unwrap();

        let error = SegmentRdbStore::new(dir.path()).unwrap_err();
        assert!(error.to_string().contains("unpointed revision generation"));
        assert!(!dir.path().join("CURRENT").exists());
    }

    #[test]
    fn aof_only_root_remains_a_supported_empty_checkpoint_baseline() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("aof.log"), b"").unwrap();
        std::fs::write(dir.path().join("aof.log.compact.tmp"), b"").unwrap();

        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let outcome = store
            .reopen_into_with_outcome(&Arc::new(Engine::new()))
            .unwrap();
        assert_eq!(
            outcome.decision,
            SegmentStartupDecision::RecoveredUncommittedEmpty
        );
        assert_eq!(outcome.checkpoint_sequence, None);
        assert_eq!(
            std::fs::read(dir.path().join("CURRENT")).unwrap(),
            b"empty\n"
        );
    }

    #[test]
    fn compact_aof_temp_without_aof_is_rejected_before_current_is_written() {
        let dir = tempfile::tempdir().unwrap();
        let compact = dir.path().join(AOF_COMPACT_TEMP_FILE);
        let bytes = b"uncommitted compact output";
        std::fs::write(&compact, bytes).unwrap();

        let error = SegmentRdbStore::new(dir.path()).unwrap_err();
        assert!(error
            .to_string()
            .contains("aof.log.compact.tmp requires regular aof.log beside it"));
        assert!(!dir.path().join(CURRENT_FILE).exists());
        assert_eq!(std::fs::read(compact).unwrap(), bytes);
    }

    #[test]
    fn invalid_root_inventory_lists_every_child_name_and_kind_in_sorted_order() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(AOF_FILE)).unwrap();
        std::fs::write(dir.path().join("alpha-foreign"), b"foreign").unwrap();
        std::fs::create_dir(dir.path().join("zeta-foreign")).unwrap();

        let error = SegmentRdbStore::new(dir.path()).unwrap_err();
        let rendered = error.to_string();
        assert!(rendered.contains(
            "invalid segment checkpoint root inventory [alpha-foreign (regular file), aof.log (directory), zeta-foreign (directory)]"
        ));
        assert!(rendered.contains("checkpoint root entry must be a regular file"));
        assert!(rendered
            .contains("unrecognized non-empty segment checkpoint root entry `alpha-foreign`"));
        assert!(rendered
            .contains("unrecognized non-empty segment checkpoint root entry `zeta-foreign`"));
        assert!(!dir.path().join(CURRENT_FILE).exists());
    }

    #[test]
    fn current_empty_remains_authoritative_over_an_unpointed_revision() {
        let dir = tempfile::tempdir().unwrap();
        let _first = SegmentRdbStore::new(dir.path()).unwrap();
        let current = std::fs::read(dir.path().join(CURRENT_FILE)).unwrap();
        std::fs::create_dir(dir.path().join("gen-7-rev-1")).unwrap();

        let reopened = SegmentRdbStore::new(dir.path()).unwrap();
        let outcome = reopened
            .reopen_into_with_outcome(&Arc::new(Engine::new()))
            .unwrap();
        assert_eq!(
            outcome.decision,
            SegmentStartupDecision::RestoredCurrentEmpty
        );
        assert_eq!(
            std::fs::read(dir.path().join(CURRENT_FILE)).unwrap(),
            current
        );
    }

    #[test]
    fn genuinely_empty_root_reports_initialization_once_then_current_empty() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(CONTAINER_VOLUME_SEED_FILE), b"").unwrap();
        let first = SegmentRdbStore::new(dir.path()).unwrap();
        assert_eq!(
            first
                .reopen_into_with_outcome(&Arc::new(Engine::new()))
                .unwrap()
                .decision,
            SegmentStartupDecision::InitializedEmptyRoot
        );

        let reopened = SegmentRdbStore::new(dir.path()).unwrap();
        assert_eq!(
            reopened
                .reopen_into_with_outcome(&Arc::new(Engine::new()))
                .unwrap()
                .decision,
            SegmentStartupDecision::RestoredCurrentEmpty
        );
    }

    #[test]
    fn empty_lost_found_is_accepted_beside_supported_root_layouts() {
        fn create_lost_found(root: &Path) {
            let lost_found = root.join(EXT_FILESYSTEM_METADATA_DIR);
            std::fs::create_dir(&lost_found).unwrap();
            assert!(std::fs::read_dir(&lost_found).unwrap().next().is_none());
        }

        let empty = tempfile::tempdir().unwrap();
        create_lost_found(empty.path());
        let store = SegmentRdbStore::new(empty.path()).unwrap();
        assert_eq!(
            store
                .reopen_into_with_outcome(&Arc::new(Engine::new()))
                .unwrap()
                .decision,
            SegmentStartupDecision::InitializedEmptyRoot
        );
        assert_eq!(
            std::fs::read(empty.path().join(CURRENT_FILE)).unwrap(),
            b"empty\n"
        );

        let aof = tempfile::tempdir().unwrap();
        create_lost_found(aof.path());
        std::fs::write(aof.path().join(AOF_FILE), b"").unwrap();
        let store = SegmentRdbStore::new(aof.path()).unwrap();
        assert_eq!(
            store
                .reopen_into_with_outcome(&Arc::new(Engine::new()))
                .unwrap()
                .decision,
            SegmentStartupDecision::RecoveredUncommittedEmpty
        );

        let legacy = tempfile::tempdir().unwrap();
        create_lost_found(legacy.path());
        let legacy_generation = legacy.path().join("gen-42");
        std::fs::create_dir(&legacy_generation).unwrap();
        let source = Arc::new(Engine::new());
        source.create_collection("u", kw_schema()).unwrap();
        index_kw(&source, "u1", "a@x.com");
        source.flush_to_segments(&legacy_generation, 42).unwrap();
        let store = SegmentRdbStore::new(legacy.path()).unwrap();
        assert_eq!(
            store
                .reopen_into_with_outcome(&Arc::new(Engine::new()))
                .unwrap()
                .decision,
            SegmentStartupDecision::AdoptedLegacy0428
        );

        let current = tempfile::tempdir().unwrap();
        create_lost_found(current.path());
        let first = SegmentRdbStore::new(current.path()).unwrap();
        drop(first);
        let reopened = SegmentRdbStore::new(current.path()).unwrap();
        assert_eq!(
            reopened
                .reopen_into_with_outcome(&Arc::new(Engine::new()))
                .unwrap()
                .decision,
            SegmentStartupDecision::RestoredCurrentEmpty
        );

        for root in [empty.path(), aof.path(), legacy.path(), current.path()] {
            let lost_found = root.join(EXT_FILESYSTEM_METADATA_DIR);
            let metadata = std::fs::symlink_metadata(&lost_found).unwrap();
            assert!(metadata.is_dir() && !metadata.file_type().is_symlink());
            assert!(std::fs::read_dir(lost_found).unwrap().next().is_none());
        }
    }

    #[test]
    fn legacy_aside_is_recovered_and_reported_before_adoption() {
        let dir = tempfile::tempdir().unwrap();
        let legacy = dir.path().join("gen-42.old");
        std::fs::create_dir(&legacy).unwrap();
        let source = Arc::new(Engine::new());
        source.create_collection("u", kw_schema()).unwrap();
        index_kw(&source, "u1", "a@x.com");
        source.flush_to_segments(&legacy, 42).unwrap();

        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let outcome = store
            .reopen_into_with_outcome(&Arc::new(Engine::new()))
            .unwrap();
        assert_eq!(outcome.decision, SegmentStartupDecision::AdoptedLegacy0428);
        assert!(outcome.recovered_legacy_aside);
        assert!(dir.path().join("gen-42").is_dir());
        assert!(!dir.path().join("gen-42.old").exists());
    }

    #[cfg(unix)]
    #[test]
    fn unknown_root_symlink_fails_before_current_is_written() {
        use std::os::unix::fs::symlink;

        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        symlink(outside.path(), dir.path().join("foreign-layout")).unwrap();

        assert!(SegmentRdbStore::new(dir.path()).is_err());
        assert!(!dir.path().join("CURRENT").exists());
        assert!(dir.path().join("foreign-layout").is_symlink());
    }

    #[test]
    fn staged_corruption_is_rejected_before_current_moves() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");
        let (revision, staged) = store.begin_next_generation(7).unwrap();
        let staging_path = staged.path().to_path_buf();
        engine.flush_to_segments(&staging_path, 7).unwrap();
        write_generation_manifest(
            &staging_path,
            &SegmentGenerationManifest {
                schema_version: GENERATION_MANIFEST_SCHEMA_VERSION,
                checkpoint_sequence: 7,
                revision,
                previous: None,
                next_collection_generation: 1,
                collections: Vec::new(),
            },
        )
        .unwrap();
        std::fs::write(first_segment_file(&staging_path), b"corrupt").unwrap();
        let record = GenerationRecord {
            name: staged.generation().clone(),
            path: staging_path,
            sequence: 7,
            revision,
            legacy: false,
            previous: None,
        };

        assert!(store.validate_record(&record).is_err());
        assert_eq!(
            store.generations.read_current().unwrap(),
            CurrentTarget::Empty
        );
    }

    #[test]
    fn staged_manifest_must_parse_and_match_the_validated_record() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");
        let (revision, staged) = store.begin_next_generation(7).unwrap();
        let staging_path = staged.path().to_path_buf();
        engine.flush_to_segments(&staging_path, 7).unwrap();
        let record = GenerationRecord {
            name: staged.generation().clone(),
            path: staging_path.clone(),
            sequence: 7,
            revision,
            legacy: false,
            previous: None,
        };

        std::fs::write(staging_path.join(GENERATION_MANIFEST_FILE), b"{\"").unwrap();
        assert!(store.validate_record(&record).is_err());

        write_generation_manifest(
            &staging_path,
            &SegmentGenerationManifest {
                schema_version: GENERATION_MANIFEST_SCHEMA_VERSION,
                checkpoint_sequence: 8,
                revision,
                previous: None,
                next_collection_generation: 1,
                collections: Vec::new(),
            },
        )
        .unwrap();
        assert!(store.validate_record(&record).is_err());
        assert_eq!(
            store.generations.read_current().unwrap(),
            CurrentTarget::Empty
        );
    }

    /// Keep the historical test name because the release gate calls it by
    /// exact name. The 0.4.29 model no longer moves the predecessor aside.
    /// It installs a complete immutable replacement first. `CURRENT` remains
    /// the sole commit point, so a crash before that pointer rename must reopen
    /// the predecessor and ignore the complete replacement.
    #[test]
    fn same_seq_resave_crash_between_aside_and_commit_recovers_predecessor() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();

        // The predecessor: a first successful save at seq 7.
        let engine_a = Arc::new(Engine::new());
        engine_a.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine_a, "a1", "a1@x.com");
        store.save(&engine_a, 7).unwrap();
        assert_eq!(store.load_latest().unwrap().unwrap().1, 7);
        let predecessor = current_generation(&store);

        // Prepare the complete replacement and perform the generation rename.
        // Do not change CURRENT. This is the exact pre-commit crash state.
        let engine_b = Arc::new(Engine::new());
        engine_b.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine_b, "b1", "b1@x.com");
        index_kw(&engine_b, "b2", "b2@x.com");
        let replacement = install_unpointed_generation(&store, &engine_b, 7, Some(&predecessor));
        assert!(store.generations.generation_path(&replacement).is_dir());
        assert_eq!(current_generation(&store), predecessor);

        // Cold start from scratch, as a restarted pod does.
        let cold_store = SegmentRdbStore::new(dir.path()).unwrap();
        let (reloaded, seq) = cold_store
            .load_latest()
            .unwrap()
            .expect("a complete generation survives the crash window");
        assert_eq!(seq, 7);
        assert_eq!(
            reloaded.stats("u").unwrap().documents_indexed,
            1,
            "recovered generation must be the predecessor (1 doc), not the \
             never-committed replacement (2 docs) or nothing"
        );

        // A normal same-sequence save activates a new immutable revision.
        cold_store.save(&engine_b, 7).unwrap();
        assert_ne!(current_generation(&cold_store), replacement);
        assert_eq!(
            cold_store
                .load_latest()
                .unwrap()
                .unwrap()
                .0
                .stats("u")
                .unwrap()
                .documents_indexed,
            2
        );
    }

    #[test]
    fn complete_unpointed_higher_generation_is_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let active_engine = Arc::new(Engine::new());
        active_engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&active_engine, "u1", "a@x.com");
        store.save(&active_engine, 7).unwrap();
        let active = current_generation(&store);

        let later_engine = Arc::new(Engine::new());
        later_engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&later_engine, "u1", "a@x.com");
        index_kw(&later_engine, "u2", "b@x.com");
        install_unpointed_generation(&store, &later_engine, 99, Some(&active));

        let restarted = SegmentRdbStore::new(dir.path()).unwrap();
        let (loaded, sequence) = restarted.load_latest().unwrap().unwrap();
        assert_eq!(sequence, 7);
        assert_eq!(loaded.stats("u").unwrap().documents_indexed, 1);
        assert_eq!(current_generation(&restarted), active);
    }

    #[test]
    fn corrupt_current_manifest_fails_without_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");
        store.save(&engine, 7).unwrap();
        store.save(&engine, 8).unwrap();
        let current = current_generation(&store);
        let current_path = store.generations.generation_path(&current);
        let mut manifest = read_generation_manifest(&current_path).unwrap();
        manifest.checkpoint_sequence = 999;
        write_generation_manifest(&current_path, &manifest).unwrap();

        let restarted = SegmentRdbStore::new(dir.path()).unwrap();
        assert!(restarted.load_latest().is_err());
        assert_eq!(current_generation(&restarted), current);
    }

    #[test]
    fn unsupported_predecessor_name_blocks_prune_without_deleting_history() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");
        store.save(&engine, 1).unwrap();
        store.save(&engine, 2).unwrap();
        let current = current_generation(&store);
        let current_path = store.generations.generation_path(&current);
        let mut manifest = read_generation_manifest(&current_path).unwrap();
        let predecessor = manifest.previous.clone().unwrap();
        manifest.previous = Some("missing-safe-name".to_owned());
        write_generation_manifest(&current_path, &manifest).unwrap();

        assert!(store.prune(1).is_err());
        assert!(current_path.is_dir());
        assert!(dir.path().join(predecessor).is_dir());
    }

    #[test]
    fn missing_future_predecessor_blocks_prune_without_deleting_history() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");
        store.save(&engine, 1).unwrap();
        store.save(&engine, 2).unwrap();
        let current = current_generation(&store);
        let current_path = store.generations.generation_path(&current);
        let mut manifest = read_generation_manifest(&current_path).unwrap();
        let predecessor = manifest.previous.clone().unwrap();
        manifest.previous = Some("gen-999-rev-1".to_owned());
        write_generation_manifest(&current_path, &manifest).unwrap();

        assert!(store.prune(1).is_err());
        assert!(current_path.is_dir());
        assert!(dir.path().join(predecessor).is_dir());
    }

    #[test]
    fn missing_older_predecessor_makes_prune_conservative() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");
        store.save(&engine, 1).unwrap();
        store.save(&engine, 2).unwrap();
        let current = current_generation(&store);
        let current_path = store.generations.generation_path(&current);
        let mut manifest = read_generation_manifest(&current_path).unwrap();
        let real_predecessor = manifest.previous.clone().unwrap();
        manifest.previous = Some("gen-0-rev-0".to_owned());
        write_generation_manifest(&current_path, &manifest).unwrap();

        assert_eq!(store.prune(1).unwrap(), 0);
        assert!(current_path.is_dir());
        assert!(dir.path().join(real_predecessor).is_dir());
    }

    #[test]
    fn full_reopen_validation_leaves_target_engine_unchanged_on_corruption() {
        let dir = tempfile::tempdir().unwrap();
        let legacy = dir.path().join("gen-42");
        std::fs::create_dir(&legacy).unwrap();
        let source = Arc::new(Engine::new());
        source.create_collection("a", kw_schema()).unwrap();
        source.create_collection("z", kw_schema()).unwrap();
        index_kw_in(&source, "a", "a1", "a@x.com");
        index_kw_in(&source, "z", "z1", "z@x.com");
        source.flush_to_segments(&legacy, 42).unwrap();
        std::fs::write(first_segment_file(&legacy.join("7a")), b"corrupt").unwrap();

        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let target = Arc::new(Engine::new());
        target.create_collection("existing", kw_schema()).unwrap();
        index_kw_in(&target, "existing", "e1", "existing@x.com");

        assert!(store.reopen_into(&target).is_err());
        assert_eq!(
            target.stats("existing").unwrap().documents_indexed,
            1,
            "validation failure must not replace or partly extend the caller engine"
        );
        assert!(target.stats("a").is_err());
        assert!(target.stats("z").is_err());
        assert!(!dir.path().join("CURRENT").exists());
    }

    #[test]
    fn malformed_unpointed_revision_does_not_block_save_or_prune() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");
        store.save(&engine, 1).unwrap();

        let malformed = dir.path().join("gen-200-rev-999");
        std::fs::create_dir(&malformed).unwrap();
        std::fs::write(malformed.join(GENERATION_MANIFEST_FILE), b"not-json").unwrap();

        store.save(&engine, 2).unwrap();
        assert_eq!(store.load_latest().unwrap().unwrap().1, 2);
        assert_eq!(store.prune(1).unwrap(), 2);
        assert!(!malformed.exists());
        assert_eq!(store.generation_seqs().unwrap(), vec![2]);
    }

    #[test]
    fn prune_follows_active_chain_and_removes_aborted_revision() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");
        store.save(&engine, 1).unwrap();
        store.save(&engine, 2).unwrap();
        let active = current_generation(&store);
        let aborted = install_unpointed_generation(&store, &engine, 200, Some(&active));
        let aborted_path = store.generations.generation_path(&aborted);
        store.save(&engine, 3).unwrap();

        assert_eq!(store.prune(2).unwrap(), 2);
        assert!(!aborted_path.exists());
        assert_eq!(store.generation_seqs().unwrap(), vec![2, 3]);
        assert_eq!(store.load_latest().unwrap().unwrap().1, 3);
    }

    #[cfg(unix)]
    #[test]
    fn parseable_generation_symlink_fails_closed() {
        use std::os::unix::fs::symlink;

        let dir = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        symlink(target.path(), dir.path().join("gen-99")).unwrap();

        assert!(SegmentRdbStore::new(dir.path()).is_err());
    }

    /// #1389 AC1: a `reshard:apply` batch applied to a target shard, and a
    /// `reshard:evict` on a source shard, both survive a cold start from a
    /// checkpoint written after those mutations — independent of any
    /// periodic-snapshot cadence, closing the restart gap `#1387`'s embedded
    /// persistence left open for reshard's direct-state-mutation admin verbs
    /// (`Engine::apply_reshard_batch` / `Engine::evict_not_owned`, added by
    /// `#1380`). This is the engine-level half of `#1389`'s proof; the
    /// driver-level half (cutover cannot fire before every touched shard's
    /// checkpoint completes) lives in `e2e/reshard_driver_e2e.rs`.
    #[test]
    fn reshard_apply_and_evict_survive_checkpoint_and_cold_start() {
        use crate::routing::VirtualBucketShardMap;

        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();

        // Target shard: receives a reshard:apply batch on top of its own
        // pre-existing data — mirrors what a shard actually looks like
        // mid-migration.
        let target = Arc::new(Engine::new());
        target.create_collection("u", kw_schema()).unwrap();
        index_kw(&target, "t-existing", "existing@x.com");

        let source = Arc::new(Engine::new());
        source.create_collection("u", kw_schema()).unwrap();
        index_kw(&source, "migrated-1", "migrated1@x.com");
        let batch = source.snapshot().unwrap();
        let apply_outcome = target.apply_reshard_batch(batch, None).unwrap();
        assert_eq!(apply_outcome.documents_upserted, 1);
        assert_eq!(target.stats("u").unwrap().documents_indexed, 2);

        // Source shard: post-cutover eviction of the bucket that just moved
        // off of it, under a 2-shard map where bucket 0 now belongs to shard
        // 1 (mirrors `reshard_evict_removes_only_moved_bucket_docs`).
        let source_after_cutover = Arc::new(Engine::new());
        source_after_cutover
            .create_collection("u", kw_schema())
            .unwrap();
        let ids: Vec<String> = (0..8).map(|i| format!("s-{i:02}")).collect();
        for id in &ids {
            index_kw(&source_after_cutover, id, &format!("{id}@x.com"));
        }
        let mut assignments = vec![0u32; 4];
        assignments[0] = 1;
        let new_map = VirtualBucketShardMap::new(1, assignments, 2).unwrap();
        let evict_outcome = source_after_cutover.evict_not_owned(&new_map, 0).unwrap();
        assert!(evict_outcome.documents_evicted > 0);
        let remaining_before_checkpoint =
            source_after_cutover.stats("u").unwrap().documents_indexed;
        assert!(remaining_before_checkpoint < ids.len() as u64);

        // Checkpoint both post-mutation states, exactly like
        // `checkpoint_touched_shards` (#1389) drives per shard before
        // cutover — this is the synchronous, awaited durability step, not a
        // background snapshot the driver has no visibility into.
        store.save(&target, 100).unwrap();
        let target_docs_before_drop = target.stats("u").unwrap().documents_indexed;
        drop(target);

        let store2 = SegmentRdbStore::new(dir.path().join("source")).unwrap();
        store2.save(&source_after_cutover, 100).unwrap();
        drop(source_after_cutover);

        // Cold start: reload from the checkpoint alone, as a restarted pod
        // would (WAL replay from `seq + 1` is orthogonal to this proof —
        // there are no un-checkpointed writes here).
        let (reloaded_target, seq) = store.load_latest().unwrap().expect("target checkpoint");
        assert_eq!(seq, 100);
        assert_eq!(
            reloaded_target.stats("u").unwrap().documents_indexed,
            target_docs_before_drop
        );

        let (reloaded_source, seq2) = store2.load_latest().unwrap().expect("source checkpoint");
        assert_eq!(seq2, 100);
        assert_eq!(
            reloaded_source.stats("u").unwrap().documents_indexed,
            remaining_before_checkpoint
        );
    }

    /// #1397 AC1: `POST /admin/checkpoint` (the checkpoint sink) and the
    /// periodic snapshotter share one `SegmentRdbStore` and can both fire at
    /// an unchanged `applied_seq` (reshard apply/evict mutate engine state
    /// without advancing `applied_seq`, so this is a routine, not a rare,
    /// interleaving). Loop the interleaving many rounds with several
    /// concurrent `save` callers per round: every round must cold-start to a
    /// complete engine, never a torn one — proving `save_lock` actually
    /// prevents `sweep_staging`/`rename` races rather than merely narrowing
    /// them.
    #[test]
    fn separately_opened_same_root_stores_hold_one_owned_save_permit() {
        let dir = tempfile::tempdir().unwrap();
        let first = SegmentRdbStore::new(dir.path()).unwrap();
        let second = SegmentRdbStore::new(dir.path()).unwrap();
        let permit = first.save_gate.lock_owned();
        let second_gate = second.save_gate.clone();
        let (attempt_tx, attempt_rx) = std::sync::mpsc::channel();
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            attempt_tx.send(()).unwrap();
            let _permit = second_gate.lock_owned();
            done_tx.send(()).unwrap();
        });
        attempt_rx
            .recv_timeout(std::time::Duration::from_secs(1))
            .unwrap();
        let acquired_early = done_rx
            .recv_timeout(std::time::Duration::from_millis(50))
            .is_ok();
        drop(permit);
        if !acquired_early {
            done_rx
                .recv_timeout(std::time::Duration::from_secs(1))
                .unwrap();
        }
        worker.join().unwrap();
        assert!(
            !acquired_early,
            "a second same-root owner acquired before the first permit dropped"
        );
    }

    #[test]
    fn concurrent_saves_at_same_seq_never_produce_torn_checkpoint() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SegmentRdbStore::new(dir.path()).unwrap());
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        for i in 0..20 {
            index_kw(&engine, &format!("u{i:02}"), &format!("u{i:02}@x.com"));
        }
        let expected_docs = engine.stats("u").unwrap().documents_indexed;

        for round in 0..50u64 {
            // Same `up_to_seq` across every concurrent caller this round,
            // mirroring a quiet cutover where `applied_seq` hasn't moved.
            let seq = round;
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    let store = store.clone();
                    let engine = engine.clone();
                    std::thread::spawn(move || store.save(&engine, seq))
                })
                .collect();
            for h in handles {
                h.join().unwrap().unwrap();
            }

            // Cold-start from scratch after the interleaving: the committed
            // generation must always be complete and loadable, never torn.
            let (reloaded, loaded_seq) = store.load_latest().unwrap().expect("a checkpoint");
            assert_eq!(loaded_seq, seq);
            assert_eq!(
                reloaded.stats("u").unwrap().documents_indexed,
                expected_docs,
                "round {round}: cold start after concurrent saves must be complete"
            );
        }
    }

    #[test]
    fn independently_opened_handles_share_checkpoint_preparation_lock() {
        let dir = tempfile::tempdir().unwrap();
        let first = Arc::new(SegmentRdbStore::new(dir.path()).unwrap());
        let second = Arc::new(SegmentRdbStore::new(dir.path()).unwrap());
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");

        for sequence in 1..=10 {
            let left = {
                let store = first.clone();
                let engine = engine.clone();
                std::thread::spawn(move || store.save(&engine, sequence))
            };
            let right = {
                let store = second.clone();
                let engine = engine.clone();
                std::thread::spawn(move || store.save(&engine, sequence))
            };
            left.join().unwrap().unwrap();
            right.join().unwrap().unwrap();
        }

        assert_eq!(first.load_latest().unwrap().unwrap().1, 10);
        assert!(!std::fs::read_dir(dir.path()).unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_str()
                .is_some_and(|name| name.starts_with(".stage-"))
        }));
    }

    #[test]
    fn catalog_rejects_noncanonical_paths_before_filesystem_normalization() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        store.save(&engine, 7).unwrap();
        let path = store
            .generations
            .generation_path(&current_generation(&store));
        let manifest = read_generation_manifest(&path).unwrap();
        for separator in ["/./", "//"] {
            let mut mutated = manifest.clone();
            let reference = &mut mutated.collections[0].segments[0];
            reference.path = reference.path.replacen('/', separator, 1);
            assert!(
                validate_catalog_references(&path, &mutated).is_err(),
                "a raw {separator:?} path component must not be normalized into an accepted path"
            );
        }
    }

    #[test]
    fn v2_manifest_reads_the_approved_collection_and_segment_matrix() {
        let dir = tempfile::tempdir().unwrap();
        let mut collections = Vec::new();
        for collection in 0..182 {
            let mut segments = Vec::new();
            let mut schema = serde_json::Map::new();
            for field in 0..14 {
                let field_name = format!("field{field:02}");
                schema.insert(field_name.clone(), serde_json::json!({"type": "keyword"}));
                for ordinal in 0..=16 {
                    let path =
                        format!("collection{collection:03}/{field_name}/segment{ordinal:02}");
                    segments.push(SegmentReference {
                        role: SegmentRole::Field,
                        field: Some(field_name.clone()),
                        ordinal,
                        kind: if ordinal == 0 {
                            SegmentKind::Base
                        } else {
                            SegmentKind::Delta
                        },
                        format: SegmentFormat::LsegV1,
                        path: format!("{path}.lseg"),
                        local_rows: (ordinal != 0).then(|| LocalRowsReference {
                            format: "lumen-local-eids-cbor-v1".into(),
                            path: format!("{path}.rows.cbor"),
                            count: 1,
                        }),
                        applied_seq: None,
                        payload_sha256: None,
                    });
                }
            }
            collections.push(CollectionCatalog {
                collection_id: format!("collection{collection:03}"),
                collection_generation: collection + 1,
                schema_version: 1,
                data_version: 17,
                schema: serde_json::Value::Object(schema),
                segments,
            });
        }
        let manifest = SegmentGenerationManifest {
            schema_version: 2,
            checkpoint_sequence: 17,
            revision: 1,
            previous: None,
            next_collection_generation: 183,
            collections,
        };
        write_generation_manifest(dir.path(), &manifest).unwrap();
        assert!(
            std::fs::metadata(dir.path().join(GENERATION_MANIFEST_FILE))
                .unwrap()
                .len()
                > 4 * 1024 * 1024
        );
        let loaded = read_generation_manifest(dir.path())
            .expect("the approved 182 collection / 14 field / 16 delta catalog must be readable");
        assert_eq!(loaded.collections.len(), 182);
        assert!(loaded
            .collections
            .iter()
            .all(|collection| collection.segments.len() == 14 * 17));
    }

    #[test]
    fn checkpoint_manifest_starts_v2_complete_catalog() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");
        store.save(&engine, 7).unwrap();

        let generation = current_generation(&store);
        let manifest =
            read_generation_manifest(&store.generations.generation_path(&generation)).unwrap();
        assert_eq!(manifest.schema_version, 2);
    }
    #[test]
    fn unchanged_checkpoint_preserves_data_version_across_sequence_change() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");
        let first = store.save_required(&engine, 31).unwrap();
        let before = read_generation_manifest(&dir.path().join(first.as_str())).unwrap();
        let second = store.save_required(&engine, 32).unwrap();
        let after = read_generation_manifest(&dir.path().join(second.as_str())).unwrap();
        assert_eq!(
            before.collections[0].data_version, after.collections[0].data_version,
            "checkpoint sequence is not a collection data version"
        );
    }

    #[test]
    fn checkpoint_epoch_advances_after_restart_and_truncate() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        let first = store.save_required(&engine, 41).unwrap();
        let before = read_generation_manifest(&dir.path().join(first.as_str())).unwrap();
        let (loaded, _) = store.load_latest().unwrap().unwrap();
        loaded.truncate_docs("u").unwrap();
        let second = store.save_required(&loaded, 42).unwrap();
        let after = read_generation_manifest(&dir.path().join(second.as_str())).unwrap();
        assert!(
            after.collections[0].collection_generation
                > before.collections[0].collection_generation,
            "truncate must allocate beyond the restored epoch"
        );
    }
    #[test]
    fn catalog_rejects_missing_required_base_before_reopening() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        let name = store.save_required(&engine, 12).unwrap();
        let path = dir.path().join(name.as_str());
        let mut manifest = read_generation_manifest(&path).unwrap();
        manifest.collections[0]
            .segments
            .retain(|s| !matches!(s.role, SegmentRole::CollectionEids));
        assert!(
            validate_catalog_references(&path, &manifest).is_err(),
            "required base reference must be validated before reopen"
        );
    }

    #[cfg(unix)]
    #[test]
    fn v1_upgrade_reuses_loaded_immutable_base() {
        use std::os::unix::fs::MetadataExt;
        let dir = tempfile::tempdir().unwrap();
        let legacy = dir.path().join("gen-42-rev-1");
        let source = Arc::new(Engine::new());
        source.create_collection("u", kw_schema()).unwrap();
        index_kw(&source, "u1", "a@x.com");
        source.flush_to_segments(&legacy, 42).unwrap();
        std::fs::write(
            legacy.join(GENERATION_MANIFEST_FILE),
            br#"{"schema_version":1,"sequence":42,"revision":1,"previous":null}"#,
        )
        .unwrap();
        std::fs::write(dir.path().join("CURRENT"), b"generation:gen-42-rev-1\n").unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let (loaded, _) = store.load_latest().unwrap().unwrap();
        let before = first_segment_file(&legacy);
        let next = store.save_required(&loaded, 43).unwrap();
        let after = dir
            .path()
            .join(next.as_str())
            .join(before.strip_prefix(&legacy).unwrap());
        assert_eq!(
            std::fs::metadata(before).unwrap().ino(),
            std::fs::metadata(after).unwrap().ino(),
            "upgrading a loaded v1 base must link unchanged bytes"
        );
    }
    #[test]
    fn new_vector_checkpoints_use_the_checkpoint_watermark() {
        for backend in ["flat-cpu", "hnsw-cpu"] {
            let dir = tempfile::tempdir().unwrap();
            let engine = Engine::new();
            engine.create_collection("u", serde_json::from_value(serde_json::json!({"fields":{"v":{"type":"vector","dim":3,"metric":"cosine","backend":backend}}})).unwrap()).unwrap();
            engine.flush_to_segments(dir.path(), 17).unwrap();
            let reader =
                crate::segment::SegmentReader::open(&dir.path().join("75/v.lseg")).unwrap();
            assert_eq!(
                reader.applied_seq(),
                17,
                "vector header must carry the checkpoint cut, not its row count"
            );
        }
    }
    #[test]
    fn base_checkpoint_field_names_are_paths_only_after_encoding() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine
            .create_collection(
                "names",
                serde_json::from_value(serde_json::json!({
                    "fields": {
                        "../outside": {"type":"keyword"},
                        "_collection.lmeta": {"type":"keyword"},
                        "tag.eids": {"type":"keyword"},
                        "tag": {"type":"vector","dim":2,"metric":"l2","backend":"flat-cpu"}
                    }
                }))
                .unwrap(),
            )
            .unwrap();
        let generation = store
            .save_required(&engine, 17)
            .expect("accepted field names must produce distinct confined checkpoint files");
        let root = dir.path().join(generation.as_str());
        assert!(
            !root.join("outside.lseg").exists(),
            "field name escaped its collection"
        );
        let manifest = read_generation_manifest(&root).unwrap();
        let paths: BTreeSet<_> = manifest.collections[0]
            .segments
            .iter()
            .map(|s| &s.path)
            .collect();
        assert_eq!(
            paths.len(),
            6,
            "collection, four fields and vector IDs must be distinct"
        );
        let (cold, cut) = store.load_latest().unwrap().unwrap();
        assert_eq!(cut, 17);
        assert_eq!(cold.list_collections().unwrap(), vec!["names"]);
    }

    #[test]
    fn checkpoint_file_writes_do_not_hold_the_live_state_lock() {
        use std::sync::mpsc;
        use std::time::Duration;
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        engine.create_collection("idle", kw_schema()).unwrap();
        index_kw(&engine, "doc", "value");
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let writer_engine = engine.clone();
        let writer = std::thread::spawn(move || {
            crate::storage::CHECKPOINT_WRITE_HOOK.with(|hook| {
                *hook.borrow_mut() = Some(Box::new(move || {
                    entered_tx.send(()).unwrap();
                    release_rx.recv_timeout(Duration::from_secs(3)).unwrap();
                }));
            });
            store.save(&writer_engine, 1).unwrap();
        });
        entered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let (query_tx, query_rx) = mpsc::channel();
        let query = std::thread::spawn(move || {
            let request = serde_json::from_value(serde_json::json!({
                "query":{"term":{"field":"email","value":"absent"}}
            }))
            .unwrap();
            query_tx.send(engine.search("idle", request)).unwrap();
        });
        let during_write = query_rx.recv_timeout(Duration::from_millis(250));
        // Release and join before asserting, including on the expected red.
        release_tx.send(()).unwrap();
        writer.join().unwrap();
        query.join().unwrap();
        assert!(
            during_write.is_ok(),
            "idle query blocked by checkpoint file I/O"
        );
        assert!(during_write.unwrap().is_ok());
    }

    #[test]
    fn concurrent_first_base_keeps_an_attachable_overlay_for_the_next_checkpoint() {
        use std::sync::mpsc;
        use std::time::Duration;
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SegmentRdbStore::new(dir.path()).unwrap());
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "base", "base");
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let writer_store = store.clone();
        let writer_engine = engine.clone();
        let writer = std::thread::spawn(move || {
            crate::storage::CHECKPOINT_WRITE_HOOK.with(|hook| {
                *hook.borrow_mut() = Some(Box::new(move || {
                    entered_tx.send(()).unwrap();
                    release_rx.recv_timeout(Duration::from_secs(3)).unwrap();
                }));
            });
            writer_store.save(&writer_engine, 1).unwrap();
        });
        entered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        index_kw(&engine, "later", "later");
        release_tx.send(()).unwrap();
        writer.join().unwrap();
        store.save(&engine, 2).unwrap();
        let request: crate::types::SearchRequest = serde_json::from_value(serde_json::json!({
            "query":{"term":{"field":"email","value":"later"}}
        }))
        .unwrap();
        assert_eq!(engine.search("u", request.clone()).unwrap().total, 1);
        let (cold, _) = store.load_latest().unwrap().unwrap();
        assert_eq!(cold.search("u", request).unwrap().total, 1);
    }

    #[test]
    fn first_base_publication_preserves_newer_field_update_and_deletion() {
        use std::sync::mpsc;
        use std::time::Duration;
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SegmentRdbStore::new(dir.path()).unwrap());
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        let keep: crate::types::FieldSpec =
            serde_json::from_value(serde_json::json!({"type":"keyword"})).unwrap();
        engine.add_field("u", "keep", keep).unwrap();
        for id in ["deleted", "updated"] {
            index_kw(&engine, id, "old");
            engine.index("u", serde_json::from_value(serde_json::json!({"items":[{"external_id":id,"field":"keep","value":"present"}]})).unwrap()).unwrap();
        }
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let writer_store = store.clone();
        let writer_engine = engine.clone();
        let writer = std::thread::spawn(move || {
            crate::storage::CHECKPOINT_WRITE_HOOK.with(|hook| {
                *hook.borrow_mut() = Some(Box::new(move || {
                    entered_tx.send(()).unwrap();
                    release_rx.recv_timeout(Duration::from_secs(3)).unwrap();
                }));
            });
            writer_store.save(&writer_engine, 1).unwrap();
        });
        entered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        engine.delete("u", "deleted", Some("email")).unwrap();
        index_kw(&engine, "updated", "new");
        let search = |engine: &Engine, value: &str| {
            engine
                .search(
                    "u",
                    serde_json::from_value(
                        serde_json::json!({"query":{"term":{"field":"email","value":value}}}),
                    )
                    .unwrap(),
                )
                .unwrap()
                .total
        };
        assert_eq!(search(&engine, "old"), 0);
        release_tx.send(()).unwrap();
        writer.join().unwrap();
        let uncached: crate::types::SearchRequest = serde_json::from_value(
            serde_json::json!({"query":{"term":{"field":"email","value":"old"}},"limit":17}),
        )
        .unwrap();
        assert_eq!(
            engine.search("u", uncached).unwrap().total,
            0,
            "first base must not resurrect a newer update or field deletion"
        );
        assert_eq!(search(&engine, "new"), 1);
        store.save(&engine, 2).unwrap();
        let (cold, _) = store.load_latest().unwrap().unwrap();
        assert_eq!(search(&cold, "old"), 0);
        assert_eq!(search(&cold, "new"), 1);
    }

    #[test]
    fn first_vector_base_publication_releases_only_acknowledged_payloads() {
        use std::sync::mpsc;
        use std::time::Duration;
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SegmentRdbStore::new(dir.path()).unwrap());
        let engine = Arc::new(Engine::new());
        engine
            .create_collection(
                "v",
                serde_json::from_value(serde_json::json!({"fields":{
                    "vec":{"type":"vector","dim":2,"metric":"l2","backend":"flat-cpu"}
                }}))
                .unwrap(),
            )
            .unwrap();
        let index = |eid: &str, value: f32| {
            engine
                .index(
                    "v",
                    serde_json::from_value(serde_json::json!({
                        "items":[{"external_id":eid,"field":"vec","value":[value,value]}]
                    }))
                    .unwrap(),
                )
                .unwrap()
        };
        index("ack", 0.0);
        index("updated", 1.0);
        index("deleted", 2.0);
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let writer_store = store.clone();
        let writer_engine = engine.clone();
        let writer = std::thread::spawn(move || {
            crate::storage::CHECKPOINT_WRITE_HOOK.with(|hook| {
                *hook.borrow_mut() = Some(Box::new(move || {
                    entered_tx.send(()).unwrap();
                    release_rx.recv_timeout(Duration::from_secs(3)).unwrap();
                }));
            });
            writer_store.save(&writer_engine, 1).unwrap();
        });
        entered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        index("updated", 3.0);
        index("new", 4.0);
        engine.delete("v", "deleted", Some("vec")).unwrap();
        let logical_snapshot = |engine: &Engine| {
            let mut value = serde_json::to_value(engine.snapshot().unwrap()).unwrap();
            let field = &mut value["collections"]["v"]["fields"]["vec"];
            field.as_object_mut().unwrap().remove("bytes");
            field["vectors"]
                .as_array_mut()
                .unwrap()
                .sort_by(|a, b| a[0].as_str().cmp(&b[0].as_str()));
            value
        };
        let reference = logical_snapshot(&engine);
        release_tx.send(()).unwrap();
        writer.join().unwrap();
        assert_eq!(
            engine.segment_field_probe("v", "vec").unwrap().0,
            2,
            "first vector base publication must release acknowledged payloads while preserving newer writes"
        );
        assert_eq!(logical_snapshot(&engine), reference);
        store.save(&engine, 2).unwrap();
        assert_eq!(engine.segment_field_probe("v", "vec").unwrap().0, 0);
        let (cold, _) = store.load_latest().unwrap().unwrap();
        assert_eq!(logical_snapshot(&cold), reference);
        assert_eq!(cold.segment_field_probe("v", "vec").unwrap().0, 0);
    }

    #[test]
    fn checkpoint_file_sync_does_not_hold_the_apply_barrier() {
        use std::sync::mpsc;
        use std::time::Duration;
        struct HoldSync(Mutex<Option<(mpsc::Sender<()>, mpsc::Receiver<()>)>>);
        impl FailureInjector for HoldSync {
            fn check(&self, point: &storage_durable::FailurePoint) -> std::io::Result<()> {
                if point.step == storage_durable::CommitStep::SyncFile {
                    if let Some((entered, release)) = self.0.lock().unwrap().take() {
                        entered.send(()).unwrap();
                        release.recv_timeout(Duration::from_secs(3)).unwrap();
                    }
                }
                Ok(())
            }
        }
        let dir = tempfile::tempdir().unwrap();
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let store = SegmentRdbStore::new_with_failure_injector(
            dir.path(),
            Arc::new(HoldSync(Mutex::new(Some((entered_tx, release_rx))))),
        )
        .unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        let saving = engine.clone();
        let save = std::thread::spawn(move || store.save(&saving, 1));
        entered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let (written_tx, written_rx) = mpsc::channel();
        let applying = engine.clone();
        let write = std::thread::spawn(move || {
            index_kw(&applying, "during-sync", "new");
            written_tx.send(()).unwrap();
        });
        let during_sync = written_rx.recv_timeout(Duration::from_millis(250));
        release_tx.send(()).unwrap();
        save.join().unwrap().unwrap();
        write.join().unwrap();
        assert!(
            during_sync.is_ok(),
            "apply blocked by checkpoint file fsync"
        );
    }

    #[test]
    fn flat_checkpoint_syncs_payload_and_generation_root_for_one_changed_collection() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        struct CountDirectorySync(Arc<AtomicUsize>);
        impl FailureInjector for CountDirectorySync {
            fn check(&self, point: &storage_durable::FailurePoint) -> std::io::Result<()> {
                if point.step == storage_durable::CommitStep::SyncDirectory {
                    self.0.fetch_add(1, Ordering::Relaxed);
                }
                Ok(())
            }
        }

        let dir = tempfile::tempdir().unwrap();
        let syncs = Arc::new(AtomicUsize::new(0));
        let store = SegmentRdbStore::new_with_failure_injector(
            dir.path(),
            Arc::new(CountDirectorySync(syncs.clone())),
        )
        .unwrap();
        let engine = Arc::new(Engine::new());
        for collection in ["c0", "c1", "c2", "c3", "c4"] {
            engine.create_collection(collection, kw_schema()).unwrap();
        }
        index_kw_in(&engine, "c0", "c0", "first");
        store.save(&engine, 1).unwrap();
        syncs.store(0, Ordering::Relaxed);

        index_kw_in(&engine, "c0", "c0", "second");
        store.save(&engine, 2).unwrap();

        // v3 stores all durable payload files under one flat directory. The
        // commit still syncs that directory and the generation root, but it
        // no longer creates or syncs one directory per unchanged collection.
        assert!(
            syncs.load(Ordering::Relaxed) >= 2,
            "checkpoint must sync payload data and the generation root"
        );
    }

    #[test]
    fn restore_during_checkpoint_sync_rejects_the_old_epoch_before_current() {
        use std::sync::mpsc;
        use std::time::Duration;
        struct HoldSync(Mutex<Option<(mpsc::Sender<()>, mpsc::Receiver<()>)>>);
        impl FailureInjector for HoldSync {
            fn check(&self, point: &storage_durable::FailurePoint) -> std::io::Result<()> {
                if point.step == storage_durable::CommitStep::SyncFile {
                    if let Some((entered, release)) = self.0.lock().unwrap().take() {
                        entered.send(()).unwrap();
                        release.recv_timeout(Duration::from_secs(3)).unwrap();
                    }
                }
                Ok(())
            }
        }
        let dir = tempfile::tempdir().unwrap();
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let store = SegmentRdbStore::new_with_failure_injector(
            dir.path(),
            Arc::new(HoldSync(Mutex::new(Some((entered_tx, release_rx))))),
        )
        .unwrap();
        let original = std::fs::read(dir.path().join("CURRENT")).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        let saving = engine.clone();
        let save = std::thread::spawn(move || store.save(&saving, 1));
        entered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let (restored_tx, restored_rx) = mpsc::channel();
        let restoring = engine.clone();
        let restore = std::thread::spawn(move || {
            let replacement = Engine::new();
            replacement
                .create_collection("replacement", kw_schema())
                .unwrap();
            restoring.restore(replacement.snapshot().unwrap()).unwrap();
            restored_tx.send(()).unwrap();
        });
        let during_sync = restored_rx.recv_timeout(Duration::from_millis(250));
        release_tx.send(()).unwrap();
        let result = save.join().unwrap();
        restore.join().unwrap();
        assert!(during_sync.is_ok(), "restore blocked by file fsync");
        let error = result.expect_err("old engine epoch must not publish");
        assert!(format!("{error:#}").contains("restored after capture"));
        assert_eq!(std::fs::read(dir.path().join("CURRENT")).unwrap(), original);
        assert_eq!(engine.list_collections().unwrap(), vec!["replacement"]);
    }
    #[test]
    fn checkpoint_validation_does_not_rebuild_collections() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");
        crate::storage::CHECKPOINT_COLLECTION_OPENS.with(|count| count.set(0));
        store.save_required(&engine, 51).unwrap();
        store.save_required(&engine, 52).unwrap();
        assert_eq!(
            crate::storage::CHECKPOINT_COLLECTION_OPENS.with(|count| count.get()),
            0,
            "checkpoint validation must inspect files without rebuilding an Engine"
        );
    }
    #[test]
    fn large_keyword_survives_each_checkpoint_and_merge() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        let mut values = Vec::new();
        for ordinal in 0..6u64 {
            let mut state = (ordinal + 1).wrapping_mul(0x9E37_79B9);
            let mut bytes = vec![0; 6 * 1024 * 1024 - 32 * 1024];
            for byte in &mut bytes {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                *byte = b'a' + (state % 26) as u8;
            }
            let marker = format!("capacity-row-{ordinal:02}-");
            bytes[..marker.len()].copy_from_slice(marker.as_bytes());
            let value = String::from_utf8(bytes).unwrap();
            index_kw(&engine, &format!("row-{ordinal}"), &value);
            assert!(has_keyword(&engine, &value), "live row {ordinal}");
            values.push(value);
            store.save_required(&engine, ordinal + 1).unwrap();
            for (row, value) in values.iter().enumerate() {
                assert!(
                    has_keyword(&engine, value),
                    "row {row} after checkpoint {ordinal}"
                );
            }
            store
                .wait_for_merges(std::time::Duration::from_secs(30))
                .unwrap();
            for (row, value) in values.iter().enumerate() {
                assert!(
                    has_keyword(&engine, value),
                    "row {row} after merge {ordinal}"
                );
            }
        }
        assert!(engine.metrics().segment_merge_completed_total.get() > 0);
        let (cold, sequence) = store.load_latest().unwrap().unwrap();
        assert_eq!(sequence, 6);
        for (row, value) in values.iter().enumerate() {
            assert!(has_keyword(&cold, value), "row {row} after cold reopen");
        }
    }

    #[test]
    fn keyword_checkpoint_writes_only_changed_local_rows() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", kw_schema()).unwrap();
        index_kw(&engine, "u1", "a@x.com");
        index_kw(&engine, "u2", "b@x.com");
        store.save_required(&engine, 61).unwrap();
        index_kw(&engine, "u1", "new@x.com");
        let next = store.save_required(&engine, 62).unwrap();
        let manifest = read_generation_manifest(&dir.path().join(next.as_str())).unwrap();
        let deltas: Vec<_> = manifest.collections[0]
            .segments
            .iter()
            .filter(|s| matches!(s.kind, SegmentKind::Delta) && s.applied_seq == Some(62))
            .collect();
        assert_eq!(deltas.len(), 1, "keyword update must write a delta");
        assert_eq!(deltas[0].local_rows.as_ref().unwrap().count, 1);
    }

    fn text_schema() -> CreateCollectionRequest {
        let mut schema = kw_schema();
        schema.fields.get_mut("email").unwrap().field_type = FieldType::Text;
        schema
    }

    /// Index one Text value the way a committed WAL record does: the value is
    /// staged into its own row reader before apply, so it lands in
    /// `TextIndex::staged_rows` instead of the in-RAM token map.
    fn committed_text(engine: &Arc<Engine>, external_id: &str, value: &str, sequence: u64) {
        let bytes = crate::wal::WalRecord::new(crate::log_entry::RaftLogEntry::Index {
            collection_id: "u".into(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: external_id.into(),
                    field: "email".into(),
                    value: FieldValue::String(value.into()),
                    version: None,
                }],
                request_id: None,
            },
        })
        .encode()
        .unwrap();
        let scanner = crate::wal::fast_index_scanner::FastIndexScanner::parse(&bytes).unwrap();
        let mut completed = None;
        assert!(engine
            .try_apply_committed_index(&scanner, sequence, |apply, outcome| {
                apply.advance_sequence(sequence);
                completed = Some(outcome);
            })
            .unwrap());
        completed.expect("committed Index must complete").unwrap();
    }

    /// #4246 drain guard: a published checkpoint generation must release the
    /// staged Text rows it captured, so the staged set stays bounded by one
    /// checkpoint interval of writes instead of growing for the life of the
    /// process.
    #[test]
    fn published_checkpoint_releases_the_staged_text_rows_it_captured() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SegmentRdbStore::new(dir.path()).unwrap());
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", text_schema()).unwrap();
        for n in 0..200u64 {
            committed_text(&engine, &format!("u{n}"), &format!("alpha term{n}"), n + 1);
        }
        assert_eq!(
            engine.staged_text_row_count("u", "email"),
            200,
            "committed Text applies through staged rows"
        );

        store.save_required(&engine, 100_000).unwrap();
        assert_eq!(
            engine.staged_text_row_count("u", "email"),
            0,
            "the first published generation must absorb every captured row"
        );

        // Rows written while the checkpoint runs belong to the NEXT generation.
        let writer_engine = engine.clone();
        let writer = std::thread::spawn(move || {
            let mut seq = 200_000u64;
            for n in 200..600u64 {
                seq += 1;
                committed_text(
                    &writer_engine,
                    &format!("u{n}"),
                    &format!("alpha term{n}"),
                    seq,
                );
            }
        });
        store.save_required(&engine, 150_000).unwrap();
        writer.join().unwrap();

        store.save_required(&engine, 300_000).unwrap();
        assert_eq!(
            engine.staged_text_row_count("u", "email"),
            0,
            "a quiescent generation must leave no staged row behind"
        );
        assert_eq!(
            engine.stats("u").unwrap().fields["email"].unique_terms,
            601,
            "alpha plus one private term per document"
        );
    }

    /// #4246: `/stats` counts distinct Text terms. Counting must not
    /// materialize a single posting — at 500k documents the owned form
    /// allocated two vectors per term per request.
    #[test]
    fn unique_term_count_over_a_sealed_segment_copies_no_posting() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SegmentRdbStore::new(dir.path()).unwrap());
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", text_schema()).unwrap();
        for n in 0..120u64 {
            committed_text(&engine, &format!("u{n}"), &format!("alpha term{n}"), n + 1);
        }
        store.save_required(&engine, 100_000).unwrap();

        crate::composed_segment::reset_text_posting_clones();
        let stats = engine.stats("u").unwrap();
        assert_eq!(
            stats.fields["email"].unique_terms, 121,
            "alpha plus one private term per document"
        );
        assert_eq!(
            crate::composed_segment::text_posting_clones(),
            0,
            "the distinct-term count must stream the dictionary, not copy postings"
        );
    }

    /// #4246: `unique_terms` must cost the same whether the sealed dictionary
    /// holds 200 terms or 2000. Both engines carry the SAME staged-row work, so
    /// any difference in probes is dictionary work — which must be zero.
    #[test]
    fn unique_term_count_cost_is_independent_of_the_sealed_dictionary_size() {
        let probes_for = |sealed: u64| -> u64 {
            let dir = tempfile::tempdir().unwrap();
            let store = Arc::new(SegmentRdbStore::new(dir.path()).unwrap());
            let engine = Arc::new(Engine::new());
            engine.create_collection("u", text_schema()).unwrap();
            // Seed the sealed corpus through the owned path: it needs no staged
            // row per document, so the dictionary can be large without holding
            // thousands of live change-budget reservations at once.
            for n in 0..sealed {
                index_kw(&engine, &format!("u{n}"), &format!("alpha term{n}"));
            }
            store.save_required(&engine, 900_000).unwrap();
            assert_eq!(engine.staged_text_row_count("u", "email"), 0);
            // The same eight staged rows in both runs.
            for n in 0..8u64 {
                committed_text(
                    &engine,
                    &format!("s{n}"),
                    &format!("alpha staged{n}"),
                    1_000_000 + n,
                );
            }
            crate::composed_segment::reset_text_term_probes();
            let stats = engine.stats("u").unwrap();
            assert_eq!(
                stats.fields["email"].unique_terms,
                sealed + 8 + 1,
                "one private term per sealed doc, one per staged doc, plus alpha"
            );
            crate::composed_segment::text_term_probes()
        };
        let small = probes_for(200);
        let large = probes_for(2_000);
        assert_eq!(
            small, large,
            "a ten-fold larger dictionary must not cost one extra term visit or posting decode"
        );
        assert_eq!(
            large, 0,
            "the count must not visit a dictionary term or decode a posting at all"
        );
    }

    /// #4246: the O(1) count must stay EXACT across every source a Text field
    /// composes — sealed base, published delta, staged rows, live tail — and a
    /// fully-deleted token must still drop out.
    #[test]
    fn unique_term_count_stays_exact_across_deletes_delta_and_staged_rows() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SegmentRdbStore::new(dir.path()).unwrap());
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", text_schema()).unwrap();
        let terms = |engine: &Engine| engine.stats("u").unwrap().fields["email"].unique_terms;

        committed_text(&engine, "u0", "alpha one", 1);
        committed_text(&engine, "u1", "alpha two", 2);
        committed_text(&engine, "u2", "beta three", 3);
        committed_text(&engine, "u3", "beta four", 4);
        store.save_required(&engine, 100_000).unwrap();
        assert_eq!(terms(&engine), 6, "alpha beta one two three four");

        // `one` lived only in the deleted document and must drop out; `alpha`
        // survives in u1.
        engine.delete("u", "u0", Some("email")).unwrap();
        assert_eq!(terms(&engine), 5, "a fully deleted token drops out");
        assert_eq!(terms(&engine), 5, "the memoized answer is the same answer");

        // Staged rows on top of a tombstoned base.
        committed_text(&engine, "u4", "gamma five", 200_001);
        committed_text(&engine, "u5", "alpha six", 200_002);
        assert_eq!(terms(&engine), 8, "plus gamma five six");

        // Publish: the staged rows become a delta layer, the tombstone stays.
        store.save_required(&engine, 300_000).unwrap();
        assert_eq!(engine.staged_text_row_count("u", "email"), 0);
        assert_eq!(terms(&engine), 8, "publication does not change the count");

        // Delete a delta-layer document: `gamma` and `five` were only ever in it.
        engine.delete("u", "u4", Some("email")).unwrap();
        assert_eq!(terms(&engine), 6, "gamma and five drop out together");

        // A fresh staged row re-introduces one of the dropped tokens.
        committed_text(&engine, "u6", "gamma seven", 400_001);
        assert_eq!(terms(&engine), 8, "gamma is live again, plus seven");
    }

    /// #4246: under a pending delete the count walks the composed dictionary
    /// once, decoding each posting only to its first live docid, and still
    /// materializes no posting — the path every production reader takes once
    /// the workload has deleted anything.
    #[test]
    fn unique_term_count_under_pending_deletes_copies_no_posting() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SegmentRdbStore::new(dir.path()).unwrap());
        let engine = Arc::new(Engine::new());
        engine.create_collection("u", text_schema()).unwrap();
        for n in 0..300u64 {
            committed_text(&engine, &format!("u{n}"), &format!("alpha term{n}"), n + 1);
        }
        store.save_required(&engine, 100_000).unwrap();
        assert_eq!(engine.staged_text_row_count("u", "email"), 0);
        engine.delete("u", "u7", Some("email")).unwrap();
        crate::composed_segment::reset_text_posting_clones();
        let stats = engine.stats("u").unwrap();
        assert_eq!(
            stats.fields["email"].unique_terms, 300,
            "alpha plus one private term per surviving document"
        );
        assert_eq!(
            crate::composed_segment::text_posting_clones(),
            0,
            "the tombstone walk must not copy a posting"
        );
    }

    #[test]
    fn text_checkpoint_writes_changed_row_with_coherent_local_statistics() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        let mut schema = kw_schema();
        schema.fields.get_mut("email").unwrap().field_type = FieldType::Text;
        engine.create_collection("u", schema).unwrap();
        index_kw(&engine, "u1", "alpha beta");
        index_kw(&engine, "u2", "alpha gamma");
        store.save_required(&engine, 71).unwrap();
        index_kw(&engine, "u1", "alpha alpha delta");
        let next = store.save_required(&engine, 72).unwrap();
        let root = dir.path().join(next.as_str());
        let manifest = read_generation_manifest(&root).unwrap();
        let deltas: Vec<_> = manifest.collections[0]
            .segments
            .iter()
            .filter(|s| matches!(s.kind, SegmentKind::Delta) && s.applied_seq == Some(72))
            .collect();
        assert_eq!(deltas.len(), 1, "text update must write a delta");
        assert_eq!(deltas[0].local_rows.as_ref().unwrap().count, 1);
        let reader = crate::segment::SegmentReader::open(&root.join(&deltas[0].path)).unwrap();
        assert_eq!(reader.text_doc_count(), 1);
        assert_eq!(reader.text_total_doc_len(), 3);
        assert_eq!(reader.text_doc_len(0), 3);
        assert_eq!(reader.text_postings("alpha"), Some((vec![0], vec![2])));
    }

    #[test]
    fn vector_checkpoint_delta_does_not_enumerate_hnsw_corpus() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        let mut schema = kw_schema();
        let field = schema.fields.get_mut("email").unwrap();
        field.field_type = FieldType::Vector;
        field.dim = Some(2);
        field.metric = Some(crate::types::VectorMetric::L2);
        field.backend = Some(crate::types::VectorBackend::HnswCpu);
        engine.create_collection("u", schema).unwrap();
        let put = |id: &str, value: Vec<f32>| {
            engine
                .index(
                    "u",
                    IndexRequest {
                        items: vec![IndexItem {
                            external_id: id.into(),
                            field: "email".into(),
                            value: FieldValue::Vector(value),
                            version: None,
                        }],
                        request_id: None,
                    },
                )
                .unwrap();
        };
        put("u1", vec![1.0, 0.0]);
        put("u2", vec![0.0, 1.0]);
        store.save_required(&engine, 81).unwrap();
        let scans = crate::vector_index::HNSW_CHECKPOINT_FULL_SCANS.with(|count| count.get());
        put("u1", vec![2.0, 0.0]);
        let next = store.save_required(&engine, 82).unwrap();
        assert_eq!(
            crate::vector_index::HNSW_CHECKPOINT_FULL_SCANS.with(|count| count.get()),
            scans,
            "incremental checkpoint must not enumerate the HNSW corpus"
        );
        let manifest = read_generation_manifest(&dir.path().join(next.as_str())).unwrap();
        let deltas: Vec<_> = manifest.collections[0]
            .segments
            .iter()
            .filter(|s| matches!(s.kind, SegmentKind::Delta) && s.applied_seq == Some(82))
            .collect();
        assert_eq!(deltas.len(), 1, "vector update must write a delta");
        assert_eq!(deltas[0].local_rows.as_ref().unwrap().count, 1);
    }

    #[test]
    fn text_delta_adds_a_previously_absent_base_field_without_subtracting_corpus() {
        let dir = tempfile::tempdir().unwrap();
        let store = SegmentRdbStore::new(dir.path()).unwrap();
        let engine = Arc::new(Engine::new());
        let mut schema = kw_schema();
        let mut text = schema.fields["email"].clone();
        text.field_type = FieldType::Text;
        text.analyzer = Some(crate::types::Analyzer::WhitespaceLower);
        schema.fields.insert("body".into(), text);
        engine.create_collection("u", schema).unwrap();
        index_kw(&engine, "u1", "first");
        index_kw(&engine, "u2", "second");
        let put = |id: &str, value: &str| {
            engine
                .index(
                    "u",
                    IndexRequest {
                        items: vec![IndexItem {
                            external_id: id.into(),
                            field: "body".into(),
                            value: FieldValue::String(value.into()),
                            version: None,
                        }],
                        request_id: None,
                    },
                )
                .unwrap();
        };
        put("u1", "alpha beta");
        store.save_required(&engine, 91).unwrap();
        put("u2", "alpha");
        assert_eq!(
            engine.stats("u").unwrap().fields["body"].avg_doc_len,
            Some(1.5)
        );
        store.save_required(&engine, 92).unwrap();
        let (cold, _) = store.load_latest().unwrap().unwrap();
        assert_eq!(
            cold.stats("u").unwrap().fields["body"].avg_doc_len,
            Some(1.5),
            "adding Text to an absent base field must increase the corpus exactly once"
        );
    }
}
// CODEGEN-END
