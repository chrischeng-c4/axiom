// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-oracle-tests.md#tests
// CODEGEN-BEGIN

//! Integration tests for `jet-parity-oracle`.
//!
//! Two layers:
//! * **Stub-backed tests** (always-on): drive the runner with a
//!   `StubBrowserSession` so we can exercise the full §Logic state
//!   machine — load, launch, navigate, await_mount, 5 channels, bundle.
//!   These cover T1, T2, T3, T4, T5, T6, T7, T8, T10 against deterministic
//!   canned data.
//! * **Live-Chromium tests** (`#[ignore]`-d): real Playwright drive.
//!   Gated on the #2139 follow-up that lands the live browser harness.

use jet_parity_oracle::{
    run_fixture, ArtifactBundle, BrowserKind, BrowserSession, MatrixEntry, Runner, RunnerConfig,
    RunnerError, StubBrowserSession,
};
use serde_json::json;
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/mui-button.tsx")
}

fn make_config(root: &std::path::Path) -> RunnerConfig {
    RunnerConfig {
        artifact_root: root.to_path_buf(),
        shell_html: PathBuf::from("fixtures/__shell__/index.html"),
        per_fixture_budget: Duration::from_secs(8),
        viewport: (800, 600),
    }
}

async fn run_with_stub(
    tmp: &std::path::Path,
    session: StubBrowserSession,
) -> Result<ArtifactBundle, RunnerError> {
    let cfg = make_config(tmp);
    let matrix = MatrixEntry {
        browser: BrowserKind::Chromium,
        dpr: 1.0,
    };
    let mut runner = Runner::with_session(cfg, matrix, Box::new(session));
    runner.run(&fixture_path()).await
}

/// T1 — runner emits all 5 artifacts for a non-IME fixture.
#[tokio::test]
async fn test_runner_emits_five_artifacts() {
    let tmp = tempfile::tempdir().unwrap();
    let bundle = run_with_stub(tmp.path(), StubBrowserSession::new())
        .await
        .unwrap();
    assert!(bundle.pixel_png.exists(), "pixel.png missing");
    assert!(bundle.a11y_json.exists(), "a11y-tree.json missing");
    assert!(bundle.focus_json.exists(), "focus-trace.json missing");
    assert!(bundle.pointer_json.exists(), "pointer-hitmap.json missing");
    assert!(bundle.ime_json.exists(), "ime-trace.json missing");
    // Each artifact non-empty:
    for p in [
        &bundle.pixel_png,
        &bundle.a11y_json,
        &bundle.focus_json,
        &bundle.pointer_json,
        &bundle.ime_json,
    ] {
        let md = std::fs::metadata(p).unwrap();
        assert!(md.len() > 0, "{} is empty", p.display());
    }
    // sha256 captured for each:
    assert_eq!(bundle.sha256s.len(), 5);
}

/// T2 — non-IME fixture writes `{events: []}` (never missing).
#[tokio::test]
async fn test_non_ime_fixture_writes_empty_ime_json() {
    let tmp = tempfile::tempdir().unwrap();
    let bundle = run_with_stub(tmp.path(), StubBrowserSession::new())
        .await
        .unwrap();
    let bytes = std::fs::read(&bundle.ime_json).unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v, json!({"events": []}));
}

/// T3 — IME fixture captures composition events.
#[tokio::test]
async fn test_ime_fixture_captures_composition() {
    // Patch the mui-button fixture into an "ime: true" manifest on the fly
    // by writing a temporary fixture file.
    let tmp = tempfile::tempdir().unwrap();
    let fx = tmp.path().join("ime-fixture.tsx");
    std::fs::write(
        &fx,
        r#"/** @fixture { "name": "ime-fixture", "ime": true, "tab_count": 4 } */
import * as React from "react";
export default function F() { return null; }
"#,
    )
    .unwrap();
    let mut stub = StubBrowserSession::new();
    stub.ime_events = vec![
        json!({"type": "compositionstart", "data": ""}),
        json!({"type": "compositionupdate", "data": "ni"}),
        json!({"type": "compositionend", "data": "你"}),
        json!({"type": "input", "data": "你"}),
    ];
    let cfg = make_config(tmp.path());
    let matrix = MatrixEntry {
        browser: BrowserKind::Chromium,
        dpr: 1.0,
    };
    let mut runner = Runner::with_session(cfg, matrix, Box::new(stub));
    let bundle = runner.run(&fx).await.unwrap();
    let v: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&bundle.ime_json).unwrap()).unwrap();
    let events = v.get("events").unwrap().as_array().unwrap();
    assert_eq!(events.len(), 4);
    assert_eq!(events[0]["type"], "compositionstart");
}

/// T4 — pixel artifact path matches `<fixture>-<browser>-<dpr>.png`.
#[tokio::test]
async fn test_pixel_artifact_naming() {
    let tmp = tempfile::tempdir().unwrap();
    let bundle = run_with_stub(tmp.path(), StubBrowserSession::new())
        .await
        .unwrap();
    assert_eq!(
        bundle.pixel_png.file_name().unwrap().to_string_lossy(),
        "mui-button-chromium-1.0.png"
    );
}

/// T5 — a11y artifact is verbatim getFullAXTree.
#[tokio::test]
async fn test_a11y_artifact_is_verbatim_axtree() {
    let mut stub = StubBrowserSession::new();
    stub.ax_tree = json!({"nodes": [{"nodeId": "1", "role": "button", "extra": [1, 2, 3]}]});
    let tmp = tempfile::tempdir().unwrap();
    let bundle = run_with_stub(tmp.path(), stub).await.unwrap();
    let v: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&bundle.a11y_json).unwrap()).unwrap();
    // Verbatim — no mutation, no filtering.
    assert_eq!(
        v,
        json!({"nodes": [{"nodeId": "1", "role": "button", "extra": [1, 2, 3]}]})
    );
}

