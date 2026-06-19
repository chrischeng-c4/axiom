//! Axum handlers. One function per route; each is annotated with
//! `#[utoipa::path]` so the generated OpenAPI document stays in lock-step with
//! the router.

use std::time::Duration;

use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{header::CONTENT_TYPE, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    Json,
};
use serde::Deserialize;

use crate::http::error::ApiErr;
use crate::http::models::*;
use crate::http::AppState;
use crate::types::{KvKey, KvValue};

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn key_of(s: &str) -> Result<KvKey, ApiErr> {
    KvKey::new(s).map_err(ApiErr::from)
}

fn ttl(ms: Option<u64>) -> Option<Duration> {
    ms.map(Duration::from_millis)
}

/// Await durability of the writes issued so far, then return — the basis for
/// durable-before-ack. No-op when persistence is disabled. Concurrent writers
/// share one fsync (group commit), so this is cheap under load.
async fn ack_durable(st: &AppState) {
    if let Some(rx) = st.engine.durability_barrier() {
        let _ = rx.await;
    }
}

/// TTL passed via query string on the raw-bytes write path.
#[derive(Debug, Deserialize)]
pub struct TtlQuery {
    pub ttl_ms: Option<u64>,
}

// ---------------------------------------------------------------------------
// single-key
// ---------------------------------------------------------------------------

/// Fetch a value. Byte blobs are returned as `application/octet-stream`;
/// everything else as a JSON `ValueResponse`.
#[utoipa::path(
    get,
    path = "/v1/kv/{key}",
    tag = "KV",
    params(("key" = String, Path, description = "Key to fetch")),
    responses(
        (status = 200, description = "Value found", body = ValueResponse),
        (status = 404, description = "Key not found", body = crate::http::error::ApiError)
    )
)]
pub async fn get_key(
    State(st): State<AppState>,
    Path(key): Path<String>,
) -> Result<Response, ApiErr> {
    let k = key_of(&key)?;
    match st.engine.get(&k) {
        None => Err(ApiErr::new(
            StatusCode::NOT_FOUND,
            "key_not_found",
            format!("key not found: {key}"),
        )),
        Some(KvValue::Bytes(b)) => {
            Ok(([(CONTENT_TYPE, "application/octet-stream")], b).into_response())
        }
        Some(v) => Ok(Json(ValueResponse {
            key,
            value: kv_to_json(v),
        })
        .into_response()),
    }
}

/// Store a value. `application/json` bodies are a `SetRequest`; an
/// `application/octet-stream` body is stored verbatim as a byte blob (TTL via
/// `?ttl_ms=`), which is the efficient path for claim-check payloads.
#[utoipa::path(
    put,
    path = "/v1/kv/{key}",
    tag = "KV",
    params(("key" = String, Path, description = "Key to set")),
    request_body(content = SetRequest, description = "JSON value + optional ttl_ms (or send a raw octet-stream blob)"),
    responses(
        (status = 200, description = "Stored", body = OkResponse),
        (status = 400, description = "Invalid key or body", body = crate::http::error::ApiError),
        (status = 415, description = "Unsupported content-type", body = crate::http::error::ApiError),
        (status = 507, description = "Out of memory", body = crate::http::error::ApiError)
    )
)]
pub async fn put_key(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Query(q): Query<TtlQuery>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, ApiErr> {
    let k = key_of(&key)?;
    let ct = headers
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/json");

    if ct.starts_with("application/octet-stream") {
        st.engine
            .set(&k, KvValue::Bytes(body.to_vec()), ttl(q.ttl_ms))
            .map_err(ApiErr::from)?;
    } else if ct.starts_with("application/json") {
        // Fast parse: JSON tokens -> KvValue directly, no serde_json::Value tree
        // (the measured write-path hot cost). SetRequest stays the OpenAPI schema.
        let req: SetRequestFast = serde_json::from_slice(&body)
            .map_err(|e| ApiErr::bad_request(format!("invalid JSON body: {e}")))?;
        st.engine
            .set(&k, req.value.0, ttl(req.ttl_ms))
            .map_err(ApiErr::from)?;
    } else {
        return Err(ApiErr::unsupported_media_type(format!(
            "unsupported content-type: {ct}"
        )));
    }
    ack_durable(&st).await;
    Ok((StatusCode::OK, Json(OkResponse { key, ok: true })).into_response())
}

/// Delete a key.
#[utoipa::path(
    delete,
    path = "/v1/kv/{key}",
    tag = "KV",
    params(("key" = String, Path, description = "Key to delete")),
    responses((status = 200, description = "Delete result", body = DeleteResponse))
)]
pub async fn delete_key(
    State(st): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<DeleteResponse>, ApiErr> {
    let k = key_of(&key)?;
    let deleted = st.engine.delete(&k);
    ack_durable(&st).await;
    Ok(Json(DeleteResponse { deleted }))
}

/// Existence check (no body).
#[utoipa::path(
    head,
    path = "/v1/kv/{key}",
    tag = "KV",
    params(("key" = String, Path, description = "Key to test")),
    responses(
        (status = 200, description = "Key exists"),
        (status = 404, description = "Key absent")
    )
)]
pub async fn head_key(State(st): State<AppState>, Path(key): Path<String>) -> StatusCode {
    match key_of(&key) {
        Ok(k) if st.engine.exists(&k) => StatusCode::OK,
        Ok(_) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::BAD_REQUEST,
    }
}

