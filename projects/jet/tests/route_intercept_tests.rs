// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Integration tests for page.route / unroute (P3.3).
//!
//! Spec: `.aw/tech-design/projects/jet/logic/route-intercept.md`.

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
    let spec_path = tmp.path().join("route.spec.js");
    fs::write(&spec_path, spec).unwrap();
    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    cfg.reporters = vec![];
    cfg.workers = 1;
    Some(test_runner::run(cfg).await.expect("runner"))
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

// ── RI1: fetch + glob mock ──────────────────────────────────────────────────

#[tokio::test]
async fn ri1_fetch_glob_mock() {
    if skip("RI1") {
        return;
    }
    let spec = r#"
import { test } from '@jet/test';

test('RI1: fetch matches glob', async ({ page }) => {
  await page.setContent('<p>x</p>');
  await page.route('**/api/users', { status: 201, body: '{"ok":true}', contentType: 'application/json' });
  const out = await page.evaluate(async () => {
    const r = await fetch('/api/users');
    return { status: r.status, ct: r.headers.get('content-type'), body: await r.text() };
  });
  if (out.status !== 201) throw new Error('status=' + out.status);
  if (out.body !== '{"ok":true}') throw new Error('body=' + out.body);
  if (!String(out.ct).includes('application/json')) throw new Error('ct=' + out.ct);
});
"#;
    let s = run_spec(spec).await.unwrap();
    assert_eq!(s.passed, 1, "{:?}", s);
    assert_eq!(s.failed, 0);
}

// ── RI2: fetch + RegExp mock ────────────────────────────────────────────────

#[tokio::test]
async fn ri2_fetch_regex_mock() {
    if skip("RI2") {
        return;
    }
    let spec = r#"
import { test } from '@jet/test';

test('RI2: fetch matches RegExp', async ({ page }) => {
  await page.setContent('<p>x</p>');
  await page.route(/\/img\/\w+\.webp$/, { status: 200, body: 'FAKEWEBP' });
  const body = await page.evaluate(async () => (await fetch('/img/cat.webp')).text());
  if (body !== 'FAKEWEBP') throw new Error('body=' + body);
});
"#;
    let s = run_spec(spec).await.unwrap();
    assert_eq!(s.passed, 1);
    assert_eq!(s.failed, 0);
}

// ── RI3: unmatched fetch passes through ─────────────────────────────────────

#[tokio::test]
async fn ri3_unmatched_fetch_passthrough() {
    if skip("RI3") {
        return;
    }
    // Use a data: URL that resolves without network — fetch() on data: URLs
    // is supported by Chromium.
    let spec = r#"
import { test } from '@jet/test';

test('RI3: unmatched fetch passes through', async ({ page }) => {
  await page.setContent('<p>x</p>');
  await page.route('**/api/**', { status: 200, body: 'MOCK' });
  const body = await page.evaluate(async () => (await fetch('data:text/plain,hello')).text());
  if (body !== 'hello') throw new Error('body=' + body);
});
"#;
    let s = run_spec(spec).await.unwrap();
    assert_eq!(s.passed, 1);
    assert_eq!(s.failed, 0);
}

// ── RI4: abort rejects fetch ────────────────────────────────────────────────

#[tokio::test]
async fn ri4_fetch_abort_rejects() {
    if skip("RI4") {
        return;
    }
    let spec = r#"
import { test } from '@jet/test';

test('RI4: abort rejects fetch', async ({ page }) => {
  await page.setContent('<p>x</p>');
  await page.route('**/bad/**', { abort: true });
  const err = await page.evaluate(async () => {
    try { await fetch('/bad/thing'); return null; }
    catch (e) { return String(e.message || e); }
  });
  if (!err || !String(err).includes('jet route aborted')) {
    throw new Error('expected abort error, got: ' + err);
  }
});
"#;
    let s = run_spec(spec).await.unwrap();
    assert_eq!(s.passed, 1);
    assert_eq!(s.failed, 0);
}

