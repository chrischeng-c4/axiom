use axum::{body::Body, http::Request};
use http_body_util::BodyExt;
use server_lifecycle::{LifecycleController, LifecyclePhase};
use service_http::{
    lifecycle_probe_routes, lifecycle_probe_routes_canonical_json, standard_probe_routes,
    standard_probe_routes_canonical_json, ReadinessHook,
};
use std::sync::Arc;
use tower::ServiceExt;
use utoipa::OpenApi;

#[derive(utoipa::OpenApi)]
#[openapi(info(title = "test", version = "1"))]
struct TestDoc;
fn doc() -> utoipa::openapi::OpenApi {
    TestDoc::openapi()
}
fn json() -> String {
    "{\"canonical\":true}".into()
}

async fn probe(router: &axum::Router, path: &str) -> (u16, String, String, String, String) {
    let response = router
        .clone()
        .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status().as_u16();
    let phase = response
        .headers()
        .get("x-lifecycle-phase")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let generation = response
        .headers()
        .get("x-lifecycle-generation")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let reason = response
        .headers()
        .get("x-lifecycle-reason-code")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let body = String::from_utf8(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
    (status, body, phase, generation, reason)
}

fn reach(c: &LifecycleController, phase: LifecyclePhase) {
    match phase {
        LifecyclePhase::Starting => {}
        LifecyclePhase::Recovering => {
            c.transition(LifecyclePhase::Recovering, "recovering", "retry")
                .unwrap();
        }
        LifecyclePhase::Serving => {
            c.transition(LifecyclePhase::Serving, "serving", "open")
                .unwrap();
        }
        LifecyclePhase::Degraded => {
            c.transition(LifecyclePhase::Degraded, "degraded", "safe")
                .unwrap();
        }
        LifecyclePhase::Draining | LifecyclePhase::Stopping | LifecyclePhase::Stopped => {
            c.transition(LifecyclePhase::Draining, "draining", "stop")
                .unwrap();
            if phase != LifecyclePhase::Draining {
                c.transition(LifecyclePhase::Stopping, "stopping", "hooks")
                    .unwrap();
            }
            if phase == LifecyclePhase::Stopped {
                c.transition(LifecyclePhase::Stopped, "stopped", "done")
                    .unwrap();
            }
        }
        LifecyclePhase::Fatal => {
            c.transition(LifecyclePhase::Fatal, "fatal", "failed")
                .unwrap();
        }
    }
}

#[tokio::test]
async fn one_controller() {
    let c = LifecycleController::new();
    let router = lifecycle_probe_routes(c.clone(), None, doc);
    assert_eq!(probe(&router, "/readyz").await.0, 503);
    c.transition(LifecyclePhase::Serving, "serving", "open")
        .unwrap();
    assert_eq!(probe(&router, "/readyz").await.0, 200);
    assert_eq!(probe(&router, "/readyz").await.2, "serving");
}

#[tokio::test]
async fn startup_contract() {
    let c = LifecycleController::new();
    let router = lifecycle_probe_routes(c.clone(), None, doc);
    for phase in [LifecyclePhase::Starting, LifecyclePhase::Recovering] {
        if phase == LifecyclePhase::Recovering {
            reach(&c, phase);
        }
        assert_eq!(probe(&router, "/healthz").await.0, 200);
        assert_eq!(probe(&router, "/readyz").await.0, 503);
    }
    c.transition(LifecyclePhase::Serving, "serving", "first success")
        .unwrap();
    assert_eq!(probe(&router, "/readyz").await.0, 200);
    c.transition(LifecyclePhase::Draining, "signal", "shutdown")
        .unwrap();
    assert_eq!(probe(&router, "/healthz").await.0, 200);
    assert_eq!(probe(&router, "/readyz").await.1, "draining");
}

#[tokio::test]
async fn state_table() {
    let expected = [
        (
            LifecyclePhase::Starting,
            (200, "ok"),
            (503, "unready"),
            0,
            "starting",
        ),
        (
            LifecyclePhase::Recovering,
            (200, "ok"),
            (503, "unready"),
            1,
            "recovering",
        ),
        (
            LifecyclePhase::Serving,
            (200, "ok"),
            (200, "ok"),
            1,
            "serving",
        ),
        (
            LifecyclePhase::Degraded,
            (200, "ok"),
            (200, "ok"),
            1,
            "degraded",
        ),
        (
            LifecyclePhase::Draining,
            (200, "ok"),
            (503, "draining"),
            1,
            "draining",
        ),
        (
            LifecyclePhase::Stopping,
            (200, "ok"),
            (503, "draining"),
            2,
            "stopping",
        ),
        (
            LifecyclePhase::Stopped,
            (503, "unhealthy"),
            (503, "unready"),
            3,
            "stopped",
        ),
        (
            LifecyclePhase::Fatal,
            (503, "unhealthy"),
            (503, "unready"),
            1,
            "fatal",
        ),
    ];
    for (phase, expected_health, expected_ready, generation, reason) in expected {
        let c = LifecycleController::new();
        reach(&c, phase);
        let router = lifecycle_probe_routes(c.clone(), None, doc);
        let health = probe(&router, "/healthz").await;
        let ready = probe(&router, "/readyz").await;
        assert_eq!(health.0, expected_health.0);
        assert_eq!(health.1, expected_health.1);
        assert_eq!(ready.0, expected_ready.0);
        assert_eq!(ready.1, expected_ready.1);
        for response in [health, ready] {
            assert_eq!(response.2, format!("{phase:?}").to_ascii_lowercase());
            assert_eq!(response.3, generation.to_string());
            assert_eq!(response.4, reason);
        }
    }
    let c = LifecycleController::serving();
    c.transition_degraded(true, "dep", "open").unwrap();
    let g = c.observation().generation;
    assert_eq!(
        probe(&lifecycle_probe_routes(c.clone(), None, doc), "/readyz")
            .await
            .0,
        200
    );
    c.transition_degraded(false, "dep", "closed").unwrap();
    assert!(c.observation().generation > g);
    let ready = probe(&lifecycle_probe_routes(c.clone(), None, doc), "/readyz").await;
    assert_eq!(ready.0, 503);
    assert_eq!(ready.1, "unready");
    assert_eq!(ready.2, "degraded");
    assert_eq!(ready.3, c.observation().generation.to_string());
    assert_eq!(ready.4, "dep");
}

#[tokio::test]
async fn compatibility() {
    struct Ready(bool);
    impl ReadinessHook for Ready {
        fn is_draining(&self) -> bool {
            self.0
        }
    }
    let r = standard_probe_routes(Arc::new(Ready(false)), None, doc);
    for path in ["/healthz", "/readyz", "/metrics", "/openapi.json", "/docs"] {
        let response = r
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_ne!(response.status(), 404, "legacy route {path} returned 404");
    }
    let response = r
        .oneshot(
            Request::builder()
                .uri("/readyz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        String::from_utf8(
            response
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .to_vec()
        )
        .unwrap(),
        "ok"
    );
    let r = standard_probe_routes(Arc::new(Ready(true)), None, doc);
    let response = r
        .oneshot(
            Request::builder()
                .uri("/readyz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        String::from_utf8(
            response
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .to_vec()
        )
        .unwrap(),
        "draining"
    );
    let r = standard_probe_routes_canonical_json(Arc::new(Ready(false)), None, json);
    let response = r
        .oneshot(
            Request::builder()
                .uri("/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(
        String::from_utf8(
            response
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .to_vec()
        )
        .unwrap(),
        json()
    );
}

#[tokio::test]
async fn lifecycle_route_inventory_includes_docs_evidence() {
    let c = LifecycleController::serving();
    let router = lifecycle_probe_routes(c.clone(), None, doc);
    let observation = c.observation();
    for path in ["/healthz", "/readyz", "/metrics", "/openapi.json", "/docs"] {
        let response = router
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_ne!(response.status(), 404, "{path}");
        assert_eq!(
            response.headers().get("x-lifecycle-phase").unwrap(),
            "serving"
        );
        assert_eq!(
            response
                .headers()
                .get("x-lifecycle-generation")
                .unwrap()
                .to_str()
                .unwrap(),
            observation.generation.to_string()
        );
        assert_eq!(
            response.headers().get("x-lifecycle-reason-code").unwrap(),
            "serving"
        );
        if path == "/docs" {
            let body = String::from_utf8(
                response
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes()
                    .to_vec(),
            )
            .unwrap();
            assert!(body.contains("SwaggerUIBundle"));
        }
    }
}

#[tokio::test]
async fn lifecycle_probe_routes_canonical_json_exact_bytes() {
    let c = LifecycleController::serving();
    let response = lifecycle_probe_routes_canonical_json(c, None, json)
        .oneshot(
            Request::builder()
                .uri("/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .as_ref(),
        json().as_bytes()
    );
}

#[tokio::test]
async fn reason_header_sanitizes_invalid_values() {
    for (reason, expected) in [
        ("", "unknown"),
        ("\u{0}\u{7f}\u{2603}", "unknown"),
        ("\u{0}\u{2603}safe-_.:42\u{7f}", "safe-_.:42"),
    ] {
        let c = LifecycleController::new();
        c.transition(LifecyclePhase::Serving, reason, "detail")
            .unwrap();
        let response = lifecycle_probe_routes(c, None, doc)
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            response
                .headers()
                .get("x-lifecycle-reason-code")
                .unwrap()
                .to_str()
                .unwrap(),
            expected
        );
    }
}
