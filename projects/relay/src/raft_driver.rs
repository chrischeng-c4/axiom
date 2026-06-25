// SPEC-MANAGED: projects/relay/tech-design/logic/h2c-raft-transport-driver-producer-redirect-to-leader.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:1fbb47da" tracker="pending-tracker" reason="RaftDriver: Arc<RaftShared{ async-mutex RaftNode, RaftStore, peers, reqwest h2c client, Arc<Relay>, subject, id }>. spawn() runs a tick task; helpers persist-before-flush, apply committed commands to the Relay, and flush the outbox by POSTing RequestVote/AppendEntries to peers and feeding responses back. handle_request_vote/handle_append_entries return the node's reply. propose(command) on the leader. leader_hint(). A raft_router(driver) exposes POST /raft/request-vote, POST /raft/append-entries, POST /v1/{subject}/publish (redirect-to-leader else propose), GET /raftz, GET /healthz."
//! Production driver that runs the [`RaftNode`] core over HTTP/2.
//!
//! A [`RaftDriver`] owns the node (behind an async mutex) together with its
//! [`RaftStore`], peer URLs, an h2c client and the local [`Relay`]. A tick task
//! advances the node on a timer; after every mutation the driver **persists
//! before flushing the outbox** (so no vote/ack leaves before it is durable),
//! then applies newly committed commands to the Relay. Outbound RequestVote /
//! AppendEntries are POSTed to peers and their responses fed back into the node.
//! Producers that hit a non-leader get a `NotLeader` hint; the leader proposes
//! the publish through Raft.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::engine::Relay;
use crate::raft::{
    AppendReq, AppendResp, Membership, NodeId, RaftMsg, RaftNode, VoteReq, VoteResp,
};
use crate::raft_store::RaftStore;

const TICK: Duration = Duration::from_millis(20);
/// Fast loop that ships outbound messages and picks up work produced by
/// responses (kept well under the election timeout so a dead peer never starves
/// heartbeats).
const PUMP: Duration = Duration::from_millis(5);
const RPC_TIMEOUT: Duration = Duration::from_millis(400);

/// A publish replicated through Raft (the command bytes of a log entry).
#[derive(Serialize, Deserialize)]
pub struct PubCommand {
    pub message_id: String,
    pub payload: serde_json::Value,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
}

/// Body of POST /raft/request-vote.
#[derive(Serialize, Deserialize)]
pub struct VoteEnvelope {
    pub from: NodeId,
    pub req: VoteReq,
}

/// Body of POST /raft/append-entries.
#[derive(Serialize, Deserialize)]
pub struct AppendEnvelope {
    pub from: NodeId,
    pub req: AppendReq,
}

/// POST /v1/{subject}/publish body.
#[derive(Serialize, Deserialize)]
pub struct PublishBody {
    pub message_id: String,
    pub payload: serde_json::Value,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
}

/// Reply when a producer hit a non-leader.
#[derive(Serialize, Deserialize)]
pub struct NotLeader {
    pub error: &'static str,
    pub leader: Option<NodeId>,
    pub leader_url: Option<String>,
}

/// /raftz status.
#[derive(Serialize, Deserialize)]
pub struct RaftStatus {
    pub id: NodeId,
    pub role: String,
    pub term: u64,
    pub commit_index: u64,
    pub last_index: u64,
    pub leader: Option<NodeId>,
    pub is_leader: bool,
}

struct Shared {
    id: NodeId,
    subject: String,
    node: Mutex<RaftNode>,
    store: RaftStore,
    peers: HashMap<NodeId, String>,
    client: reqwest::Client,
    relay: Arc<Relay>,
}

impl Shared {
    /// Persist the node's hard state (best-effort; called while holding the lock,
    /// before any reply/heartbeat is flushed).
    fn persist(&self, node: &RaftNode) {
        let _ = self.store.save(&node.persisted());
    }

    /// Apply newly committed commands to the local Relay (idempotent publish).
    fn apply_committed(&self, node: &mut RaftNode) {
        for e in node.take_committed() {
            if let Ok(cmd) = serde_json::from_slice::<PubCommand>(&e.command) {
                let _ = self.relay.publish(
                    &self.subject,
                    &cmd.message_id,
                    cmd.payload,
                    cmd.headers,
                    Utc::now(),
                );
            }
        }
    }

