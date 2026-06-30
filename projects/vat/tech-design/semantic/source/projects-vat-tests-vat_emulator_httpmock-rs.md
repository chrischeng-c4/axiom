---
id: projects-vat-tests-vat_emulator_httpmock-rs
summary: >
  rust-source-unit TD AST payload for projects/vat/tests/vat_emulator_httpmock.rs.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This rust-source-unit TD preserves vat source ownership while migrating #39 off group-level source replay."
---

# Standardized projects/vat/tests/vat_emulator_httpmock.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/tests/vat_emulator_httpmock.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Self-contained integration test for the built-in HTTP mock proxy. Spawns
//! `vat emulator http-mock`, then via reqwest exercises: a stub over the proxy
//! (plain http), an HTTPS-MITM stubbed request (the headline — CONNECT + TLS +
//! intercept with no real upstream), and a record→replay round-trip over a local
//! plain-HTTP upstream that is then taken down.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use serde_json::json;

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

/// A one-shot HTTP upstream: serve one request with `body`, then stop listening.
fn spawn_oneshot_upstream(body: &'static str) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(resp.as_bytes());
        }
        // listener drops here → upstream is down for the replay leg.
    });
    port
}

#[tokio::test]
async fn http_mock_stub_mitm_and_record_replay() {
    let proxy_port = free_port();
    let proxy_addr = format!("127.0.0.1:{proxy_port}");
    let tmp = tempfile::tempdir().unwrap();
    let ca_path = tmp.path().join("ca.pem");
    let cassette_dir = tmp.path().join("cassettes");

    let child = Command::new(vat_bin())
        .args([
            "emulator",
            "http-mock",
            "--host-port",
            &proxy_addr,
            "--ca-path",
            ca_path.to_str().unwrap(),
            "--cassette-dir",
            cassette_dir.to_str().unwrap(),
        ])
        .spawn()
        .expect("spawn http-mock emulator");
    let _guard = Killed(child);
    wait_for_port(&proxy_addr);

    // The CA pem is written before the listener binds; read it for the MITM client.
    let ca_pem = std::fs::read(&ca_path).expect("CA pem written");

    let admin = reqwest::Client::new(); // direct, not proxied
    let proxy = reqwest::Proxy::all(format!("http://{proxy_addr}")).unwrap();

    // --- register two stubs via the admin API ---
    for path in ["/v1/x", "/v1/y"] {
        admin
            .post(format!("http://{proxy_addr}/__admin/stubs"))
            .json(&json!({
                "match": { "host": "api.test", "path": path },
                "response": { "status": 200, "body": format!("stub {path}") }
            }))
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap();
    }

    // --- (1) stub over the proxy, plain http (no network) ---
    let http_client = reqwest::Client::builder()
        .proxy(proxy.clone())
        .build()
        .unwrap();
    let r = http_client
        .get("http://api.test/v1/x")
        .send()
        .await
        .unwrap();
    assert_eq!(r.text().await.unwrap(), "stub /v1/x");

    // --- (2) HTTPS MITM + stub: client trusts our CA, proxies, hits https ---
    let mitm_client = reqwest::Client::builder()
        .proxy(proxy.clone())
        .add_root_certificate(reqwest::Certificate::from_pem(&ca_pem).unwrap())
        .build()
        .unwrap();
    let r = mitm_client
        .get("https://api.test/v1/y")
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    assert_eq!(r.text().await.unwrap(), "stub /v1/y");

    // --- (3) record then replay over a plain-http upstream that goes down ---
    let upstream = spawn_oneshot_upstream("recorded-body");
    let url = format!("http://127.0.0.1:{upstream}/data");
    // first call: no stub → forward + record.
    let first = http_client.get(&url).send().await.unwrap();
    assert_eq!(first.text().await.unwrap(), "recorded-body");
    // give the upstream thread a moment to exit (listener dropped).
    tokio::time::sleep(Duration::from_millis(200)).await;
    // second call: upstream is down → must replay from the cassette.
    let second = http_client.get(&url).send().await.unwrap();
    assert_eq!(second.status(), 200);
    assert_eq!(second.text().await.unwrap(), "recorded-body");
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/tests/vat_emulator_httpmock.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/vat/tests/vat_emulator_httpmock.rs` captured during #39 vat standardization.
```
