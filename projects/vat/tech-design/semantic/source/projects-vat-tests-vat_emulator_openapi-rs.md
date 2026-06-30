---
id: projects-vat-tests-vat_emulator_openapi-rs
summary: >
  rust-source-unit TD AST payload for projects/vat/tests/vat_emulator_openapi.rs.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This rust-source-unit TD preserves vat source ownership while migrating #39 off group-level source replay."
---

# Standardized projects/vat/tests/vat_emulator_openapi.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/tests/vat_emulator_openapi.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Self-contained integration test for the OpenAPI-driven mock. Spawns
//! `vat emulator openapi --spec <tmp spec>` and asserts a documented operation
//! returns the spec's example (404 otherwise); then spawns `vat emulator
//! http-mock`, registers the same spec for a host via `/__admin/openapi`, and
//! asserts a proxied HTTPS-MITM GET to that host is answered from the spec — the
//! headline: a contract becomes a working fake with no stub and no real upstream.

use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command};
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

const SPEC: &str = r##"
openapi: 3.0.0
info: { title: Pets, version: "1.0" }
paths:
  /pets/{id}:
    get:
      responses:
        "200":
          content:
            application/json:
              example: { id: 7, name: "Rex" }
  /widgets:
    get:
      responses:
        "200":
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/Widget"
components:
  schemas:
    Widget:
      type: object
      properties:
        id: { type: integer }
        label: { type: string }
"##;

#[tokio::test]
async fn openapi_standalone_and_http_mock_source() {
    let tmp = tempfile::tempdir().unwrap();
    let spec_path = tmp.path().join("api.yaml");
    std::fs::write(&spec_path, SPEC).unwrap();

    // --- standalone openapi preset server ---
    let api_port = free_port();
    let api_addr = format!("127.0.0.1:{api_port}");
    let api = Command::new(vat_bin())
        .args([
            "emulator",
            "openapi",
            "--host-port",
            &api_addr,
            "--spec",
            spec_path.to_str().unwrap(),
        ])
        .spawn()
        .expect("spawn openapi emulator");
    let _api_guard = Killed(api);
    wait_for_port(&api_addr);

    let client = reqwest::Client::new();
    // documented operation with path templating → the example.
    let r = client
        .get(format!("http://{api_addr}/pets/123"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let v: Value = r.json().await.unwrap();
    assert_eq!(v["id"], 7);
    assert_eq!(v["name"], "Rex");
    // schema-synthesized body (no example) via $ref.
    let w: Value = client
        .get(format!("http://{api_addr}/widgets"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(w["label"], "string");
    // undocumented path → 404.
    let miss = client
        .get(format!("http://{api_addr}/nope"))
        .send()
        .await
        .unwrap();
    assert_eq!(miss.status(), 404);

    // --- the headline: http-mock proxy answers from the spec, transparently ---
    let proxy_port = free_port();
    let proxy_addr = format!("127.0.0.1:{proxy_port}");
    let ca_path = tmp.path().join("ca.pem");
    let cassette_dir = tmp.path().join("cassettes");
    let mock = Command::new(vat_bin())
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
    let _mock_guard = Killed(mock);
    wait_for_port(&proxy_addr);
    let ca_pem = std::fs::read(&ca_path).expect("CA pem written");

    // register the spec for host api.test (no stub, no upstream).
    reqwest::Client::new()
        .post(format!("http://{proxy_addr}/__admin/openapi"))
        .json(&json!({ "host": "api.test", "spec": SPEC }))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    // a proxied + CA-trusting client GETs https://api.test/pets/1 → spec example.
    let mitm = reqwest::Client::builder()
        .proxy(reqwest::Proxy::all(format!("http://{proxy_addr}")).unwrap())
        .add_root_certificate(reqwest::Certificate::from_pem(&ca_pem).unwrap())
        .build()
        .unwrap();
    let resp = mitm.get("https://api.test/pets/1").send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["id"], 7);
    assert_eq!(body["name"], "Rex");
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/tests/vat_emulator_openapi.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/vat/tests/vat_emulator_openapi.rs` captured during #39 vat standardization.
```
