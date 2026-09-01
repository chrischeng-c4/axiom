// HANDWRITE-BEGIN gap="missing-generator:unit-test:27bf4b0e" tracker="1873" reason="Start real Sift, collect a Lumen JSONL file containing an invalid line and valid correlated events, query logs, and prove checkpoint replay idempotency."
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde_json::{json, Value};

const TRACE_ID: &str = "0af7651916cd43dd8448eb211c80319c";

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
        "service": {
            "name": "lumen",
            "version": "0.4.21"
        },
        "event": event,
        "message": message,
        "span_id": span_id,
        "trace_flags": "01",
        "attributes": {
            "collection_id": "docs",
            "component": "lumen.audit"
        }
    });
    if let Some(trace_id) = trace_id {
        value["trace_id"] = Value::String(trace_id.to_string());
        value["parent_span_id"] = Value::String("00f067aa0ba902b7".to_string());
    }
    value
}

fn run_collector(source: &Path, checkpoint: &Path, quarantine: &Path, endpoint: &str) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_sift"))
        .args([
            "collect",
            "--source",
            source.to_str().unwrap(),
            "--source-id",
            "vat:lumen:stdout",
            "--endpoint",
            endpoint,
            "--project",
            "local",
            "--environment",
            "test",
            "--checkpoint",
            checkpoint.to_str().unwrap(),
            "--quarantine",
            quarantine.to_str().unwrap(),
            "--batch-size",
            "2",
            "--max-retries",
            "2",
        ])
        .env_remove("SIFT_TOKEN")
        .output()
        .expect("run real Sift collector");
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
        "project": "local",
        "environment": "test",
        "signal": {"kind": "logs"},
        "limit": 20,
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

async fn wait_for_records(
    client: &reqwest::Client,
    base: &str,
    trace_id: Option<&str>,
    count: usize,
) -> Value {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let page = query_logs(client, base, trace_id).await;
        if page["records"]
            .as_array()
            .is_some_and(|records| records.len() == count)
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
async fn real_file_collector_ingests_queries_and_resumes() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("lumen.stdout.jsonl");
    let checkpoint = temp.path().join("collector.checkpoint.json");
    let quarantine = temp.path().join("collector.rejected.jsonl");
    let lines = [
        "{not-json".to_string(),
        serde_json::to_string(&service_event(
            "collection_create_or_extend",
            "collection created",
            Some(TRACE_ID),
            "b7ad6b7169203331",
        ))
        .unwrap(),
        serde_json::to_string(&service_event(
            "request_complete",
            "request completed",
            None,
            "c7ad6b7169203332",
        ))
        .unwrap(),
    ];
    std::fs::write(&source, format!("{}\n", lines.join("\n"))).unwrap();

    let port = reserve_port();
    let base = format!("http://127.0.0.1:{port}");
    let client = reqwest::Client::builder()
        .http1_only()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();
    let mut process = SiftProcess::spawn(port, &temp.path().join("sift-data"));
    wait_ready(&mut process, &client, &base).await;

    let first = run_collector(&source, &checkpoint, &quarantine, &base);
    assert_eq!(first["accepted"], 2);
    assert_eq!(first["duplicates"], 0);
    assert_eq!(first["rejected"], 1);
    assert_eq!(first["lines"], 3);
    assert_eq!(
        first["final_offset"],
        std::fs::metadata(&source).unwrap().len()
    );
    assert_eq!(
        std::fs::read_to_string(&quarantine)
            .unwrap()
            .lines()
            .count(),
        1
    );

    let traced = wait_for_records(&client, &base, Some(TRACE_ID), 1).await;
    let record = &traced["records"][0];
    assert_eq!(record["resource"]["service.name"], "lumen");
    assert_eq!(record["trace_id"], TRACE_ID);
    assert_eq!(record["span_id"], "b7ad6b7169203331");
    assert_eq!(
        record["attributes"]["event.name"]["value"],
        "collection_create_or_extend"
    );
    assert_eq!(record["body_text"], "collection created");

    let resumed = run_collector(&source, &checkpoint, &quarantine, &base);
    assert_eq!(resumed["lines"], 0);
    assert_eq!(resumed["accepted"], 0);
    assert_eq!(resumed["duplicates"], 0);
    assert_eq!(resumed["rejected"], 0);

    std::fs::remove_file(&checkpoint).unwrap();
    let replayed = run_collector(&source, &checkpoint, &quarantine, &base);
    assert_eq!(replayed["accepted"], 2);
    assert_eq!(replayed["duplicates"], 0);
    assert_eq!(replayed["rejected"], 1);

    let all = wait_for_records(&client, &base, None, 2).await;
    assert_eq!(all["records"].as_array().unwrap().len(), 2);
}
// HANDWRITE-END