    /// Drain the outbox and deliver each request to its peer over h2c — each in
    /// its own task (fire-and-forget). A response feeds back into the node and
    /// triggers another flush, so progress is event-driven and a dead/slow peer
    /// never stalls heartbeats or elections (no barrier on the slowest peer).
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
    /// Replies (VoteResp/AppendResp) are returned by inbound handlers, never
    /// sent here.
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
            self.apply_committed(&mut n);
            // Any new work the response produced (became leader -> heartbeats;
            // commit advanced -> propagate) is picked up by the pump loop — no
            // recursive flush here (which would make this future un-sendable).
        }
    }
}

/// Drives a single-shard Raft group and serves it over h2c.
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
    /// start its tick task.
    pub fn spawn(
        id: NodeId,
        membership: Membership,
        subject: impl Into<String>,
        peers: HashMap<NodeId, String>,
        relay: Arc<Relay>,
        store: RaftStore,
    ) -> RaftDriver {
        let node = match store.load().ok().flatten() {
            Some(state) => RaftNode::from_persisted(id, &membership, state),
            None => RaftNode::new(id, &membership),
        };
        // Shared h2c client construction (#468): equivalent to the inline
        // `http2_prior_knowledge().timeout(RPC_TIMEOUT)` builder, via h2c.
        let client = h2c::h2c_client_with(Some(RPC_TIMEOUT), None).expect("reqwest client");
        let shared = Arc::new(Shared {
            id,
            subject: subject.into(),
            node: Mutex::new(node),
            store,
            peers,
            client,
            relay,
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
        RaftDriver { shared, tick, pump }
    }

    pub fn relay(&self) -> Arc<Relay> {
        Arc::clone(&self.shared.relay)
    }

    pub async fn is_leader(&self) -> bool {
        self.shared.node.lock().await.is_leader()
    }

    pub async fn leader(&self) -> Option<NodeId> {
        self.shared.node.lock().await.leader()
    }

    pub fn stop(&self) {
        self.tick.abort();
        self.pump.abort();
    }

    /// Router exposing the Raft RPCs, producer publish, and status.
    pub fn router(&self) -> Router {
        Router::new()
            .route("/raft/request-vote", post(request_vote))
            .route("/raft/append-entries", post(append_entries))
            .route("/v1/{subject}/publish", post(publish))
            .route("/raftz", get(raftz))
            .route("/healthz", get(|| async { "ok" }))
            .with_state(Arc::clone(&self.shared))
    }
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
    let resp = match resp {
        Some(RaftMsg::VoteResp(r)) => r,
        _ => VoteResp {
            term: 0,
            granted: false,
        },
    };
    Json(resp)
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
    let resp = match resp {
        Some(RaftMsg::AppendResp(r)) => r,
        _ => AppendResp {
            term: 0,
            success: false,
            match_index: 0,
        },
    };
    Json(resp)
}

/// Pull the single reply addressed to `to` out of the node's outbox (the
/// inbound RPC handler returns it as the HTTP response).
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

async fn publish(
    State(s): State<Arc<Shared>>,
    Path(_subject): Path<String>,
    Json(body): Json<PublishBody>,
) -> axum::response::Response {
    let index = {
        let mut n = s.node.lock().await;
        if !n.is_leader() {
            let leader = n.leader();
            let leader_url = leader.and_then(|l| s.peers.get(&l).cloned());
            drop(n);
            return (
                StatusCode::MISDIRECTED_REQUEST,
                Json(NotLeader {
                    error: "not-leader",
                    leader,
                    leader_url,
                }),
            )
                .into_response();
        }
        let cmd = serde_json::to_vec(&PubCommand {
            message_id: body.message_id,
            payload: body.payload,
            headers: body.headers,
        })
        .unwrap_or_default();
        let idx = n.propose(cmd);
        s.persist(&n);
        s.apply_committed(&mut n);
        idx
    };
    // Kick replication (fire-and-forget) and wait for the entry to commit+apply.
    s.flush().await;
    let Some(index) = index else {
        return (
            StatusCode::OK,
            Json(serde_json::json!({ "committed": true })),
        )
            .into_response();
    };
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let committed = {
            let mut n = s.node.lock().await;
            s.apply_committed(&mut n);
            n.commit_index() >= index
        };
        if committed {
            return (
                StatusCode::OK,
                Json(serde_json::json!({ "committed": true, "index": index })),
            )
                .into_response();
        }
        if Instant::now() >= deadline {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "committed": false, "index": index })),
            )
                .into_response();
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
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
// HANDWRITE-END
