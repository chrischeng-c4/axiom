// SPEC-MANAGED: projects/relay/tech-design/interfaces/rest/http-2-openapi-transport-client-side-sharding-work-queue-consume.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:a8062fb3" tracker="pending-tracker" reason="axum h2c app over the relay core: publish/lease/ack handlers (JSON + CBOR) and the streaming broadcast subscribe handler."
//! axum HTTP/2 (h2c) application over the relay core.
//!
//! `publish` / `lease` / `ack` / `lease-batch` / `ack-batch` / `heartbeat` are
//! request/response (JSON, plus an `application/cbor` fast path for hot calls);
//! `consume` is the streaming work-queue path. The core is internally
//! synchronized (per-shard locking, #128), so the server holds it as a plain
//! `Arc<Relay>` — no global lock.
//!
//! The operational surface is the shared service shell (#1205): the standard
//! probe routes (`/healthz` `/readyz` `/metrics` `/openapi.json` `/docs`)
//! come from `service_http::standard_probe_routes` merged with the `/v1`
//! data plane; error responses render the shared `{error, message}` envelope
//! ([`service_http::ApiErr`]); per-op request metrics are recorded by
//! [`crate::metrics::track`] on the data plane.
//!
//! Request auth is the shared `libs/service-auth` bearer contract (#1206):
//! the blanket `service_auth::auth_middleware` runs on the `/v1` data plane
//! ONLY (probes stay tokenless), injecting a [`RoleMapPrincipal`] each
//! handler authorizes on its `{subject}` via [`crate::auth::authorize`] —
//! publish family = write, consume family = read.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Extension, Path, State},
    http::{header, HeaderMap, StatusCode},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use chrono::Utc;
use service_auth::{Role, RoleMapPrincipal, StaticRoleMapVerifier};
use service_http::{ApiErr, MetricsProvider};

use crate::engine::Relay;
use crate::metrics::RelayMetrics;
use crate::server_config::RelayServerConfig;
use crate::wire::{
    self, AckBatchRequest, AckBatchResponse, AckRequest, AckResponse, HeartbeatRequest,
    HeartbeatResponse, LeaseBatchRequest, LeaseBatchResponse, LeaseRequest, LeaseResponse,
    PublishBatchRequest, PublishBatchResponse, PublishRequest,
};

/// Shared application state: the relay core plus this shard's config, the
/// per-op request metrics, the drain flag `/readyz` reports, the bearer
/// verifier the data-plane auth layer runs (#1206), and — in replica/HA mode
/// (#544) — the raft group the publish path proposes through.
#[derive(Clone)]
pub struct AppState {
    relay: Arc<Relay>,
    config: Arc<RelayServerConfig>,
    metrics: Arc<RelayMetrics>,
    draining: Arc<AtomicBool>,
    verifier: Arc<StaticRoleMapVerifier>,
    raft: Option<Arc<crate::raft::RelayRaft>>,
}

impl AppState {
    /// Build state with a fresh relay core from `config`. Auth is open
    /// (tokenless — the `RELAY_AUTH=off` default); production serving builds
    /// through [`AppState::with_auth`].
    pub fn new(config: RelayServerConfig) -> Self {
        let relay = Relay::new(config.core.clone());
        AppState {
            relay: Arc::new(relay),
            config: Arc::new(config),
            metrics: Arc::new(RelayMetrics::new()),
            draining: Arc::new(AtomicBool::new(false)),
            verifier: Arc::new(StaticRoleMapVerifier::open()),
            raft: None,
        }
    }

    /// Build state with a resolved auth config (`--auth` /
    /// `--token-registry-file`): the data-plane auth layer runs the registry
    /// verifier when auth is required, the open verifier when off.
    pub fn with_auth(config: RelayServerConfig, auth: crate::auth::AuthConfig) -> Self {
        let mut state = AppState::new(config);
        state.verifier = Arc::new(auth.verifier());
        state
    }

    /// The bearer verifier the data-plane auth middleware runs.
    pub fn verifier(&self) -> Arc<StaticRoleMapVerifier> {
        Arc::clone(&self.verifier)
    }

    /// Attach the raft group (replica/HA mode, #544): publish/publish-batch
    /// propose through it instead of writing the engine directly. The group
    /// MUST replicate into this state's engine (`relay_handle()`).
    pub fn set_raft(&mut self, raft: Arc<crate::raft::RelayRaft>) {
        self.raft = Some(raft);
    }

    /// The raft group, when running in replica/HA mode.
    pub fn raft(&self) -> Option<Arc<crate::raft::RelayRaft>> {
        self.raft.clone()
    }

