// SPEC-MANAGED: projects/relay/tech-design/interfaces/rest/http-2-openapi-transport-client-side-sharding-streaming-subscrib.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:a8062fb3" tracker="pending-tracker" reason="axum h2c app over the relay core: publish/lease/ack handlers (JSON + CBOR) and the streaming broadcast subscribe handler."
//! axum HTTP/2 (h2c) application over the relay core.
//!
//! `publish` / `lease` / `ack` are request/response (JSON, plus an
//! `application/cbor` fast path for lease/ack); `subscribe` opens a long-lived
//! HTTP/2 stream of length-prefixed CBOR [`crate::LogEntry`] frames from a seq.
//! State is the shared relay core behind a mutex — locks are never held across
//! an `.await`.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
    self, AckRequest, AckResponse, HeartbeatRequest, HeartbeatResponse, LeaseRequest,
    LeaseResponse, PublishRequest, SubscribeQuery,
};

/// Shared application state: the relay core plus this shard's config.
#[derive(Clone)]
pub struct AppState {
    relay: Arc<Mutex<Relay>>,
    config: Arc<RelayServerConfig>,
    next_sub: Arc<AtomicU64>,
}

impl AppState {
    /// Build state with a fresh relay core from `config`.
    pub fn new(config: RelayServerConfig) -> Self {
        let relay = Relay::new(config.core.clone());
        AppState {
            relay: Arc::new(Mutex::new(relay)),
            config: Arc::new(config),
            next_sub: Arc::new(AtomicU64::new(0)),
        }
    }

    /// This shard's advertised config.
    pub fn config(&self) -> &RelayServerConfig {
        &self.config
    }

    /// A handle to the shared relay core, for the background reconciler.
    pub fn relay_handle(&self) -> Arc<Mutex<Relay>> {
        Arc::clone(&self.relay)
    }
}

/// Build the HTTP/2 router for the relay transport.
///
/// @spec projects/relay/tech-design/interfaces/rest/http-2-openapi-transport-client-side-sharding-streaming-subscrib.md#logic
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/v1/{subject}/publish", post(publish))
        .route("/v1/{subject}/lease", post(lease))
        .route("/v1/{subject}/ack", post(ack))
        .route("/v1/{subject}/heartbeat", post(heartbeat))
        .route("/v1/{subject}/subscribe", get(subscribe))
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
    body: Bytes,
) -> Response {
    let req: PublishRequest = match decode_body(false, &body) {
        Ok(r) => r,
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };
    let now = Utc::now();
    let result = {
        let mut relay = st.relay.lock().expect("relay mutex");
        relay.publish(&subject, &req.message_id, req.payload, req.headers, now)
    };
    match result {
        Ok(outcome) => encode_body(false, StatusCode::OK, &outcome),
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
    let lease = {
        let mut relay = st.relay.lock().expect("relay mutex");
        relay.lease(&subject, &req.consumer_id, now).unwrap_or(None)
    };
    encode_body(cbor, StatusCode::OK, &LeaseResponse { lease })
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
    let (acked, committed_seq) = {
        let mut relay = st.relay.lock().expect("relay mutex");
        let acked = relay
            .ack(&subject, &req.lease_id, req.epoch)
            .unwrap_or(false);
        let committed = relay
            .committed_offset(&subject)
            .ok()
            .flatten()
            .map(|c| c.committed_seq);
        (acked, committed)
    };
    encode_body(
        cbor,
        StatusCode::OK,
        &AckResponse {
            acked,
            committed_seq,
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
    let expires_at = {
        let mut relay = st.relay.lock().expect("relay mutex");
        relay
            .heartbeat(&subject, &req.lease_id, req.epoch, now)
            .unwrap_or(None)
    };
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

    {
        let mut relay = st.relay.lock().expect("relay mutex");
        if let Err(e) = relay.subscribe(&subject, &subscriber_id, query.from_seq) {
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    }

    let stream = futures::stream::unfold(
        (st, subject, subscriber_id),
        |(st, subject, subscriber_id)| async move {
            loop {
                let frames = {
                    let mut relay = st.relay.lock().expect("relay mutex");
                    relay.poll(&subject, &subscriber_id).unwrap_or_default()
                };
                if !frames.is_empty() {
                    let mut buf = Vec::new();
                    for entry in &frames {
                        buf.extend(wire::encode_frame(entry));
                    }
                    let item: Result<Bytes, std::convert::Infallible> = Ok(Bytes::from(buf));
                    return Some((item, (st, subject, subscriber_id)));
                }
                tokio::time::sleep(Duration::from_millis(25)).await;
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
