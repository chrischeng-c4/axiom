// HANDWRITE-BEGIN gap="sift-otlp-gcp-ingest-tests" tracker="1658" reason="Verify golden JSON/protobuf/gzip payloads, partial success, duplicates, auth, body/schema/quota, and overload behavior using the real router and journal."
use std::{collections::HashMap, io::Write, sync::Arc};

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use flate2::{write::GzEncoder, Compression};
use prost::Message;
use service_auth::{Role, TokenClaims};
use sift::{
    auth::{SiftAuthConfig, SiftVerifier},
    ingest::{
        otlp::wire::{
            AnyValue, ExportLogsServiceRequest, KeyValue, LogRecord, Resource, ResourceLogs,
            ScopeLogs,
        },
        IngestLimits,
    },
    protected_router, router, EventEnvelope, ServiceState, SignalKind,
};
use tower::ServiceExt;

fn event(id: &str, project: &str) -> EventEnvelope {
    let mut event = EventEnvelope::for_project(
        project,
        "test",
        id,
        SignalKind::Log,
        serde_json::json!({"message":"accepted"}),
    );
    event
        .resource
        .insert("service.name".into(), "checkout".into());
    event
}

async fn json_body(response: axum::response::Response) -> serde_json::Value {
    let bytes = to_bytes(response.into_body(), 8 * 1024 * 1024)
        .await
        .expect("read response body");
    serde_json::from_slice(&bytes).expect("response JSON")
}

