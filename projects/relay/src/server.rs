// SPEC-MANAGED: projects/relay/tech-design/interfaces/rest/http-2-openapi-transport-client-side-sharding-streaming-subscrib.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:a8062fb3" tracker="pending-tracker" reason="axum h2c app over the relay core: publish/lease/ack handlers (JSON + CBOR) and the streaming broadcast subscribe handler."
//! axum HTTP/2 (h2c) application over the relay core.
//!
//! `publish` / `lease` / `ack` / `lease-batch` / `ack-batch` are
//! request/response (JSON, plus an `application/cbor` fast path for hot calls);
//! `subscribe` opens a long-lived HTTP/2 stream of length-prefixed CBOR
//! [`crate::LogEntry`] frames from a seq. The core is internally synchronized
//! (per-shard locking, #128), so the server holds it as a plain `Arc<Relay>` —
//! no global lock.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use axum::{
    body::{Body, Bytes},
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use chrono::Utc;

use crate::engine::Relay;
use crate::server_config::RelayServerConfig;
use crate::wire::{
    self, AckBatchRequest, AckBatchResponse, AckRequest, AckResponse, HeartbeatRequest,
    HeartbeatResponse, LeaseBatchRequest, LeaseBatchResponse, LeaseRequest, LeaseResponse,
    PublishBatchRequest, PublishBatchResponse, PublishRequest, SubscribeQuery,
};

/// Shared application state: the relay core plus this shard's config.
#[derive(Clone)]
pub struct AppState {
    relay: Arc<Relay>,
    config: Arc<RelayServerConfig>,
    next_sub: Arc<AtomicU64>,
}

impl AppState {
    /// Build state with a fresh relay core from `config`.
    pub fn new(config: RelayServerConfig) -> Self {
        let relay = Relay::new(config.core.clone());
        AppState {
            relay: Arc::new(relay),
            config: Arc::new(config),
            next_sub: Arc::new(AtomicU64::new(0)),
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
}

/// Build the HTTP/2 router for the relay transport.
///
/// @spec projects/relay/tech-design/interfaces/rest/http-2-openapi-transport-client-side-sharding-streaming-subscrib.md#logic
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/v1/{subject}/publish", post(publish))
        .route("/v1/{subject}/publish-batch", post(publish_batch))
        .route("/v1/{subject}/lease", post(lease))
        .route("/v1/{subject}/ack", post(ack))
        .route("/v1/{subject}/lease-batch", post(lease_batch))
        .route("/v1/{subject}/ack-batch", post(ack_batch))
        .route("/v1/{subject}/heartbeat", post(heartbeat))
        .route("/v1/{subject}/subscribe", get(subscribe))
        .route("/v1/{subject}/len", get(log_len))
        .route("/healthz", get(healthz))
        .route("/openapi.json", get(openapi_json))
        .with_state(state)
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
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
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
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };
    let now = Utc::now();
    let result = st
        .relay
        .publish(&subject, &req.message_id, req.payload, req.headers, now);
    match result {
        Ok(outcome) => encode_body(cbor, StatusCode::OK, &outcome),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
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
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };
    let now = Utc::now();
    let messages = req
        .messages
        .into_iter()
        .map(|m| (m.message_id, m.payload, m.headers))
        .collect();
    match st.relay.publish_batch(&subject, messages, now) {
        Ok(outcomes) => encode_body(cbor, StatusCode::OK, &PublishBatchResponse { outcomes }),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
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
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
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
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
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
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
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
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
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
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
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

/// `GET /v1/{subject}/subscribe?from_seq=` — tail the broadcast stream.
#[utoipa::path(
    get,
    path = "/v1/{subject}/subscribe",
    params(
        ("subject" = String, Path, description = "Target subject"),
        ("from_seq" = u64, Query, description = "Seq to start replay from")
    ),
    responses((status = 200, description = "Stream of length-prefixed CBOR log entries"))
)]
pub async fn subscribe(
    State(st): State<AppState>,
    Path(subject): Path<String>,
    Query(query): Query<SubscribeQuery>,
) -> Response {
    let subscriber_id = query
        .subscriber_id
        .clone()
        .unwrap_or_else(|| format!("sub-{}", st.next_sub.fetch_add(1, Ordering::Relaxed)));

    if let Err(e) = st.relay.subscribe(&subject, &subscriber_id, query.from_seq) {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    let wake = st.relay.subscribe_wake(&subject);
    let stream = futures::stream::unfold(
        (st, subject, subscriber_id, wake),
        |(st, subject, subscriber_id, mut wake)| async move {
            loop {
                let frames = st.relay.poll(&subject, &subscriber_id).unwrap_or_default();
                if !frames.is_empty() {
                    let mut buf = Vec::new();
                    for entry in &frames {
                        buf.extend(wire::encode_frame(entry));
                    }
                    let item: Result<Bytes, std::convert::Infallible> = Ok(Bytes::from(buf));
                    return Some((item, (st, subject, subscriber_id, wake)));
                }
                if wake.changed().await.is_err() {
                    return None;
                }
            }
        },
    );

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, wire::CBOR_SEQ)],
        Body::from_stream(stream),
    )
        .into_response()
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
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn healthz() -> StatusCode {
    StatusCode::OK
}

async fn openapi_json() -> Response {
    let doc = crate::openapi::api_doc_json();
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        doc,
    )
        .into_response()
}
// HANDWRITE-END
