//! axum HTTP application over the tape journal.
//!
//! `append` / `replay` / `checkpoint_get` / `checkpoint_put` are thin
//! handlers wrapping the unchanged [`crate::TapeJournal`] API — no new
//! domain behavior lives here. The operational surface is the shared
//! service shell: the standard probe routes (`/healthz` `/readyz`
//! `/metrics` `/openapi.json` `/docs`) come from
//! `service_http::standard_probe_routes` merged with the `/topics` data
//! plane; error responses render the shared `{error, message}` envelope
//! ([`service_http::ApiErr`]); per-op request metrics are recorded by
//! [`crate::metrics::track`] on the data plane.
//!
//! Request auth is the shared `libs/service-auth` bearer contract (#1326):
//! the blanket `service_auth::auth_middleware` runs on the `/topics` data
//! plane ONLY (probes stay tokenless), injecting an [`AuditedRoleMapPrincipal`] each
//! handler authorizes on its `{topic}` via [`crate::auth::authorize`] —
//! `append` = write, `replay`/`checkpoint_get`/`checkpoint_put` = read.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use axum::extract::{DefaultBodyLimit, Extension, Path, Query, State};
use axum::http::{header, Method, StatusCode};
use axum::middleware::{from_fn, from_fn_with_state, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use service_auth::{AuditedRoleMapPrincipal, ReloadableRoleMapVerifier, Role};
use service_http::{ApiErr, MetricsProvider};
use utoipa::ToSchema;

use crate::metrics::TapeMetrics;
use crate::raft::{TapeOutcome, TapeRaft};
use crate::{
    ConsumerCheckpoint, PullSubscriptionBatch, RetentionPolicy, Subscription, SubscriptionAckError,
    SubscriptionError, TapeError, TapeEvent, TapeJournal,
};

/// Data-plane request body cap. `libs/service-http::HttpConfig` documents
/// `body_limit_bytes` as the max request body size (bytes) for the data
/// plane, with no exported default constant (`HttpConfig::new` takes it as a
/// required, service-supplied argument) and no size limit on the probe
/// routes; this mirrors the 8 MiB literal `HttpConfig`'s own tests use as
/// that field's value (`libs/service-http/src/config.rs`), the same
/// local-constant pattern `apps/keep/src/http/mod.rs::DEFAULT_BODY_LIMIT`
/// uses for its own data plane. Probes are exempt (unbounded), matching
/// `service_http`'s documented probe behavior.
const DEFAULT_BODY_LIMIT_BYTES: usize = 8 * 1024 * 1024;

/// Shared application state: the journal (behind a `std::sync::Mutex` — an
/// in-memory `BTreeMap` core with no async internal awaits), the per-op
/// request metrics, the drain flag `/readyz` reports, the optional file the
/// journal persists to on every mutation (`--store`, mirroring the CLI's
/// `load_journal`/`save_journal`), the bearer verifier the data-plane auth
/// layer runs (#1326), and the optional raft group (#1327) that replicates
/// append/checkpoint-put in HA (`REPLICAS_PER_SHARD > 1`) mode. `raft` stays
/// `None` in single-node serving — the direct-journal path below is
/// unchanged.
#[derive(Clone)]
pub struct AppState {
    journal: Arc<Mutex<TapeJournal>>,
    metrics: Arc<TapeMetrics>,
    draining: Arc<AtomicBool>,
    store: Option<PathBuf>,
    verifier: Arc<ReloadableRoleMapVerifier>,
    raft: Option<Arc<TapeRaft>>,
}

impl AppState {
    /// Build state from an already-loaded journal (empty when no `--store`
    /// file exists yet, mirroring the CLI's `load_journal`). Auth is open
    /// (tokenless — the `TAPE_AUTH=off` default); production serving builds
    /// through [`AppState::with_auth`].
    pub fn new(journal: TapeJournal, store: Option<PathBuf>) -> Self {
        Self {
            journal: Arc::new(Mutex::new(journal)),
            metrics: Arc::new(TapeMetrics::new()),
            draining: Arc::new(AtomicBool::new(false)),
            store,
            verifier: Arc::new(ReloadableRoleMapVerifier::open()),
            raft: None,
        }
    }

    /// Build state with a resolved auth config (`--auth` /
    /// `--token-registry-file`): the data-plane auth layer runs the registry
    /// verifier when auth is required, the open verifier when off.
    pub fn with_auth(
        journal: TapeJournal,
        store: Option<PathBuf>,
        auth: crate::auth::AuthConfig,
    ) -> Self {
        let mut state = Self::new(journal, store);
        state.verifier = Arc::new(auth.verifier());
        state
    }

    /// The bearer verifier the data-plane auth middleware runs.
    pub fn verifier(&self) -> Arc<ReloadableRoleMapVerifier> {
        Arc::clone(&self.verifier)
    }

    /// The per-op request metrics `/metrics` renders.
    pub fn metrics(&self) -> Arc<TapeMetrics> {
        Arc::clone(&self.metrics)
    }

    /// The shared journal handle, for wiring a [`crate::raft::TapeRaft`]
    /// group onto the SAME journal this state serves reads from (#1327).
    pub fn journal_handle(&self) -> Arc<Mutex<TapeJournal>> {
        Arc::clone(&self.journal)
    }

    /// Attach the raft group (auto-mode HA serve path, #1327). Once set,
    /// `append`/`checkpoint_put` propose through it instead of mutating the
    /// journal directly.
    pub fn set_raft(&mut self, raft: Arc<TapeRaft>) {
        self.raft = Some(raft);
    }

    /// The raft group this state proposes through, when running in HA mode.
    pub fn raft(&self) -> Option<Arc<TapeRaft>> {
        self.raft.clone()
    }

    /// Flip readiness to draining so `/readyz` returns 503. Called on
    /// SIGTERM via `service_http::shutdown_with_drain`.
    pub fn start_drain(&self) {
        self.draining.store(true, Ordering::SeqCst);
    }

    pub fn is_draining(&self) -> bool {
        self.draining.load(Ordering::SeqCst)
    }

    /// Persist the journal to `--store`, when configured. Best-effort: a
    /// write failure is logged and surfaced as a 500, mirroring the CLI's
    /// `save_journal`.
    fn persist(&self, journal: &TapeJournal) -> std::io::Result<()> {
        let Some(path) = &self.store else {
            return Ok(());
        };
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec_pretty(journal)?;
        std::fs::write(path, bytes)
    }
}

/// Readiness source for the shared probe router: `/readyz` reports 503 once
/// SIGTERM flips `start_drain`.
impl service_http::ReadinessHook for AppState {
    fn is_draining(&self) -> bool {
        self.draining.load(Ordering::SeqCst)
    }
}

/// Prometheus exposition for the shared `/metrics` route: tape's per-op
/// request counts + latency.
impl service_http::MetricsProvider for AppState {
    fn render_metrics(&self) -> String {
        self.metrics.render()
    }
}

// <HANDWRITE gap="missing-generator:public-peer-route-isolation" tracker="#1805" reason="public-peer-route-isolation section in server.rs is hand-written pending codegen support">
/// Build the HTTP router for the tape transport: the `/topics` data plane
/// merged onto the shared service shell's standard probe routes.
pub fn router(state: AppState) -> Router {
    router_with_admission(state, None)
}

/// Build the public router for a deployment whose Raft peer routes are owned
/// by the dedicated mTLS listener. The underlying application composition is
/// deliberately unchanged so data, probes, auth, admission, and metrics stay
/// identical; a first middleware rejects the two peer route families before
/// route dispatch can expose them on the public h2c listener.
pub fn router_without_raft_routes(state: AppState) -> Router {
    router_without_raft_routes_with_admission(state, None)
}

/// Build the secure-peer public router with optional shared request admission.
/// Peer route isolation stays outermost, so the public listener rejects Raft
/// routes before admission can account for them.
pub fn router_without_raft_routes_with_admission(
    state: AppState,
    admission: Option<service_http::AdmissionController>,
) -> Router {
    router_with_admission(state, admission).layer(from_fn(reject_public_raft_routes))
}

async fn reject_public_raft_routes(request: axum::extract::Request, next: Next) -> Response {
    let path = request.uri().path();
    if path == "/raftz" || path.starts_with("/raft/") {
        StatusCode::NOT_FOUND.into_response()
    } else {
        next.run(request).await
    }
}
// </HANDWRITE>

/// Build Tape with optional shared request admission. Tape owns the
/// read/write/admin route classes; `service-http` owns opaque-key retention,
/// token buckets, eviction, observability, and the 429 wire response.
pub fn router_with_admission(
    state: AppState,
    admission: Option<service_http::AdmissionController>,
) -> Router {
    let req_metrics = state.metrics();
    let verifier = state.verifier();
    let raft = state.raft();
    let data_plane = Router::new()
        .route("/topics/{topic}/append", axum::routing::post(append))
        .route("/topics/{topic}/replay", get(replay))
        .route("/topics/{topic}/replay/stream", get(replay_stream))
        .route(
            "/topics/{topic}/consumers/{consumer}/checkpoint",
            get(checkpoint_get).put(checkpoint_put),
        )
        .route(
            "/topics/{topic}/subscriptions",
            get(subscription_list).post(subscription_create),
        )
        .route(
            "/topics/{topic}/subscriptions/{name}",
            get(subscription_get).delete(subscription_delete),
        )
        .route(
            "/topics/{topic}/subscriptions/{name}/pull",
            axum::routing::post(subscription_pull),
        )
        .route(
            "/topics/{topic}/subscriptions/{name}/ack",
            axum::routing::post(subscription_ack),
        )
        .route(
            "/topics/{topic}/retention",
            get(retention_get).put(retention_put),
        )
        // Cluster-wide admin op (#1329): a consistent snapshot of the journal
        // for backup runners. Inside the auth layer (unlike probes) — needs
        // `admin` on `*`.
        .route("/admin/backup", get(admin_backup))
        // Shared bearer auth (#1326) on the data plane ONLY — probes stay
        // tokenless. The blanket middleware authenticates (401 on a
        // missing/unknown token when required) and injects the
        // AuditedRoleMapPrincipal each handler authorizes on its {topic}.
        .route_layer(from_fn_with_state(
            verifier,
            service_auth::auth_middleware::<ReloadableRoleMapVerifier>,
        ))
        // Per-op request metrics (counts + latency). route_layer => only for
        // matched data-plane routes, and MatchedPath is populated. Added
        // after (= outside) the auth layer so rejected requests are still
        // counted.
        .route_layer(from_fn_with_state(req_metrics, crate::metrics::track))
        .with_state(state.clone())
        // Data-plane-only request body cap (#2484); probes below stay
        // unbounded, matching `service_http`'s documented probe behavior.
        .layer(DefaultBodyLimit::max(DEFAULT_BODY_LIMIT_BYTES));
    let data_plane = match admission {
        Some(controller) => data_plane.route_layer(from_fn_with_state(
            service_http::AdmissionMiddleware::new(controller, |request| {
                let path = request.uri().path();
                let class = if path.starts_with("/admin/") {
                    "tape.admin"
                } else if *request.method() == Method::GET {
                    "tape.read"
                } else {
                    "tape.write"
                };
                let key = request
                    .headers()
                    .get(header::AUTHORIZATION)
                    .map(|value| value.as_bytes())
                    .unwrap_or(b"anonymous");
                Some(service_http::AdmissionInput::new(class, key))
            }),
            service_http::admission_middleware,
        )),
        None => data_plane,
    };

    // Standard probes (`/healthz`, `/readyz`, `/metrics`, `/openapi.json`,
    // `/docs`) come from the shared service shell so the operational
    // surface matches every other service in the ecosystem. AppState
    // supplies readiness + Prometheus metrics; `/readyz` reports 503 while
    // draining.
    let probe_state = Arc::new(state);
    let metrics: Arc<dyn MetricsProvider> = probe_state.clone();
    let probes =
        service_http::standard_probe_routes(probe_state, Some(metrics), crate::openapi::openapi);

    let app = probes
        .merge(data_plane)
        // One INFO-level tracing span per request — spans probes + data plane.
        .layer(service_http::trace_layer());

    // Peer raft RPCs + leader forward + `/raftz` (#1327) — merged OUTSIDE the
    // bearer-auth data plane, like the probes, since this is cluster traffic
    // between tape nodes rather than a client-facing route.
    match raft {
        Some(raft) => app.merge(raft.router()),
        None => app,
    }
}

/// Request body for `POST /topics/{topic}/append`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct AppendRequest {
    /// Optional partitioning/idempotency key carried in the event envelope.
    #[serde(default)]
    pub key: Option<String>,
    /// Event payload.
    pub payload: serde_json::Value,
    /// Override event timestamp for deterministic tests/backfill.
    #[serde(default)]
    pub timestamp_ms: Option<u64>,
}

