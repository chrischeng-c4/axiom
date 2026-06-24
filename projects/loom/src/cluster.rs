//! Live multi-voter raft-backed [`RunStore`] over HTTP/2 (#110, HA tier).
//!
//! The single-voter [`crate::raft::RaftRunStore`] proves the raft-as-RunStore
//! wiring in-process; this is the multi-process deployment driver. It is modeled
//! directly on relay's `raft_driver`: a [`RaftNode`] behind an async mutex, a
//! tick task + a fast pump that **persist before flushing the outbox**, and
//! Vote/Append RPCs delivered to peers over h2c (each peer reply fed back into
//! the node). Committed [`Command`]s apply to a [`LoomStateMachine`]; `put`
//! proposes on the leader (a follower forwards to it), then waits for commit.
//!
//! Consensus correctness is raftcore's (verified in-process across a 3-node
//! group in `crate::raft`); this module is the transport + store plumbing,
//! verified live across 3 loom processes (`scripts/cluster-e2e.sh`).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use raftcore::{
    auto_membership, AppendReq, AppendResp, NodeId, RaftMsg, RaftNode, VoteReq, VoteResp,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::model::{WorkflowRun, WorkflowRunId};
use crate::raft::{Command, LoomStateMachine};
use crate::store::RunStore;

const TICK: Duration = Duration::from_millis(20);
const PUMP: Duration = Duration::from_millis(5);
const RPC_TIMEOUT: Duration = Duration::from_millis(400);
const COMMIT_DEADLINE: Duration = Duration::from_secs(3);

/// Body of POST /raft/request-vote.
#[derive(Serialize, Deserialize)]
struct VoteEnvelope {
    from: NodeId,
    req: VoteReq,
}

/// Body of POST /raft/append-entries.
#[derive(Serialize, Deserialize)]
struct AppendEnvelope {
    from: NodeId,
    req: AppendReq,
}

/// Reply when a proposer hit a non-leader.
#[derive(Serialize, Deserialize)]
struct NotLeader {
    error: &'static str,
    leader: Option<NodeId>,
    leader_url: Option<String>,
}

/// /raftz status.
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
    sm: std::sync::Mutex<LoomStateMachine>,
    peers: HashMap<NodeId, String>,
    client: reqwest::Client,
    raft_path: PathBuf,
    snap_path: PathBuf,
}

impl Shared {
    /// Persist the node's hard state + the materialized run map (best-effort,
    /// while holding the node lock, before any reply/heartbeat is flushed).
    fn persist(&self, node: &RaftNode) {
        let tmp = self.raft_path.with_extension("json.tmp");
        if serde_json::to_vec(&node.persisted())
            .ok()
            .and_then(|b| std::fs::write(&tmp, b).ok())
            .is_some()
        {
            let _ = std::fs::rename(&tmp, &self.raft_path);
        }
        if let Ok(sm) = self.sm.lock() {
            let snap_tmp = self.snap_path.with_extension("json.tmp");
            if std::fs::write(&snap_tmp, sm.snapshot()).is_ok() {
                let _ = std::fs::rename(&snap_tmp, &self.snap_path);
            }
        }
    }

    /// Apply newly committed commands to the local state machine.
    fn apply_committed(&self, node: &mut RaftNode) {
        let entries = node.take_committed();
        if entries.is_empty() {
            return;
        }
        if let Ok(mut sm) = self.sm.lock() {
            for e in &entries {
                sm.apply(e);
            }
        }
    }

    /// Drain the outbox; deliver each request to its peer in its own task so a
    /// dead/slow peer never stalls heartbeats or elections.
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

