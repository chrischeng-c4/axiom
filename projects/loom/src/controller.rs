//! `loom controller` — the control plane: a thin HTTP/2 API (#165) over the
//! [`RunStore`], plus (later) the scheduler loop that drives dispatch over
//! relay and folds in completions.
//!
//! API surface (#165): clients submit and query runs here; payload bytes never
//! traverse loom (claim-check via keep). Served h2c (HTTP/2 cleartext) + HTTP/1
//! on one port, like keep/lumen.

use std::collections::BTreeSet;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use axum::routing::{get, post};
use axum::Router;
use serde::{Deserialize, Serialize};

use crate::model::{KeepRef, Node, NodeId, RunStatus, StageId, TaskSpec, WorkflowRun, WorkflowRunId};
use crate::runner::RunnerClass;
use crate::scheduler::{dispatch_ready, CompletionMsg, Dispatcher, FanOutSpec, MemDispatcher};
use crate::store::{MemStore, RunStore};

/// Shared control-plane state.
#[derive(Clone)]
pub struct AppState {
    pub store: Arc<dyn RunStore>,
    pub dispatcher: Arc<dyn Dispatcher>,
}

/// One node in a submitted workflow.
#[derive(Debug, Clone, Deserialize)]
pub struct NodeSpec {
    pub id: String,
    pub task_name: String,
    #[serde(default)]
    pub deps: Vec<String>,
    #[serde(default)]
    pub runner: RunnerClass,
    #[serde(default)]
    pub input_refs: Vec<KeepRef>,
}

/// `POST /runs` body: a client-supplied run id (idempotency key) + the DAG nodes.
#[derive(Debug, Clone, Deserialize)]
pub struct SubmitRequest {
    pub run_id: String,
    pub nodes: Vec<NodeSpec>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubmitResponse {
    pub run_id: String,
    pub status: RunStatus,
    pub node_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct NodeView {
    pub id: String,
    pub state: crate::model::NodeState,
    pub attempt: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunView {
    pub run_id: String,
    pub status: RunStatus,
    pub nodes: Vec<NodeView>,
}

#[derive(Debug, Clone, Serialize)]
struct ApiError {
    error: String,
}

fn bad_request(msg: impl Into<String>) -> (StatusCode, Json<ApiError>) {
    (StatusCode::BAD_REQUEST, Json(ApiError { error: msg.into() }))
}

/// Build a [`WorkflowRun`] from a submit request, validating that every `deps`
/// edge references a node declared in the same request.
fn build_run(req: &SubmitRequest) -> Result<WorkflowRun, String> {
    if req.nodes.is_empty() {
        return Err("a workflow must have at least one node".into());
    }
    let ids: BTreeSet<&str> = req.nodes.iter().map(|n| n.id.as_str()).collect();
    if ids.len() != req.nodes.len() {
        return Err("duplicate node id".into());
    }
    let mut run = WorkflowRun::new(WorkflowRunId::new(&req.run_id));
    for spec in &req.nodes {
        for dep in &spec.deps {
            if !ids.contains(dep.as_str()) {
                return Err(format!("node `{}` depends on unknown node `{}`", spec.id, dep));
            }
        }
        let mut task = TaskSpec::new(&spec.task_name);
        task.runner = spec.runner;
        task.input_refs = spec.input_refs.clone();
        let deps: BTreeSet<NodeId> = spec.deps.iter().map(NodeId::new).collect();
        run.add_node(Node::new(
            NodeId::new(&spec.id),
            StageId::new(&spec.id),
            task,
            deps,
        ));
    }
    Ok(run)
}

fn view(run: &WorkflowRun) -> RunView {
    RunView {
        run_id: run.id.0.clone(),
        status: run.status,
        nodes: run
            .nodes
            .values()
            .map(|n| NodeView { id: n.id.0.clone(), state: n.state, attempt: n.attempt })
            .collect(),
    }
}

async fn healthz() -> &'static str {
    "ok"
}

async fn submit(
    State(state): State<AppState>,
    Json(req): Json<SubmitRequest>,
) -> impl IntoResponse {
    let mut run = match build_run(&req) {
        Ok(run) => run,
        Err(e) => return bad_request(e).into_response(),
    };
    // Dispatch the root nodes immediately (loom → relay); the run advances as
    // completions arrive at `/runs/{id}/nodes/{node}/complete`.
    if let Err(e) = dispatch_ready(&mut run, state.dispatcher.as_ref()).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: e.to_string() }))
            .into_response();
    }
    let resp = SubmitResponse {
        run_id: run.id.0.clone(),
        status: run.status,
        node_count: run.nodes.len(),
    };
    if let Err(e) = state.store.put(run).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: e.to_string() }))
            .into_response();
    }
    (StatusCode::CREATED, Json(resp)).into_response()
}

