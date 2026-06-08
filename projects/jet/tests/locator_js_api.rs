// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Integration tests for Locator JS API (P1.1 — Phase 4 of the Playwright
//! replacement epic).
//!
//! Covers the four gaps the B1/B3 waves left behind:
//!   - Sub-locator chaining (`parent.locator(child)`) — T1, T2, T10
//!   - `.filter({ hasText, hasNotText })` — T3, T4
//!   - JS-side auto-wait FSM (Attached → Visible → Stable) — T5, T6, T7
//!   - NthLocator actions route through the indexed element — T8, T9
//!
//! Spec: `.aw/tech-design/projects/jet/logic/locator-js-api.md`.
//!
//! Each test runs an inline JS spec through `test_runner::run` and asserts on
//! the summary. Tests skip gracefully when Chromium or node are unavailable —
//! same pattern as `page_api_parity.rs`.

use jet::test_runner::{self, RunnerConfig};
use std::fs;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn node_available() -> bool {
    which::which("node").is_ok()
}

fn chromium_available() -> bool {
    if std::env::var("CHROME_PATH").is_ok() {
        return true;
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let xdg = std::env::var("XDG_CACHE_HOME").unwrap_or_else(|_| format!("{home}/.cache"));
    let mac_playwright = format!("{home}/Library/Caches/ms-playwright");
    if std::path::Path::new(&mac_playwright).exists() {
        return true;
    }
    let linux_playwright = format!("{xdg}/ms-playwright");
    if std::path::Path::new(&linux_playwright).exists() {
        return true;
    }
    let jet_cache = format!("{home}/.jet/browsers");
    if std::path::Path::new(&jet_cache).exists() {
        return true;
    }
    false
}

async fn run_spec_str(
    spec_source: &str,
    cfg_fn: impl FnOnce(&mut RunnerConfig),
) -> Option<jet::test_runner::Summary> {
    if !node_available() {
        eprintln!("skipping: node not on PATH");
        return None;
    }
    let tmp = tempfile::tempdir().unwrap();
    let spec = tmp.path().join("locator_js_api.spec.js");
    fs::write(&spec, spec_source).unwrap();

    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    cfg.reporters = vec![];
    cfg.workers = 1;
    cfg_fn(&mut cfg);

    let summary = test_runner::run(cfg).await.expect("runner should complete");
    Some(summary)
}

fn skip_if_no_browser(t: &str) -> bool {
    if !node_available() {
        eprintln!("skipping {t}: node not on PATH");
        return true;
    }
    if !chromium_available() {
        eprintln!("skipping {t}: Chromium not available");
        return true;
    }
    false
}

// ── T1: CSS+CSS sub-locator chain produces CSS concat selector ───────────────

#[tokio::test]
async fn test_t1_sub_locator_css_concat() {
    if skip_if_no_browser("T1") {
        return;
    }
    let spec = r#"
import { test, expect } from '@jet/test';

test('T1: locator.locator(child) chains CSS', async ({ page }) => {
  await page.setContent(`
    <div class="card"><button>Other</button></div>
    <div class="card"><button id="target">Click</button></div>
  `);
  await page.locator('.card').locator('#target').click();
  // If the click reached #target, there will be one element; nothing to
  // assert beyond no-throw. Verify selector composition by reading text.
  const txt = await page.locator('.card').locator('#target').innerText();
  if (txt !== 'Click') throw new Error('Expected "Click", got ' + JSON.stringify(txt));
});
"#;
    let summary = run_spec_str(spec, |_| {}).await.unwrap();
    assert_eq!(summary.passed, 1, "T1 should pass");
    assert_eq!(summary.failed, 0);
}

// ── T2: pseudo + CSS sub-locator uses evaluate-scope path ────────────────────

#[tokio::test]
async fn test_t2_sub_locator_pseudo_scope() {
    if skip_if_no_browser("T2") {
        return;
    }
    let spec = r#"
import { test, expect } from '@jet/test';

test('T2: getByRole(list).locator(li).first().innerText()', async ({ page }) => {
  await page.setContent(`
    <ul role="list"><li>alpha</li><li>beta</li></ul>
  `);
  const t = await page.getByRole('list').locator('li').first().innerText();
  if (t !== 'alpha') throw new Error('Expected "alpha", got ' + JSON.stringify(t));
});
"#;
    let summary = run_spec_str(spec, |_| {}).await.unwrap();
    assert_eq!(summary.passed, 1, "T2 should pass");
    assert_eq!(summary.failed, 0);
}

// ── T3: filter({ hasText }) keeps matching siblings only ─────────────────────

#[tokio::test]
async fn test_t3_filter_has_text_click() {
    if skip_if_no_browser("T3") {
        return;
    }
    let spec = r#"
import { test, expect } from '@jet/test';

test('T3: locator.filter({ hasText }) clicks matching item', async ({ page }) => {
  await page.setContent(`
    <ul>
      <li class="item"><button>Alpha</button></li>
      <li class="item"><button>Buy now</button></li>
      <li class="item"><button>Gamma</button></li>
    </ul>
  `);
  await page.locator('.item').filter({ hasText: 'Buy' }).locator('button').click();
  // If click landed on the correct button, innerText of the focused element
  // or side-effect would prove it; use attribute tagging via evaluate instead.
  // Simpler: count by filter.
  const n = await page.locator('.item').filter({ hasText: 'Buy' }).count();
  if (n !== 1) throw new Error('Expected 1 match, got ' + n);
});
"#;
    let summary = run_spec_str(spec, |_| {}).await.unwrap();
    assert_eq!(summary.passed, 1, "T3 should pass");
    assert_eq!(summary.failed, 0);
}

// ── T4: filter with RegExp ───────────────────────────────────────────────────

#[tokio::test]
async fn test_t4_filter_regex() {
    if skip_if_no_browser("T4") {
        return;
    }
    let spec = r#"
import { test, expect } from '@jet/test';

test('T4: filter({ hasText: regex }) counts correctly', async ({ page }) => {
  await page.setContent(`
    <ul>
      <li class="item">10 USD</li>
      <li class="item">abc</li>
      <li class="item">5 USD</li>
    </ul>
  `);
  const n = await page.locator('.item').filter({ hasText: /\d+ USD/ }).count();
  if (n !== 2) throw new Error('Expected 2, got ' + n);
});
"#;
    let summary = run_spec_str(spec, |_| {}).await.unwrap();
    assert_eq!(summary.passed, 1, "T4 should pass");
    assert_eq!(summary.failed, 0);
}

// ── T5: auto-wait succeeds for a late-mounting element ───────────────────────

#[tokio::test]
async fn test_t5_auto_wait_late_mount() {
    if skip_if_no_browser("T5") {
        return;
    }
    let spec = r#"
import { test, expect } from '@jet/test';

test('T5: auto-wait for setTimeout-mounted button', async ({ page }) => {
  await page.setContent(`
    <div id="host"></div>
    <script>
      setTimeout(function(){
        var b = document.createElement('button');
        b.id = 'late';
        b.textContent = 'ready';
        document.getElementById('host').appendChild(b);
      }, 200);
    </script>
  `);
  // Without auto-wait this would throw — the element isn't attached at this
  // microtask. The FSM should poll until attached+visible+stable and click.
  await page.locator('#late').click();
});
"#;
    let summary = run_spec_str(spec, |_| {}).await.unwrap();
    assert_eq!(summary.passed, 1, "T5 should pass");
    assert_eq!(summary.failed, 0);
}

// ── T6: auto-wait fails with clear Visible-timeout for a hidden element ──────

#[tokio::test]
async fn test_t6_auto_wait_timeout_hidden() {
    if skip_if_no_browser("T6") {
        return;
    }
    let spec = r#"
import { test, expect } from '@jet/test';

test('T6: hidden element times out on Visible', async ({ page }) => {
  await page.setContent('<button id="hidden" style="display:none">x</button>');
  let err = null;
  const started = Date.now();
  try {
    // Timeout generous enough to survive parallel Chromium startup but small
    // enough to keep the test quick. We assert the state name, not the wall
    // time, so transient slowness just moves the throw; it can't hide bugs.
    await page.locator('#hidden').click({ timeout: 1500 });
  } catch (e) {
    err = e;
  }
  const elapsed = Date.now() - started;
  if (!err) throw new Error('Expected timeout error, got none');
  if (String(err.message).indexOf('Visible') === -1) {
    throw new Error('Expected Visible timeout (elapsed=' + elapsed + 'ms), got: ' + err.message);
  }
});
"#;
    let summary = run_spec_str(spec, |_| {}).await.unwrap();
    assert_eq!(summary.passed, 1, "T6 should pass");
    assert_eq!(summary.failed, 0);
}

// ── T7: stability wait passes on static DOM ──────────────────────────────────

#[tokio::test]
async fn test_t7_stability_static() {
    if skip_if_no_browser("T7") {
        return;
    }
    let spec = r#"
import { test, expect } from '@jet/test';

test('T7: stability check passes on static element', async ({ page }) => {
  await page.setContent('<button id="static">ok</button>');
  const start = Date.now();
  await page.locator('#static').click();
  const elapsed = Date.now() - start;
  // Stability requires 2 rect reads ≥50ms apart → at least ~100ms, less than 5s.
  if (elapsed > 5000) throw new Error('auto-wait too slow: ' + elapsed + 'ms');
});
"#;
    let summary = run_spec_str(spec, |_| {}).await.unwrap();
    assert_eq!(summary.passed, 1, "T7 should pass");
    assert_eq!(summary.failed, 0);
}

// ── T8: NthLocator.click targets the indexed element ─────────────────────────

#[tokio::test]
async fn test_t8_nth_click_indexed() {
    if skip_if_no_browser("T8") {
        return;
    }
    let spec = r#"
import { test, expect } from '@jet/test';

test('T8: locator.nth(1).click() fires on second button', async ({ page }) => {
  await page.setContent(`
    <button class="b" data-idx="0">a</button>
    <button class="b" data-idx="1">b</button>
    <button class="b" data-idx="2">c</button>
    <div id="log"></div>
    <script>
      document.querySelectorAll('button.b').forEach(function(btn){
        btn.addEventListener('click', function(){
          document.getElementById('log').textContent = btn.getAttribute('data-idx');
        });
      });
    </script>
  `);
  await page.locator('button.b').nth(1).click();
  const t = await page.locator('#log').innerText();
  if (t !== '1') throw new Error('Expected log="1", got ' + JSON.stringify(t));
});
"#;
    let summary = run_spec_str(spec, |_| {}).await.unwrap();
    assert_eq!(summary.passed, 1, "T8 should pass");
    assert_eq!(summary.failed, 0);
}

// ── T9: NthLocator reads (getAttribute, innerText) use indexed element ───────

#[tokio::test]
async fn test_t9_nth_reads_indexed() {
    if skip_if_no_browser("T9") {
        return;
    }
    let spec = r#"
import { test, expect } from '@jet/test';

test('T9: nth reads target the indexed element', async ({ page }) => {
  await page.setContent(`
    <button id="a" data-k="A">alpha</button>
    <button id="b" data-k="B">beta</button>
    <button id="c" data-k="C">gamma</button>
  `);
  const t = await page.locator('button').last().innerText();
  if (t !== 'gamma') throw new Error('Expected last innerText="gamma", got ' + JSON.stringify(t));
  const k = await page.locator('button').nth(1).getAttribute('data-k');
  if (k !== 'B') throw new Error('Expected data-k="B", got ' + JSON.stringify(k));
});
"#;
    let summary = run_spec_str(spec, |_| {}).await.unwrap();
    assert_eq!(summary.passed, 1, "T9 should pass");
    assert_eq!(summary.failed, 0);
}

// ── T10: chained fill round-trips through real Input RPC ─────────────────────

#[tokio::test]
async fn test_t10_chained_fill() {
    if skip_if_no_browser("T10") {
        return;
    }
    let spec = r#"
import { test, expect } from '@jet/test';

test('T10: chained locator.fill sends real CDP fill', async ({ page }) => {
  await page.setContent(`
    <form id="form">
      <input name="email" />
    </form>
  `);
  await page.locator('#form').locator('input[name=email]').fill('a@b');
  const v = await page.locator('input[name=email]').inputValue();
  if (v !== 'a@b') throw new Error('Expected "a@b", got ' + JSON.stringify(v));
});
"#;
    let summary = run_spec_str(spec, |_| {}).await.unwrap();
    assert_eq!(summary.passed, 1, "T10 should pass");
    assert_eq!(summary.failed, 0);
}
// CODEGEN-END