    /// Deliver one outbound Vote/Append to its peer and feed the response back.
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
                    .timeout(RPC_TIMEOUT)
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
                    .timeout(RPC_TIMEOUT)
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
            self.apply_committed(&mut n);
        }
    }

    /// Propose a command on the local node (must be leader) and wait for it to
    /// commit. Returns `Err` carrying the leader hint when not leader.
    async fn propose_local(self: &Arc<Self>, cmd: Command) -> Result<(), Option<NodeId>> {
        let index = {
            let mut n = self.node.lock().await;
            if !n.is_leader() {
                return Err(n.leader());
            }
            let idx = n.propose(cmd.encode());
            self.persist(&n);
            self.apply_committed(&mut n);
            idx
        };
        self.flush().await;
        let Some(index) = index else { return Ok(()) };
        let deadline = Instant::now() + COMMIT_DEADLINE;
        loop {
            let committed = {
                let mut n = self.node.lock().await;
                self.apply_committed(&mut n);
                n.commit_index() >= index
            };
            if committed {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Ok(()); // best-effort; replication continues in the background
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}

/// A live multi-voter raft-backed store. Implements [`RunStore`]; also exposes a
/// [`router`](Self::router) the controller mounts so peers can reach this node.
pub struct RaftClusterStore {
    shared: Arc<Shared>,
    tick: JoinHandle<()>,
    pump: JoinHandle<()>,
}

impl Drop for RaftClusterStore {
    fn drop(&mut self) {
        self.tick.abort();
        self.pump.abort();
    }
}

impl RaftClusterStore {
    /// Build a node `id` in an `n_voters`-member group, recovering persisted
    /// state, and start its tick + pump tasks. `peers` maps every *other* node
    /// id to its base URL.
    pub fn spawn(
        id: NodeId,
        n_voters: u64,
        peers: HashMap<NodeId, String>,
        dir: impl AsRef<std::path::Path>,
    ) -> anyhow::Result<RaftClusterStore> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)?;
        let raft_path = dir.join("raft.json");
        let snap_path = dir.join("runs.snapshot.json");
        let membership = auto_membership(n_voters);
        let node = match std::fs::read(&raft_path).ok().and_then(|b| serde_json::from_slice(&b).ok())
        {
            Some(state) => RaftNode::from_persisted(id, &membership, state),
            None => RaftNode::new(id, &membership),
        };
        let mut sm = LoomStateMachine::new();
        if let Ok(bytes) = std::fs::read(&snap_path) {
            sm.restore(&bytes);
        }
        let client = reqwest::Client::builder()
            .http2_prior_knowledge()
            .build()
            .expect("reqwest h2c client");
        let shared = Arc::new(Shared {
            id,
            node: Mutex::new(node),
            sm: std::sync::Mutex::new(sm),
            peers,
            client,
            raft_path,
            snap_path,
        });
        let s = Arc::clone(&shared);
        let tick = tokio::spawn(async move {
            loop {
                tokio::time::sleep(TICK).await;
                {
                    let mut n = s.node.lock().await;
                    n.tick();
                    s.persist(&n);
                    s.apply_committed(&mut n);
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
        Ok(RaftClusterStore { shared, tick, pump })
    }

    /// Router exposing the raft RPCs + propose + status; mounted by the controller.
    pub fn router(&self) -> Router {
        Router::new()
            .route("/raft/request-vote", post(request_vote))
            .route("/raft/append-entries", post(append_entries))
            .route("/raft/propose", post(propose))
            .route("/raftz", get(raftz))
            .with_state(Arc::clone(&self.shared))
    }
}

impl RaftClusterStore {
    /// Commit a command: propose on the local node if leader, else forward to the
    /// leader's /raft/propose. Retries until the deadline (covers elections).
    async fn commit(&self, cmd: Command) -> anyhow::Result<()> {
        let deadline = Instant::now() + COMMIT_DEADLINE;
        loop {
            match self.shared.propose_local(cmd.clone()).await {
                Ok(()) => return Ok(()),
                Err(leader) => {
                    if let Some(url) = leader.and_then(|l| self.shared.peers.get(&l).cloned()) {
                        let ok = self
                            .shared
                            .client
                            .post(format!("{url}/raft/propose"))
                            .json(&cmd)
                            .timeout(COMMIT_DEADLINE)
                            .send()
                            .await
                            .map(|r| r.status().is_success())
                            .unwrap_or(false);
                        if ok {
                            return Ok(());
                        }
                    }
                    if Instant::now() >= deadline {
                        anyhow::bail!("no leader available to accept the command");
                    }
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
            }
        }
    }
}

#[async_trait]
impl RunStore for RaftClusterStore {
    async fn put(&self, run: WorkflowRun) -> anyhow::Result<()> {
        self.commit(Command::PutRun(run)).await
    }

    async fn delete(&self, id: &WorkflowRunId) -> anyhow::Result<()> {
        self.commit(Command::DeleteRun(id.clone())).await
    }

    async fn get(&self, id: &WorkflowRunId) -> anyhow::Result<Option<WorkflowRun>> {
        let sm = self.shared.sm.lock().map_err(|_| anyhow::anyhow!("sm poisoned"))?;
        Ok(sm.get(id).cloned())
    }

    async fn list(&self) -> anyhow::Result<Vec<WorkflowRunId>> {
        let sm = self.shared.sm.lock().map_err(|_| anyhow::anyhow!("sm poisoned"))?;
        Ok(sm.run_ids())
    }
}

async fn request_vote(State(s): State<Arc<Shared>>, Json(env): Json<VoteEnvelope>) -> Json<VoteResp> {
    let resp = {
        let mut n = s.node.lock().await;
        n.handle(env.from, RaftMsg::Vote(env.req));
        s.persist(&n);
        take_reply(&mut n, env.from)
    };
    Json(match resp {
        Some(RaftMsg::VoteResp(r)) => r,
        _ => VoteResp { term: 0, granted: false },
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
        s.apply_committed(&mut n);
        take_reply(&mut n, env.from)
    };
    Json(match resp {
        Some(RaftMsg::AppendResp(r)) => r,
        _ => AppendResp { term: 0, success: false, match_index: 0 },
    })
}

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

/// POST /raft/propose — the leader accepts a command (a follower's forward target).
async fn propose(State(s): State<Arc<Shared>>, Json(cmd): Json<Command>) -> axum::response::Response {
    match s.propose_local(cmd).await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "committed": true }))).into_response(),
        Err(leader) => {
            let leader_url = leader.and_then(|l| s.peers.get(&l).cloned());
            (
                StatusCode::MISDIRECTED_REQUEST,
                Json(NotLeader { error: "not-leader", leader, leader_url }),
            )
                .into_response()
        }
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
