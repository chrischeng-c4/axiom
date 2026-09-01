//! Gateway and query roles share the store role's one durable data plane.

use std::{
    net::TcpListener,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

use opentelemetry_proto::tonic::{
    collector::logs::v1::{logs_service_client::LogsServiceClient, ExportLogsServiceRequest},
    common::v1::{any_value, AnyValue, KeyValue},
    logs::v1::{LogRecord, ResourceLogs, ScopeLogs},
    resource::v1::Resource,
};
use tonic::Request as GrpcRequest;

struct Process(Child);

impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn spawn_role(
    role: &str,
    port: u16,
    data: &std::path::Path,
    grpc_port: Option<u16>,
    store: Option<&str>,
    store_grpc: Option<&str>,
    query: Option<&str>,
) -> Process {
    let mut command = Command::new(env!("CARGO_BIN_EXE_sift"));
    command
        .args([
            "serve",
            "--role",
            role,
            "--host",
            "127.0.0.1",
            "--port",
            &port.to_string(),
            "--data-dir",
        ])
        .arg(data)
        .env("SIFT_AUTH", "off")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(grpc_port) = grpc_port {
        command.args(["--grpc-port", &grpc_port.to_string()]);
    }
    if let Some(store) = store {
        command.env("SIFT_STORE_ENDPOINT", store);
    }
    if let Some(store_grpc) = store_grpc {
        command.env("SIFT_STORE_GRPC_ENDPOINT", store_grpc);
    }
    if let Some(query) = query {
        command.env("SIFT_QUERY_ENDPOINT", query);
    }
    Process(command.spawn().expect("start Sift role"))
}

fn grpc_log() -> ExportLogsServiceRequest {
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
                ..Default::default()
            }),
            scope_logs: vec![ScopeLogs {
                scope: None,
                log_records: vec![LogRecord {
                    time_unix_nano: now,
                    observed_time_unix_nano: now,
                    body: Some(AnyValue {
                        value: Some(any_value::Value::StringValue(
                            "routed through grpc gateway".into(),
                        )),
                    }),
                    ..Default::default()
                }],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }],
    }
}