/// Query params for `GET /topics/{topic}/replay`.
#[derive(Debug, Deserialize)]
pub struct ReplayQuery {
    #[serde(default)]
    pub from_offset: Option<u64>,
    #[serde(default)]
    pub from_timestamp_ms: Option<u64>,
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Response body for `GET /topics/{topic}/replay`.
#[derive(Debug, Serialize, ToSchema)]
pub struct ReplayResponse {
    pub events: Vec<TapeEvent>,
}

/// Response body for `GET /topics/{topic}/consumers/{consumer}/checkpoint`.
#[derive(Debug, Serialize, ToSchema)]
pub struct CheckpointResponse {
    pub checkpoint: Option<ConsumerCheckpoint>,
}

/// Request body for `PUT /topics/{topic}/consumers/{consumer}/checkpoint`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CheckpointPutRequest {
    pub offset: u64,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SubscriptionCreateRequest {
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SubscriptionListResponse {
    pub subscriptions: Vec<Subscription>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SubscriptionPullRequest {
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SubscriptionAckRequest {
    pub offset: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RetentionGetResponse {
    pub policy: Option<RetentionPolicy>,
}

/// `POST /topics/{topic}/append` — append one event envelope to the topic
/// journal.
#[utoipa::path(
    post,
    path = "/topics/{topic}/append",
    params(("topic" = String, Path, description = "Topic name")),
    request_body = AppendRequest,
    responses((status = 200, description = "The appended event", body = TapeEvent))
)]
pub async fn append(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path(topic): Path<String>,
    body: axum::body::Bytes,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &topic, Role::Write) {
        return deny.into_response();
    }
    let req: AppendRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => {
            return ApiErr::new(StatusCode::BAD_REQUEST, "bad_request", e.to_string())
                .into_response()
        }
    };
    // Resolve the timestamp BEFORE touching raft so every replica applies the
    // identical value (#1327) — same rule the direct-journal path already
    // follows via `TapeJournal::append`'s own `Option<u64>` -> `now_ms()`
    // fallback, just hoisted here so the proposed command carries it.
    let timestamp_ms = req.timestamp_ms.unwrap_or_else(crate::now_ms);

    if let Some(raft) = st.raft() {
        // append is NOT idempotent (unlike a message_id-keyed publish), so an
        // aged-out or failed outcome cannot be safely recomputed locally —
        // surface 503 rather than silently re-appending a possible duplicate.
        return match raft
            .propose_append(topic, req.key, req.payload, timestamp_ms)
            .await
        {
            Ok((_, Some(TapeOutcome::Appended(event)))) => {
                (StatusCode::OK, Json(event)).into_response()
            }
            Ok((_, Some(_))) => ApiErr::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal",
                "raft outcome kind mismatch for append",
            )
            .into_response(),
            Ok((_, None)) => ApiErr::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "raft_unavailable",
                "append outcome aged out before this node could read it back",
            )
            .into_response(),
            Err(e) => ApiErr::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "raft_unavailable",
                e.to_string(),
            )
            .into_response(),
        };
    }

    let mut journal = st.journal.lock().expect("journal mutex poisoned");
    let event = journal.append(topic, req.key, req.payload, Some(timestamp_ms));
    if let Err(e) = st.persist(&journal) {
        return ApiErr::new(StatusCode::INTERNAL_SERVER_ERROR, "internal", e.to_string())
            .into_response();
    }
    (StatusCode::OK, Json(event)).into_response()
}

