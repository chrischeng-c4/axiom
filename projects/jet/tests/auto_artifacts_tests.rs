// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Integration tests for auto artifacts (P3.4).
//!
//! Spec: `.aw/tech-design/projects/jet/logic/auto-artifacts.md`.

use jet::browser::BrowserLauncher;
use jet::test_runner::{self, RunnerConfig};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::Mutex;

// Hard ceiling so a stuck runner fails the test (with a useful message) instead
// of hanging cargo test indefinitely. Bug #2220.
const RUNNER_HARD_TIMEOUT: Duration = Duration::from_secs(120);
static AUTO_ARTIFACTS_TEST_LOCK: Mutex<()> = Mutex::const_new(());

fn node_available() -> bool {
    which::which("node").is_ok()
}

fn chromium_available() -> bool {
    // CHROME_PATH override must point to an existing file (not just be set).
    if let Ok(p) = std::env::var("CHROME_PATH") {
        return Path::new(&p).is_file();
    }
    // Prefer jet's own cache layout — `BrowserLauncher::find_chrome_in`
    // verifies a real, executable binary exists under
    // `<root>/chromium-<rev>/...`, not merely that the directory exists.
    if let Ok(home) = std::env::var("HOME") {
        let cache = PathBuf::from(home).join(".jet").join("browsers");
        if BrowserLauncher::find_chrome_in(&cache).is_some() {
            return true;
        }
    }
    // System Chrome/Chromium fallbacks (same paths the launcher checks).
    let system_candidates: &[&str] = if cfg!(target_os = "macos") {
        &[
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
            "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary",
        ]
    } else if cfg!(target_os = "linux") {
        &[
            "/usr/bin/google-chrome",
            "/usr/bin/chromium",
            "/usr/bin/chromium-browser",
        ]
    } else {
        &[
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        ]
    };
    system_candidates.iter().any(|p| Path::new(p).is_file())
}

async fn run_spec(
    spec: &str,
    cfg_fn: impl FnOnce(&mut RunnerConfig),
) -> Option<test_runner::Summary> {
    if !node_available() {
        return None;
    }
    let _guard = AUTO_ARTIFACTS_TEST_LOCK.lock().await;
    let tmp = tempfile::tempdir().unwrap();
    let spec_path = tmp.path().join("artifacts.spec.js");
    fs::write(&spec_path, spec).unwrap();
    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    cfg.reporters = vec![];
    cfg.workers = 1;
    cfg_fn(&mut cfg);
    // Bounded outer timeout: if `test_runner::run` ever fails to release the
    // Chromium subprocess or stalls waiting on a wire response, fail the test
    // with a clear message rather than wedging cargo test. Bug #2220.
    let summary = match tokio::time::timeout(RUNNER_HARD_TIMEOUT, test_runner::run(cfg)).await {
        Ok(res) => res.expect("runner"),
        Err(_) => panic!(
            "test_runner::run did not complete within {}s — hang suspected (#2220)",
            RUNNER_HARD_TIMEOUT.as_secs()
        ),
    };
    let _ = tmp; // explicit scope
    Some(summary)
}

fn skip(label: &str) -> bool {
    if !node_available() {
        eprintln!("skipping {label}: node");
        return true;
    }
    if !chromium_available() {
        eprintln!("skipping {label}: Chromium");
        return true;
    }
    false
}

// ── AA1: failing spec captures PNG + reports path ────────────────────────────

#[tokio::test]
async fn aa1_failing_test_produces_artifact() {
    if skip("AA1") {
        return;
    }
    // Use a tmp dir OUTSIDE the spec's tmp so we can inspect artifacts after
    // the runner returns (the spec runner tmpdir lives only during the run).
    let art_tmp = tempfile::tempdir().unwrap();
    let art_dir = art_tmp.path().to_path_buf();
    let spec = r#"
import { test, expect } from '@jet/test';

test('AA1: explicitly fails', async ({ page }) => {
  await page.setContent('<h1>hi</h1>');
  throw new Error('intentional failure');
});
"#;
    let art_dir_clone = art_dir.clone();
    let summary = run_spec(spec, move |cfg| {
        cfg.auto_artifacts = true;
        cfg.auto_artifacts_dir = art_dir_clone;
    })
    .await
    .unwrap();
    assert_eq!(summary.failed, 1, "expected 1 failure");
    let report = &summary.reports[0];
    assert!(
        !report.artifacts.is_empty(),
        "expected ≥1 artifact, got {:?}",
        report.artifacts
    );
    // Paths should point to real files under art_dir.
    for p in &report.artifacts {
        assert!(p.starts_with(&art_dir), "artifact outside dir: {p:?}");
        assert!(p.exists(), "artifact missing on disk: {p:?}");
        let bytes = std::fs::metadata(p).unwrap().len();
        assert!(bytes > 0, "PNG is empty: {p:?}");
    }
}

// ── AA2: disabled via auto_artifacts=false ───────────────────────────────────

#[tokio::test]
async fn aa2_disabled_produces_empty() {
    if skip("AA2") {
        return;
    }
    let art_tmp = tempfile::tempdir().unwrap();
    let art_dir = art_tmp.path().to_path_buf();
    let spec = r#"
import { test } from '@jet/test';

test('AA2: fails with auto_artifacts=false', async ({ page }) => {
  await page.setContent('<p>x</p>');
  throw new Error('boom');
});
"#;
    let art_dir_clone = art_dir.clone();
    let summary = run_spec(spec, move |cfg| {
        cfg.auto_artifacts = false;
        cfg.auto_artifacts_dir = art_dir_clone;
    })
    .await
    .unwrap();
    assert_eq!(summary.failed, 1);
    assert!(summary.reports[0].artifacts.is_empty());
}

// ── AA3: multi-page capture ─────────────────────────────────────────────────

#[tokio::test]
async fn aa3_multi_page_capture() {
    if skip("AA3") {
        return;
    }
    let art_tmp = tempfile::tempdir().unwrap();
    let art_dir = art_tmp.path().to_path_buf();
    let spec = r#"
import { test, browser } from '@jet/test';

test('AA3: failing spec with default + extra page', async ({ page }) => {
  await page.setContent('<p>default</p>');
  const ctx = await browser.newContext();
  const extra = await ctx.newPage();
  await extra.setContent('<p>extra</p>');
  throw new Error('two-page boom');
});
"#;
    let art_dir_clone = art_dir.clone();
    let summary = run_spec(spec, move |cfg| {
        cfg.auto_artifacts = true;
        cfg.auto_artifacts_dir = art_dir_clone;
    })
    .await
    .unwrap();
    assert_eq!(summary.failed, 1);
    let n = summary.reports[0].artifacts.len();
    assert!(n >= 2, "expected ≥2 artifacts, got {n}");
}

// ── AA4: passing spec emits no artifacts ────────────────────────────────────

#[tokio::test]
async fn aa4_passing_test_no_artifacts() {
    if skip("AA4") {
        return;
    }
    let art_tmp = tempfile::tempdir().unwrap();
    let art_dir = art_tmp.path().to_path_buf();
    let spec = r#"
import { test } from '@jet/test';

test('AA4: passes', async ({ page }) => {
  await page.setContent('<p>ok</p>');
});
"#;
    let art_dir_clone = art_dir.clone();
    let summary = run_spec(spec, move |cfg| {
        cfg.auto_artifacts = true;
        cfg.auto_artifacts_dir = art_dir_clone;
    })
    .await
    .unwrap();
    assert_eq!(summary.passed, 1);
    assert!(summary.reports[0].artifacts.is_empty());
}
// CODEGEN-END