async fn wait_ready(endpoint: &str) {
    let client = reqwest::Client::new();
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if client
            .get(format!("{endpoint}/readyz"))
            .send()
            .await
            .is_ok_and(|response| response.status().is_success())
        {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("{endpoint} did not become ready");
}

async fn query(endpoint: &str) -> serde_json::Value {
    reqwest::Client::new()
        .post(format!("{endpoint}/api/v1/query"))
        .header("x-sift-project", "project-a")
        .json(&serde_json::json!({
            "version":1,
            "project":"project-a",
            "signal":{"kind":"logs"},
            "limit":10,
            "mode":"sync"
        }))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap()
}

fn record_count(response: &serde_json::Value) -> usize {
    response["data"]["records"]
        .as_array()
        .map(Vec::len)
        .unwrap_or(0)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn gateway_ingest_and_query_use_the_store_role_source_of_truth() {
    let store_port = port();
    let store_grpc_port = port();
    let query_port = port();
    let gateway_port = port();
    let gateway_grpc_port = port();
    let store_endpoint = format!("http://127.0.0.1:{store_port}");
    let store_grpc_endpoint = format!("http://127.0.0.1:{store_grpc_port}");
    let query_endpoint = format!("http://127.0.0.1:{query_port}");
    let gateway_endpoint = format!("http://127.0.0.1:{gateway_port}");
    let store_data = tempfile::tempdir().unwrap();
    let query_data = tempfile::tempdir().unwrap();
    let gateway_data = tempfile::tempdir().unwrap();

    let _store = spawn_role(
        "store",
        store_port,
        store_data.path(),
        Some(store_grpc_port),
        None,
        None,
        None,
    );
    wait_ready(&store_endpoint).await;
    let _query = spawn_role(
        "query",
        query_port,
        query_data.path(),
        None,
        Some(&store_endpoint),
        None,
        None,
    );
    wait_ready(&query_endpoint).await;
    let _gateway = spawn_role(
        "gateway",
        gateway_port,
        gateway_data.path(),
        Some(gateway_grpc_port),
        Some(&store_endpoint),
        Some(&store_grpc_endpoint),
        Some(&query_endpoint),
    );
    wait_ready(&gateway_endpoint).await;

    let ingest = reqwest::Client::new()
        .post(format!("{gateway_endpoint}/v1/logs"))
        .header("x-sift-project", "project-a")
        .json(&serde_json::json!({
            "resourceLogs":[{
                "resource":{"attributes":[
                    {"key":"service.name","value":{"stringValue":"checkout"}},
                    {"key":"deployment.environment.name","value":{"stringValue":"prod"}}
                ]},
                "scopeLogs":[{"logRecords":[{
                    "body":{"stringValue":"routed through gateway"}
                }]}]
            }]
        }))
        .send()
        .await
        .unwrap();
    let ingest_status = ingest.status();
    let ingest_body = ingest.text().await.unwrap();
    assert!(ingest_status.is_success(), "ingest: {}", ingest_body);

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let stored = query(&store_endpoint).await;
        if record_count(&stored) != 0 {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "gateway did not write the store role; ingest={ingest_body}; store={stored}"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    let through_gateway = query(&gateway_endpoint).await;
    assert_eq!(record_count(&through_gateway), 1);

    let mut grpc = LogsServiceClient::connect(format!("http://127.0.0.1:{gateway_grpc_port}"))
        .await
        .unwrap();
    let mut request = GrpcRequest::new(grpc_log());
    request
        .metadata_mut()
        .insert("x-sift-project", "project-a".parse().unwrap());
    grpc.export(request).await.unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let stored = query(&store_endpoint).await;
        if record_count(&stored) == 2 {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "gateway OTLP/gRPC did not write the store role: {stored}"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn query_role_owns_async_job_state_across_its_restart() {
    let store_port = port();
    let query_port = port();
    let gateway_port = port();
    let store_endpoint = format!("http://127.0.0.1:{store_port}");
    let query_endpoint = format!("http://127.0.0.1:{query_port}");
    let gateway_endpoint = format!("http://127.0.0.1:{gateway_port}");
    let store_data = tempfile::tempdir().unwrap();
    let query_data = tempfile::tempdir().unwrap();
    let gateway_data = tempfile::tempdir().unwrap();

    let _store = spawn_role(
        "store",
        store_port,
        store_data.path(),
        Some(port()),
        None,
        None,
        None,
    );
    wait_ready(&store_endpoint).await;
    let mut query_process = Some(spawn_role(
        "query",
        query_port,
        query_data.path(),
        None,
        Some(&store_endpoint),
        None,
        None,
    ));
    wait_ready(&query_endpoint).await;
    let _gateway = spawn_role(
        "gateway",
        gateway_port,
        gateway_data.path(),
        Some(port()),
        Some(&store_endpoint),
        Some(&format!("http://127.0.0.1:{}", port())),
        Some(&query_endpoint),
    );
    wait_ready(&gateway_endpoint).await;

    let client = reqwest::Client::new();
    let accepted = client
        .post(format!("{gateway_endpoint}/api/v1/query"))
        .header("x-sift-project", "project-a")
        .json(&serde_json::json!({
            "version": 1,
            "project": "project-a",
            "signal": {"kind": "logs"},
            "limit": 10,
            "mode": "async"
        }))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    let query_id = accepted["query_id"].as_str().unwrap();
    let job_path = query_data
        .path()
        .join("query-jobs")
        .join(format!("{query_id}.json"));
    assert!(
        job_path.is_file(),
        "query role did not persist {} on its own data root",
        job_path.display()
    );

    drop(query_process.take());
    query_process = Some(spawn_role(
        "query",
        query_port,
        query_data.path(),
        None,
        Some(&store_endpoint),
        None,
        None,
    ));
    wait_ready(&query_endpoint).await;

    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let response = client
            .get(format!(
                "{gateway_endpoint}/api/v1/queries/{query_id}?project=project-a"
            ))
            .header("x-sift-project", "project-a")
            .send()
            .await;
        if let Ok(response) = response {
            if response.status().is_success() {
                let job = response.json::<serde_json::Value>().await.unwrap();
                assert!(matches!(
                    job["status"].as_str(),
                    Some("succeeded" | "failed")
                ));
                break;
            }
        }
        assert!(
            Instant::now() < deadline,
            "query job did not survive the query role restart"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    drop(query_process);
}
