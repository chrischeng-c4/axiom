// SPEC-MANAGED: apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-admin-plane" tracker="#1290" reason="Admin plane needs generator primitives that do not exist yet.">
//! Builds the admin `axum::Router` against `AdminState` (TD Logic section
//! `build_admin_router` node): `/healthz`, `/readyz`, `/metrics`,
//! `/openapi.json`, `/docs`, `GET /pools`, `GET /pools/{pool}/stats`,
//! `POST /drain` (R1, R3).

use axum::routing::{get, post};
use axum::Router;

use crate::admin::handlers;
use crate::admin::state::AdminState;

/// The exact served route set, kept in the same order as `build_router`'s
/// `.route()` calls below so the two never drift; also the fixture the
/// offline-vs-served conformance test (R5/AC3) walks.
pub const ADMIN_ROUTES: &[(&str, &str)] = &[
    ("GET", "/healthz"),
    ("GET", "/readyz"),
    ("GET", "/metrics"),
    ("GET", "/openapi.json"),
    ("GET", "/docs"),
    ("GET", "/pools"),
    ("GET", "/pools/{pool}/stats"),
    ("POST", "/drain"),
];

/// Builds the admin `Router` from `AdminState` (R1, R3): registers exactly
/// the routes in [`ADMIN_ROUTES`], nothing more.
pub fn build_router(state: AdminState) -> Router {
    Router::new()
        .route("/healthz", get(handlers::healthz))
        .route("/readyz", get(handlers::readyz))
        .route("/metrics", get(handlers::metrics_handler))
        .route("/openapi.json", get(handlers::openapi_json))
        .route("/docs", get(handlers::docs))
        .route("/pools", get(handlers::pools))
        .route("/pools/{pool}/stats", get(handlers::pool_stats))
        .route("/drain", post(handlers::drain))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin::NamedPool;
    use crate::pool::{BackendPool, PoolConfig};
    use crate::proxy::BackendEndpointConfig;
    use crate::wire::WireCodecConfig;
    use crate::PoolMode;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use server_core::{ConnectionBudget, DrainController};
    use std::time::Duration;
    use tower::ServiceExt;

    fn test_pool_config() -> PoolConfig {
        PoolConfig {
            endpoint: BackendEndpointConfig {
                host: "127.0.0.1".to_string(),
                port: 5432,
            },
            max_backend_connections: 4,
            acquire_timeout: Duration::from_millis(50),
            backend_connect_timeout: Duration::from_millis(50),
            wire: WireCodecConfig::default(),
        }
    }

    fn test_state(drain: DrainController) -> AdminState {
        AdminState::new(
            drain,
            vec![NamedPool {
                name: "default".to_string(),
                mode: PoolMode::Transaction,
                budget: ConnectionBudget::new(10),
                pool: BackendPool::new(test_pool_config()),
            }],
        )
    }

    async fn call(router: &Router, method: &str, path: &str) -> (StatusCode, String) {
        let request = Request::builder()
            .method(method)
            .uri(path)
            .body(Body::empty())
            .unwrap();
        let response = router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8_lossy(&bytes).into_owned())
    }

    /// verify: admin::router_registers_exact_route_set (R1)
    #[tokio::test]
    async fn router_registers_exact_route_set() {
        let router = build_router(test_state(DrainController::new()));
        for (method, path) in ADMIN_ROUTES {
            let concrete = path.replace("{pool}", "default");
            let (status, _) = call(&router, method, &concrete).await;
            assert_ne!(
                status,
                StatusCode::NOT_FOUND,
                "{method} {path} should be routed"
            );
        }
        let (status, _) = call(&router, "GET", "/not-a-real-route").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    /// verify: admin::healthz_returns_200_regardless_of_drain_state (R1)
    #[tokio::test]
    async fn healthz_returns_200_regardless_of_drain_state() {
        let drain = DrainController::new();
        let router = build_router(test_state(drain.clone()));
        let (status, body) = call(&router, "GET", "/healthz").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "ok");

        drain.start_drain();
        let (status, body) = call(&router, "GET", "/healthz").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "ok");
    }

    /// verify: admin::readyz_returns_200_ok_when_not_draining (R2)
    #[tokio::test]
    async fn readyz_returns_200_ok_when_not_draining() {
        let router = build_router(test_state(DrainController::new()));
        let (status, body) = call(&router, "GET", "/readyz").await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("\"status\":\"ok\""));
    }

    /// verify: admin::readyz_returns_503_draining_after_start_drain (R2)
    #[tokio::test]
    async fn readyz_returns_503_draining_after_start_drain() {
        let drain = DrainController::new();
        let state = test_state(drain.clone());
        // `AdminState::new` subscribes a held-open receiver, matching
        // production where the TCP frontend/signal task always keep one
        // alive; `tokio::sync::watch::Sender::send` is a receiver-less
        // no-op otherwise (see `AdminState::_drain_signal`'s doc comment).
        drain.start_drain();
        let router = build_router(state);
        let (status, body) = call(&router, "GET", "/readyz").await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert!(body.contains("\"status\":\"draining\""));
    }

    /// verify: admin::post_drain_flips_shared_drain_controller_observed_by_readyz (R2)
    #[tokio::test]
    async fn post_drain_flips_shared_drain_controller_observed_by_readyz() {
        let drain = DrainController::new();
        let router = build_router(test_state(drain.clone()));

        let (status, _) = call(&router, "GET", "/readyz").await;
        assert_eq!(status, StatusCode::OK);

        let (status, body) = call(&router, "POST", "/drain").await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("\"draining\":true"));

        // The SAME controller instance observes the flip, not a copy.
        assert!(drain.is_draining());
        let (status, _) = call(&router, "GET", "/readyz").await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    }

    /// verify: admin::repeated_post_drain_is_idempotent (R2)
    #[tokio::test]
    async fn repeated_post_drain_is_idempotent() {
        let router = build_router(test_state(DrainController::new()));
        for _ in 0..3 {
            let (status, body) = call(&router, "POST", "/drain").await;
            assert_eq!(status, StatusCode::OK);
            assert!(body.contains("\"draining\":true"));
        }
    }

    /// verify: admin::pools_endpoint_lists_one_entry_per_named_pool (R3)
    #[tokio::test]
    async fn pools_endpoint_lists_one_entry_per_named_pool() {
        let router = build_router(test_state(DrainController::new()));
        let (status, body) = call(&router, "GET", "/pools").await;
        assert_eq!(status, StatusCode::OK);
        let value: serde_json::Value = serde_json::from_str(&body).unwrap();
        let pools = value["pools"].as_array().unwrap();
        assert_eq!(pools.len(), 1);
        assert_eq!(pools[0]["name"], "default");
        assert_eq!(pools[0]["mode"], "transaction");
    }

    /// verify: admin::pool_stats_returns_404_for_unknown_pool_name (R3)
    #[tokio::test]
    async fn pool_stats_returns_404_for_unknown_pool_name() {
        let router = build_router(test_state(DrainController::new()));
        let (status, body) = call(&router, "GET", "/pools/nope/stats").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(body.contains("nope"));
    }

    /// verify: admin::pool_stats_reflects_live_budget_and_pool_state (R3)
    #[tokio::test]
    async fn pool_stats_reflects_live_budget_and_pool_state() {
        let state = test_state(DrainController::new());
        let budget = state.pools[0].budget.clone();
        let router = build_router(state);

        let (_, body) = call(&router, "GET", "/pools/default/stats").await;
        let before: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(before["frontend_active"], 0);

        let permit = budget.try_acquire().expect("permit available");
        let (_, body) = call(&router, "GET", "/pools/default/stats").await;
        let after: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(after["frontend_active"], 1);
        drop(permit);
    }

    /// verify: admin::openapi_json_endpoint_matches_spec_openapi_value_exactly (R5)
    #[tokio::test]
    async fn openapi_json_endpoint_matches_spec_openapi_value_exactly() {
        let router = build_router(test_state(DrainController::new()));
        let (status, body) = call(&router, "GET", "/openapi.json").await;
        assert_eq!(status, StatusCode::OK);
        let served: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(served, crate::spec::openapi());
    }

    /// verify: admin::served_route_set_matches_offline_routes_json_inventory (R5)
    #[tokio::test]
    async fn served_route_set_matches_offline_routes_json_inventory() {
        let offline: serde_json::Value = serde_json::from_str(&crate::spec::routes_json()).unwrap();
        let offline_routes: Vec<(String, String)> = offline["routes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|route| {
                (
                    route["method"].as_str().unwrap().to_string(),
                    route["path"].as_str().unwrap().to_string(),
                )
            })
            .collect();
        let served_routes: Vec<(String, String)> = ADMIN_ROUTES
            .iter()
            .map(|(m, p)| (m.to_string(), p.to_string()))
            .collect();
        assert_eq!(offline_routes, served_routes);
    }

    /// verify: admin::docs_serves_swagger_ui_html_referencing_openapi_json (R6)
    #[tokio::test]
    async fn docs_serves_swagger_ui_html_referencing_openapi_json() {
        let router = build_router(test_state(DrainController::new()));
        let (status, body) = call(&router, "GET", "/docs").await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("swagger-ui"));
        assert!(body.contains("/openapi.json"));
    }
}
// </HANDWRITE>
