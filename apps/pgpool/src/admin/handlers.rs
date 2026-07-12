// SPEC-MANAGED: apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-admin-plane" tracker="#1290" reason="Admin plane needs generator primitives that do not exist yet.">
//! Handler bodies for every admin-plane route (TD Logic section
//! `request_kind` decision and its terminal nodes).

use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Json};

use crate::admin::metrics;
use crate::admin::state::{AdminState, NamedPool};
use crate::admin::types::{DrainResponse, PoolListResponse, PoolStatsResponse, ReadyzResponse};
use crate::PoolMode;

/// `GET /healthz` — always 200 `ok`; liveness never reflects drain state
/// (R1).
pub async fn healthz() -> &'static str {
    "ok"
}

/// `GET /readyz` — reads `drain.is_draining()` off the shared controller:
/// 200 `ok` when ready, 503 `draining` once drain has started (R2).
pub async fn readyz(State(state): State<AdminState>) -> (StatusCode, Json<ReadyzResponse>) {
    if state.drain.is_draining() {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ReadyzResponse { status: "draining" }),
        )
    } else {
        (StatusCode::OK, Json(ReadyzResponse { status: "ok" }))
    }
}

/// `GET /metrics` — Prometheus text-format gauges for every named pool
/// (AC4).
pub async fn metrics_handler(State(state): State<AdminState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, metrics::CONTENT_TYPE)],
        metrics::render(&state),
    )
}

/// `GET /openapi.json` — the IDENTICAL `serde_json::Value`
/// `pgpool spec --format openapi` serializes offline (R4/AC3 single
/// source of truth) — never a separately-typed OpenAPI document.
pub async fn openapi_json() -> Json<serde_json::Value> {
    Json(crate::spec::openapi())
}

/// `GET /docs` — a minimal Swagger UI page loading `/openapi.json`,
/// mirroring `libs/service-http`'s `docs_swagger` convention (R6).
pub async fn docs() -> impl IntoResponse {
    Html(SWAGGER_HTML)
}

/// `GET /pools` — one `PoolStatsResponse` entry per `AdminState.pools`
/// member, read live (R3).
pub async fn pools(State(state): State<AdminState>) -> Json<PoolListResponse> {
    Json(PoolListResponse {
        pools: state.pools.iter().map(pool_stats_response).collect(),
    })
}

/// `GET /pools/{pool}/stats` — 200 with the live stats when `{pool}`
/// matches a configured pool name, 404 naming the unknown pool otherwise
/// (R3).
pub async fn pool_stats(
    State(state): State<AdminState>,
    Path(pool): Path<String>,
) -> Result<Json<PoolStatsResponse>, (StatusCode, String)> {
    match state.find(&pool) {
        Some(named) => Ok(Json(pool_stats_response(named))),
        None => Err((StatusCode::NOT_FOUND, format!("unknown pool: {pool}"))),
    }
}

/// `POST /drain` — calls the SAME shared `DrainController::start_drain()`
/// the SIGTERM/SIGINT path uses; idempotent on repeated calls (R2).
pub async fn drain(State(state): State<AdminState>) -> Json<DrainResponse> {
    state.drain.start_drain();
    Json(DrainResponse {
        draining: state.drain.is_draining(),
    })
}

fn pool_stats_response(named: &NamedPool) -> PoolStatsResponse {
    let stats = named.pool.stats();
    PoolStatsResponse {
        name: named.name.clone(),
        mode: pool_mode_str(&named.mode).to_string(),
        frontend_active: named.budget.active(),
        backend_active: stats.backend_active,
        backend_idle: stats.backend_idle,
    }
}

/// Renders `PoolMode` as the lowercase string `spec.rs`'s `PoolStats`
/// schema enum declares (`["session", "transaction"]`) — deliberately NOT
/// `PoolMode`'s own `Debug`/serde spelling, which is PascalCase.
fn pool_mode_str(mode: &PoolMode) -> &'static str {
    match mode {
        PoolMode::Session => "session",
        PoolMode::Transaction => "transaction",
    }
}

/// Standalone Swagger UI page that renders whatever `/openapi.json`
/// returns, matching `libs/service-http/src/probes.rs`'s page.
const SWAGGER_HTML: &str = r##"<!doctype html>
<html>
  <head>
    <title>pgpool admin API docs</title>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui.css" />
    <style>body { margin: 0; }</style>
  </head>
  <body>
    <div id="swagger-ui"></div>
    <script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
    <script>
      window.ui = SwaggerUIBundle({
        url: "/openapi.json",
        dom_id: "#swagger-ui",
        deepLinking: true,
      });
    </script>
  </body>