/// `GET /topics/{topic}/replay` — replay topic history by offset or
/// timestamp.
#[utoipa::path(
    get,
    path = "/topics/{topic}/replay",
    params(
        ("topic" = String, Path, description = "Topic name"),
        ("from_offset" = Option<u64>, Query, description = "First offset to include"),
        ("from_timestamp_ms" = Option<u64>, Query, description = "First event timestamp to include"),
        ("limit" = Option<usize>, Query, description = "Maximum number of events to return"),
    ),
    responses((status = 200, description = "Matching events", body = ReplayResponse))
)]
pub async fn replay(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path(topic): Path<String>,
    Query(q): Query<ReplayQuery>,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &topic, Role::Read) {
        return deny.into_response();
    }
    let journal = st.journal.lock().expect("journal mutex poisoned");
    let events = journal.replay(&topic, q.from_offset, q.from_timestamp_ms, q.limit);
    (StatusCode::OK, Json(ReplayResponse { events })).into_response()
}

/// `GET /topics/{topic}/replay/stream` — compact read-only h2c bulk replay.
/// The topic is carried by the path once; each frame retains offset, event
/// time, optional key, and opaque JSON payload bytes.
#[utoipa::path(
    get,
    path = "/topics/{topic}/replay/stream",
    params(
        ("topic" = String, Path, description = "Topic name"),
        ("from_offset" = Option<u64>, Query, description = "First offset to include"),
        ("from_timestamp_ms" = Option<u64>, Query, description = "First event timestamp to include"),
        ("limit" = Option<usize>, Query, description = "Maximum number of events to return"),
    ),
    responses((status = 200, description = "Length-framed Tape replay stream", content_type = "application/vnd.tape.replay.v1", body = Vec<u8>))
)]
pub async fn replay_stream(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path(topic): Path<String>,
    Query(q): Query<ReplayQuery>,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &topic, Role::Read) {
        return deny.into_response();
    }
    let encoded = {
        let journal = st.journal.lock().expect("journal mutex poisoned");
        let events = journal.replay_refs(&topic, q.from_offset, q.from_timestamp_ms, q.limit);
        crate::replay_wire::encode(&events)
    };
    match encoded {
        Ok(body) => (
            [
                (header::CONTENT_TYPE, crate::replay_wire::CONTENT_TYPE),
                (header::CACHE_CONTROL, "no-store"),
            ],
            body,
        )
            .into_response(),
        Err(error) => ApiErr::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            error.to_string(),
        )
        .into_response(),
    }
}