/// Atomic increment/decrement of an integer value.
#[utoipa::path(
    post,
    path = "/v1/kv/{key}/incr",
    tag = "KV",
    params(("key" = String, Path, description = "Counter key")),
    request_body = IncrRequest,
    responses(
        (status = 200, description = "New value", body = IncrResponse),
        (status = 409, description = "Existing value is not an integer", body = crate::http::error::ApiError)
    )
)]
pub async fn incr_key(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<IncrRequest>,
) -> Result<Json<IncrResponse>, ApiErr> {
    let k = key_of(&key)?;
    let value = st.engine.incr(&k, req.delta).map_err(ApiErr::from)?;
    ack_durable(&st).await;
    Ok(Json(IncrResponse { value }))
}

/// Compare-and-swap.
#[utoipa::path(
    post,
    path = "/v1/kv/{key}/cas",
    tag = "KV",
    params(("key" = String, Path, description = "Key to swap")),
    request_body = CasRequest,
    responses((status = 200, description = "Swap result", body = CasResponse))
)]
pub async fn cas_key(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<CasRequest>,
) -> Result<Json<CasResponse>, ApiErr> {
    let k = key_of(&key)?;
    let expected = json_to_kv(req.expected);
    let swapped = st
        .engine
        .cas(&k, &expected, json_to_kv(req.new), ttl(req.ttl_ms))
        .map_err(ApiErr::from)?;
    ack_durable(&st).await;
    Ok(Json(CasResponse { swapped }))
}

/// Set only if the key does not already exist.
#[utoipa::path(
    post,
    path = "/v1/kv/{key}/setnx",
    tag = "KV",
    params(("key" = String, Path, description = "Key to set")),
    request_body = SetRequest,
    responses((status = 200, description = "Set result", body = SetNxResponse))
)]
pub async fn setnx_key(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<SetRequest>,
) -> Result<Json<SetNxResponse>, ApiErr> {
    let k = key_of(&key)?;
    let set = st
        .engine
        .setnx(&k, json_to_kv(req.value), ttl(req.ttl_ms))
        .map_err(ApiErr::from)?;
    ack_durable(&st).await;
    Ok(Json(SetNxResponse { set }))
}

// ---------------------------------------------------------------------------
// batch
// ---------------------------------------------------------------------------

/// Fetch many keys at once.
#[utoipa::path(
    post,
    path = "/v1/kv:mget",
    tag = "Batch",
    request_body = MGetRequest,
    responses((status = 200, description = "Values parallel to the request keys", body = MGetResponse))
)]
pub async fn mget(
    State(st): State<AppState>,
    Json(req): Json<MGetRequest>,
) -> Result<Json<MGetResponse>, ApiErr> {
    let keys: Vec<KvKey> = req
        .keys
        .iter()
        .map(|s| KvKey::new(s))
        .collect::<Result<_, _>>()
        .map_err(ApiErr::from)?;
    let refs: Vec<&KvKey> = keys.iter().collect();
    let values = st
        .engine
        .mget(&refs)
        .into_iter()
        .map(|o| o.map(kv_to_json))
        .collect();
    Ok(Json(MGetResponse { values }))
}

/// Set many keys at once with a shared TTL.
#[utoipa::path(
    post,
    path = "/v1/kv:mset",
    tag = "Batch",
    request_body = MSetRequest,
    responses((status = 200, description = "Number of keys written", body = CountResponse))
)]
pub async fn mset(
    State(st): State<AppState>,
    body: Bytes,
) -> Result<Json<CountResponse>, ApiErr> {
    // Fast parse: entry values -> KvValue directly (no serde_json::Value each).
    let req: MSetRequestFast = serde_json::from_slice(&body)
        .map_err(|e| ApiErr::bad_request(format!("invalid JSON body: {e}")))?;
    let mut keys = Vec::with_capacity(req.entries.len());
    let mut vals = Vec::with_capacity(req.entries.len());
    for (k, v) in req.entries {
        keys.push(KvKey::new(&k).map_err(ApiErr::from)?);
        vals.push(v.0);
    }
    let pairs: Vec<(&KvKey, KvValue)> = keys.iter().zip(vals).collect();
    st.engine
        .mset(&pairs, ttl(req.ttl_ms))
        .map_err(ApiErr::from)?;
    ack_durable(&st).await;
    Ok(Json(CountResponse { count: pairs.len() }))
}

