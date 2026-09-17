// CODEGEN-BEGIN
//! `RaftHost` — drives a [`raft_core::RaftNode`] for a [`RaftStateMachine`] over
//! an h2c peer transport, with read-your-write `propose` and snapshot/compaction.
//!
//! Generalizes the per-service drivers (relay/lumen/keep each hand-rolled this):
//! the host is the **sole applier** — committed entries are fed to the state
//! machine in index order on a separate worker, so `propose` can return after the
//! command *applies* (not just commits), and `compact(applied, snapshot)` is
//! always sound.

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex as StdMutex, RwLock as StdRwLock};
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use axum::body::{to_bytes, Body};
use axum::extract::{FromRequest, Request, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use raft_core::{
    AppendReq, AppendResp, DemotionRefused, Index, InstallSnapshotReq, InstallSnapshotResp,
    Membership, NodeId, PromotionRefused, RaftMsg, RaftNode, RemovalRefused, TimeoutNowReq,
    TransferRefused, VoteReq, VoteResp,
};
use serde::{Deserialize, Serialize};
use server_lifecycle::ShutdownDeadline;
use tokio::sync::{watch, Mutex, Notify, OwnedMutexGuard};
use tokio::task::{JoinHandle, JoinSet};

use crate::config::{HostConfig, SnapshotPolicy};
use crate::group::{GroupId, LEGACY_GROUP_ID};
use crate::peer_transport::PeerTransport;
use crate::state_machine::{AdmissionPermit, Command, RaftStateMachine, SnapshotPreparation};
use crate::store::RaftStore;

#[path = "host/ordered_apply.rs"]
mod ordered_apply;

/// The maximum size of a snapshot chunk streamed during snapshot generation.
pub const SNAPSHOT_CHUNK_SIZE: usize = 64 * 1024;
const PUBLISH_BODY_LIMIT: usize = 2 * 1024 * 1024;
const DEFAULT_MAX_RESIDENT_LOG_BYTES: usize = 2 * 1024 * 1024 * 1024;

/// A bounded chunk sink that consumes streaming writes in chunks of up to `SNAPSHOT_CHUNK_SIZE`.
pub struct ChunkSink<F = Box<dyn FnMut(&[u8]) -> std::io::Result<()> + Send>> {
    chunk_size: usize,
    buffer: Vec<u8>,
    handler: Option<F>,
    collected: Option<Vec<u8>>,
}

impl ChunkSink<Box<dyn FnMut(&[u8]) -> std::io::Result<()> + Send>> {
    /// Create a collecting sink that accumulates all chunks into an internal buffer.
    pub fn new(chunk_size: usize) -> Self {
        Self::collecting(chunk_size)
    }

    /// Create a collecting sink that accumulates all chunks into an internal buffer.
    pub fn collecting(chunk_size: usize) -> Self {
        Self {
            chunk_size,
            buffer: Vec::with_capacity(chunk_size),
            handler: None,
            collected: Some(Vec::new()),
        }
    }

    /// Create a streaming chunk sink that emits chunks of up to `chunk_size` to `handler`
    /// without unbounded memory retention.
    pub fn streaming<H>(
        chunk_size: usize,
        mut handler: H,
    ) -> ChunkSink<Box<dyn FnMut(&[u8]) -> std::io::Result<()> + Send>>
    where
        H: FnMut(&[u8]) + Send + 'static,
    {
        ChunkSink {
            chunk_size,
            buffer: Vec::with_capacity(chunk_size),
            handler: Some(Box::new(move |chunk| {
                handler(chunk);
                Ok(())
            })),
            collected: None,
        }
    }

    /// Return the accumulated bytes (for collecting mode).
    pub fn into_bytes(mut self) -> Vec<u8> {
        if !self.buffer.is_empty() {
            if let Some(ref mut c) = self.collected {
                c.extend_from_slice(&self.buffer);
            }
        }
        self.collected.unwrap_or(self.buffer)
    }
}

impl<F> std::io::Write for ChunkSink<F>
where
    F: FnMut(&[u8]) -> std::io::Result<()>,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let space = self.chunk_size.saturating_sub(self.buffer.len());
        let to_copy = buf.len().min(space);
        if to_copy > 0 {
            self.buffer.extend_from_slice(&buf[..to_copy]);
        }
        if self.buffer.len() >= self.chunk_size {
            self.flush_chunk()?;
        }
        if to_copy == 0 && !buf.is_empty() {
            self.flush_chunk()?;
            let to_copy = buf.len().min(self.chunk_size);
            self.buffer.extend_from_slice(&buf[..to_copy]);
            return Ok(to_copy);
        }
        Ok(to_copy)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if !self.buffer.is_empty() {
            self.flush_chunk()?;
        }
        Ok(())
    }
}

impl<F> ChunkSink<F>
where
    F: FnMut(&[u8]) -> std::io::Result<()>,
{
    fn flush_chunk(&mut self) -> std::io::Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        if let Some(ref mut handler) = self.handler {
            handler(&self.buffer)?;
            self.buffer.clear();
        } else if let Some(ref mut c) = self.collected {
            c.extend_from_slice(&self.buffer);
            self.buffer.clear();
        }
        Ok(())
    }
}

// --- peer RPC envelopes (the `from` id rides alongside the raft_core message) ---

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct VoteEnvelope {
    pub(crate) group_id: String,
    pub(crate) from: NodeId,
    pub(crate) req: VoteReq,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct AppendEnvelope {
    pub(crate) group_id: String,
    pub(crate) from: NodeId,
    pub(crate) req: AppendReq,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct SnapEnvelope {
    pub(crate) group_id: String,
    pub(crate) from: NodeId,
    pub(crate) req: InstallSnapshotReq,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct CapableSnapEnvelope {
    pub(crate) group_id: String,
    pub(crate) from: NodeId,
    pub(crate) req: InstallSnapshotReq,
    pub(crate) snapshot_capability: String,
    pub(crate) snapshot_nonce: u64,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct CapableSnapshotResp {
    pub(crate) term: u64,
    #[serde(default)]
    pub(crate) accepted: bool,
    pub(crate) snapshot_index: u64,
    pub(crate) snapshot_capability: String,
    pub(crate) snapshot_nonce: u64,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct TimeoutNowEnvelope {
    pub(crate) group_id: String,
    pub(crate) from: NodeId,
    pub(crate) req: TimeoutNowReq,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct PublishEnvelope {
    pub(crate) group_id: String,
    pub(crate) command: Vec<u8>,
}
#[derive(Serialize, Deserialize)]
pub(crate) struct NotLeader {
    pub(crate) error: &'static str,
    pub(crate) leader: Option<NodeId>,
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MembershipPhase {
    Stable,
    Joint,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RaftStatus {
    pub group_id: String,
    pub id: NodeId,
    pub role: String,
    pub term: u64,
    pub commit_index: u64,
    pub last_index: u64,
    pub snapshot_index: u64,
    pub applied_index: u64,
    pub leader: Option<NodeId>,
    pub is_leader: bool,
    pub durability_error: Option<String>,
    pub committed_voters: Vec<NodeId>,
    pub incoming_voters: Option<Vec<NodeId>>,
    pub learners: Vec<NodeId>,
    pub membership_phase: MembershipPhase,
    pub undeliverable_never_addressed: u64,
    pub undeliverable_withdrawn_address: u64,
    /// Proposals refused by `propose_outcome` before route selection.
    pub proposal_rejected_before_routing: u64,
    /// Proposals refused by the leader-side check before log append.
    pub proposal_rejected_before_append: u64,
    pub proposal_admission_closed: bool,
    pub lifecycle_generation: u64,
    #[serde(default)]
    pub snapshot_capability: Option<String>,
    #[serde(default)]
    pub resident_log_bytes: u64,
    #[serde(default)]
    pub max_resident_log_bytes: u64,
}

/// Result of one externally coordinated snapshot attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapshotCompactionOutcome {
    pub snapshot_index: Index,
    pub installed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageFailed {
    pub node_id: NodeId,
    pub operation: &'static str,
    pub path: std::path::PathBuf,
    pub kind: std::io::ErrorKind,
}

impl std::fmt::Display for StorageFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "durable storage failed for node {} on {} at {}: {:?}",
            self.node_id,
            self.operation,
            self.path.display(),
            self.kind
        )
    }
}

impl std::error::Error for StorageFailed {}

/// Terminal outcome of a Raft proposal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposalOutcome {
    Completed {
        index: Index,
    },
    RejectedBeforeAdmission {
        reason: String,
    },
    Ambiguous {
        index: Option<Index>,
        reason: String,
    },
    DurabilityFailure {
        index: Option<Index>,
        failure: StorageFailed,
    },
}

/// A leader has no capacity for a proposal before it allocated a Raft index.
/// Callers can downcast the error returned by [`RaftHost::propose`] to select a
/// retry policy without treating an uncertain append as safely retryable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalBackpressure {
    pub reason: String,
    pub retry_after_seconds: u64,
}

impl std::fmt::Display for ProposalBackpressure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}; retry after {} seconds",
            self.reason, self.retry_after_seconds
        )
    }
}

impl std::error::Error for ProposalBackpressure {}

const BACKPRESSURE_REASON_PREFIX: &str = "\u{1e}raft-proposal-backpressure:";

fn encode_backpressure(backpressure: &ProposalBackpressure) -> String {
    format!(
        "{BACKPRESSURE_REASON_PREFIX}{}",
        serde_json::json!({
            "reason": backpressure.reason,
            "retry_after_seconds": backpressure.retry_after_seconds,
        })
    )
}

fn decode_backpressure(reason: &str) -> Option<ProposalBackpressure> {
    let body = reason.strip_prefix(BACKPRESSURE_REASON_PREFIX)?;
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    Some(ProposalBackpressure {
        reason: value.get("reason")?.as_str()?.to_owned(),
        retry_after_seconds: value.get("retry_after_seconds")?.as_u64()?,
    })
}

fn rejected_admission(error: anyhow::Error) -> ProposalOutcome {
    if let Some(backpressure) = error.downcast_ref::<ProposalBackpressure>() {
        ProposalOutcome::RejectedBeforeAdmission {
            reason: encode_backpressure(backpressure),
        }
    } else {
        ProposalOutcome::RejectedBeforeAdmission {
            reason: error.to_string(),
        }
    }
}

/// The outcome of an attempt to hand off leadership before shutdown (#3664).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeadershipHandoff {
    Transferred { target: NodeId },
    NotLeader,
    SoleVoter,
    NoCaughtUpVoter { voters: usize },
}

/// The four sequential phases of host shutdown (#3672).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShutdownPhase {
    Quiesce,
    LeadershipHandoff,
    BackgroundTasks,
    PeerRpcDrain,
}

/// The status of an individual shutdown phase (#3672).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhaseStatus {
    Completed,
    DeadlineExpired,
    StorageFailed,
}

/// A record of one shutdown phase execution (#3672).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhaseRecord {
    pub phase: ShutdownPhase,
    pub status: PhaseStatus,
    pub elapsed: Duration,
}

/// Which caller role this report represents in host shutdown (#3683).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShutdownCaller {
    Executed,
    Joined,
}

/// The terminal report returned by `shutdown_within` (#3672, #3683).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostShutdownReport {
    pub caller: ShutdownCaller,
    pub phases: Vec<PhaseRecord>,
    pub handoff: LeadershipHandoff,
    pub incomplete_phase: Option<ShutdownPhase>,
    pub peer_listener_close_safe: bool,
    pub storage_failure: Option<StorageFailed>,
}

impl HostShutdownReport {
    /// Convert this shutdown report into a `Result<()>`.
    ///
    /// Returns `Ok(())` if shutdown completed cleanly. Returns `Err` if a storage
    /// failure was observed or if shutdown stopped early in any phase.
    pub fn into_result(self) -> Result<()> {
        if let Some(failure) = self.storage_failure {
            return Err(anyhow!("{failure}"));
        }
        if let Some(phase) = self.incomplete_phase {
            return Err(anyhow!("raft: shutdown stopped early in phase {phase:?}"));
        }
        Ok(())
    }
}

/// Return the absolute cumulative cutoff for each sequential shutdown phase.
///
/// The final cutoff is the caller's original usable end. Any duration that
/// does not divide evenly into quarters remains available to the final phase.
fn shutdown_phase_cutoffs(
    deadline: ShutdownDeadline,
    started_at: tokio::time::Instant,
) -> [tokio::time::Instant; 4] {
    let usable_end = deadline.expires_at - deadline.reserve;
    let usable_interval = usable_end.saturating_duration_since(started_at);
    let quarter = usable_interval / 4;

    [
        started_at + quarter,
        started_at + quarter * 2,
        started_at + quarter * 3,
        usable_end,
    ]
}

/// Return whether a shutdown phase has reached its cumulative cutoff.
fn shutdown_phase_cutoff_elapsed(cutoff: tokio::time::Instant) -> bool {
    tokio::time::Instant::now() >= cutoff
}

/// Why a leader refused a learner admission request.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdmissionRefused {
    Unroutable { target: NodeId },
    NotLeaderOrTransferInFlight,
}

impl std::fmt::Display for AdmissionRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdmissionRefused::Unroutable { target } => {
                write!(f, "no address registered for peer {target}")
            }
            AdmissionRefused::NotLeaderOrTransferInFlight => {
                write!(f, "not leader or leadership transfer in flight")
            }
        }
    }
}

impl std::error::Error for AdmissionRefused {}

