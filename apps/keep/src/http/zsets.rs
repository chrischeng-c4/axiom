//! Sorted-set routes under `/zsets/{key}` (string members, f64 scores).
//! WAL-backed and durable-before-ack.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::http::error::ApiErr;
use crate::http::handlers::{ack_durable, key_of, propose_write};
use crate::http::models::CountResponse;
use crate::http::AppState;
use crate::persistence::format::WalOp;

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
#[utoipa::path(post, path = "/zsets/{key}", tag = "SortedSets",
    params(("key" = String, Path, description = "Sorted-set key")), request_body = ZAddRequest,
    responses((status = 200, description = "Members added", body = CountResponse)))]
pub async fn zadd(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<ZAddRequest>,
) -> Result<Json<CountResponse>, ApiErr> {
    let k = key_of(&key)?;
    let members: Vec<(String, f64)> = req
        .members
        .into_iter()
        .map(|m| (m.member, m.score))
        .collect();
    let existing = members
        .iter()
        .filter(|(member, _)| st.engine.zscore(&k, member).ok().flatten().is_some())
        .count();
    let count = if propose_write(
        &st,
        &k,
        WalOp::ZAdd {
            key: k.as_str().to_string(),
            members: members.clone(),
        },
    )
    .await?
    {
        members.len().saturating_sub(existing)
    } else {
        st.engine.zadd(&k, members).map_err(ApiErr::from)?
    };
    ack_durable(&st).await;
    Ok(Json(CountResponse { count }))
}

/// Members in a rank range, ascending by score (ZRANGE). Negative indices count
/// from the end (`stop` defaults to -1 = last).
#[utoipa::path(get, path = "/zsets/{key}", tag = "SortedSets",
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
#[utoipa::path(delete, path = "/zsets/{key}", tag = "SortedSets",
    params(("key" = String, Path, description = "Sorted-set key")), request_body = ZRemRequest,
    responses((status = 200, description = "Members removed", body = CountResponse)))]
pub async fn zrem(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<ZRemRequest>,
) -> Result<Json<CountResponse>, ApiErr> {
    let k = key_of(&key)?;
    let existing = req
        .members
        .iter()
        .filter(|member| st.engine.zscore(&k, member).ok().flatten().is_some())
        .count();
    let count = if propose_write(
        &st,
        &k,
        WalOp::ZRem {
            key: k.as_str().to_string(),
            members: req.members.clone(),
        },
    )
    .await?
    {
        existing
    } else {
        st.engine.zrem(&k, req.members).map_err(ApiErr::from)?
    };
    ack_durable(&st).await;
    Ok(Json(CountResponse { count }))
}

/// Cardinality (ZCARD).
#[utoipa::path(get, path = "/zsets/{key}/length", tag = "SortedSets",
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
#[utoipa::path(post, path = "/zsets/{key}/incr", tag = "SortedSets",
    params(("key" = String, Path, description = "Sorted-set key")), request_body = ZIncrRequest,
    responses((status = 200, description = "New score", body = FloatValueResponse)))]
pub async fn zincr(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<ZIncrRequest>,
) -> Result<Json<FloatValueResponse>, ApiErr> {
    let k = key_of(&key)?;
    let value = if propose_write(
        &st,
        &k,
        WalOp::ZIncrBy {
            key: k.as_str().to_string(),
            member: req.member.clone(),
            delta: req.delta,
        },
    )
    .await?
    {
        st.engine
            .zscore(&k, &req.member)
            .map_err(ApiErr::from)?
            .unwrap_or(req.delta)
    } else {
        st.engine
            .zincrby(&k, &req.member, req.delta)
            .map_err(ApiErr::from)?
    };
    ack_durable(&st).await;
    Ok(Json(FloatValueResponse { value }))
}

/// A member's score (ZSCORE).
#[utoipa::path(get, path = "/zsets/{key}/members/{member}/score", tag = "SortedSets",
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
#[utoipa::path(get, path = "/zsets/{key}/members/{member}/rank", tag = "SortedSets",
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
