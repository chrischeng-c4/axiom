//! Production driver that runs the shared [`RaftNode`] core for lumen.
//!
//! Mirrors relay's `raft_driver` **structurally** but with one critical
//! difference: it does **not** apply committed entries to the engine. lumen
//! already has exactly one applier — the [`crate::coordinator::WriteCoordinator`]
//! apply loop tailing the WAL — so this driver instead *surfaces* committed
//! entries as `(index, WalRecord)` into a buffer that [`crate::wal_raft::RaftWal`]
//! exposes through the `WalLog` seam. The raft log index **is** the WAL seq
//! (both 1-based), so no offset bridging is needed.
//!
//! Slice 2 (this file) is multi-pod: a tick task advances the node and a fast
//! pump ships the outbox to peers over h2c; inbound Vote/Append RPCs are served
//! by `router()`. A write to a follower is forwarded to the leader via
//! `POST /raft/publish` (leader redirect lives below the `WalLog` seam, so the
//! `WriteCoordinator` never sees it).

use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Result};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tokio::sync::{watch, Mutex};
use tokio::task::JoinHandle;

use crate::raft_core::{
    AppendReq, AppendResp, Membership, NodeId, RaftMsg, RaftNode, VoteReq, VoteResp,
};
use crate::raft_store::RaftStore;
use crate::wal::WalRecord;

/// Logical tick interval (a sole voter elects after ~`ELECTION_MIN` ticks).
const TICK: Duration = Duration::from_millis(20);
/// Fast loop that ships outbound messages and picks up work produced by
/// responses (kept well under the election timeout so a dead peer never starves
/// heartbeats).
const PUMP: Duration = Duration::from_millis(5);
const RPC_TIMEOUT: Duration = Duration::from_millis(400);

/// Committed records surfaced from the raft log, index-ordered (1-based,
/// contiguous): position `i` holds the entry with index `i + 1`.
type Committed = Arc<StdMutex<Vec<(u64, WalRecord)>>>;

/// Body of `POST /raft/request-vote`.
#[derive(Serialize, Deserialize)]
struct VoteEnvelope {
    from: NodeId,
    req: VoteReq,
}

/// Body of `POST /raft/append-entries`.
#[derive(Serialize, Deserialize)]
struct AppendEnvelope {
    from: NodeId,
    req: AppendReq,
}

/// Reply when a write hit a non-leader.
#[derive(Serialize, Deserialize)]
struct NotLeader {
    error: &'static str,
    leader: Option<NodeId>,
}

/// `/raftz` status.
#[derive(Serialize, Deserialize)]
struct RaftStatus {
    id: NodeId,
    role: String,
    term: u64,
    commit_index: u64,
    last_index: u64,
    leader: Option<NodeId>,
    is_leader: bool,
}

struct Shared {
    id: NodeId,
    node: Mutex<RaftNode>,
    store: RaftStore,
    committed: Committed,
    commit_tx: watch::Sender<u64>,
    /// Peer base URLs (NodeId → `http://host:port`), excluding self.
    peers: HashMap<NodeId, String>,
    client: reqwest::Client,
}

impl Shared {
    /// Persist the node's hard state (best-effort; called while holding the lock,
    /// before any reply/heartbeat is flushed).
    fn persist(&self, node: &RaftNode) {
        let _ = self.store.save(&node.persisted());
    }

    /// Drain newly-committed entries into the surfaced buffer and bump the watch
    /// so subscribers wake. Called while holding the node lock, so `take_committed`
    /// (which advances `last_applied`) and the buffer push stay strictly ordered.
    fn surface(&self, node: &mut RaftNode) {
        let entries = node.take_committed();
        if entries.is_empty() {
            return;
        }
        let mut last = 0;
        {
            let mut buf = self
                .committed
                .lock()
                .expect("raft committed buffer poisoned");
            for e in entries {
                match WalRecord::decode(&e.command) {
                    Ok(rec) => {
                        buf.push((e.index, rec));
                        last = e.index;
                    }
                    Err(err) => {
                        tracing::error!(index = e.index, error = %err, "raft: undecodable committed entry");
                    }
                }
            }
        }
        if last > 0 {
            let _ = self.commit_tx.send(last);
        }
    }

