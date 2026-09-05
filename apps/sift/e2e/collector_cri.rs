// HANDWRITE-BEGIN gap="missing-generator:unit-test:c01b686c" tracker="1675" reason="Prove framing, correlation, metadata, rotation/restart, dedupe, outage recovery, and loss against real Sift."
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde_json::{json, Value};

const TRACE_ID: &str = "0af7651916cd43dd8448eb211c80319c";
static PROCESS_TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

struct SiftProcess {
    child: Child,
    stderr: Arc<Mutex<Vec<String>>>,
    reader: Option<JoinHandle<()>>,
}

impl SiftProcess {
    fn spawn(port: u16, data_dir: &Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_sift"))
            .args([
                "serve",
                "--host",
                "127.0.0.1",
                "--port",
                &port.to_string(),
                "--data-dir",
                data_dir.to_str().unwrap(),
                "--log-level",
                "warn",
                "--log-format",
                "json",
            ])
            .env("SIFT_AUTH", "off")
            .env_remove("SIFT_TOKEN")
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn real Sift server");
        let stderr = Arc::new(Mutex::new(Vec::new()));
        let destination = Arc::clone(&stderr);
        let stream = child.stderr.take().expect("Sift stderr pipe");
        let reader = thread::spawn(move || {
            for line in BufReader::new(stream).lines().map_while(Result::ok) {
                destination.lock().unwrap().push(line);
            }
        });
        Self {
            child,
            stderr,
            reader: Some(reader),
        }
    }

    fn stderr_text(&self) -> String {
        self.stderr.lock().unwrap().join("\n")
    }
}

impl Drop for SiftProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

fn reserve_port() -> u16 {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("reserve loopback port");
    listener.local_addr().unwrap().port()
}

