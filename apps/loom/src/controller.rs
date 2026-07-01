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

use crate::model::{
    KeepRef, Node, NodeId, RunStatus, StageId, TaskSpec, WorkflowRun, WorkflowRunId,
};
use crate::runner::RunnerClass;
use crate::scheduler::{dispatch_ready, CompletionMsg, Dispatcher, FanOutSpec, MemDispatcher};
use crate::store::{MemStore, RunStore};

/// Shared control-plane state.
#[derive(Clone)]
pub struct AppState {
    pub store: Arc<dyn RunStore>,
    pub dispatcher: Arc<dyn Dispatcher>,
    /// Liveness/readiness + Prometheus counters behind the standard probe routes.
    pub health: Arc<Health>,
}

/// Control-plane liveness/readiness + lightweight Prometheus counters, shared
/// (`Arc`) between the data-plane handlers and the archetype probe routes
/// (`/readyz` reads `is_draining`; `/metrics` reads the counters).
#[derive(Default)]
pub struct Health {
    draining: std::sync::atomic::AtomicBool,
    runs_submitted: std::sync::atomic::AtomicU64,
    node_completions: std::sync::atomic::AtomicU64,
}

impl Health {
    /// Flip readiness to draining so `/readyz` reports 503 (graceful drain).
    pub fn start_drain(&self) {
        self.draining.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    fn inc_runs(&self) {
        self.runs_submitted.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    fn inc_completions(&self) {
        self.node_completions.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

impl service_http::ReadinessHook for Health {
    fn is_draining(&self) -> bool {
        self.draining.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl service_http::MetricsProvider for Health {
    fn render_metrics(&self) -> String {
        use std::sync::atomic::Ordering::Relaxed;
        let submitted = self.runs_submitted.load(Relaxed);
        let completions = self.node_completions.load(Relaxed);
        format!(
            "# HELP loom_up 1 if the control plane is serving.\n\
             # TYPE loom_up gauge\n\
             loom_up 1\n\
             # HELP loom_runs_submitted_total Runs accepted via POST /runs.\n\
             # TYPE loom_runs_submitted_total counter\n\
             loom_runs_submitted_total {submitted}\n\
             # HELP loom_node_completions_total Node completions folded via the control API.\n\
             # TYPE loom_node_completions_total counter\n\
             loom_node_completions_total {completions}\n"
        )
    }
}

/// One node in a submitted workflow.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
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
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct SubmitRequest {
    pub run_id: String,
    pub nodes: Vec<NodeSpec>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct SubmitResponse {
    pub run_id: String,
    pub status: RunStatus,
    pub node_count: usize,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct NodeView {
    pub id: String,
    pub state: crate::model::NodeState,
    pub attempt: u32,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct RunView {
    pub run_id: String,
    pub status: RunStatus,
    pub nodes: Vec<NodeView>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ApiError {
    error: String,
}

fn bad_request(msg: impl Into<String>) -> (StatusCode, Json<ApiError>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ApiError { error: msg.into() }),
    )
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
                return Err(format!(
                    "node `{}` depends on unknown node `{}`",
                    spec.id, dep
                ));
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
            .map(|n| NodeView {
                id: n.id.0.clone(),
                state: n.state,
                attempt: n.attempt,
            })
            .collect(),
    }
}

/// `POST /runs` — submit a workflow run (a DAG of nodes). Root nodes dispatch
/// immediately; the run advances as node completions arrive.
#[utoipa::path(
    post,
    path = "/runs",
    tag = "Runs",
    request_body = SubmitRequest,
    responses(
        (status = 201, description = "Run accepted; roots dispatched", body = SubmitResponse),
        (status = 400, description = "Invalid DAG (empty, duplicate id, or unknown dep)", body = ApiError),
    )
)]
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
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
            .into_response();
    }
    let resp = SubmitResponse {
        run_id: run.id.0.clone(),
        status: run.status,
        node_count: run.nodes.len(),
    };
    if let Err(e) = state.store.put(run).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
            .into_response();
    }
    state.health.inc_runs();
    (StatusCode::CREATED, Json(resp)).into_response()
}

/// `POST /runs/{id}/nodes/{node}/complete` body: how a node finished. In
/// production a relay ack drives this; the endpoint also lets a test/dev worker
/// report completion directly.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct CompleteRequest {
    /// keep ref to the result payload, if any.
    #[serde(default)]
    pub result_ref: Option<String>,
    /// The attempt this completion is for (#437 dedup). Omit to target the node's
    /// current in-flight attempt (manual completes don't need to track it).
    #[serde(default)]
    pub attempt: Option<u32>,
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
    attempt: u32,
    result_ref: Option<KeepRef>,
    result_inline: Option<Vec<u8>>,
    failed: bool,
    fan_out: &[FanOutSpec],
) -> anyhow::Result<()> {
    // #437 idempotent fold: drop duplicate/stale completions (at-least-once
    // redelivery) so fan-out is never re-spliced and children never re-run.
    if !run.completion_is_current(node, attempt) {
        return Ok(());
    }
    if failed {
        run.mark_failed(node);
    } else {
        run.mark_done_inline(node, result_ref, result_inline);
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

/// `POST /runs/{id}/nodes/{node}/complete` — report a node completion, splice in
/// any runtime fan-out children (#116), and dispatch newly-ready nodes. In
/// production a relay ack drives this; the endpoint also allows manual completion.
#[utoipa::path(
    post,
    path = "/runs/{id}/nodes/{node}/complete",
    tag = "Runs",
    params(
        ("id" = String, Path, description = "Run id"),
        ("node" = String, Path, description = "Node id"),
    ),
    request_body = CompleteRequest,
    responses(
        (status = 200, description = "Updated run view", body = RunView),
        (status = 404, description = "Unknown run", body = ApiError),
    )
)]
async fn complete_node(
    State(state): State<AppState>,
    Path((id, node)): Path<(String, String)>,
    Json(req): Json<CompleteRequest>,
) -> impl IntoResponse {
    let run_id = WorkflowRunId::new(&id);
    let mut run = match state.store.get(&run_id).await {
        Ok(Some(run)) => run,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiError {
                    error: "run not found".into(),
                }),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: e.to_string(),
                }),
            )
                .into_response()
        }
    };
    let result_ref = if req.failed {
        None
    } else {
        req.result_ref.clone().map(KeepRef)
    };
    let node_id = NodeId::new(&node);
    // Manual completes may omit attempt → target the node's current in-flight one.
    let attempt = req
        .attempt
        .unwrap_or_else(|| run.nodes.get(&node_id).map_or(0, |n| n.attempt));
    if let Err(e) = apply_node_completion(
        &mut run,
        state.dispatcher.as_ref(),
        &node_id,
        attempt,
        result_ref,
        None,
        req.failed,
        &req.fan_out,
    )
    .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
            .into_response();
    }
    let v = view(&run);
    if let Err(e) = state.store.put(run).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
            .into_response();
    }
    state.health.inc_completions();
    (StatusCode::OK, Json(v)).into_response()
}