    /// `(leader, base_url)` if the current leader is a known peer.
    fn leader_url(&self, node: &RaftNode) -> (Option<NodeId>, Option<String>) {
        let leader = node.leader();
        let url = leader.and_then(|l| self.peers.get(&l).cloned());
        (leader, url)
    }

    /// Drain the outbox and deliver each request to its peer over h2c — each in
    /// its own task (fire-and-forget). A response feeds back into the node and
    /// triggers progress via the pump, so a dead/slow peer never stalls
    /// heartbeats or elections.
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

    /// Deliver one outbound request to its peer and feed the response back.
    /// Replies (VoteResp/AppendResp) are returned by inbound handlers, never here.
    async fn send_request(self: Arc<Self>, to: NodeId, msg: RaftMsg) {
        let Some(base) = self.peers.get(&to).cloned() else {
            return;
        };
        let reply: Option<RaftMsg> = match msg {
            RaftMsg::Vote(req) => {
                let env = VoteEnvelope { from: self.id, req };
                match self
                    .client
                    .post(format!("{base}/raft/request-vote"))
                    .json(&env)
                    .send()
                    .await
                {
                    Ok(r) => r.json::<VoteResp>().await.ok().map(RaftMsg::VoteResp),
                    Err(_) => None,
                }
            }
            RaftMsg::Append(req) => {
                let env = AppendEnvelope { from: self.id, req };
                match self
                    .client
                    .post(format!("{base}/raft/append-entries"))
                    .json(&env)
                    .send()
                    .await
                {
                    Ok(r) => r.json::<AppendResp>().await.ok().map(RaftMsg::AppendResp),
                    Err(_) => None,
                }
            }
            _ => None,
        };
        if let Some(reply) = reply {
            let mut n = self.node.lock().await;
            n.handle(to, reply);
            self.persist(&n);
            self.surface(&mut n);
            // New work (became leader → heartbeats; commit advanced) is picked up
            // by the pump — no recursive flush (which would make this un-sendable).
        }
    }

    /// Leader-side: propose `cmd`, kick replication, and return its seq once
    /// committed. Errors if leadership is lost or commit times out.
    async fn propose_committed(self: &Arc<Self>, cmd: Vec<u8>) -> Result<u64> {
        let index = {
            let mut n = self.node.lock().await;
            if !n.is_leader() {
                bail!("raft: not leader");
            }
            let idx = n
                .propose(cmd)
                .ok_or_else(|| anyhow!("raft: lost leadership during propose"))?;
            self.persist(&n);
            self.surface(&mut n);
            idx
        };
        // Kick replication immediately; the pump keeps it moving.
        self.flush().await;

        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            {
                let mut n = self.node.lock().await;
                self.surface(&mut n);
                if n.commit_index() >= index {
                    return Ok(index);
                }
            }
            if Instant::now() >= deadline {
                bail!("raft: commit timeout at index {index}");
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }
}

/// Drives a single-shard Raft group for lumen and surfaces its committed log to
/// the WAL seam.
pub struct RaftDriver {
    shared: Arc<Shared>,
    tick: JoinHandle<()>,
    pump: JoinHandle<()>,
}

impl Drop for RaftDriver {
    fn drop(&mut self) {
        self.tick.abort();
        self.pump.abort();
    }
}

impl RaftDriver {
    /// Build a driver for node `id`, recovering persisted state if present, and
    /// start its tick + pump tasks. `peers` maps the other group members to their
    /// base URLs (empty ⇒ single-node).
    pub fn spawn(
        id: NodeId,
        membership: Membership,
        peers: HashMap<NodeId, String>,
        store: RaftStore,
    ) -> RaftDriver {
        let node = match store.load().ok().flatten() {
            Some(state) => RaftNode::from_persisted(id, &membership, state),
            None => RaftNode::new(id, &membership),
        };
        let client = h2c::h2c_client_with(Some(RPC_TIMEOUT), None).expect("h2c client");
        let (commit_tx, _rx) = watch::channel(0u64);
        let shared = Arc::new(Shared {
            id,
            node: Mutex::new(node),
            store,
            committed: Arc::new(StdMutex::new(Vec::new())),
            commit_tx,
            peers,
            client,
        });
        let s = Arc::clone(&shared);
        let tick = tokio::spawn(async move {
            loop {
                tokio::time::sleep(TICK).await;
                {
                    let mut n = s.node.lock().await;
                    n.tick();
                    s.persist(&n);
                    s.surface(&mut n);
                }
                s.flush().await;
            }
        });
        let p = Arc::clone(&shared);
        let pump = tokio::spawn(async move {
            loop {
                tokio::time::sleep(PUMP).await;
                p.flush().await;
            }
        });
        RaftDriver { shared, tick, pump }
    }