async fn wait_ready(process: &mut SiftProcess, client: &reqwest::Client, base: &str) {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if let Ok(response) = client.get(format!("{base}/healthz")).send().await {
            if response.status().is_success() {
                return;
            }
        }
        if let Some(status) = process.child.try_wait().expect("poll Sift child") {
            panic!(
                "Sift exited before readiness with {status}: {}",
                process.stderr_text()
            );
        }
        assert!(
            Instant::now() < deadline,
            "Sift readiness timeout: {}",
            process.stderr_text()
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

fn service_event(event: &str, message: &str, trace_id: Option<&str>, span_id: &str) -> Value {
    let mut value = json!({
        "schema": "axiom.service.log.v1",
        "timestamp": "2026-07-17T10:00:00Z",
        "severity": "INFO",
        "service": { "name": "lumen", "version": "0.4.21" },
        "event": event,
        "message": message,
        "span_id": span_id,
        "trace_flags": "01",
        "attributes": { "collection_id": "docs", "component": "lumen.audit" }
    });
    if let Some(trace_id) = trace_id {
        value["trace_id"] = Value::String(trace_id.to_string());
        value["parent_span_id"] = Value::String("00f067aa0ba902b7".to_string());
    }
    value
}

fn pod_log(root: &Path) -> PathBuf {
    let directory = root.join("prod_lumen-0_1234-abcd").join("lumen");
    std::fs::create_dir_all(&directory).unwrap();
    directory.join("0.log")
}

fn cri_line(stream: &str, tag: &str, content: &str) -> String {
    format!("2026-07-17T10:00:00.123456789Z {stream} {tag} {content}\n")
}

fn collector_command(root: &Path, checkpoint: &Path, quarantine: &Path, endpoint: &str) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_sift"));
    command.args([
        "collect",
        "--cri-root",
        root.to_str().unwrap(),
        "--endpoint",
        endpoint,
        "--project",
        "project-a",
        "--environment",
        "prod",
        "--gcp-project",
        "project-a",
        "--cluster",
        "cluster-a",
        "--location",
        "asia-east1",
        "--node",
        "node-a",
        "--checkpoint",
        checkpoint.to_str().unwrap(),
        "--quarantine",
        quarantine.to_str().unwrap(),
        "--batch-size",
        "20",
        "--max-retries",
        "0",
        "--request-timeout-secs",
        "1",
    ]);
    command.env_remove("SIFT_TOKEN");
    command
}

fn run_collector(root: &Path, checkpoint: &Path, quarantine: &Path, endpoint: &str) -> Output {
    collector_command(root, checkpoint, quarantine, endpoint)
        .output()
        .expect("run real CRI collector")
}

fn successful_summary(output: Output) -> Value {
    assert!(
        output.status.success(),
        "collector failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("collector terminal JSON")
}

async fn query_logs(client: &reqwest::Client, base: &str, trace_id: Option<&str>) -> Value {
    let mut query = json!({
        "version": 1,
        "project": "project-a",
        "environment": "prod",
        "signal": {"kind": "logs"},
        "limit": 50,
        "mode": "sync"
    });
    if let Some(trace_id) = trace_id {
        query["signal"]["filter"] = json!({
            "op": "eq",
            "field": "trace_id",
            "value": trace_id
        });
    }
    let response = client
        .post(format!("{base}/api/v1/query"))
        .json(&query)
        .send()
        .await
        .expect("query Sift logs");
    assert!(
        response.status().is_success(),
        "query status {}",
        response.status()
    );
    response
        .json::<Value>()
        .await
        .expect("decode log query response")["data"]
        .clone()
}

async fn wait_for_count(client: &reqwest::Client, base: &str, count: usize) -> Value {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let page = query_logs(client, base, None).await;
        if page["records"]
            .as_array()
            .is_some_and(|rows| rows.len() == count)
        {
            return page;
        }
        assert!(
            Instant::now() < deadline,
            "projection did not reach {count}: {page:#}"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cri_partial_rotation_trace_metadata_and_cloud_coexistence() {
    let _process_guard = PROCESS_TEST_LOCK.lock().await;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("pods");
    let log = pod_log(&root);
    let checkpoint = temp.path().join("cri.checkpoint.json");
    let quarantine = temp.path().join("cri.rejected.jsonl");
    let traced = serde_json::to_string(&service_event(
        "collection_create_or_extend",
        "partial traced event",
        Some(TRACE_ID),
        "b7ad6b7169203331",
    ))
    .unwrap();
    let split = traced.len() / 2;
    let stderr_event = serde_json::to_string(&service_event(
        "request_complete",
        "stderr event",
        None,
        "c7ad6b7169203332",
    ))
    .unwrap();
    std::fs::write(&log, cri_line("stdout", "P", &traced[..split])).unwrap();

    let port = reserve_port();
    let base = format!("http://127.0.0.1:{port}");
    let client = reqwest::Client::builder()
        .http1_only()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();
    let mut process = SiftProcess::spawn(port, &temp.path().join("sift-data"));
    wait_ready(&mut process, &client, &base).await;

    let incomplete = successful_summary(run_collector(&root, &checkpoint, &quarantine, &base));
    assert_eq!(incomplete["accepted"], 0);
    assert_eq!(incomplete["lines"], 0);
    let checkpoint_value: Value =
        serde_json::from_slice(&std::fs::read(&checkpoint).unwrap()).unwrap();
    assert_eq!(
        checkpoint_value["files"]
            .as_object()
            .unwrap()
            .values()
            .next()
            .unwrap()["offset"],
        0
    );

    let mut file = OpenOptions::new().append(true).open(&log).unwrap();
    write!(file, "{}", cri_line("stdout", "F", &traced[split..])).unwrap();
    write!(file, "{}", cri_line("stderr", "F", &stderr_event)).unwrap();
    write!(file, "{}", cri_line("stdout", "F", "not-json")).unwrap();
    file.sync_all().unwrap();

    let first = successful_summary(run_collector(&root, &checkpoint, &quarantine, &base));
    assert_eq!(first["accepted"], 2);
    assert_eq!(first["rejected"], 1);
    let traced_page = query_logs(&client, &base, Some(TRACE_ID)).await;
    let record = &traced_page["records"][0];
    assert_eq!(record["trace_id"], TRACE_ID);
    assert_eq!(record["resource"]["service.name"], "lumen");
    assert_eq!(record["resource"]["gcp.resource.type"], "k8s_container");
    assert_eq!(record["resource"]["gcp.project_id"], "project-a");
    assert_eq!(record["resource"]["k8s.cluster.name"], "cluster-a");
    assert_eq!(record["resource"]["k8s.namespace.name"], "prod");
    assert_eq!(record["resource"]["k8s.pod.name"], "lumen-0");
    assert_eq!(record["resource"]["k8s.pod.uid"], "1234-abcd");
    assert_eq!(record["resource"]["k8s.container.name"], "lumen");
    assert_eq!(record["resource"]["k8s.node.name"], "node-a");
    assert_eq!(record["attributes"]["collector.stream"]["type"], "string");
    assert_eq!(record["attributes"]["collector.stream"]["value"], "stdout");

    let rotated = log.with_file_name("0.log.rotated");
    std::fs::rename(&log, &rotated).unwrap();
    let old_event = serde_json::to_string(&service_event(
        "rotation_old_inode",
        "old inode drained",
        None,
        "d7ad6b7169203333",
    ))
    .unwrap();
    OpenOptions::new()
        .append(true)
        .open(&rotated)
        .unwrap()
        .write_all(cri_line("stdout", "F", &old_event).as_bytes())
        .unwrap();
    let new_event = serde_json::to_string(&service_event(
        "rotation_new_inode",
        "new inode started",
        None,
        "e7ad6b7169203334",
    ))
    .unwrap();
    std::fs::write(&log, cri_line("stdout", "F", &new_event)).unwrap();

    let rotation = successful_summary(run_collector(&root, &checkpoint, &quarantine, &base));
    assert_eq!(rotation["accepted"], 2);
    let all = wait_for_count(&client, &base, 4).await;
    let bodies = all["records"]
        .as_array()
        .unwrap()
        .iter()
        .map(|record| record["body_text"].as_str().unwrap())
        .collect::<Vec<_>>();
    let old_index = bodies
        .iter()
        .position(|body| *body == "old inode drained")
        .unwrap();
    let new_index = bodies
        .iter()
        .position(|body| *body == "new inode started")
        .unwrap();
    assert!(
        old_index < new_index,
        "known rotated inode must drain first: {bodies:?}"
    );

    let resumed = successful_summary(run_collector(&root, &checkpoint, &quarantine, &base));
    assert_eq!(resumed["accepted"], 0);
    assert_eq!(resumed["duplicates"], 0);
    assert_eq!(resumed["lines"], 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn endpoint_outage_retains_offset_and_recovery_drains_file() {
    let _process_guard = PROCESS_TEST_LOCK.lock().await;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("pods");
    let log = pod_log(&root);
    let checkpoint = temp.path().join("cri.checkpoint.json");
    let quarantine = temp.path().join("cri.rejected.jsonl");
    let event = serde_json::to_string(&service_event(
        "outage_recovery",
        "retained during outage",
        Some(TRACE_ID),
        "f7ad6b7169203335",
    ))
    .unwrap();
    std::fs::write(&log, cri_line("stdout", "F", &event)).unwrap();
    let port = reserve_port();
    let base = format!("http://127.0.0.1:{port}");

    let failed = run_collector(&root, &checkpoint, &quarantine, &base);
    assert!(!failed.status.success());
    let value: Value = serde_json::from_slice(&std::fs::read(&checkpoint).unwrap()).unwrap();
    assert_eq!(
        value["files"].as_object().unwrap().values().next().unwrap()["offset"],
        0
    );

    let client = reqwest::Client::builder().http1_only().build().unwrap();
    let mut process = SiftProcess::spawn(port, &temp.path().join("sift-data"));
    wait_ready(&mut process, &client, &base).await;
    let recovered = successful_summary(run_collector(&root, &checkpoint, &quarantine, &base));
    assert_eq!(recovered["accepted"], 1);
    assert_eq!(
        query_logs(&client, &base, Some(TRACE_ID)).await["records"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn missing_observed_unread_source_is_durably_accounted() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("pods");
    let log = pod_log(&root);
    let checkpoint = temp.path().join("cri.checkpoint.json");
    let quarantine = temp.path().join("cri.rejected.jsonl");
    let event = serde_json::to_string(&service_event(
        "will_be_lost",
        "loss must be visible",
        None,
        "07ad6b7169203336",
    ))
    .unwrap();
    std::fs::write(&log, cri_line("stdout", "F", &event)).unwrap();
    let unused = format!("http://127.0.0.1:{}", reserve_port());
    assert!(!run_collector(&root, &checkpoint, &quarantine, &unused)
        .status
        .success());
    let observed_len = std::fs::metadata(&log).unwrap().len();
    std::fs::remove_file(&log).unwrap();

    let summary = successful_summary(run_collector(&root, &checkpoint, &quarantine, &unused));
    assert_eq!(summary["accepted"], 0);
    assert_eq!(summary["rejected"], 1);
    assert_eq!(summary["lost_sources"], 1);
    assert_eq!(summary["lost_bytes"], observed_len);
    let rejection = std::fs::read_to_string(&quarantine).unwrap();
    assert!(rejection.contains("source_lost"));
    assert!(rejection.contains("observed uncommitted bytes"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn follow_collector_survives_outage_without_process_restart() {
    let _process_guard = PROCESS_TEST_LOCK.lock().await;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("pods");
    let log = pod_log(&root);
    let checkpoint = temp.path().join("checkpoint.json");
    let quarantine = temp.path().join("quarantine.jsonl");
    let event = serde_json::to_string(&service_event(
        "continuous_outage",
        "same process drains after outage",
        Some(TRACE_ID),
        "f7ad6b7169203335",
    ))
    .unwrap();
    std::fs::write(&log, cri_line("stdout", "F", &event)).unwrap();
    let port = reserve_port();
    let base = format!("http://127.0.0.1:{port}");
    let child = collector_command(&root, &checkpoint, &quarantine, &base)
        .arg("--follow")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut collector = SiftProcess {
        child,
        stderr: Arc::new(Mutex::new(Vec::new())),
        reader: None,
    };
    let original_pid = collector.child.id();
    let startup_deadline = Instant::now() + Duration::from_secs(10);
    while !checkpoint.exists() {
        assert!(
            collector.child.try_wait().unwrap().is_none(),
            "collector exited during the startup outage"
        );
        assert!(
            Instant::now() < startup_deadline,
            "collector did not open its source"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    tokio::time::sleep(Duration::from_millis(1200)).await;
    assert!(
        collector.child.try_wait().unwrap().is_none(),
        "follow collector must keep its pending batch through a retryable outage"
    );
    let value: Value = serde_json::from_slice(&std::fs::read(&checkpoint).unwrap()).unwrap();
    assert_eq!(
        value["files"].as_object().unwrap().values().next().unwrap()["offset"],
        0
    );

    let client = reqwest::Client::builder().http1_only().build().unwrap();
    let mut server = SiftProcess::spawn(port, &temp.path().join("sift-data"));
    wait_ready(&mut server, &client, &base).await;
    wait_for_count(&client, &base, 1).await;
    assert_eq!(collector.child.id(), original_pid);
    assert!(collector.child.try_wait().unwrap().is_none());
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let value: Value = serde_json::from_slice(&std::fs::read(&checkpoint).unwrap()).unwrap();
        if value["files"].as_object().unwrap().values().next().unwrap()["offset"]
            .as_u64()
            .unwrap()
            > 0
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "successful delivery must commit the checkpoint"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn follow_collector_stops_on_explicit_permanent_502() {
    let _process_guard = PROCESS_TEST_LOCK.lock().await;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("pods");
    let log = pod_log(&root);
    let checkpoint = temp.path().join("checkpoint.json");
    let quarantine = temp.path().join("quarantine.jsonl");
    let event = serde_json::to_string(&service_event(
        "permanent_failure",
        "pending",
        None,
        "f7ad6b7169203335",
    ))
    .unwrap();
    std::fs::write(&log, cri_line("stdout", "F", &event)).unwrap();
    let requests = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let seen = requests.clone();
    let app = axum::Router::new().route("/v1/logs", axum::routing::post(move || {
        let seen = seen.clone();
        async move {
            seen.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            (axum::http::StatusCode::BAD_GATEWAY, axum::Json(json!({
                "error": "invalid_upstream", "message": "permanent configuration error", "retryable": false
            })))
        }
    }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let child = collector_command(&root, &checkpoint, &quarantine, &base)
        .arg("--follow")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut collector = SiftProcess {
        child,
        stderr: Arc::new(Mutex::new(Vec::new())),
        reader: None,
    };
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        if let Some(status) = collector.child.try_wait().unwrap() {
            assert!(!status.success());
            break;
        }
        assert!(
            Instant::now() < deadline,
            "explicit permanent failure must not retry forever"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(requests.load(std::sync::atomic::Ordering::SeqCst), 1);
    let value: Value = serde_json::from_slice(&std::fs::read(&checkpoint).unwrap()).unwrap();
    assert_eq!(
        value["files"].as_object().unwrap().values().next().unwrap()["offset"],
        0
    );
    server.abort();
}

// HANDWRITE-END