#[tokio::test]
async fn bounded_batch_preserves_outcomes_and_normalizes_gcp_structured_logs() {
    let temp = tempfile::tempdir().unwrap();
    let state = Arc::new(ServiceState::open(temp.path()).unwrap());
    let app = router(state.clone());
    let request = serde_json::json!({
        "events": [
            event("json-1", "project-a"),
            event("json-1", "project-a"),
            {"schema_version": 2, "event_id": "invalid"},
            {
                "insertId": "gcp-1",
                "timestamp": "2026-07-14T00:00:00Z",
                "severity": "ERROR",
                "jsonPayload": {"message": "pod failed", "attempt": 3},
                "resource": {
                    "type": "k8s_container",
                    "labels": {
                        "project_id": "project-a",
                        "cluster_name": "prod",
                        "namespace_name": "payments",
                        "pod_name": "checkout-1",
                        "container_name": "app"
                    }
                },
                "trace": "projects/project-a/traces/0af7651916cd43dd8448eb211c80319c",
                "spanId": "b7ad6b7169203331",
                "httpRequest": {"requestId": "req-1"}
            }
        ]
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events:write")
                .header("content-type", "application/json")
                .header("x-sift-project", "project-a")
                .body(Body::from(serde_json::to_vec(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["accepted"], 2);
    assert_eq!(body["duplicates"], 1);
    assert_eq!(body["rejected"], 1);
    assert_eq!(body["results"][0]["outcome"], "accepted");
    assert_eq!(body["results"][1]["outcome"], "duplicate");
    assert_eq!(body["results"][2]["outcome"], "rejected");

    let stored = state.journal().query(Default::default()).unwrap();
    let gcp = stored
        .iter()
        .find(|row| row.event.event_id == "gcp-1")
        .expect("normalized GCP event");
    assert_eq!(gcp.event.resource["gcp.resource.type"], "k8s_container");
    assert_eq!(gcp.event.resource["k8s.pod.name"], "checkout-1");
    assert_eq!(gcp.event.severity.as_deref(), Some("ERROR"));
    assert_eq!(gcp.event.trace_id.as_deref(), Some("0af7651916cd43dd8448eb211c80319c"));
    assert_eq!(gcp.event.payload["jsonPayload"]["attempt"], 3);
}

#[tokio::test]
async fn otlp_json_endpoints_accept_all_signals_and_report_partial_success() {
    let temp = tempfile::tempdir().unwrap();
    let state = Arc::new(ServiceState::open(temp.path()).unwrap());
    let app = router(state.clone());
    let fixtures = [
        (
            "/v1/logs",
            serde_json::json!({"resourceLogs":[{"resource":{"attributes":[{"key":"service.name","value":{"stringValue":"checkout"}}]},"scopeLogs":[{"scope":{"name":"test.logs"},"logRecords":[{"timeUnixNano":"1783987200000000000","severityText":"INFO","body":{"stringValue":"ok"}},{"body":null}]}]}]}),
        ),
        (
            "/v1/traces",
            serde_json::json!({"resourceSpans":[{"resource":{"attributes":[{"key":"service.name","value":{"stringValue":"checkout"}}]},"scopeSpans":[{"scope":{"name":"test.traces"},"spans":[{"traceId":"0af7651916cd43dd8448eb211c80319c","spanId":"b7ad6b7169203331","name":"GET /checkout","startTimeUnixNano":"1783987200000000000","endTimeUnixNano":"1783987200001000000"}]}]}]}),
        ),
        (
            "/v1/metrics",
            serde_json::json!({"resourceMetrics":[{"resource":{"attributes":[{"key":"service.name","value":{"stringValue":"checkout"}}]},"scopeMetrics":[{"scope":{"name":"test.metrics"},"metrics":[{"name":"checkout.duration","unit":"ms","gauge":{"dataPoints":[{"timeUnixNano":"1783987200000000000","asDouble":42.5}]}}]}]}]}),
        ),
        (
            "/v1/profiles",
            serde_json::json!({"resourceProfiles":[{"resource":{"attributes":[{"key":"service.name","value":{"stringValue":"checkout"}}]},"scopeProfiles":[{"scope":{"name":"test.profiles"},"profiles":[{"profileId":"cHJvZmlsZS0x","startTimeUnixNano":"1783987200000000000","durationNano":"1000000"}]}]}]}),
        ),
    ];

    for (path, fixture) in fixtures {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(path)
                    .header("content-type", "application/json")
                    .header("x-sift-project", "project-a")
                    .body(Body::from(serde_json::to_vec(&fixture).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK, "{path}");
        let body = json_body(response).await;
        if path == "/v1/logs" {
            assert_eq!(body["partialSuccess"]["rejectedLogRecords"], 1);
        }
    }

    let rows = state.journal().query(Default::default()).unwrap();
    assert!(rows.iter().any(|row| row.event.signal == SignalKind::Log));
    assert!(rows.iter().any(|row| row.event.signal == SignalKind::Span));
    assert!(rows.iter().any(|row| row.event.signal == SignalKind::Metric));
    assert!(rows.iter().any(|row| row.event.signal == SignalKind::Profile));
}

#[tokio::test]
async fn otlp_protobuf_gzip_round_trips_an_official_log_envelope() {
    let temp = tempfile::tempdir().unwrap();
    let state = Arc::new(ServiceState::open(temp.path()).unwrap());
    let app = router(state.clone());
    let payload = ExportLogsServiceRequest {
        resource_logs: vec![ResourceLogs {
            resource: Some(Resource {
                attributes: vec![KeyValue {
                    key: "service.name".into(),
                    value: Some(AnyValue::string("checkout")),
                }],
                dropped_attributes_count: 0,
            }),
            scope_logs: vec![ScopeLogs {
                scope: None,
                log_records: vec![LogRecord {
                    time_unix_nano: 1_783_987_200_000_000_000,
                    observed_time_unix_nano: 1_783_987_200_000_000_000,
                    severity_text: "WARN".into(),
                    body: Some(AnyValue::string("slow request")),
                    ..Default::default()
                }],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }],
    }
    .encode_to_vec();
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&payload).unwrap();
    let compressed = encoder.finish().unwrap();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/logs")
                .header("content-type", "application/x-protobuf")
                .header("content-encoding", "gzip")
                .header("x-sift-project", "project-a")
                .body(Body::from(compressed))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "application/x-protobuf");
    assert_eq!(state.journal().query(Default::default()).unwrap().len(), 1);
}

#[tokio::test]
async fn project_auth_limits_quota_and_draining_return_explicit_errors() {
    let temp = tempfile::tempdir().unwrap();
    let limits = IngestLimits {
        max_compressed_body_bytes: 1024,
        max_decoded_body_bytes: 1024,
        max_event_bytes: 512,
        max_events_per_batch: 4,
        max_concurrent_requests_per_project: 1,
        max_items_per_project_window: 1,
        quota_window_secs: 60,
    };
    let state = Arc::new(ServiceState::open_with_ingest_limits(temp.path(), limits).unwrap());
    let verifier = Arc::new(SiftVerifier::new(SiftAuthConfig {
        required: true,
        tokens: HashMap::from([(
            "project-a-token".into(),
            TokenClaims {
                subject: "collector".into(),
                roles: HashMap::from([("project-a".into(), Role::Write)]),
            },
        )]),
    }));
    let app = protected_router(state.clone(), verifier);

    let forbidden = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events:write")
                .header("authorization", "Bearer project-a-token")
                .header("content-type", "application/json")
                .header("x-sift-project", "project-b")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({"events":[event("b", "project-b")]}))
                        .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);

    let accepted_body = serde_json::to_vec(
        &serde_json::json!({"events":[event("a-1", "project-a")]}),
    )
    .unwrap();
    let accepted = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events:write")
                .header("authorization", "Bearer project-a-token")
                .header("content-type", "application/json")
                .header("x-sift-project", "project-a")
                .body(Body::from(accepted_body.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(accepted.status(), StatusCode::OK);

    let quota = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events:write")
                .header("authorization", "Bearer project-a-token")
                .header("content-type", "application/json")
                .header("x-sift-project", "project-a")
                .body(Body::from(accepted_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(quota.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(quota.headers().contains_key("retry-after"));

    state.start_drain();
    let draining = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events:write")
                .header("authorization", "Bearer project-a-token")
                .header("content-type", "application/json")
                .header("x-sift-project", "project-c")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({"events":[event("c", "project-c")]}))
                        .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(draining.status(), StatusCode::SERVICE_UNAVAILABLE);
}
// HANDWRITE-END
