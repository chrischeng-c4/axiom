---
id: projects-vat-tests-vat_emulator_storage-rs
summary: >
  rust-source-unit TD AST payload for projects/vat/tests/vat_emulator_storage.rs.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This rust-source-unit TD preserves vat source ownership while migrating #39 off group-level source replay."
---

# Standardized projects/vat/tests/vat_emulator_storage.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/tests/vat_emulator_storage.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Self-contained integration test for the built-in Cloud Storage (GCS)
//! emulator. Spawns `vat emulator cloud-storage` and exercises the JSON API via
//! reqwest: media + multipart upload, download (alt=media, byte-identical),
//! metadata, list (prefix), and delete — including a slashed object name.

use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use serde_json::Value;

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

/// Minimal percent-encoding for an object-name path segment (encode '/').
fn enc(s: &str) -> String {
    s.replace('/', "%2F")
}

#[tokio::test]
async fn cloud_storage_emulator_roundtrips() {
    let port = free_port();
    let addr = format!("127.0.0.1:{port}");
    let child = Command::new(vat_bin())
        .args(["emulator", "cloud-storage", "--host-port", &addr])
        .spawn()
        .expect("spawn cloud-storage emulator");
    let _guard = Killed(child);
    wait_for_port(&addr);

    let base = format!("http://{addr}");
    let bucket = "demo-bucket";
    let client = reqwest::Client::new();

    // --- media upload (object name has a slash) ---
    let name = "dir/hello.txt";
    let body = b"hello gcs".to_vec();
    let up: Value = client
        .post(format!(
            "{base}/upload/storage/v1/b/{bucket}/o?uploadType=media&name={}",
            enc(name)
        ))
        .header("Content-Type", "text/plain")
        .body(body.clone())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(up["name"], name);
    assert_eq!(up["size"], body.len().to_string());
    assert!(up["md5Hash"].as_str().is_some());

    // --- download alt=media (byte-identical), object name percent-encoded ---
    let dl = client
        .get(format!(
            "{base}/storage/v1/b/{bucket}/o/{}?alt=media",
            enc(name)
        ))
        .send()
        .await
        .unwrap();
    assert!(dl.status().is_success());
    assert_eq!(dl.bytes().await.unwrap().as_ref(), body.as_slice());

    // --- metadata ---
    let meta: Value = client
        .get(format!("{base}/storage/v1/b/{bucket}/o/{}", enc(name)))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(meta["contentType"], "text/plain");
    assert_eq!(meta["size"], "9");

    // --- multipart upload ---
    let boundary = "vatboundary";
    let mp = format!(
        "--{b}\r\nContent-Type: application/json\r\n\r\n{{\"name\":\"dir/multi.json\"}}\r\n--{b}\r\nContent-Type: application/json\r\n\r\n{{\"k\":1}}\r\n--{b}--",
        b = boundary
    );
    client
        .post(format!(
            "{base}/upload/storage/v1/b/{bucket}/o?uploadType=multipart"
        ))
        .header(
            "Content-Type",
            format!("multipart/related; boundary={boundary}"),
        )
        .body(mp)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let multi = client
        .get(format!(
            "{base}/storage/v1/b/{bucket}/o/{}?alt=media",
            enc("dir/multi.json")
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(multi.text().await.unwrap(), "{\"k\":1}");

    // --- list with prefix ---
    let list: Value = client
        .get(format!("{base}/storage/v1/b/{bucket}/o?prefix=dir/"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let names: Vec<&str> = list["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|o| o["name"].as_str())
        .collect();
    assert!(
        names.contains(&"dir/hello.txt"),
        "list missing media object: {names:?}"
    );
    assert!(
        names.contains(&"dir/multi.json"),
        "list missing multipart object: {names:?}"
    );

    // --- delete then 404 ---
    client
        .delete(format!("{base}/storage/v1/b/{bucket}/o/{}", enc(name)))
        .send()
        .await
        .unwrap();
    let after = client
        .get(format!(
            "{base}/storage/v1/b/{bucket}/o/{}?alt=media",
            enc(name)
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(after.status().as_u16(), 404);
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/tests/vat_emulator_storage.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/vat/tests/vat_emulator_storage.rs` captured during #39 vat standardization.
```
