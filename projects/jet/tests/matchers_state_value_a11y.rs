// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Integration tests for Phase 5 matchers (P2.1..P2.4).
//!
//! Covers `toBeChecked`, `toBeDisabled`, `toBeEnabled`, `toBeFocused`,
//! `toHaveCSS`, `toHaveAccessibleName`, `toHaveRole`, `toMatchObject`.
//!
//! Spec: `.aw/tech-design/projects/jet/logic/matchers-state-value-a11y.md`.

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
    let paths = [
        format!("{home}/Library/Caches/ms-playwright"),
        format!("{xdg}/ms-playwright"),
        format!("{home}/.jet/browsers"),
    ];
    paths.iter().any(|p| std::path::Path::new(p).exists())
}

async fn run_spec_str(spec_source: &str) -> Option<test_runner::Summary> {
    if !node_available() {
        eprintln!("skipping: node not on PATH");
        return None;
    }
    let tmp = tempfile::tempdir().unwrap();
    let spec = tmp.path().join("matchers.spec.js");
    fs::write(&spec, spec_source).unwrap();
    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    cfg.reporters = vec![];
    cfg.workers = 1;
    Some(test_runner::run(cfg).await.expect("runner should complete"))
}

fn skip(label: &str) -> bool {
    if !node_available() {
        eprintln!("skipping {label}: node not on PATH");
        return true;
    }
    if !chromium_available() {
        eprintln!("skipping {label}: Chromium not available");
        return true;
    }
    false
}

// ── M1: toBeChecked ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_m1a_to_be_checked_pass() {
    if skip("M1a") {
        return;
    }
    let spec = r#"
import { test, expect } from '@jet/test';

test('M1a: toBeChecked after click', async ({ page }) => {
  await page.setContent('<input id="cb" type="checkbox" />');
  await page.locator('#cb').click();
  await expect(page.locator('#cb')).toBeChecked();
});
"#;
    let s = run_spec_str(spec).await.unwrap();
    assert_eq!(s.passed, 1);
    assert_eq!(s.failed, 0);
}

#[tokio::test]
async fn test_m1b_to_be_checked_timeout() {
    if skip("M1b") {
        return;
    }
    let spec = r#"
import { test, expect } from '@jet/test';

test('M1b: toBeChecked fails fast on unchecked', async ({ page }) => {
  await page.setContent('<input id="cb" type="checkbox" />');
  let err = null;
  try {
    await expect(page.locator('#cb')).toBeChecked({ timeout: 500 });
  } catch (e) { err = e; }
  if (!err) throw new Error('Expected timeout error');
  if (String(err.message).indexOf('checked') === -1 &&
      String(err.message).indexOf('toBeChecked') === -1) {
    throw new Error('Unexpected error: ' + err.message);
  }
});
"#;
    let s = run_spec_str(spec).await.unwrap();
    assert_eq!(s.passed, 1);
    assert_eq!(s.failed, 0);
}

// ── M2 / M3: toBeDisabled / toBeEnabled ──────────────────────────────────────

#[tokio::test]
async fn test_m2_m3_disabled_enabled() {
    if skip("M2/M3") {
        return;
    }
    let spec = r#"
import { test, expect } from '@jet/test';

test('M2/M3: toBeDisabled + toBeEnabled', async ({ page }) => {
  await page.setContent('<button id="a" disabled>x</button><button id="b">y</button>');
  await expect(page.locator('#a')).toBeDisabled();
  await expect(page.locator('#b')).toBeEnabled();
});
"#;
    let s = run_spec_str(spec).await.unwrap();
    assert_eq!(s.passed, 1);
    assert_eq!(s.failed, 0);
}

// ── M4: toBeFocused ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_m4_focused() {
    if skip("M4") {
        return;
    }
    let spec = r#"
import { test, expect } from '@jet/test';

test('M4: toBeFocused after focus()', async ({ page }) => {
  await page.setContent('<input id="i" />');
  await page.evaluate("document.getElementById('i').focus()");
  await expect(page.locator('#i')).toBeFocused();
});
"#;
    let s = run_spec_str(spec).await.unwrap();
    assert_eq!(s.passed, 1);
    assert_eq!(s.failed, 0);
}

// ── M5: toHaveCSS ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_m5_css() {
    if skip("M5") {
        return;
    }
    let spec = r#"
import { test, expect } from '@jet/test';

test('M5: toHaveCSS on inline style', async ({ page }) => {
  await page.setContent('<div id="d" style="color: rgb(255, 0, 0); display: none">x</div>');
  await expect(page.locator('#d')).toHaveCSS('color', 'rgb(255, 0, 0)');
  await expect(page.locator('#d')).toHaveCSS('display', /none/);
});
"#;
    let s = run_spec_str(spec).await.unwrap();
    assert_eq!(s.passed, 1);
    assert_eq!(s.failed, 0);
}

// ── M6: toHaveAccessibleName ─────────────────────────────────────────────────

#[tokio::test]
async fn test_m6_accessible_name() {
    if skip("M6") {
        return;
    }
    let spec = r#"
import { test, expect } from '@jet/test';

test('M6: aria-label + innerText fallback', async ({ page }) => {
  await page.setContent(`
    <button id="a" aria-label="Close">X</button>
    <button id="b">Save</button>
  `);
  await expect(page.locator('#a')).toHaveAccessibleName('Close');
  await expect(page.locator('#b')).toHaveAccessibleName('Save');
});
"#;
    let s = run_spec_str(spec).await.unwrap();
    assert_eq!(s.passed, 1);
    assert_eq!(s.failed, 0);
}

// ── M7: toHaveRole ───────────────────────────────────────────────────────────

#[tokio::test]
async fn test_m7_role() {
    if skip("M7") {
        return;
    }
    let spec = r##"
import { test, expect } from '@jet/test';

test('M7: implicit + explicit role', async ({ page }) => {
  await page.setContent(`
    <button id="b">x</button>
    <div id="d" role="alert">x</div>
    <a id="a" href="#">x</a>
    <input id="i" type="checkbox" />
  `);
  await expect(page.locator('#b')).toHaveRole('button');
  await expect(page.locator('#d')).toHaveRole('alert');
  await expect(page.locator('#a')).toHaveRole('link');
  await expect(page.locator('#i')).toHaveRole('checkbox');
});
"##;
    let s = run_spec_str(spec).await.unwrap();
    assert_eq!(s.passed, 1);
    assert_eq!(s.failed, 0);
}

// ── M8: toMatchObject ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_m8_match_object() {
    if skip("M8") {
        return;
    }
    let spec = r#"
import { test, expect } from '@jet/test';

test('M8: partial object match', async ({ page: _page }) => {
  expect({ a: 1, b: 2 }).toMatchObject({ a: 1 });
  expect({ a: { x: '42', y: 'z' } }).toMatchObject({ a: { x: /^\d+$/ } });
  let threw = false;
  try {
    expect({ b: 2 }).toMatchObject({ a: 1 });
  } catch { threw = true; }
  if (!threw) throw new Error('Expected toMatchObject to throw on missing key');
  threw = false;
  try {
    expect([1, 2, 3]).toMatchObject([1, 2]);
  } catch { threw = true; }
  if (!threw) throw new Error('Expected toMatchObject to throw on array length mismatch');
});
"#;
    let s = run_spec_str(spec).await.unwrap();
    assert_eq!(s.passed, 1);
    assert_eq!(s.failed, 0);
}
// CODEGEN-END
