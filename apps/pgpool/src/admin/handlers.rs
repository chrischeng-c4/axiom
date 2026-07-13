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
