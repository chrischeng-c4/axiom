// CODEGEN-BEGIN
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
    response::{Html, IntoResponse, Json, Response},
    routing::get,
    Router,
};

use crate::metrics::MetricsProvider;
use crate::readiness::ReadinessHook;
use server_lifecycle::{LifecycleController, LifecycleObservation};

/// State the probe handlers close over. Cheap to clone (two `Arc`s + a fn
/// pointer); `axum` clones it per request.
#[derive(Clone)]
struct ProbeState {
    readiness: Arc<dyn ReadinessHook>,
    metrics: Option<Arc<dyn MetricsProvider>>,
    openapi: OpenapiSource,
}

/// The public OpenAPI source used by standard probes.
///
/// Most services hand the helper an `utoipa` document. Services that require
/// one byte-identical document for an offline CLI, a committed snapshot, and
/// the live route use [`OpenapiSource::CanonicalJson`] instead.
#[derive(Clone, Copy)]
enum OpenapiSource {
    Typed(fn() -> utoipa::openapi::OpenApi),
    CanonicalJson(fn() -> String),
}

/// Build the five standard probe routes:
///
/// - `GET /healthz` → 200 `ok` (process is alive).
/// - `GET /readyz` → 200 `ok`, or 503 `draining` when
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
        openapi: OpenapiSource::Typed(openapi),
    };
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/metrics", get(metrics_handler))
        .route("/openapi.json", get(openapi_spec))
        .route("/docs", get(docs_swagger))
        .with_state(state)
}

/// Build standard probe routes with a canonical OpenAPI JSON producer.
///
/// The returned `/openapi.json` body is exactly the producer's bytes. This is
/// intentionally additive: existing services keep the typed-`utoipa` API,
/// while a service with a committed client snapshot can prove all public
/// OpenAPI surfaces share one canonical serialization.
pub fn standard_probe_routes_canonical_json<R: ReadinessHook + 'static>(
    readiness: Arc<R>,
    metrics: Option<Arc<dyn MetricsProvider>>,
    openapi_json: fn() -> String,
) -> Router {
    let state = ProbeState {
        readiness,
        metrics,
        openapi: OpenapiSource::CanonicalJson(openapi_json),
    };
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/metrics", get(metrics_handler))
        .route("/openapi.json", get(openapi_spec))
        .route("/docs", get(docs_swagger))
        .with_state(state)
}

#[derive(Clone)]
struct LifecycleProbeState {
    lifecycle: LifecycleController,
    metrics: Option<Arc<dyn MetricsProvider>>,
    openapi: OpenapiSource,
}

/// Production probes backed by one caller-owned lifecycle controller.
pub fn lifecycle_probe_routes(
    lifecycle: LifecycleController,
    metrics: Option<Arc<dyn MetricsProvider>>,
    openapi: fn() -> utoipa::openapi::OpenApi,
) -> Router {
    lifecycle_probe_router(LifecycleProbeState {
        lifecycle,
        metrics,
        openapi: OpenapiSource::Typed(openapi),
    })
}

pub fn lifecycle_probe_routes_canonical_json(
    lifecycle: LifecycleController,
    metrics: Option<Arc<dyn MetricsProvider>>,
    openapi_json: fn() -> String,
) -> Router {
    lifecycle_probe_router(LifecycleProbeState {
        lifecycle,
        metrics,
        openapi: OpenapiSource::CanonicalJson(openapi_json),
    })
}

fn lifecycle_probe_router(state: LifecycleProbeState) -> Router {
    Router::new()
        .route("/healthz", get(lifecycle_healthz))
        .route("/readyz", get(lifecycle_readyz))
        .route("/metrics", get(lifecycle_metrics))
        .route("/openapi.json", get(lifecycle_openapi))
        .route("/docs", get(lifecycle_docs))
        .with_state(state)
}

fn evidence_headers(observation: &LifecycleObservation) -> [(axum::http::HeaderName, String); 3] {
    [
        (
            axum::http::HeaderName::from_static("x-lifecycle-phase"),
            format!("{:?}", observation.phase).to_ascii_lowercase(),
        ),
        (
            axum::http::HeaderName::from_static("x-lifecycle-generation"),
            observation.generation.to_string(),
        ),
        (
            axum::http::HeaderName::from_static("x-lifecycle-reason-code"),
            sanitize_header(&observation.reason_code),
        ),
    ]
}

fn sanitize_header(value: &str) -> String {
    let value: String = value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ':'))
        .take(64)
        .collect();
    if value.is_empty() {
        "unknown".into()
    } else {
        value
    }
}

async fn lifecycle_healthz(State(state): State<LifecycleProbeState>) -> Response {
    let observation = state.lifecycle.observation();
    let status = if observation.is_healthy() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    let body = if observation.is_healthy() {
        "ok"
    } else {
        "unhealthy"
    };
    (status, evidence_headers(&observation), body).into_response()
}

async fn lifecycle_readyz(State(state): State<LifecycleProbeState>) -> Response {
    let observation = state.lifecycle.observation();
    let status = if observation.is_ready() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    let body = if matches!(
        observation.phase,
        server_lifecycle::LifecyclePhase::Draining | server_lifecycle::LifecyclePhase::Stopping
    ) {
        "draining"
    } else if observation.is_ready() {
        "ok"
    } else {
        "unready"
    };
    (status, evidence_headers(&observation), body).into_response()
}

async fn lifecycle_metrics(State(state): State<LifecycleProbeState>) -> Response {
    let observation = state.lifecycle.observation();
    let body = state
        .metrics
        .as_ref()
        .map(|m| m.render_metrics())
        .unwrap_or_default();
    (
        StatusCode::OK,
        evidence_headers(&observation),
        [("content-type", "text/plain; version=0.0.4")],
        body,
    )
        .into_response()
}

async fn lifecycle_openapi(State(state): State<LifecycleProbeState>) -> Response {
    let headers = evidence_headers(&state.lifecycle.observation());
    match state.openapi {
        OpenapiSource::Typed(f) => (headers, Json(f())).into_response(),
        OpenapiSource::CanonicalJson(f) => {
            (headers, [("content-type", "application/json")], f()).into_response()
        }
    }
}

async fn lifecycle_docs(State(state): State<LifecycleProbeState>) -> Response {
    (
        evidence_headers(&state.lifecycle.observation()),
        Html(SWAGGER_HTML),
    )
        .into_response()
}

/// `GET /healthz` — liveness. 200 as long as the process can answer.
async fn healthz() -> &'static str {
    "ok"
}

/// `GET /readyz` — readiness. 503 `draining` once shutdown begins so k8s stops
/// routing during the grace window; 200 `ok` otherwise.
async fn readyz(State(state): State<ProbeState>) -> (StatusCode, &'static str) {
    if state.readiness.is_draining() {
        (StatusCode::SERVICE_UNAVAILABLE, "draining")
    } else {
        (StatusCode::OK, "ok")
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
async fn openapi_spec(State(state): State<ProbeState>) -> Response {
    match state.openapi {
        OpenapiSource::Typed(openapi) => Json(openapi()).into_response(),
        OpenapiSource::CanonicalJson(openapi_json) => {
            ([("content-type", "application/json")], openapi_json()).into_response()
        }
    }
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
        assert_eq!(body, "ok");
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
    async fn canonical_openapi_keeps_producer_bytes() {
        fn canonical() -> String {
            "{\n  \"openapi\": \"3.2.0\"\n}".into()
        }
        let router =
            standard_probe_routes_canonical_json(Arc::new(Draining(false)), None, canonical);
        let (status, body) = get(router, "/openapi.json").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, canonical());
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
// CODEGEN-END
