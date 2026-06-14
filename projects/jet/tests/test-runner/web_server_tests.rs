// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Integration tests for `[test.web_server]` supervisor (P3.1).
//!
//! Spec: `.aw/tech-design/projects/jet/logic/web-server.md`.
//!
//! No Chromium required. Tests use `python3 -m http.server` or
//! in-process TCP listeners.

use jet::task_runner::config::{JetConfig, WebServerConfig};
use jet::test_runner::web_server;
use std::time::{Duration, Instant};
use tokio::net::TcpListener;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn python_available() -> bool {
    which::which("python3").is_ok()
}

/// Bind a tokio TCP listener on 127.0.0.1:0 to obtain a free port, drop the
/// listener, return the port. There is a small race window where another
/// process could bind the same port before the test uses it — acceptable for
/// a unit test.
async fn free_port() -> u16 {
    let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
    l.local_addr().unwrap().port()
}

// ── W_parse_full / W_parse_minimal ───────────────────────────────────────────

#[test]
fn w_parse_full_toml() {
    let toml = r#"
[test.web_server]
command = "npm run dev"
port = 3000
url = "http://localhost:3000/ready"
timeout_ms = 120000
reuse_existing = false
cwd = "frontend"
"#;
    let cfg: JetConfig = toml::from_str(toml).unwrap();
    let ws = cfg.test.web_server.unwrap();
    assert_eq!(ws.command, "npm run dev");
    assert_eq!(ws.port, Some(3000));
    assert_eq!(ws.url.as_deref(), Some("http://localhost:3000/ready"));
    assert_eq!(ws.timeout_ms, 120000);
    assert!(!ws.reuse_existing);
    assert_eq!(ws.cwd.as_deref(), Some("frontend"));
}

#[test]
fn w_parse_minimal_toml_uses_defaults() {
    let toml = r#"
[test.web_server]
command = "npm run dev"
"#;
    let cfg: JetConfig = toml::from_str(toml).unwrap();
    let ws = cfg.test.web_server.unwrap();
    assert_eq!(ws.command, "npm run dev");
    assert_eq!(ws.port, None);
    assert_eq!(ws.url, None);
    assert_eq!(ws.timeout_ms, 60000);
    assert!(ws.reuse_existing);
    assert_eq!(ws.cwd, None);
}

// ── W_boot_with_tcp_probe ────────────────────────────────────────────────────

#[tokio::test]
async fn w_boot_with_tcp_probe() {
    if !python_available() {
        eprintln!("skipping: python3 not on PATH");
        return;
    }
    let port = free_port().await;
    let tmp = tempfile::tempdir().unwrap();
    let cfg = WebServerConfig {
        command: format!("python3 -m http.server {port}"),
        port: Some(port),
        url: None,
        timeout_ms: 10_000,
        reuse_existing: false,
        cwd: None,
    };
    let started = Instant::now();
    let handle = web_server::boot(&cfg, tmp.path())
        .await
        .expect("boot should succeed");
    assert!(started.elapsed() < Duration::from_secs(10));
    // Probe again — the server should still be up.
    let live = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .is_ok();
    assert!(live);
    drop(handle);
    // After drop, the server should die within a reasonable window.
    let mut dead = false;
    for _ in 0..30 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        if tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .is_err()
        {
            dead = true;
            break;
        }
    }
    assert!(dead, "child should have died within 3s of handle drop");
}

// ── W_boot_with_http_probe ───────────────────────────────────────────────────

#[tokio::test]
async fn w_boot_with_http_probe() {
    if !python_available() {
        eprintln!("skipping: python3 not on PATH");
        return;
    }
    let port = free_port().await;
    let tmp = tempfile::tempdir().unwrap();
    let cfg = WebServerConfig {
        command: format!("python3 -m http.server {port}"),
        port: None,
        url: Some(format!("http://127.0.0.1:{port}/")),
        timeout_ms: 10_000,
        reuse_existing: false,
        cwd: None,
    };
    let handle = web_server::boot(&cfg, tmp.path()).await.expect("boot ok");
    drop(handle);
}

// ── W_boot_fails_fast_on_bad_command ─────────────────────────────────────────

#[tokio::test]
async fn w_boot_fails_fast_on_bad_command() {
    let tmp = tempfile::tempdir().unwrap();
    let port = free_port().await;
    let cfg = WebServerConfig {
        // `false` always exits with status 1 immediately.
        command: "false".to_string(),
        port: Some(port),
        url: None,
        timeout_ms: 10_000,
        reuse_existing: false,
        cwd: None,
    };
    let started = Instant::now();
    let err = web_server::boot(&cfg, tmp.path()).await.unwrap_err();
    let elapsed = started.elapsed();
    assert!(
        elapsed < Duration::from_secs(3),
        "should fail-fast, elapsed={elapsed:?}"
    );
    let msg = format!("{err}");
    assert!(
        msg.contains("exited before becoming ready"),
        "unexpected error: {msg}"
    );
}

// ── W_boot_times_out ─────────────────────────────────────────────────────────

#[tokio::test]
async fn w_boot_times_out() {
    let tmp = tempfile::tempdir().unwrap();
    let port = free_port().await;
    let cfg = WebServerConfig {
        // Sleep forever but never open the port.
        command: "sleep 10".to_string(),
        port: Some(port),
        url: None,
        timeout_ms: 500,
        reuse_existing: false,
        cwd: None,
    };
    let started = Instant::now();
    let err = web_server::boot(&cfg, tmp.path()).await.unwrap_err();
    let elapsed = started.elapsed();
    assert!(elapsed >= Duration::from_millis(500));
    assert!(
        elapsed < Duration::from_secs(3),
        "timeout path overshot: {elapsed:?}"
    );
    let msg = format!("{err}");
    assert!(
        msg.contains("did not become ready"),
        "unexpected error: {msg}"
    );
}

// ── W_reuse_existing ─────────────────────────────────────────────────────────

#[tokio::test]
async fn w_reuse_existing_skips_spawn() {
    let port = free_port().await;
    // Start a persistent listener that accepts but never closes. Hold it in
    // a detached task so we control its lifetime.
    let listener = TcpListener::bind(("127.0.0.1", port)).await.unwrap();
    let accept_task = tokio::spawn(async move {
        loop {
            let _ = listener.accept().await;
        }
    });
    let tmp = tempfile::tempdir().unwrap();
    let cfg = WebServerConfig {
        // This command would fail instantly if boot tried to spawn.
        command: "false".to_string(),
        port: Some(port),
        url: None,
        timeout_ms: 5_000,
        reuse_existing: true,
        cwd: None,
    };
    let handle = web_server::boot(&cfg, tmp.path())
        .await
        .expect("reuse_existing should short-circuit and not spawn");
    drop(handle);
    accept_task.abort();
    // Port may still be listening — that's fine; the test's contract is
    // only that `boot` did not spawn and did not error.
}
// CODEGEN-END