pub(crate) struct Shared {
    pub(crate) id: NodeId,
    pub(crate) group_id: GroupId,
    pub(crate) node: Mutex<RaftNode>,
    pub(crate) store: RaftStore,
    pub(crate) sm: Arc<dyn RaftStateMachine>,
    /// Application permits become host-owned immediately after Raft assigned
    /// their index. They are keyed by index, never command content, so a caller
    /// cancellation cannot release a live appended command.
    pub(crate) pending_admission: StdMutex<BTreeMap<(Index, u64), AdmissionPermit>>,
    pub(crate) peers: StdRwLock<HashMap<NodeId, String>>,
    /// One coalescing RPC lane per peer. Raft's latest AppendEntries contains
    /// the complete missing suffix, so retaining every intermediate request
    /// only creates out-of-order progress and repeated durable writes.
    pub(crate) peer_lanes: StdRwLock<HashMap<NodeId, Arc<PeerLane>>>,
    pub(crate) client: reqwest::Client,
    pub(crate) peer_transport: Option<PeerTransport>,
    /// Fires (with the SM's applied head) whenever apply advances.
    pub(crate) applied_tx: watch::Sender<Index>,
    pub(crate) cfg: HostConfig,
    pub(crate) rpc_tracker: Arc<RpcTracker>,
    pub(crate) latched_failure: StdMutex<Option<StorageFailed>>,
    pub(crate) undeliverable_never_addressed: AtomicU64,
    pub(crate) undeliverable_withdrawn_address: AtomicU64,
    pub(crate) proposal_rejected_before_routing: AtomicU64,
    pub(crate) proposal_rejected_before_append: AtomicU64,
    pub(crate) lifecycle_generation: AtomicU64,
    pub(crate) snapshot_nonce: AtomicU64,
    pub(crate) snapshot_rpc_timeout: Duration,
    pub(crate) snapshot_install: Arc<Mutex<()>>,
    pub(crate) apply_running: AtomicBool,
    pub(crate) apply_stopped: AtomicBool,
    pub(crate) apply_tracker: Arc<RpcTracker>,
    pub(crate) max_resident_log_bytes: usize,
    pub(crate) shutdown_started: AtomicBool,
    pub(crate) shutdown_tx: watch::Sender<Option<HostShutdownReport>>,
}

#[derive(Default)]
pub(crate) struct RpcTracker {
    pub(crate) active: AtomicUsize,
    pub(crate) idle: Notify,
}

/// Pending work for one peer lane.
///
/// Only consecutive, unsent `Append` requests may coalesce. Every other Raft
/// message is a FIFO barrier: a later heartbeat must neither replace nor pass
/// a pending control message such as `TimeoutNow`.
#[derive(Default)]
pub(crate) struct PeerLaneQueue {
    messages: VecDeque<RaftMsg>,
}

impl PeerLaneQueue {
    pub(crate) fn enqueue(&mut self, message: RaftMsg) {
        match message {
            RaftMsg::Append(next) => {
                if let Some(RaftMsg::Append(pending)) = self.messages.back_mut() {
                    *pending = next;
                } else {
                    self.messages.push_back(RaftMsg::Append(next));
                }
            }
            control => self.messages.push_back(control),
        }
    }

    pub(crate) fn dequeue(&mut self) -> Option<RaftMsg> {
        self.messages.pop_front()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub(crate) fn len(&self) -> usize {
        self.messages.len()
    }
}

/// Apply the durable Raft state into a caller supplied state machine.  The
/// production host and the deterministic conformance host intentionally share
/// this primitive so cold replay, installed snapshots, and command ordering do
/// not have two subtly different implementations.
pub(crate) fn apply_ready(
    node: &mut RaftNode,
    sm: &dyn RaftStateMachine,
    applied_tx: Option<&watch::Sender<Index>>,
    snapshot_policy: SnapshotPolicy,
    strict: bool,
) -> anyhow::Result<()> {
    apply_ready_with_admission(node, sm, applied_tx, snapshot_policy, strict, None)
}

fn apply_ready_with_admission(
    node: &mut RaftNode,
    sm: &dyn RaftStateMachine,
    applied_tx: Option<&watch::Sender<Index>>,
    snapshot_policy: SnapshotPolicy,
    strict: bool,
    pending_admission: Option<&StdMutex<BTreeMap<(Index, u64), AdmissionPermit>>>,
) -> anyhow::Result<()> {
    if let Some(bytes) = node.take_installed_snapshot() {
        sm.restore(&mut std::io::Cursor::new(bytes))?;
    }
    let mut advanced = false;
    while let Some((index, term, kind)) = node.peek_next_committed_identity() {
        let permit = pending_admission.and_then(|pending| {
            let mut pending = pending.lock().unwrap_or_else(|p| p.into_inner());
            pending.retain(|(at, entry_term), _| *at != index || *entry_term == term);
            pending.remove(&(index, term))
        });
        if kind == raft_core::EntryKind::Command && index > sm.applied_index() {
            // Cold start and deterministic conformance own the node directly.
            // Borrow exactly this command; never materialize a committed batch.
            let persisted = node.persisted_ref();
            let offset = (index - persisted.snapshot_index - 1) as usize;
            let entry = &persisted.log[offset];
            sm.apply_admitted(index, &entry.command, permit)?;
            if sm.applied_index() < index {
                anyhow::bail!("state machine returned success without applying index {index}");
            }
            advanced = true;
        }
        if !node.finish_committed_identity(index, term) {
            anyhow::bail!("committed identity changed while applying index {index}");
        }
    }
    if advanced {
        if let Some(tx) = applied_tx {
            tx.send_replace(sm.applied_index());
        }
    }
    let SnapshotPolicy::EveryEntries(every) = snapshot_policy else {
        return Ok(());
    };
    let applied = sm.applied_index();
    if applied == 0 || applied.saturating_sub(node.snapshot_index()) < every {
        return Ok(());
    }
    let mut sink = ChunkSink::new(SNAPSHOT_CHUNK_SIZE);
    match sm.snapshot_at(applied, &mut sink) {
        Ok(()) => node.compact(applied, sink.into_bytes()),
        Err(e) if strict => return Err(e),
        Err(e) => tracing::warn!(error = %e, "raft: snapshot capture failed; skip compaction"),
    }
    Ok(())
}

/// Cold-start uses the same applier as every ordinary step.  It intentionally
/// leaves store-load policy to the caller: the production host retains its
/// historical best-effort load behavior, while conformance opening returns a
/// load error to its caller.
pub(crate) fn cold_start(
    node: &mut RaftNode,
    sm: &dyn RaftStateMachine,
    strict: bool,
) -> anyhow::Result<()> {
    apply_ready(node, sm, None, SnapshotPolicy::Disabled, strict)
}

/// Persist exactly the core durable image.  The production wrapper adds its
/// latched-failure policy; deterministic conformance returns this error to the
/// scheduler.  Both therefore save identical bytes at identical step points.
pub(crate) fn persist_node(store: &RaftStore, node: &RaftNode) -> std::io::Result<()> {
    store.save_ref(&node.persisted_ref())
}

/// Advance one periodic tick without revalidating an unchanged durable image.
/// `RaftNode::tick` currently changes durable state only when it advances the
/// term to start an election. The first tick persists a fresh node because a
/// learner may otherwise never elect and would leave its membership unsaved.
fn tick_then_maybe_persist<P, L>(
    node: &mut RaftNode,
    first_tick: &mut bool,
    persist: P,
    is_latched: L,
) -> bool
where
    P: FnOnce(&RaftNode) -> bool,
    L: FnOnce() -> bool,
{
    let term_before = node.current_term();
    node.tick();
    let must_persist = std::mem::replace(first_tick, false)
        || node.current_term() != term_before;
    if must_persist && !persist(node) {
        return false;
    }
    !is_latched()
}

#[derive(Default)]
pub(crate) struct PeerLane {
    pending: Mutex<PeerLaneQueue>,
    running: AtomicBool,
}

impl RpcTracker {
    pub(crate) async fn wait_idle(&self) {
        loop {
            let notified = self.idle.notified();
            if self.active.load(Ordering::SeqCst) == 0 {
                return;
            }
            notified.await;
        }
    }
}

struct RpcGuard {
    tracker: Arc<RpcTracker>,
}

impl Drop for RpcGuard {
    fn drop(&mut self) {
        if self.tracker.active.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.tracker.idle.notify_waiters();
        }
    }
}

async fn preflight_snapshot_with_serial(
    sm: Arc<dyn RaftStateMachine>,
    snapshot_install: Arc<Mutex<()>>,
    operation: Arc<RpcGuard>,
) -> Result<(Option<Box<dyn SnapshotPreparation>>, OwnedMutexGuard<()>)> {
    loop {
        let sm = Arc::clone(&sm);
        let preflight_operation = Arc::clone(&operation);
        let preparation = tokio::task::spawn_blocking(move || {
            // The caller can be cancelled while a product waits for checkpoint
            // capacity. Keep the drain lease until that worker releases it.
            let _operation = preflight_operation;
            sm.preflight_snapshot()
        })
        .await??;
        match Arc::clone(&snapshot_install).try_lock_owned() {
            Ok(state_machine_lease) => return Ok((preparation, state_machine_lease)),
            Err(_) => {
                // A preparation can own a product save permit. Do not retain
                // it while an incoming restore owns this serial lease.
                drop(preparation);
                let state_machine_lease = Arc::clone(&snapshot_install).lock_owned().await;
                drop(state_machine_lease);
            }
        }
    }
}

impl Shared {
    /// A public applied head requires both a completed callback and the current
    /// state machine's durable floor. A consumer reporting a lower floor must
    /// still be treated as lagging by all-voter safety barriers.
    fn completed_applied_index(&self) -> Index {
        (*self.applied_tx.borrow()).min(self.sm.applied_index())
    }

    /// Register direct peer work before checking the shutdown gate. The shared
    /// ordering with shutdown and wait_idle means an admitted operation cannot
    /// be missed by the drain. Proposal quiesce alone still allows final flush.
    fn begin_coordinated_peer_work(&self) -> Result<Arc<RpcGuard>> {
        let tracker = Arc::clone(&self.rpc_tracker);
        tracker.active.fetch_add(1, Ordering::SeqCst);
        let guard = Arc::new(RpcGuard { tracker });
        if self.shutdown_started.load(Ordering::SeqCst) {
            return Err(anyhow!(
                "raft: coordinated peer work is closed because shutdown has started"
            ));
        }
        Ok(guard)
    }

    fn http_client(&self) -> reqwest::Client {
        self.peer_transport
            .as_ref()
            .map(PeerTransport::http_client)
            .unwrap_or_else(|| self.client.clone())
    }

    fn persist(&self, node: &RaftNode) -> Result<(), StorageFailed> {
        if let Some(err) = self.latched_failure.lock().unwrap().clone() {
            return Err(err);
        }
        match persist_node(&self.store, node) {
            Ok(()) => Ok(()),
            Err(e) => {
                let err = StorageFailed {
                    node_id: self.id,
                    operation: "save",
                    path: self.store.path().to_path_buf(),
                    kind: e.kind(),
                };
                *self.latched_failure.lock().unwrap() = Some(err.clone());
                Err(err)
            }
        }
    }

    /// Schedule the sole ordered worker after the selected core state is durable.
    /// The caller holds node only for this metadata-only scheduling boundary.
    fn apply_ready(self: &Arc<Self>, node: &mut RaftNode) {
        self.schedule_apply(node);
    }

    fn leader_url(&self, node: &RaftNode) -> (Option<NodeId>, Option<String>) {
        let leader = node.leader();
        let url = leader.and_then(|l| {
            self.peers
                .read()
                .unwrap_or_else(|p| p.into_inner())
                .get(&l)
                .cloned()
        });
        (leader, url)
    }

    /// Drain the outbox and deliver each request to its peer over h2c, one task
    /// each (fire-and-forget). Replies feed back into the node + drive apply.
    async fn flush(self: &Arc<Self>) {
        let outs = {
            let mut n = self.node.lock().await;
            n.take_outgoing()
        };
        for o in outs {
            let lane = {
                let lanes = self.peer_lanes.read().unwrap_or_else(|p| p.into_inner());
                if let Some(l) = lanes.get(&o.to).cloned() {
                    Some(l)
                } else if self
                    .peers
                    .read()
                    .unwrap_or_else(|p| p.into_inner())
                    .contains_key(&o.to)
                {
                    drop(lanes);
                    let mut lanes = self.peer_lanes.write().unwrap_or_else(|p| p.into_inner());
                    Some(
                        lanes
                            .entry(o.to)
                            .or_insert_with(|| Arc::new(PeerLane::default()))
                            .clone(),
                    )
                } else {
                    None
                }
            };
            let Some(lane) = lane else {
                self.undeliverable_never_addressed
                    .fetch_add(1, Ordering::Relaxed);
                tracing::warn!(
                    target = o.to,
                    group = %self.group_id.0,
                    "raft: discarded message to peer with no registered address"
                );
                continue;
            };
            let spawn_worker = {
                let mut pending = lane.pending.lock().await;
                pending.enqueue(o.msg);
                !lane.running.swap(true, Ordering::AcqRel)
            };
            if spawn_worker {
                let s = Arc::clone(self);
                let tracker = Arc::clone(&self.rpc_tracker);
                tracker.active.fetch_add(1, Ordering::AcqRel);
                tokio::spawn(async move {
                    let _guard = RpcGuard { tracker };
                    loop {
                        let next = {
                            let mut pending = lane.pending.lock().await;
                            match pending.dequeue() {
                                Some(msg) => Some(msg),
                                None => {
                                    // Producers update the queue and observe
                                    // `running` under this same lock, closing
                                    // the empty-lane/spawn race.
                                    lane.running.store(false, Ordering::Release);
                                    None
                                }
                            }
                        };
                        let Some(msg) = next else {
                            break;
                        };
                        Arc::clone(&s).send_request(o.to, msg).await;
                    }
                });
            }
        }
    }

    async fn send_request(self: Arc<Self>, to: NodeId, msg: RaftMsg) {
        let Some(base) = self
            .peers
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .get(&to)
            .cloned()
        else {
            self.undeliverable_withdrawn_address
                .fetch_add(1, Ordering::Relaxed);
            tracing::warn!(
                target = to,
                group = %self.group_id.0,
                "raft: discarded in-flight message to withdrawn peer address"
            );
            return;
        };
        let reply: Option<RaftMsg> = match msg {
            RaftMsg::Vote(req) => self
                .post(
                    &format!("{base}/raft/request-vote"),
                    &VoteEnvelope {
                        group_id: self.group_id.0.clone(),
                        from: self.id,
                        req,
                    },
                )
                .await
                .and_then(|r| serde_json::from_slice::<VoteResp>(&r).ok())
                .map(RaftMsg::VoteResp),
            RaftMsg::Append(req) => self
                .post(
                    &format!("{base}/raft/append-entries"),
                    &AppendEnvelope {
                        group_id: self.group_id.0.clone(),
                        from: self.id,
                        req,
                    },
                )
                .await
                .and_then(|r| serde_json::from_slice::<AppendResp>(&r).ok())
                .map(RaftMsg::AppendResp),
            RaftMsg::InstallSnapshot(req) => self
                .request_snapshot(to, req, self.sm.snapshot_capability())
                .await
                .ok()
                .map(RaftMsg::InstallSnapshotResp),
            RaftMsg::TimeoutNow(req) => {
                self.post(
                    &format!("{base}/raft/timeout-now"),
                    &TimeoutNowEnvelope {
                        group_id: self.group_id.0.clone(),
                        from: self.id,
                        req,
                    },
                )
                .await;
                None
            }
            _ => None,
        };
        if let Some(reply) = reply {
            let mut n = self.node.lock().await;
            n.handle(to, reply);
            if self.persist(&n).is_err() {
                return;
            }
            self.apply_ready(&mut n);
            // Subsequent outbound work is shipped by the pump (no recursive flush).
        }
    }

