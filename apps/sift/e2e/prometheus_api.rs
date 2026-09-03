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

async fn write_remote_series(app: &axum::Router, timeseries: Vec<TimeSeries>) {
    let request = WriteRequest {
        timeseries,
        metadata: Vec::new(),
    };
    let compressed = metrics_remote_write::encode_snappy(&request.encode_to_vec()).unwrap();
    let response = app
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
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

fn decimal_seconds(nanos: i64) -> String {
    let magnitude = i128::from(nanos).abs();
    let sign = if nanos < 0 { "-" } else { "" };
    format!(
        "{sign}{}.{:09}",
        magnitude / 1_000_000_000,
        magnitude % 1_000_000_000
    )
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
        range["data"]["result"][0]["values"],
        serde_json::json!([
            [start_seconds as f64, "2"],
            [(start_seconds + 1) as f64, "5"],
            [(start_seconds + 2) as f64, "5"]
        ])
    );

    let fractional_end = start_seconds as f64 + 0.3;
    let fractional = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/prometheus/api/v1/query_range?project=project-a&environment=prod&query=http_requests_total&start={start_seconds}&end={fractional_end}&step=0.1"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(fractional.status(), StatusCode::OK);
    let fractional = body_json(fractional).await;
    let fractional_values = fractional["data"]["result"][0]["values"]
        .as_array()
        .unwrap();
    assert_eq!(fractional_values.len(), 4);
    let last_evaluation = fractional_values[3][0].as_f64().unwrap();
    assert!((last_evaluation - fractional_end).abs() < 0.000_001);

    let rate = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/prometheus/api/v1/query_range?project=project-a&environment=prod&query=rate(http_requests_total)&start={start_seconds}&end={}&step=1",
                    start_seconds + 2
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rate.status(), StatusCode::OK);
    let rate = body_json(rate).await;
    assert_eq!(
        rate["data"]["result"][0]["values"],
        serde_json::json!([
            [(start_seconds + 1) as f64, "3"],
            [(start_seconds + 2) as f64, "3"]
        ])
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
async fn otlp_submillisecond_lookback_uses_an_exact_open_lower_bound() {
    let temp = tempfile::tempdir().unwrap();
    let app = router(Arc::new(ServiceState::open(temp.path()).unwrap()));
    let sample_nanos = (chrono::Utc::now().timestamp() - 301) * 1_000_000_000 + 999_600;
    let evaluation_nanos = sample_nanos + 300_000_000_000;
    let fixture = serde_json::json!({
        "resourceMetrics": [{
            "scopeMetrics": [{
                "metrics": [{
                    "name": "boundary_excluded_gauge",
                    "gauge": {"dataPoints": [{
                        "timeUnixNano": sample_nanos.to_string(),
                        "asDouble": 42.5
                    }]}
                }, {
                    "name": "boundary_visible_gauge",
                    "gauge": {"dataPoints": [{
                        "timeUnixNano": (sample_nanos + 1).to_string(),
                        "asDouble": 43.5
                    }]}
                }]
            }]
        }]
    });
    let written = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/metrics")
                .header("content-type", "application/json")
                .header("x-sift-project", "project-a")
                .body(Body::from(serde_json::to_vec(&fixture).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(written.status(), StatusCode::OK);

    let excluded = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/prometheus/api/v1/query_range?project=project-a&query=boundary_excluded_gauge&start={}&end={}&step=1",
                    decimal_seconds(evaluation_nanos),
                    decimal_seconds(evaluation_nanos + 1)
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(excluded.status(), StatusCode::OK);
    let excluded = body_json(excluded).await;
    assert_eq!(excluded["data"]["result"], serde_json::json!([]));

    let visible = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/prometheus/api/v1/query_range?project=project-a&query=boundary_visible_gauge&start={}&end={}&step=1",
                    decimal_seconds(evaluation_nanos),
                    decimal_seconds(evaluation_nanos + 1)
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(visible.status(), StatusCode::OK);
    let visible = body_json(visible).await;
    assert_eq!(visible["data"]["result"][0]["values"][0][1], "43.5");
}

#[tokio::test]
async fn instant_and_range_queries_include_the_maximum_nanosecond() {
    let temp = tempfile::tempdir().unwrap();
    let app = router(Arc::new(ServiceState::open(temp.path()).unwrap()));
    let fixture = serde_json::json!({
        "resourceMetrics": [{
            "scopeMetrics": [{
                "metrics": [{
                    "name": "max_time_gauge",
                    "gauge": {"dataPoints": [{
                        "timeUnixNano": i64::MAX.to_string(),
                        "asDouble": 7.0
                    }]}
                }]
            }]
        }]
    });
    let written = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/metrics")
                .header("content-type", "application/json")
                .header("x-sift-project", "project-a")
                .body(Body::from(serde_json::to_vec(&fixture).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(written.status(), StatusCode::OK);

    let maximum = decimal_seconds(i64::MAX);
    let instant = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/prometheus/api/v1/query?project=project-a&query=max_time_gauge&time={maximum}"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(instant.status(), StatusCode::OK);
    let instant = body_json(instant).await;
    assert_eq!(instant["data"]["result"][0]["value"][1], "7");

    let range = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/prometheus/api/v1/query_range?project=project-a&query=max_time_gauge&start={}&end={maximum}&step=0.000000001",
                    decimal_seconds(i64::MAX - 1)
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(range.status(), StatusCode::OK);
    let range = body_json(range).await;
    assert_eq!(
        range["data"]["result"][0]["values"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(range["data"]["result"][0]["values"][0][1], "7");
}

#[tokio::test]
async fn instant_and_range_queries_accept_the_minimum_nanosecond() {
    let temp = tempfile::tempdir().unwrap();
    let app = router(Arc::new(ServiceState::open(temp.path()).unwrap()));

    let minimum = decimal_seconds(i64::MIN);
    let instant = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/prometheus/api/v1/query?project=project-a&query=min_time_gauge&time={minimum}"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(instant.status(), StatusCode::OK);
    let instant = body_json(instant).await;
    assert_eq!(instant["data"]["result"], serde_json::json!([]));

    let range = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/prometheus/api/v1/query_range?project=project-a&query=min_time_gauge&start={minimum}&end={}&step=1",
                    decimal_seconds(i64::MIN + 1)
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(range.status(), StatusCode::OK);
    let range = body_json(range).await;
    assert_eq!(range["data"]["result"], serde_json::json!([]));
}

#[tokio::test]
async fn range_query_evaluates_each_step_before_aggregating_series() {
    let temp = tempfile::tempdir().unwrap();
    let app = router(Arc::new(ServiceState::open(temp.path()).unwrap()));
    let start_seconds = chrono::Utc::now().timestamp() - 2;
    let labels = |instance: &str| {
        vec![
            Label {
                name: "__name__".into(),
                value: "range_gauge".into(),
            },
            Label {
                name: "environment".into(),
                value: "prod".into(),
            },
            Label {
                name: "instance".into(),
                value: instance.into(),
            },
            Label {
                name: "project".into(),
                value: "project-a".into(),
            },
        ]
    };
    let request = WriteRequest {
        timeseries: vec![
            TimeSeries {
                labels: labels("a"),
                samples: vec![
                    Sample {
                        value: 1.0,
                        timestamp: start_seconds * 1_000,
                    },
                    Sample {
                        value: 2.0,
                        timestamp: start_seconds * 1_000 + 500,
                    },
                ],
                exemplars: Vec::new(),
            },
            TimeSeries {
                labels: labels("b"),
                samples: vec![
                    Sample {
                        value: 10.0,
                        timestamp: start_seconds * 1_000,
                    },
                    Sample {
                        value: 20.0,
                        timestamp: start_seconds * 1_000 + 500,
                    },
                ],
                exemplars: Vec::new(),
            },
        ],
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

    let range = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/prometheus/api/v1/query_range?project=project-a&environment=prod&query=sum(range_gauge)&start={start_seconds}&end={}&step=1",
                    start_seconds + 2
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(range.status(), StatusCode::OK);
    let range = body_json(range).await;
    assert_eq!(
        range["data"]["result"][0]["values"],
        serde_json::json!([
            [start_seconds as f64, "11"],
            [(start_seconds + 1) as f64, "22"],
            [(start_seconds + 2) as f64, "22"]
        ])
    );
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
        body_json(range).await["data"]["result"][0]["values"],
        serde_json::json!([
            [start_seconds as f64, "7"],
            [(start_seconds + 2) as f64, "9"],
            [(start_seconds + 3) as f64, "9"]
        ])
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

#[tokio::test]
async fn prometheus_queries_reject_truncated_series_and_excessive_range_work() {
    let temp = tempfile::tempdir().unwrap();
    let app = router(Arc::new(ServiceState::open(temp.path()).unwrap()));
    let sample_seconds = chrono::Utc::now().timestamp() - 1;
    let make_series = |index: usize| TimeSeries {
        labels: vec![
            Label {
                name: "__name__".into(),
                value: "cardinality_gauge".into(),
            },
            Label {
                name: "budget".into(),
                value: if index < 1_000 { "yes" } else { "no" }.into(),
            },
            Label {
                name: "instance".into(),
                value: index.to_string(),
            },
            Label {
                name: "project".into(),
                value: "project-a".into(),
            },
        ],
        samples: vec![Sample {
            value: index as f64,
            timestamp: sample_seconds * 1_000,
        }],
        exemplars: Vec::new(),
    };

    write_remote_series(&app, (0..1_000).map(&make_series).collect()).await;
    write_remote_series(&app, vec![make_series(1_000)]).await;

    for query in ["cardinality_gauge", "sum(cardinality_gauge)"] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/prometheus/api/v1/query?project=project-a&query={query}&time={sample_seconds}"
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let response = body_json(response).await;
        assert_eq!(response["error"], "bad_data");
        assert!(response["message"]
            .as_str()
            .unwrap()
            .contains("more than 1000 series"));
    }

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/prometheus/api/v1/query_range?project=project-a&query=cardinality_gauge%7Bbudget%3D%22yes%22%7D&start={sample_seconds}&end={}&step=1",
                    sample_seconds + 1_000
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let response = body_json(response).await;
    assert_eq!(response["error"], "bad_data");
    assert!(response["message"]
        .as_str()
        .unwrap()
        .contains("the limit is 1000000"));
}
