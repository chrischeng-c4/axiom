// HANDWRITE-BEGIN gap="sift-otlp-gcp-ingest-tests" tracker="1658" reason="Verify golden JSON/protobuf/gzip payloads, partial success, duplicates, auth, body/schema/quota, and overload behavior using the real router and journal."
use std::{collections::HashMap, io::Write, sync::Arc};

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use flate2::{write::GzEncoder, Compression};
use prost14::Message;
use service_auth::{Role, TokenClaims};
use sift::{
    auth::{SiftAuthConfig, SiftVerifier},
    ingest::{
        otlp::wire::{
            any_value, AnyValue, ExportLogsServiceRequest, ExportTraceServiceRequest, KeyValue,
            LogRecord, Resource, ResourceLogs, ResourceSpans, ScopeLogs, ScopeSpans, Span,
        },
        IngestLimits,
    },
    projection::PROJECTION_TRACE_STORE,
    protected_router, router, AttributeValue, ServiceState, SignalKind,
};
use tower::ServiceExt;

fn log_fixture(message: &str) -> serde_json::Value {
    serde_json::json!({
        "resourceLogs": [{
            "resource": {"attributes": [
                {"key": "service.name", "value": {"stringValue": "checkout"}},
                {"key": "gcp.resource.type", "value": {"stringValue": "k8s_container"}},
                {"key": "gcp.project_id", "value": {"stringValue": "infra-project"}},
                {"key": "k8s.pod.name", "value": {"stringValue": "checkout-1"}}
            ]},
            "scopeLogs": [{
                "scope": {"name": "test.logs"},
                "logRecords": [{
                    "timeUnixNano": "1783987200000000000",
                    "severityText": "ERROR",
                    "traceId": "0af7651916cd43dd8448eb211c80319c",
                    "spanId": "b7ad6b7169203331",
                    "body": {"stringValue": message}
                }]
            }]
        }]
    })
}

async fn json_body(response: axum::response::Response) -> serde_json::Value {
    let bytes = to_bytes(response.into_body(), 8 * 1024 * 1024)
        .await
        .expect("read response body");
    serde_json::from_slice(&bytes).expect("response JSON")
}