/// `POST /runs/{id}/nodes/{node}/complete` body: how a node finished. In
/// production a relay ack drives this; the endpoint also lets a test/dev worker
/// report completion directly.
#[derive(Debug, Clone, Deserialize)]
pub struct CompleteRequest {
    /// keep ref to the result payload, if any.
    #[serde(default)]
    pub result_ref: Option<String>,
    /// Set when the attempt failed (triggers retry-or-fail).
    #[serde(default)]
    pub failed: bool,
    /// Runtime fan-out children to splice in after this node (#116).
    #[serde(default)]
    pub fan_out: Vec<FanOutSpec>,
}

/// Mark a node's completion, splice in any runtime fan-out children (#116, the
/// dynamic stage-expand), and dispatch newly-ready nodes.
async fn apply_node_completion(
    run: &mut WorkflowRun,
    dispatcher: &dyn Dispatcher,
    node: &NodeId,
    result_ref: Option<KeepRef>,
    failed: bool,
    fan_out: &[FanOutSpec],
) -> anyhow::Result<()> {
    if failed {
        run.mark_failed(node);
    } else {
        run.mark_done(node, result_ref);
        if !fan_out.is_empty() {
            let children: Vec<Node> = fan_out
                .iter()
                .map(|s| {
                    let mut task = TaskSpec::new(&s.task_name);
                    task.input_refs = s.input_refs.clone();
                    Node::new(
                        NodeId::new(&s.id),
                        StageId::new(format!("dyn:{node}")),
                        task,
                        BTreeSet::new(),
                    )
                })
                .collect();
            run.expand(node, children);
        }
    }
    dispatch_ready(run, dispatcher).await.map(|_| ())
}

async fn complete_node(
    State(state): State<AppState>,
    Path((id, node)): Path<(String, String)>,
    Json(req): Json<CompleteRequest>,
) -> impl IntoResponse {
    let run_id = WorkflowRunId::new(&id);
    let mut run = match state.store.get(&run_id).await {
        Ok(Some(run)) => run,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, Json(ApiError { error: "run not found".into() }))
                .into_response()
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: e.to_string() }))
                .into_response()
        }
    };
    let result_ref = if req.failed { None } else { req.result_ref.clone().map(KeepRef) };
    if let Err(e) = apply_node_completion(
        &mut run,
        state.dispatcher.as_ref(),
        &NodeId::new(&node),
        result_ref,
        req.failed,
        &req.fan_out,
    )
    .await
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: e.to_string() }))
            .into_response();
    }
    let v = view(&run);
    if let Err(e) = state.store.put(run).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: e.to_string() }))
            .into_response();
    }
    (StatusCode::OK, Json(v)).into_response()
}

async fn get_run(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.store.get(&WorkflowRunId::new(&id)).await {
        Ok(Some(run)) => (StatusCode::OK, Json(view(&run))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(ApiError { error: "run not found".into() }))
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: e.to_string() }))
            .into_response(),
    }
}

/// The control-plane router over a [`RunStore`].
pub fn router(store: Arc<dyn RunStore>, dispatcher: Arc<dyn Dispatcher>) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/runs", post(submit))
        .route("/runs/{id}", get(get_run))
        .route("/runs/{id}/nodes/{node}/complete", post(complete_node))
        .with_state(AppState { store, dispatcher })
}

