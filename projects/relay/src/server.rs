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

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use chrono::Utc;
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
/// per-op request metrics, and the drain flag `/readyz` reports.
#[derive(Clone)]
pub struct AppState {
    relay: Arc<Relay>,
    config: Arc<RelayServerConfig>,
    metrics: Arc<RelayMetrics>,
    draining: Arc<AtomicBool>,
}

impl AppState {
    /// Build state with a fresh relay core from `config`.
    pub fn new(config: RelayServerConfig) -> Self {
        let relay = Relay::new(config.core.clone());
        AppState {
            relay: Arc::new(relay),
            config: Arc::new(config),
            metrics: Arc::new(RelayMetrics::new()),
            draining: Arc::new(AtomicBool::new(false)),
        }
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
        // Per-op request metrics (counts + latency). route_layer => only for
        // matched data-plane routes, and MatchedPath is populated.
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
    Path(subject): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
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

/// `POST /v1/{subject}/publish-batch` — append many messages (group commit).
#[utoipa::path(
    post,
    path = "/v1/{subject}/publish-batch",
    params(("subject" = String, Path, description = "Target subject")),
    responses((status = 200, description = "One append outcome per message, in order"))
)]
pub async fn publish_batch(
    State(st): State<AppState>,
    Path(subject): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let cbor = wants_cbor(&headers);
    let req: PublishBatchRequest = match decode_body(cbor, &body) {
        Ok(r) => r,
        Err(e) => return ApiErr::new(StatusCode::BAD_REQUEST, "bad_request", e).into_response(),
    };
    let now = Utc::now();
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
    Path(subject): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
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
    Path(subject): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
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
    Path(subject): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
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
    Path(subject): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
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
    Path(subject): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
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

/// `GET /v1/{subject}/len` — current append count for the subject log.
#[utoipa::path(
    get,
    path = "/v1/{subject}/len",
    params(("subject" = String, Path, description = "Target subject")),
    responses((status = 200, description = "Current log length { latest_seq }"))
)]
pub async fn log_len(State(st): State<AppState>, Path(subject): Path<String>) -> Response {
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