/// `GET /topics/{topic}/consumers/{consumer}/checkpoint` — read a consumer
/// checkpoint.
#[utoipa::path(
    get,
    path = "/topics/{topic}/consumers/{consumer}/checkpoint",
    params(
        ("topic" = String, Path, description = "Topic name"),
        ("consumer" = String, Path, description = "Consumer name"),
    ),
    responses((status = 200, description = "The consumer's checkpoint, if any", body = CheckpointResponse))
)]
pub async fn checkpoint_get(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path((topic, consumer)): Path<(String, String)>,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &topic, Role::Read) {
        return deny.into_response();
    }
    let journal = st.journal.lock().expect("journal mutex poisoned");
    let checkpoint = journal.checkpoint(&topic, &consumer).cloned();
    (StatusCode::OK, Json(CheckpointResponse { checkpoint })).into_response()
}

/// `PUT /topics/{topic}/consumers/{consumer}/checkpoint` — advance a
/// consumer checkpoint.
#[utoipa::path(
    put,
    path = "/topics/{topic}/consumers/{consumer}/checkpoint",
    params(
        ("topic" = String, Path, description = "Topic name"),
        ("consumer" = String, Path, description = "Consumer name"),
    ),
    request_body = CheckpointPutRequest,
    responses(
        (status = 200, description = "The advanced checkpoint", body = ConsumerCheckpoint),
        (status = 409, description = "Stale or beyond-end checkpoint offset")
    )
)]
pub async fn checkpoint_put(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path((topic, consumer)): Path<(String, String)>,
    body: axum::body::Bytes,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &topic, Role::Read) {
        return deny.into_response();
    }
    let req: CheckpointPutRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => {
            return ApiErr::new(StatusCode::BAD_REQUEST, "bad_request", e.to_string())
                .into_response()
        }
    };

    if let Some(raft) = st.raft() {
        let updated_at_ms = crate::now_ms();
        return match raft
            .propose_checkpoint(topic, consumer, req.offset, updated_at_ms)
            .await
        {
            Ok((_, Some(TapeOutcome::Checkpoint(Ok(checkpoint))))) => {
                (StatusCode::OK, Json(checkpoint)).into_response()
            }
            Ok((_, Some(TapeOutcome::Checkpoint(Err(e @ TapeError::StaleCheckpoint { .. }))))) => {
                ApiErr::new(StatusCode::CONFLICT, "conflict", e.to_string()).into_response()
            }
            Ok((
                _,
                Some(TapeOutcome::Checkpoint(Err(e @ TapeError::CheckpointBeyondEnd { .. }))),
            )) => ApiErr::new(StatusCode::CONFLICT, "conflict", e.to_string()).into_response(),
            Ok((_, Some(_))) => ApiErr::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal",
                "raft outcome kind mismatch for checkpoint_put",
            )
            .into_response(),
            Ok((_, None)) => ApiErr::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "raft_unavailable",
                "checkpoint outcome aged out before this node could read it back",
            )
            .into_response(),
            Err(e) => ApiErr::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "raft_unavailable",
                e.to_string(),
            )
            .into_response(),
        };
    }

    let mut journal = st.journal.lock().expect("journal mutex poisoned");
    match journal.put_checkpoint(topic, consumer, req.offset) {
        Ok(checkpoint) => {
            if let Err(e) = st.persist(&journal) {
                return ApiErr::new(StatusCode::INTERNAL_SERVER_ERROR, "internal", e.to_string())
                    .into_response();
            }
            (StatusCode::OK, Json(checkpoint)).into_response()
        }
        Err(e @ TapeError::StaleCheckpoint { .. }) => {
            ApiErr::new(StatusCode::CONFLICT, "conflict", e.to_string()).into_response()
        }
        Err(e @ TapeError::CheckpointBeyondEnd { .. }) => {
            ApiErr::new(StatusCode::CONFLICT, "conflict", e.to_string()).into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = "/topics/{topic}/subscriptions",
    params(("topic" = String, Path, description = "Topic name")),
    request_body = SubscriptionCreateRequest,
    responses((status = 201, description = "Created subscription", body = Subscription))
)]
pub async fn subscription_create(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path(topic): Path<String>,
    body: axum::body::Bytes,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &topic, Role::Write) {
        return deny.into_response();
    }
    let req: SubscriptionCreateRequest = match serde_json::from_slice(&body) {
        Ok(request) => request,
        Err(error) => {
            return ApiErr::new(StatusCode::BAD_REQUEST, "bad_request", error.to_string())
                .into_response()
        }
    };
    if let Some(raft) = st.raft() {
        return match raft.propose_subscription_create(topic, req.name).await {
            Ok((_, Some(TapeOutcome::SubscriptionCreated(Ok(subscription))))) => {
                (StatusCode::CREATED, Json(subscription)).into_response()
            }
            Ok((_, Some(TapeOutcome::SubscriptionCreated(Err(error))))) => {
                subscription_error(error)
            }
            Ok((_, Some(_))) => outcome_mismatch("subscription_create"),
            Ok((_, None)) => missing_outcome("subscription_create"),
            Err(error) => raft_unavailable(error),
        };
    }
    let mut journal = st.journal.lock().expect("journal mutex poisoned");
    match journal.create_subscription(topic, req.name) {
        Ok(subscription) => {
            if let Err(error) = st.persist(&journal) {
                return ApiErr::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal",
                    error.to_string(),
                )
                .into_response();
            }
            (StatusCode::CREATED, Json(subscription)).into_response()
        }
        Err(error) => subscription_error(error),
    }
}