/// Entry point for `loom controller`. Serves the control API h2c on `LOOM_ADDR`
/// (default `0.0.0.0:7474`). The scheduler loop (relay dispatch + completion
/// fold) wires in once relay/keep transport lands.
pub fn run() -> anyhow::Result<()> {
    let addr = std::env::var("LOOM_ADDR").unwrap_or_else(|_| "0.0.0.0:7474".to_string());
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        // Store backend: multi-voter raft CLUSTER (#110 HA, LOOM_CLUSTER_PEERS) >
        // single-voter raft (LOOM_RAFT_DIR) > file crash-recovery (LOOM_DATA_DIR)
        // > in-memory. The cluster store also exposes a raft router peers reach.
        let mut raft_router: Option<Router> = None;
        let store: Arc<dyn RunStore> = if let Ok(peers_env) =
            std::env::var("LOOM_CLUSTER_PEERS")
        {
            let id = std::env::var("LOOM_NODE_ID")
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let dir =
                std::env::var("LOOM_RAFT_DIR").unwrap_or_else(|_| format!("/tmp/loom-raft-{id}"));
            // LOOM_CLUSTER_PEERS = "0=http://h0,1=http://h1,2=http://h2" (all
            // members incl. self); build the peer map excluding self.
            let mut peers = std::collections::HashMap::new();
            for part in peers_env.split(',') {
                if let Some((nid, url)) = part.split_once('=') {
                    if let Ok(nid) = nid.trim().parse::<u64>() {
                        if nid != id {
                            peers.insert(nid, url.trim().to_string());
                        }
                    }
                }
            }
            let n_voters = peers.len() as u64 + 1;
            eprintln!("loom: raft CLUSTER node {id}/{n_voters}, peers {peers:?}, dir {dir}");
            let cs = crate::cluster::RaftClusterStore::spawn(id, n_voters, peers, &dir)?;
            raft_router = Some(cs.router());
            Arc::new(cs)
        } else if let Ok(dir) = std::env::var("LOOM_RAFT_DIR") {
            eprintln!("loom: raft-backed durable store (single-voter) under {dir}");
            Arc::new(crate::raft::RaftRunStore::open(0, &dir)?)
        } else if let Ok(dir) = std::env::var("LOOM_DATA_DIR") {
            eprintln!("loom: persisting runs under {dir}");
            Arc::new(crate::store::FileStore::open(&dir)?)
        } else {
            Arc::new(MemStore::new())
        };
        // Dispatch to a real relay when LOOM_RELAY is set; else an in-memory
        // dispatcher records dispatches (dev/test) without a broker.
        let relay_base = std::env::var("LOOM_RELAY").ok();
        let dispatcher: Arc<dyn Dispatcher> = match &relay_base {
            Some(base) => {
                eprintln!("loom: dispatching to relay at {base}");
                Arc::new(crate::relay_client::RelayDispatcher::new(base.clone())?)
            }
            None => {
                eprintln!("loom: LOOM_RELAY unset — using in-memory dispatcher (no broker)");
                Arc::new(MemDispatcher::new())
            }
        };
        let mut app = router(store.clone(), dispatcher.clone());
        if let Some(rr) = raft_router {
            app = app.merge(rr);
        }
        // With a real relay, consume worker completions and advance the DAG.
        // Run one consumer per shard (#127): completions are published to
        // `loom.completions.{shard_of(run_id)}`, so per-run folding stays serial
        // (no race) while distinct runs fold in parallel. LOOM_COMPLETION_SHARDS
        // must match the workers' sink (same default).
        if let Some(base) = relay_base {
            let shards = std::env::var("LOOM_COMPLETION_SHARDS")
                .ok()
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(8)
                .max(1);
            for k in 0..shards {
                let subject = if shards <= 1 {
                    "loom.completions".to_string()
                } else {
                    format!("loom.completions.{k}")
                };
                tokio::spawn(completion_consumer(
                    base.clone(),
                    subject,
                    store.clone(),
                    dispatcher.clone(),
                ));
            }
        }
        serve(&addr, app).await
    })
}

/// Background loop: lease worker completions from the `loom.completions` relay
/// subject and fold them into the DAG (which dispatches newly-ready nodes).
async fn completion_consumer(
    relay_base: String,
    subject: String,
    store: Arc<dyn RunStore>,
    dispatcher: Arc<dyn Dispatcher>,
) {
    let client = match reqwest::Client::builder().http2_prior_knowledge().build() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("loom completion consumer: client init failed: {e}");
            return;
        }
    };
    let lease_url = format!("{relay_base}/v1/{subject}/lease");
    let ack_url = format!("{relay_base}/v1/{subject}/ack");
    let consumer_id = format!("loom-controller-{subject}");
    let idle = std::time::Duration::from_millis(200);
    eprintln!("loom: consuming completions from {lease_url}");
    loop {
        let leased = client
            .post(&lease_url)
            .json(&serde_json::json!({ "consumer_id": consumer_id }))
            .send()
            .await;
        let body: serde_json::Value = match leased {
            Ok(r) => r.json().await.unwrap_or(serde_json::Value::Null),
            Err(_) => {
                tokio::time::sleep(idle).await;
                continue;
            }
        };
        let lease = body.get("lease").filter(|v| !v.is_null());
        let entry = body.get("entry").filter(|v| !v.is_null());
        let (Some(lease), Some(entry)) = (lease, entry) else {
            tokio::time::sleep(idle).await;
            continue;
        };
        let lease_id = lease.get("lease_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let epoch = lease.get("epoch").and_then(|v| v.as_u64()).unwrap_or(0);
        let payload = entry.get("payload").cloned().unwrap_or(serde_json::Value::Null);
        if let Ok(cm) = serde_json::from_value::<CompletionMsg>(payload) {
            apply_completion_msg(&store, dispatcher.as_ref(), cm).await;
        }
        let _ = client
            .post(&ack_url)
            .json(&serde_json::json!({ "lease_id": lease_id, "epoch": epoch }))
            .send()
            .await;
    }
}