    /// This shard's advertised config.
    pub fn config(&self) -> &RelayServerConfig {
        &self.config
    }

    /// A handle to the shared relay core, for the background reconciler.
    pub fn relay_handle(&self) -> Arc<Relay> {
        Arc::clone(&self.relay)
    }

    /// The per-op request metrics `/metrics` renders.
    pub fn metrics(&self) -> Arc<RelayMetrics> {
        Arc::clone(&self.metrics)
    }

    /// Flip readiness to draining so `/readyz` returns 503. Called on SIGTERM
    /// via `service_http::shutdown_with_drain`.
    pub fn start_drain(&self) {
        self.draining.store(true, Ordering::SeqCst);
    }

    pub fn is_draining(&self) -> bool {
        self.draining.load(Ordering::SeqCst)
    }
}

/// Readiness source for the shared probe router: `/readyz` reports 503 once
/// SIGTERM flips `start_drain`.
impl service_http::ReadinessHook for AppState {
    fn is_draining(&self) -> bool {
        self.draining.load(Ordering::SeqCst)
    }
}

/// Prometheus exposition for the shared `/metrics` route: relay's per-op
/// request counts + latency (the engine exposes no aggregate gauges today).
impl service_http::MetricsProvider for AppState {
    fn render_metrics(&self) -> String {
        self.metrics.render()
    }
}

/// Build the HTTP/2 router for the relay transport.
///
/// @spec projects/relay/tech-design/interfaces/rest/http-2-openapi-transport-client-side-sharding-work-queue-consume.md#logic
pub fn router(state: AppState) -> Router {
    let req_metrics = state.metrics();
    let verifier = state.verifier();
    let data_plane = Router::new()
        .route("/v1/{subject}/publish", post(publish))
        .route("/v1/{subject}/publish-batch", post(publish_batch))
        // Streaming work-queue consume (#449) — the primary consume path, listed
        // in the OpenAPI doc. As of #463 every loom consumer uses it: the schema
        // layer's task source (#449) and the controller's completion consumer.
        //
        // The polling lease/ack/lease-batch/ack-batch/heartbeat routes below are
        // DEPRECATED but RETAINED with rationale (#463): (1) they are a
        // SPEC-MANAGED public HTTP surface whose removal is a breaking change that
        // belongs in a dedicated TD-driven deprecation, not this migration; (2)
        // the direct-worker mode (`loom worker` without LOOM_SCHEMA_LAYER, via
        // RelayWorkConsumer) is still a supported, tested deployment; (3) the
        // engine lease/ack/release substrate they wrap is shared with /consume and
        // stays regardless. New consumers must use /consume.
        .route("/v1/{subject}/consume", post(crate::consume::consume))
        .route("/v1/{subject}/lease", post(lease)) // DEPRECATED → use /consume
        .route("/v1/{subject}/ack", post(ack)) // DEPRECATED → use /consume
        .route("/v1/{subject}/lease-batch", post(lease_batch)) // DEPRECATED → /consume
        .route("/v1/{subject}/ack-batch", post(ack_batch)) // DEPRECATED → /consume
        .route("/v1/{subject}/heartbeat", post(heartbeat)) // DEPRECATED → /consume
        .route("/v1/{subject}/len", get(log_len))
        // Admin surface (#1209): a consistent snapshot of the live engine
        // state for `relay backup`. Inside the auth layer (unlike probes) —
        // the handler requires Role::Admin on `*` (lumen's guard).
        .route("/admin/backup", get(admin_backup))
        // Shared bearer auth (#1206) on the data plane ONLY — probes stay
        // tokenless. The blanket middleware authenticates (401 on a
        // missing/unknown token when required) and injects the
        // RoleMapPrincipal each handler authorizes on its {subject}.
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
    // `/docs`) come from the shared service shell so the operational surface
    // matches every other service; the hand-rolled healthz/openapi_json
    // handlers are gone. AppState supplies readiness + Prometheus metrics;
    // `/readyz` reports 503 while draining.
    let probe_state = Arc::new(state);
    let metrics: Arc<dyn MetricsProvider> = probe_state.clone();
    let probes =
        service_http::standard_probe_routes(probe_state, Some(metrics), crate::openapi::openapi);

    // relay is single-tenant per deployment (tenancy = k8s namespace); no
    // app-level namespace rewrite. Run one relay per tenant for isolation.
    probes
        .merge(data_plane)
        // One INFO-level tracing span per request — spans probes + data plane.
        .layer(service_http::trace_layer())
}

fn wants_cbor(headers: &HeaderMap) -> bool {
    let is = |name: header::HeaderName| {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.contains("cbor"))
            .unwrap_or(false)
    };
    is(header::CONTENT_TYPE) || is(header::ACCEPT)
}