    async fn post<T: Serialize>(&self, url: &str, body: &T) -> Option<Vec<u8>> {
        self.post_with_timeout(url, body, self.cfg.rpc_timeout)
            .await
    }

    async fn post_with_timeout<T: Serialize>(
        &self,
        url: &str,
        body: &T,
        timeout: Duration,
    ) -> Option<Vec<u8>> {
        match self
            .http_client()
            .post(url)
            .timeout(timeout)
            .json(body)
            .send()
            .await
        {
            Ok(r) => r.bytes().await.ok().map(|b| b.to_vec()),
            Err(_) => None,
        }
    }

    async fn request_snapshot(
        &self,
        to: NodeId,
        req: InstallSnapshotReq,
        required_capability: Option<&'static str>,
    ) -> Result<InstallSnapshotResp> {
        let base = self
            .peers
            .read()
            .unwrap_or_else(|peers| peers.into_inner())
            .get(&to)
            .cloned()
            .ok_or_else(|| anyhow!("raft: voter {to} has no registered peer address"))?;
        if let Some(required) = required_capability {
            let nonce = self.snapshot_nonce.fetch_add(1, Ordering::AcqRel);
            let bytes = self
                .post_with_timeout(
                    &format!("{base}/raft/install-snapshot-capable"),
                    &CapableSnapEnvelope {
                        group_id: self.group_id.0.clone(),
                        from: self.id,
                        req,
                        snapshot_capability: required.to_string(),
                        snapshot_nonce: nonce,
                    },
                    self.snapshot_rpc_timeout,
                )
                .await
                .ok_or_else(|| {
                    anyhow!("raft: voter {to} did not answer capable snapshot install")
                })?;
            let response: CapableSnapshotResp =
                serde_json::from_slice(&bytes).map_err(|error| {
                    anyhow!("raft: voter {to} returned an invalid capable snapshot reply: {error}")
                })?;
            if response.snapshot_capability != required || response.snapshot_nonce != nonce {
                return Err(anyhow!(
                    "raft: voter {to} did not echo snapshot capability {required}"
                ));
            }
            return Ok(InstallSnapshotResp {
                term: response.term,
                accepted: response.accepted,
                snapshot_index: response.snapshot_index,
            });
        }
        let bytes = self
            .post_with_timeout(
                &format!("{base}/raft/install-snapshot"),
                &SnapEnvelope {
                    group_id: self.group_id.0.clone(),
                    from: self.id,
                    req,
                },
                self.snapshot_rpc_timeout,
            )
            .await
            .ok_or_else(|| anyhow!("raft: voter {to} did not answer snapshot install"))?;
        serde_json::from_slice(&bytes).map_err(|error| {
            anyhow!("raft: voter {to} returned an invalid snapshot reply: {error}")
        })
    }

    async fn request_status(&self, to: NodeId) -> Result<RaftStatus> {
        let base = self
            .peers
            .read()
            .unwrap_or_else(|peers| peers.into_inner())
            .get(&to)
            .cloned()
            .ok_or_else(|| anyhow!("raft: voter {to} has no registered peer address"))?;
        let bytes = match self
            .http_client()
            .get(format!("{base}/raftz"))
            .timeout(self.cfg.rpc_timeout)
            .send()
            .await
        {
            Ok(response) => response
                .bytes()
                .await
                .map_err(|error| anyhow!("raft: voter {to} returned an invalid status: {error}"))?,
            Err(error) => return Err(anyhow!("raft: voter {to} did not answer status: {error}")),
        };
        serde_json::from_slice(&bytes)
            .map_err(|error| anyhow!("raft: voter {to} returned an invalid status: {error}"))
    }

    /// Try a leader-side proposal and return its index once the **state machine
    /// applies** it (read-your-write). `None` means this node was no longer
    /// leader when the node lock was acquired, so the caller may safely
    /// re-route the command: it was not appended. Once an index is allocated,
    /// an apply timeout remains an error and must not be retried blindly.
    async fn try_propose_applied(self: &Arc<Self>, command: Command) -> Option<ProposalOutcome> {
        if self.lifecycle_generation.load(Ordering::Acquire) > 0 {
            self.proposal_rejected_before_append
                .fetch_add(1, Ordering::Relaxed);
            return Some(ProposalOutcome::RejectedBeforeAdmission {
                reason: "raft: proposal admission closed".to_string(),
            });
        }
        let permit = match self.sm.admit_proposal(&command) {
            Ok(permit) => permit,
            Err(error) => {
                self.proposal_rejected_before_append
                    .fetch_add(1, Ordering::Relaxed);
                return Some(rejected_admission(error));
            }
        };
        let index = {
            let mut n = self.node.lock().await;
            if self.lifecycle_generation.load(Ordering::Acquire) > 0 {
                self.proposal_rejected_before_append
                    .fetch_add(1, Ordering::Relaxed);
                return Some(ProposalOutcome::RejectedBeforeAdmission {
                    reason: "raft: proposal admission closed".to_string(),
                });
            }
            if n.resident_log_bytes().saturating_add(command.len()) > self.max_resident_log_bytes {
                return Some(ProposalOutcome::RejectedBeforeAdmission {
                    reason: format!(
                        "raft: resident log memory limit reached ({} bytes); retry after snapshot compaction",
                        self.max_resident_log_bytes
                    ),
                });
            }
            let Some(idx) = n.propose(command) else {
                return None;
            };
            let term = n.current_term();
            if let Some(permit) = permit {
                let previous = self
                    .pending_admission
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .insert((idx, term), permit);
                debug_assert!(previous.is_none(), "Raft index permit was replaced");
            }
            if let Err(e) = self.persist(&n) {
                return Some(ProposalOutcome::DurabilityFailure {
                    index: Some(idx),
                    failure: e,
                });
            }
            self.apply_ready(&mut n); // sole voter queues its committed apply here
            idx
        };
        self.flush().await;

        if let Some(failure) = self.latched_failure.lock().unwrap().clone() {
            return Some(ProposalOutcome::DurabilityFailure {
                index: Some(index),
                failure,
            });
        }
        if self.completed_applied_index() >= index {
            return Some(ProposalOutcome::Completed { index });
        }
        let mut rx = self.applied_tx.subscribe();
        let deadline = Instant::now() + self.cfg.propose_timeout;
        loop {
            {
                let mut n = self.node.lock().await;
                self.apply_ready(&mut n);
                if let Some(failure) = self.latched_failure.lock().unwrap().clone() {
                    return Some(ProposalOutcome::DurabilityFailure {
                        index: Some(index),
                        failure,
                    });
                }
                if self.completed_applied_index() >= index {
                    return Some(ProposalOutcome::Completed { index });
                }
            }
            tokio::select! {
                _ = rx.changed() => {}
                _ = tokio::time::sleep(Duration::from_millis(5)) => {}
            }
            if Instant::now() >= deadline {
                return Some(ProposalOutcome::Ambiguous {
                    index: Some(index),
                    reason: format!("raft: apply timeout at index {index}"),
                });
            }
        }
    }
}

/// A running raft group host. Cheap to hold; aborts its tasks on drop.
pub struct RaftHost {
    pub(crate) shared: Arc<Shared>,
    tasks: StdMutex<Option<(JoinHandle<()>, JoinHandle<()>)>>,
}

impl Drop for RaftHost {
    fn drop(&mut self) {
        self.shared.apply_stopped.store(true, Ordering::Release);
        if let Some((tick, pump)) = self.tasks.lock().expect("raft task mutex poisoned").take() {
            tick.abort();
            pump.abort();
        }
    }
}

impl RaftHost {
    /// Build a host for node `id`, recovering persisted state + replaying the
    /// resident committed log into the state machine, and start the tick + pump.
    /// `peers` maps the other members to base URLs (empty ⇒ single-node).
    pub fn spawn(
        id: NodeId,
        membership: Membership,
        peers: HashMap<NodeId, String>,
        store: RaftStore,
        sm: Arc<dyn RaftStateMachine>,
        cfg: HostConfig,
    ) -> RaftHost {
        Self::spawn_group(
            id,
            GroupId(LEGACY_GROUP_ID.to_string()),
            membership,
            peers,
            store,
            sm,
            cfg,
        )
    }

    pub fn spawn_group(
        id: NodeId,
        group_id: GroupId,
        membership: Membership,
        peers: HashMap<NodeId, String>,
        store: RaftStore,
        sm: Arc<dyn RaftStateMachine>,
        cfg: HostConfig,
    ) -> RaftHost {
        Self::spawn_inner(id, group_id, membership, peers, store, sm, cfg, None)
    }

    /// Access the group identity of this host.
    pub fn group_id(&self) -> &GroupId {
        &self.shared.group_id
    }

    /// Spawn a host whose outgoing peer RPCs use the current generation of a
    /// shared mutually authenticated HTTPS transport. Callers serve
    /// [`Self::router`] on [`PeerTransport::serve`] using the same clone.
    pub fn spawn_with_peer_transport(
        id: NodeId,
        membership: Membership,
        peers: HashMap<NodeId, String>,
        store: RaftStore,
        sm: Arc<dyn RaftStateMachine>,
        cfg: HostConfig,
        peer_transport: PeerTransport,
    ) -> RaftHost {
        Self::spawn_with_peer_transport_group(
            id,
            GroupId(LEGACY_GROUP_ID.to_string()),
            membership,
            peers,
            store,
            sm,
            cfg,
            peer_transport,
        )
    }

    pub fn spawn_with_peer_transport_group(
        id: NodeId,
        group_id: GroupId,
        membership: Membership,
        peers: HashMap<NodeId, String>,
        store: RaftStore,
        sm: Arc<dyn RaftStateMachine>,
        cfg: HostConfig,
        peer_transport: PeerTransport,
    ) -> RaftHost {
        Self::spawn_inner(
            id,
            group_id,
            membership,
            peers,
            store,
            sm,
            cfg,
            Some(peer_transport),
        )
    }

    fn spawn_inner(
        id: NodeId,
        group_id: GroupId,
        membership: Membership,
        peers: HashMap<NodeId, String>,
        store: RaftStore,
        sm: Arc<dyn RaftStateMachine>,
        cfg: HostConfig,
        peer_transport: Option<PeerTransport>,
    ) -> RaftHost {
        let loaded = store.load().unwrap_or_else(|error| {
            panic!(
                "raft: refuse to start node {id}; durable state {} is invalid: {error}",
                store.path().display()
            )
        });
        let mut node = match loaded {
            Some(state) => RaftNode::from_persisted(id, &membership, state),
            None => RaftNode::new(id, &membership),
        };
        cold_start(&mut node, sm.as_ref(), true).unwrap_or_else(|error| {
            panic!("raft: refuse to start node {id}; committed replay failed: {error}")
        });

        let client =
            transport_h2c::h2c_client_with(Some(cfg.rpc_timeout), None).expect("h2c client");
        let (applied_tx, _rx) = watch::channel(sm.applied_index());
        let (shutdown_tx, _shutdown_rx) = watch::channel(None);
        let peer_lanes = peers
            .keys()
            .copied()
            .map(|peer| (peer, Arc::new(PeerLane::default())))
            .collect();
        let max_resident_log_bytes = std::env::var("RAFT_RUNTIME_MAX_RESIDENT_LOG_BYTES")
            .ok()
            .map(|value| {
                value.parse::<usize>().unwrap_or_else(|error| {
                    panic!(
                        "RAFT_RUNTIME_MAX_RESIDENT_LOG_BYTES must be a positive integer: {error}"
                    )
                })
            })
            .unwrap_or(DEFAULT_MAX_RESIDENT_LOG_BYTES);
        let snapshot_rpc_timeout = std::env::var("RAFT_RUNTIME_SNAPSHOT_RPC_TIMEOUT_SECONDS")
            .ok()
            .map(|value| {
                value.parse::<u64>().unwrap_or_else(|error| {
                    panic!(
                        "RAFT_RUNTIME_SNAPSHOT_RPC_TIMEOUT_SECONDS must be a positive integer: {error}"
                    )
                })
            })
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(600))
            .max(cfg.rpc_timeout);
        assert!(
            max_resident_log_bytes > 0,
            "RAFT_RUNTIME_MAX_RESIDENT_LOG_BYTES must be greater than zero"
        );
        let shared = Arc::new(Shared {
            id,
            group_id,
            node: Mutex::new(node),
            store,
            sm,
            pending_admission: StdMutex::new(BTreeMap::new()),
            peers: StdRwLock::new(peers),
            peer_lanes: StdRwLock::new(peer_lanes),
            client,
            peer_transport,
            applied_tx,
            cfg,
            rpc_tracker: Arc::new(RpcTracker::default()),
            latched_failure: StdMutex::new(None),
            undeliverable_never_addressed: AtomicU64::new(0),
            undeliverable_withdrawn_address: AtomicU64::new(0),
            proposal_rejected_before_routing: AtomicU64::new(0),
            proposal_rejected_before_append: AtomicU64::new(0),
            lifecycle_generation: AtomicU64::new(0),
            snapshot_nonce: AtomicU64::new(1),
            snapshot_rpc_timeout,
            snapshot_install: Arc::new(Mutex::new(())),
            apply_running: AtomicBool::new(false),
            apply_stopped: AtomicBool::new(false),
            apply_tracker: Arc::new(RpcTracker::default()),
            max_resident_log_bytes,
            shutdown_started: AtomicBool::new(false),
            shutdown_tx,
        });

