// HANDWRITE-BEGIN gap="missing-generator:unit-test:0571ffd4" tracker="1871" reason="Run the real Lumen binary, make valid, invalid, and missing traceparent HTTP writes, capture stdout concurrently, and assert the shared JSONL and correlation contracts."
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde_json::{json, Value};

const TRACE_ID: &str = "0af7651916cd43dd8448eb211c80319c";
const PARENT_SPAN_ID: &str = "00f067aa0ba902b7";
const TRACEPARENT: &str = "00-0af7651916cd43dd8448eb211c80319c-00f067aa0ba902b7-01";

struct LumenProcess {
    child: Child,
    stdout: Arc<Mutex<Vec<String>>>,
    stderr: Arc<Mutex<Vec<String>>>,
    stdout_reader: Option<JoinHandle<()>>,
    stderr_reader: Option<JoinHandle<()>>,
}

impl LumenProcess {
    fn spawn(port: u16) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_lumen"))
            .args([
                "serve",
                "--host",
                "127.0.0.1",
                "--port",
                &port.to_string(),
                "--wal",
                "embedded",
                "--log-level",
                "info",
                "--log-format",
                "json",
            ])
            .env("LUMEN_AUTH", "off")
            .env_remove("RUST_LOG")
            .env_remove("LUMEN_LOG_FORMAT")
            .env_remove("LUMEN_OTLP_ENDPOINT")
            .env_remove("OTEL_EXPORTER_OTLP_ENDPOINT")
            .env_remove("SIFT_URL")
            .env_remove("SIFT_ENDPOINT")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn real lumen process");

        let stdout = Arc::new(Mutex::new(Vec::new()));
        let stderr = Arc::new(Mutex::new(Vec::new()));
        let stdout_reader = drain_lines(
            child.stdout.take().expect("lumen stdout pipe"),
            Arc::clone(&stdout),
        );
        let stderr_reader = drain_lines(
            child.stderr.take().expect("lumen stderr pipe"),
            Arc::clone(&stderr),
        );
        Self {
            child,
            stdout,
            stderr,
            stdout_reader: Some(stdout_reader),
            stderr_reader: Some(stderr_reader),
        }
    }

    fn stderr_text(&self) -> String {
        self.stderr.lock().unwrap().join("\n")
    }

    fn finish(&mut self) -> Vec<String> {
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(reader) = self.stdout_reader.take() {
            reader.join().expect("join lumen stdout reader");
        }
        if let Some(reader) = self.stderr_reader.take() {
            reader.join().expect("join lumen stderr reader");
        }
        self.stdout.lock().unwrap().clone()
    }
}

impl Drop for LumenProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(reader) = self.stdout_reader.take() {
            let _ = reader.join();
        }
        if let Some(reader) = self.stderr_reader.take() {
            let _ = reader.join();
        }
    }
}

fn drain_lines(
    stream: impl std::io::Read + Send + 'static,
    destination: Arc<Mutex<Vec<String>>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        for line in BufReader::new(stream).lines().map_while(Result::ok) {
            destination.lock().unwrap().push(line);
        }
    })
}

fn reserve_port() -> u16 {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("reserve loopback port");
    listener.local_addr().unwrap().port()
}

async fn wait_ready(process: &mut LumenProcess, client: &reqwest::Client, base: &str) {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if let Ok(response) = client.get(format!("{base}/healthz")).send().await {
            if response.status().is_success() {
                return;
            }
        }
        if let Some(status) = process.child.try_wait().expect("poll lumen child") {
            panic!(
                "lumen exited before readiness with {status}: {}",
                process.stderr_text()
            );
        }
        assert!(
            Instant::now() < deadline,
            "lumen readiness timeout: {}",
            process.stderr_text()
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn create_collection(
    client: &reqwest::Client,
    base: &str,
    collection_id: &str,
    traceparent: Option<&str>,
) {
    let mut request = client
        .put(format!("{base}/collections/{collection_id}"))
        .json(&json!({ "fields": { "title": { "type": "text" } } }));
    if let Some(traceparent) = traceparent {
        request = request.header("traceparent", traceparent);
    }
    let response = request.send().await.expect("create collection request");
    assert!(
        response.status().is_success(),
        "collection {collection_id} failed with {}: {}",
        response.status(),
        response.text().await.unwrap_or_default()
    );
}

fn audit_event<'a>(events: &'a [Value], collection_id: &str) -> &'a Value {
    events
        .iter()
        .find(|event| {
            event["event"] == "collection_create_or_extend"
                && event["attributes"]["collection_id"] == collection_id
        })
        .unwrap_or_else(|| panic!("missing audit event for {collection_id}: {events:#?}"))
}

fn required_str<'a>(event: &'a Value, field: &str) -> &'a str {
    event
        .get(field)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("missing string field {field}: {event:#}"))
}

fn valid_nonzero_lower_hex(value: &str, len: usize) -> bool {
    value.len() == len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        && value.bytes().any(|byte| byte != b'0')
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn real_lumen_process_correlates_structured_stdout() {
    let port = reserve_port();
    let base = format!("http://127.0.0.1:{port}");
    let client = reqwest::Client::builder()
        .http1_only()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();
    let mut process = LumenProcess::spawn(port);
    wait_ready(&mut process, &client, &base).await;

    create_collection(&client, &base, "trace-valid", Some(TRACEPARENT)).await;
    create_collection(
        &client,
        &base,
        "trace-invalid",
        Some("00-00000000000000000000000000000000-0000000000000000-01"),
    )
    .await;
    create_collection(&client, &base, "trace-missing", None).await;

    tokio::time::sleep(Duration::from_millis(100)).await;
    let lines = process.finish();
    assert!(!lines.is_empty(), "lumen emitted no stdout records");

    let events = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            serde_json::from_str::<Value>(line)
                .unwrap_or_else(|error| panic!("non-JSON stdout line ({error}): {line}"))
        })
        .collect::<Vec<_>>();
    assert_eq!(events.len(), lines.len());
    for event in &events {
        assert_eq!(event["schema"], "axiom.service.log.v1", "{event:#}");
        assert_eq!(event["service"]["name"], "lumen", "{event:#}");
    }

    let valid = audit_event(&events, "trace-valid");
    assert_eq!(required_str(valid, "trace_id"), TRACE_ID);
    assert_eq!(required_str(valid, "parent_span_id"), PARENT_SPAN_ID);
    assert_eq!(required_str(valid, "trace_flags"), "01");
    let valid_span = required_str(valid, "span_id");
    assert!(valid_nonzero_lower_hex(valid_span, 16));
    assert_ne!(valid_span, PARENT_SPAN_ID);

    let invalid = audit_event(&events, "trace-invalid");
    let invalid_trace = required_str(invalid, "trace_id");
    let invalid_span = required_str(invalid, "span_id");
    assert!(valid_nonzero_lower_hex(invalid_trace, 32));
    assert!(valid_nonzero_lower_hex(invalid_span, 16));
    assert!(invalid.get("parent_span_id").is_none());

    let missing = audit_event(&events, "trace-missing");
    let missing_trace = required_str(missing, "trace_id");
    let missing_span = required_str(missing, "span_id");
    assert!(valid_nonzero_lower_hex(missing_trace, 32));
    assert!(valid_nonzero_lower_hex(missing_span, 16));
    assert!(missing.get("parent_span_id").is_none());
    assert_ne!(invalid_trace, missing_trace);

    assert!(
        !include_str!("../Cargo.toml").contains("sift ="),
        "Lumen must not depend on Sift"
    );
}
// HANDWRITE-END