</html>"##;
// </HANDWRITE>
// SPEC-MANAGED: apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#logic
// CODEGEN-BEGIN
pub fn serve_entry() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // SPEC-REF: apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#pgpool-admin-plane-logic-flow-share_drain
    // TODO: Implement process step: Clone the shared DrainController into TcpServerConfig.drain (replacing the fresh DrainController::new() TcpServerConfig::new() builds by default) and into AdminState, so the data plane, the admin plane, and readiness all read/write the same watch channel (R2, scope: drain coordination)
    todo!("process: Clone the shared DrainController into TcpServerConfig.drain (replacing the fresh DrainController::new() TcpServerConfig::new() builds by default) and into AdminState, so the data plane, the admin plane, and readiness all read/write the same watch channel (R2, scope: drain coordination)");
    // SPEC-REF: apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#pgpool-admin-plane-logic-flow-spawn_signal_task
    // TODO: Implement process step: Spawn a background task awaiting server_core::signal::wait_shutdown_signal(); when SIGTERM/SIGINT resolves it, the task calls drain.start_drain() on the shared controller (R2)
    todo!("process: Spawn a background task awaiting server_core::signal::wait_shutdown_signal(); when SIGTERM/SIGINT resolves it, the task calls drain.start_drain() on the shared controller (R2)");
    // SPEC-REF: apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#pgpool-admin-plane-logic-flow-build_admin_router
    // TODO: Implement process step: Build the admin axum Router directly against AdminState (shared DrainController clone + Vec<NamedPool>, each pairing a pool name/mode with its ConnectionBudget and BackendPool clone): /healthz, /readyz, /metrics, /openapi.json, /docs, GET /pools, GET /pools/{pool}/stats, POST /drain (R1, R3) - hand-rolled rather than libs/service-http's standard_probe_routes because that helper's openapi arg type is fn() -> utoipa::openapi::OpenApi, while apps/pgpool/src/spec.rs's single-source-of-truth OpenAPI document is a serde_json::Value the offline `pgpool spec --format openapi` CLI already serializes directly; routing /openapi.json through a typed utoipa round-trip would risk breaking the byte-for-byte parity R4/AC3 requires, so /openapi.json instead returns Json(pgpool::spec::openapi()) - the exact same Value
    todo!("process: Build the admin axum Router directly against AdminState (shared DrainController clone + Vec<NamedPool>, each pairing a pool name/mode with its ConnectionBudget and BackendPool clone): /healthz, /readyz, /metrics, /openapi.json, /docs, GET /pools, GET /pools/{pool}/stats, POST /drain (R1, R3) - hand-rolled rather than libs/service-http's standard_probe_routes because that helper's openapi arg type is fn() -> utoipa::openapi::OpenApi, while apps/pgpool/src/spec.rs's single-source-of-truth OpenAPI document is a serde_json::Value the offline `pgpool spec --format openapi` CLI already serializes directly; routing /openapi.json through a typed utoipa round-trip would risk breaking the byte-for-byte parity R4/AC3 requires, so /openapi.json instead returns Json(pgpool::spec::openapi()) - the exact same Value");
    // SPEC-REF: apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#pgpool-admin-plane-logic-flow-run_both_planes
    // TODO: Implement process step: tokio::join! the TCP frontend (tcp_server::serve, existing PoolHandler dispatch, unchanged from WI #1289) and the admin plane (http_server::serve_h2c_with_options) concurrently; each is given its OWN one-shot shutdown future that awaits drain.signal().changed()
    todo!("process: tokio::join! the TCP frontend (tcp_server::serve, existing PoolHandler dispatch, unchanged from WI #1289) and the admin plane (http_server::serve_h2c_with_options) concurrently; each is given its OWN one-shot shutdown future that awaits drain.signal().changed()");
    // Decision: Which admin request arrives while both planes are running?
    if todo!("decision: Which admin request arrives while both planes are running?") /* GET /healthz */ {
        todo!("terminal: GET /healthz: always 200 'ok' - liveness only, never reflects drain state (R1)");
    } else if todo!("decision branch: {}", "GET /readyz") { /* GET /readyz */
        // SPEC-REF: apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#pgpool-admin-plane-logic-flow-readyz_req
        // TODO: Implement process step: GET /readyz: reads drain.is_draining() off the shared controller
        todo!("process: GET /readyz: reads drain.is_draining() off the shared controller");
        // Decision: drain.is_draining()?
        if todo!("decision: drain.is_draining()?") /* not draining */ {
            todo!("terminal: false: 200 'ok'");
        } else { /* draining */
            todo!("terminal: true: 503 'draining' (R2)");
        }
    } else if todo!("decision branch: {}", "GET /metrics") { /* GET /metrics */
        todo!("terminal: GET /metrics: renders Prometheus text-format gauges (pgpool_frontend_active, pgpool_backend_active, pgpool_backend_idle, each labeled pool=<name>) from every AdminState.pools entry's ConnectionBudget::active() and BackendPool::stats() (AC4)");
    } else if todo!("decision branch: {}", "GET /openapi.json") { /* GET /openapi.json */
        todo!("terminal: GET /openapi.json: returns Json(pgpool::spec::openapi()) - the identical serde_json::Value apps/pgpool/src/spec.rs already builds for `pgpool spec --format openapi` (R4)");
    } else if todo!("decision branch: {}", "GET /docs") { /* GET /docs */
        todo!("terminal: GET /docs: static Swagger UI HTML page that loads /openapi.json, mirroring libs/service-http's docs_swagger convention");
    } else if todo!("decision branch: {}", "GET /pools") { /* GET /pools */
        todo!("terminal: GET /pools: 200 Json(PoolList{pools: [...]}) - one PoolStats entry per AdminState.pools member, matching spec.rs's PoolList/PoolStats schema (R3)");
    } else if todo!("decision branch: {}", "GET /pools/{pool}/stats") { /* GET /pools/{pool}/stats */
        // Decision: GET /pools/{pool}/stats: does {pool} match a name in AdminState.pools?
        if todo!("decision: GET /pools/{pool}/stats: does {pool} match a name in AdminState.pools?") /* name matches */ {
            todo!("terminal: found: 200 Json(PoolStats{name, mode, frontend_active, backend_active, backend_idle}) (R3, AC4)");
        } else { /* no match */
            todo!("terminal: not found: 404, body names the unknown pool");
        }
    } else { /* POST /drain */
        // SPEC-REF: apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#pgpool-admin-plane-logic-flow-drain_post
        // TODO: Implement process step: POST /drain: calls the SAME shared DrainController::start_drain() the SIGTERM path uses - one drain trigger, two sources
        todo!("process: POST /drain: calls the SAME shared DrainController::start_drain() the SIGTERM path uses - one drain trigger, two sources");
        todo!("terminal: The shared watch channel flips to Draining: /readyz starts returning 503, and the TCP frontend's own shutdown future (also awaiting drain.signal().changed()) resolves so its accept loop stops admitting new frontend connections, while already-established sessions/transactions keep relaying until they end or TcpServerConfig.drain_timeout elapses (R2, AC2); the handler returns 200 Json(DrainState{draining:true})");
    }
    todo!("terminal: The shared watch channel flips to Draining: /readyz starts returning 503, and the TCP frontend's own shutdown future (also awaiting drain.signal().changed()) resolves so its accept loop stops admitting new frontend connections, while already-established sessions/transactions keep relaying until they end or TcpServerConfig.drain_timeout elapses (R2, AC2); the handler returns 200 Json(DrainState{draining:true})");
    // Terminal: docs_req -> GET /docs: static Swagger UI HTML page that loads /openapi.json, mirroring libs/service-http's docs_swagger convention
    // Terminal: drain_effect -> The shared watch channel flips to Draining: /readyz starts returning 503, and the TCP frontend's own shutdown future (also awaiting drain.signal().changed()) resolves so its accept loop stops admitting new frontend connections, while already-established sessions/transactions keep relaying until they end or TcpServerConfig.drain_timeout elapses (R2, AC2); the handler returns 200 Json(DrainState{draining:true})
    // Terminal: healthz_req -> GET /healthz: always 200 'ok' - liveness only, never reflects drain state (R1)
    // Terminal: metrics_req -> GET /metrics: renders Prometheus text-format gauges (pgpool_frontend_active, pgpool_backend_active, pgpool_backend_idle, each labeled pool=<name>) from every AdminState.pools entry's ConnectionBudget::active() and BackendPool::stats() (AC4)
    // Terminal: openapi_req -> GET /openapi.json: returns Json(pgpool::spec::openapi()) - the identical serde_json::Value apps/pgpool/src/spec.rs already builds for `pgpool spec --format openapi` (R4)
    // Terminal: pool_stats_found -> found: 200 Json(PoolStats{name, mode, frontend_active, backend_active, backend_idle}) (R3, AC4)
    // Terminal: pool_stats_missing -> not found: 404, body names the unknown pool
    // Terminal: pools_req -> GET /pools: 200 Json(PoolList{pools: [...]}) - one PoolStats entry per AdminState.pools member, matching spec.rs's PoolList/PoolStats schema (R3)
    // Terminal: process_exit -> serve() returns Ok(()); the process exits cleanly with no forcibly-dropped in-flight session or admin request (AC2)
    // Terminal: readyz_draining -> true: 503 'draining' (R2)
    // Terminal: readyz_ready -> false: 200 'ok'
}
// CODEGEN-END
