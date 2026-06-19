//! Hash (field-map) routes under `/v1/hashes/{key}`.
//!
//! Note: hash values live in memory only — the WAL covers scalar ops, so hashes
//! are not restored on recovery (a known engine limitation; the durable result
//! path is scalar `/v1/kv`).

use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::http::error::ApiErr;
use crate::http::handlers::key_of;
use crate::http::models::{json_to_kv, kv_to_json, CountResponse};
use crate::http::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct HSetRequest {
    /// Field -> value map to write.
    pub fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HGetAllResponse {
    pub fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HGetResponse {
    /// Field value, or null if the field is absent.
    pub value: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct FieldsRequest {
    pub fields: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HMGetResponse {
    /// Parallel to the requested fields; null where absent.
    pub values: Vec<Option<serde_json::Value>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct HIncrRequest {
    pub field: String,
    /// Amount to add (negative to decrement). Default 1.
    #[serde(default = "one")]
    pub delta: i64,
}
fn one() -> i64 {
    1
}

#[derive(Debug, Serialize, ToSchema)]
pub struct IntValueResponse {
    pub value: i64,
}

/// Set hash fields (HSET). Returns the number of newly-added fields.
#[utoipa::path(post, path = "/v1/hashes/{key}", tag = "Hashes",
    params(("key" = String, Path, description = "Hash key")), request_body = HSetRequest,
    responses((status = 200, description = "Fields added", body = CountResponse)))]
pub async fn hset(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<HSetRequest>,
) -> Result<Json<CountResponse>, ApiErr> {
    let k = key_of(&key)?;
    let fields: Vec<(String, _)> = req
        .fields
        .into_iter()
        .map(|(f, v)| (f, json_to_kv(v)))
        .collect();
    let count = st.engine.hset(&k, fields).map_err(ApiErr::from)?;
    Ok(Json(CountResponse { count }))
}

/// All fields of a hash (HGETALL).
#[utoipa::path(get, path = "/v1/hashes/{key}", tag = "Hashes",
    params(("key" = String, Path, description = "Hash key")),
    responses((status = 200, description = "Field map", body = HGetAllResponse)))]
pub async fn hgetall(
    State(st): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<HGetAllResponse>, ApiErr> {
    let k = key_of(&key)?;
    let fields = st
        .engine
        .hgetall(&k)
        .map_err(ApiErr::from)?
        .into_iter()
        .map(|(f, v)| (f, kv_to_json(v)))
        .collect();
    Ok(Json(HGetAllResponse { fields }))
}

/// Delete hash fields (HDEL). Returns the number removed.
#[utoipa::path(delete, path = "/v1/hashes/{key}", tag = "Hashes",
    params(("key" = String, Path, description = "Hash key")), request_body = FieldsRequest,
    responses((status = 200, description = "Fields removed", body = CountResponse)))]
pub async fn hdel(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<FieldsRequest>,
) -> Result<Json<CountResponse>, ApiErr> {
    let k = key_of(&key)?;
    let refs: Vec<&str> = req.fields.iter().map(String::as_str).collect();
    let count = st.engine.hdel(&k, &refs).map_err(ApiErr::from)?;
    Ok(Json(CountResponse { count }))
}

/// Number of fields in a hash (HLEN).
#[utoipa::path(get, path = "/v1/hashes/{key}/length", tag = "Hashes",
    params(("key" = String, Path, description = "Hash key")),
    responses((status = 200, description = "Field count", body = CountResponse)))]
pub async fn hlen(
    State(st): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<CountResponse>, ApiErr> {
    let k = key_of(&key)?;
    Ok(Json(CountResponse {
        count: st.engine.hlen(&k).map_err(ApiErr::from)?,
    }))
}

/// Fetch several hash fields at once (HMGET).
#[utoipa::path(post, path = "/v1/hashes/{key}/mget", tag = "Hashes",
    params(("key" = String, Path, description = "Hash key")), request_body = FieldsRequest,
    responses((status = 200, description = "Values parallel to fields", body = HMGetResponse)))]
pub async fn hmget(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<FieldsRequest>,
) -> Result<Json<HMGetResponse>, ApiErr> {
    let k = key_of(&key)?;
    let refs: Vec<&str> = req.fields.iter().map(String::as_str).collect();
    let values = st
        .engine
        .hmget(&k, &refs)
        .map_err(ApiErr::from)?
        .into_iter()
        .map(|o| o.map(kv_to_json))
        .collect();
    Ok(Json(HMGetResponse { values }))
}

/// Increment an integer hash field (HINCRBY).
#[utoipa::path(post, path = "/v1/hashes/{key}/incr", tag = "Hashes",
    params(("key" = String, Path, description = "Hash key")), request_body = HIncrRequest,
    responses((status = 200, description = "New value", body = IntValueResponse)))]
pub async fn hincr(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<HIncrRequest>,
) -> Result<Json<IntValueResponse>, ApiErr> {
    let k = key_of(&key)?;
    let value = st
        .engine
        .hincrby(&k, &req.field, req.delta)
        .map_err(ApiErr::from)?;
    Ok(Json(IntValueResponse { value }))
}

/// Single hash field (HGET).
#[utoipa::path(get, path = "/v1/hashes/{key}/fields/{field}", tag = "Hashes",
    params(("key" = String, Path, description = "Hash key"), ("field" = String, Path, description = "Field")),
    responses((status = 200, description = "Field value or null", body = HGetResponse)))]
pub async fn hget(
    State(st): State<AppState>,
    Path((key, field)): Path<(String, String)>,
) -> Result<Json<HGetResponse>, ApiErr> {
    let k = key_of(&key)?;
    let value = st
        .engine
        .hget(&k, &field)
        .map_err(ApiErr::from)?
        .map(kv_to_json);
    Ok(Json(HGetResponse { value }))
}

/// Field existence (HEXISTS), no body.
#[utoipa::path(head, path = "/v1/hashes/{key}/fields/{field}", tag = "Hashes",
    params(("key" = String, Path, description = "Hash key"), ("field" = String, Path, description = "Field")),
    responses((status = 200, description = "Field exists"), (status = 404, description = "Field absent")))]
pub async fn hexists(
    State(st): State<AppState>,
    Path((key, field)): Path<(String, String)>,
) -> StatusCode {
    match key_of(&key) {
        Ok(k) => match st.engine.hexists(&k, &field) {
            Ok(true) => StatusCode::OK,
            Ok(false) => StatusCode::NOT_FOUND,
            Err(_) => StatusCode::CONFLICT,
        },
        Err(_) => StatusCode::BAD_REQUEST,
    }
}