/// `GET /runs/{id}` — poll a run's status and per-node state.
#[utoipa::path(
    get,
    path = "/runs/{id}",
    tag = "Runs",
    params(("id" = String, Path, description = "Run id")),
    responses(
        (status = 200, description = "Run view", body = RunView),
        (status = 404, description = "Unknown run", body = ApiError),
    )
)]
async fn get_run(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.store.get(&WorkflowRunId::new(&id)).await {
        Ok(Some(run)) => (StatusCode::OK, Json(view(&run))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiError {
                error: "run not found".into(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

/// The control-plane data-plane router over a [`RunStore`] — the `/runs` API
/// only. The archetype probe/admin routes are added by [`surface`].
pub fn router(store: Arc<dyn RunStore>, dispatcher: Arc<dyn Dispatcher>, health: Arc<Health>) -> Router {
    Router::new()
        .route("/runs", post(submit))
        .route("/runs/{id}", get(get_run))
        .route("/runs/{id}/nodes/{node}/complete", post(complete_node))
        .with_state(AppState { store, dispatcher, health })
}

/// The full controller HTTP surface: the archetype probe/admin routes
/// (`/healthz` `/readyz` `/metrics` `/openapi.json` `/docs`, via
/// [`service_http::standard_probe_routes`]) merged with the control-plane API.
/// No outer tracing layer — [`run`] adds it after any raft-peer routes merge in.
pub fn surface(store: Arc<dyn RunStore>, dispatcher: Arc<dyn Dispatcher>, health: Arc<Health>) -> Router {
    let metrics: Arc<dyn service_http::MetricsProvider> = health.clone();
    service_http::standard_probe_routes(health.clone(), Some(metrics), openapi)
        .merge(router(store, dispatcher, health))
}

/// The control-plane OpenAPI document — served at `/openapi.json` + `/docs` and
/// emitted offline by `loom spec`. The single generated-doc accessor the probe
/// routes and the `spec` CLI both read.
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi as _;
    ApiDoc::openapi()
}

/// utoipa-generated OpenAPI for loom's control API. Payload bytes never traverse
/// loom (claim-check via keep), so the surface is small JSON control messages.
#[derive(utoipa::OpenApi)]
#[openapi(
    info(
        title = "loom",
        description = "DAG workflow scheduler — the control plane over relay (broker) + keep (store). Small JSON control messages only; payload bytes never traverse loom (claim-check via keep).",
        license(name = "MIT")
    ),
    servers(
        (url = "http://loom-svc:7474", description = "in-cluster ClusterIP"),
        (url = "http://localhost:7474", description = "local dev")
    ),
    tags((name = "Runs", description = "Workflow run submit / status / completion")),
    paths(submit, get_run, complete_node),
    components(schemas(
        SubmitRequest,
        NodeSpec,
        SubmitResponse,
        RunView,
        NodeView,
        CompleteRequest,
        ApiError,
        crate::model::KeepRef,
        crate::model::RunStatus,
        crate::model::NodeState,
        crate::runner::RunnerClass,
        crate::scheduler::FanOutSpec,
    ))
)]
pub struct ApiDoc;

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
        let store: Arc<dyn RunStore> = if raft_host::replica_mode() {
            // k8s scale-out (REPLICAS_PER_SHARD > 1): derive node id / voters /
            // peers from the StatefulSet downward API. `LOOM_PEERS` overrides the
            // peer DNS to run a multi-node group on one host.
            let dir = std::env::var("LOOM_RAFT_DIR").unwrap_or_else(|_| "/data/raft".to_string());
            let topo =
                raft_host::ClusterTopology::from_env("loom", "loom-headless", 7474, "LOOM_PEERS")?;
            eprintln!(
                "loom: raft REPLICA mode — node {}, {} peer(s), dir {dir}",
                topo.node_id,
                topo.peers.len()
            );
            let rs = crate::raft::RaftRunStore::from_topology(topo, &dir)?;
            raft_router = Some(rs.router());
            Arc::new(rs)
        } else if let Ok(peers_env) = std::env::var("LOOM_CLUSTER_PEERS") {
            // Local multi-node testing: an explicit `0=url,1=url,…` peer map (all
            // members incl. self); build the peer map excluding self.
            let id = std::env::var("LOOM_NODE_ID")
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let dir =
                std::env::var("LOOM_RAFT_DIR").unwrap_or_else(|_| format!("/tmp/loom-raft-{id}"));
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
            let rs = crate::raft::RaftRunStore::cluster(id, n_voters, peers, &dir)?;
            raft_router = Some(rs.router());
            Arc::new(rs)
        } else if let Ok(dir) = std::env::var("LOOM_RAFT_DIR") {
            // Single-node durable raft (its own majority) — the archetype default.
            eprintln!("loom: raft-backed durable store (single-node) under {dir}");
            let rs = crate::raft::RaftRunStore::single_node(&dir)?;
            raft_router = Some(rs.router());
            Arc::new(rs)
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
        // Liveness/readiness + metrics carrier, shared with the probe routes.
        let health = Arc::new(Health::default());
        // Archetype surface (probes + /openapi.json + /docs) + control API;
        // raft-peer routes merge in before the outer tracing layer.
        let mut app = surface(store.clone(), dispatcher.clone(), health.clone());
        if let Some(rr) = raft_router {
            app = app.merge(rr);
        }
        let app = app.layer(service_http::trace_layer());
        // Completed-DAG GC (#106): reap terminal runs after a retention window.
        // LOOM_GC_RETENTION_SECS (default 3600); 0 disables.
        let gc_retention = std::env::var("LOOM_GC_RETENTION_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(3600);
        if gc_retention > 0 {
            tokio::spawn(crate::gc::gc_loop(store.clone(), gc_retention));
        }
        // Dispatch deadline (#438): re-dispatch acked-but-silent nodes. Opt-in —
        // set above your longest task (no worker heartbeat yet). 0 = off.
        let dispatch_deadline = std::env::var("LOOM_DISPATCH_DEADLINE_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        if dispatch_deadline > 0 {
            tokio::spawn(crate::deadline::deadline_loop(
                store.clone(),
                dispatcher.clone(),
                dispatch_deadline,
            ));
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
        serve(&addr, app, health).await
    })
}

/// Background loop: consume worker completions from the `loom.completions` relay
/// subject over the **bidi `/consume` stream** (#463) and fold them into the DAG
/// (which dispatches newly-ready nodes). One persistent stream per shard:
/// `Subscribe` up, leased completions down, `Ack` up on the same stream. The
/// stream is the same path the schema layer's task consumer uses (#449), so
/// relay's only consume path is now bidi `/consume`. On a stream drop it
/// reconnects; in-flight (unacked) completions redeliver via relay's lease TTL.
async fn completion_consumer(
    relay_base: String,
    subject: String,
    store: Arc<dyn RunStore>,
    dispatcher: Arc<dyn Dispatcher>,
) {
    use crate::schema_layer::{encode_frame, FrameDecoder};
    use futures::StreamExt;

    let client = match crate::relay_client::relay_http_client() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("loom completion consumer: client init failed: {e}");
            return;
        }
    };
    let url = format!("{relay_base}/v1/{subject}/consume");
    let idle = std::time::Duration::from_millis(200);
    eprintln!("loom: consuming completions from {url} (bidi /consume)");
    loop {
        // One bidi stream: Subscribe up, leased completions down, Ack up. The
        // up-channel is buffered so the Subscribe frame (and later Acks) queue
        // without blocking; dropping `up_tx` at loop end ends the request body.
        let (up_tx, up_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(256);
        if up_tx
            .send(encode_frame(
                &serde_json::json!({ "type": "subscribe", "prefetch": 64 }),
            ))
            .await
            .is_err()
        {
            return;
        }
        let body = reqwest::Body::wrap_stream(async_stream::stream! {
            let mut rx = up_rx;
            while let Some(b) = rx.recv().await { yield Ok::<Vec<u8>, std::io::Error>(b); }
        });
        let resp = match client.post(&url).body(body).send().await {
            Ok(r) => r,
            Err(_) => {
                tokio::time::sleep(idle).await;
                continue;
            }
        };
        let mut down = resp.bytes_stream();
        let mut dec = FrameDecoder::default();
        while let Some(chunk) = down.next().await {
            let Ok(chunk) = chunk else { break };
            dec.push(&chunk);
            while let Some(raw) = dec.next_frame() {
                let Ok(v) = serde_json::from_slice::<serde_json::Value>(&raw) else {
                    continue;
                };
                let (Some(lease_id), Some(epoch), Some(payload)) = (
                    v.get("lease_id").and_then(|x| x.as_str()),
                    v.get("epoch").and_then(|x| x.as_u64()),
                    v.get("payload"),
                ) else {
                    continue;
                };
                if let Ok(cm) = serde_json::from_value::<CompletionMsg>(payload.clone()) {
                    apply_completion_msg(&store, dispatcher.as_ref(), cm).await;
                }
                // Ack on the same stream so relay commits the completion and frees
                // a credit. A send error means the stream is gone → reconnect.
                if up_tx
                    .send(encode_frame(
                        &serde_json::json!({ "type": "ack", "lease_id": lease_id, "epoch": epoch }),
                    ))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        }
        // Stream ended — reconnect; unacked completions redeliver via relay's TTL.
        tokio::time::sleep(idle).await;
    }
}

async fn apply_completion_msg(
    store: &Arc<dyn RunStore>,
    dispatcher: &dyn Dispatcher,
    cm: CompletionMsg,
) {
    let run_id = WorkflowRunId::new(&cm.run_id);
    let Ok(Some(mut run)) = store.get(&run_id).await else {
        return;
    };
    let result_ref = if cm.failed {
        None
    } else {
        cm.result_ref.map(KeepRef)
    };
    let result_inline = if cm.failed { None } else { cm.result_inline };
    if apply_node_completion(
        &mut run,
        dispatcher,
        &NodeId::new(&cm.node_id),
        cm.attempt,
        result_ref,
        result_inline,
        cm.failed,
        &cm.fan_out,
    )
    .await
    .is_ok()
    {
        let _ = store.put(run).await;
    }
}

/// Bind `addr` and serve the controller (HTTP/1.1 + h2c on one port) via the
/// shared [`service_http::serve`] loop, draining gracefully on SIGINT/SIGTERM:
/// `/readyz` flips to 503, then a grace window (`LOOM_DRAIN_GRACE_SECS`, default
/// 10s) elapses before the listener closes so k8s stops routing first.
async fn serve(addr: &str, app: Router, health: Arc<Health>) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!("loom controller listening (h2c) on {addr}");
    let grace = std::time::Duration::from_secs(
        std::env::var("LOOM_DRAIN_GRACE_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10),
    );
    service_http::serve(
        listener,
        app,
        service_http::shutdown_with_drain(move || health.start_drain(), grace),
    )
    .await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use axum::http::Request;
    use tower::ServiceExt;

    fn test_router() -> Router {
        // The full archetype surface (probes + /openapi.json + /docs) merged
        // with the control API, so tests exercise what `run` actually serves.
        surface(
            Arc::new(MemStore::new()),
            Arc::new(MemDispatcher::new()),
            Arc::new(Health::default()),
        )
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

    /// The archetype probe/admin surface (`/readyz`, `/metrics`, `/openapi.json`)
    /// is wired via `service_http::standard_probe_routes` and documents the API.
    #[tokio::test]
    async fn standard_endpoints_served() {
        for (path, want) in [("/readyz", "ok"), ("/metrics", "loom_up 1")] {
            let resp = test_router()
                .oneshot(Request::get(path).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(resp.status(), StatusCode::OK, "{path}");
            let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
            let body = String::from_utf8(bytes.to_vec()).unwrap();
            assert!(body.contains(want), "{path} body missing `{want}`: {body}");
        }
        let resp = test_router()
            .oneshot(Request::get("/openapi.json").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let doc = body_json(resp).await;
        assert_eq!(doc["info"]["title"], "loom");
        assert!(doc["paths"]["/runs"].is_object(), "control API not documented in OpenAPI");
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

    #[tokio::test]
    async fn duplicate_completion_is_idempotent() {
        // #437: a replayed completion (at-least-once redelivery) must be a no-op
        // — no re-spliced fan-out, no reset/re-run of already-progressed children.
        let app = test_router();
        let cpost = |path: &str, body: &'static str| {
            Request::post(path)
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap()
        };
        let fanout = r#"{"fan_out":[{"id":"c0","task_name":"t"},{"id":"c1","task_name":"t"}]}"#;

        app.clone()
            .oneshot(cpost(
                "/runs",
                r#"{"run_id":"idem","nodes":[{"id":"a","task_name":"t"}]}"#,
            ))
            .await
            .unwrap();
        // complete `a` → splices c0,c1
        app.clone()
            .oneshot(cpost("/runs/idem/nodes/a/complete", fanout))
            .await
            .unwrap();
        // complete c0 → c0 Done
        app.clone()
            .oneshot(cpost("/runs/idem/nodes/c0/complete", "{}"))
            .await
            .unwrap();
        // DUPLICATE complete of `a` (same fan-out) — must be ignored by the guard
        let dup = app
            .clone()
            .oneshot(cpost("/runs/idem/nodes/a/complete", fanout))
            .await
            .unwrap();
        assert_eq!(dup.status(), StatusCode::OK);

        let g = app
            .oneshot(Request::get("/runs/idem").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let view = body_json(g).await;
        let nodes = view["nodes"].as_array().unwrap();
        assert_eq!(nodes.len(), 3, "duplicate must not add nodes");
        let c0 = nodes.iter().find(|n| n["id"] == "c0").unwrap();
        assert_eq!(
            c0["state"], "done",
            "duplicate completion must NOT reset/re-run c0"
        );
    }
}