#[utoipa::path(
    get,
    path = "/topics/{topic}/subscriptions",
    params(("topic" = String, Path, description = "Topic name")),
    responses((status = 200, description = "Topic subscriptions", body = SubscriptionListResponse))
)]
pub async fn subscription_list(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path(topic): Path<String>,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &topic, Role::Read) {
        return deny.into_response();
    }
    let subscriptions = st
        .journal
        .lock()
        .expect("journal mutex poisoned")
        .subscriptions(&topic);
    (
        StatusCode::OK,
        Json(SubscriptionListResponse { subscriptions }),
    )
        .into_response()
}

#[utoipa::path(
    get,
    path = "/topics/{topic}/subscriptions/{name}",
    params(
        ("topic" = String, Path, description = "Topic name"),
        ("name" = String, Path, description = "Subscription name")
    ),
    responses((status = 200, description = "Subscription", body = Subscription))
)]
pub async fn subscription_get(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path((topic, name)): Path<(String, String)>,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &topic, Role::Read) {
        return deny.into_response();
    }
    match st
        .journal
        .lock()
        .expect("journal mutex poisoned")
        .subscription(&topic, &name)
        .cloned()
    {
        Some(subscription) => (StatusCode::OK, Json(subscription)).into_response(),
        None => subscription_error(SubscriptionError::NotFound { topic, name }),
    }
}

#[utoipa::path(
    delete,
    path = "/topics/{topic}/subscriptions/{name}",
    params(
        ("topic" = String, Path, description = "Topic name"),
        ("name" = String, Path, description = "Subscription name")
    ),
    responses((status = 200, description = "Deleted subscription", body = Subscription))
)]
pub async fn subscription_delete(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path((topic, name)): Path<(String, String)>,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &topic, Role::Write) {
        return deny.into_response();
    }
    if let Some(raft) = st.raft() {
        return match raft.propose_subscription_delete(topic, name).await {
            Ok((_, Some(TapeOutcome::SubscriptionDeleted(Ok(subscription))))) => {
                (StatusCode::OK, Json(subscription)).into_response()
            }
            Ok((_, Some(TapeOutcome::SubscriptionDeleted(Err(error))))) => {
                subscription_error(error)
            }
            Ok((_, Some(_))) => outcome_mismatch("subscription_delete"),
            Ok((_, None)) => missing_outcome("subscription_delete"),
            Err(error) => raft_unavailable(error),
        };
    }
    let mut journal = st.journal.lock().expect("journal mutex poisoned");
    match journal.delete_subscription(&topic, &name) {
        Ok(subscription) => {
            if let Err(error) = st.persist(&journal) {
                return ApiErr::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal",
                    error.to_string(),
                )
                .into_response();
            }
            (StatusCode::OK, Json(subscription)).into_response()
        }
        Err(error) => subscription_error(error),
    }
}

