//! End-to-end pipeline proof: measure → compare → gate → report → exit, driven
//! against two in-test stub HTTP servers (fast base, slow peer). No real
//! lumen/pg/OS needed — this is the CI guard for arena's whole contract.
//!
//! Load is kept gentle (20 qps, well-spaced) so connection churn does not
//! produce spurious failures; the point is the comparison pipeline, not a load
//! stress test.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use arena::engine::{run, RunOpts};
use arena::spec::Spec;

/// Spawn a stub HTTP server that answers every request `200 {}` after `delay`.
/// Mirrors rig's own loadgen-test stub (read once, respond, drop) so it does
/// not drop connections under the offered load. Returns its `/s` URL.
fn spawn_stub(delay: Duration) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut s) = stream else { break };
            thread::spawn(move || {
                let _ = s.set_nodelay(true);
                let mut buf = [0u8; 4096];
                let _ = s.read(&mut buf); // drain the (tiny) request headers + body
                if !delay.is_zero() {
                    thread::sleep(delay);
                }
                let body = r#"{"ok":true}"#;
                let resp = format!(
                    "HTTP/1.1 200 OK\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = s.write_all(resp.as_bytes());
                let _ = s.flush();
                // Clean close: FIN after the response, then drain any leftover
                // request bytes so the OS does not send an RST (which the client
                // would see as a failed request and poison achieved_qps).
                let _ = s.shutdown(std::net::Shutdown::Write);
                let _ = s.read(&mut buf);
            });
        }
    });
    format!("http://127.0.0.1:{port}/s")
}

/// Build a one-cell spec: base vs peer, with the peer's gate set by `gate`.
fn spec_for(base_url: &str, peer_url: &str, gate: &str) -> Spec {
    let toml = format!(
        r#"
spec_version = 1
name = "test-cmp"
base = "base"
metric = "p99_ms"
ratchet = 0.8

[targets.base]
kind = "service"
[targets.base.load]
target_qps = 20
workers = 4
duration_secs = 2
warmup_secs = 0

[targets.peer]
kind = "service"
[targets.peer.load]
target_qps = 20
workers = 4
duration_secs = 2
warmup_secs = 0

[[cells]]
name = "c1"
[cells.targets.base]
request = {{ method = "POST", url = "{base_url}", body = "{{}}" }}
[cells.targets.peer]
gate = "{gate}"
request = {{ method = "POST", url = "{peer_url}", body = "{{}}" }}
"#
    );
    Spec::parse(&toml).unwrap()
}

#[test]
fn win_cell_with_no_baseline_is_clean_and_records_a_ratio_over_one() {
    let base = spawn_stub(Duration::from_millis(2));
    let peer = spawn_stub(Duration::from_millis(24));
    let spec = spec_for(&base, &peer, "win");

    let tmp = tempfile::tempdir().unwrap();
    let opts = RunOpts {
        baseline_path: Some(tmp.path().join("baselines.json")),
        ..Default::default()
    };
    let report = run(&spec, &opts);

    // No baseline yet -> PinMissingBaseline is Info -> clean / exit 0.
    assert_eq!(
        report.exit_code(),
        0,
        "no-baseline WIN run must be clean; findings: {:?}",
        report.base.summary.sample
    );
    assert!(report.base.clean);
    // The peer is the slow server, so peer/base latency ratio > 1 (base wins).
    let row = &report.comparison[0];
    assert_eq!(row.cell, "c1");
    let peer_cell = &row.peers[0];
    assert!(peer_cell.trustworthy, "load should be honest at 20 qps");
    assert!(
        peer_cell.ratio > 1.0,
        "slow peer => ratio {} should exceed 1",
        peer_cell.ratio
    );
    assert!(peer_cell.verdict.starts_with("WIN"));
}

#[test]
fn win_breach_against_an_unbeatable_baseline_is_exit_2() {
    let base = spawn_stub(Duration::from_millis(2));
    let peer = spawn_stub(Duration::from_millis(24));
    let spec = spec_for(&base, &peer, "win");

    // Pre-seed an unreachable baseline ratio (50x) so req = 0.8*50 = 40 — the
    // measured ratio (~12x) cannot clear it: a WIN regression.
    let tmp = tempfile::tempdir().unwrap();
    let bpath = tmp.path().join("baselines.json");
    let mut store = rig::pins::BaselineStore::load_at(&bpath);
    store.record("test-cmp/c1/peer", "ratio", 50.0);
    store.save().unwrap();

    let opts = RunOpts {
        baseline_path: Some(bpath),
        ..Default::default()
    };
    let report = run(&spec, &opts);

    assert_eq!(
        report.exit_code(),
        2,
        "WIN breach must be exit 2 (regression)"
    );
    assert!(report
        .base
        .findings
        .iter()
        .any(|f| f.kind == rig::report::Kind::PinRegression));
}