fn decode_body<T: serde::de::DeserializeOwned>(cbor: bool, body: &[u8]) -> Result<T, String> {
    if cbor {
        wire::from_cbor(body).map_err(|e| e.to_string())
    } else {
        serde_json::from_slice(body).map_err(|e| e.to_string())
    }
}

fn encode_body<T: serde::Serialize>(cbor: bool, status: StatusCode, value: &T) -> Response {
    if cbor {
        (
            status,
            [(header::CONTENT_TYPE, wire::CBOR)],
            wire::to_cbor(value),
        )
            .into_response()
    } else {
        match serde_json::to_vec(value) {
            Ok(bytes) => {
                (status, [(header::CONTENT_TYPE, "application/json")], bytes).into_response()
            }
            Err(e) => ApiErr::new(StatusCode::INTERNAL_SERVER_ERROR, "internal", e.to_string())
                .into_response(),
        }
    }
}

/// `POST /v1/{subject}/publish` — append a message (idempotent on message_id).
#[utoipa::path(
    post,
    path = "/v1/{subject}/publish",
    params(("subject" = String, Path, description = "Target subject")),
    responses((status = 200, description = "Append outcome { seq, deduped }"))
)]
pub async fn publish(
    State(st): State<AppState>,
    Extension(principal): Extension<RoleMapPrincipal>,
    Path(subject): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &subject, Role::Write) {
        return deny.into_response();
    }
    let cbor = wants_cbor(&headers);
    let req: PublishRequest = match decode_body(cbor, &body) {
        Ok(r) => r,
        Err(e) => return ApiErr::new(StatusCode::BAD_REQUEST, "bad_request", e).into_response(),
    };
    let now = Utc::now();
    // Resolve the optional visibility gate: explicit not_before wins, else
    // delay_ms is a countdown from now (delayed / ETA / countdown delivery).
    let not_before = req.not_before.or_else(|| {
        req.delay_ms
            .map(|ms| now + chrono::Duration::milliseconds(ms as i64))
    });
    // Replica/HA mode (#544): replicate the publish through raft — the host
    // proposes locally on the leader or forwards to the leader, and returns
    // once THIS node's engine applied it (read-your-write).
    if let Some(raft) = st.raft() {
        let cmd = crate::raft::PubCommand {
            subject,
            message_id: req.message_id,
            payload: req.payload,
            headers: req.headers,
            priority: req.priority,
            not_before,
        };
        return match propose_publish(&st, &raft, cmd).await {
            Ok(outcome) => encode_body(cbor, StatusCode::OK, &outcome),
            Err(resp) => resp,
        };
    }
    let result = st.relay.publish_at(
        &subject,
        &req.message_id,
        req.payload,
        req.headers,
        not_before,
        req.priority,
        now,
    );
    match result {
        Ok(outcome) => encode_body(cbor, StatusCode::OK, &outcome),
        Err(e) => ApiErr::new(StatusCode::INTERNAL_SERVER_ERROR, "internal", e.to_string())
            .into_response(),
    }
}

/// Propose one publish through the raft group and resolve its engine outcome:
/// normally claimed from the state machine's outcome window; if it aged out
/// (an unclaimed backlog raced past the window) the engine's idempotent
/// publish returns the existing `{seq, deduped}`. Raft unavailability (no
/// leader / apply timeout) maps to 503.
async fn propose_publish(
    st: &AppState,
    raft: &crate::raft::RelayRaft,
    cmd: crate::raft::PubCommand,
) -> Result<crate::types::AppendOutcome, Response> {
    match raft.publish(&cmd).await {
        Ok((_, Some(outcome))) => Ok(outcome),
        Ok((_, None)) => st
            .relay
            .publish_at(
                &cmd.subject,
                &cmd.message_id,
                cmd.payload,
                cmd.headers,
                cmd.not_before,
                cmd.priority,
                Utc::now(),
            )
            .map_err(|e| {
                ApiErr::new(StatusCode::INTERNAL_SERVER_ERROR, "internal", e.to_string())
                    .into_response()
            }),
        Err(e) => Err(ApiErr::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "raft_unavailable",
            e.to_string(),
        )
        .into_response()),
    }
}