#[utoipa::path(
    post,
    path = "/topics/{topic}/subscriptions/{name}/pull",
    params(
        ("topic" = String, Path, description = "Topic name"),
        ("name" = String, Path, description = "Subscription name")
    ),
    request_body = Option<SubscriptionPullRequest>,
    responses((status = 200, description = "Side-effect-free bounded replay window", body = PullSubscriptionBatch))
)]
pub async fn subscription_pull(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path((topic, name)): Path<(String, String)>,
    body: axum::body::Bytes,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &topic, Role::Read) {
        return deny.into_response();
    }
    let req: SubscriptionPullRequest = if body.is_empty() {
        SubscriptionPullRequest { limit: None }
    } else {
        match serde_json::from_slice(&body) {
            Ok(request) => request,
            Err(error) => {
                return ApiErr::new(StatusCode::BAD_REQUEST, "bad_request", error.to_string())
                    .into_response()
            }
        }
    };
    match st
        .journal
        .lock()
        .expect("journal mutex poisoned")
        .pull_subscription(&topic, &name, req.limit)
    {
        Ok(batch) => (StatusCode::OK, Json::<PullSubscriptionBatch>(batch)).into_response(),
        Err(error) => subscription_error(error),
    }
}

#[utoipa::path(
    post,
    path = "/topics/{topic}/subscriptions/{name}/ack",
    params(
        ("topic" = String, Path, description = "Topic name"),
        ("name" = String, Path, description = "Subscription name")
    ),
    request_body = SubscriptionAckRequest,
    responses((status = 200, description = "Explicitly advanced checkpoint", body = ConsumerCheckpoint))
)]
pub async fn subscription_ack(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path((topic, name)): Path<(String, String)>,
    body: axum::body::Bytes,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &topic, Role::Read) {
        return deny.into_response();
    }
    let req: SubscriptionAckRequest = match serde_json::from_slice(&body) {
        Ok(request) => request,
        Err(error) => {
            return ApiErr::new(StatusCode::BAD_REQUEST, "bad_request", error.to_string())
                .into_response()
        }
    };
    if let Some(raft) = st.raft() {
        return match raft
            .propose_subscription_ack(topic, name, req.offset, crate::now_ms())
            .await
        {
            Ok((_, Some(TapeOutcome::SubscriptionAcked(Ok(checkpoint))))) => {
                (StatusCode::OK, Json(checkpoint)).into_response()
            }
            Ok((_, Some(TapeOutcome::SubscriptionAcked(Err(error))))) => {
                subscription_ack_error(error)
            }
            Ok((_, Some(_))) => outcome_mismatch("subscription_ack"),
            Ok((_, None)) => missing_outcome("subscription_ack"),
            Err(error) => raft_unavailable(error),
        };
    }
    let mut journal = st.journal.lock().expect("journal mutex poisoned");
    match journal.ack_subscription(&topic, &name, req.offset) {
        Ok(checkpoint) => {
            if let Err(error) = st.persist(&journal) {
                return ApiErr::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal",
                    error.to_string(),
                )
                .into_response();
            }
            (StatusCode::OK, Json(checkpoint)).into_response()
        }
        Err(error) => subscription_ack_error(error),
    }
}

fn subscription_error(error: SubscriptionError) -> Response {
    let status = match error {
        SubscriptionError::NotFound { .. } => StatusCode::NOT_FOUND,
        SubscriptionError::AlreadyExists { .. } => StatusCode::CONFLICT,
        SubscriptionError::PullBatchTooLarge { .. } => StatusCode::BAD_REQUEST,
    };
    ApiErr::new(status, "subscription_error", error.to_string()).into_response()
}

fn subscription_ack_error(error: SubscriptionAckError) -> Response {
    match error {
        SubscriptionAckError::Subscription(error) => subscription_error(error),
        SubscriptionAckError::Checkpoint(error) => {
            ApiErr::new(StatusCode::CONFLICT, "conflict", error.to_string()).into_response()
        }
    }
}

fn raft_unavailable(error: anyhow::Error) -> Response {
    ApiErr::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "raft_unavailable",
        error.to_string(),
    )
    .into_response()
}

fn missing_outcome(operation: &str) -> Response {
    ApiErr::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "raft_unavailable",
        format!("{operation} outcome unavailable after commit"),
    )
    .into_response()
}

fn outcome_mismatch(operation: &str) -> Response {
    ApiErr::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal",
        format!("raft outcome kind mismatch for {operation}"),
    )
    .into_response()
}

#[utoipa::path(
    get,
    path = "/topics/{topic}/retention",
    params(("topic" = String, Path, description = "Topic name")),
    responses((status = 200, description = "Topic retention policy", body = RetentionGetResponse))
)]
pub async fn retention_get(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path(topic): Path<String>,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &topic, Role::Read) {
        return deny.into_response();
    }
    let policy = st
        .journal
        .lock()
        .expect("journal mutex poisoned")
        .retention(&topic)
        .cloned();
    (StatusCode::OK, Json(RetentionGetResponse { policy })).into_response()
}

