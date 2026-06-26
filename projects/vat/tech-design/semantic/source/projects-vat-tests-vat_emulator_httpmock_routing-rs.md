---
id: projects-vat-tests-vat_emulator_httpmock_routing-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: local-agent-test-runner-protocol
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This rust-source-unit TD preserves vat e2e test source behavior for the local agent test runner protocol."
---

# Standardized projects/vat/tests/vat_emulator_httpmock_routing.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/tests/vat_emulator_httpmock_routing.rs`, captured as a rust-source-unit (td_ast) item-tree
during vat standardization onto the codegen ladder.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `vat_bin` | projects/vat/tests/vat_emulator_httpmock_routing.rs | function | private | 14 | fn vat_bin() -> &'static str |
| `free_port` | projects/vat/tests/vat_emulator_httpmock_routing.rs | function | private | 18 | fn free_port() -> u16 |
| `wait_for_port` | projects/vat/tests/vat_emulator_httpmock_routing.rs | function | private | 26 | fn wait_for_port(addr: &str) |
| `drop` | projects/vat/tests/vat_emulator_httpmock_routing.rs | function | private | 39 | fn drop(&mut self) |
| `spawn_sink` | projects/vat/tests/vat_emulator_httpmock_routing.rs | function | private | 47 | fn spawn_sink() -> (u16, mpsc::Receiver<String>) |
| `spawn_proxy` | projects/vat/tests/vat_emulator_httpmock_routing.rs | function | private | 71 | fn spawn_proxy(routes: &[(&str, u16)]) -> (String, Killed) |
| `http_mock_routes_known_host_to_local_sink` | projects/vat/tests/vat_emulator_httpmock_routing.rs | function | private | 101 | async fn http_mock_routes_known_host_to_local_sink() |
| `http_mock_admin_registers_route_at_runtime` | projects/vat/tests/vat_emulator_httpmock_routing.rs | function | private | 124 | async fn http_mock_admin_registers_route_at_runtime() |
| `http_mock_routes_https_via_mitm` | projects/vat/tests/vat_emulator_httpmock_routing.rs | function | private | 153 | async fn http_mock_routes_https_via_mitm() |

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Integration test for http-mock host-routing (network sandbox v1).
//! Spawns `vat emulator http-mock` with a route and asserts a proxied request to
//! the routed host is served by a local sink — over plain HTTP, via the
//! /__admin/routes runtime API, and over HTTPS (CONNECT MITM).
//!
//! @command cargo test -p vat --test vat_emulator_httpmock_routing -- --nocapture

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command};
use std::sync::mpsc;
use std::time::{Duration, Instant};

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

/// A local HTTP sink that accepts repeated connections; for each it reads the
/// request, replies `200 routed`, and sends the request text over the channel.
fn spawn_sink() -> (u16, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for conn in listener.incoming() {
            let Ok(mut stream) = conn else { continue };
            stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
            let mut buf = [0u8; 8192];
            let n = stream.read(&mut buf).unwrap_or(0);
            let req = String::from_utf8_lossy(&buf[..n]).to_string();
            let _ = stream.write_all(
                b"HTTP/1.1 200 OK\r\nContent-Length: 6\r\nContent-Type: text/plain\r\n\r\nrouted",
            );
            if tx.send(req).is_err() {
                break;
            }
        }
    });
    (port, rx)
}

/// Spawn the http-mock proxy with optional `--route host=base` seeds; returns the
/// proxy addr + a kill guard. CA/cassette paths go under the test's temp dir.
fn spawn_proxy(routes: &[(&str, u16)]) -> (String, Killed) {
    let port = free_port();
    let addr = format!("127.0.0.1:{port}");
    let tmp = std::env::temp_dir().join(format!("vat-httpmock-routing-{port}"));
    std::fs::create_dir_all(&tmp).unwrap();
    let ca_path = tmp.join("ca.pem");
    let cassette_dir = tmp.join("cassettes");
    let mut args: Vec<String> = vec![
        "emulator".into(),
        "http-mock".into(),
        "--host-port".into(),
        addr.clone(),
        "--ca-path".into(),
        ca_path.to_string_lossy().into_owned(),
        "--cassette-dir".into(),
        cassette_dir.to_string_lossy().into_owned(),
    ];
    for (host, sink) in routes {
        args.push("--route".into());
        args.push(format!("{host}=http://127.0.0.1:{sink}"));
    }
    let child = Command::new(vat_bin())
        .args(&args)
        .spawn()
        .expect("spawn http-mock proxy");
    wait_for_port(&addr);
    (addr, Killed(child))
}

#[tokio::test]
async fn http_mock_routes_known_host_to_local_sink() {
    let (sink, rx) = spawn_sink();
    let (proxy, _guard) = spawn_proxy(&[("example.test", sink)]);

    let client = reqwest::Client::builder()
        .proxy(reqwest::Proxy::all(format!("http://{proxy}")).unwrap())
        .build()
        .unwrap();
    let resp = client
        .get("http://example.test/work")
        .send()
        .await
        .expect("proxied request failed");
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.text().await.unwrap(), "routed");

    let got = rx
        .recv_timeout(Duration::from_secs(8))
        .expect("sink did not receive the routed request");
    assert!(got.contains("GET /work"), "wrong request line: {got}");
}

#[tokio::test]
async fn http_mock_admin_registers_route_at_runtime() {
    let (sink, rx) = spawn_sink();
    let (proxy, _guard) = spawn_proxy(&[]); // no seeded routes

    // Register a route at runtime via the admin API (direct origin-form request).
    let admin = reqwest::Client::new();
    admin
        .post(format!("http://{proxy}/__admin/routes"))
        .json(&serde_json::json!({ "host": "api.test", "target": format!("http://127.0.0.1:{sink}") }))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .expect("admin route registration failed");

    let client = reqwest::Client::builder()
        .proxy(reqwest::Proxy::all(format!("http://{proxy}")).unwrap())
        .build()
        .unwrap();
    let resp = client.get("http://api.test/x").send().await.unwrap();
    assert_eq!(resp.status(), 200);

    let got = rx
        .recv_timeout(Duration::from_secs(8))
        .expect("sink did not receive the runtime-registered route");
    assert!(got.contains("GET /x"), "wrong request line: {got}");
}

#[tokio::test]
async fn http_mock_routes_https_via_mitm() {
    let (sink, rx) = spawn_sink();
    let (proxy, _guard) = spawn_proxy(&[("secure.test", sink)]);

    // HTTPS through the proxy: reqwest CONNECTs, the proxy MITMs with a CA leaf
    // (accepted via danger_accept_invalid_certs), then routes to the http sink.
    let client = reqwest::Client::builder()
        .proxy(reqwest::Proxy::all(format!("http://{proxy}")).unwrap())
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    let resp = client
        .get("https://secure.test/s")
        .send()
        .await
        .expect("proxied https request failed");
    assert_eq!(resp.status(), 200);

    let got = rx
        .recv_timeout(Duration::from_secs(8))
        .expect("sink did not receive the MITM-routed https request");
    assert!(got.contains("GET /s"), "wrong request line: {got}");
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/tests/vat_emulator_httpmock_routing.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/vat/tests/vat_emulator_httpmock_routing.rs` captured during vat standardization.
```