        let s = Arc::clone(&shared);
        let tick = tokio::spawn(async move {
            let mut first_tick = true;
            loop {
                tokio::time::sleep(s.cfg.tick).await;
                {
                    let mut n = s.node.lock().await;
                    if tick_then_maybe_persist(
                        &mut n,
                        &mut first_tick,
                        |node| s.persist(node).is_ok(),
                        || s.latched_failure.lock().unwrap().is_some(),
                    ) {
                        s.apply_ready(&mut n);
                    }
                }
                s.flush().await;
            }
        });
        let p = Arc::clone(&shared);
        let pump = tokio::spawn(async move {
            loop {
                tokio::time::sleep(p.cfg.pump).await;
                p.flush().await;
            }
        });
        RaftHost {
            shared,
            tasks: StdMutex::new(Some((tick, pump))),
        }
    }

    /// Quiesce proposal admission on this host. Returns `true` if this call
    /// transitioned admission from open to closed, or `false` if already closed.
    pub fn quiesce_proposals(&self) -> bool {
        self.shared
            .lifecycle_generation
            .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    /// Run host shutdown bounded by the caller's shared absolute deadline (#3672, #3683).
    ///
    /// Executes the four shutdown phases in fixed sequential order:
    /// 1. `Quiesce` — stop accepting new proposals
    /// 2. `LeadershipHandoff` — hand off leadership to a caught-up peer
    /// 3. `BackgroundTasks` — abort and await the tick and pump loops
    /// 4. `PeerRpcDrain` — wait for in-flight peer RPCs to finish
    ///
    /// Every phase uses one cumulative absolute cutoff from the usable interval
    /// captured when shutdown begins. If a cutoff expires, the run stops
    /// immediately, later phases are neither run nor recorded, and
    /// `incomplete_phase` names that phase.
    ///
    /// A host performs this shutdown sequence at most once across its whole lifetime.
    /// The first caller executes the phases and receives a report with
    /// [`ShutdownCaller::Executed`]; concurrent or repeat callers wait for completion
    /// and receive the identical terminal outcome with [`ShutdownCaller::Joined`].
    pub async fn shutdown_within(&self, deadline: ShutdownDeadline) -> HostShutdownReport {
        if self
            .shared
            .shutdown_started
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            let mut rx = self.shared.shutdown_tx.subscribe();
            while rx.borrow().is_none() {
                if rx.changed().await.is_err() {
                    break;
                }
            }
            let terminal = rx
                .borrow()
                .as_ref()
                .expect("shutdown report published")
                .clone();
            return HostShutdownReport {
                caller: ShutdownCaller::Joined,
                phases: terminal.phases,
                handoff: terminal.handoff,
                incomplete_phase: terminal.incomplete_phase,
                peer_listener_close_safe: terminal.peer_listener_close_safe,
                storage_failure: terminal.storage_failure,
            };
        }

        let report = self.execute_shutdown(deadline).await;
        self.shared.shutdown_tx.send_replace(Some(report.clone()));
        report
    }

    async fn execute_shutdown(&self, deadline: ShutdownDeadline) -> HostShutdownReport {
        let mut phases = Vec::with_capacity(4);
        let mut handoff = LeadershipHandoff::NotLeader;
        let mut observed_storage_failure = None;
        let phase_cutoffs = shutdown_phase_cutoffs(deadline, tokio::time::Instant::now());

        // Phase 1: Quiesce
        let start = Instant::now();
        if shutdown_phase_cutoff_elapsed(phase_cutoffs[0]) {
            phases.push(PhaseRecord {
                phase: ShutdownPhase::Quiesce,
                status: PhaseStatus::DeadlineExpired,
                elapsed: start.elapsed(),
            });
            return HostShutdownReport {
                caller: ShutdownCaller::Executed,
                phases,
                handoff,
                incomplete_phase: Some(ShutdownPhase::Quiesce),
                peer_listener_close_safe: false,
                storage_failure: self.shared.latched_failure.lock().unwrap().clone(),
            };
        }
        self.quiesce_proposals();
        let current_failure = self.shared.latched_failure.lock().unwrap().clone();
        let status = if observed_storage_failure.is_none() && current_failure.is_some() {
            observed_storage_failure = current_failure;
            PhaseStatus::StorageFailed
        } else {
            PhaseStatus::Completed
        };
        phases.push(PhaseRecord {
            phase: ShutdownPhase::Quiesce,
            status,
            elapsed: start.elapsed(),
        });

        // Phase 2: LeadershipHandoff
        let start = Instant::now();
        if shutdown_phase_cutoff_elapsed(phase_cutoffs[1]) {
            phases.push(PhaseRecord {
                phase: ShutdownPhase::LeadershipHandoff,
                status: PhaseStatus::DeadlineExpired,
                elapsed: start.elapsed(),
            });
            return HostShutdownReport {
                caller: ShutdownCaller::Executed,
                phases,
                handoff,
                incomplete_phase: Some(ShutdownPhase::LeadershipHandoff),
                peer_listener_close_safe: false,
                storage_failure: observed_storage_failure
                    .or_else(|| self.shared.latched_failure.lock().unwrap().clone()),
            };
        }
        match tokio::time::timeout_at(phase_cutoffs[1], self.handoff_leadership()).await {
            Ok(outcome) => {
                handoff = outcome;
                let current_failure = self.shared.latched_failure.lock().unwrap().clone();
                let status = if observed_storage_failure.is_none() && current_failure.is_some() {
                    observed_storage_failure = current_failure;
                    PhaseStatus::StorageFailed
                } else {
                    PhaseStatus::Completed
                };
                phases.push(PhaseRecord {
                    phase: ShutdownPhase::LeadershipHandoff,
                    status,
                    elapsed: start.elapsed(),
                });
            }
            Err(_) => {
                phases.push(PhaseRecord {
                    phase: ShutdownPhase::LeadershipHandoff,
                    status: PhaseStatus::DeadlineExpired,
                    elapsed: start.elapsed(),
                });
                return HostShutdownReport {
                    caller: ShutdownCaller::Executed,
                    phases,
                    handoff,
                    incomplete_phase: Some(ShutdownPhase::LeadershipHandoff),
                    peer_listener_close_safe: false,
                    storage_failure: observed_storage_failure
                        .or_else(|| self.shared.latched_failure.lock().unwrap().clone()),
                };
            }
        }

        // Phase 3: BackgroundTasks
        let start = Instant::now();
        let timed_out = if shutdown_phase_cutoff_elapsed(phase_cutoffs[2]) {
            true
        } else {
            let tasks = self.tasks.lock().expect("raft task mutex poisoned").take();
            if let Some((tick, pump)) = tasks {
                tick.abort();
                pump.abort();
                let shared = Arc::clone(&self.shared);
                let join_all = async move {
                    let _ = tick.await;
                    let _ = pump.await;
                    {
                        let _node = shared.node.lock().await;
                        shared.apply_stopped.store(true, Ordering::Release);
                    }
                    shared.apply_tracker.wait_idle().await;
                };
                tokio::time::timeout_at(phase_cutoffs[2], join_all)
                    .await
                    .is_err()
            } else {
                false
            }
        };
        if timed_out {
            phases.push(PhaseRecord {
                phase: ShutdownPhase::BackgroundTasks,
                status: PhaseStatus::DeadlineExpired,
                elapsed: start.elapsed(),
            });
            return HostShutdownReport {
                caller: ShutdownCaller::Executed,
                phases,
                handoff,
                incomplete_phase: Some(ShutdownPhase::BackgroundTasks),
                peer_listener_close_safe: false,
                storage_failure: observed_storage_failure
                    .or_else(|| self.shared.latched_failure.lock().unwrap().clone()),
            };
        }
        let current_failure = self.shared.latched_failure.lock().unwrap().clone();
        let status = if observed_storage_failure.is_none() && current_failure.is_some() {
            observed_storage_failure = current_failure;
            PhaseStatus::StorageFailed
        } else {
            PhaseStatus::Completed
        };
        phases.push(PhaseRecord {
            phase: ShutdownPhase::BackgroundTasks,
            status,
            elapsed: start.elapsed(),
        });

        // Phase 4: PeerRpcDrain
        let start = Instant::now();
        if shutdown_phase_cutoff_elapsed(phase_cutoffs[3])
            || tokio::time::timeout_at(phase_cutoffs[3], self.shared.rpc_tracker.wait_idle())
                .await
                .is_err()
        {
            phases.push(PhaseRecord {
                phase: ShutdownPhase::PeerRpcDrain,
                status: PhaseStatus::DeadlineExpired,
                elapsed: start.elapsed(),
            });
            return HostShutdownReport {
                caller: ShutdownCaller::Executed,
                phases,
                handoff,
                incomplete_phase: Some(ShutdownPhase::PeerRpcDrain),
                peer_listener_close_safe: false,
                storage_failure: observed_storage_failure
                    .or_else(|| self.shared.latched_failure.lock().unwrap().clone()),
            };
        }
        let current_failure = self.shared.latched_failure.lock().unwrap().clone();
        let status = if observed_storage_failure.is_none() && current_failure.is_some() {
            observed_storage_failure = current_failure;
            PhaseStatus::StorageFailed
        } else {
            PhaseStatus::Completed
        };
        phases.push(PhaseRecord {
            phase: ShutdownPhase::PeerRpcDrain,
            status,
            elapsed: start.elapsed(),
        });

        HostShutdownReport {
            caller: ShutdownCaller::Executed,
            phases,
            handoff,
            incomplete_phase: None,
            peer_listener_close_safe: true,
            storage_failure: observed_storage_failure,
        }
    }

    /// Stop the periodic host loops and wait for every already-dispatched peer
    /// RPC to finish before the h2 client is dropped. Service shutdown should
    /// stop public ingress first, call this method, then close the peer
    /// listener only when `peer_listener_close_safe` is true. The bounded wait
    /// prevents active HTTP/2 streams from being torn down with the Tokio
    /// runtime.
    pub async fn shutdown(&self) -> Result<()> {
        let timeout = self.shared.cfg.rpc_timeout + self.shared.cfg.rpc_timeout;
        let deadline =
            ShutdownDeadline::from_now(timeout, Duration::ZERO).map_err(|e| anyhow!("{e}"))?;
        let report = self.shutdown_within(deadline).await;
        report.into_result()
    }

    /// Access the underlying raft store.
    pub fn store(&self) -> &RaftStore {
        &self.shared.store
    }

    pub async fn is_leader(&self) -> bool {
        self.shared.node.lock().await.is_leader()
    }
    pub async fn leader(&self) -> Option<NodeId> {
        self.shared.node.lock().await.leader()
    }
    /// Hand leadership to an eligible caught-up voter before shutdown (#3664).
    pub async fn handoff_leadership(&self) -> LeadershipHandoff {
        let (outcome, transferred) = {
            let mut node = self.shared.node.lock().await;
            if !node.is_leader() {
                (LeadershipHandoff::NotLeader, false)
            } else {
                let voters = node.conf_state().membership.voters.len();
                if voters <= 1 {
                    (LeadershipHandoff::SoleVoter, false)
                } else if let Some(target) = node.handoff_candidate() {
                    match node.transfer_leadership(target) {
                        Ok(()) => (LeadershipHandoff::Transferred { target }, true),
                        Err(_) => (LeadershipHandoff::NoCaughtUpVoter { voters }, false),
                    }
                } else {
                    (LeadershipHandoff::NoCaughtUpVoter { voters }, false)
                }
            }
        };
        if transferred {
            self.shared.flush().await;
        }
        outcome
    }
    /// Transfer leadership to a named caught-up voter (#3586).
    pub async fn transfer_leadership(
        &self,
        target: NodeId,
    ) -> std::result::Result<(), TransferRefused> {
        let res = self.shared.node.lock().await.transfer_leadership(target);
        if res.is_ok() {
            self.shared.flush().await;
        }
        res
    }
    /// Promote a caught-up learner to voter (#3646).
    pub async fn promote_learner(
        &self,
        target: NodeId,
    ) -> std::result::Result<Index, PromotionRefused> {
        let res = self.shared.node.lock().await.promote_learner(target);
        if res.is_ok() {
            self.shared.flush().await;
        }
        res
    }
    /// Demote a voter to a learner (#3646).
    pub async fn demote_voter(
        &self,
        target: NodeId,
    ) -> std::result::Result<Index, DemotionRefused> {
        let res = self.shared.node.lock().await.demote_voter(target);
        if res.is_ok() {
            self.shared.flush().await;
        }
        res
    }
    /// Remove a member from the group (#3646).
    pub async fn remove_member(
        &self,
        target: NodeId,
    ) -> std::result::Result<Index, RemovalRefused> {
        let res = self.shared.node.lock().await.remove_member(target);
        if res.is_ok() {
            self.shared.flush().await;
        }
        res
    }
    /// Add or update the address of a peer (#3650).
    pub async fn upsert_peer(&self, peer: NodeId, url: String) {
        self.shared
            .peers
            .write()
            .unwrap_or_else(|p| p.into_inner())
            .insert(peer, url);
    }
    /// Remove the address of a peer (#3650).
    pub async fn forget_peer(&self, peer: NodeId) {
        self.shared
            .peers
            .write()
            .unwrap_or_else(|p| p.into_inner())
            .remove(&peer);
        self.shared
            .peer_lanes
            .write()
            .unwrap_or_else(|p| p.into_inner())
            .remove(&peer);
    }
    /// Admit a new learner to the group (#3650).
    pub async fn add_learner(
        &self,
        target: NodeId,
    ) -> std::result::Result<Index, AdmissionRefused> {
        let is_routable = self
            .shared
            .peers
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .contains_key(&target);
        if !is_routable {
            return Err(AdmissionRefused::Unroutable { target });
        }
        let res = {
            let mut node = self.shared.node.lock().await;
            match node.add_learner(target) {
                Some(idx) => Ok(idx),
                None => Err(AdmissionRefused::NotLeaderOrTransferInFlight),
            }
        };
        if res.is_ok() {
            self.shared.flush().await;
        }
        res
    }
    /// Watch the state machine's applied head (followers await an index here).
    pub fn applied_watch(&self) -> watch::Receiver<Index> {
        self.shared.applied_tx.subscribe()
    }

    /// Propose on the leader (locally), else forward to the current leader's
    /// `/raft/publish` over h2c. Returns the assigned index once applied
    /// (read-your-write). Retries within the propose deadline while no leader.
    pub async fn propose(&self, command: Command) -> Result<Index> {
        match self.propose_outcome(command).await {
            ProposalOutcome::Completed { index } => Ok(index),
            ProposalOutcome::RejectedBeforeAdmission { reason } => {
                if let Some(backpressure) = decode_backpressure(&reason) {
                    Err(anyhow::Error::new(backpressure))
                } else {
                    Err(anyhow!(reason))
                }
            }
            ProposalOutcome::Ambiguous { reason, .. } => Err(anyhow!(reason)),
            ProposalOutcome::DurabilityFailure { failure, .. } => Err(anyhow!(failure)),
        }
    }

    /// Propose on the leader (locally), else forward to the current leader's
    /// `/raft/publish` over h2c. Returns a typed [`ProposalOutcome`]
    /// classifying the terminal state of the proposal. Retries within the
    /// propose deadline while no leader.
    pub async fn propose_outcome(&self, command: Command) -> ProposalOutcome {
        let s = &self.shared;
        if s.lifecycle_generation.load(Ordering::Acquire) > 0 {
            s.proposal_rejected_before_routing
                .fetch_add(1, Ordering::Relaxed);
            return ProposalOutcome::RejectedBeforeAdmission {
                reason: "raft: proposal admission closed".to_string(),
            };
        }
        if let Some(err) = s.latched_failure.lock().unwrap().clone() {
            return ProposalOutcome::DurabilityFailure {
                index: None,
                failure: err,
            };
        }
        let deadline = Instant::now() + s.cfg.propose_timeout;
        let mut last_route_error = None;
        loop {
            let route = {
                let n = s.node.lock().await;
                if n.is_leader() {
                    Route::Local
                } else {
                    match s.leader_url(&n).1 {
                        Some(url) => Route::Remote(url),
                        None => Route::Unknown,
                    }
                }
            };
            match route {
                Route::Local => {
                    if let Some(outcome) = s.try_propose_applied(command.clone()).await {
                        return outcome;
                    }
                }
                Route::Remote(url) => match self.forward(&url, &command).await {
                    ProposalOutcome::Completed { index } => {
                        return ProposalOutcome::Completed { index }
                    }
                    ProposalOutcome::Ambiguous {
                        index: Some(seq),
                        reason,
                    } => {
                        return ProposalOutcome::Ambiguous {
                            index: Some(seq),
                            reason,
                        };
                    }
                    outcome @ ProposalOutcome::DurabilityFailure { .. } => return outcome,
                    outcome @ ProposalOutcome::RejectedBeforeAdmission { .. } => return outcome,
                    ProposalOutcome::Ambiguous {
                        index: None,
                        reason,
                    } => {
                        last_route_error = Some(reason);
                    }
                },
                Route::Unknown => {}
            }
            if Instant::now() >= deadline {
                return match last_route_error {
                    Some(error) => ProposalOutcome::Ambiguous {
                        index: None,
                        reason: format!("raft: proposal routing timed out: {error}"),
                    },
                    None => ProposalOutcome::RejectedBeforeAdmission {
                        reason: "raft: no leader elected (cluster not ready)".to_string(),
                    },
                };
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    /// Forward a command to the leader and wait until **this node's** state
    /// machine applies the returned index (read-your-write on a follower).
    async fn forward(&self, leader_url: &str, command: &[u8]) -> ProposalOutcome {
        let resp = match self
            .shared
            .http_client()
            .post(format!("{leader_url}{}", Self::PUBLISH_PATH))
            // Unlike Vote/AppendEntries, publish includes durable quorum
            // commit and local state-machine apply. Its request budget is the
            // proposal deadline, not the short peer transport timeout.
            .timeout(self.shared.cfg.propose_timeout)
            .json(&PublishEnvelope {
                group_id: self.shared.group_id.0.clone(),
                command: command.to_vec(),
            })
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                return ProposalOutcome::Ambiguous {
                    index: None,
                    reason: e.to_string(),
                };
            }
        };
        let status = resp.status();
        if status == StatusCode::TOO_MANY_REQUESTS {
            let value: serde_json::Value = match resp.json().await {
                Ok(value) => value,
                Err(error) => {
                    return ProposalOutcome::Ambiguous {
                        index: None,
                        reason: error.to_string(),
                    }
                }
            };
            let Some(reason) = value.get("error").and_then(|value| value.as_str()) else {
                return ProposalOutcome::Ambiguous {
                    index: None,
                    reason: "raft: leader backpressure reply missing error".to_string(),
                };
            };
            let Some(retry_after_seconds) = value
                .get("retry_after_seconds")
                .and_then(|value| value.as_u64())
            else {
                return ProposalOutcome::Ambiguous {
                    index: None,
                    reason: "raft: leader backpressure reply missing retry_after_seconds"
                        .to_string(),
                };
            };
            return ProposalOutcome::RejectedBeforeAdmission {
                reason: encode_backpressure(&ProposalBackpressure {
                    reason: reason.to_string(),
                    retry_after_seconds,
                }),
            };
        }
        if status == StatusCode::SERVICE_UNAVAILABLE {
            let v: serde_json::Value = match resp.json().await {
                Ok(v) => v,
                Err(e) => {
                    return ProposalOutcome::Ambiguous {
                        index: None,
                        reason: e.to_string(),
                    };
                }
            };
            if v.get("outcome").and_then(|outcome| outcome.as_str())
                == Some("rejected_before_admission")
            {
                if let Some(reason) = v.get("error").and_then(|error| error.as_str()) {
                    return ProposalOutcome::RejectedBeforeAdmission {
                        reason: reason.to_string(),
                    };
                }
            }
            return ProposalOutcome::Ambiguous {
                index: None,
                reason: format!("raft: leader redirect returned {status}"),
            };
        }
        if status != StatusCode::OK {
            return ProposalOutcome::Ambiguous {
                index: None,
                reason: format!("raft: leader redirect returned {status}"),
            };
        }
        let v: serde_json::Value = match resp.json().await {
            Ok(v) => v,
            Err(e) => {
                return ProposalOutcome::Ambiguous {
                    index: None,
                    reason: e.to_string(),
                };
            }
        };
        let seq = match v.get("seq").and_then(|s| s.as_u64()) {
            Some(s) => s,
            None => {
                return ProposalOutcome::Ambiguous {
                    index: None,
                    reason: "raft: leader reply missing seq".to_string(),
                };
            }
        };
        // Wait for our own apply (the leader's commit propagates via AppendEntries).
        let mut rx = self.shared.applied_tx.subscribe();
        let deadline = Instant::now() + self.shared.cfg.propose_timeout;
        while self.shared.completed_applied_index() < seq {
            tokio::select! {
                _ = rx.changed() => {}
                _ = tokio::time::sleep(Duration::from_millis(5)) => {}
            }
            if Instant::now() >= deadline {
                return ProposalOutcome::Ambiguous {
                    index: Some(seq),
                    reason: format!("raft: follower apply timeout at index {seq}"),
                };
            }
        }
        ProposalOutcome::Completed { index: seq }
    }

    /// Capture a state-machine snapshot and compact the log up to the applied
    /// index (for `SnapshotPolicy::External` consumers driving their own cadence).
    pub async fn snapshot_and_compact(&self) -> Result<Index> {
        Ok(self
            .snapshot_and_compact_with_policy(None, true)
            .await?
            .snapshot_index)
    }

    /// Return the durable local snapshot index without changing Raft state.
    pub async fn snapshot_index(&self) -> Index {
        self.shared.node.lock().await.snapshot_index()
    }

    /// Verify that every voter advertises the local state machine's snapshot
    /// capability before a product performs a snapshot-dependent mutation.
    pub async fn require_snapshot_capability_on_all_voters(&self) -> Result<()> {
        let _operation = self.shared.begin_coordinated_peer_work()?;
        let (term, voters) = {
            let node = self.shared.node.lock().await;
            if !node.is_leader() && node.conf_state().membership.voters.len() > 1 {
                return Err(anyhow!(
                    "only the Raft leader can verify voter snapshot capabilities"
                ));
            }
            (
                node.current_term(),
                node.conf_state().membership.voters.clone(),
            )
        };
        let required = self
            .shared
            .sm
            .snapshot_capability()
            .ok_or_else(|| anyhow!("state machine does not advertise a snapshot capability"))?;
        for voter in voters
            .iter()
            .copied()
            .filter(|voter| *voter != self.shared.id)
        {
            let status = self.shared.request_status(voter).await?;
            if status.snapshot_capability.as_deref() != Some(required) {
                return Err(anyhow!(
                    "raft: voter {voter} does not advertise snapshot capability {required}"
                ));
            }
        }
        let node = self.shared.node.lock().await;
        if node.current_term() != term
            || (!node.is_leader() && node.conf_state().membership.voters.len() > 1)
        {
            return Err(anyhow!(
                "raft leadership changed while verifying snapshot capabilities"
            ));
        }
        Ok(())
    }

    /// Verify that every voter has applied at least `index` while this node
    /// remains leader in the same term.
    ///
    /// Products use this barrier before a destructive transition whose query
    /// fence must be visible on every serving replica, not only on a quorum.
    pub async fn require_applied_index_on_all_voters(&self, index: Index) -> Result<()> {
        let _operation = self.shared.begin_coordinated_peer_work()?;
        let (term, voters) = {
            let node = self.shared.node.lock().await;
            if !node.is_leader() && node.conf_state().membership.voters.len() > 1 {
                return Err(anyhow!(
                    "only the Raft leader can verify voter applied indexes"
                ));
            }
            let local_applied = self.shared.completed_applied_index();
            if local_applied < index {
                return Err(anyhow!(
                    "raft: local voter {} has applied index {local_applied}, below required index {index}",
                    self.shared.id
                ));
            }
            (
                node.current_term(),
                node.conf_state().membership.voters.clone(),
            )
        };
        for voter in voters
            .iter()
            .copied()
            .filter(|voter| *voter != self.shared.id)
        {
            let status = self.shared.request_status(voter).await?;
            if status.applied_index < index {
                return Err(anyhow!(
                    "raft: voter {voter} has applied index {}, below required index {index}",
                    status.applied_index
                ));
            }
        }
        let node = self.shared.node.lock().await;
        if node.current_term() != term
            || (!node.is_leader() && node.conf_state().membership.voters.len() > 1)
        {
            return Err(anyhow!(
                "raft leadership changed while verifying voter applied indexes"
            ));
        }
        Ok(())
    }

    /// Capture a product-defined checkpoint and compact only the requested
    /// applied prefix.
    ///
    /// This is for durable data-plane products that can prove an older Raft
    /// prefix is already recoverable from their own committed storage. The
    /// state machine receives the exact prefix through `snapshot_at` and can
    /// refuse when that proof is not available.
    pub async fn snapshot_and_compact_through(&self, up_to: Index) -> Result<Index> {
        Ok(self
            .snapshot_and_compact_through_outcome(up_to)
            .await?
            .snapshot_index)
    }

    /// Coordinate a product snapshot and report whether voters installed new
    /// bytes. `installed` is false when `up_to` was already compacted.
    pub async fn snapshot_and_compact_through_outcome(
        &self,
        up_to: Index,
    ) -> Result<SnapshotCompactionOutcome> {
        self.snapshot_and_compact_with_policy(Some(up_to), true)
            .await
    }

    /// Compact after a voting quorum installs a self-contained snapshot.
    ///
    /// The caller must keep every external object referenced by the snapshot
    /// until lagging voters have installed it. This method bounds the leader's
    /// resident log during one voter outage, but it does not authorize product
    /// data or archive garbage collection.
    pub async fn snapshot_and_compact_through_quorum_outcome(
        &self,
        up_to: Index,
    ) -> Result<SnapshotCompactionOutcome> {
        self.snapshot_and_compact_with_policy(Some(up_to), false)
            .await
    }

    async fn snapshot_and_compact_with_policy(
        &self,
        requested_index: Option<Index>,
        require_every_voter: bool,
    ) -> Result<SnapshotCompactionOutcome> {
        let operation = self.shared.begin_coordinated_peer_work()?;
        let (preparation, state_machine_lease) = preflight_snapshot_with_serial(
            Arc::clone(&self.shared.sm),
            Arc::clone(&self.shared.snapshot_install),
            Arc::clone(&operation),
        )
        .await?;
        let mut state_machine_lease = Some(state_machine_lease);
        let (term, snapshot_term, voters, checkpoint_index, existing_bytes) = {
            let n = self.shared.node.lock().await;
            if let Some(failure) = self.shared.latched_failure.lock().unwrap().clone() {
                return Err(failure.into());
            }
            let applied = self.shared.completed_applied_index();
            let up_to = requested_index.unwrap_or(applied);
            if up_to == 0 {
                return Ok(SnapshotCompactionOutcome {
                    snapshot_index: n.snapshot_index(),
                    installed: false,
                });
            }
            if up_to > applied {
                return Err(anyhow!(
                    "cannot compact unapplied Raft prefix {up_to}; state machine applied index is {applied}"
                ));
            }
            if up_to <= n.snapshot_index()
                && (!require_every_voter || n.conf_state().membership.voters.len() <= 1)
            {
                return Ok(SnapshotCompactionOutcome {
                    snapshot_index: n.snapshot_index(),
                    installed: false,
                });
            }
            if !n.is_leader() && n.conf_state().membership.voters.len() > 1 {
                return Err(anyhow!(
                    "only the Raft leader can coordinate voter compaction"
                ));
            }
            if up_to <= n.snapshot_index() {
                drop(state_machine_lease.take());
                let persisted = n.persisted_ref();
                (
                    n.current_term(),
                    persisted.snapshot_term,
                    n.conf_state().membership.voters.clone(),
                    persisted.snapshot_index,
                    Some(persisted.snapshot.to_vec()),
                )
            } else {
                let snapshot_term = n.term_at_index(up_to).ok_or_else(|| {
                    anyhow!("Raft prefix {up_to} has no term and cannot be compacted")
                })?;
                let term = n.current_term();
                let voters = n.conf_state().membership.voters.clone();
                drop(n);
                (term, snapshot_term, voters, up_to, None)
            }
        };
        let up_to = checkpoint_index;
        let already_compacted = existing_bytes.is_some();
        let bytes = if let Some(bytes) = existing_bytes {
            // A prepared product handle can retain a save permit. There is no
            // new capture to make after loading persisted bytes, so release it
            // before status checks and snapshot resend work begin.
            drop(preparation);
            bytes
        } else if let Some(preparation) = preparation {
            let capture_operation = Arc::clone(&operation);
            let prepared = tokio::task::spawn_blocking(move || {
                let _operation = capture_operation;
                let _serial = state_machine_lease
                    .take()
                    .expect("new snapshot must retain its state-machine lease");
                preparation.capture_at(up_to)
            })
            .await??;
            let export_operation = Arc::clone(&operation);
            tokio::task::spawn_blocking(move || {
                // The immutable capture is safe to encode while ordered apply
                // proceeds. Retain peer-work ownership until output ends.
                let _operation = export_operation;
                let mut sink = ChunkSink::new(SNAPSHOT_CHUNK_SIZE);
                prepared.write_to(&mut sink)?;
                Ok::<_, anyhow::Error>(sink.into_bytes())
            })
            .await??
        } else {
            let legacy_operation = Arc::clone(&operation);
            let sm = Arc::clone(&self.shared.sm);
            tokio::task::spawn_blocking(move || {
                let _operation = legacy_operation;
                let _serial = state_machine_lease
                    .take()
                    .expect("legacy snapshot must retain its state-machine lease");
                let mut sink = ChunkSink::new(SNAPSHOT_CHUNK_SIZE);
                sm.snapshot_at(up_to, &mut sink)?;
                Ok::<_, anyhow::Error>(sink.into_bytes())
            })
            .await??
        };

        let mut replies = Vec::new();
        let required_capability = self.shared.sm.snapshot_capability();
        if require_every_voter {
            if let Some(required) = required_capability {
                for voter in voters
                    .iter()
                    .copied()
                    .filter(|voter| *voter != self.shared.id)
                {
                    let status = self.shared.request_status(voter).await?;
                    if status.snapshot_capability.as_deref() != Some(required) {
                        return Err(anyhow!(
                            "raft: voter {voter} does not advertise snapshot capability {required}"
                        ));
                    }
                }
            }
        }
        let quorum = voters.len() / 2 + 1;
        let remote_voters = voters
            .into_iter()
            .filter(|voter| *voter != self.shared.id)
            .collect::<Vec<_>>();
        let required_remote_replies = if require_every_voter {
            remote_voters.len()
        } else {
            quorum.saturating_sub(1)
        };
        let mut refusals = Vec::new();
        let mut requests = JoinSet::new();
        for voter in remote_voters {
            let shared = Arc::clone(&self.shared);
            let data = bytes.clone();
            let operation = Arc::clone(&operation);
            requests.spawn(async move {
                // Cancellation of the caller only schedules JoinSet aborts.
                // Keep the drain lease until this worker actually stops too.
                let _operation = operation;
                let response = shared
                    .request_snapshot(
                        voter,
                        InstallSnapshotReq {
                            term,
                            leader: shared.id,
                            snapshot_index: up_to,
                            snapshot_term,
                            data,
                        },
                        required_capability,
                    )
                    .await;
                (voter, response)
            });
        }
        while replies.len() < required_remote_replies {
            let Some(joined) = requests.join_next().await else {
                break;
            };
            let (voter, response) = match joined {
                Ok(response) => response,
                Err(error) if require_every_voter => {
                    return Err(anyhow!("raft: snapshot worker failed: {error}"));
                }
                Err(error) => {
                    refusals.push(format!("snapshot worker: {error}"));
                    continue;
                }
            };
            let response = match response {
                Ok(response) => response,
                Err(error) if require_every_voter => return Err(error),
                Err(error) => {
                    refusals.push(format!("voter {voter}: {error}"));
                    continue;
                }
            };
            if response.term > term {
                let mut node = self.shared.node.lock().await;
                node.handle(voter, RaftMsg::InstallSnapshotResp(response.clone()));
                self.shared.persist(&node)?;
                return Err(anyhow!(
                    "raft: voter {voter} advanced the term while refusing snapshot {up_to}"
                ));
            }
            if response.term != term || !response.accepted || response.snapshot_index < up_to {
                let refusal = anyhow!(
                    "raft: voter {voter} refused snapshot {up_to} at term {term}; replied term {} accepted {} index {}",
                    response.term,
                    response.accepted,
                    response.snapshot_index
                );
                if require_every_voter {
                    return Err(refusal);
                }
                refusals.push(refusal.to_string());
                continue;
            }
            replies.push((voter, response));
        }
        requests.abort_all();

        if replies.len() + 1 < quorum {
            return Err(anyhow!(
                "raft: snapshot {up_to} reached {} of {} required voters; {}",
                replies.len() + 1,
                quorum,
                refusals.join("; ")
            ));
        }

        let mut n = self.shared.node.lock().await;
        if n.current_term() != term
            || (!n.is_leader() && n.conf_state().membership.voters.len() > 1)
        {
            return Err(anyhow!(
                "raft leadership changed while coordinating snapshot {up_to}"
            ));
        }
        for (voter, response) in replies {
            n.handle(voter, RaftMsg::InstallSnapshotResp(response));
        }
        n.compact(up_to, bytes);
        self.shared.persist(&n)?;
        Ok(SnapshotCompactionOutcome {
            snapshot_index: up_to,
            installed: !already_compacted,
        })
    }

    /// The leader-side write target. The direct router, registry router, and
    /// follower forward client all use this one public path.
    pub const PUBLISH_PATH: &'static str = "/raft/publish";

    /// Peer raft RPCs + producer forward + status; merge into the service app so
    /// they ride the h2c serve port.
    pub fn router(&self) -> Router {
        Router::new()
            .route("/raft/request-vote", post(request_vote))
            .route("/raft/append-entries", post(append_entries))
            .route("/raft/install-snapshot", post(install_snapshot))
            .route(
                "/raft/install-snapshot-capable",
                post(install_snapshot_capable),
            )
            .route("/raft/timeout-now", post(timeout_now))
            .route(Self::PUBLISH_PATH, post(publish_handler))
            .route("/raftz", get(raftz))
            .with_state(Arc::clone(&self.shared))
    }
}

