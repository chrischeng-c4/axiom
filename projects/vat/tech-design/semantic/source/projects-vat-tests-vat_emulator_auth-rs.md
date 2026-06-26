---
id: projects-vat-tests-vat_emulator_auth-rs
summary: >
  rust-source-unit TD AST payload for projects/vat/tests/vat_emulator_auth.rs.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This rust-source-unit TD preserves vat source ownership while migrating #39 off group-level source replay."
---

# Standardized projects/vat/tests/vat_emulator_auth.rs

## Overview
<!-- type: overview lang: markdown -->

Rust source-unit TD for `projects/vat/tests/vat_emulator_auth.rs`, captured during #39 vat migration onto td_ast lossless source generation.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Self-contained integration test for the built-in Firebase Auth emulator.
//! Spawns `vat emulator firebase-auth`, drives signUp -> signInWithPassword ->
//! lookup over raw HTTP (no external client/tooling), and asserts the JWT
//! idToken + the created user round-trip.

use std::io::{Read, Write};
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

fn post(addr: &str, path: &str, body: &str) -> Value {
    let mut stream = TcpStream::connect(addr).unwrap();
    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(req.as_bytes()).unwrap();
    let mut resp = String::new();
    stream.read_to_string(&mut resp).unwrap();
    let start = resp.find('{').expect("json body");
    let end = resp.rfind('}').expect("json body");
    serde_json::from_str(&resp[start..=end]).unwrap()
}

struct Killed(Child);
impl Drop for Killed {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn firebase_auth_emulator_signup_signin_lookup() {
    let port = free_port();
    let addr = format!("127.0.0.1:{port}");
    let child = Command::new(vat_bin())
        .args(["emulator", "firebase-auth", "--host-port", &addr])
        .spawn()
        .expect("spawn firebase-auth emulator");
    let _guard = Killed(child);
    wait_for_port(&addr);

    let signup = post(
        &addr,
        "/identitytoolkit.googleapis.com/v1/accounts:signUp",
        r#"{"email":"a@b.com","password":"pw","returnSecureToken":true}"#,
    );
    let id_token = signup["idToken"].as_str().expect("idToken").to_string();
    let local_id = signup["localId"].as_str().expect("localId").to_string();
    assert!(id_token.split('.').count() == 3, "idToken is a JWT");

    let signin = post(
        &addr,
        "/identitytoolkit.googleapis.com/v1/accounts:signInWithPassword",
        r#"{"email":"a@b.com","password":"pw","returnSecureToken":true}"#,
    );
    assert_eq!(signin["localId"].as_str(), Some(local_id.as_str()));

    let bad = post(
        &addr,
        "/identitytoolkit.googleapis.com/v1/accounts:signInWithPassword",
        r#"{"email":"a@b.com","password":"wrong"}"#,
    );
    assert!(bad.get("error").is_some(), "wrong password is rejected");

    let lookup = post(
        &addr,
        "/identitytoolkit.googleapis.com/v1/accounts:lookup",
        &format!(r#"{{"idToken":"{id_token}"}}"#),
    );
    assert_eq!(
        lookup["users"][0]["localId"].as_str(),
        Some(local_id.as_str()),
        "lookup resolves the signed-up user"
    );
    assert_eq!(lookup["users"][0]["email"].as_str(), Some("a@b.com"));
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/tests/vat_emulator_auth.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/vat/tests/vat_emulator_auth.rs` captured during #39 vat standardization.
```