/// `POST /v1/{subject}/publish-batch` — append many messages (group commit).
#[utoipa::path(
    post,
    path = "/v1/{subject}/publish-batch",
    params(("subject" = String, Path, description = "Target subject")),
    responses((status = 200, description = "One append outcome per message, in order"))
)]
pub async fn publish_batch(
    State(st): State<AppState>,
    Extension(principal): Extension<RoleMapPrincipal>,
    Path(subject): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &subject, Role::Write) {
        return deny.into_response();
    }
    let cbor = wants_cbor(&headers);
    let req: PublishBatchRequest = match decode_body(cbor, &body) {
        Ok(r) => r,
        Err(e) => return ApiErr::new(StatusCode::BAD_REQUEST, "bad_request", e).into_response(),
    };
    let now = Utc::now();
    // Replica/HA mode (#544): each message is one raft command, proposed in
    // input order (per-entry raft fsync replaces the local group commit — an
    // accepted HA trade documented in the TD).
    if let Some(raft) = st.raft() {
        let mut outcomes = Vec::with_capacity(req.messages.len());
        for m in req.messages {
            let cmd = crate::raft::PubCommand {
                subject: subject.clone(),
                message_id: m.message_id,
                payload: m.payload,
                headers: m.headers,
                priority: m.priority,
                not_before: None,
            };
            match propose_publish(&st, &raft, cmd).await {
                Ok(outcome) => outcomes.push(outcome),
                Err(resp) => return resp,
            }
        }
        return encode_body(cbor, StatusCode::OK, &PublishBatchResponse { outcomes });
    }
    let messages = req
        .messages
        .into_iter()
        .map(|m| (m.message_id, m.payload, m.headers, m.priority))
        .collect();
    match st.relay.publish_batch(&subject, messages, now) {
        Ok(outcomes) => encode_body(cbor, StatusCode::OK, &PublishBatchResponse { outcomes }),
        Err(e) => ApiErr::new(StatusCode::INTERNAL_SERVER_ERROR, "internal", e.to_string())
            .into_response(),
    }
}

/// `POST /v1/{subject}/lease` — lease the next eligible entry (CBOR fast path).
#[utoipa::path(
    post,
    path = "/v1/{subject}/lease",
    params(("subject" = String, Path, description = "Target subject")),
    responses((status = 200, description = "A lease, or null when nothing is available"))
)]
pub async fn lease(
    State(st): State<AppState>,
    Extension(principal): Extension<RoleMapPrincipal>,
    Path(subject): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &subject, Role::Read) {
        return deny.into_response();
    }
    let cbor = wants_cbor(&headers);
    let req: LeaseRequest = match decode_body(cbor, &body) {
        Ok(r) => r,
        Err(e) => return ApiErr::new(StatusCode::BAD_REQUEST, "bad_request", e).into_response(),
    };
    let now = Utc::now();
    let lease = st
        .relay
        .lease(&subject, &req.consumer_id, now)
        .unwrap_or(None);
    // Attach the leased entry body so the consumer knows what it leased (#166).
    let entry = match &lease {
        Some(l) => st.relay.entry(&l.subject, l.shard, l.seq).unwrap_or(None),
        None => None,
    };
    encode_body(cbor, StatusCode::OK, &LeaseResponse { lease, entry })
}

/// `POST /v1/{subject}/ack` — acknowledge a lease (CBOR fast path).
#[utoipa::path(
    post,
    path = "/v1/{subject}/ack",
    params(("subject" = String, Path, description = "Target subject")),
    responses((status = 200, description = "Ack result { acked, committed_seq }"))
)]
pub async fn ack(
    State(st): State<AppState>,
    Extension(principal): Extension<RoleMapPrincipal>,
    Path(subject): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &subject, Role::Read) {
        return deny.into_response();
    }
    let cbor = wants_cbor(&headers);
    let req: AckRequest = match decode_body(cbor, &body) {
        Ok(r) => r,
        Err(e) => return ApiErr::new(StatusCode::BAD_REQUEST, "bad_request", e).into_response(),
    };
    let acked = st
        .relay
        .ack(&subject, &req.lease_id, req.epoch)
        .unwrap_or(false);
    let committed_seq = st
        .relay
        .committed_offset(&subject)
        .ok()
        .flatten()
        .map(|c| c.committed_seq);
    encode_body(
        cbor,
        StatusCode::OK,
        &AckResponse {
            acked,
            committed_seq,
        },
    )
}

/// `POST /v1/{subject}/lease-batch` — lease up to `max` entries in one call.
#[utoipa::path(
    post,
    path = "/v1/{subject}/lease-batch",
    params(("subject" = String, Path, description = "Target subject")),
    responses((status = 200, description = "Up to max leases in seq order"))
)]
pub async fn lease_batch(
    State(st): State<AppState>,
    Extension(principal): Extension<RoleMapPrincipal>,
    Path(subject): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &subject, Role::Read) {
        return deny.into_response();
    }
    let cbor = wants_cbor(&headers);
    let req: LeaseBatchRequest = match decode_body(cbor, &body) {
        Ok(r) => r,
        Err(e) => return ApiErr::new(StatusCode::BAD_REQUEST, "bad_request", e).into_response(),
    };
    let now = Utc::now();
    let leases = st
        .relay
        .lease_batch(&subject, &req.consumer_id, req.max, now)
        .unwrap_or_default();
    encode_body(cbor, StatusCode::OK, &LeaseBatchResponse { leases })
}