#[utoipa::path(
    put,
    path = "/topics/{topic}/retention",
    params(("topic" = String, Path, description = "Topic name")),
    request_body = RetentionPolicy,
    responses((status = 200, description = "Applied policy and compaction result", body = RetentionOutcome))
)]
pub async fn retention_put(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
    Path(topic): Path<String>,
    body: axum::body::Bytes,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &topic, Role::Write) {
        return deny.into_response();
    }
    let policy: RetentionPolicy = match serde_json::from_slice(&body) {
        Ok(policy) => policy,
        Err(error) => {
            return ApiErr::new(StatusCode::BAD_REQUEST, "bad_request", error.to_string())
                .into_response()
        }
    };
    let now_ms = crate::now_ms();
    if let Some(raft) = st.raft() {
        return match raft.propose_retention(topic, policy, now_ms).await {
            Ok((_, Some(TapeOutcome::RetentionUpdated(outcome)))) => {
                (StatusCode::OK, Json(outcome)).into_response()
            }
            Ok((_, Some(_))) => outcome_mismatch("retention_put"),
            Ok((_, None)) => missing_outcome("retention_put"),
            Err(error) => raft_unavailable(error),
        };
    }
    let mut journal = st.journal.lock().expect("journal mutex poisoned");
    let outcome = journal.put_retention(topic, policy, now_ms);
    if let Err(error) = st.persist(&journal) {
        return ApiErr::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            error.to_string(),
        )
        .into_response();
    }
    (StatusCode::OK, Json(outcome)).into_response()
}