/// Pull the single reply addressed to `to` out of the node's outbox.
fn take_reply(node: &mut RaftNode, to: NodeId) -> Option<RaftMsg> {
    let mut reply = None;
    for o in node.take_outgoing() {
        if o.to == to
            && reply.is_none()
            && matches!(
                o.msg,
                RaftMsg::VoteResp(_) | RaftMsg::AppendResp(_) | RaftMsg::InstallSnapshotResp(_)
            )
        {
            reply = Some(o.msg);
        }
    }
    reply
}

pub(crate) async fn request_vote(
    State(s): State<Arc<Shared>>,
    Json(env): Json<VoteEnvelope>,
) -> axum::response::Response {
    if env.group_id != s.group_id.0 {
        return (StatusCode::BAD_REQUEST, "group id mismatch").into_response();
    }
    let mut n = s.node.lock().await;
    n.handle(env.from, RaftMsg::Vote(env.req));
    if s.persist(&n).is_err() {
        return Json(VoteResp {
            term: 0,
            granted: false,
        })
        .into_response();
    }
    Json(match take_reply(&mut n, env.from) {
        Some(RaftMsg::VoteResp(r)) => r,
        _ => VoteResp {
            term: 0,
            granted: false,
        },
    })
    .into_response()
}

pub(crate) async fn append_entries(
    State(s): State<Arc<Shared>>,
    Json(env): Json<AppendEnvelope>,
) -> axum::response::Response {
    if env.group_id != s.group_id.0 {
        return (StatusCode::BAD_REQUEST, "group id mismatch").into_response();
    }
    let mut n = s.node.lock().await;
    n.handle(env.from, RaftMsg::Append(env.req));
    if s.persist(&n).is_err() {
        return Json(AppendResp {
            term: 0,
            success: false,
            match_index: 0,
        })
        .into_response();
    }
    s.apply_ready(&mut n);
    Json(match take_reply(&mut n, env.from) {
        Some(RaftMsg::AppendResp(r)) => r,
        _ => AppendResp {
            term: 0,
            success: false,
            match_index: 0,
        },
    })
    .into_response()
}