/// `POST /v1/{subject}/ack-batch` — acknowledge many leases in one call.
#[utoipa::path(
    post,
    path = "/v1/{subject}/ack-batch",
    params(("subject" = String, Path, description = "Target subject")),
    responses((status = 200, description = "Count accepted + committed offset"))
)]
pub async fn ack_batch(
    State(st): State<AppState>,
    Extension(principal): Extension<RoleMapPrincipal>,
    Path(subject): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &subject, Role::Read) {
        return deny.into_response();
    }
    let cbor = wants_cbor(&headers);
    let req: AckBatchRequest = match decode_body(cbor, &body) {
        Ok(r) => r,
        Err(e) => return ApiErr::new(StatusCode::BAD_REQUEST, "bad_request", e).into_response(),
    };
    let acks: Vec<(String, Option<u64>)> = req
        .acks
        .into_iter()
        .map(|a| (a.lease_id, a.epoch))
        .collect();
    let (acked, committed) = st.relay.ack_batch(&subject, &acks).unwrap_or((0, None));
    encode_body(
        cbor,
        StatusCode::OK,
        &AckBatchResponse {
            acked,
            committed_seq: committed.map(|c| c.committed_seq),
        },
    )
}

/// `POST /v1/{subject}/heartbeat` — extend a held lease (CBOR fast path).
#[utoipa::path(
    post,
    path = "/v1/{subject}/heartbeat",
    params(("subject" = String, Path, description = "Target subject")),
    responses((status = 200, description = "Heartbeat result { extended, expires_at }"))
)]
pub async fn heartbeat(
    State(st): State<AppState>,
    Extension(principal): Extension<RoleMapPrincipal>,
    Path(subject): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &subject, Role::Read) {
        return deny.into_response();
    }
    let cbor = wants_cbor(&headers);
    let req: HeartbeatRequest = match decode_body(cbor, &body) {
        Ok(r) => r,
        Err(e) => return ApiErr::new(StatusCode::BAD_REQUEST, "bad_request", e).into_response(),
    };
    let now = Utc::now();
    let expires_at = st
        .relay
        .heartbeat(&subject, &req.lease_id, req.epoch, now)
        .unwrap_or(None);
    encode_body(
        cbor,
        StatusCode::OK,
        &HeartbeatResponse {
            extended: expires_at.is_some(),
            expires_at,
        },
    )
}

/// `GET /admin/backup` — a consistent snapshot of the live (un-acked) engine
/// state for backup runners (#1209): the EXACT bytes
/// [`crate::raft::RelayStateMachine`]'s raft snapshot produces
/// ([`crate::raft::snapshot_bytes`] — `dump_live` + the applied index; 0 on a
/// raft-less single node). A cluster-wide admin op: requires `admin` on `*`
/// when auth is required (lumen's guard). Restore = feed the bytes to
/// `raft::load_snapshot_bytes` on a fresh node (`load_live` merge).
#[utoipa::path(
    get,
    path = "/admin/backup",
    responses((status = 200, description = "EngineSnapshot JSON { up_to, subjects } — the live un-acked backlog at the applied raft index"))
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
    match crate::raft::snapshot_bytes(&st.relay, applied) {
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

/// `GET /v1/{subject}/len` — current append count for the subject log.
#[utoipa::path(
    get,
    path = "/v1/{subject}/len",
    params(("subject" = String, Path, description = "Target subject")),
    responses((status = 200, description = "Current log length { latest_seq }"))
)]
pub async fn log_len(
    State(st): State<AppState>,
    Extension(principal): Extension<RoleMapPrincipal>,
    Path(subject): Path<String>,
) -> Response {
    if let Err(deny) = crate::auth::authorize(&principal, &subject, Role::Read) {
        return deny.into_response();
    }
    match st.relay.log_len(&subject) {
        Ok(latest_seq) => encode_body(
            false,
            StatusCode::OK,
            &serde_json::json!({
                "latest_seq": latest_seq,
            }),
        ),
        Err(e) => ApiErr::new(StatusCode::INTERNAL_SERVER_ERROR, "internal", e.to_string())
            .into_response(),
    }
}
// HANDWRITE-END