/// `GET /admin/backup` — a consistent snapshot of the whole journal for
/// backup runners (#1329): the EXACT bytes [`crate::raft::TapeStateMachine`]'s
/// raft snapshot produces ([`crate::raft::snapshot_bytes`] — the whole
/// journal + the applied index; 0 on a raft-less single node). A
/// cluster-wide admin op: requires `admin` on `*` when auth is required.
/// Restore = feed the bytes to `TapeStateMachine::restore` on a fresh node
/// (the existing raft-side merge path); no restore CLI verb is added here.
#[utoipa::path(
    get,
    path = "/admin/backup",
    responses((status = 200, description = "JournalSnapshot JSON { up_to, journal } — the whole journal at the applied raft index"))
)]
pub async fn admin_backup(
    State(st): State<AppState>,
    Extension(principal): Extension<AuditedRoleMapPrincipal>,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, "*", Role::Admin) {
        return deny.into_response();
    }
    let raft = st.raft();
    let applied = raft.as_ref().map(|raft| raft.applied_index()).unwrap_or(0);
    let snapshot = match raft {
        Some(raft) => raft.snapshot_bytes(),
        None => crate::raft::snapshot_bytes(&st.journal, applied),
    };
    match snapshot {
        Ok(bytes) => {
            // Audit only the low-frequency management operation. Append and
            // consumer checkpoint traffic is deliberately not duplicated into
            // logs: its durable, payload-free audit trail is the Tape journal
            // itself, while credentials/denials are already emitted through
            // the shared service-auth redacted audit sink.
            tracing::info!(
                target: "tape.audit",
                event = "backup_snapshot_served",
                subject = principal.subject().unwrap_or("anonymous"),
                applied_index = applied,
                bytes = bytes.len(),
            );
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/json")],
                bytes,
            )
                .into_response()
        }
        Err(e) => ApiErr::new(StatusCode::INTERNAL_SERVER_ERROR, "internal", e.to_string())
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn append_replay_and_checkpoint_round_trip() {
        let state = AppState::new(TapeJournal::default(), None);
        let app = router(state);

        let resp = crate::server::tests::post_json(
            app.clone(),
            "/topics/orders/append",
            &serde_json::json!({ "payload": { "n": 1 } }),
        )
        .await;
        assert_eq!(resp.0, StatusCode::OK);

        let resp = crate::server::tests::get(app.clone(), "/topics/orders/replay").await;
        assert_eq!(resp.0, StatusCode::OK);
        let body: serde_json::Value = serde_json::from_str(&resp.1).unwrap();
        assert_eq!(body["events"].as_array().unwrap().len(), 1);

        let resp = crate::server::tests::put_json(
            app.clone(),
            "/topics/orders/consumers/c1/checkpoint",
            &serde_json::json!({ "offset": 1 }),
        )
        .await;
        assert_eq!(resp.0, StatusCode::OK);

        let resp =
            crate::server::tests::get(app.clone(), "/topics/orders/consumers/c1/checkpoint").await;
        assert_eq!(resp.0, StatusCode::OK);
        let body: serde_json::Value = serde_json::from_str(&resp.1).unwrap();
        assert_eq!(body["checkpoint"]["offset"], 1);
    }

    #[tokio::test]
    async fn pull_subscription_is_bounded_side_effect_free_and_explicitly_acked() {
        let app = router(AppState::new(TapeJournal::default(), None));
        for n in 0..2 {
            let response = post_json(
                app.clone(),
                "/topics/orders/append",
                &serde_json::json!({ "payload": { "n": n } }),
            )
            .await;
            assert_eq!(response.0, StatusCode::OK);
        }
        let created = post_json(
            app.clone(),
            "/topics/orders/subscriptions",
            &serde_json::json!({ "name": "audit" }),
        )
        .await;
        assert_eq!(created.0, StatusCode::CREATED);

        let push = post_json(
            app.clone(),
            "/topics/orders/subscriptions",
            &serde_json::json!({
                "name": "webhook",
                "delivery": { "mode": "push", "endpoint": "https://example.invalid" }
            }),
        )
        .await;
        assert_eq!(push.0, StatusCode::BAD_REQUEST, "push is not a Tape mode");

        let first = post_json(
            app.clone(),
            "/topics/orders/subscriptions/audit/pull",
            &serde_json::json!({ "limit": 2 }),
        )
        .await;
        assert_eq!(first.0, StatusCode::OK);
        let first_body: serde_json::Value = serde_json::from_str(&first.1).unwrap();
        assert_eq!(first_body["cursor"], 0);
        assert_eq!(first_body["next_offset"], 2);
        assert_eq!(first_body["events"].as_array().unwrap().len(), 2);

        let repeated = post_json(
            app.clone(),
            "/topics/orders/subscriptions/audit/pull",
            &serde_json::json!({ "limit": 2 }),
        )
        .await;
        let repeated_body: serde_json::Value = serde_json::from_str(&repeated.1).unwrap();
        assert_eq!(repeated_body["cursor"], 0, "pull must not implicitly ack");

        let acked = post_json(
            app.clone(),
            "/topics/orders/subscriptions/audit/ack",
            &serde_json::json!({ "offset": 2 }),
        )
        .await;
        assert_eq!(acked.0, StatusCode::OK);

        let drained = post_json(
            app.clone(),
            "/topics/orders/subscriptions/audit/pull",
            &serde_json::json!({ "limit": 2 }),
        )
        .await;
        let drained_body: serde_json::Value = serde_json::from_str(&drained.1).unwrap();
        assert_eq!(drained_body["cursor"], 2);
        assert!(drained_body["events"].as_array().unwrap().is_empty());

        let stale = post_json(
            app,
            "/topics/orders/subscriptions/audit/ack",
            &serde_json::json!({ "offset": 1 }),
        )
        .await;
        assert_eq!(stale.0, StatusCode::CONFLICT);
    }

    /// R1: `GET /admin/backup` denies a non-admin principal (403) and
    /// streams exactly the `raft::snapshot_bytes` bytes to an admin-on-`*`
    /// principal (200), over an in-process `oneshot` request (no real
    /// socket — `tests/backup.rs` covers the live-HTTP + 401 case).
    #[tokio::test]
    async fn admin_backup_requires_admin_and_streams_snapshot() {
        use tower::ServiceExt;
        let tokens = serde_json::json!({
            "admin-token": { "subject": "ops", "roles": { "*": "admin" } },
            "reader-token": { "subject": "worker", "roles": { "*": "read" } },
        })
        .to_string();
        let auth = crate::auth::AuthConfig::resolve("required", None, Some(&tokens)).unwrap();
        let mut journal = TapeJournal::default();
        journal.append("orders", None, serde_json::json!({"n": 1}), Some(100));
        let state = AppState::with_auth(journal, None, auth);
        let handle = state.journal_handle();
        let app = router(state);

        let deny = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/admin/backup")
                    .header("authorization", "Bearer reader-token")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(deny.status(), StatusCode::FORBIDDEN);

        let ok = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/admin/backup")
                    .header("authorization", "Bearer admin-token")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(ok.status(), StatusCode::OK);
        let bytes = http_body_util::BodyExt::collect(ok.into_body())
            .await
            .unwrap()
            .to_bytes();
        let expected = crate::raft::snapshot_bytes(&handle, 0).unwrap();
        assert_eq!(&bytes[..], &expected[..]);
    }

    #[tokio::test]
    async fn secure_peer_mode_does_not_expose_raft_routes_on_public_router() {
        use tower::ServiceExt;

        let journal = Arc::new(Mutex::new(TapeJournal::default()));
        let dir = tempfile::tempdir().unwrap();
        let raft = Arc::new(
            TapeRaft::spawn(
                Arc::clone(&journal),
                dir.path(),
                0,
                raft_runtime::Membership {
                    voters: vec![0],
                    learners: vec![],
                },
                std::collections::HashMap::new(),
                TapeRaft::host_config(1024),
            )
            .unwrap(),
        );
        let mut state = AppState::new(TapeJournal::default(), None);
        state.set_raft(raft);

        let response = router_without_raft_routes(state)
            .oneshot(
                axum::http::Request::builder()
                    .uri("/raftz")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // Small oneshot helpers so both this module's tests and
    // `tests/http_transport.rs` share one shape (the integration test drives
    // the router over real HTTP instead — these stay unit-level).
    pub(crate) async fn get(app: Router, path: &str) -> (StatusCode, String) {
        use http_body_util::BodyExt;
        use tower::ServiceExt;
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .uri(path)
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    pub(crate) async fn post_json(
        app: Router,
        path: &str,
        body: &serde_json::Value,
    ) -> (StatusCode, String) {
        use http_body_util::BodyExt;
        use tower::ServiceExt;
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(path)
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    pub(crate) async fn put_json(
        app: Router,
        path: &str,
        body: &serde_json::Value,
    ) -> (StatusCode, String) {
        use http_body_util::BodyExt;
        use tower::ServiceExt;
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .method("PUT")
                    .uri(path)
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }
}