    /// Shared handle to the surfaced committed log (for `RaftWal::subscribe`).
    pub fn committed(&self) -> Committed {
        Arc::clone(&self.shared.committed)
    }

    /// A receiver that fires when the committed head advances.
    pub fn commit_watch(&self) -> watch::Receiver<u64> {
        self.shared.commit_tx.subscribe()
    }

    /// Highest committed (surfaced) seq, or 0.
    pub fn latest_committed(&self) -> u64 {
        self.shared
            .committed
            .lock()
            .ok()
            .and_then(|b| b.last().map(|(i, _)| *i))
            .unwrap_or(0)
    }

    pub async fn is_leader(&self) -> bool {
        self.shared.node.lock().await.is_leader()
    }

    pub async fn leader(&self) -> Option<NodeId> {
        self.shared.node.lock().await.leader()
    }

    /// The write path: propose on the leader (locally), else forward the command
    /// to the current leader's `POST /raft/publish` over h2c. Retries within a
    /// deadline while the leader is unknown (election in progress).
    pub async fn publish(&self, cmd: Vec<u8>) -> Result<u64> {
        let s = &self.shared;
        let deadline = Instant::now() + Duration::from_secs(10);
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
                Route::Local => return s.propose_committed(cmd).await,
                Route::Remote(url) => match self.forward(&url, &cmd).await {
                    Ok(seq) => return Ok(seq),
                    Err(_) => {} // leader stale / unreachable → retry
                },
                Route::Unknown => {}
            }
            if Instant::now() >= deadline {
                bail!("raft: no leader elected (cluster not ready)");
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    /// Forward a command to the leader's `/raft/publish` and return the seq.
    async fn forward(&self, leader_url: &str, cmd: &[u8]) -> Result<u64> {
        let resp = self
            .shared
            .client
            .post(format!("{leader_url}/raft/publish"))
            .body(cmd.to_vec())
            .send()
            .await?;
        if resp.status() != StatusCode::OK {
            bail!("raft: leader redirect returned {}", resp.status());
        }
        let v: serde_json::Value = resp.json().await?;
        v.get("seq")
            .and_then(|s| s.as_u64())
            .ok_or_else(|| anyhow!("raft: leader reply missing seq"))
    }

    /// Router for the peer raft RPCs + producer forward + status. Merge this into
    /// the service app so it shares the h2c `serve` port.
    pub fn router(&self) -> Router {
        Router::new()
            .route("/raft/request-vote", post(request_vote))
            .route("/raft/append-entries", post(append_entries))
            .route("/raft/publish", post(publish_handler))
            .route("/raftz", get(raftz))
            .with_state(Arc::clone(&self.shared))
    }
}

/// Pull the single reply addressed to `to` out of the node's outbox (the inbound
/// RPC handler returns it as the HTTP response).
fn take_reply(node: &mut RaftNode, to: NodeId) -> Option<RaftMsg> {
    let mut reply = None;
    for o in node.take_outgoing() {
        if o.to == to
            && reply.is_none()
            && matches!(o.msg, RaftMsg::VoteResp(_) | RaftMsg::AppendResp(_))
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
    let resp = {
        let mut n = s.node.lock().await;
        n.handle(env.from, RaftMsg::Vote(env.req));
        s.persist(&n);
        take_reply(&mut n, env.from)
    };
    Json(match resp {
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
    let resp = {
        let mut n = s.node.lock().await;
        n.handle(env.from, RaftMsg::Append(env.req));
        s.persist(&n);
        s.surface(&mut n);
        take_reply(&mut n, env.from)
    };
    Json(match resp {
        Some(RaftMsg::AppendResp(r)) => r,
        _ => AppendResp {
            term: 0,
            success: false,
            match_index: 0,
        },
    })
}

/// Leader-side write target: propose the forwarded command and return its seq,
/// or `421 Misdirected` with a leader hint if this node is not the leader.
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
    match s.propose_committed(body.to_vec()).await {
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
        leader: n.leader(),
        is_leader: n.is_leader(),
    })
}

enum Route {
    Local,
    Remote(String),
    Unknown,
}