/// Delete many keys at once.
#[utoipa::path(
    post,
    path = "/v1/kv:mdel",
    tag = "Batch",
    request_body = MDelRequest,
    responses((status = 200, description = "Number of keys deleted", body = CountResponse))
)]
pub async fn mdel(
    State(st): State<AppState>,
    Json(req): Json<MDelRequest>,
) -> Result<Json<CountResponse>, ApiErr> {
    let keys: Vec<KvKey> = req
        .keys
        .iter()
        .map(|s| KvKey::new(s))
        .collect::<Result<_, _>>()
        .map_err(ApiErr::from)?;
    let refs: Vec<&KvKey> = keys.iter().collect();
    let count = st.engine.mdel(&refs);
    ack_durable(&st).await;
    Ok(Json(CountResponse { count }))
}

/// List keys, optionally filtered by prefix.
#[utoipa::path(
    get,
    path = "/v1/kv",
    tag = "KV",
    params(
        ("prefix" = Option<String>, Query, description = "Only keys with this prefix"),
        ("limit" = Option<usize>, Query, description = "Max keys to return (default 100)")
    ),
    responses((status = 200, description = "Matching keys", body = ScanResponse))
)]
pub async fn scan(
    State(st): State<AppState>,
    Query(q): Query<ScanQuery>,
) -> Json<ScanResponse> {
    Json(ScanResponse {
        keys: st.engine.scan(q.prefix.as_deref(), q.limit),
    })
}

// ---------------------------------------------------------------------------
// locks
// ---------------------------------------------------------------------------

/// Acquire a leased lock. Idempotent for the same owner.
#[utoipa::path(
    post,
    path = "/v1/locks/{key}",
    tag = "Locks",
    params(("key" = String, Path, description = "Lock name")),
    request_body = LockRequest,
    responses((status = 200, description = "Acquire result", body = LockResponse))
)]
pub async fn lock(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<LockRequest>,
) -> Result<Json<LockResponse>, ApiErr> {
    let k = key_of(&key)?;
    let acquired = st
        .engine
        .lock(&k, &req.owner, Duration::from_millis(req.ttl_ms));
    Ok(Json(LockResponse { acquired }))
}

/// Release a lock held by `owner`.
#[utoipa::path(
    delete,
    path = "/v1/locks/{key}",
    tag = "Locks",
    params(("key" = String, Path, description = "Lock name")),
    request_body = UnlockRequest,
    responses(
        (status = 200, description = "Release result", body = UnlockResponse),
        (status = 409, description = "Lock held by a different owner", body = crate::http::error::ApiError)
    )
)]
pub async fn unlock(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<UnlockRequest>,
) -> Result<Json<UnlockResponse>, ApiErr> {
    let k = key_of(&key)?;
    let released = st.engine.unlock(&k, &req.owner).map_err(ApiErr::from)?;
    Ok(Json(UnlockResponse { released }))
}

/// Extend the lease on a held lock.
#[utoipa::path(
    patch,
    path = "/v1/locks/{key}",
    tag = "Locks",
    params(("key" = String, Path, description = "Lock name")),
    request_body = ExtendLockRequest,
    responses(
        (status = 200, description = "Extend result", body = ExtendLockResponse),
        (status = 409, description = "Lock not held or owner mismatch", body = crate::http::error::ApiError)
    )
)]
pub async fn extend_lock(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<ExtendLockRequest>,
) -> Result<Json<ExtendLockResponse>, ApiErr> {
    let k = key_of(&key)?;
    let extended = st
        .engine
        .extend_lock(&k, &req.owner, Duration::from_millis(req.ttl_ms))
        .map_err(ApiErr::from)?;
    Ok(Json(ExtendLockResponse { extended }))
}

// ---------------------------------------------------------------------------
// lists
// ---------------------------------------------------------------------------

/// Prepend values to a list (LPUSH).
#[utoipa::path(
    post,
    path = "/v1/lists/{key}/lpush",
    tag = "Lists",
    params(("key" = String, Path, description = "List key")),
    request_body = PushRequest,
    responses((status = 200, description = "New list length", body = PushResponse))
)]
pub async fn lpush(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<PushRequest>,
) -> Result<Json<PushResponse>, ApiErr> {
    let k = key_of(&key)?;
    let values = req.values.into_iter().map(json_to_kv).collect();
    let length = st.engine.lpush(&k, values).map_err(ApiErr::from)?;
    Ok(Json(PushResponse { length }))
}

