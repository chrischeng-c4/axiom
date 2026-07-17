// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Integration tests for page.route / unroute / unrouteAll — declarative
//! v1 network interception via the CDP Fetch domain (#1911, superseding
//! the P3.3 fetch/XHR-override MVP).
//!
//! v1 is DECLARATIVE: `page.route(pattern, descriptor)` registers a route
//! over the wire and matching + resolution happen entirely on the Rust
//! side (`browser::route`, driven by `Fetch.requestPaused`). There is no
//! JS handler-callback channel — see RI10 for the function-form rejection.
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

// ── RI1: fetch + glob mock (fulfill) ────────────────────────────────────────

#[tokio::test]
async fn ri1_fetch_glob_mock() {
    if skip("RI1") {
        return;
    }
    let spec = r#"
import { test } from '@jet/test';

test('RI1: fetch matches glob', async ({ page }) => {
  await page.setContent('<p>x</p>');
  await page.route('**/api/users', { fulfill: { status: 201, body: '{"ok":true}', contentType: 'application/json' } });
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

// ── RI2: fetch + glob extension mock ────────────────────────────────────────
// Regex patterns are out of scope for v1 (architecture decision, #1911) —
// this used to be a RegExp test; the equivalent glob for "any path ending
// in /img/<name>.webp" is `**/img/*.webp`.

#[tokio::test]
async fn ri2_fetch_glob_extension_mock() {
    if skip("RI2") {
        return;
    }
    let spec = r#"
import { test } from '@jet/test';

test('RI2: fetch matches glob extension pattern', async ({ page }) => {
  await page.setContent('<p>x</p>');
  await page.route('**/img/*.webp', { fulfill: { status: 200, body: 'FAKEWEBP' } });
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
    // data: URLs are resolved in-renderer and never traverse the network
    // stack, so they never reach CDP's Fetch domain (no
    // Fetch.requestPaused fires) — this exercises "a registered route
    // that doesn't match doesn't interfere with unrelated traffic" rather
    // than the Fetch-domain match-miss path specifically.
    let spec = r#"
import { test } from '@jet/test';

test('RI3: unmatched fetch passes through', async ({ page }) => {
  await page.setContent('<p>x</p>');
  await page.route('**/api/**', { fulfill: { status: 200, body: 'MOCK' } });
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
    // Chromium deliberately returns an opaque `TypeError: Failed to
    // fetch` for every network-level fetch() failure regardless of the
    // underlying CDP error reason, so the assertion only pins "the fetch
    // rejected" — not jet-specific wording (that belonged to the retired
    // JS-override MVP, which threw its own synthetic error message).
    let spec = r#"
import { test } from '@jet/test';

test('RI4: abort rejects fetch', async ({ page }) => {
  await page.setContent('<p>x</p>');
  await page.route('**/bad/**', { abort: true });
  const err = await page.evaluate(async () => {
    try { await fetch('/bad/thing'); return null; }
    catch (e) { return String(e.message || e); }
  });
  if (!err) throw new Error('expected fetch to reject for an aborted route');
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
  await page.route('**/a', { fulfill: { status: 200, body: 'A' } });
  await page.route('**/b', { fulfill: { status: 200, body: 'B' } });
  const n = await page.unroute('**/a');
  if (n !== 1) throw new Error('unroute count=' + n);
  // /b should still be mocked after removing only the /a route.
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
// CDP's Fetch domain intercepts at the network-stack layer, below any
// specific JS request API — unlike the old per-API JS override MVP, XHR
// needs no separate handling from fetch().

#[tokio::test]
async fn ri7_xhr_mock() {
    if skip("RI7") {
        return;
    }
    let spec = r#"
import { test } from '@jet/test';

test('RI7: xhr fulfilled', async ({ page }) => {
  await page.setContent('<p>x</p>');
  await page.route('**/xhr/**', { fulfill: { status: 202, body: 'XOK' } });
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

// ── RI9: last-registered route wins ─────────────────────────────────────────
// Playwright precedence (architecture decision, #1911): the LAST
// registered overlapping route wins, not the first. The pre-#1911 MVP had
// this backwards (first-match-wins) — this spec pins the corrected
// direction.

#[tokio::test]
async fn ri9_last_registered_route_wins() {
    if skip("RI9") {
        return;
    }
    let spec = r#"
import { test } from '@jet/test';

test('RI9: last registered route wins', async ({ page }) => {
  await page.setContent('<p>x</p>');
  await page.route('**/api/**', { fulfill: { status: 200, body: 'FIRST' } });
  await page.route('**/api/**', { fulfill: { status: 200, body: 'SECOND' } });
  const body = await page.evaluate(async () => (await fetch('/api/x')).text());
  if (body !== 'SECOND') throw new Error('body=' + body);
});
"#;
    let s = run_spec(spec).await.unwrap();
    assert_eq!(s.passed, 1);
    assert_eq!(s.failed, 0);
}

// ── RI10: function-handler form throws an actionable error (v1 stub) ───────
// Playwright's `page.route(pattern, async route => {...})` form is
// recognized for shape parity but is unsupported in v1 (architecture
// decision, #1911) — it must throw synchronously naming the supported
// descriptor form, never silently no-op or reach the wire.

#[tokio::test]
async fn ri10_function_handler_throws_actionable_error() {
    if skip("RI10") {
        return;
    }
    let spec = r#"
import { test } from '@jet/test';

test('RI10: function handler throws actionable error', async ({ page }) => {
  await page.setContent('<p>x</p>');
  let message = null;
  try {
    await page.route('**/api/**', async (route) => { await route.continue(); });
  } catch (e) {
    message = String(e.message || e);
  }
  if (!message) throw new Error('expected page.route(pattern, fn) to throw');
  if (!message.includes('descriptor')) {
    throw new Error('error should name the descriptor alternative: ' + message);
  }
});
"#;
    let s = run_spec(spec).await.unwrap();
    assert_eq!(s.passed, 1, "{:?}", s);
    assert_eq!(s.failed, 0);
}
// CODEGEN-END