async fn apply_completion_msg(store: &Arc<dyn RunStore>, dispatcher: &dyn Dispatcher, cm: CompletionMsg) {
    let run_id = WorkflowRunId::new(&cm.run_id);
    let Ok(Some(mut run)) = store.get(&run_id).await else {
        return;
    };
    let result_ref = if cm.failed { None } else { cm.result_ref.map(KeepRef) };
    if apply_node_completion(
        &mut run,
        dispatcher,
        &NodeId::new(&cm.node_id),
        result_ref,
        cm.failed,
        &cm.fan_out,
    )
    .await
    .is_ok()
    {
        let _ = store.put(run).await;
    }
}

async fn serve(addr: &str, app: Router) -> anyhow::Result<()> {
    use hyper_util::rt::{TokioExecutor, TokioIo};
    use hyper_util::server::conn::auto;
    use tokio::net::TcpListener;
    use tower::ServiceExt;

    let listener = TcpListener::bind(addr).await?;
    eprintln!("loom controller listening (h2c) on {addr}");
    let mut builder = auto::Builder::new(TokioExecutor::new());
    builder.http2().max_concurrent_streams(4096);
    loop {
        let (stream, _peer) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let app = app.clone();
        let svc = hyper::service::service_fn(move |req| app.clone().oneshot(req));
        let conn = builder.serve_connection_with_upgrades(io, svc).into_owned();
        tokio::spawn(async move {
            let _ = conn.await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use axum::http::Request;
    use tower::ServiceExt;

    fn test_router() -> Router {
        router(Arc::new(MemStore::new()), Arc::new(MemDispatcher::new()))
    }

    async fn body_json(resp: axum::response::Response) -> serde_json::Value {
        let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn healthz_ok() {
        let resp = test_router()
            .oneshot(Request::get("/healthz").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn submit_then_query_roundtrip() {
        let app = test_router();
        let req = Request::post("/runs")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"run_id":"r1","nodes":[{"id":"a","task_name":"t"},{"id":"b","task_name":"t","deps":["a"]}]}"#,
            ))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let body = body_json(resp).await;
        assert_eq!(body["run_id"], "r1");
        assert_eq!(body["node_count"], 2);

        let resp = app
            .oneshot(Request::get("/runs/r1").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["run_id"], "r1");
        assert_eq!(body["nodes"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn submit_rejects_unknown_dep() {
        let req = Request::post("/runs")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"run_id":"r2","nodes":[{"id":"a","task_name":"t","deps":["ghost"]}]}"#,
            ))
            .unwrap();
        let resp = test_router().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn missing_run_is_404() {
        let resp = test_router()
            .oneshot(Request::get("/runs/nope").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    /// End-to-end through the API with an in-process dispatcher: submit a chain
    /// a→b, complete each node, and watch loom drive the run to `succeeded`.
    #[tokio::test]
    async fn drives_dag_to_completion_via_api() {
        let app = test_router();

        let submit = app
            .clone()
            .oneshot(
                Request::post("/runs")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"run_id":"e2e","nodes":[{"id":"a","task_name":"t"},{"id":"b","task_name":"t","deps":["a"]}]}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(submit.status(), StatusCode::CREATED);
        // root `a` dispatched on submit → the run is running.
        assert_eq!(body_json(submit).await["status"], "running");

        // complete `a` → `b` becomes ready and is dispatched.
        let r = app
            .clone()
            .oneshot(
                Request::post("/runs/e2e/nodes/a/complete")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"result_ref":"k/a"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);

        // complete `b` → the whole run succeeds.
        let r = app
            .oneshot(
                Request::post("/runs/e2e/nodes/b/complete")
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(body_json(r).await["status"], "succeeded");
    }
}