#[tokio::test]
async fn otlp_logs_report_partial_success_and_preserve_gcp_resource_fields() {
    let temp = tempfile::tempdir().unwrap();
    let state = Arc::new(ServiceState::open(temp.path()).unwrap());
    let app = router(state.clone());
    let mut request = log_fixture("pod failed");
    request["resourceLogs"][0]["scopeLogs"][0]["logRecords"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"body": null}));
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/logs")
                .header("content-type", "application/json")
                .header("x-sift-project", "project-a")
                .body(Body::from(serde_json::to_vec(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["partialSuccess"]["rejectedLogRecords"], 1);

    let stored = state.journal().query(Default::default()).unwrap();
    let gcp = stored.first().expect("normalized OTLP log");
    assert_eq!(gcp.event.project, "project-a");
    assert_eq!(gcp.event.resource["gcp.resource.type"], "k8s_container");
    assert_eq!(gcp.event.resource["gcp.project_id"], "infra-project");
    assert_eq!(gcp.event.resource["k8s.pod.name"], "checkout-1");
    assert_eq!(gcp.event.severity.as_deref(), Some("ERROR"));
    assert_eq!(
        gcp.event.trace_id.as_deref(),
        Some("0af7651916cd43dd8448eb211c80319c")
    );
}

#[tokio::test]
async fn otlp_rejects_items_older_than_180_days_as_non_retryable_partial_success() {
    let temp = tempfile::tempdir().unwrap();
    let state = Arc::new(ServiceState::open(temp.path()).unwrap());
    let app = router(state.clone());
    let current = chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64;
    let expired = (chrono::Utc::now() - chrono::Duration::days(181))
        .timestamp_nanos_opt()
        .unwrap() as u64;
    let fixture = serde_json::json!({
        "resourceLogs": [{
            "resource": {"attributes": [{
                "key": "service.name", "value": {"stringValue": "checkout"}
            }]},
            "scopeLogs": [{"logRecords": [
                {"timeUnixNano": current.to_string(), "body": {"stringValue": "current"}},
                {"timeUnixNano": expired.to_string(), "body": {"stringValue": "expired"}}
            ]}]
        }]
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/logs")
                .header("content-type", "application/json")
                .header("x-sift-project", "project-a")
                .body(Body::from(serde_json::to_vec(&fixture).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["partialSuccess"]["rejectedLogRecords"], 1);
    assert!(body["partialSuccess"]["errorMessage"]
        .as_str()
        .unwrap()
        .contains("180-day retention"));
    assert_eq!(state.journal().query(Default::default()).unwrap().len(), 1);
}

#[tokio::test]
async fn otlp_json_endpoints_accept_logs_metrics_and_traces() {
    let temp = tempfile::tempdir().unwrap();
    let state = Arc::new(ServiceState::open(temp.path()).unwrap());
    let app = router(state.clone());
    let fixtures = [
        (
            "/v1/logs",
            serde_json::json!({"resourceLogs":[{"resource":{"attributes":[{"key":"service.name","value":{"stringValue":"checkout"}}]},"scopeLogs":[{"scope":{"name":"test.logs"},"logRecords":[{"timeUnixNano":"1783987200000000000","severityText":"INFO","body":{"stringValue":"ok"},"attributes":[{"key":"attempt","value":{"intValue":"7"}},{"key":"fingerprint","value":{"bytesValue":"AQI="}}]},{"body":null}]}]}]}),
        ),
        (
            "/v1/traces",
            serde_json::json!({"resourceSpans":[{"resource":{"attributes":[{"key":"service.name","value":{"stringValue":"checkout"}}]},"scopeSpans":[{"scope":{"name":"test.traces"},"spans":[{"traceId":"0af7651916cd43dd8448eb211c80319c","spanId":"b7ad6b7169203331","name":"GET /checkout","startTimeUnixNano":"1783987200000000000","endTimeUnixNano":"1783987200001000000"}]}]}]}),
        ),
        (
            "/v1/metrics",
            serde_json::json!({"resourceMetrics":[{"resource":{"attributes":[{"key":"service.name","value":{"stringValue":"checkout"}}]},"scopeMetrics":[{"scope":{"name":"test.metrics"},"metrics":[{"name":"checkout.duration","unit":"ms","gauge":{"dataPoints":[{"timeUnixNano":"1783987200000000000","asDouble":42.5}]}}]}]}]}),
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
    assert!(rows
        .iter()
        .any(|row| row.event.signal == SignalKind::Metric));
    let log = rows
        .iter()
        .find(|row| row.event.signal == SignalKind::Log)
        .expect("OTLP JSON log");
    assert_eq!(log.event.attributes["attempt"], AttributeValue::Int(7));
    assert_eq!(
        log.event.attributes["fingerprint"],
        AttributeValue::Bytes("AQI=".into())
    );
    state
        .projections()
        .catch_up(PROJECTION_TRACE_STORE)
        .expect("project accepted OTLP JSON trace");
    let trace = state
        .projections()
        .get_trace("project-a", "0af7651916cd43dd8448eb211c80319c")
        .expect("read projected OTLP JSON trace")
        .expect("projected OTLP JSON trace exists");
    assert_eq!(trace.spans.len(), 1);
    assert_eq!(trace.spans[0].name, "GET /checkout");
    assert_eq!(
        trace.spans[0].start_time_unix_nano,
        1_783_987_200_000_000_000
    );
    assert_eq!(trace.spans[0].end_time_unix_nano, 1_783_987_200_001_000_000);
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
                    value: Some(AnyValue {
                        value: Some(any_value::Value::StringValue("checkout".into())),
                    }),
                    ..Default::default()
                }],
                dropped_attributes_count: 0,
                entity_refs: Vec::new(),
            }),
            scope_logs: vec![ScopeLogs {
                scope: None,
                log_records: vec![LogRecord {
                    time_unix_nano: 1_783_987_200_000_000_000,
                    observed_time_unix_nano: 1_783_987_200_000_000_000,
                    severity_text: "WARN".into(),
                    body: Some(AnyValue {
                        value: Some(any_value::Value::StringValue("slow request".into())),
                    }),
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
async fn otlp_protobuf_trace_projects_canonical_timing_and_kind() {
    let temp = tempfile::tempdir().unwrap();
    let state = Arc::new(ServiceState::open(temp.path()).unwrap());
    let app = router(state.clone());
    let trace_id = hex::decode("0af7651916cd43dd8448eb211c80319c").unwrap();
    let span_id = hex::decode("b7ad6b7169203331").unwrap();
    let payload = ExportTraceServiceRequest {
        resource_spans: vec![ResourceSpans {
            resource: Some(Resource {
                attributes: vec![KeyValue {
                    key: "service.name".into(),
                    value: Some(AnyValue {
                        value: Some(any_value::Value::StringValue("checkout".into())),
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            scope_spans: vec![ScopeSpans {
                scope: None,
                spans: vec![Span {
                    trace_id,
                    span_id,
                    name: "POST /checkout".into(),
                    kind: opentelemetry_proto::tonic::trace::v1::span::SpanKind::Server as i32,
                    start_time_unix_nano: 1_783_987_200_000_000_000,
                    end_time_unix_nano: 1_783_987_200_002_000_000,
                    ..Default::default()
                }],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }],
    }
    .encode_to_vec();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/traces")
                .header("content-type", "application/x-protobuf")
                .header("x-sift-project", "project-a")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    state
        .projections()
        .catch_up(PROJECTION_TRACE_STORE)
        .expect("project accepted OTLP protobuf trace");
    let trace = state
        .projections()
        .get_trace("project-a", "0af7651916cd43dd8448eb211c80319c")
        .expect("read projected OTLP protobuf trace")
        .expect("projected OTLP protobuf trace exists");
    assert_eq!(trace.spans.len(), 1);
    assert_eq!(trace.spans[0].name, "POST /checkout");
    assert_eq!(trace.spans[0].kind.as_deref(), Some("SPAN_KIND_SERVER"));
    assert_eq!(trace.duration_unix_nano, 2_000_000);
}

#[tokio::test]
async fn project_auth_limits_quota_and_draining_return_explicit_errors() {
    let temp = tempfile::tempdir().unwrap();
    let limits = IngestLimits {
        max_compressed_body_bytes: 1024,
        max_decoded_body_bytes: 1024,
        max_event_bytes: 4096,
        max_events_per_batch: 4,
        max_concurrent_requests_per_project: 1,
        max_items_per_project_window: 1,
        quota_window_secs: 60,
        ..IngestLimits::default()
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

    let too_large = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/logs")
                .header("authorization", "Bearer project-a-token")
                .header("content-type", "application/json")
                .header("x-sift-project", "project-a")
                .body(Body::from(vec![b'x'; 1_025]))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(too_large.status(), StatusCode::PAYLOAD_TOO_LARGE);

    let forbidden = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/logs")
                .header("authorization", "Bearer project-a-token")
                .header("content-type", "application/json")
                .header("x-sift-project", "project-b")
                .body(Body::from(
                    serde_json::to_vec(&log_fixture("forbidden")).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);

    let removed_legacy_route = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("authorization", "Bearer project-a-token")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&log_fixture("legacy")).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(removed_legacy_route.status(), StatusCode::NOT_FOUND);

    let accepted_body = serde_json::to_vec(&log_fixture("accepted")).unwrap();
    let accepted = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/logs")
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
                .uri("/v1/logs")
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
                .uri("/v1/logs")
                .header("authorization", "Bearer project-a-token")
                .header("content-type", "application/json")
                .header("x-sift-project", "project-a")
                .body(Body::from(
                    serde_json::to_vec(&log_fixture("draining")).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(draining.status(), StatusCode::SERVICE_UNAVAILABLE);
}
// HANDWRITE-END