pub(crate) async fn install_snapshot(
    State(s): State<Arc<Shared>>,
    Json(env): Json<SnapEnvelope>,
) -> axum::response::Response {
    if env.group_id != s.group_id.0 {
        return (StatusCode::BAD_REQUEST, "group id mismatch").into_response();
    }
    Json(install_snapshot_response(&s, env.from, env.req).await).into_response()
}

pub(crate) async fn install_snapshot_capable(
    State(s): State<Arc<Shared>>,
    Json(env): Json<CapableSnapEnvelope>,
) -> axum::response::Response {
    if env.group_id != s.group_id.0 {
        return (StatusCode::BAD_REQUEST, "group id mismatch").into_response();
    }
    if env.snapshot_nonce == 0
        || s.sm.snapshot_capability() != Some(env.snapshot_capability.as_str())
    {
        return (
            StatusCode::CONFLICT,
            "snapshot capability is not supported by this voter",
        )
            .into_response();
    }
    let response = install_snapshot_response(&s, env.from, env.req).await;
    Json(CapableSnapshotResp {
        term: response.term,
        accepted: response.accepted,
        snapshot_index: response.snapshot_index,
        snapshot_capability: env.snapshot_capability,
        snapshot_nonce: env.snapshot_nonce,
    })
    .into_response()
}

async fn install_snapshot_response(
    s: &Arc<Shared>,
    from: NodeId,
    req: InstallSnapshotReq,
) -> InstallSnapshotResp {
    let serial = Arc::clone(&s.snapshot_install).lock_owned().await;
    let shared = Arc::clone(s);
    match tokio::task::spawn_blocking(move || {
        let _serial = serial;
        shared.install_snapshot_serial(from, req)
    })
    .await
    {
        Ok(response) => response,
        Err(error) => {
            tracing::error!(%error, "raft: snapshot install worker failed");
            *s.latched_failure.lock().unwrap() = Some(StorageFailed {
                node_id: s.id,
                operation: "state-machine-restore-panic",
                path: s.store.path().to_path_buf(),
                kind: std::io::ErrorKind::InvalidData,
            });
            InstallSnapshotResp {
                term: 0,
                accepted: false,
                snapshot_index: 0,
            }
        }
    }
}

pub(crate) async fn timeout_now(
    State(s): State<Arc<Shared>>,
    Json(env): Json<TimeoutNowEnvelope>,
) -> axum::response::Response {
    if env.group_id != s.group_id.0 {
        return (StatusCode::BAD_REQUEST, "group id mismatch").into_response();
    }
    let mut n = s.node.lock().await;
    n.handle(env.from, RaftMsg::TimeoutNow(env.req));
    if s.persist(&n).is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    StatusCode::OK.into_response()
}

/// Leader-side write target (the redirect destination): propose + apply, return
/// the seq; or `421` with a leader hint if this node is not the leader.
pub(crate) async fn publish_handler(
    State(s): State<Arc<Shared>>,
    request: Request,
) -> axum::response::Response {
    let (parts, body) = request.into_parts();
    let body = match to_bytes(body, PUBLISH_BODY_LIMIT).await {
        Ok(body) => body,
        Err(error) => return (StatusCode::BAD_REQUEST, error.to_string()).into_response(),
    };

    // A well-formed foreign envelope is a caller error on every host. This is
    // the only body verdict that precedes the follower routing response.
    if let Ok(env) = serde_json::from_slice::<PublishEnvelope>(&body) {
        if env.group_id != s.group_id.0 {
            return (StatusCode::BAD_REQUEST, "group id mismatch").into_response();
        }
    }

    // Read both values in one node-lock snapshot. A follower must answer with
    // its route before media-type or JSON validation, but must not return a
    // leader hint that came from a different node state.
    let leader = {
        let n = s.node.lock().await;
        if n.is_leader() {
            None
        } else {
            Some(s.leader_url(&n).0)
        }
    };
    if let Some(leader) = leader {
        return (
            StatusCode::MISDIRECTED_REQUEST,
            Json(NotLeader {
                error: "not-leader",
                leader,
            }),
        )
            .into_response();
    }

    // Reconstruct the original request so the elected leader retains Axum's
    // standard Json extractor behavior: 415 for media type, 400 for syntax,
    // and 422 for an invalid envelope shape.
    let request = Request::from_parts(parts, Body::from(body));
    let Json(env) = match Json::<PublishEnvelope>::from_request(request, &()).await {
        Ok(env) => env,
        Err(rejection) => return rejection.into_response(),
    };
    if env.group_id != s.group_id.0 {
        return (StatusCode::BAD_REQUEST, "group id mismatch").into_response();
    }
    match s.try_propose_applied(env.command).await {
        Some(ProposalOutcome::Completed { index: seq }) => {
            (StatusCode::OK, Json(serde_json::json!({ "seq": seq }))).into_response()
        }
        None => {
            let leader = {
                let node = s.node.lock().await;
                s.leader_url(&node).0
            };
            (
                StatusCode::MISDIRECTED_REQUEST,
                Json(NotLeader {
                    error: "not-leader",
                    leader,
                }),
            )
                .into_response()
        }
        Some(ProposalOutcome::RejectedBeforeAdmission { reason }) => {
            if let Some(backpressure) = decode_backpressure(&reason) {
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    [(
                        axum::http::header::RETRY_AFTER,
                        backpressure.retry_after_seconds.to_string(),
                    )],
                    Json(serde_json::json!({
                        "outcome": "rejected_before_admission",
                        "error": backpressure.reason,
                        "retry_after_seconds": backpressure.retry_after_seconds,
                    })),
                )
                    .into_response()
            } else {
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(serde_json::json!({
                        "outcome": "rejected_before_admission",
                        "error": reason,
                    })),
                )
                    .into_response()
            }
        }
        Some(ProposalOutcome::Ambiguous { reason, .. }) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": reason })),
        )
            .into_response(),
        Some(ProposalOutcome::DurabilityFailure { failure, .. }) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": failure.to_string() })),
        )
            .into_response(),
    }
}

pub(crate) async fn host_status(s: &Shared) -> RaftStatus {
    let n = s.node.lock().await;
    let durability_error = s
        .latched_failure
        .lock()
        .unwrap()
        .as_ref()
        .map(|e| e.to_string());
    let conf = n.conf_state();
    let (committed_voters, incoming_voters, membership_phase) = match &conf.outgoing {
        Some(outgoing) => (
            outgoing.clone(),
            Some(conf.membership.voters.clone()),
            MembershipPhase::Joint,
        ),
        None => (
            conf.membership.voters.clone(),
            None,
            MembershipPhase::Stable,
        ),
    };
    let learners = conf.membership.learners.clone();
    let role = if !n.is_voter() {
        "Learner".to_string()
    } else {
        format!("{:?}", n.role())
    };
    RaftStatus {
        group_id: s.group_id.0.clone(),
        id: s.id,
        role,
        term: n.current_term(),
        commit_index: n.commit_index(),
        last_index: n.last_index(),
        snapshot_index: n.snapshot_index(),
        applied_index: s.completed_applied_index(),
        leader: n.leader(),
        is_leader: n.is_leader(),
        durability_error,
        committed_voters,
        incoming_voters,
        learners,
        membership_phase,
        undeliverable_never_addressed: s.undeliverable_never_addressed.load(Ordering::Relaxed),
        undeliverable_withdrawn_address: s.undeliverable_withdrawn_address.load(Ordering::Relaxed),
        proposal_rejected_before_routing: s
            .proposal_rejected_before_routing
            .load(Ordering::Relaxed),
        proposal_rejected_before_append: s.proposal_rejected_before_append.load(Ordering::Relaxed),
        proposal_admission_closed: s.lifecycle_generation.load(Ordering::Acquire) > 0,
        lifecycle_generation: s.lifecycle_generation.load(Ordering::Acquire),
        snapshot_capability: s.sm.snapshot_capability().map(str::to_string),
        resident_log_bytes: n.resident_log_bytes() as u64,
        max_resident_log_bytes: s.max_resident_log_bytes as u64,
    }
}