/// T6 — focus trace has exactly tab_count entries.
#[tokio::test]
async fn test_focus_trace_length_and_shape() {
    let tmp = tempfile::tempdir().unwrap();
    let bundle = run_with_stub(tmp.path(), StubBrowserSession::new())
        .await
        .unwrap();
    let v: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&bundle.focus_json).unwrap()).unwrap();
    let arr = v.as_array().unwrap();
    // mui-button fixture declares tab_count: 8.
    assert_eq!(arr.len(), 8);
    for entry in arr {
        assert!(entry.get("step").is_some());
        assert!(entry.get("selector").is_some());
        assert!(entry.get("role").is_some());
        assert!(entry.get("name").is_some());
        assert!(entry.get("bounds").is_some());
    }
}

/// T7 — pointer hitmap has 1000 entries, PRNG seeded by `fnv1a64(fixture_name)`.
#[tokio::test]
async fn test_pointer_hitmap_seeded_1000() {
    let tmp = tempfile::tempdir().unwrap();
    let bundle = run_with_stub(tmp.path(), StubBrowserSession::new())
        .await
        .unwrap();
    let v: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&bundle.pointer_json).unwrap()).unwrap();
    let arr = v.as_array().unwrap();
    assert_eq!(arr.len(), 1000);
    // Determinism: run again, expect identical sha256.
    let tmp2 = tempfile::tempdir().unwrap();
    let bundle2 = run_with_stub(tmp2.path(), StubBrowserSession::new())
        .await
        .unwrap();
    assert_eq!(
        bundle.sha256s.get("pointer-hitmap.json"),
        bundle2.sha256s.get("pointer-hitmap.json"),
        "pointer hitmap not byte-equivalent across runs"
    );
}

/// T8 — running same fixture twice yields byte-equivalent artifact sha256s.
#[tokio::test]
async fn test_byte_equivalent_replay() {
    let tmp1 = tempfile::tempdir().unwrap();
    let tmp2 = tempfile::tempdir().unwrap();
    let b1 = run_with_stub(tmp1.path(), StubBrowserSession::new())
        .await
        .unwrap();
    let b2 = run_with_stub(tmp2.path(), StubBrowserSession::new())
        .await
        .unwrap();
    for key in [
        "a11y-tree.json",
        "focus-trace.json",
        "pointer-hitmap.json",
        "ime-trace.json",
    ] {
        assert_eq!(
            b1.sha256s.get(key),
            b2.sha256s.get(key),
            "{} differs across runs",
            key
        );
    }
    assert_eq!(
        b1.sha256s.get("mui-button-chromium-1.0.png"),
        b2.sha256s.get("mui-button-chromium-1.0.png"),
        "pixel.png differs across runs"
    );
}

/// T9 — runner attaches CDP session and can call Accessibility/Input domains.
/// Stub-backed: ensures the BrowserSession trait carries the required hooks.
/// (Live verification is gated on the #2139 browser harness follow-up.)
#[tokio::test]
async fn test_cdp_session_attached() {
    let mut stub = StubBrowserSession::new();
    // Domain reachability is modeled as: ax_full_tree + capture_focus_trace
    // + capture_ime_trace all callable without panic.
    let ax = stub.browser_kind();
    assert_eq!(ax, BrowserKind::Chromium);
    // Drive the full lifecycle to make sure all three CDP-mapped hooks are
    // invoked through Runner.
    let tmp = tempfile::tempdir().unwrap();
    let bundle = run_with_stub(tmp.path(), stub).await.unwrap();
    assert!(bundle.a11y_json.exists());
    assert!(bundle.focus_json.exists());
    assert!(bundle.ime_json.exists());
}

/// T10 — fixture whose shell never sets `__jet_oracle_mounted` fails with `MountTimeout`.
#[tokio::test]
async fn test_mount_sentinel_timeout() {
    let mut stub = StubBrowserSession::new();
    stub.will_mount = false;
    let tmp = tempfile::tempdir().unwrap();
    let err = run_with_stub(tmp.path(), stub).await.unwrap_err();
    assert!(
        matches!(err, RunnerError::MountTimeout(_)),
        "expected MountTimeout, got {:?}",
        err
    );
}

/// T11 — single-fixture wall clock stays under 8s on the reference runner.
/// Stub-backed timing covers the harness budget; live timing is gated.
#[tokio::test]
async fn test_per_fixture_budget_under_8s() {
    let tmp = tempfile::tempdir().unwrap();
    let start = Instant::now();
    let _ = run_with_stub(tmp.path(), StubBrowserSession::new())
        .await
        .unwrap();
    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_secs(8),
        "stub run took {:?}, > 8s budget",
        elapsed
    );
}

/// Live-Chromium smoke test (R11 wall-clock + R9 byte-equivalent replay).
/// Skipped by default — depends on the browser-harness follow-up.
#[tokio::test]
#[ignore = "blocked on live browser harness — issue #2139 follow-up"]
async fn test_runner_live_chromium_smoke() {
    if std::env::var("JET_PARITY_ORACLE_SKIP").is_ok() {
        eprintln!("skipped: JET_PARITY_ORACLE_SKIP set");
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let cfg = make_config(tmp.path());
    let matrix = MatrixEntry {
        browser: BrowserKind::Chromium,
        dpr: 1.0,
    };
    let _ = run_fixture(&cfg, &fixture_path(), matrix).await.unwrap();
}
// CODEGEN-END
