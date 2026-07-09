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
use axum::http::{header, StatusCode};
use axum::middleware::from_fn_with_state;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use service_auth::{Role, RoleMapPrincipal, StaticRoleMapVerifier};
use service_http::{ApiErr, MetricsProvider};
use utoipa::ToSchema;

use crate::metrics::TapeMetrics;
use crate::raft::{TapeOutcome, TapeRaft};
use crate::{ConsumerCheckpoint, TapeError, TapeEvent, TapeJournal};

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
    verifier: Arc<StaticRoleMapVerifier>,
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
            verifier: Arc::new(StaticRoleMapVerifier::open()),
            raft: None,
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

/// Build the HTTP router for the tape transport: the `/topics` data plane
/// merged onto the shared service shell's standard probe routes.
pub fn router(state: AppState) -> Router {
    let req_metrics = state.metrics();
    let verifier = state.verifier();
    let raft = state.raft();
    let data_plane = Router::new()
        .route("/topics/{topic}/append", axum::routing::post(append))
        .route("/topics/{topic}/replay", get(replay))
        .route(
            "/topics/{topic}/consumers/{consumer}/checkpoint",
            get(checkpoint_get).put(checkpoint_put),
        )
        // Cluster-wide admin op (#1329): a consistent snapshot of the journal
        // for backup runners. Inside the auth layer (unlike probes) — needs
        // `admin` on `*`.
        .route("/admin/backup", get(admin_backup))
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
            Ok((_, Some(TapeOutcome::Appended(event)))) => (StatusCode::OK, Json(event)).into_response(),
            Ok((_, Some(TapeOutcome::Checkpoint(_)))) => ApiErr::new(
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
            Err(e) => {
                ApiErr::new(StatusCode::SERVICE_UNAVAILABLE, "raft_unavailable", e.to_string())
                    .into_response()
            }
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
            Ok((_, Some(TapeOutcome::Checkpoint(Err(e @ TapeError::CheckpointBeyondEnd { .. }))))) => {
                ApiErr::new(StatusCode::CONFLICT, "conflict", e.to_string()).into_response()
            }
            Ok((_, Some(TapeOutcome::Appended(_)))) => ApiErr::new(
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
            Err(e) => {
                ApiErr::new(StatusCode::SERVICE_UNAVAILABLE, "raft_unavailable", e.to_string())
                    .into_response()
            }
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
    Extension(principal): Extension<RoleMapPrincipal>,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, "*", Role::Admin) {
        return deny.into_response();
    }
    let applied = match st.raft() {
        Some(raft) => raft.applied_index(),
        None => 0,
    };
    match crate::raft::snapshot_bytes(&st.journal, applied) {
        Ok(bytes) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            bytes,
        )
            .into_response(),
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
