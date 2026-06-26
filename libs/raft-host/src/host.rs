//! `RaftHost` — drives a [`raft_core::RaftNode`] for a [`RaftStateMachine`] over
//! an h2c peer transport, with read-your-write `propose` and snapshot/compaction.
//!
//! Generalizes the per-service drivers (relay/lumen/keep each hand-rolled this):
//! the host is the **sole applier** — committed entries are fed to the state
//! machine in index order under the node lock, so `propose` can return after the
//! command *applies* (not just commits), and `compact(applied, snapshot)` is
//! always sound.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Result};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use raft_core::{
    AppendReq, AppendResp, Index, InstallSnapshotReq, InstallSnapshotResp, Membership, NodeId,
    RaftMsg, RaftNode, VoteReq, VoteResp,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{watch, Mutex};
use tokio::task::JoinHandle;

use crate::config::{HostConfig, SnapshotPolicy};
use crate::state_machine::{Command, RaftStateMachine};
use crate::store::RaftStore;

// --- peer RPC envelopes (the `from` id rides alongside the raft_core message) ---

#[derive(Serialize, Deserialize)]
struct VoteEnvelope {
    from: NodeId,
    req: VoteReq,
}
#[derive(Serialize, Deserialize)]
struct AppendEnvelope {
    from: NodeId,
    req: AppendReq,
}
#[derive(Serialize, Deserialize)]
struct SnapEnvelope {
    from: NodeId,
    req: InstallSnapshotReq,
}
#[derive(Serialize, Deserialize)]
struct NotLeader {
    error: &'static str,
    leader: Option<NodeId>,
}
#[derive(Serialize, Deserialize)]
struct RaftStatus {
    id: NodeId,
    role: String,
    term: u64,
    commit_index: u64,
    last_index: u64,
    snapshot_index: u64,
    applied_index: u64,
    leader: Option<NodeId>,
    is_leader: bool,
}

struct Shared {
    id: NodeId,
    node: Mutex<RaftNode>,
    store: RaftStore,
    sm: Arc<dyn RaftStateMachine>,
    peers: HashMap<NodeId, String>,
    client: reqwest::Client,
    /// Fires (with the SM's applied head) whenever apply advances.
    applied_tx: watch::Sender<Index>,
    cfg: HostConfig,
}

impl Shared {
    fn persist(&self, node: &RaftNode) {
        let _ = self.store.save(&node.persisted());
    }