#[test]
fn exempt_cell_never_gates() {
    let base = spawn_stub(Duration::from_millis(2));
    let peer = spawn_stub(Duration::from_millis(24));
    let spec = spec_for(&base, &peer, "exempt");

    let tmp = tempfile::tempdir().unwrap();
    let opts = RunOpts {
        baseline_path: Some(tmp.path().join("baselines.json")),
        ..Default::default()
    };
    let report = run(&spec, &opts);

    assert_eq!(report.exit_code(), 0);
    assert_eq!(report.comparison[0].peers[0].verdict, "exempt");
    assert!(
        report.base.findings.is_empty(),
        "exempt cells emit no findings"
    );
}

#[test]
fn update_baselines_records_the_measured_ratio() {
    let base = spawn_stub(Duration::from_millis(2));
    let peer = spawn_stub(Duration::from_millis(24));
    let spec = spec_for(&base, &peer, "win");

    let tmp = tempfile::tempdir().unwrap();
    let bpath = tmp.path().join("baselines.json");
    let opts = RunOpts {
        update_baselines: true,
        baseline_path: Some(bpath.clone()),
        ..Default::default()
    };
    let report = run(&spec, &opts);
    assert_eq!(report.exit_code(), 0, "update run does not gate");

    let store = rig::pins::BaselineStore::load_at(&bpath);
    let entry = store.get("test-cmp/c1/peer", "ratio");
    assert!(entry.is_some(), "the measured ratio should be recorded");
    assert!(entry.unwrap().value > 1.0);
}

/// Cross-transport comparison: an HTTP target (stub) vs a real Postgres target,
/// both driven by rig's ONE scheduler. Skips gracefully if no local pg.
#[test]
fn http_vs_postgres_compares_across_transports() {
    let dsn = std::env::var("ARENA_TEST_PG_DSN").unwrap_or_else(|_| {
        let user = std::env::var("USER").unwrap_or_else(|_| "postgres".to_string());
        format!("host=/tmp user={user} dbname=postgres")
    });
    // Skip pattern (CLAUDE.md): no local pg => return, don't fail.
    let Ok(mut client) = postgres::Client::connect(&dsn, postgres::NoTls) else {
        eprintln!("skipping http_vs_postgres: no postgres reachable at `{dsn}`");
        return;
    };
    let _ = client.batch_execute("SELECT 1");
    drop(client);

    let web = spawn_stub(Duration::from_millis(3));
    let toml = format!(
        r#"
spec_version = 1
name = "http-vs-pg"
base = "web"
metric = "p99_ms"

[targets.web]
kind = "http"
[targets.web.load]
target_qps = 20
workers = 4
duration_secs = 2

[targets.db]
kind = "postgres"
dsn = "{dsn}"
[targets.db.load]
target_qps = 20
workers = 4
duration_secs = 2

[[cells]]
name = "ping"
[cells.targets.web]
request = {{ method = "POST", url = "{web}", body = "{{}}" }}
[cells.targets.db]
gate = "exempt"
query = "SELECT 1"
"#
    );
    let spec = Spec::parse(&toml).unwrap();

    let tmp = tempfile::tempdir().unwrap();
    let opts = RunOpts {
        baseline_path: Some(tmp.path().join("baselines.json")),
        ..Default::default()
    };
    let report = run(&spec, &opts);

    assert_eq!(
        report.exit_code(),
        0,
        "prompt: {}",
        report.base.agent_prompt
    );
    let row = &report.comparison[0];
    let db = row
        .peers
        .iter()
        .find(|p| p.target == "db")
        .expect("db peer present");
    assert!(db.trustworthy, "pg load should be honest at 20 qps");
    assert!(db.value > 0.0, "pg p99 should be measured (> 0)");
    assert!(db.ratio.is_finite(), "ratio computed across transports");
    assert_eq!(db.verdict, "exempt");
}
