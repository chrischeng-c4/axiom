// HANDWRITE-BEGIN gap="missing-generator:unit-test:14ba48e5" tracker="#2490" reason="Live-router coverage for Server-Timing header presence/parseability, phase-append round trip, and disclosure posture."
use std::time::Duration;

use axum::body::Body;
use axum::extract::Extension;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use service_http::{server_timing_middleware, ServerTimingDisclosure, ServerTimingExt};
use tower::ServiceExt as _;

/// Parsed `name -> dur_ms` map from a `Server-Timing` header value, plus the
/// original entry order (the spec renders metrics in the order the server
/// emitted them, and callers care that phases stay in push order).
struct ParsedServerTiming {
    order: Vec<String>,
    dur_ms: std::collections::HashMap<String, f64>,
}

fn parse_server_timing(value: &str) -> ParsedServerTiming {
    let mut order = Vec::new();
    let mut dur_ms = std::collections::HashMap::new();
    for entry in value.split(',') {
        let entry = entry.trim();
        let (name, rest) = entry
            .split_once(';')
            .unwrap_or_else(|| panic!("Server-Timing entry missing ';dur=': {entry:?}"));
        let dur = rest
            .strip_prefix("dur=")
            .unwrap_or_else(|| panic!("Server-Timing entry missing dur=: {entry:?}"));
        let dur: f64 = dur
            .parse()
            .unwrap_or_else(|e| panic!("Server-Timing dur value {dur:?} did not parse: {e}"));
        order.push(name.to_string());
        dur_ms.insert(name.to_string(), dur);
    }
    ParsedServerTiming { order, dur_ms }
}

async fn slow_ok() -> &'static str {
    tokio::time::sleep(Duration::from_millis(5)).await;
    "ok"
}

async fn pushes_phases_without_disclosure(Extension(ext): Extension<ServerTimingExt>) -> Response {
    ext.push("search", Duration::from_millis(3));
    ext.push("rank", Duration::from_millis(2));
    "ok".into_response()
}

async fn pushes_phases_with_full_disclosure(
    Extension(ext): Extension<ServerTimingExt>,
) -> Response {
    ext.push("search", Duration::from_millis(3));
    ext.push("rank", Duration::from_millis(2));
    let mut response = "ok".into_response();
    response
        .extensions_mut()
        .insert(ServerTimingDisclosure::Full);
    response
}

fn app(handler: axum::routing::MethodRouter) -> Router {
    Router::new()
        .route("/", handler)
        .layer(axum::middleware::from_fn(server_timing_middleware))
}

#[tokio::test(flavor = "current_thread")]
async fn header_is_present_and_parseable_on_a_live_router() {
    let response = app(get(slow_ok))
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let header = response
        .headers()
        .get("server-timing")
        .expect("Server-Timing header present")
        .to_str()
        .expect("Server-Timing header is valid ASCII");
    let parsed = parse_server_timing(header);

    assert_eq!(parsed.order, vec!["app"], "baseline-only by default");
    let app_dur = parsed.dur_ms["app"];
    assert!(
        app_dur >= 4.0,
        "app;dur= should reflect the handler's ~5ms sleep, got {app_dur}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn default_posture_hides_pushed_phases() {
    let response = app(get(pushes_phases_without_disclosure))
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let header = response
        .headers()
        .get("server-timing")
        .expect("Server-Timing header present")
        .to_str()
        .unwrap();
    let parsed = parse_server_timing(header);

    assert_eq!(
        parsed.order,
        vec!["app"],
        "TotalOnly is the default: pushed phases must not leak into the header \
         without an explicit ServerTimingDisclosure::Full response marker: {header:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn full_disclosure_reveals_phases_in_push_order_after_baseline() {
    let response = app(get(pushes_phases_with_full_disclosure))
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let header = response
        .headers()
        .get("server-timing")
        .expect("Server-Timing header present")
        .to_str()
        .unwrap();
    let parsed = parse_server_timing(header);

    assert_eq!(
        parsed.order,
        vec!["app", "search", "rank"],
        "app first, then phases in push order: {header:?}"
    );
    assert!((parsed.dur_ms["search"] - 3.0).abs() < 0.5);
    assert!((parsed.dur_ms["rank"] - 2.0).abs() < 0.5);
}

#[tokio::test(flavor = "current_thread")]
async fn phase_append_extension_is_per_request_not_shared_across_calls() {
    let router = app(get(pushes_phases_with_full_disclosure));

    for _ in 0..2 {
        let response = router
            .clone()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let header = response
            .headers()
            .get("server-timing")
            .unwrap()
            .to_str()
            .unwrap();
        let parsed = parse_server_timing(header);
        // If the collector leaked across requests, the second call would
        // render "search" and "rank" twice.
        assert_eq!(parsed.order, vec!["app", "search", "rank"], "{header:?}");
    }
}

#[tokio::test(flavor = "current_thread")]
async fn disallowed_phase_name_bytes_are_sanitized_not_dropped() {
    async fn handler(Extension(ext): Extension<ServerTimingExt>) -> Response {
        ext.push("db query;name,weird", Duration::from_millis(1));
        let mut response = "ok".into_response();
        response
            .extensions_mut()
            .insert(ServerTimingDisclosure::Full);
        response
    }

    let response = app(get(handler))
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let header = response
        .headers()
        .get("server-timing")
        .unwrap()
        .to_str()
        .unwrap();

    // The raw name would have broken the comma/semicolon-delimited grammar;
    // the sanitized form must still parse as exactly two entries.
    let parsed = parse_server_timing(header);
    assert_eq!(parsed.order.len(), 2, "{header:?}");
    assert_eq!(parsed.order[0], "app");
    assert!(
        !parsed.order[1].contains(';') && !parsed.order[1].contains(','),
        "sanitized phase name still contains header-breaking bytes: {:?}",
        parsed.order[1]
    );
}
// HANDWRITE-END
