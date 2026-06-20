//! Router assembly. Probe/admin endpoints stay open (no body limit, no auth)
//! so k8s probes and Prometheus scrape always reach them; the data plane gets
//! the configured body limit.

use axum::{
    extract::DefaultBodyLimit,
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};

use crate::http::{handlers, hash, lists, meta, metrics, sets, zsets, AppState};

/// Build the full application router.
pub fn router(state: AppState) -> Router {
    let body_limit = state.body_limit;
    let req_metrics = state.metrics.clone();

    let data_plane = Router::new()
        // single-key
        .route("/v1/kv", get(handlers::scan))
        .route(
            "/v1/kv/{key}",
            get(handlers::get_key)
                .put(handlers::put_key)
                .delete(handlers::delete_key)
                .head(handlers::head_key),
        )
        .route("/v1/kv/{key}/incr", post(handlers::incr_key))
        .route("/v1/kv/{key}/cas", post(handlers::cas_key))
        .route("/v1/kv/{key}/setnx", post(handlers::setnx_key))
        // batch
        .route("/v1/kv:mget", post(handlers::mget))
        .route("/v1/kv:mset", post(handlers::mset))
        .route("/v1/kv:mdel", post(handlers::mdel))
        // locks
        .route(
            "/v1/locks/{key}",
            post(handlers::lock)
                .delete(handlers::unlock)
                .patch(handlers::extend_lock),
        )
        // lists
        .route("/v1/lists/{key}", get(meta::lrange))
        .route("/v1/lists/{key}/length", get(meta::llen))
        .route("/v1/lists/{key}/lpush", post(handlers::lpush))
        .route("/v1/lists/{key}/rpush", post(handlers::rpush))
        .route("/v1/lists/{key}/lpop", post(handlers::lpop))
        .route("/v1/lists/{key}/rpop", post(handlers::rpop))
        .route("/v1/lists/{key}/blpop", post(lists::blpop))
        .route("/v1/lists/{key}/brpop", post(lists::brpop))
        // expiry (any key)
        .route("/v1/kv/{key}/expire", post(meta::expire))
        .route("/v1/kv/{key}/ttl", get(meta::ttl))
        .route("/v1/kv/{key}/persist", post(meta::persist))
        .route("/v1/kv/{key}/getex", post(meta::getex))
        // hashes
        .route(
            "/v1/hashes/{key}",
            post(hash::hset).get(hash::hgetall).delete(hash::hdel),
        )
        .route("/v1/hashes/{key}/length", get(hash::hlen))
        .route("/v1/hashes/{key}/mget", post(hash::hmget))
        .route("/v1/hashes/{key}/incr", post(hash::hincr))
        .route(
            "/v1/hashes/{key}/fields/{field}",
            get(hash::hget).head(hash::hexists),
        )
        // sets
        .route(
            "/v1/sets/{key}",
            post(sets::sadd).get(sets::smembers).delete(sets::srem),
        )
        .route("/v1/sets/{key}/length", get(sets::scard))
        .route("/v1/sets/{key}/members/{member}", axum::routing::head(sets::sismember))
        // sorted sets
        .route(
            "/v1/zsets/{key}",
            post(zsets::zadd).get(zsets::zrange).delete(zsets::zrem),
        )
        .route("/v1/zsets/{key}/length", get(zsets::zcard))
        .route("/v1/zsets/{key}/incr", post(zsets::zincr))
        .route("/v1/zsets/{key}/members/{member}/score", get(zsets::zscore))
        .route("/v1/zsets/{key}/members/{member}/rank", get(zsets::zrank))
        // Per-route request metrics (counts + latency). route_layer => only for
        // matched data-plane routes, and MatchedPath is populated.
        .route_layer(from_fn_with_state(req_metrics, metrics::track))
        .layer(DefaultBodyLimit::max(body_limit));

    Router::new()
        .route("/healthz", get(handlers::healthz))
        .route("/readyz", get(handlers::readyz))
        .route("/metrics", get(handlers::metrics))
        .route("/info", get(handlers::info))
        .route("/cluster", get(handlers::cluster))
        .route("/openapi.json", get(handlers::openapi_spec))
        .route("/docs", get(handlers::docs))
        .merge(data_plane)
        // One INFO-level tracing span per request — structured access logs.
        .layer(
            tower_http::trace::TraceLayer::new_for_http().make_span_with(
                tower_http::trace::DefaultMakeSpan::new().level(tracing::Level::INFO),
            ),
        )
        .with_state(state)
}
