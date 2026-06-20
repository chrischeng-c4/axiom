//! Blocking list pops (BLPOP / BRPOP) under `/v1/lists/{key}/{blpop,brpop}`.
//!
//! HTTP long-poll: block up to `timeout_ms` for an element, then return it (or
//! null on timeout). The pop itself is durable-before-ack like any list pop.

use std::time::{Duration, Instant};

use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::http::error::ApiErr;
use crate::http::handlers::{ack_durable, key_of};
use crate::http::models::{kv_to_json, PopResponse};
use crate::http::AppState;

/// Upper bound on a single blocking wait, so a long-poll can't pin a connection
/// indefinitely (Redis BLPOP's 0 = block-forever is intentionally not offered).
const MAX_TIMEOUT_MS: u64 = 60_000;

#[derive(Debug, Deserialize, ToSchema)]
pub struct BlockingPopRequest {
    /// Max time to block, in milliseconds (clamped to [1, 60000]).
    pub timeout_ms: u64,
}

#[derive(Clone, Copy)]
enum Side {
    Left,
    Right,
}

async fn blocking_pop(
    st: AppState,
    key: String,
    req: BlockingPopRequest,
    side: Side,
) -> Result<Json<PopResponse>, ApiErr> {
    let k = key_of(&key)?;
    let timeout = Duration::from_millis(req.timeout_ms.clamp(1, MAX_TIMEOUT_MS));
    let deadline = Instant::now() + timeout;

    let pop = |st: &AppState| match side {
        Side::Left => st.engine.lpop(&k),
        Side::Right => st.engine.rpop(&k),
    };

    loop {
        if let Some(v) = pop(&st) {
            ack_durable(&st).await;
            return Ok(Json(PopResponse {
                value: Some(kv_to_json(v)),
            }));
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Ok(Json(PopResponse { value: None }));
        }

        // Register interest BEFORE the final re-check so a push that lands in
        // the gap can't be missed (enable() arms the waiter eagerly).
        let notify = st.waiters.waiter(&key);
        let notified = notify.notified();
        tokio::pin!(notified);
        notified.as_mut().enable();

        if let Some(v) = pop(&st) {
            ack_durable(&st).await;
            return Ok(Json(PopResponse {
                value: Some(kv_to_json(v)),
            }));
        }

        tokio::select! {
            _ = &mut notified => {}                       // woke: retry the pop
            _ = tokio::time::sleep(remaining) => {}       // timed out: loop returns null
        }
    }
}

/// Block until an element is available at the head of the list, then pop it
/// (BLPOP). Returns null on timeout.
#[utoipa::path(post, path = "/v1/lists/{key}/blpop", tag = "Lists",
    params(("key" = String, Path, description = "List key")), request_body = BlockingPopRequest,
    responses((status = 200, description = "Popped value or null on timeout", body = PopResponse)))]
pub async fn blpop(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<BlockingPopRequest>,
) -> Result<Json<PopResponse>, ApiErr> {
    blocking_pop(st, key, req, Side::Left).await
}

/// Block until an element is available at the tail of the list, then pop it
/// (BRPOP). Returns null on timeout.
#[utoipa::path(post, path = "/v1/lists/{key}/brpop", tag = "Lists",
    params(("key" = String, Path, description = "List key")), request_body = BlockingPopRequest,
    responses((status = 200, description = "Popped value or null on timeout", body = PopResponse)))]
pub async fn brpop(
    State(st): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<BlockingPopRequest>,
) -> Result<Json<PopResponse>, ApiErr> {
    blocking_pop(st, key, req, Side::Right).await
}
