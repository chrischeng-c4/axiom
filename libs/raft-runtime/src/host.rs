// SPEC-MANAGED: libs/raft-runtime/tech-design/semantic/source/libs-raft-runtime-src-host-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! `RaftHost` — drives a [`raft_core::RaftNode`] for a [`RaftStateMachine`] over
//! an h2c peer transport, with read-your-write `propose` and snapshot/compaction.
//!
//! Generalizes the per-service drivers (relay/lumen/keep each hand-rolled this):
//! the host is the **sole applier** — committed entries are fed to the state
//! machine in index order under the node lock, so `propose` can return after the
//! command *applies* (not just commits), and `compact(applied, snapshot)` is
//! always sound.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex as StdMutex, RwLock as StdRwLock};
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use axum::extract::State;
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
use tokio::sync::{watch, Mutex, Notify};
use tokio::task::JoinHandle;

use crate::config::{HostConfig, SnapshotPolicy};
use crate::group::{GroupId, LEGACY_GROUP_ID};
use crate::peer_transport::PeerTransport;
use crate::state_machine::{Command, RaftStateMachine};
use crate::store::RaftStore;

/// The maximum size of a snapshot chunk streamed during snapshot generation.
pub const SNAPSHOT_CHUNK_SIZE: usize = 64 * 1024;

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
    pub fn streaming<H>(chunk_size: usize, mut handler: H) -> ChunkSink<Box<dyn FnMut(&[u8]) -> std::io::Result<()> + Send>>
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
    pub proposal_admission_closed: bool,
    pub lifecycle_generation: u64,
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
    pub(crate) lifecycle_generation: AtomicU64,
    pub(crate) shutdown_started: AtomicBool,
    pub(crate) shutdown_tx: watch::Sender<Option<HostShutdownReport>>,
}

#[derive(Default)]
pub(crate) struct RpcTracker {
    pub(crate) active: AtomicUsize,
    pub(crate) idle: Notify,
}

#[derive(Default)]
pub(crate) struct PeerLane {
    pub(crate) pending: Mutex<Option<RaftMsg>>,
    pub(crate) running: AtomicBool,
}

