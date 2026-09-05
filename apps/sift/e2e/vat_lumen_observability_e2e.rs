// HANDWRITE-BEGIN gap="missing-generator:unit-test:7f82566a" tracker="1902" reason="Prove the real VAT-managed Lumen stdout to Sift collector and query journey."
#![cfg(unix)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

const TRACE_ID: &str = "0af7651916cd43dd8448eb211c80319c";
const PARENT_SPAN_ID: &str = "00f067aa0ba902b7";
const TRACEPARENT: &str = "00-0af7651916cd43dd8448eb211c80319c-00f067aa0ba902b7-01";

fn debug_binary(name: &str) -> PathBuf {
    Path::new(env!("CARGO_BIN_EXE_sift"))
        .parent()
        .expect("Sift debug binary directory")
        .join(name)
}

fn require_binary(path: &Path) {
    assert!(
        path.is_file(),
        "missing current-workspace binary {}; run `cargo build -p vat -p lumen -p sift --bins`",
        path.display()
    );
}

fn jsonl(stdout: &[u8]) -> Vec<Value> {
    String::from_utf8_lossy(stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("VAT stdout JSONL"))
        .collect()
}

#[test]
fn architecture_runbook_names_owned_boundaries_and_repro_command() {
    let runbook = include_str!("../observability/structured-stdout.md");
    for required in [
        "axiom.service.log.v1",
        "W3C `traceparent`",
        "VAT_SERVICE_LUMEN_STDOUT_LOG",
        "Sift-owned collector plane",
        "CRI/GKE source adapter",
        "cargo build -p vat -p lumen -p sift --bins",
        "vat_managed_lumen_stdout_reaches_real_sift_query",
        "cargo test -p sift --test collector_cri",
    ] {
        assert!(runbook.contains(required), "runbook missing {required:?}");
    }
}

