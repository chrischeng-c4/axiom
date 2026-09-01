use std::sync::Arc;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use prost::Message;
use sift::{
    prometheus::{
        remote::{Label, Sample, TimeSeries, WriteRequest},
        PROMETHEUS_STALE_NAN_BITS,
    },
    router, ServiceState,
};
use tower::ServiceExt;

async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn remote_write_one_and_promql_query_round_trip() {
    let temp = tempfile::tempdir().unwrap();
    let app = router(Arc::new(ServiceState::open(temp.path()).unwrap()));
    let start_seconds = chrono::Utc::now().timestamp() - 1;
    let request = WriteRequest {
        timeseries: vec![TimeSeries {
            labels: vec![
                Label {
                    name: "__name__".into(),
                    value: "http_requests_total".into(),
                },
                Label {
                    name: "environment".into(),
                    value: "prod".into(),
                },
                Label {
                    name: "method".into(),
                    value: "POST".into(),
                },
                Label {
                    name: "project".into(),
                    value: "project-a".into(),
                },
                Label {
                    name: "service.name".into(),
                    value: "checkout".into(),
                },
            ],
            samples: vec![
                Sample {
                    value: 2.0,
                    timestamp: start_seconds * 1_000,
                },
                Sample {
                    value: 5.0,
                    timestamp: (start_seconds + 1) * 1_000,
                },
            ],
            exemplars: Vec::new(),
        }],
        metadata: Vec::new(),
    };
    let compressed = metrics_remote_write::encode_snappy(&request.encode_to_vec()).unwrap();
    let written = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/prometheus/api/v1/write")
                .header("content-type", "application/x-protobuf")
                .header("content-encoding", "snappy")
                .header("x-prometheus-remote-write-version", "0.1.0")
                .body(Body::from(compressed))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(written.status(), StatusCode::NO_CONTENT);

    let instant = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/prometheus/api/v1/query?project=project-a&environment=prod&query=sum(http_requests_total%7Bmethod%3D%22POST%22%7D)&time={}", start_seconds + 1))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(instant.status(), StatusCode::OK);
    let instant = body_json(instant).await;
    assert_eq!(instant["status"], "success");
    assert_eq!(instant["data"]["resultType"], "vector");
    assert_eq!(instant["data"]["result"][0]["value"][1], "5");

    let range = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/prometheus/api/v1/query_range?project=project-a&environment=prod&query=http_requests_total&start={start_seconds}&end={}&step=1", start_seconds + 2))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(range.status(), StatusCode::OK);
    let range = body_json(range).await;
    assert_eq!(range["data"]["resultType"], "matrix");
    assert_eq!(
        range["data"]["result"][0]["values"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    let rejected_v2 = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/prometheus/api/v1/write")
                .header(
                    "content-type",
                    "application/x-protobuf;proto=io.prometheus.write.v2.Request",
                )
                .header("content-encoding", "snappy")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rejected_v2.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
async fn remote_write_stale_marker_survives_restart_and_hides_the_series() {
    let temp = tempfile::tempdir().unwrap();
    let start_seconds = chrono::Utc::now().timestamp() - 10;
    let request = WriteRequest {
        timeseries: vec![TimeSeries {
            labels: vec![
                Label {
                    name: "__name__".into(),
                    value: "queue_depth".into(),
                },
                Label {
                    name: "environment".into(),
                    value: "prod".into(),
                },
                Label {
                    name: "project".into(),
                    value: "project-a".into(),
                },
            ],
            samples: vec![
                Sample {
                    value: 7.0,
                    timestamp: start_seconds * 1_000,
                },
                Sample {
                    value: f64::from_bits(PROMETHEUS_STALE_NAN_BITS),
                    timestamp: (start_seconds + 1) * 1_000,
                },
                Sample {
                    value: 9.0,
                    timestamp: (start_seconds + 2) * 1_000,
                },
            ],
            exemplars: Vec::new(),
        }],
        metadata: Vec::new(),
    };
    let compressed = metrics_remote_write::encode_snappy(&request.encode_to_vec()).unwrap();

    let state = Arc::new(ServiceState::open(temp.path()).unwrap());
    let app = router(state.clone());
    let written = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/prometheus/api/v1/write")
                .header("content-type", "application/x-protobuf")
                .header("content-encoding", "snappy")
                .header("x-prometheus-remote-write-version", "0.1.0")
                .body(Body::from(compressed))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(written.status(), StatusCode::NO_CONTENT);

    let stale = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/prometheus/api/v1/query?project=project-a&environment=prod&query=queue_depth&time={}",
                    start_seconds + 1
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(stale.status(), StatusCode::OK);
    assert_eq!(
        body_json(stale).await["data"]["result"]
            .as_array()
            .unwrap()
            .len(),
        0
    );

    let reappeared = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/prometheus/api/v1/query?project=project-a&environment=prod&query=queue_depth&time={}",
                    start_seconds + 2
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reappeared.status(), StatusCode::OK);
    assert_eq!(
        body_json(reappeared).await["data"]["result"][0]["value"][1],
        "9"
    );

    let range = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/prometheus/api/v1/query_range?project=project-a&environment=prod&query=queue_depth&start={start_seconds}&end={}&step=1",
                    start_seconds + 3
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(range.status(), StatusCode::OK);
    assert_eq!(
        body_json(range).await["data"]["result"][0]["values"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    let invalid_nan = WriteRequest {
        timeseries: vec![TimeSeries {
            labels: vec![
                Label {
                    name: "__name__".into(),
                    value: "queue_depth".into(),
                },
                Label {
                    name: "environment".into(),
                    value: "prod".into(),
                },
                Label {
                    name: "project".into(),
                    value: "project-a".into(),
                },
            ],
            samples: vec![Sample {
                value: f64::NAN,
                timestamp: (start_seconds + 3) * 1_000,
            }],
            exemplars: Vec::new(),
        }],
        metadata: Vec::new(),
    };
    let invalid_nan = metrics_remote_write::encode_snappy(&invalid_nan.encode_to_vec()).unwrap();
    let rejected_nan = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/prometheus/api/v1/write")
                .header("content-type", "application/x-protobuf")
                .header("content-encoding", "snappy")
                .header("x-prometheus-remote-write-version", "0.1.0")
                .body(Body::from(invalid_nan))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rejected_nan.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        body_json(rejected_nan).await["error"],
        "invalid_remote_write"
    );

    state.finish_drain().await.unwrap();
    drop(app);
    drop(state);
    let reopened = router(Arc::new(ServiceState::open(temp.path()).unwrap()));
    let stale_after_restart = reopened
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/prometheus/api/v1/query?project=project-a&environment=prod&query=queue_depth&time={}",
                    start_seconds + 1
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(stale_after_restart.status(), StatusCode::OK);
    assert_eq!(
        body_json(stale_after_restart).await["data"]["result"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
}
