//! Sorted-set routes under `/v1/zsets/{key}` (string members, f64 scores).
//! In-memory only (see the hash module note on collection durability).

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::http::error::ApiErr;
use crate::http::handlers::key_of;
use crate::http::models::CountResponse;
use crate::http::AppState;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ScoredMember {
    pub member: String,
    pub score: f64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ZAddRequest {
    pub members: Vec<ScoredMember>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ZRemRequest {
    pub members: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ZRangeResponse {
    /// Members in ascending score order.
    pub entries: Vec<ScoredMember>,
}

#[derive(Debug, Deserialize)]
pub struct RangeQuery {
    #[serde(default)]
    pub start: i64,
    #[serde(default = "neg_one")]
    pub stop: i64,
}
fn neg_one() -> i64 {
    -1
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ZIncrRequest {
    pub member: String,
    pub delta: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ScoreResponse {
    /// Member score, or null if absent.
    pub score: Option<f64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FloatValueResponse {
    pub value: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RankResponse {
    /// 0-based ascending rank, or null if absent.
    pub rank: Option<usize>,
}

/// Add scored members (ZADD). Returns the number newly added.
#[utoipa::path(post, path = "/v1/zsets/{key}", tag = "SortedSets",
    params(("key" = String, Path, description = "Sorted-set key")), request_body = ZAddRequest,
    responses((status = 200, description = "Members added", body = CountResponse)))]
pub async fn zadd(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<ZAddRequest>,
) -> Result<Json<CountResponse>, ApiErr> {
    let k = key_of(&key)?;
    let members: Vec<(String, f64)> = req.members.into_iter().map(|m| (m.member, m.score)).collect();
    let count = st.engine.zadd(&k, members).map_err(ApiErr::from)?;
    Ok(Json(CountResponse { count }))
}

/// Members in a rank range, ascending by score (ZRANGE). Negative indices count
/// from the end (`stop` defaults to -1 = last).
#[utoipa::path(get, path = "/v1/zsets/{key}", tag = "SortedSets",
    params(("key" = String, Path, description = "Sorted-set key"),
        ("start" = Option<i64>, Query, description = "Start rank (default 0)"),
        ("stop" = Option<i64>, Query, description = "Stop rank (default -1)")),
    responses((status = 200, description = "Ranked members", body = ZRangeResponse)))]
pub async fn zrange(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Query(q): Query<RangeQuery>,
) -> Result<Json<ZRangeResponse>, ApiErr> {
    let k = key_of(&key)?;
    let entries = st
        .engine
        .zrange(&k, q.start, q.stop)
        .map_err(ApiErr::from)?
        .into_iter()
        .map(|(member, score)| ScoredMember { member, score })
        .collect();
    Ok(Json(ZRangeResponse { entries }))
}

/// Remove members (ZREM). Returns the number removed.
#[utoipa::path(delete, path = "/v1/zsets/{key}", tag = "SortedSets",
    params(("key" = String, Path, description = "Sorted-set key")), request_body = ZRemRequest,
    responses((status = 200, description = "Members removed", body = CountResponse)))]
pub async fn zrem(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<ZRemRequest>,
) -> Result<Json<CountResponse>, ApiErr> {
    let k = key_of(&key)?;
    let count = st.engine.zrem(&k, req.members).map_err(ApiErr::from)?;
    Ok(Json(CountResponse { count }))
}

/// Cardinality (ZCARD).
#[utoipa::path(get, path = "/v1/zsets/{key}/length", tag = "SortedSets",
    params(("key" = String, Path, description = "Sorted-set key")),
    responses((status = 200, description = "Member count", body = CountResponse)))]
pub async fn zcard(
    State(st): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<CountResponse>, ApiErr> {
    let k = key_of(&key)?;
    Ok(Json(CountResponse {
        count: st.engine.zcard(&k).map_err(ApiErr::from)?,
    }))
}

/// Increment a member's score (ZINCRBY). Returns the new score.
#[utoipa::path(post, path = "/v1/zsets/{key}/incr", tag = "SortedSets",
    params(("key" = String, Path, description = "Sorted-set key")), request_body = ZIncrRequest,
    responses((status = 200, description = "New score", body = FloatValueResponse)))]
pub async fn zincr(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<ZIncrRequest>,
) -> Result<Json<FloatValueResponse>, ApiErr> {
    let k = key_of(&key)?;
    let value = st
        .engine
        .zincrby(&k, &req.member, req.delta)
        .map_err(ApiErr::from)?;
    Ok(Json(FloatValueResponse { value }))
}

/// A member's score (ZSCORE).
#[utoipa::path(get, path = "/v1/zsets/{key}/members/{member}/score", tag = "SortedSets",
    params(("key" = String, Path, description = "Key"), ("member" = String, Path, description = "Member")),
    responses((status = 200, description = "Score or null", body = ScoreResponse)))]
pub async fn zscore(
    State(st): State<AppState>,
    Path((key, member)): Path<(String, String)>,
) -> Result<Json<ScoreResponse>, ApiErr> {
    let k = key_of(&key)?;
    Ok(Json(ScoreResponse {
        score: st.engine.zscore(&k, &member).map_err(ApiErr::from)?,
    }))
}

/// A member's 0-based ascending rank (ZRANK).
#[utoipa::path(get, path = "/v1/zsets/{key}/members/{member}/rank", tag = "SortedSets",
    params(("key" = String, Path, description = "Key"), ("member" = String, Path, description = "Member")),
    responses((status = 200, description = "Rank or null", body = RankResponse)))]
pub async fn zrank(
    State(st): State<AppState>,
    Path((key, member)): Path<(String, String)>,
) -> Result<Json<RankResponse>, ApiErr> {
    let k = key_of(&key)?;
    Ok(Json(RankResponse {
        rank: st.engine.zrank(&k, &member).map_err(ApiErr::from)?,
    }))
}
