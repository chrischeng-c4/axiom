//! Router assembly. Probe/admin endpoints stay open (no body limit, no auth)
//! so k8s probes and Prometheus scrape always reach them; the data plane gets
//! the configured body limit.

use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};

use crate::http::{handlers, AppState};

/// Build the full application router.
pub fn router(state: AppState) -> Router {
    let body_limit = state.body_limit;

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
        .route("/v1/lists/{key}/lpush", post(handlers::lpush))
        .route("/v1/lists/{key}/rpush", post(handlers::rpush))
        .route("/v1/lists/{key}/lpop", post(handlers::lpop))
        .route("/v1/lists/{key}/rpop", post(handlers::rpop))
        .layer(DefaultBodyLimit::max(body_limit));

    Router::new()
        .route("/healthz", get(handlers::healthz))
        .route("/readyz", get(handlers::readyz))
        .route("/metrics", get(handlers::metrics))
        .route("/info", get(handlers::info))
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