#[test]
fn vat_managed_lumen_stdout_reaches_real_sift_query() {
    if std::env::var_os("VAT_OBSERVABILITY_PROBE").is_some() {
        return;
    }

    let vat = debug_binary("vat");
    let lumen = debug_binary("lumen");
    let sift = debug_binary("sift");
    require_binary(&vat);
    require_binary(&lumen);
    require_binary(&sift);
    let probe = std::env::current_exe().expect("current E2E test executable");

    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    let config = format!(
        r#"
version = 1
name = "lumen-sift-observability"

[workspace]
base = "."
workdir = "."
keep = "always"

[env]
LUMEN_AUTH = "off"
SIFT_AUTH = "off"
VAT_OBSERVABILITY_PROBE = "1"

[[services]]
id = "lumen"
cmd = [{lumen}, "serve", "--host", "127.0.0.1", "--port", "{{port}}", "--wal", "embedded", "--log-level", "info", "--log-format", "json"]
ready_http = "http://127.0.0.1:{{port}}/readyz"
export = {{ LUMEN_URL = "http://{{host}}:{{port}}" }}
timeout_s = 20

[[services]]
id = "sift"
cmd = [{sift}, "serve", "--host", "127.0.0.1", "--port", "{{port}}", "--data-dir", "sift-data", "--log-level", "warn", "--log-format", "json"]
ready_http = "http://127.0.0.1:{{port}}/readyz"
export = {{ SIFT_URL = "http://{{host}}:{{port}}" }}
timeout_s = 20

[[runners]]
id = "observability"
requires = ["lumen", "sift"]
cmd = [{probe}, "--exact", "vat_runner_observability_probe", "--nocapture"]
timeout_s = 30
artifacts = ["observability-proof.json"]
"#,
        lumen = serde_json::to_string(&lumen.to_string_lossy()).unwrap(),
        sift = serde_json::to_string(&sift.to_string_lossy()).unwrap(),
        probe = serde_json::to_string(&probe.to_string_lossy()).unwrap(),
    );
    std::fs::write(project.path().join("vat.toml"), config).unwrap();

    let output = Command::new(&vat)
        .current_dir(project.path())
        .env("VAT_HOME", vat_home.path())
        .env_remove("RUST_LOG")
        .env_remove("LUMEN_OTLP_ENDPOINT")
        .env_remove("OTEL_EXPORTER_OTLP_ENDPOINT")
        .env_remove("SIFT_TOKEN")
        .args(["run", "observability"])
        .output()
        .expect("run real VAT observability journey");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let events = jsonl(&output.stdout);
    let result = events
        .iter()
        .find(|event| event["type"] == "result")
        .expect("VAT result event");
    let vat_id = result["id"].as_str().expect("VAT id");
    if !output.status.success() {
        let logs = Command::new(&vat)
            .env("VAT_HOME", vat_home.path())
            .args(["logs", vat_id, "runner"])
            .output()
            .expect("read failed VAT runner logs");
        panic!(
            "VAT journey failed: stdout=\n{}\nstderr=\n{}\nrunner stdout=\n{}\nrunner stderr=\n{}",
            stdout,
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&logs.stdout),
            String::from_utf8_lossy(&logs.stderr)
        );
    }
    assert!(
        !stdout.contains(TRACE_ID),
        "VAT must not replay captured Lumen log bytes into its stdout: {stdout}"
    );
    assert_eq!(result["ok"], true);
    assert_eq!(result["state"], "kept");

    let state_output = Command::new(&vat)
        .env("VAT_HOME", vat_home.path())
        .args(["state", vat_id, "--compact"])
        .output()
        .expect("read retained VAT state");
    assert!(state_output.status.success());
    let state: Value = serde_json::from_slice(&state_output.stdout).unwrap();
    assert!(
        state.to_string().contains("observability-proof.json"),
        "retained VAT state lacks proof artifact: {state:#}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vat_runner_observability_probe() {
    if std::env::var_os("VAT_OBSERVABILITY_PROBE").is_none() {
        return;
    }

    let lumen_url = std::env::var("LUMEN_URL").expect("VAT Lumen endpoint export");
    let sift_url = std::env::var("SIFT_URL").expect("VAT Sift endpoint export");
    let logs_dir = PathBuf::from(std::env::var("VAT_LOGS_DIR").expect("VAT logs directory"));
    let lumen_stdout = PathBuf::from(
        std::env::var("VAT_SERVICE_LUMEN_STDOUT_LOG").expect("VAT Lumen stdout path export"),
    );
    assert!(lumen_stdout.starts_with(&logs_dir));
    assert!(lumen_stdout.is_file());

    let client = reqwest::Client::builder()
        .http1_only()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();
    let response = client
        .put(format!("{lumen_url}/collections/vat-trace"))
        .header("traceparent", TRACEPARENT)
        .json(&json!({ "fields": { "title": { "type": "text" } } }))
        .send()
        .await
        .expect("send traced Lumen request");
    assert!(
        response.status().is_success(),
        "Lumen request failed with {}: {}",
        response.status(),
        response.text().await.unwrap_or_default()
    );

    let captured = wait_for_captured_audit(&lumen_stdout).await;
    assert_eq!(captured["schema"], "axiom.service.log.v1");
    assert_eq!(captured["service"]["name"], "lumen");
    assert_eq!(captured["trace_id"], TRACE_ID);
    assert_eq!(captured["parent_span_id"], PARENT_SPAN_ID);
    assert_eq!(captured["trace_flags"], "01");
    let local_span = captured["span_id"].as_str().expect("local span id");
    assert_eq!(local_span.len(), 16);
    assert_ne!(local_span, PARENT_SPAN_ID);

    let collector = Command::new(env!("CARGO_BIN_EXE_sift"))
        .args([
            "collect",
            "--source",
            lumen_stdout.to_str().unwrap(),
            "--source-id",
            "vat:lumen:stdout",
            "--endpoint",
            &sift_url,
            "--project",
            "local",
            "--environment",
            "test",
            "--checkpoint",
            "collector.checkpoint.json",
            "--quarantine",
            "collector.rejected.jsonl",
            "--max-retries",
            "2",
        ])
        .env_remove("SIFT_TOKEN")
        .output()
        .expect("run real Sift collector");
    assert!(
        collector.status.success(),
        "collector failed: stdout={} stderr={}",
        String::from_utf8_lossy(&collector.stdout),
        String::from_utf8_lossy(&collector.stderr)
    );
    let summary: Value = serde_json::from_slice(&collector.stdout).unwrap();
    assert!(
        summary["accepted"].as_u64().unwrap_or(0) >= 1,
        "{summary:#}"
    );
    assert_eq!(summary["rejected"], 0);

    let record = wait_for_sift_record(&client, &sift_url).await;
    assert_eq!(record["resource"]["service.name"], "lumen");
    assert_eq!(record["trace_id"], TRACE_ID);
    assert_eq!(record["span_id"], local_span);
    assert_eq!(record["attributes"]["parent_span_id"]["type"], "string");
    assert_eq!(
        record["attributes"]["parent_span_id"]["value"],
        PARENT_SPAN_ID
    );
    assert_eq!(record["attributes"]["trace.flags"]["type"], "string");
    assert_eq!(record["attributes"]["trace.flags"]["value"], "01");
    assert_eq!(
        record["attributes"]["event.name"]["value"],
        "collection_create_or_extend"
    );
    assert_eq!(record["attributes"]["collection_id"]["value"], "vat-trace");
    assert_eq!(
        record["body_text"],
        record["json_payload"]["body"]["stringValue"]
    );
    assert!(record["body_text"]
        .as_str()
        .is_some_and(|value| !value.is_empty()));

    let proof = json!({
        "schema": "sift.local-observability-proof.v1",
        "source": lumen_stdout,
        "trace_id": TRACE_ID,
        "parent_span_id": PARENT_SPAN_ID,
        "span_id": local_span,
        "service": record["resource"]["service.name"],
        "event": record["attributes"]["event.name"]["value"],
        "message": record["body_text"],
        "collector": summary,
    });
    std::fs::write(
        "observability-proof.json",
        serde_json::to_vec_pretty(&proof).unwrap(),
    )
    .unwrap();
}