impl RpcTracker {
    pub(crate) async fn wait_idle(&self) {
        loop {
            let notified = self.idle.notified();
            if self.active.load(Ordering::Acquire) == 0 {
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

/// @spec libs/raft-runtime/tech-design/semantic/source/libs-raft-runtime-src-host-rs.md#source
impl Shared {
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
        match self.store.save(&node.persisted()) {
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

    /// The single applier. Called under the node lock everywhere committed
    /// entries can appear (tick, inbound append, reply feedback, propose).
    /// Installs any received snapshot, applies newly committed entries to the
    /// state machine in order, bumps `applied_tx`, then maybe compacts.
    fn apply_ready(&self, node: &mut RaftNode) {
        // 1. install a received snapshot first.
        if let Some(bytes) = node.take_installed_snapshot() {
            let mut reader = std::io::Cursor::new(bytes);
            if let Err(e) = self.sm.restore(&mut reader) {
                tracing::error!(error = %e, "raft: state-machine restore from snapshot failed");
            }
        }
        // 2. apply newly committed entries (idempotent: skip <= applied).
        let mut advanced = false;
        for e in node.take_committed() {
            if e.index <= self.sm.applied_index() {
                continue;
            }
            if let Err(err) = self.sm.apply(e.index, &e.command) {
                tracing::warn!(index = e.index, error = %err, "raft: apply error (entry no-ops)");
            }
            advanced = true;
        }
        if advanced {
            let _ = self.applied_tx.send_replace(self.sm.applied_index());
        }
        // 3. compaction (gated on policy; safe — the SM and node share one applier).
        self.maybe_compact(node);
    }

    fn maybe_compact(&self, node: &mut RaftNode) {
        let SnapshotPolicy::EveryEntries(n) = self.cfg.snapshot else {
            return;
        };
        let applied = self.sm.applied_index();
        if applied == 0 || applied.saturating_sub(node.snapshot_index()) < n {
            return;
        }
        let mut sink = ChunkSink::new(SNAPSHOT_CHUNK_SIZE);
        match self.sm.snapshot(&mut sink) {
            Ok(()) => {
                let bytes = sink.into_bytes();
                node.compact(applied, bytes);
            }
            Err(e) => tracing::warn!(error = %e, "raft: snapshot capture failed; skip compaction"),
        }
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
                *pending = Some(o.msg);
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
                            match pending.take() {
                                Some(msg) => Some(msg),
                                None => {
                                    // Producers update `pending` and observe
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
                .post(
                    &format!("{base}/raft/install-snapshot"),
                    &SnapEnvelope {
                        group_id: self.group_id.0.clone(),
                        from: self.id,
                        req,
                    },
                )
                .await
                .and_then(|r| serde_json::from_slice::<InstallSnapshotResp>(&r).ok())
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
        match self
            .http_client()
            .post(url)
            .timeout(self.cfg.rpc_timeout)
            .json(body)
            .send()
            .await
        {
            Ok(r) => r.bytes().await.ok().map(|b| b.to_vec()),
            Err(_) => None,
        }
    }

    /// Try a leader-side proposal and return its index once the **state machine
    /// applies** it (read-your-write). `None` means this node was no longer
    /// leader when the node lock was acquired, so the caller may safely
    /// re-route the command: it was not appended. Once an index is allocated,
    /// an apply timeout remains an error and must not be retried blindly.
    async fn try_propose_applied(self: &Arc<Self>, command: Command) -> Option<ProposalOutcome> {
        if self.lifecycle_generation.load(Ordering::Acquire) > 0 {
            return Some(ProposalOutcome::RejectedBeforeAdmission {
                reason: "raft: proposal admission closed".to_string(),
            });
        }
        let index = {
            let mut n = self.node.lock().await;
            let Some(idx) = n.propose(command) else {
                return None;
            };
            if let Err(e) = self.persist(&n) {
                return Some(ProposalOutcome::DurabilityFailure {
                    index: Some(idx),
                    failure: e,
                });
            }
            self.apply_ready(&mut n); // sole voter commits+applies here
            idx
        };
        self.flush().await;

        if self.sm.applied_index() >= index {
            return Some(ProposalOutcome::Completed { index });
        }
        let mut rx = self.applied_tx.subscribe();
        let deadline = Instant::now() + self.cfg.propose_timeout;
        loop {
            {
                let mut n = self.node.lock().await;
                self.apply_ready(&mut n);
                if self.sm.applied_index() >= index {
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
/// @spec libs/raft-runtime/tech-design/semantic/source/libs-raft-runtime-src-host-rs.md#source
pub struct RaftHost {
    pub(crate) shared: Arc<Shared>,
    tasks: StdMutex<Option<(JoinHandle<()>, JoinHandle<()>)>>,
}

/// @spec libs/raft-runtime/tech-design/semantic/source/libs-raft-runtime-src-host-rs.md#source
impl Drop for RaftHost {
    fn drop(&mut self) {
        if let Some((tick, pump)) = self.tasks.lock().expect("raft task mutex poisoned").take() {
            tick.abort();
            pump.abort();
        }
    }
}

/// @spec libs/raft-runtime/tech-design/semantic/source/libs-raft-runtime-src-host-rs.md#source
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
        let mut node = match store.load().ok().flatten() {
            Some(state) => RaftNode::from_persisted(id, &membership, state),
            None => RaftNode::new(id, &membership),
        };
        // Cold-start: drive any committed-but-unapplied resident entries (and a
        // persisted snapshot) into the state machine before serving — inline,
        // before the node enters the Mutex (no async lock available here).
        if let Some(bytes) = node.take_installed_snapshot() {
            let mut reader = std::io::Cursor::new(bytes);
            if let Err(e) = sm.restore(&mut reader) {
                tracing::error!(error = %e, "raft: cold-start snapshot restore failed");
            }
        }
        for e in node.take_committed() {
            if e.index > sm.applied_index() {
                if let Err(err) = sm.apply(e.index, &e.command) {
                    tracing::warn!(index = e.index, error = %err, "raft: cold-start apply error");
                }
            }
        }

        let client =
            transport_h2c::h2c_client_with(Some(cfg.rpc_timeout), None).expect("h2c client");
        let (applied_tx, _rx) = watch::channel(sm.applied_index());
        let (shutdown_tx, _shutdown_rx) = watch::channel(None);
        let peer_lanes = peers
            .keys()
            .copied()
            .map(|peer| (peer, Arc::new(PeerLane::default())))
            .collect();
        let shared = Arc::new(Shared {
            id,
            group_id,
            node: Mutex::new(node),
            store,
            sm,
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
            lifecycle_generation: AtomicU64::new(0),
            shutdown_started: AtomicBool::new(false),
            shutdown_tx,
        });

        let s = Arc::clone(&shared);
        let tick = tokio::spawn(async move {
            loop {
                tokio::time::sleep(s.cfg.tick).await;
                {
                    let mut n = s.node.lock().await;
                    n.tick();
                    if s.persist(&n).is_ok() {
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

        // Phase 1: Quiesce
        let start = Instant::now();
        let budget = deadline.usable_remaining();
        if budget.is_zero() {
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
        let budget = deadline.usable_remaining();
        if budget.is_zero() {
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
        match tokio::time::timeout(budget, self.handoff_leadership()).await {
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
        let budget = deadline.usable_remaining();
        if budget.is_zero() {
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
        let tasks = self.tasks.lock().expect("raft task mutex poisoned").take();
        let timed_out = if let Some((tick, pump)) = tasks {
            tick.abort();
            pump.abort();
            let join_all = async move {
                let _ = tick.await;
                let _ = pump.await;
            };
            tokio::time::timeout(budget, join_all).await.is_err()
        } else {
            false
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
        let budget = deadline.usable_remaining();
        if budget.is_zero() {
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
        if tokio::time::timeout(budget, self.shared.rpc_tracker.wait_idle())
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
        let deadline = ShutdownDeadline::from_now(timeout, Duration::ZERO)
            .map_err(|e| anyhow!("{e}"))?;
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
            ProposalOutcome::RejectedBeforeAdmission { reason } => Err(anyhow!(reason)),
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
                    ProposalOutcome::RejectedBeforeAdmission { reason } => {
                        last_route_error = Some(reason);
                    }
                    ProposalOutcome::Ambiguous { index: None, reason } => {
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
            .post(format!("{leader_url}/raft/publish"))
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
        if resp.status() != StatusCode::OK {
            return ProposalOutcome::Ambiguous {
                index: None,
                reason: format!("raft: leader redirect returned {}", resp.status()),
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
        while self.shared.sm.applied_index() < seq {
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
        let mut n = self.shared.node.lock().await;
        let applied = self.shared.sm.applied_index();
        if applied == 0 {
            return Ok(0);
        }
        let mut sink = ChunkSink::new(SNAPSHOT_CHUNK_SIZE);
        self.shared.sm.snapshot(&mut sink)?;
        let bytes = sink.into_bytes();
        n.compact(applied, bytes);
        self.shared.persist(&n)?;
        Ok(applied)
    }

    /// Peer raft RPCs + producer forward + status; merge into the service app so
    /// they ride the h2c serve port.
    pub fn router(&self) -> Router {
        Router::new()
            .route("/raft/request-vote", post(request_vote))
            .route("/raft/append-entries", post(append_entries))
            .route("/raft/install-snapshot", post(install_snapshot))
            .route("/raft/timeout-now", post(timeout_now))
            .route("/raft/publish", post(publish_handler))
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
    let mut n = s.node.lock().await;
    n.handle(env.from, RaftMsg::InstallSnapshot(env.req));
    if s.persist(&n).is_err() {
        return Json(InstallSnapshotResp {
            term: 0,
            snapshot_index: 0,
        })
        .into_response();
    }
    s.apply_ready(&mut n); // restores the SM from the installed snapshot
    Json(match take_reply(&mut n, env.from) {
        Some(RaftMsg::InstallSnapshotResp(r)) => r,
        _ => InstallSnapshotResp {
            term: 0,
            snapshot_index: 0,
        },
    })
    .into_response()
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
    Json(env): Json<PublishEnvelope>,
) -> axum::response::Response {
    if env.group_id != s.group_id.0 {
        return (StatusCode::BAD_REQUEST, "group id mismatch").into_response();
    }
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
        Some(ProposalOutcome::RejectedBeforeAdmission { reason }) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": reason })),
        )
            .into_response(),
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
        applied_index: s.sm.applied_index(),
        leader: n.leader(),
        is_leader: n.is_leader(),
        durability_error,
        committed_voters,
        incoming_voters,
        learners,
        membership_phase,
        undeliverable_never_addressed: s
            .undeliverable_never_addressed
            .load(Ordering::Relaxed),
        undeliverable_withdrawn_address: s
            .undeliverable_withdrawn_address
            .load(Ordering::Relaxed),
        proposal_admission_closed: s.lifecycle_generation.load(Ordering::Acquire) > 0,
        lifecycle_generation: s.lifecycle_generation.load(Ordering::Acquire),
    }
}

pub(crate) async fn raftz(State(s): State<Arc<Shared>>) -> Json<RaftStatus> {
    Json(host_status(&s).await)
}

enum Route {
    Local,
    Remote(String),
    Unknown,
}
// CODEGEN-END
