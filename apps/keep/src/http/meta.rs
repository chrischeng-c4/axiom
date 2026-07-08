//! Cross-cutting routes: TTL / expiry on any key (`/kv/{key}/...`) and list
//! read ops (`/lists/{key}`).

use std::time::Duration;

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::http::error::ApiErr;
use crate::http::handlers::{ack_durable, key_of, propose_write};
use crate::http::models::kv_to_json;
use crate::http::AppState;
use crate::persistence::format::WalOp;

// ---------------------------------------------------------------------------
// expiry
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExpireRequest {
    /// TTL in seconds (EXPIRE). Ignored if `ms` is set.
    #[serde(default)]
    pub seconds: Option<i64>,
    /// TTL in milliseconds (PEXPIRE). Takes precedence over `seconds`.
    #[serde(default)]
    pub ms: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AppliedResponse {
    /// True if the key existed and the change was applied.
    pub applied: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TtlResponse {
    /// Seconds to live: -2 = key absent, -1 = no expiry, else remaining seconds.
    pub ttl_secs: i64,
    /// Milliseconds to live, same -2 / -1 convention.
    pub ttl_ms: i64,
}

/// Set a key's TTL (EXPIRE / PEXPIRE).
#[utoipa::path(post, path = "/kv/{key}/expire", tag = "Expiry",
    params(("key" = String, Path, description = "Key")), request_body = ExpireRequest,
    responses((status = 200, description = "Whether applied", body = AppliedResponse)))]
pub async fn expire(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<ExpireRequest>,
) -> Result<Json<AppliedResponse>, ApiErr> {
    let k = key_of(&key)?;
    let applied = st.engine.exists(&k);
    let proposed = match req.ms {
        Some(ms) => {
            propose_write(
                &st,
                &k,
                WalOp::PExpire {
                    key: k.as_str().to_string(),
                    milliseconds: ms,
                },
            )
            .await?
        }
        None => {
            propose_write(
                &st,
                &k,
                WalOp::Expire {
                    key: k.as_str().to_string(),
                    seconds: req.seconds.unwrap_or(0),
                },
            )
            .await?
        }
    };
    let applied = if proposed {
        applied
    } else {
        match req.ms {
            Some(ms) => st.engine.pexpire(&k, ms) == 1,
            None => st.engine.expire(&k, req.seconds.unwrap_or(0)) == 1,
        }
    };
    ack_durable(&st).await;
    Ok(Json(AppliedResponse { applied }))
}

/// Remaining time-to-live (TTL / PTTL).
#[utoipa::path(get, path = "/kv/{key}/ttl", tag = "Expiry",
    params(("key" = String, Path, description = "Key")),
    responses((status = 200, description = "TTL", body = TtlResponse)))]
pub async fn ttl(
    State(st): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<TtlResponse>, ApiErr> {
    let k = key_of(&key)?;
    Ok(Json(TtlResponse {
        ttl_secs: st.engine.ttl(&k),
        ttl_ms: st.engine.pttl(&k),
    }))
}

/// Remove a key's expiry, making it persistent (PERSIST).
#[utoipa::path(post, path = "/kv/{key}/persist", tag = "Expiry",
    params(("key" = String, Path, description = "Key")),
    responses((status = 200, description = "Whether an expiry was removed", body = AppliedResponse)))]
pub async fn persist(
    State(st): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<AppliedResponse>, ApiErr> {
    let k = key_of(&key)?;
    let applied = st.engine.pttl(&k) >= 0;
    let proposed = propose_write(
        &st,
        &k,
        WalOp::Persist {
            key: k.as_str().to_string(),
        },
    )
    .await?;
    let applied = if proposed {
        applied
    } else {
        st.engine.persist(&k) == 1
    };
    ack_durable(&st).await;
    Ok(Json(AppliedResponse { applied }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct GetExRequest {
    /// New TTL in milliseconds. Omit (with `persist=false`) for a plain read.
    #[serde(default)]
    pub ttl_ms: Option<u64>,
    /// Remove any existing TTL (ignored if `ttl_ms` is set).
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GetExResponse {
    /// Current value, or null if the key is absent.
    pub value: Option<serde_json::Value>,
}

/// Get a value and atomically adjust its TTL (GETEX). With neither `ttl_ms` nor
/// `persist` it is a plain read; otherwise the TTL change is durable-before-ack.
#[utoipa::path(post, path = "/kv/{key}/getex", tag = "Expiry",
    params(("key" = String, Path, description = "Key")), request_body = GetExRequest,
    responses((status = 200, description = "Value", body = GetExResponse)))]
pub async fn getex(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<GetExRequest>,
) -> Result<Json<GetExResponse>, ApiErr> {
    let k = key_of(&key)?;
    let ttl = req.ttl_ms.map(Duration::from_millis);
    let mutates = ttl.is_some() || req.persist;
    let value = st.engine.get(&k).map(kv_to_json);
    if mutates {
        if !propose_write(
            &st,
            &k,
            WalOp::GetEx {
                key: k.as_str().to_string(),
                ttl_ms: req.ttl_ms,
                persist: req.persist,
            },
        )
        .await?
        {
            let value = st.engine.getex(&k, ttl, req.persist).map(kv_to_json);
            ack_durable(&st).await;
            return Ok(Json(GetExResponse { value }));
        }
        ack_durable(&st).await;
    } else {
        return Ok(Json(GetExResponse { value }));
    }
    Ok(Json(GetExResponse { value }))
}

// ---------------------------------------------------------------------------
// list reads
// ---------------------------------------------------------------------------

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

#[derive(Debug, Serialize, ToSchema)]
pub struct ListRangeResponse {
    pub values: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LenResponse {
    pub length: usize,
}

/// Elements in a list range (LRANGE). Negative indices count from the end.
#[utoipa::path(get, path = "/lists/{key}", tag = "Lists",
    params(("key" = String, Path, description = "List key"),
        ("start" = Option<i64>, Query, description = "Start index (default 0)"),
        ("stop" = Option<i64>, Query, description = "Stop index (default -1)")),
    responses((status = 200, description = "Elements", body = ListRangeResponse)))]
pub async fn lrange(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Query(q): Query<RangeQuery>,
) -> Result<Json<ListRangeResponse>, ApiErr> {
    let k = key_of(&key)?;
    let values = st
        .engine
        .lrange(&k, q.start, q.stop)
        .map_err(ApiErr::from)?
        .into_iter()
        .map(kv_to_json)
        .collect();
    Ok(Json(ListRangeResponse { values }))
}

/// List length (LLEN).
#[utoipa::path(get, path = "/lists/{key}/length", tag = "Lists",
    params(("key" = String, Path, description = "List key")),
    responses((status = 200, description = "Length", body = LenResponse)))]
pub async fn llen(
    State(st): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<LenResponse>, ApiErr> {
    let k = key_of(&key)?;
    Ok(Json(LenResponse {
        length: st.engine.llen(&k).map_err(ApiErr::from)?,
    }))
}
