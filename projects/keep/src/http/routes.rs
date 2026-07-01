//! Router assembly. Probe/admin endpoints stay open (no body limit, no auth)
//! so k8s probes and Prometheus scrape always reach them; the data plane gets
//! the configured body limit.

use std::sync::Arc;

use axum::{
    extract::DefaultBodyLimit,
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use service_http::MetricsProvider;

use crate::http::{handlers, hash, lists, meta, metrics, sets, zsets, AppState};

/// The keep OpenAPI document — the accessor the shared `service_http`
/// `/openapi.json` and `/docs` routes serve.
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    crate::http::openapi::ApiDoc::openapi()
}

/// Build the full application router.
pub fn router(state: AppState) -> Router {
    let body_limit = state.body_limit;
    let req_metrics = state.metrics.clone();

    let data_plane = Router::new()
        // single-key
        .route("/kv", get(handlers::scan))
        .route(
            "/kv/{key}",
            get(handlers::get_key)
                .put(handlers::put_key)
                .delete(handlers::delete_key)
                .head(handlers::head_key),
        )
        .route("/kv/{key}/incr", post(handlers::incr_key))
        .route("/kv/{key}/cas", post(handlers::cas_key))
        .route("/kv/{key}/setnx", post(handlers::setnx_key))
        // batch
        .route("/kv:mget", post(handlers::mget))
        .route("/kv:mset", post(handlers::mset))
        .route("/kv:mdel", post(handlers::mdel))
        // claim-check: job input/result payloads by id (#167)
        .route(
            "/inputs/{id}",
            get(handlers::get_input).put(handlers::put_input),
        )
        .route(
            "/results/{id}",
            get(handlers::get_result).put(handlers::put_result),
        )
        // locks
        .route(
            "/locks/{key}",
            post(handlers::lock)
                .delete(handlers::unlock)
                .patch(handlers::extend_lock),
        )
        // lists
        .route("/lists/{key}", get(meta::lrange))
        .route("/lists/{key}/length", get(meta::llen))
        .route("/lists/{key}/lpush", post(handlers::lpush))
        .route("/lists/{key}/rpush", post(handlers::rpush))
        .route("/lists/{key}/lpop", post(handlers::lpop))
        .route("/lists/{key}/rpop", post(handlers::rpop))
        .route("/lists/{key}/blpop", post(lists::blpop))
        .route("/lists/{key}/brpop", post(lists::brpop))
        // expiry (any key)
        .route("/kv/{key}/expire", post(meta::expire))
        .route("/kv/{key}/ttl", get(meta::ttl))
        .route("/kv/{key}/persist", post(meta::persist))
        .route("/kv/{key}/getex", post(meta::getex))
        // hashes
        .route(
            "/hashes/{key}",
            post(hash::hset).get(hash::hgetall).delete(hash::hdel),
        )
        .route("/hashes/{key}/length", get(hash::hlen))
        .route("/hashes/{key}/mget", post(hash::hmget))
        .route("/hashes/{key}/incr", post(hash::hincr))
        .route(
            "/hashes/{key}/fields/{field}",
            get(hash::hget).head(hash::hexists),
        )
        // sets
        .route(
            "/sets/{key}",
            post(sets::sadd).get(sets::smembers).delete(sets::srem),
        )
        .route("/sets/{key}/length", get(sets::scard))
        .route(
            "/sets/{key}/members/{member}",
            axum::routing::head(sets::sismember),
        )
        // sorted sets
        .route(
            "/zsets/{key}",
            post(zsets::zadd).get(zsets::zrange).delete(zsets::zrem),
        )
        .route("/zsets/{key}/length", get(zsets::zcard))
        .route("/zsets/{key}/incr", post(zsets::zincr))
        .route("/zsets/{key}/members/{member}/score", get(zsets::zscore))
        .route("/zsets/{key}/members/{member}/rank", get(zsets::zrank))
        // Per-route request metrics (counts + latency). route_layer => only for
        // matched data-plane routes, and MatchedPath is populated.
        .route_layer(from_fn_with_state(req_metrics, metrics::track))
        .layer(DefaultBodyLimit::max(body_limit));

    // Standard probes (`/healthz`, `/readyz`, `/metrics`, `/openapi.json`,
    // `/docs`) come from the shared service shell so the operational surface
    // matches every other service. AppState supplies readiness + Prometheus
    // metrics; `/readyz` reports 503 while draining.
    let probe_state = Arc::new(state.clone());
    let metrics: Arc<dyn MetricsProvider> = probe_state.clone();
    let probes = service_http::standard_probe_routes(probe_state, Some(metrics), openapi);

    // keep-specific admin routes the shared shell does not own.
    let admin = Router::new()
        .route("/info", get(handlers::info))
        .route("/cluster", get(handlers::cluster));

    probes
        .merge(admin.with_state(state.clone()))
        .merge(data_plane.with_state(state))
        // One INFO-level tracing span per request — spans probes + data plane.
        .layer(service_http::trace_layer())
}
