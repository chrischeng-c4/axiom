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
//! plane ONLY (probes stay tokenless), injecting a [`RoleMapPrincipal`] each
//! handler authorizes on its `{topic}` via [`crate::auth::authorize`] —
//! `append` = write, `replay`/`checkpoint_get`/`checkpoint_put` = read.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::middleware::from_fn_with_state;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use service_auth::{Role, RoleMapPrincipal, StaticRoleMapVerifier};
use service_http::{ApiErr, MetricsProvider};
use utoipa::ToSchema;

use crate::metrics::TapeMetrics;
use crate::{ConsumerCheckpoint, TapeError, TapeEvent, TapeJournal};

/// Shared application state: the journal (behind a `std::sync::Mutex` — an
/// in-memory `BTreeMap` core with no async internal awaits), the per-op
/// request metrics, the drain flag `/readyz` reports, the optional file the
/// journal persists to on every mutation (`--store`, mirroring the CLI's
/// `load_journal`/`save_journal`), and the bearer verifier the data-plane
/// auth layer runs (#1326).
#[derive(Clone)]
pub struct AppState {
    journal: Arc<Mutex<TapeJournal>>,
    metrics: Arc<TapeMetrics>,
    draining: Arc<AtomicBool>,
    store: Option<PathBuf>,
    verifier: Arc<StaticRoleMapVerifier>,
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
            verifier: Arc::new(StaticRoleMapVerifier::open()),
        }
    }

    /// Build state with a resolved auth config (`--auth` /
    /// `--token-registry-file`): the data-plane auth layer runs the registry
    /// verifier when auth is required, the open verifier when off.
    pub fn with_auth(journal: TapeJournal, store: Option<PathBuf>, auth: crate::auth::AuthConfig) -> Self {
        let mut state = Self::new(journal, store);
        state.verifier = Arc::new(auth.verifier());
        state
    }

    /// The bearer verifier the data-plane auth middleware runs.
    pub fn verifier(&self) -> Arc<StaticRoleMapVerifier> {
        Arc::clone(&self.verifier)
    }

    /// The per-op request metrics `/metrics` renders.
    pub fn metrics(&self) -> Arc<TapeMetrics> {
        Arc::clone(&self.metrics)
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

/// Build the HTTP router for the tape transport: the `/topics` data plane
/// merged onto the shared service shell's standard probe routes.
pub fn router(state: AppState) -> Router {
    let req_metrics = state.metrics();
    let verifier = state.verifier();
    let data_plane = Router::new()
        .route("/topics/{topic}/append", axum::routing::post(append))
        .route("/topics/{topic}/replay", get(replay))
        .route(
            "/topics/{topic}/consumers/{consumer}/checkpoint",
            get(checkpoint_get).put(checkpoint_put),
        )
        // Shared bearer auth (#1326) on the data plane ONLY — probes stay
        // tokenless. The blanket middleware authenticates (401 on a
        // missing/unknown token when required) and injects the
        // RoleMapPrincipal each handler authorizes on its {topic}.
        .route_layer(from_fn_with_state(
            verifier,
            service_auth::auth_middleware::<StaticRoleMapVerifier>,
        ))
        // Per-op request metrics (counts + latency). route_layer => only for
        // matched data-plane routes, and MatchedPath is populated. Added
        // after (= outside) the auth layer so rejected requests are still
        // counted.
        .route_layer(from_fn_with_state(req_metrics, crate::metrics::track))
        .with_state(state.clone());

    // Standard probes (`/healthz`, `/readyz`, `/metrics`, `/openapi.json`,
    // `/docs`) come from the shared service shell so the operational
    // surface matches every other service in the ecosystem. AppState
    // supplies readiness + Prometheus metrics; `/readyz` reports 503 while
    // draining.
    let probe_state = Arc::new(state);
    let metrics: Arc<dyn MetricsProvider> = probe_state.clone();
    let probes =
        service_http::standard_probe_routes(probe_state, Some(metrics), crate::openapi::openapi);

    probes
        .merge(data_plane)
        // One INFO-level tracing span per request — spans probes + data plane.
        .layer(service_http::trace_layer())
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
    Extension(principal): Extension<RoleMapPrincipal>,
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
    let mut journal = st.journal.lock().expect("journal mutex poisoned");
    let event = journal.append(topic, req.key, req.payload, req.timestamp_ms);
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
    Extension(principal): Extension<RoleMapPrincipal>,
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
    Extension(principal): Extension<RoleMapPrincipal>,
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
    Extension(principal): Extension<RoleMapPrincipal>,
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
