---
id: projects-vat-tests-vat_emulator_workflows-rs
summary: >
  rust-source-unit TD AST payload for projects/vat/tests/vat_emulator_workflows.rs.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This rust-source-unit TD preserves vat source ownership while migrating #39 off group-level source replay."
---

# Standardized projects/vat/tests/vat_emulator_workflows.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/tests/vat_emulator_workflows.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Self-contained integration test for the built-in Cloud Workflows emulator.
//! Spawns `vat emulator cloud-workflows`, creates a workflow whose `main`
//! assigns, `call: http.post`s to a local sink, and returns, then runs an
//! execution and asserts both the dispatched call and the SUCCEEDED result.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

fn vat_bin() -> &'static str {
    env!("CARGO_BIN_EXE_vat")
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn wait_for_port(addr: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if TcpStream::connect(addr).is_ok() {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("emulator did not open {addr}");
}

struct Killed(Child);
impl Drop for Killed {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn spawn_sink() -> (u16, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
            let mut buf = [0u8; 8192];
            let n = stream.read(&mut buf).unwrap_or(0);
            let req = String::from_utf8_lossy(&buf[..n]).to_string();
            let _ = stream.write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}",
            );
            let _ = tx.send(req);
        }
    });
    (port, rx)
}

#[tokio::test]
async fn cloud_workflows_emulator_runs_and_dispatches() {
    let port = free_port();
    let addr = format!("127.0.0.1:{port}");
    let child = Command::new(vat_bin())
        .args(["emulator", "cloud-workflows", "--host-port", &addr])
        .spawn()
        .expect("spawn cloud-workflows emulator");
    let _guard = Killed(child);
    wait_for_port(&addr);

    let base = format!("http://{addr}/v1");
    let parent = "projects/demo-vat/locations/us-central1";
    let client = reqwest::Client::new();

    let (sink_port, rx) = spawn_sink();
    // main: assign n, POST a message to the sink, then return a computed value.
    let source = format!(
        r#"
main:
  params: [args]
  steps:
    - setup:
        assign:
          - n: ${{args.n}}
    - send:
        call: http.post
        args:
          url: "http://127.0.0.1:{sink_port}/hook"
          body:
            message: ${{"n=" + n}}
        result: resp
    - finish:
        return: ${{n * 2}}
"#
    );

    client
        .post(format!("{base}/{parent}/workflows/wf"))
        .json(&json!({ "sourceContents": source }))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    // createExecution with a JSON-string argument (GCP shape).
    let exec: Value = client
        .post(format!("{base}/{parent}/workflows/wf/executions"))
        .json(&json!({ "argument": "{\"n\": 21}" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(exec["state"], "SUCCEEDED", "execution failed: {exec}");
    // result is a JSON-encoded string per the Workflows API.
    assert_eq!(exec["result"], json!("42"));

    let got = rx
        .recv_timeout(Duration::from_secs(8))
        .expect("sink did not receive the workflow's http call");
    assert!(got.contains("POST /hook"), "wrong request line: {got}");
    assert!(got.contains("n=21"), "missing interpolated body: {got}");
}

#[tokio::test]
async fn cloud_workflows_try_except_recovers() {
    let port = free_port();
    let addr = format!("127.0.0.1:{port}");
    let child = Command::new(vat_bin())
        .args(["emulator", "cloud-workflows", "--host-port", &addr])
        .spawn()
        .expect("spawn cloud-workflows emulator");
    let _guard = Killed(child);
    wait_for_port(&addr);

    let base = format!("http://{addr}/v1");
    let parent = "projects/demo-vat/locations/us-central1";
    let client = reqwest::Client::new();

    // try calls a dead port; except returns a fallback (proves no panic).
    let source = r#"
main:
  steps:
    - attempt:
        try:
          steps:
            - dead:
                call: http.get
                args:
                  url: "http://127.0.0.1:1/nope"
                result: r
        except:
          as: e
          steps:
            - recover:
                return: "fallback"
"#;
    client
        .post(format!("{base}/{parent}/workflows/wf2"))
        .json(&json!({ "sourceContents": source }))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let exec: Value = client
        .post(format!("{base}/{parent}/workflows/wf2/executions"))
        .json(&json!({}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(exec["state"], "SUCCEEDED", "execution failed: {exec}");
    assert_eq!(exec["result"], json!("\"fallback\""));
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/tests/vat_emulator_workflows.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/vat/tests/vat_emulator_workflows.rs` captured during #39 vat standardization.
```
