//! Multi-group Raft registry.
//!
//! Exposes multiple independently durable consensus groups behind a single `/raft/*`
//! HTTP/2 listener, routing each incoming RPC to its designated group by `group_id`.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::group::GroupId;
use crate::host::{
    append_entries, host_status, install_snapshot, publish_handler, request_vote, timeout_now,
    AppendEnvelope, PublishEnvelope, RaftHost, RaftStatus, SnapEnvelope, TimeoutNowEnvelope,
    VoteEnvelope,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    AlreadyRegistered(GroupId),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::AlreadyRegistered(gid) => {
                write!(f, "group {:?} is already registered", gid.0)
            }
        }
    }
}

impl std::error::Error for RegistryError {}

struct RegistryShared {
    groups: Mutex<HashMap<GroupId, Arc<RaftHost>>>,
}

/// Registry holding multiple [`RaftHost`]s multiplexed across a single `/raft/*` router.
#[derive(Clone, Default)]
pub struct RaftRegistry {
    shared: Arc<RegistryShared>,
}

pub type GroupRegistry = RaftRegistry;

impl Default for RegistryShared {
    fn default() -> Self {
        Self {
            groups: Mutex::new(HashMap::new()),
        }
    }
}

impl RaftRegistry {
    /// Create a new empty multi-group registry.
    pub fn new() -> Self {
        Self {
            shared: Arc::new(RegistryShared::default()),
        }
    }

    /// Register a host under its configured `group_id`.
    ///
    /// Returns `Err(RegistryError::AlreadyRegistered)` if the group ID is already registered,
    /// keeping the previously registered host serving and unmodified.
    pub fn register(&self, host: impl Into<Arc<RaftHost>>) -> Result<(), RegistryError> {
        let host = host.into();
        let gid = host.group_id().clone();
        let mut groups = self.shared.groups.lock().unwrap();
        if groups.contains_key(&gid) {
            return Err(RegistryError::AlreadyRegistered(gid));
        }
        groups.insert(gid, host);
        Ok(())
    }

    /// Look up a registered host by `group_id`.
    pub fn get(&self, group_id: &GroupId) -> Option<Arc<RaftHost>> {
        let groups = self.shared.groups.lock().unwrap();
        groups.get(group_id).cloned()
    }

    /// Build the unified `/raft/*` router for all registered groups.
    ///
    /// Routes incoming RPCs to the matching group by `group_id`.
    /// An unknown or unregistered group returns `404 Not Found`.
    pub fn router(&self) -> Router {
        Router::new()
            .route("/raft/request-vote", post(request_vote_demux))
            .route("/raft/append-entries", post(append_entries_demux))
            .route("/raft/install-snapshot", post(install_snapshot_demux))
            .route("/raft/timeout-now", post(timeout_now_demux))
            .route(RaftHost::PUBLISH_PATH, post(publish_demux))
            .route("/raftz", get(raftz_demux))
            .with_state(Arc::clone(&self.shared))
    }
}

async fn request_vote_demux(
    State(reg): State<Arc<RegistryShared>>,
    Json(env): Json<VoteEnvelope>,
) -> axum::response::Response {
    let host = {
        let groups = reg.groups.lock().unwrap();
        groups.get(&GroupId(env.group_id.clone())).cloned()
    };
    match host {
        Some(h) => request_vote(State(Arc::clone(&h.shared)), Json(env)).await,
        None => (StatusCode::NOT_FOUND, "group not found").into_response(),
    }
}

async fn append_entries_demux(
    State(reg): State<Arc<RegistryShared>>,
    Json(env): Json<AppendEnvelope>,
) -> axum::response::Response {
    let host = {
        let groups = reg.groups.lock().unwrap();
        groups.get(&GroupId(env.group_id.clone())).cloned()
    };
    match host {
        Some(h) => append_entries(State(Arc::clone(&h.shared)), Json(env)).await,
        None => (StatusCode::NOT_FOUND, "group not found").into_response(),
    }
}

async fn install_snapshot_demux(
    State(reg): State<Arc<RegistryShared>>,
    Json(env): Json<SnapEnvelope>,
) -> axum::response::Response {
    let host = {
        let groups = reg.groups.lock().unwrap();
        groups.get(&GroupId(env.group_id.clone())).cloned()
    };
    match host {
        Some(h) => install_snapshot(State(Arc::clone(&h.shared)), Json(env)).await,
        None => (StatusCode::NOT_FOUND, "group not found").into_response(),
    }
}

async fn timeout_now_demux(
    State(reg): State<Arc<RegistryShared>>,
    Json(env): Json<TimeoutNowEnvelope>,
) -> axum::response::Response {
    let host = {
        let groups = reg.groups.lock().unwrap();
        groups.get(&GroupId(env.group_id.clone())).cloned()
    };
    match host {
        Some(h) => timeout_now(State(Arc::clone(&h.shared)), Json(env)).await,
        None => (StatusCode::NOT_FOUND, "group not found").into_response(),
    }
}

/// The group is the routing key, so an envelope the demux cannot read cannot
/// be routed to a host that would judge leadership before the body; that
/// ordering starts once a group's [`publish_handler`] is reached.
async fn publish_demux(
    State(reg): State<Arc<RegistryShared>>,
    body: axum::body::Bytes,
) -> axum::response::Response {
    let Ok(env) = serde_json::from_slice::<PublishEnvelope>(&body) else {
        return (StatusCode::BAD_REQUEST, "invalid publish envelope").into_response();
    };
    let host = {
        let groups = reg.groups.lock().unwrap();
        groups.get(&GroupId(env.group_id)).cloned()
    };
    match host {
        Some(h) => publish_handler(State(Arc::clone(&h.shared)), body).await,
        None => (StatusCode::NOT_FOUND, "group not found").into_response(),
    }
}

async fn raftz_demux(State(reg): State<Arc<RegistryShared>>) -> Json<BTreeMap<String, RaftStatus>> {
    let hosts: Vec<Arc<RaftHost>> = {
        let groups = reg.groups.lock().unwrap();
        groups.values().cloned().collect()
    };
    let mut map = BTreeMap::new();
    for h in hosts {
        let s = host_status(&h.shared).await;
        map.insert(h.shared.group_id.0.clone(), s);
    }
    Json(map)
}