async fn wait_for_captured_audit(path: &Path) -> Value {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let source = std::fs::read_to_string(path).unwrap_or_default();
        if let Some(event) = source
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .find(|event| {
                event["event"] == "collection_create_or_extend"
                    && event["attributes"]["collection_id"] == "vat-trace"
                    && event["trace_id"] == TRACE_ID
            })
        {
            return event;
        }
        assert!(
            Instant::now() < deadline,
            "missing correlated Lumen audit in {}: {source}",
            path.display()
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn wait_for_sift_record(client: &reqwest::Client, sift_url: &str) -> Value {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let response = client
            .post(format!("{sift_url}/api/v1/query"))
            .json(&json!({
                "version": 1,
                "project": "local",
                "environment": "test",
                "signal": {
                    "kind": "logs",
                    "filter": {"op": "eq", "field": "trace_id", "value": TRACE_ID}
                },
                "limit": 20
            }))
            .send()
            .await
            .expect("query real Sift logging API");
        assert!(
            response.status().is_success(),
            "Sift query status {}",
            response.status()
        );
        let page: Value = response.json().await.unwrap();
        if let Some(record) = page["data"]["records"].as_array().and_then(|records| {
            records.iter().find(|record| {
                record["attributes"]["event.name"]["value"] == "collection_create_or_extend"
                    && record["attributes"]["collection_id"]["value"] == "vat-trace"
            })
        }) {
            return record.clone();
        }
        assert!(
            Instant::now() < deadline,
            "missing correlated Sift record: {page:#}"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
// HANDWRITE-END
