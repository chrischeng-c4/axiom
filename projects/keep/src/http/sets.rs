//! Set routes under `/v1/sets/{key}` (string members). WAL-backed and
//! durable-before-ack.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::http::error::ApiErr;
use crate::http::handlers::{ack_durable, key_of};
use crate::http::models::CountResponse;
use crate::http::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct MembersRequest {
    pub members: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MembersResponse {
    pub members: Vec<String>,
}

/// Add members (SADD). Returns the number newly added.
#[utoipa::path(post, path = "/v1/sets/{key}", tag = "Sets",
    params(("key" = String, Path, description = "Set key")), request_body = MembersRequest,
    responses((status = 200, description = "Members added", body = CountResponse)))]
pub async fn sadd(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<MembersRequest>,
) -> Result<Json<CountResponse>, ApiErr> {
    let k = key_of(&key)?;
    let count = st.engine.sadd(&k, req.members).map_err(ApiErr::from)?;
    ack_durable(&st).await;
    Ok(Json(CountResponse { count }))
}

/// All members (SMEMBERS).
#[utoipa::path(get, path = "/v1/sets/{key}", tag = "Sets",
    params(("key" = String, Path, description = "Set key")),
    responses((status = 200, description = "Members", body = MembersResponse)))]
pub async fn smembers(
    State(st): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<MembersResponse>, ApiErr> {
    let k = key_of(&key)?;
    Ok(Json(MembersResponse {
        members: st.engine.smembers(&k).map_err(ApiErr::from)?,
    }))
}

/// Remove members (SREM). Returns the number removed.
#[utoipa::path(delete, path = "/v1/sets/{key}", tag = "Sets",
    params(("key" = String, Path, description = "Set key")), request_body = MembersRequest,
    responses((status = 200, description = "Members removed", body = CountResponse)))]
pub async fn srem(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<MembersRequest>,
) -> Result<Json<CountResponse>, ApiErr> {
    let k = key_of(&key)?;
    let count = st.engine.srem(&k, req.members).map_err(ApiErr::from)?;
    ack_durable(&st).await;
    Ok(Json(CountResponse { count }))
}

/// Cardinality (SCARD).
#[utoipa::path(get, path = "/v1/sets/{key}/length", tag = "Sets",
    params(("key" = String, Path, description = "Set key")),
    responses((status = 200, description = "Member count", body = CountResponse)))]
pub async fn scard(
    State(st): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<CountResponse>, ApiErr> {
    let k = key_of(&key)?;
    Ok(Json(CountResponse {
        count: st.engine.scard(&k).map_err(ApiErr::from)?,
    }))
}

/// Membership test (SISMEMBER), no body.
#[utoipa::path(head, path = "/v1/sets/{key}/members/{member}", tag = "Sets",
    params(("key" = String, Path, description = "Set key"), ("member" = String, Path, description = "Member")),
    responses((status = 200, description = "Is a member"), (status = 404, description = "Not a member")))]
pub async fn sismember(
    State(st): State<AppState>,
    Path((key, member)): Path<(String, String)>,
) -> StatusCode {
    match key_of(&key) {
        Ok(k) => match st.engine.sismember(&k, &member) {
            Ok(true) => StatusCode::OK,
            Ok(false) => StatusCode::NOT_FOUND,
            Err(_) => StatusCode::CONFLICT,
        },
        Err(_) => StatusCode::BAD_REQUEST,
    }
}
