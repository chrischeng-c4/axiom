//! The five standard probe/admin endpoints every k8s-native service ships
//! (CONTRIBUTING.md "standard endpoints"): `/healthz`, `/readyz`, `/metrics`,
//! `/openapi.json`, `/docs`.
//!
//! These routes carry **no auth and no body limit** — k8s liveness/readiness
//! probes and Prometheus scrape must reach them token-free even when the data
//! plane requires auth. A service merges its own (auth'd, body-limited) data
//! plane onto the router returned here. This is the exact shape lumen
//! (`api::router`) and keep (`http::routes::router`) hand-roll today.

use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};

use crate::metrics::MetricsProvider;
use crate::readiness::ReadinessHook;

/// State the probe handlers close over. Cheap to clone (two `Arc`s + a fn
/// pointer); `axum` clones it per request.
#[derive(Clone)]
struct ProbeState {
    readiness: Arc<dyn ReadinessHook>,
    metrics: Option<Arc<dyn MetricsProvider>>,
    openapi: fn() -> utoipa::openapi::OpenApi,
}

/// Build the five standard probe routes:
///
/// - `GET /healthz` → 200 `ok` (process is alive).
/// - `GET /readyz` → 200 `ready`, or 503 `draining` when
///   [`ReadinessHook::is_draining`] is `true`.
/// - `GET /metrics` → `text/plain; version=0.0.4` from `metrics`
///   ([`MetricsProvider::render_metrics`]), or an empty body when `None`.
/// - `GET /openapi.json` → the service's OpenAPI document as JSON.
/// - `GET /docs` → a Swagger UI page that loads `/openapi.json`.
///
/// The returned router has **no auth layer and no body limit**; a service
/// `.merge`s its data plane (which carries those) onto it. `openapi` is a fn
/// pointer to the service's generated-doc accessor (e.g. `lumen::api::openapi`).
pub fn standard_probe_routes<R: ReadinessHook + 'static>(
    readiness: Arc<R>,
    metrics: Option<Arc<dyn MetricsProvider>>,
    openapi: fn() -> utoipa::openapi::OpenApi,
) -> Router {
    let state = ProbeState {
        readiness,
        metrics,
        openapi,
    };
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/metrics", get(metrics_handler))
        .route("/openapi.json", get(openapi_spec))
        .route("/docs", get(docs_swagger))
        .with_state(state)
}

/// `GET /healthz` — liveness. 200 as long as the process can answer.
async fn healthz() -> &'static str {
    "ok"
}

/// `GET /readyz` — readiness. 503 `draining` once shutdown begins so k8s stops
/// routing during the grace window; 200 `ready` otherwise.
async fn readyz(State(state): State<ProbeState>) -> (StatusCode, &'static str) {
    if state.readiness.is_draining() {
        (StatusCode::SERVICE_UNAVAILABLE, "draining")
    } else {
        (StatusCode::OK, "ready")
    }
}

/// `GET /metrics` — Prometheus text-format. Empty body when no provider is set.
async fn metrics_handler(
    State(state): State<ProbeState>,
) -> (StatusCode, [(&'static str, &'static str); 1], String) {
    let body = state
        .metrics
        .as_ref()
        .map(|m| m.render_metrics())
        .unwrap_or_default();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4")],
        body,
    )
}

/// `GET /openapi.json` — the live OpenAPI 3 document for external consumers.
async fn openapi_spec(State(state): State<ProbeState>) -> Json<utoipa::openapi::OpenApi> {
    Json((state.openapi)())
}

/// `GET /docs` — interactive Swagger UI (FastAPI convention). The page pulls the
/// live spec from `/openapi.json`, so its "Try it out" buttons fire real
/// requests against this pod. A minimal hand-rolled HTML page over the
/// swagger-ui-dist CDN — the same approach lumen/keep use, which keeps the
/// workspace lock free of a `utoipa-swagger-ui` version pinned against utoipa 4
/// / axum 0.8 (see the crate docs note).
async fn docs_swagger() -> impl IntoResponse {
    Html(SWAGGER_HTML)
}

/// Standalone Swagger UI page that renders whatever `/openapi.json` returns.
const SWAGGER_HTML: &str = r##"<!doctype html>
<html>
  <head>
    <title>API docs</title>
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    use utoipa::OpenApi as _;

    #[derive(utoipa::OpenApi)]
    #[openapi(info(title = "test", description = "probe-route test doc"))]
    struct TestDoc;

    fn test_openapi() -> utoipa::openapi::OpenApi {
        TestDoc::openapi()
    }

    struct Draining(bool);
    impl ReadinessHook for Draining {
        fn is_draining(&self) -> bool {
            self.0
        }
    }

    struct StaticMetrics(&'static str);
    impl MetricsProvider for StaticMetrics {
        fn render_metrics(&self) -> String {
            self.0.to_string()
        }
    }

    async fn get(router: Router, path: &str) -> (StatusCode, String) {
        let resp = router
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn healthz_is_ok() {
        let router = standard_probe_routes(Arc::new(Draining(false)), None, test_openapi);
        let (status, body) = get(router, "/healthz").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "ok");
    }

    #[tokio::test]
    async fn readyz_200_when_not_draining() {
        let router = standard_probe_routes(Arc::new(Draining(false)), None, test_openapi);
        let (status, body) = get(router, "/readyz").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "ready");
    }

    #[tokio::test]
    async fn readyz_503_when_draining() {
        let router = standard_probe_routes(Arc::new(Draining(true)), None, test_openapi);
        let (status, body) = get(router, "/readyz").await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body, "draining");
    }

    #[tokio::test]
    async fn metrics_renders_provider_text() {
        let metrics: Arc<dyn MetricsProvider> = Arc::new(StaticMetrics("svc_up 1\n"));
        let router = standard_probe_routes(Arc::new(Draining(false)), Some(metrics), test_openapi);
        let (status, body) = get(router, "/metrics").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "svc_up 1\n");
    }

    #[tokio::test]
    async fn metrics_empty_when_no_provider() {
        let router = standard_probe_routes(Arc::new(Draining(false)), None, test_openapi);
        let (status, body) = get(router, "/metrics").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "");
    }

    #[tokio::test]
    async fn openapi_json_parses() {
        let router = standard_probe_routes(Arc::new(Draining(false)), None, test_openapi);
        let (status, body) = get(router, "/openapi.json").await;
        assert_eq!(status, StatusCode::OK);
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(parsed["info"]["title"], "test");
    }

    #[tokio::test]
    async fn docs_serves_swagger_page() {
        let router = standard_probe_routes(Arc::new(Draining(false)), None, test_openapi);
        let (status, body) = get(router, "/docs").await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("swagger-ui"));
        assert!(body.contains("/openapi.json"));
    }
}
