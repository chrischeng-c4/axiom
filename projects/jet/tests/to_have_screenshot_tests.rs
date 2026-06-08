// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Integration tests for toHaveScreenshot (P2.5).
//!
//! Spec: `.aw/tech-design/projects/jet/logic/to-have-screenshot.md`.

use jet::test_runner::{self, RunnerConfig};
use std::fs;

fn node_available() -> bool {
    which::which("node").is_ok()
}

fn chromium_available() -> bool {
    if std::env::var("CHROME_PATH").is_ok() {
        return true;
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let xdg = std::env::var("XDG_CACHE_HOME").unwrap_or_else(|_| format!("{home}/.cache"));
    [
        format!("{home}/Library/Caches/ms-playwright"),
        format!("{xdg}/ms-playwright"),
        format!("{home}/.jet/browsers"),
    ]
    .iter()
    .any(|p| std::path::Path::new(p).exists())
}

// Special helper: persist spec dir across the test so we can run the same
// spec twice (first run writes baseline, second run matches).
async fn run_spec_in_dir(
    dir: &std::path::Path,
    spec_filename: &str,
    spec: &str,
) -> test_runner::Summary {
    let spec_path = dir.join(spec_filename);
    fs::write(&spec_path, spec).unwrap();
    let mut cfg = RunnerConfig::default_for_root(dir).unwrap();
    cfg.reporters = vec![];
    cfg.workers = 1;
    test_runner::run(cfg).await.expect("runner")
}

fn skip(l: &str) -> bool {
    if !node_available() {
        eprintln!("skipping {l}: node");
        return true;
    }
    if !chromium_available() {
        eprintln!("skipping {l}: Chromium");
        return true;
    }
    false
}

// ── TS1: first run writes baseline, second run passes ───────────────────────

#[tokio::test]
async fn ts1_first_run_writes_second_matches() {
    if skip("TS1") {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let spec = r#"
import { test, expect } from '@jet/test';

test('TS1: baseline snapshot', async ({ page }) => {
  await page.setContent('<html><body style="background:#fff"><h1>TS1</h1></body></html>');
  await expect(page).toHaveScreenshot();
});
"#;
    // First run — baseline writes.
    let s1 = run_spec_in_dir(dir.path(), "ts1.spec.js", spec).await;
    assert_eq!(s1.passed, 1, "{:?}", s1);
    // Second run — baseline matches.
    let s2 = run_spec_in_dir(dir.path(), "ts1.spec.js", spec).await;
    assert_eq!(s2.passed, 1, "{:?}", s2);
}

// ── TS2: named baseline file ────────────────────────────────────────────────

#[tokio::test]
async fn ts2_named_baseline() {
    if skip("TS2") {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let spec = r#"
import { test, expect } from '@jet/test';

test('TS2: named baseline', async ({ page }) => {
  await page.setContent('<html><body><p>home</p></body></html>');
  await expect(page).toHaveScreenshot('home');
});
"#;
    let s = run_spec_in_dir(dir.path(), "ts2.spec.js", spec).await;
    assert_eq!(s.passed, 1, "{:?}", s);
    // Verify the baseline file name is `home.png`.
    // __snapshots__/<slug>/home.png should exist.
    let mut found = false;
    for entry in walkdir::WalkDir::new(dir.path()) {
        let e = entry.unwrap();
        if e.file_name() == "home.png" {
            found = true;
            break;
        }
    }
    assert!(found, "home.png baseline not written");
}

// ── TS3: locator target rejected ────────────────────────────────────────────

#[tokio::test]
async fn ts3_locator_rejected() {
    if skip("TS3") {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let spec = r#"
import { test, expect } from '@jet/test';

test('TS3: locator not supported', async ({ page }) => {
  await page.setContent('<button id="b">x</button>');
  let err = null;
  try { await expect(page.locator('#b')).toHaveScreenshot(); }
  catch (e) { err = e; }
  if (!err) throw new Error('expected error');
  if (!String(err.message).includes('expected a Page object')) {
    throw new Error('unexpected: ' + err.message);
  }
});
"#;
    let s = run_spec_in_dir(dir.path(), "ts3.spec.js", spec).await;
    assert_eq!(s.passed, 1, "{:?}", s);
}
// CODEGEN-END
