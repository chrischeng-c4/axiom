use std::sync::Arc;

use opentelemetry_proto::tonic::{
    collector::logs::v1::{logs_service_client::LogsServiceClient, ExportLogsServiceRequest},
    common::v1::{any_value, AnyValue, KeyValue},
    logs::v1::{LogRecord, ResourceLogs, ScopeLogs},
    resource::v1::Resource,
};
use sift::{
    auth::{SiftAuthConfig, SiftVerifier},
    EventQuery, ServiceState, SignalKind,
};
use tonic::{codec::CompressionEncoding, Request};

fn request() -> ExportLogsServiceRequest {
    let now = u64::try_from(
        chrono::Utc::now()
            .timestamp_nanos_opt()
            .expect("current time is representable as OTLP nanoseconds"),
    )
    .expect("current OTLP timestamp is non-negative");
    ExportLogsServiceRequest {
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
                log_records: vec![
                    LogRecord {
                        time_unix_nano: now,
                        observed_time_unix_nano: now,
                        severity_text: "ERROR".into(),
                        body: Some(AnyValue {
                            value: Some(any_value::Value::StringValue("failed".into())),
                        }),
                        ..Default::default()
                    },
                    LogRecord {
                        time_unix_nano: now + 1,
                        ..Default::default()
                    },
                ],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }],
    }
}

#[tokio::test]
async fn official_otlp_grpc_types_gzip_and_partial_success_work() {
    let temp = tempfile::tempdir().unwrap();
    let state = Arc::new(ServiceState::open(temp.path()).unwrap());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let verifier = Arc::new(SiftVerifier::new(SiftAuthConfig::open()));
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let server_state = state.clone();
    let server = tokio::spawn(async move {
        sift::grpc::serve(listener, server_state, verifier, async {
            let _ = shutdown_rx.await;
        })
        .await
        .unwrap();
    });

    let mut client = LogsServiceClient::connect(format!("http://{address}"))
        .await
        .unwrap()
        .send_compressed(CompressionEncoding::Gzip)
        .accept_compressed(CompressionEncoding::Gzip);
    let mut grpc_request = Request::new(request());
    grpc_request
        .metadata_mut()
        .insert("x-sift-project", "project-a".parse().unwrap());
    let response = client.export(grpc_request).await.unwrap().into_inner();
    let partial = response.partial_success.unwrap();
    assert_eq!(partial.rejected_log_records, 1);
    assert!(partial.error_message.contains("body is required"));

    let logs = state
        .journal()
        .query(EventQuery {
            signal: Some(SignalKind::Log),
            after: 0,
            limit: 10,
        })
        .unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].event.project, "project-a");

    shutdown_tx.send(()).unwrap();
    server.await.unwrap();
}

#[tokio::test]
async fn grpc_gateway_proxy_preserves_gzip_and_partial_success() {
    let temp = tempfile::tempdir().unwrap();
    let state = Arc::new(ServiceState::open(temp.path()).unwrap());
    let store_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let store_address = store_listener.local_addr().unwrap();
    let proxy_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_address = proxy_listener.local_addr().unwrap();
    let verifier = Arc::new(SiftVerifier::new(SiftAuthConfig::open()));
    let (store_shutdown_tx, store_shutdown_rx) = tokio::sync::oneshot::channel();
    let (proxy_shutdown_tx, proxy_shutdown_rx) = tokio::sync::oneshot::channel();

    let server_state = state.clone();
    let store_verifier = verifier.clone();
    let store = tokio::spawn(async move {
        sift::grpc::serve(store_listener, server_state, store_verifier, async {
            let _ = store_shutdown_rx.await;
        })
        .await
        .unwrap();
    });
    let proxy = tokio::spawn(async move {
        sift::grpc::serve_proxy(
            proxy_listener,
            &format!("http://{store_address}"),
            verifier,
            1024 * 1024,
            async {
                let _ = proxy_shutdown_rx.await;
            },
        )
        .await
        .unwrap();
    });

    let mut client = LogsServiceClient::connect(format!("http://{proxy_address}"))
        .await
        .unwrap()
        .send_compressed(CompressionEncoding::Gzip)
        .accept_compressed(CompressionEncoding::Gzip);
    let mut grpc_request = Request::new(request());
    grpc_request
        .metadata_mut()
        .insert("x-sift-project", "project-a".parse().unwrap());
    let response = client.export(grpc_request).await.unwrap().into_inner();
    assert_eq!(response.partial_success.unwrap().rejected_log_records, 1);
    assert_eq!(state.journal().total_event_count(), 1);

    proxy_shutdown_tx.send(()).unwrap();
    store_shutdown_tx.send(()).unwrap();
    proxy.await.unwrap();
    store.await.unwrap();
}

#[tokio::test]
async fn hidden_acceptance_grpc_client_uses_the_public_otlp_service() {
    let temp = tempfile::tempdir().unwrap();
    let state = Arc::new(ServiceState::open(temp.path()).unwrap());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let verifier = Arc::new(SiftVerifier::new(SiftAuthConfig::open()));
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let server_state = state.clone();
    let server = tokio::spawn(async move {
        sift::grpc::serve(listener, server_state, verifier, async {
            let _ = shutdown_rx.await;
        })
        .await
        .unwrap();
    });

    let output = tokio::process::Command::new(env!("CARGO_BIN_EXE_sift"))
        .args([
            "acceptance-grpc",
            "--endpoint",
            &format!("http://{address}"),
            "--project",
            "project-a",
        ])
        .output()
        .await
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["accepted"], 1);
    assert_eq!(result["rejected"], 1);
    assert_eq!(result["compression"], "gzip");

    let logs = state
        .journal()
        .query(EventQuery {
            signal: Some(SignalKind::Log),
            after: 0,
            limit: 10,
        })
        .unwrap();
    assert_eq!(logs.len(), 1);

    shutdown_tx.send(()).unwrap();
    server.await.unwrap();
}