pub(crate) async fn raftz(State(s): State<Arc<Shared>>) -> Json<RaftStatus> {
    Json(host_status(&s).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::sync::atomic::{AtomicBool, AtomicU64};
    use tokio::sync::oneshot;

    struct PermitPreparation {
        released: StdMutex<Option<oneshot::Sender<()>>>,
    }

    impl Drop for PermitPreparation {
        fn drop(&mut self) {
            if let Some(released) = self
                .released
                .lock()
                .expect("test preparation release mutex poisoned")
                .take()
            {
                let _ = released.send(());
            }
        }
    }

    struct PermitCapture;

    impl crate::PreparedSnapshot for PermitCapture {
        fn write_to(self: Box<Self>, _writer: &mut dyn Write) -> Result<()> {
            Ok(())
        }
    }

    impl crate::SnapshotPreparation for PermitPreparation {
        fn capture_at(self: Box<Self>, _index: Index) -> Result<Box<dyn crate::PreparedSnapshot>> {
            Ok(Box::new(PermitCapture))
        }
    }

    struct PermitPreflightSm {
        preflight_entered: AtomicBool,
        released: StdMutex<Option<oneshot::Sender<()>>>,
    }

    impl RaftStateMachine for PermitPreflightSm {
        fn apply(&self, _index: Index, _command: &[u8]) -> Result<()> {
            Ok(())
        }

        fn snapshot(&self, _writer: &mut dyn Write) -> Result<()> {
            Ok(())
        }

        fn preflight_snapshot(&self) -> Result<Option<Box<dyn SnapshotPreparation>>> {
            self.preflight_entered.store(true, Ordering::Release);
            let released = self
                .released
                .lock()
                .expect("test preflight release mutex poisoned")
                .take();
            Ok(Some(Box::new(PermitPreparation {
                released: StdMutex::new(released),
            })))
        }

        fn restore(&self, _reader: &mut dyn Read) -> Result<()> {
            Ok(())
        }

        fn applied_index(&self) -> Index {
            0
        }
    }

    #[tokio::test]
    async fn preflight_preparation_releases_product_permit_before_waiting_for_snapshot_lease() {
        let (released_tx, mut released_rx) = oneshot::channel();
        let state_machine = Arc::new(PermitPreflightSm {
            preflight_entered: AtomicBool::new(false),
            released: StdMutex::new(Some(released_tx)),
        });
        let snapshot_install = Arc::new(Mutex::new(()));
        let held_lease = Arc::clone(&snapshot_install).lock_owned().await;
        let tracker = Arc::new(RpcTracker::default());
        tracker.active.fetch_add(1, Ordering::SeqCst);
        let operation = Arc::new(RpcGuard { tracker });
        let waiting = tokio::spawn(preflight_snapshot_with_serial(
            state_machine.clone() as Arc<dyn RaftStateMachine>,
            Arc::clone(&snapshot_install),
            operation,
        ));

        let entered = tokio::time::timeout(Duration::from_secs(1), async {
            while !state_machine.preflight_entered.load(Ordering::Acquire) {
                tokio::task::yield_now().await;
            }
        })
        .await;
        let released_while_host_lease_is_held =
            tokio::time::timeout(Duration::from_millis(100), &mut released_rx)
                .await
                .is_ok();

        waiting.abort();
        let _ = waiting.await;
        drop(held_lease);
        if !released_while_host_lease_is_held {
            let _ = released_rx.await;
        }

        assert!(
            entered.is_ok(),
            "preflight must start before lease acquisition"
        );
        assert!(
            released_while_host_lease_is_held,
            "preflight preparation must release its product permit before awaiting snapshot_install"
        );
    }

    struct TestPermit {
        id: u64,
        drops: Arc<AtomicU64>,
    }

    impl Drop for TestPermit {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }

    struct AdmissionSm {
        reject: Option<ProposalBackpressure>,
        next_permit: AtomicU64,
        admitted: AtomicU64,
        applied: AtomicU64,
        drops: Arc<AtomicU64>,
        gate_entered: AtomicBool,
        gate_release: AtomicBool,
        early_watermark_then_fail: AtomicBool,
    }

    impl AdmissionSm {
        fn accepting() -> Arc<Self> {
            Arc::new(Self {
                reject: None,
                next_permit: AtomicU64::new(0),
                admitted: AtomicU64::new(0),
                applied: AtomicU64::new(0),
                drops: Arc::new(AtomicU64::new(0)),
                gate_entered: AtomicBool::new(false),
                gate_release: AtomicBool::new(true),
                early_watermark_then_fail: AtomicBool::new(false),
            })
        }

        fn rejecting(reason: &str, retry_after_seconds: u64) -> Arc<Self> {
            Arc::new(Self {
                reject: Some(ProposalBackpressure {
                    reason: reason.to_string(),
                    retry_after_seconds,
                }),
                next_permit: AtomicU64::new(0),
                admitted: AtomicU64::new(0),
                applied: AtomicU64::new(0),
                drops: Arc::new(AtomicU64::new(0)),
                gate_entered: AtomicBool::new(false),
                gate_release: AtomicBool::new(true),
                early_watermark_then_fail: AtomicBool::new(false),
            })
        }
    }

    impl RaftStateMachine for AdmissionSm {
        fn admit_proposal(&self, _command: &[u8]) -> anyhow::Result<Option<AdmissionPermit>> {
            if let Some(backpressure) = &self.reject {
                return Err(anyhow::Error::new(backpressure.clone()));
            }
            let id = self.next_permit.fetch_add(1, Ordering::SeqCst) + 1;
            self.admitted.fetch_add(1, Ordering::SeqCst);
            Ok(Some(Box::new(TestPermit {
                id,
                drops: self.drops.clone(),
            })))
        }

        fn apply_admitted(
            &self,
            index: Index,
            _command: &[u8],
            permit: Option<AdmissionPermit>,
        ) -> anyhow::Result<()> {
            let permit =
                permit.map(|permit| permit.downcast::<TestPermit>().expect("test permit type"));
            if let Some(permit) = &permit {
                assert_eq!(permit.id, index, "permit identity follows Raft index");
            }
            if self.early_watermark_then_fail.load(Ordering::SeqCst) {
                self.applied.store(index, Ordering::SeqCst);
            }
            self.gate_entered.store(true, Ordering::SeqCst);
            while !self.gate_release.load(Ordering::SeqCst) {
                std::thread::yield_now();
            }
            if self.early_watermark_then_fail.load(Ordering::SeqCst) {
                anyhow::bail!("injected error after an early state-machine watermark");
            }
            self.applied.store(index, Ordering::SeqCst);
            drop(permit);
            Ok(())
        }

        fn apply(&self, index: Index, _command: &[u8]) -> anyhow::Result<()> {
            self.applied.store(index, Ordering::SeqCst);
            Ok(())
        }
        fn snapshot(&self, _writer: &mut dyn Write) -> anyhow::Result<()> {
            Ok(())
        }
        fn restore(&self, _reader: &mut dyn Read) -> anyhow::Result<()> {
            Ok(())
        }
        fn applied_index(&self) -> Index {
            self.applied.load(Ordering::SeqCst)
        }
    }

    struct AdmissionTestHost {
        host: RaftHost,
        _directory: tempfile::TempDir,
    }
    impl std::ops::Deref for AdmissionTestHost {
        type Target = RaftHost;
        fn deref(&self) -> &Self::Target {
            &self.host
        }
    }
    async fn elected_single_host(sm: Arc<AdmissionSm>) -> AdmissionTestHost {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();
        let host = RaftHost::spawn(
            0,
            Membership {
                voters: vec![0],
                learners: vec![],
            },
            HashMap::new(),
            RaftStore::open(path.to_str().unwrap(), 0, crate::store::FsyncPolicy::Os).unwrap(),
            sm as Arc<dyn RaftStateMachine>,
            HostConfig::default(),
        );
        {
            let mut node = host.shared.node.lock().await;
            for _ in 0..raft_core::ELECTION_TIMEOUT_FLOOR_TICKS {
                node.tick();
            }
        }
        AdmissionTestHost {
            host,
            _directory: dir,
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn proposal_ack_waits_for_callback_completion_not_an_early_watermark() {
        struct ReleaseGate(Arc<AdmissionSm>);
        impl Drop for ReleaseGate {
            fn drop(&mut self) {
                self.0.gate_release.store(true, Ordering::SeqCst);
            }
        }
        let sm = AdmissionSm::accepting();
        let host = Arc::new(elected_single_host(sm.clone()).await);
        sm.early_watermark_then_fail.store(true, Ordering::SeqCst);
        sm.gate_release.store(false, Ordering::SeqCst);
        let release = ReleaseGate(sm.clone());
        let submitting = host.clone();
        let submit =
            tokio::spawn(async move { submitting.propose_outcome(b"early".to_vec()).await });
        let entered = tokio::time::timeout(Duration::from_secs(2), async {
            while !sm.gate_entered.load(Ordering::SeqCst) {
                tokio::task::yield_now().await;
            }
        })
        .await;
        let before = tokio::time::timeout(Duration::from_secs(2), host_status(&host.shared)).await;
        // Give the public waiter a chance to observe the premature SM watermark.
        tokio::time::sleep(Duration::from_millis(30)).await;
        let acknowledged_while_callback_blocked = submit.is_finished();
        drop(release);
        let joined = tokio::time::timeout(Duration::from_secs(2), submit).await;
        let after = tokio::time::timeout(Duration::from_secs(2), host_status(&host.shared)).await;
        assert!(
            entered.is_ok(),
            "apply callback must reach the early watermark gate"
        );
        assert!(
            !acknowledged_while_callback_blocked,
            "public proposal returned before apply callback completed"
        );
        assert_eq!(
            before.unwrap().applied_index,
            0,
            "public applied head requires callback completion"
        );
        assert!(matches!(
            joined.unwrap().unwrap(),
            ProposalOutcome::DurabilityFailure { index: Some(1), .. }
        ));
        assert_eq!(
            after.unwrap().applied_index,
            0,
            "failed callback must not publish an applied head"
        );
    }

    #[tokio::test]
    async fn proposal_admission_rejects_before_raft_append() {
        let sm = AdmissionSm::rejecting("test budget full", 7);
        let host = elected_single_host(sm).await;
        let error = host.propose(b"full".to_vec()).await.unwrap_err();
        let backpressure = error.downcast_ref::<ProposalBackpressure>().unwrap();
        assert_eq!(backpressure.retry_after_seconds, 7);
        assert_eq!(host.shared.node.lock().await.last_index(), 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn indexed_permit_survives_cancelled_producer_until_apply_returns() {
        struct ReleaseGate(Arc<AdmissionSm>);
        impl Drop for ReleaseGate {
            fn drop(&mut self) {
                self.0.gate_release.store(true, Ordering::SeqCst);
            }
        }
        let sm = AdmissionSm::accepting();
        sm.gate_release.store(false, Ordering::SeqCst);
        let release = ReleaseGate(sm.clone());
        let host = Arc::new(elected_single_host(sm.clone()).await);
        let mut submit = tokio::spawn({
            let host = host.clone();
            async move { host.propose(vec![1]).await }
        });
        let entered = tokio::time::timeout(Duration::from_secs(2), async {
            while !sm.gate_entered.load(Ordering::SeqCst) {
                tokio::task::yield_now().await;
            }
        })
        .await;
        let before_cancel = sm.drops.load(Ordering::SeqCst);
        submit.abort();
        let after_cancel = sm.drops.load(Ordering::SeqCst);
        // Release the worker before any assertion or join, including failures.
        drop(release);
        let joined = tokio::time::timeout(Duration::from_secs(2), &mut submit).await;
        let applied = tokio::time::timeout(Duration::from_secs(2), async {
            // The SM floor can precede callback return and permit drop.
            // Observe the host publication, which follows both.
            while host.shared.completed_applied_index() < 1 {
                tokio::task::yield_now().await;
            }
        })
        .await;
        assert!(entered.is_ok(), "apply did not reach the gate");
        assert!(
            joined.is_ok(),
            "cancelled producer did not finish after release"
        );
        assert!(
            applied.is_ok(),
            "committed apply did not finish after release"
        );
        assert_eq!(before_cancel, 0, "permit must reach the apply callback");
        assert_eq!(after_cancel, 0, "cancellation cannot drop the apply permit");
        assert_eq!(sm.drops.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn identical_commands_get_distinct_index_permits() {
        let sm = AdmissionSm::accepting();
        let host = elected_single_host(sm.clone()).await;
        assert_eq!(host.propose(vec![9]).await.unwrap(), 1);
        assert_eq!(host.propose(vec![9]).await.unwrap(), 2);
        assert_eq!(sm.admitted.load(Ordering::SeqCst), 2);
        assert_eq!(sm.drops.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn persistence_failure_after_index_keeps_the_host_owned_permit() {
        let sm = AdmissionSm::accepting();
        let host = elected_single_host(sm.clone()).await;
        host.shared
            .store
            .inject_next_save_failure_with_kind(std::io::ErrorKind::Other);
        assert!(host.propose(vec![4]).await.is_err());
        assert_eq!(host.shared.node.lock().await.last_index(), 1);
        assert_eq!(
            host.shared.pending_admission.lock().unwrap().len(),
            1,
            "the appended index still owns its permit after uncertain persistence"
        );
        assert_eq!(sm.drops.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn forwarded_backpressure_reply_restores_typed_error_data() {
        let leader = elected_single_host(AdmissionSm::rejecting("leader full", 11)).await;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, leader.router()).await.unwrap();
        });
        let client = elected_single_host(AdmissionSm::accepting()).await;
        let outcome = client
            .forward(&format!("http://{address}"), b"forwarded")
            .await;
        server.abort();
        let ProposalOutcome::RejectedBeforeAdmission { reason } = outcome else {
            panic!("forwarded 429 must remain a pre-append rejection");
        };
        assert_eq!(
            decode_backpressure(&reason),
            Some(ProposalBackpressure {
                reason: "leader full".to_string(),
                retry_after_seconds: 11,
            })
        );
    }

    #[test]
    fn conflicting_term_at_same_index_drops_only_replaced_uncommitted_permit() {
        let membership = Membership {
            voters: vec![0, 1],
            learners: vec![],
        };
        let mut node = RaftNode::new(0, &membership);
        for _ in 0..raft_core::ELECTION_TIMEOUT_FLOOR_TICKS {
            node.tick();
        }
        node.handle(
            1,
            RaftMsg::VoteResp(VoteResp {
                term: 1,
                granted: true,
            }),
        );
        let index = node.propose(vec![1]).unwrap();
        let old_term = node.current_term();
        assert_eq!(index, 1);
        node.handle(
            1,
            RaftMsg::Append(AppendReq {
                term: old_term + 1,
                leader: 1,
                prev_log_index: 0,
                prev_log_term: 0,
                entries: vec![raft_core::RaftEntry {
                    term: old_term + 1,
                    index,
                    command: vec![2],
                    kind: raft_core::EntryKind::Command,
                }],
                leader_commit: index,
            }),
        );
        let sm = AdmissionSm::accepting();
        let drops = Arc::new(AtomicU64::new(0));
        let pending = StdMutex::new(BTreeMap::from([(
            (index, old_term),
            Box::new(TestPermit {
                id: 99,
                drops: drops.clone(),
            }) as AdmissionPermit,
        )]));
        apply_ready_with_admission(
            &mut node,
            sm.as_ref(),
            None,
            SnapshotPolicy::Disabled,
            true,
            Some(&pending),
        )
        .unwrap();
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        assert!(pending.lock().unwrap().is_empty());
        assert_eq!(sm.applied_index(), index);
    }

    #[test]
    fn shutdown_phase_cutoffs_are_cumulative_and_ordered() {
        let started_at = tokio::time::Instant::now();
        let deadline = ShutdownDeadline {
            expires_at: started_at + Duration::from_millis(100),
            total: Duration::from_millis(100),
            reserve: Duration::ZERO,
        };

        let cutoffs = shutdown_phase_cutoffs(deadline, started_at);

        assert_eq!(cutoffs[0], started_at + Duration::from_millis(25));
        assert_eq!(cutoffs[1], started_at + Duration::from_millis(50));
        assert_eq!(cutoffs[2], started_at + Duration::from_millis(75));
        assert_eq!(cutoffs[3], deadline.expires_at);
        assert!(cutoffs.windows(2).all(|pair| pair[0] <= pair[1]));
    }

    #[test]
    fn shutdown_phase_cutoffs_handle_zero_and_very_short_usable_intervals() {
        let started_at = tokio::time::Instant::now();
        let zero = ShutdownDeadline {
            expires_at: started_at,
            total: Duration::ZERO,
            reserve: Duration::ZERO,
        };
        assert_eq!(shutdown_phase_cutoffs(zero, started_at), [started_at; 4]);

        let short = ShutdownDeadline {
            expires_at: started_at + Duration::from_nanos(3),
            total: Duration::from_nanos(3),
            reserve: Duration::ZERO,
        };
        assert_eq!(
            shutdown_phase_cutoffs(short, started_at),
            [
                started_at,
                started_at,
                started_at,
                started_at + Duration::from_nanos(3),
            ]
        );
    }

    #[test]
    fn shutdown_phase_cutoffs_keep_remainder_for_final_cutoff_and_preserve_reserve() {
        let started_at = tokio::time::Instant::now();
        let deadline = ShutdownDeadline {
            expires_at: started_at + Duration::from_nanos(11),
            total: Duration::from_nanos(11),
            reserve: Duration::from_nanos(2),
        };

        let cutoffs = shutdown_phase_cutoffs(deadline, started_at);
        let usable_end = deadline.expires_at - deadline.reserve;

        assert_eq!(cutoffs[0], started_at + Duration::from_nanos(2));
        assert_eq!(cutoffs[1], started_at + Duration::from_nanos(4));
        assert_eq!(cutoffs[2], started_at + Duration::from_nanos(6));
        assert_eq!(cutoffs[3], usable_end);
        assert_eq!(
            deadline.expires_at.duration_since(cutoffs[3]),
            deadline.reserve
        );
    }

    #[test]
    fn expired_background_cutoff_is_detected_before_absent_tasks_are_skipped() {
        let cutoff = tokio::time::Instant::now() - Duration::from_millis(1);

        assert!(shutdown_phase_cutoff_elapsed(cutoff));
    }

    #[test]
    fn publish_path_is_the_canonical_peer_write_route() {
        assert_eq!(RaftHost::PUBLISH_PATH, "/raft/publish");
    }

    fn append(marker: u64) -> RaftMsg {
        RaftMsg::Append(AppendReq {
            term: marker,
            leader: 7,
            prev_log_index: 0,
            prev_log_term: 0,
            entries: vec![],
            leader_commit: marker,
        })
    }

    fn timeout_now(marker: u64) -> RaftMsg {
        RaftMsg::TimeoutNow(TimeoutNowReq {
            term: marker,
            leader: 7,
        })
    }

    fn assert_append(message: Option<RaftMsg>, marker: u64) {
        match message {
            Some(RaftMsg::Append(req)) => assert_eq!(req.leader_commit, marker),
            other => panic!("expected Append({marker}), got {other:?}"),
        }
    }

    fn assert_timeout_now(message: Option<RaftMsg>, term: u64, leader: u64) {
        match message {
            Some(RaftMsg::TimeoutNow(req)) => {
                assert_eq!(req.term, term);
                assert_eq!(req.leader, leader);
            }
            other => panic!("expected TimeoutNow({term}, {leader}), got {other:?}"),
        }
    }

    fn assert_same_message(actual: RaftMsg, expected: RaftMsg) {
        match (actual, expected) {
            (RaftMsg::Vote(actual), RaftMsg::Vote(expected)) => {
                assert_eq!(actual.term, expected.term);
                assert_eq!(actual.candidate, expected.candidate);
                assert_eq!(actual.last_log_index, expected.last_log_index);
                assert_eq!(actual.last_log_term, expected.last_log_term);
            }
            (RaftMsg::VoteResp(actual), RaftMsg::VoteResp(expected)) => {
                assert_eq!(actual.term, expected.term);
                assert_eq!(actual.granted, expected.granted);
            }
            (RaftMsg::Append(actual), RaftMsg::Append(expected)) => {
                assert_eq!(actual.term, expected.term);
                assert_eq!(actual.leader, expected.leader);
                assert_eq!(actual.prev_log_index, expected.prev_log_index);
                assert_eq!(actual.prev_log_term, expected.prev_log_term);
                assert_eq!(actual.entries, expected.entries);
                assert_eq!(actual.leader_commit, expected.leader_commit);
            }
            (RaftMsg::AppendResp(actual), RaftMsg::AppendResp(expected)) => {
                assert_eq!(actual.term, expected.term);
                assert_eq!(actual.success, expected.success);
                assert_eq!(actual.match_index, expected.match_index);
            }
            (RaftMsg::InstallSnapshot(actual), RaftMsg::InstallSnapshot(expected)) => {
                assert_eq!(actual.term, expected.term);
                assert_eq!(actual.leader, expected.leader);
                assert_eq!(actual.snapshot_index, expected.snapshot_index);
                assert_eq!(actual.snapshot_term, expected.snapshot_term);
                assert_eq!(actual.data, expected.data);
            }
            (RaftMsg::InstallSnapshotResp(actual), RaftMsg::InstallSnapshotResp(expected)) => {
                assert_eq!(actual.term, expected.term);
                assert_eq!(actual.accepted, expected.accepted);
                assert_eq!(actual.snapshot_index, expected.snapshot_index);
            }
            (RaftMsg::TimeoutNow(actual), RaftMsg::TimeoutNow(expected)) => {
                assert_eq!(actual.term, expected.term);
                assert_eq!(actual.leader, expected.leader);
            }
            (actual, expected) => panic!("expected {expected:?}, got {actual:?}"),
        }
    }

    #[test]
    fn peer_lane_keeps_timeout_now_before_later_append_when_worker_is_occupied() {
        let mut queue = PeerLaneQueue::default();

        queue.enqueue(append(10));
        assert_append(queue.dequeue(), 10); // The lane worker is now in flight.

        queue.enqueue(timeout_now(20));
        queue.enqueue(append(30)); // A later leader heartbeat.

        assert_timeout_now(queue.dequeue(), 20, 7);
        assert_append(queue.dequeue(), 30);
        assert!(queue.dequeue().is_none());
    }

    #[test]
    fn peer_lane_coalesces_only_adjacent_pending_appends() {
        let mut adjacent = PeerLaneQueue::default();
        adjacent.enqueue(append(10));
        adjacent.enqueue(append(20));
        assert_append(adjacent.dequeue(), 20);
        assert!(adjacent.dequeue().is_none());

        let mut separated = PeerLaneQueue::default();
        separated.enqueue(append(10));
        separated.enqueue(timeout_now(20));
        separated.enqueue(append(30));

        assert_append(separated.dequeue(), 10);
        assert_timeout_now(separated.dequeue(), 20, 7);
        assert_append(separated.dequeue(), 30);
        assert!(separated.dequeue().is_none());
    }

    #[test]
    fn peer_lane_keeps_every_non_append_message_lossless_and_fifo() {
        let mut queue = PeerLaneQueue::default();
        let expected = vec![
            RaftMsg::Vote(VoteReq {
                term: 10,
                candidate: 1,
                last_log_index: 2,
                last_log_term: 3,
            }),
            RaftMsg::VoteResp(VoteResp {
                term: 20,
                granted: true,
            }),
            RaftMsg::AppendResp(AppendResp {
                term: 30,
                success: true,
                match_index: 4,
            }),
            RaftMsg::InstallSnapshot(InstallSnapshotReq {
                term: 40,
                leader: 1,
                snapshot_index: 5,
                snapshot_term: 6,
                data: vec![7],
            }),
            RaftMsg::InstallSnapshotResp(InstallSnapshotResp {
                term: 50,
                accepted: true,
                snapshot_index: 8,
            }),
            timeout_now(60),
            append(70),
        ];
        for message in expected.iter().cloned() {
            queue.enqueue(message);
        }

        for expected_message in expected {
            let actual = queue.dequeue().expect("every queued message is retained");
            assert_same_message(actual, expected_message);
        }
        assert!(queue.dequeue().is_none());
    }

    fn fresh_node(membership: Membership) -> RaftNode {
        RaftNode::new(0, &membership)
    }

    #[test]
    fn tick_persists_initial_learner_image_then_skips_unchanged_tick() {
        let mut node = fresh_node(Membership {
            voters: vec![],
            learners: vec![0],
        });
        let initial = node.persisted();
        let mut first_tick = true;
        let mut calls = 0;

        assert!(tick_then_maybe_persist(
            &mut node,
            &mut first_tick,
            |persisted| {
                calls += 1;
                assert_eq!(persisted.persisted(), initial);
                true
            },
            || false,
        ));
        assert_eq!(calls, 1, "first learner tick persists fresh membership");
        assert_eq!(node.persisted(), initial, "timers are not durable state");

        assert!(tick_then_maybe_persist(
            &mut node,
            &mut first_tick,
            |_| panic!("unchanged learner tick must not persist"),
            || false,
        ));
        assert_eq!(node.persisted(), initial, "unchanged tick keeps full image");
    }

    #[test]
    fn tick_persists_election_image_and_failure_blocks_apply_and_stays_latched() {
        let mut node = fresh_node(Membership {
            voters: vec![0],
            learners: vec![],
        });
        let mut first_tick = true;
        let mut calls = 0;
        for _ in 0..raft_core::ELECTION_TIMEOUT_FLOOR_TICKS - 1 {
            assert!(tick_then_maybe_persist(
                &mut node,
                &mut first_tick,
                |_| {
                    calls += 1;
                    true
                },
                || false,
            ));
        }
        assert_eq!(calls, 1, "only startup has persisted before election");

        let latched = std::cell::Cell::new(false);
        assert!(!tick_then_maybe_persist(
            &mut node,
            &mut first_tick,
            |persisted| {
                calls += 1;
                let image = persisted.persisted();
                assert_eq!(image.term, 1);
                assert_eq!(image.voted_for, Some(0));
                latched.set(true);
                false
            },
            || latched.get(),
        ));
        assert_eq!(calls, 2, "term-changing election persists");
        assert!(!tick_then_maybe_persist(
            &mut node,
            &mut first_tick,
            |_| panic!("latched unchanged tick must not persist"),
            || true,
        ));
    }

    #[test]
    fn tick_persists_joint_election_with_leave_joint_entry() {
        let membership = Membership {
            voters: vec![0],
            learners: vec![],
        };
        let mut node = fresh_node(membership.clone());
        assert!(node.adopt_conf(raft_core::ConfState {
            membership,
            outgoing: Some(vec![0]),
            generation: 1,
        }));
        let mut first_tick = true;
        for _ in 0..raft_core::ELECTION_TIMEOUT_FLOOR_TICKS - 1 {
            assert!(tick_then_maybe_persist(
                &mut node,
                &mut first_tick,
                |_| true,
                || false,
            ));
        }
        assert!(tick_then_maybe_persist(
            &mut node,
            &mut first_tick,
            |persisted| {
                let image = persisted.persisted();
                assert_eq!(image.term, 1);
                assert!(image.conf.as_ref().is_some_and(|conf| conf.outgoing.is_some()));
                assert!(matches!(
                    image.log.last().map(|entry| entry.kind),
                    Some(raft_core::EntryKind::Config)
                ));
                true
            },
            || false,
        ));
    }

    #[test]
    fn elected_leader_heartbeat_ticks_skip_persistence_and_keep_full_image() {
        let mut node = fresh_node(Membership {
            voters: vec![0],
            learners: vec![],
        });
        let mut first_tick = true;
        for _ in 0..raft_core::ELECTION_TIMEOUT_FLOOR_TICKS - 1 {
            assert!(tick_then_maybe_persist(
                &mut node,
                &mut first_tick,
                |_| true,
                || false,
            ));
        }
        assert!(tick_then_maybe_persist(
            &mut node,
            &mut first_tick,
            |_| true,
            || false,
        ));
        assert!(node.is_leader());
        let leader_image = node.persisted();

        for _ in 0..raft_core::HEARTBEAT_INTERVAL_TICKS * 2 {
            assert!(tick_then_maybe_persist(
                &mut node,
                &mut first_tick,
                |_| panic!("idle leader heartbeat tick must not persist"),
                || false,
            ));
            assert_eq!(
                node.persisted(),
                leader_image,
                "heartbeat timers and outbox must not alter the durable image"
            );
        }
    }
}

enum Route {
    Local,
    Remote(String),
    Unknown,
}
// CODEGEN-END
