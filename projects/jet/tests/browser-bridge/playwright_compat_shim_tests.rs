// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Integration tests for the `@playwright/test` runtime compat shim (P4.5).
//!
//! Distinct from `playwright_compat_tests.rs` which covers the
//! `--playwright` escape hatch (subprocess delegate).
//!
//! Spec: `.aw/tech-design/projects/jet/logic/playwright-compat-shim-reexport.md`.

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

async fn run_spec(spec: &str) -> Option<test_runner::Summary> {
    if !node_available() {
        return None;
    }
    let tmp = tempfile::tempdir().unwrap();
    let spec_path = tmp.path().join("pw.spec.js");
    fs::write(&spec_path, spec).unwrap();
    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    cfg.reporters = vec![];
    cfg.workers = 1;
    Some(test_runner::run(cfg).await.expect("runner"))
}

// ── PC1: named imports resolve ──────────────────────────────────────────────

#[tokio::test]
async fn pc1_named_imports() {
    if !node_available() {
        return;
    }
    let spec = r#"
import { test, expect } from '@playwright/test';

test('PC1: named imports work', () => {
  expect(1 + 1).toBe(2);
});
"#;
    let s = run_spec(spec).await.unwrap();
    assert_eq!(s.passed, 1, "{:?}", s);
    assert_eq!(s.failed, 0);
}

// ── PC2: browser namespace ──────────────────────────────────────────────────

#[tokio::test]
async fn pc2_browser_namespace() {
    if !node_available() || !chromium_available() {
        eprintln!("skipping PC2: chromium needed");
        return;
    }
    let spec = r#"
import { test, browser } from '@playwright/test';

test('PC2: browser.newContext via compat shim', async () => {
  const ctx = await browser.newContext();
  await ctx.close();
});
"#;
    let s = run_spec(spec).await.unwrap();
    assert_eq!(s.passed, 1, "{:?}", s);
    assert_eq!(s.failed, 0);
}

// ── PC3: default import yields namespace ────────────────────────────────────

#[tokio::test]
async fn pc3_default_namespace() {
    if !node_available() {
        return;
    }
    let spec = r#"
import pw from '@playwright/test';

pw.test('PC3: default-import namespace', () => {
  pw.expect('hello').toBe('hello');
});
"#;
    let s = run_spec(spec).await.unwrap();
    assert_eq!(s.passed, 1, "{:?}", s);
    assert_eq!(s.failed, 0);
}
// CODEGEN-END
