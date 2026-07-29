// SPEC-MANAGED: apps/lumen/tech-design/semantic/lumen-tests.md#unit-test
// CODEGEN-BEGIN
//! Phase-1 request-authentication contract (#2871).
//!
//! The bearer/identity registry this file used to exercise is gone, and the
//! Kubernetes TokenReview/SubjectAccessReview verifier that replaces it has
//! not landed. That leaves exactly two states worth pinning, and the value of
//! pinning them is that the gap between them is where a silent fallback would
//! live:
//!
//! - `auth: disabled` serves every route to an unauthenticated caller,
//!   unchanged from before the removal.
//! - `auth: required` has nothing to verify with, so it must refuse — at the
//!   router for a hand-built config, and at process start for the real binary,
//!   which must not reach the point of binding a serving port.

use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum_test::TestServer;
use serde_json::json;

use lumen::api::{router, AppState};
use lumen::auth::AuthConfig;
use lumen::storage::Engine;

fn server(cfg: AuthConfig) -> TestServer {
    let engine = Arc::new(Engine::new());
    let app = router(AppState::new(engine, Arc::new(cfg)));
    TestServer::new(app).expect("test server")
}

/// AC4: `auth: disabled` serves the whole data plane to a caller carrying no
/// credential — declare, index, search, list, drop. This is the mode every
/// deployment runs in today, so "unchanged" has to mean the full loop, not a
/// single probe.
#[tokio::test]
async fn disabled_auth_serves_the_whole_data_plane_unauthenticated() {
    let s = server(AuthConfig::open());

    s.put("/collections/u")
        .json(&json!({ "fields": { "e": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    s.post("/collections/u/index")
        .json(&json!({
            "items": [{ "external_id": "u1", "field": "e", "value": "a@x.com" }]
        }))
        .await
        .assert_status_ok();
    s.post("/collections/u/search")
        .json(&json!({
            "query": { "term": { "field": "e", "value": "a@x.com" } },
            "limit": 5
        }))
        .await
        .assert_status_ok();
    s.get("/collections/u/stats").await.assert_status_ok();

    let listed: serde_json::Value = s.get("/collections").await.json();
    assert_eq!(
        listed.as_array().map(|a| a.len()),
        Some(1),
        "an open server lists every collection: {listed}"
    );

    // Drop is asynchronous — 202, not 200.
    s.delete("/collections/u")
        .await
        .assert_status(axum::http::StatusCode::ACCEPTED);
}

/// A presented bearer resolves to nothing now, and "nothing" is not the same
/// as "absent": an unknown credential is still an unknown identity, so it is
/// rejected even on an open server. Pinned because the tempting shortcut —
/// ignoring the header entirely — would make a stale client look authenticated
/// against a server that cannot authenticate anyone.
#[tokio::test]
async fn a_presented_bearer_resolves_to_nothing_even_when_auth_is_disabled() {
    let s = server(AuthConfig::open());
    s.get("/collections")
        .add_header("authorization", "Bearer whatever")
        .await
        .assert_status_unauthorized();
}

/// The router half of fail-closed: a `required` config built by hand (the only
/// way to get one — `AuthConfig::from_env` refuses to return one) rejects every
/// data-plane request, with and without a credential.
#[tokio::test]
async fn required_auth_rejects_every_request_because_nothing_can_verify_one() {
    let s = server(AuthConfig { required: true });
    let schema = json!({ "fields": { "e": { "type": "keyword" } } });

    s.get("/collections").await.assert_status_unauthorized();
    s.put("/collections/u")
        .json(&schema)
        .await
        .assert_status_unauthorized();
    s.put("/collections/u")
        .add_header("authorization", "Bearer tok-admin")
        .json(&schema)
        .await
        .assert_status_unauthorized();
}

/// Probe/scrape routes stay exempt in both states — an operator has to be able
/// to see that a pod is unhealthy without holding a credential.
#[tokio::test]
async fn probe_and_scrape_routes_stay_exempt_in_both_states() {
    for cfg in [AuthConfig::open(), AuthConfig { required: true }] {
        let s = server(cfg);
        s.get("/metrics").await.assert_status_ok();
        s.get("/healthz").await.assert_status_ok();
        s.get("/readyz").await.assert_status_ok();
    }
}

/// A free localhost port, released before the caller uses it. Racy in
/// principle; in practice the window is a few milliseconds and the assertion
/// below (nothing ever accepts) does not depend on winning the race.
fn free_port() -> u16 {
    let l = std::net::TcpListener::bind("127.0.0.1:0").expect("bind ephemeral");
    let port = l.local_addr().expect("local addr").port();
    drop(l);
    port
}

/// AC3: the process half of fail-closed. `LUMEN_AUTH=required` must not
/// produce a running server. The oracle is deliberately two-sided — the exit
/// has to be non-zero *and* the port has to stay unbound for the whole
/// startup window — because a process that exits after binding, or binds and
/// then exits, is exactly the window an unauthenticated request could slip
/// through.
#[test]
fn required_auth_exits_at_startup_without_ever_binding_a_serving_port() {
    let port = free_port();
    let mut child = Command::new(env!("CARGO_BIN_EXE_lumen"))
        .args([
            "serve",
            "--host",
            "127.0.0.1",
            "--port",
            &port.to_string(),
            "--wal",
            "embedded",
        ])
        .env("LUMEN_AUTH", "required")
        .env_remove("RUST_LOG")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn lumen serve");

    // Poll the port for the whole startup budget rather than sampling once
    // after the exit: a bind that opened and closed inside the window is still
    // a bind, and a single post-hoc check would miss it.
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut ever_bound = false;
    let mut status = None;
    while Instant::now() < deadline {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            ever_bound = true;
        }
        match child.try_wait().expect("poll child") {
            Some(s) => {
                status = Some(s);
                break;
            }
            None => std::thread::sleep(Duration::from_millis(25)),
        }
    }

    let status = match status {
        Some(s) => s,
        None => {
            let _ = child.kill();
            panic!("`LUMEN_AUTH=required` kept running instead of refusing to start");
        }
    };

    let mut stderr = String::new();
    if let Some(err) = child.stderr.take() {
        for line in BufReader::new(err).lines().map_while(Result::ok) {
            stderr.push_str(&line);
            stderr.push('\n');
        }
    }

    assert!(
        !status.success(),
        "`LUMEN_AUTH=required` exited successfully; stderr:\n{stderr}"
    );
    assert!(
        !ever_bound,
        "`LUMEN_AUTH=required` bound :{port} before giving up — for that window an \
         unauthenticated request would have been served"
    );
    // The message is part of the contract: an operator reading only this line
    // has to learn that the mode is unimplemented, not that they mistyped it.
    for needle in ["LUMEN_AUTH=required", "TokenReview", "not implemented yet"] {
        assert!(
            stderr.contains(needle),
            "startup refusal does not name `{needle}`; stderr:\n{stderr}"
        );
    }
}
// CODEGEN-END
