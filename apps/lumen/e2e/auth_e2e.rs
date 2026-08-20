// CODEGEN-BEGIN
//! The three request-authentication states of a serving process (#2869).
//!
//! Lumen holds no credentials; `LUMEN_AUTH=required` delegates authentication
//! to Kubernetes `TokenReview` and authorization to `SubjectAccessReview`. That
//! leaves three states, and the value of pinning all three is that the gaps
//! between them are where a silent fallback would live:
//!
//! - `auth: disabled` serves every route to an unauthenticated caller — and
//!   still refuses a *presented* credential, because nothing here could have
//!   checked it.
//! - `auth: required` with no review backend wired (reachable only by building
//!   the config by hand) refuses every data-plane request rather than
//!   degrading to the open mode.
//! - `LUMEN_AUTH=required` in a process that cannot delegate — no namespace to
//!   scope the review to, or no transport linked — must fail at startup
//!   without ever binding a serving port.
//!
//! The positive path (a real ServiceAccount identity allowed or denied by a
//! scripted apiserver) lives in `authz_matrix_e2e.rs`, which drives the same
//! router through a fake `ReviewBackend`.

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

/// A presented bearer is rejected even on an open server. Pinned because the
/// tempting shortcut — ignoring the header entirely — would make a stale client
/// look authenticated against a server that verified nothing.
#[tokio::test]
async fn a_presented_bearer_is_rejected_even_when_auth_is_disabled() {
    let s = server(AuthConfig::open());
    s.get("/collections")
        .add_header("authorization", "Bearer whatever")
        .await
        .assert_status_unauthorized();
}

/// The router half of fail-closed: a `required` config whose verifier was never
/// wired to a review backend rejects every data-plane request, with and without
/// a credential. There is no degradation to the open mode.
#[tokio::test]
async fn required_auth_without_a_review_backend_rejects_every_request() {
    let s = server(AuthConfig::required_in("serving"));
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
    for cfg in [AuthConfig::open(), AuthConfig::required_in("serving")] {
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

/// Run `lumen serve` with the given extra env and report `(exit ok, ever bound,
/// stderr)`. The port is polled for the whole startup budget rather than
/// sampled once after the exit: a bind that opened and closed inside the window
/// is still a bind, and a single post-hoc check would miss it.
fn serve_startup(env: &[(&str, &str)]) -> (bool, bool, String) {
    let port = free_port();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_lumen"));
    cmd.args([
        "serve",
        "--host",
        "127.0.0.1",
        "--port",
        &port.to_string(),
        "--wal",
        "embedded",
    ])
    .env_remove("RUST_LOG")
    .env_remove("LUMEN_AUTH_NAMESPACE")
    .env_remove("POD_NAMESPACE")
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());
    for (k, v) in env {
        cmd.env(k, v);
    }
    let mut child = cmd.spawn().expect("spawn lumen serve");

    let deadline = Instant::now() + Duration::from_secs(20);
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

    let mut stderr = String::new();
    if let Some(err) = child.stderr.take() {
        for line in BufReader::new(err).lines().map_while(Result::ok) {
            stderr.push_str(&line);
            stderr.push('\n');
        }
    }

    let status = match status {
        Some(s) => s,
        None => {
            let _ = child.kill();
            panic!("`LUMEN_AUTH=required` kept running instead of refusing to start:\n{stderr}");
        }
    };
    (status.success(), ever_bound, stderr)
}

/// AC3: the process half of fail-closed, with no namespace to scope the
/// review to. An unscoped `SubjectAccessReview` asks a different question than
/// the request did, so the process must refuse rather than pick a namespace.
#[test]
fn required_auth_without_a_namespace_refuses_to_start() {
    let (success, ever_bound, stderr) = serve_startup(&[("LUMEN_AUTH", "required")]);

    assert!(
        !success,
        "`LUMEN_AUTH=required` with no namespace exited successfully; stderr:\n{stderr}"
    );
    assert!(
        !ever_bound,
        "`LUMEN_AUTH=required` bound a serving port before giving up — for that window an \
         unauthenticated request would have been served"
    );
    for needle in [
        "LUMEN_AUTH=required",
        "LUMEN_AUTH_NAMESPACE",
        "SubjectAccessReview",
    ] {
        assert!(
            stderr.contains(needle),
            "startup refusal does not name `{needle}`; stderr:\n{stderr}"
        );
    }
}

/// AC3 continued: a namespace is not enough. The process still has to reach an
/// apiserver that will answer `TokenReview`/`SubjectAccessReview` for it, and
/// a build without the transport cannot even try. Neither case may bind.
///
/// The needles differ by build because the two failures are different facts —
/// a missing transport is a build mistake, an unreachable apiserver is a
/// deployment one — and an operator reading a single line has to be able to
/// tell which they have.
#[test]
fn required_auth_refuses_to_start_when_it_cannot_delegate() {
    let (success, ever_bound, stderr) = serve_startup(&[
        ("LUMEN_AUTH", "required"),
        ("LUMEN_AUTH_NAMESPACE", "serving"),
        // Keep the in-cluster client from finding a real cluster if the test
        // host happens to carry a kubeconfig.
        ("KUBERNETES_SERVICE_HOST", ""),
        ("KUBECONFIG", "/nonexistent/kubeconfig"),
    ]);

    assert!(
        !success,
        "`LUMEN_AUTH=required` exited successfully without a working delegation path; \
         stderr:\n{stderr}"
    );
    assert!(
        !ever_bound,
        "`LUMEN_AUTH=required` bound a serving port before giving up — for that window an \
         unauthenticated request would have been served"
    );
    assert!(
        stderr.contains("LUMEN_AUTH=required"),
        "startup refusal does not name the mode that caused it; stderr:\n{stderr}"
    );

    #[cfg(not(feature = "delegated-auth"))]
    for needle in ["delegated-auth", "Refusing to start"] {
        assert!(
            stderr.contains(needle),
            "a build without the transport must say so; `{needle}` missing from:\n{stderr}"
        );
    }
    #[cfg(feature = "delegated-auth")]
    assert!(
        stderr.contains("kube-apiserver") || stderr.contains("system:auth-delegator"),
        "a build with the transport must name the delegation failure; stderr:\n{stderr}"
    );
}
// CODEGEN-END