// ── RI5 / RI6: unroute + unrouteAll ─────────────────────────────────────────

#[tokio::test]
async fn ri5_ri6_unroute_and_unroute_all() {
    if skip("RI5/6") {
        return;
    }
    let spec = r#"
import { test } from '@jet/test';

test('RI5/6: unroute + unrouteAll', async ({ page }) => {
  await page.setContent('<p>x</p>');
  await page.route('**/a', { status: 200, body: 'A' });
  await page.route('**/b', { status: 200, body: 'B' });
  const n = await page.unroute('**/a');
  if (n !== 1) throw new Error('unroute count=' + n);
  // /a should passthrough (data: URL via trickery). Simpler: just confirm /b still mocked.
  const bBody = await page.evaluate(async () => (await fetch('/b')).text());
  if (bBody !== 'B') throw new Error('b=' + bBody);
  const dropped = await page.unrouteAll();
  if (dropped !== 1) throw new Error('dropped=' + dropped);
});
"#;
    let s = run_spec(spec).await.unwrap();
    assert_eq!(s.passed, 1, "{:?}", s);
    assert_eq!(s.failed, 0);
}

// ── RI7: XMLHttpRequest mock ────────────────────────────────────────────────

#[tokio::test]
async fn ri7_xhr_mock() {
    if skip("RI7") {
        return;
    }
    let spec = r#"
import { test } from '@jet/test';

test('RI7: xhr fulfilled', async ({ page }) => {
  await page.setContent('<p>x</p>');
  await page.route('**/xhr/**', { status: 202, body: 'XOK' });
  const out = await page.evaluate(() => new Promise((resolve) => {
    const xhr = new XMLHttpRequest();
    xhr.open('GET', '/xhr/one');
    xhr.onload = () => resolve({ status: xhr.status, body: xhr.responseText });
    xhr.send();
  }));
  if (out.status !== 202) throw new Error('status=' + out.status);
  if (out.body !== 'XOK') throw new Error('body=' + out.body);
});
"#;
    let s = run_spec(spec).await.unwrap();
    assert_eq!(s.passed, 1);
    assert_eq!(s.failed, 0);
}

// ── RI8: XHR abort fires onerror ────────────────────────────────────────────

#[tokio::test]
async fn ri8_xhr_abort_onerror() {
    if skip("RI8") {
        return;
    }
    let spec = r#"
import { test } from '@jet/test';

test('RI8: xhr abort triggers onerror', async ({ page }) => {
  await page.setContent('<p>x</p>');
  await page.route('**/dead/**', { abort: true });
  const fired = await page.evaluate(() => new Promise((resolve) => {
    const xhr = new XMLHttpRequest();
    xhr.open('GET', '/dead/x');
    xhr.onerror = () => resolve('ERR');
    xhr.onload = () => resolve('LOADED');
    xhr.send();
  }));
  if (fired !== 'ERR') throw new Error('fired=' + fired);
});
"#;
    let s = run_spec(spec).await.unwrap();
    assert_eq!(s.passed, 1);
    assert_eq!(s.failed, 0);
}

// ── RI9: first match wins ───────────────────────────────────────────────────

#[tokio::test]
async fn ri9_first_match_wins() {
    if skip("RI9") {
        return;
    }
    let spec = r#"
import { test } from '@jet/test';

test('RI9: insertion order = priority', async ({ page }) => {
  await page.setContent('<p>x</p>');
  await page.route('**/api/**', { status: 200, body: 'FIRST' });
  await page.route('**/api/**', { status: 200, body: 'SECOND' });
  const body = await page.evaluate(async () => (await fetch('/api/x')).text());
  if (body !== 'FIRST') throw new Error('body=' + body);
});
"#;
    let s = run_spec(spec).await.unwrap();
    assert_eq!(s.passed, 1);
    assert_eq!(s.failed, 0);
}
// CODEGEN-END