    /// The single applier. Called under the node lock everywhere committed
    /// entries can appear (tick, inbound append, reply feedback, propose).
    /// Installs any received snapshot, applies newly committed entries to the
    /// state machine in order, bumps `applied_tx`, then maybe compacts.
    fn apply_ready(&self, node: &mut RaftNode) {
        // 1. install a received snapshot first.
        if let Some(bytes) = node.take_installed_snapshot() {
            if let Err(e) = self.sm.restore(&bytes) {
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
            let _ = self.applied_tx.send(self.sm.applied_index());
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
        match self.sm.snapshot() {
            Ok(bytes) => node.compact(applied, bytes),
            Err(e) => tracing::warn!(error = %e, "raft: snapshot capture failed; skip compaction"),
        }
    }

    fn leader_url(&self, node: &RaftNode) -> (Option<NodeId>, Option<String>) {
        let leader = node.leader();
        let url = leader.and_then(|l| self.peers.get(&l).cloned());
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
            let s = Arc::clone(self);
            tokio::spawn(async move { s.send_request(o.to, o.msg).await });
        }
    }

    async fn send_request(self: Arc<Self>, to: NodeId, msg: RaftMsg) {
        let Some(base) = self.peers.get(&to).cloned() else {
            return;
        };
        let reply: Option<RaftMsg> = match msg {
            RaftMsg::Vote(req) => self
                .post(
                    &format!("{base}/raft/request-vote"),
                    &VoteEnvelope { from: self.id, req },
                )
                .await
                .and_then(|r| serde_json::from_slice::<VoteResp>(&r).ok())
                .map(RaftMsg::VoteResp),
            RaftMsg::Append(req) => self
                .post(
                    &format!("{base}/raft/append-entries"),
                    &AppendEnvelope { from: self.id, req },
                )
                .await
                .and_then(|r| serde_json::from_slice::<AppendResp>(&r).ok())
                .map(RaftMsg::AppendResp),
            RaftMsg::InstallSnapshot(req) => self
                .post(
                    &format!("{base}/raft/install-snapshot"),
                    &SnapEnvelope { from: self.id, req },
                )
                .await
                .and_then(|r| serde_json::from_slice::<InstallSnapshotResp>(&r).ok())
                .map(RaftMsg::InstallSnapshotResp),
            _ => None,
        };
        if let Some(reply) = reply {
            let mut n = self.node.lock().await;
            n.handle(to, reply);
            self.persist(&n);
            self.apply_ready(&mut n);
            // Subsequent outbound work is shipped by the pump (no recursive flush).
        }
    }

    async fn post<T: Serialize>(&self, url: &str, body: &T) -> Option<Vec<u8>> {
        match self.client.post(url).json(body).send().await {
            Ok(r) => r.bytes().await.ok().map(|b| b.to_vec()),
            Err(_) => None,
        }
    }

    /// Leader-side propose; returns the index once the **state machine applies**
    /// it (read-your-write). Errors if not leader or apply times out.
    async fn propose_applied(self: &Arc<Self>, command: Command) -> Result<Index> {
        let index = {
            let mut n = self.node.lock().await;
            if !n.is_leader() {
                bail!("raft: not leader");
            }
            let idx = n
                .propose(command)
                .ok_or_else(|| anyhow!("raft: lost leadership during propose"))?;
            self.persist(&n);
            self.apply_ready(&mut n); // sole voter commits+applies here
            idx
        };
        self.flush().await;

        if self.sm.applied_index() >= index {
            return Ok(index);
        }
        let mut rx = self.applied_tx.subscribe();
        let deadline = Instant::now() + self.cfg.propose_timeout;
        loop {
            {
                let mut n = self.node.lock().await;
                self.apply_ready(&mut n);
                if self.sm.applied_index() >= index {
                    return Ok(index);
                }
            }
            tokio::select! {
                _ = rx.changed() => {}
                _ = tokio::time::sleep(Duration::from_millis(5)) => {}
            }
            if Instant::now() >= deadline {
                bail!("raft: apply timeout at index {index}");
            }
        }
    }
}

/// A running raft group host. Cheap to hold; aborts its tasks on drop.
pub struct RaftHost {
    shared: Arc<Shared>,
    tick: JoinHandle<()>,
    pump: JoinHandle<()>,
}

impl Drop for RaftHost {
    fn drop(&mut self) {
        self.tick.abort();
        self.pump.abort();
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
        let mut node = match store.load().ok().flatten() {
            Some(state) => RaftNode::from_persisted(id, &membership, state),
            None => RaftNode::new(id, &membership),
        };
        // Cold-start: drive any committed-but-unapplied resident entries (and a
        // persisted snapshot) into the state machine before serving — inline,
        // before the node enters the Mutex (no async lock available here).
        if let Some(bytes) = node.take_installed_snapshot() {
            if let Err(e) = sm.restore(&bytes) {
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

        let client = h2c::h2c_client_with(Some(cfg.rpc_timeout), None).expect("h2c client");
        let (applied_tx, _rx) = watch::channel(sm.applied_index());
        let shared = Arc::new(Shared {
            id,
            node: Mutex::new(node),
            store,
            sm,
            peers,
            client,
            applied_tx,
            cfg,
        });

        let s = Arc::clone(&shared);
        let tick = tokio::spawn(async move {
            loop {
                tokio::time::sleep(s.cfg.tick).await;
                {
                    let mut n = s.node.lock().await;
                    n.tick();
                    s.persist(&n);
                    s.apply_ready(&mut n);
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
        RaftHost { shared, tick, pump }
    }

    pub async fn is_leader(&self) -> bool {
        self.shared.node.lock().await.is_leader()
    }
    pub async fn leader(&self) -> Option<NodeId> {
        self.shared.node.lock().await.leader()
    }
    /// Watch the state machine's applied head (followers await an index here).
    pub fn applied_watch(&self) -> watch::Receiver<Index> {
        self.shared.applied_tx.subscribe()
    }

    /// Propose on the leader (locally), else forward to the current leader's
    /// `/raft/publish` over h2c. Returns the assigned index once applied
    /// (read-your-write). Retries within the propose deadline while no leader.
    pub async fn propose(&self, command: Command) -> Result<Index> {
        let s = &self.shared;
        let deadline = Instant::now() + s.cfg.propose_timeout;
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
                Route::Local => return s.propose_applied(command).await,
                Route::Remote(url) => {
                    if let Ok(seq) = self.forward(&url, &command).await {
                        return Ok(seq);
                    }
                }
                Route::Unknown => {}
            }
            if Instant::now() >= deadline {
                bail!("raft: no leader elected (cluster not ready)");
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    /// Forward a command to the leader and wait until **this node's** state
    /// machine applies the returned index (read-your-write on a follower).
    async fn forward(&self, leader_url: &str, command: &[u8]) -> Result<Index> {
        let resp = self
            .shared
            .client
            .post(format!("{leader_url}/raft/publish"))
            .body(command.to_vec())
            .send()
            .await?;
        if resp.status() != StatusCode::OK {
            bail!("raft: leader redirect returned {}", resp.status());
        }
        let v: serde_json::Value = resp.json().await?;
        let seq = v
            .get("seq")
            .and_then(|s| s.as_u64())
            .ok_or_else(|| anyhow!("raft: leader reply missing seq"))?;
        // Wait for our own apply (the leader's commit propagates via AppendEntries).
        let mut rx = self.shared.applied_tx.subscribe();
        let deadline = Instant::now() + self.shared.cfg.propose_timeout;
        while self.shared.sm.applied_index() < seq {
            tokio::select! {
                _ = rx.changed() => {}
                _ = tokio::time::sleep(Duration::from_millis(5)) => {}
            }
            if Instant::now() >= deadline {
                bail!("raft: follower apply timeout at index {seq}");
            }
        }
        Ok(seq)
    }

    /// Capture a state-machine snapshot and compact the log up to the applied
    /// index (for `SnapshotPolicy::External` consumers driving their own cadence).
    pub async fn snapshot_and_compact(&self) -> Result<Index> {
        let mut n = self.shared.node.lock().await;
        let applied = self.shared.sm.applied_index();
        if applied == 0 {
            return Ok(0);
        }
        let bytes = self.shared.sm.snapshot()?;
        n.compact(applied, bytes);
        self.shared.persist(&n);
        Ok(applied)
    }

    /// Peer raft RPCs + producer forward + status; merge into the service app so
    /// they ride the h2c serve port.
    pub fn router(&self) -> Router {
        Router::new()
            .route("/raft/request-vote", post(request_vote))
            .route("/raft/append-entries", post(append_entries))
            .route("/raft/install-snapshot", post(install_snapshot))
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

async fn request_vote(
    State(s): State<Arc<Shared>>,
    Json(env): Json<VoteEnvelope>,
) -> Json<VoteResp> {
    let mut n = s.node.lock().await;
    n.handle(env.from, RaftMsg::Vote(env.req));
    s.persist(&n);
    Json(match take_reply(&mut n, env.from) {
        Some(RaftMsg::VoteResp(r)) => r,
        _ => VoteResp {
            term: 0,
            granted: false,
        },
    })
}

async fn append_entries(
    State(s): State<Arc<Shared>>,
    Json(env): Json<AppendEnvelope>,
) -> Json<AppendResp> {
    let mut n = s.node.lock().await;
    n.handle(env.from, RaftMsg::Append(env.req));
    s.persist(&n);
    s.apply_ready(&mut n);
    Json(match take_reply(&mut n, env.from) {
        Some(RaftMsg::AppendResp(r)) => r,
        _ => AppendResp {
            term: 0,
            success: false,
            match_index: 0,
        },
    })
}

async fn install_snapshot(
    State(s): State<Arc<Shared>>,
    Json(env): Json<SnapEnvelope>,
) -> Json<InstallSnapshotResp> {
    let mut n = s.node.lock().await;
    n.handle(env.from, RaftMsg::InstallSnapshot(env.req));
    s.persist(&n);
    s.apply_ready(&mut n); // restores the SM from the installed snapshot
    Json(match take_reply(&mut n, env.from) {
        Some(RaftMsg::InstallSnapshotResp(r)) => r,
        _ => InstallSnapshotResp {
            term: 0,
            snapshot_index: 0,
        },
    })
}

/// Leader-side write target (the redirect destination): propose + apply, return
/// the seq; or `421` with a leader hint if this node is not the leader.
async fn publish_handler(
    State(s): State<Arc<Shared>>,
    body: axum::body::Bytes,
) -> axum::response::Response {
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
    match s.propose_applied(body.to_vec()).await {
        Ok(seq) => (StatusCode::OK, Json(serde_json::json!({ "seq": seq }))).into_response(),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn raftz(State(s): State<Arc<Shared>>) -> Json<RaftStatus> {
    let n = s.node.lock().await;
    Json(RaftStatus {
        id: s.id,
        role: format!("{:?}", n.role()),
        term: n.current_term(),
        commit_index: n.commit_index(),
        last_index: n.last_index(),
        snapshot_index: n.snapshot_index(),
        applied_index: s.sm.applied_index(),
        leader: n.leader(),
        is_leader: n.is_leader(),
    })
}

enum Route {
    Local,
    Remote(String),
    Unknown,
}