/// Append values to a list (RPUSH).
#[utoipa::path(
    post,
    path = "/v1/lists/{key}/rpush",
    tag = "Lists",
    params(("key" = String, Path, description = "List key")),
    request_body = PushRequest,
    responses((status = 200, description = "New list length", body = PushResponse))
)]
pub async fn rpush(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<PushRequest>,
) -> Result<Json<PushResponse>, ApiErr> {
    let k = key_of(&key)?;
    let values = req.values.into_iter().map(json_to_kv).collect();
    let length = st.engine.rpush(&k, values).map_err(ApiErr::from)?;
    Ok(Json(PushResponse { length }))
}

/// Pop from the head of a list (LPOP).
#[utoipa::path(
    post,
    path = "/v1/lists/{key}/lpop",
    tag = "Lists",
    params(("key" = String, Path, description = "List key")),
    responses((status = 200, description = "Popped value or null", body = PopResponse))
)]
pub async fn lpop(
    State(st): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<PopResponse>, ApiErr> {
    let k = key_of(&key)?;
    Ok(Json(PopResponse {
        value: st.engine.lpop(&k).map(kv_to_json),
    }))
}

/// Pop from the tail of a list (RPOP).
#[utoipa::path(
    post,
    path = "/v1/lists/{key}/rpop",
    tag = "Lists",
    params(("key" = String, Path, description = "List key")),
    responses((status = 200, description = "Popped value or null", body = PopResponse))
)]
pub async fn rpop(
    State(st): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<PopResponse>, ApiErr> {
    let k = key_of(&key)?;
    Ok(Json(PopResponse {
        value: st.engine.rpop(&k).map(kv_to_json),
    }))
}

// ---------------------------------------------------------------------------
// admin / probes
// ---------------------------------------------------------------------------

/// Liveness probe — process is up. Always 200 unless the process is wedged.
#[utoipa::path(
    get,
    path = "/healthz",
    tag = "Admin",
    responses((status = 200, description = "Alive", body = String))
)]
pub async fn healthz() -> &'static str {
    "ok"
}

/// Readiness probe — 200 when serving, 503 while draining on SIGTERM so k8s
/// stops routing traffic before the pod exits.
#[utoipa::path(
    get,
    path = "/readyz",
    tag = "Admin",
    responses(
        (status = 200, description = "Ready"),
        (status = 503, description = "Draining / not ready")
    )
)]
pub async fn readyz(State(st): State<AppState>) -> (StatusCode, &'static str) {
    if st.is_draining() {
        (StatusCode::SERVICE_UNAVAILABLE, "draining")
    } else {
        (StatusCode::OK, "ok")
    }
}

/// Prometheus text-format metrics.
#[utoipa::path(
    get,
    path = "/metrics",
    tag = "Admin",
    responses((status = 200, description = "Prometheus exposition", body = String))
)]
pub async fn metrics(
    State(st): State<AppState>,
) -> (StatusCode, [(&'static str, &'static str); 1], String) {
    let keys = st.engine.len();
    let shards = st.engine.num_shards();
    let mem = st.engine.estimate_memory();
    let body = format!(
        "# HELP keep_keys_total Number of keys across all shards.\n\
         # TYPE keep_keys_total gauge\n\
         keep_keys_total {keys}\n\
         # HELP keep_shards Number of engine shards.\n\
         # TYPE keep_shards gauge\n\
         keep_shards {shards}\n\
         # HELP keep_memory_bytes Estimated resident bytes of stored data.\n\
         # TYPE keep_memory_bytes gauge\n\
         keep_memory_bytes {mem}\n"
    );
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4")],
        body,
    )
}

/// Server info snapshot.
#[utoipa::path(
    get,
    path = "/info",
    tag = "Admin",
    responses((status = 200, description = "Server info", body = InfoResponse))
)]
pub async fn info(State(st): State<AppState>) -> Json<InfoResponse> {
    Json(InfoResponse {
        keys: st.engine.len(),
        shards: st.engine.num_shards(),
        memory_bytes: st.engine.estimate_memory(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// The generated OpenAPI document.
#[utoipa::path(get, path = "/openapi.json", tag = "Admin", responses((status = 200, description = "OpenAPI 3 document")))]
pub async fn openapi_spec() -> Json<utoipa::openapi::OpenApi> {
    use utoipa::OpenApi;
    Json(crate::http::openapi::ApiDoc::openapi())
}

/// Minimal API docs page (Redoc, loads `/openapi.json`).
pub async fn docs() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html>
  <head>
    <title>keep API</title>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1"/>
  </head>
  <body>
    <redoc spec-url="/openapi.json"></redoc>
    <script src="https://cdn.redoc.ly/redoc/latest/bundles/redoc.standalone.js"></script>
  </body>
</html>"#,
    )
}
