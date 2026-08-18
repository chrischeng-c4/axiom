// HANDWRITE-BEGIN gap="missing-generator:unit-test:bb6358e0" tracker="#2415" reason="Verify response JSONL schema, W3C handling, no-Sift boundary, and workload configuration from a real process."
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde_json::Value;

const TRACE_ID: &str = "0af7651916cd43dd8448eb211c80319c";
const PARENT_SPAN_ID: &str = "00f067aa0ba902b7";
const TRACEPARENT: &str = "00-0af7651916cd43dd8448eb211c80319c-00f067aa0ba902b7-01";

struct LoomProcess {
    child: Child,
    stdout: Arc<Mutex<Vec<String>>>,
    stderr: Arc<Mutex<Vec<String>>>,
    stdout_reader: Option<JoinHandle<()>>,
    stderr_reader: Option<JoinHandle<()>>,
}

impl LoomProcess {
    fn spawn(port: u16) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_loom"))
            .arg("controller")
            .env("LOOM_ADDR", format!("127.0.0.1:{port}"))
            .env("LOOM_LOG_FORMAT", "json")
            .env("LOOM_LOG_LEVEL", "info")
            .env_remove("RUST_LOG")
            .env_remove("LOOM_OTLP_ENDPOINT")
            .env_remove("OTEL_EXPORTER_OTLP_ENDPOINT")
            .env_remove("SIFT_URL")
            .env_remove("SIFT_ENDPOINT")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn real loom controller");
        let stdout = Arc::new(Mutex::new(Vec::new()));
        let stderr = Arc::new(Mutex::new(Vec::new()));
        let stdout_reader = drain_lines(
            child.stdout.take().expect("loom stdout pipe"),
            Arc::clone(&stdout),
        );
        let stderr_reader = drain_lines(
            child.stderr.take().expect("loom stderr pipe"),
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
            reader.join().expect("join loom stdout reader");
        }
        if let Some(reader) = self.stderr_reader.take() {
            reader.join().expect("join loom stderr reader");
        }
        self.stdout.lock().unwrap().clone()
    }
}

impl Drop for LoomProcess {
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

async fn wait_ready(process: &mut LoomProcess, client: &reqwest::Client, base: &str) {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if let Ok(response) = client.get(format!("{base}/healthz")).send().await {
            if response.status().is_success() {
                return;
            }
        }
        if let Some(status) = process.child.try_wait().expect("poll loom child") {
            panic!(
                "loom exited before readiness with {status}: {}",
                process.stderr_text()
            );
        }
        assert!(
            Instant::now() < deadline,
            "loom readiness timeout: {}",
            process.stderr_text()
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn get_with_traceparent(
    client: &reqwest::Client,
    base: &str,
    path: &str,
    traceparent: Option<&str>,
) {
    let mut request = client.get(format!("{base}{path}"));
    if let Some(traceparent) = traceparent {
        request = request.header("traceparent", traceparent);
    }
    let response = request.send().await.expect("request loom probe");
    assert!(
        response.status().is_success(),
        "request {path} failed with {}: {}",
        response.status(),
        response.text().await.unwrap_or_default()
    );
}

fn request_event<'a>(events: &'a [Value], path: &str) -> &'a Value {
    events
        .iter()
        .find(|event| event["attributes"]["uri"].as_str() == Some(path))
        .unwrap_or_else(|| panic!("missing request event for {path}: {events:#?}"))
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
async fn real_loom_controller_correlates_structured_stdout() {
    let port = reserve_port();
    let base = format!("http://127.0.0.1:{port}");
    let client = reqwest::Client::builder()
        .http1_only()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();
    let mut process = LoomProcess::spawn(port);
    wait_ready(&mut process, &client, &base).await;

    get_with_traceparent(&client, &base, "/readyz", Some(TRACEPARENT)).await;
    get_with_traceparent(
        &client,
        &base,
        "/metrics",
        Some("00-00000000000000000000000000000000-0000000000000000-01"),
    )
    .await;
    get_with_traceparent(&client, &base, "/openapi.json", None).await;

    tokio::time::sleep(Duration::from_millis(100)).await;
    let lines = process.finish();
    assert!(!lines.is_empty(), "loom emitted no stdout records");
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
        assert_eq!(event["service"]["name"], "loom", "{event:#}");
        assert_eq!(event["event"], "http_request_complete", "{event:#}");
    }

    let valid = request_event(&events, "/readyz");
    assert_eq!(required_str(valid, "trace_id"), TRACE_ID);
    assert_eq!(required_str(valid, "parent_span_id"), PARENT_SPAN_ID);
    assert_eq!(required_str(valid, "trace_flags"), "01");
    assert!(valid_nonzero_lower_hex(required_str(valid, "span_id"), 16));

    let invalid = request_event(&events, "/metrics");
    let invalid_trace = required_str(invalid, "trace_id");
    assert!(valid_nonzero_lower_hex(invalid_trace, 32));
    assert!(valid_nonzero_lower_hex(required_str(invalid, "span_id"), 16));
    assert!(invalid.get("parent_span_id").is_none());

    let missing = request_event(&events, "/openapi.json");
    let missing_trace = required_str(missing, "trace_id");
    assert!(valid_nonzero_lower_hex(missing_trace, 32));
    assert!(valid_nonzero_lower_hex(required_str(missing, "span_id"), 16));
    assert!(missing.get("parent_span_id").is_none());
    assert_ne!(invalid_trace, missing_trace);
}

#[test]
fn loom_remains_sift_agnostic() {
    assert!(
        !include_str!("../Cargo.toml").contains("sift =")
            && !include_str!("../src/main.rs").contains("SIFT_"),
        "Loom must not depend on or configure Sift directly"
    );
}

#[test]
fn loom_workloads_request_json_logging() {
    assert!(include_str!("../k8s/base/statefulset.yaml").contains("LOOM_LOG_FORMAT"));
    assert!(include_str!("../src/operator/render.rs").contains("LOOM_LOG_FORMAT"));
}
// HANDWRITE-END
