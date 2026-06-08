// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Integration tests for storage_state (P3.2).
//!
//! Spec: `.aw/tech-design/projects/jet/logic/storage-state.md`.

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
        return None;
    }
    let tmp = tempfile::tempdir().unwrap();
    let spec = tmp.path().join("storage.spec.js");
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

// ── S_t1: addCookies → cookies roundtrip ─────────────────────────────────────

#[tokio::test]
async fn s_t1_add_cookies_roundtrip() {
    if skip("S_t1") {
        return;
    }
    let spec = r#"
import { test, expect, browser } from '@jet/test';

test('S_t1: addCookies + cookies roundtrip', async () => {
  const ctx = await browser.newContext();
  await ctx.addCookies([
    { name: 'session', value: 'abc123', url: 'http://example.com/' },
  ]);
  const cs = await ctx.cookies();
  const found = cs.find((c) => c.name === 'session');
  if (!found) throw new Error('Expected session cookie, got: ' + JSON.stringify(cs));
  if (found.value !== 'abc123') throw new Error('Wrong value: ' + found.value);
  await ctx.close();
});
"#;
    let s = run_spec_str(spec).await.unwrap();
    assert_eq!(s.passed, 1, "S_t1 should pass, summary={:?}", s);
    assert_eq!(s.failed, 0);
}

// ── S_t2: clearCookies empties the jar ───────────────────────────────────────

#[tokio::test]
async fn s_t2_clear_cookies() {
    if skip("S_t2") {
        return;
    }
    let spec = r#"
import { test, expect, browser } from '@jet/test';

test('S_t2: clearCookies removes everything', async () => {
  const ctx = await browser.newContext();
  await ctx.addCookies([
    { name: 'a', value: '1', url: 'http://example.com/' },
    { name: 'b', value: '2', url: 'http://example.com/' },
  ]);
  const before = await ctx.cookies();
  if (before.length < 2) throw new Error('Expected ≥2 cookies, got ' + before.length);
  await ctx.clearCookies();
  const after = await ctx.cookies();
  const leftover = after.filter((c) => c.name === 'a' || c.name === 'b');
  if (leftover.length !== 0) throw new Error('leftover cookies: ' + JSON.stringify(leftover));
  await ctx.close();
});
"#;
    let s = run_spec_str(spec).await.unwrap();
    assert_eq!(s.passed, 1);
    assert_eq!(s.failed, 0);
}

// ── S_t3: storageState shape ─────────────────────────────────────────────────

#[tokio::test]
async fn s_t3_storage_state_shape() {
    if skip("S_t3") {
        return;
    }
    let spec = r#"
import { test, expect, browser } from '@jet/test';

test('S_t3: storageState returns {cookies, origins:[]}', async () => {
  const ctx = await browser.newContext();
  await ctx.addCookies([{ name: 'k', value: 'v', url: 'http://example.com/' }]);
  const state = await ctx.storageState();
  if (!Array.isArray(state.cookies)) throw new Error('cookies not array');
  if (!Array.isArray(state.origins)) throw new Error('origins not array');
  if (state.origins.length !== 0) throw new Error('origins should be empty for MVP');
  const found = state.cookies.find((c) => c.name === 'k');
  if (!found) throw new Error('cookie missing from state');
  await ctx.close();
});
"#;
    let s = run_spec_str(spec).await.unwrap();
    assert_eq!(s.passed, 1);
    assert_eq!(s.failed, 0);
}

// ── S_t4: setStorageState applies cookies ───────────────────────────────────

#[tokio::test]
async fn s_t4_set_storage_state() {
    if skip("S_t4") {
        return;
    }
    let spec = r#"
import { test, expect, browser } from '@jet/test';

test('S_t4: setStorageState populates cookies', async () => {
  const ctx = await browser.newContext();
  await ctx.setStorageState({
    cookies: [{ name: 'auth', value: 'tok', url: 'http://example.com/' }],
    origins: [],
  });
  const cs = await ctx.cookies();
  const found = cs.find((c) => c.name === 'auth');
  if (!found) throw new Error('auth cookie missing');
  if (found.value !== 'tok') throw new Error('wrong value: ' + found.value);
  await ctx.close();
});
"#;
    let s = run_spec_str(spec).await.unwrap();
    assert_eq!(s.passed, 1);
    assert_eq!(s.failed, 0);
}

// ── S_t5: storageState({path}) → newContext({storageState: path}) ────────────

#[tokio::test]
async fn s_t5_file_roundtrip() {
    if skip("S_t5") {
        return;
    }
    // Create a temp file path the JS spec will read/write. The spec owns
    // the lifetime of the file via `os.tmpdir()`.
    let spec = r#"
import { test, expect, browser } from '@jet/test';

test('S_t5: storageState file save+load roundtrip', async () => {
  const os = await import('node:os');
  const path = await import('node:path');
  const crypto = await import('node:crypto');
  const fs = await import('node:fs/promises');
  const tmpfile = path.join(os.tmpdir(), 'jet-storage-' + crypto.randomUUID() + '.json');
  try {
    // Save.
    const ctx1 = await browser.newContext();
    await ctx1.addCookies([{ name: 'sess', value: 'xyz', url: 'http://example.com/' }]);
    await ctx1.storageState({ path: tmpfile });
    await ctx1.close();
    // Verify file wrote something.
    const body = JSON.parse(await fs.readFile(tmpfile, 'utf-8'));
    if (!Array.isArray(body.cookies)) throw new Error('file missing cookies');
    // Load into a fresh context.
    const ctx2 = await browser.newContext({ storageState: tmpfile });
    const cs = await ctx2.cookies();
    const found = cs.find((c) => c.name === 'sess');
    if (!found || found.value !== 'xyz') throw new Error('roundtrip failed: ' + JSON.stringify(cs));
    await ctx2.close();
  } finally {
    await fs.rm(tmpfile, { force: true });
  }
});
"#;
    let s = run_spec_str(spec).await.unwrap();
    assert_eq!(s.passed, 1, "S_t5 should pass, summary={:?}", s);
    assert_eq!(s.failed, 0);
}

// ── S_t6: unknown context_id surfaces error ──────────────────────────────────

#[tokio::test]
async fn s_t6_unknown_context_error() {
    if skip("S_t6") {
        return;
    }
    // Construct a __JetBrowserContext with a bogus id and verify the error
    // channel surfaces with a clear message. Since __JetBrowserContext is
    // not directly exported, go through browser.newContext then mutate the
    // internal id before the next call.
    let spec = r#"
import { test, expect, browser } from '@jet/test';

test('S_t6: unknown browserContextId errors with id in message', async () => {
  const ctx = await browser.newContext();
  ctx.__jet_context_id = 'nope-does-not-exist';
  let err = null;
  try { await ctx.cookies(); } catch (e) { err = e; }
  if (!err) throw new Error('Expected cookies() to reject');
  if (String(err.message).indexOf('nope-does-not-exist') === -1) {
    throw new Error('Expected id in message, got: ' + err.message);
  }
});
"#;
    let s = run_spec_str(spec).await.unwrap();
    assert_eq!(s.passed, 1);
    assert_eq!(s.failed, 0);
}
// CODEGEN-END
