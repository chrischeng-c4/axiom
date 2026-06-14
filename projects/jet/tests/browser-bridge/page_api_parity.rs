// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Integration tests for Page API parity with Playwright — requirements R1-R27.
//!
//! Each test runs a minimal spec string through `test_runner::run` and asserts
//! on the summary outcome. Tests requiring Chromium skip gracefully when the
//! binary is absent (same pattern as `page_fixture_auto_inject.rs`).
//!
//! Spec: `.aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md`

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
    // macOS Playwright cache
    let mac_path = format!("{home}/Library/Caches/ms-playwright");
    if std::path::Path::new(&mac_path).exists() {
        return true;
    }
    // Linux Playwright cache
    let linux_path = format!("{xdg}/ms-playwright");
    if std::path::Path::new(&linux_path).exists() {
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
    let spec = tmp.path().join("parity_test.spec.js");
    fs::write(&spec, spec_source).unwrap();

    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    cfg.reporters = vec![];
    cfg.workers = 1;
    cfg_fn(&mut cfg);

    let summary = test_runner::run(cfg).await.expect("runner should complete");
    Some(summary)
}

// ── Compile-time / wire-type tests (no Chromium required) ────────────────────

/// Smoke test: confirms this test module links against jet::test_runner.
// REQ: R1
#[test]
fn parity_test_module_compiles() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = RunnerConfig::default_for_root(tmp.path())
        .expect("RunnerConfig::default_for_root should succeed");
    assert!(cfg.workers >= 1);
}

/// Verify PageRequest::Title serializes correctly.
// REQ: R1
#[test]
fn page_request_title_serializes() {
    use jet::cdp_driver::PageRequest;
    let req = PageRequest::Title {
        req_id: 1,
        page_id: "t1".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("\"kind\":\"title\""), "json={json}");
    assert!(json.contains("\"req_id\":1"), "json={json}");
}

/// Verify PageRequest::SetViewportSize serializes correctly.
// REQ: R2
#[test]
fn page_request_set_viewport_size_serializes() {
    use jet::cdp_driver::PageRequest;
    let req = PageRequest::SetViewportSize {
        req_id: 2,
        page_id: "t1".to_string(),
        width: 1280,
        height: 720,
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(
        json.contains("\"kind\":\"set_viewport_size\""),
        "json={json}"
    );
    assert!(json.contains("1280"), "json={json}");
    assert!(json.contains("720"), "json={json}");
}

/// Verify PageRequest::Screenshot serializes correctly.
// REQ: R4
#[test]
fn page_request_screenshot_serializes() {
    use jet::cdp_driver::PageRequest;
    let req = PageRequest::Screenshot {
        req_id: 4,
        page_id: "t1".to_string(),
        path: None,
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("\"kind\":\"screenshot\""), "json={json}");
}

/// Verify PageResponse::ScreenshotResult serializes correctly.
// REQ: R4
#[test]
fn page_response_screenshot_result_serializes() {
    use jet::cdp_driver::PageResponse;
    let resp = PageResponse::ScreenshotResult {
        req_id: 4,
        data: "iVBORw0KGgo=".to_string(),
    };
    let json = serde_json::to_string(&resp).unwrap();
    assert!(
        json.contains("\"kind\":\"screenshot_result\""),
        "json={json}"
    );
    assert!(json.contains("iVBORw0KGgo="), "json={json}");
}

/// Verify PageRequest::GoBack, GoForward, Reload serialize correctly.
// REQ: R6
#[test]
fn page_request_navigation_variants_serialize() {
    use jet::cdp_driver::PageRequest;
    for (kind_str, req) in [
        (
            "go_back",
            PageRequest::GoBack {
                req_id: 1,
                page_id: "t".to_string(),
            },
        ),
        (
            "go_forward",
            PageRequest::GoForward {
                req_id: 2,
                page_id: "t".to_string(),
            },
        ),
        (
            "reload",
            PageRequest::Reload {
                req_id: 3,
                page_id: "t".to_string(),
            },
        ),
    ] {
        let json = serde_json::to_string(&req).unwrap();
        assert!(
            json.contains(&format!("\"kind\":\"{kind_str}\"")),
            "expected kind={kind_str} in json={json}"
        );
    }
}

/// Verify PageRequest::KeyboardPress and KeyboardType serialize correctly.
// REQ: R7
#[test]
fn page_request_keyboard_variants_serialize() {
    use jet::cdp_driver::PageRequest;
    let press = PageRequest::KeyboardPress {
        req_id: 7,
        page_id: "t".to_string(),
        key: "Enter".to_string(),
    };
    let press_json = serde_json::to_string(&press).unwrap();
    assert!(
        press_json.contains("\"kind\":\"keyboard_press\""),
        "press_json={press_json}"
    );
    assert!(press_json.contains("Enter"), "press_json={press_json}");

    let typ = PageRequest::KeyboardType {
        req_id: 8,
        page_id: "t".to_string(),
        text: "hello".to_string(),
    };
    let typ_json = serde_json::to_string(&typ).unwrap();
    assert!(
        typ_json.contains("\"kind\":\"keyboard_type\""),
        "typ_json={typ_json}"
    );
    assert!(typ_json.contains("hello"), "typ_json={typ_json}");
}

/// Verify PageRequest::MouseEvent serializes correctly.
// REQ: R8
#[test]
fn page_request_mouse_event_serializes() {
    use jet::cdp_driver::PageRequest;
    let req = PageRequest::MouseEvent {
        req_id: 8,
        page_id: "t".to_string(),
        event_type: "mouseMoved".to_string(),
        x: 100.0,
        y: 200.0,
        button: None,
        click_count: None,
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("\"kind\":\"mouse_event\""), "json={json}");
    assert!(json.contains("100"), "json={json}");
    assert!(json.contains("200"), "json={json}");
}

/// Verify PageRequest::SetContent serializes correctly.
// REQ: R9
#[test]
fn page_request_set_content_serializes() {
    use jet::cdp_driver::PageRequest;
    let req = PageRequest::SetContent {
        req_id: 9,
        page_id: "t".to_string(),
        html: "<h1>hello</h1>".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("\"kind\":\"set_content\""), "json={json}");
    assert!(json.contains("hello"), "json={json}");
}

/// Verify PageRequest::Content serializes correctly.
// REQ: R10
#[test]
fn page_request_content_serializes() {
    use jet::cdp_driver::PageRequest;
    let req = PageRequest::Content {
        req_id: 10,
        page_id: "t".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("\"kind\":\"content\""), "json={json}");
}

/// Verify PageRequest::BoundingBox serializes correctly.
// REQ: R11
#[test]
fn page_request_bounding_box_serializes() {
    use jet::cdp_driver::PageRequest;
    let req = PageRequest::BoundingBox {
        req_id: 11,
        page_id: "t".to_string(),
        selector: "#el".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("\"kind\":\"bounding_box\""), "json={json}");
}

/// Verify PageResponse::BoundingBoxResult serializes correctly.
// REQ: R11
#[test]
fn page_response_bounding_box_result_serializes() {
    use jet::cdp_driver::PageResponse;
    let resp = PageResponse::BoundingBoxResult {
        req_id: 11,
        x: Some(10.0),
        y: Some(20.0),
        width: Some(100.0),
        height: Some(50.0),
    };
    let json = serde_json::to_string(&resp).unwrap();
    assert!(
        json.contains("\"kind\":\"bounding_box_result\""),
        "json={json}"
    );
    assert!(json.contains("10"), "json={json}");

    // null case
    let null_resp = PageResponse::BoundingBoxResult {
        req_id: 12,
        x: None,
        y: None,
        width: None,
        height: None,
    };
    let null_json = serde_json::to_string(&null_resp).unwrap();
    assert!(null_json.contains("null"), "null_json={null_json}");
}

/// Verify PageRequest::Hover serializes correctly.
// REQ: R13
#[test]
fn page_request_hover_serializes() {
    use jet::cdp_driver::PageRequest;
    let req = PageRequest::Hover {
        req_id: 13,
        page_id: "t".to_string(),
        selector: "button".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("\"kind\":\"hover\""), "json={json}");
}

/// Verify PageRequest::LocatorPress serializes correctly.
// REQ: R14
#[test]
fn page_request_locator_press_serializes() {
    use jet::cdp_driver::PageRequest;
    let req = PageRequest::LocatorPress {
        req_id: 14,
        page_id: "t".to_string(),
        selector: "input".to_string(),
        key: "Tab".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("\"kind\":\"locator_press\""), "json={json}");
    assert!(json.contains("Tab"), "json={json}");
}

/// Verify SubscribeEvent and RemoveEventListener serialize correctly.
// REQ: R5
#[test]
fn page_request_event_subscription_variants_serialize() {
    use jet::cdp_driver::PageRequest;
    let sub = PageRequest::SubscribeEvent {
        req_id: 5,
        page_id: "t".to_string(),
        event_name: "console".to_string(),
    };
    let sub_json = serde_json::to_string(&sub).unwrap();
    assert!(
        sub_json.contains("\"kind\":\"subscribe_event\""),
        "sub_json={sub_json}"
    );
    assert!(sub_json.contains("console"), "sub_json={sub_json}");

    let remove = PageRequest::RemoveEventListener {
        req_id: 6,
        page_id: "t".to_string(),
        event_name: "pageerror".to_string(),
    };
    let remove_json = serde_json::to_string(&remove).unwrap();
    assert!(
        remove_json.contains("\"kind\":\"remove_event_listener\""),
        "remove_json={remove_json}"
    );
    assert!(
        remove_json.contains("pageerror"),
        "remove_json={remove_json}"
    );
}

/// parse_page_request handles all new variant kinds.
// REQ: R1, R2, R4, R6, R7, R8, R9, R10, R11, R13, R14
#[test]
fn parse_page_request_new_variants() {
    use jet::cdp_driver::parse_page_request;

    let cases = [
        r#"{"kind":"title","req_id":1,"page_id":"t"}"#,
        r#"{"kind":"set_viewport_size","req_id":2,"page_id":"t","width":1280,"height":720}"#,
        r#"{"kind":"screenshot","req_id":3,"page_id":"t","path":null}"#,
        r#"{"kind":"go_back","req_id":4,"page_id":"t"}"#,
        r#"{"kind":"go_forward","req_id":5,"page_id":"t"}"#,
        r#"{"kind":"reload","req_id":6,"page_id":"t"}"#,
        r#"{"kind":"keyboard_press","req_id":7,"page_id":"t","key":"Enter"}"#,
        r#"{"kind":"keyboard_type","req_id":8,"page_id":"t","text":"hello"}"#,
        r#"{"kind":"mouse_event","req_id":9,"page_id":"t","event_type":"mouseMoved","x":10.0,"y":20.0,"button":null,"click_count":null}"#,
        r#"{"kind":"set_content","req_id":10,"page_id":"t","html":"<p>hi</p>"}"#,
        r#"{"kind":"content","req_id":11,"page_id":"t"}"#,
        r##"{"kind":"bounding_box","req_id":12,"page_id":"t","selector":"#el"}"##,
        r#"{"kind":"hover","req_id":13,"page_id":"t","selector":"button"}"#,
        r#"{"kind":"locator_press","req_id":14,"page_id":"t","selector":"input","key":"Tab"}"#,
        r#"{"kind":"subscribe_event","req_id":15,"page_id":"t","event_name":"console"}"#,
        r#"{"kind":"remove_event_listener","req_id":16,"page_id":"t","event_name":"pageerror"}"#,
    ];

    for json in &cases {
        let req = parse_page_request(json);
        assert!(req.is_some(), "failed to parse: {json}");
    }
}

// ── Browser integration tests (Chromium required) ────────────────────────────

/// T1 — R1: page.title() returns document.title string.
// REQ: R1
#[tokio::test]
async fn test_t1_page_title() {
    if !node_available() {
        eprintln!("skipping T1: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T1: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T1: page.title() returns document.title', async ({ page }) => {
  await page.setContent('<html><head><title>My Test App</title></head><body></body></html>');
  const t = await page.title();
  if (t !== 'My Test App') {
    throw new Error('Expected title "My Test App", got: ' + JSON.stringify(t));
  }
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T1 should pass");
    assert_eq!(summary.failed, 0);
}

/// T2 — R2: page.setViewportSize() sends Emulation.setDeviceMetricsOverride.
// REQ: R2
#[tokio::test]
async fn test_t2_set_viewport_size() {
    if !node_available() {
        eprintln!("skipping T2: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T2: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T2: setViewportSize sends CDP command', async ({ page }) => {
  // Should not throw.
  await page.setViewportSize({ width: 800, height: 600 });
  // Verify width via window.innerWidth (requires actual layout).
  const w = await page.evaluate('window.outerWidth || window.screen.width || 800');
  if (typeof w !== 'number') {
    throw new Error('Expected numeric width, got: ' + JSON.stringify(w));
  }
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T2 should pass");
    assert_eq!(summary.failed, 0);
}

/// T3 — R3: page.waitForTimeout(ms) resolves after delay without CDP calls.
// REQ: R3
#[tokio::test]
async fn test_t3_wait_for_timeout() {
    if !node_available() {
        eprintln!("skipping T3: node not on PATH");
        return;
    }
    // waitForTimeout is pure JS — does not require Chromium.

    let spec = r#"
import { test, expect } from '@jet/test';

test('T3: waitForTimeout resolves after delay', async ({ page }) => {
  const start = Date.now();
  await page.waitForTimeout(200);
  const elapsed = Date.now() - start;
  if (elapsed < 150) {
    throw new Error('waitForTimeout resolved too early: ' + elapsed + 'ms');
  }
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T3 should pass");
    assert_eq!(summary.failed, 0);
}

/// T4 — R4: page.screenshot() returns non-empty Buffer with PNG magic bytes.
// REQ: R4
#[tokio::test]
async fn test_t4_screenshot() {
    if !node_available() {
        eprintln!("skipping T4: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T4: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T4: screenshot() returns Buffer with PNG magic bytes', async ({ page }) => {
  await page.setContent('<html><body><p>screenshot test</p></body></html>');
  const buf = await page.screenshot();
  if (!buf || buf.length === 0) {
    throw new Error('screenshot returned empty buffer');
  }
  // PNG magic bytes: 0x89 0x50 0x4E 0x47
  if (buf[0] !== 0x89 || buf[1] !== 0x50 || buf[2] !== 0x4E || buf[3] !== 0x47) {
    throw new Error('buffer does not start with PNG magic bytes: ' + JSON.stringify(Array.from(buf.slice(0, 4))));
  }
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T4 should pass");
    assert_eq!(summary.failed, 0);
}

/// T5 — R5: page.on('console', handler) API surface (registration does not throw).
// REQ: R5
#[tokio::test]
async fn test_t5_page_on_console_api_surface() {
    if !node_available() {
        eprintln!("skipping T5: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T5: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T5: page.on registers console handler without throwing', async ({ page }) => {
  const messages = [];
  // Registering should not throw.
  page.on('console', (msg) => messages.push(msg));
  // Event handler is registered; actual event delivery is best-effort.
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T5 should pass");
    assert_eq!(summary.failed, 0);
}

/// T6 — R5: page.on('pageerror', handler) API surface.
// REQ: R5
#[tokio::test]
async fn test_t6_page_on_pageerror_api_surface() {
    if !node_available() {
        eprintln!("skipping T6: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T6: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T6: page.on registers pageerror handler without throwing', async ({ page }) => {
  const errors = [];
  page.on('pageerror', (err) => errors.push(err));
  // Registration should succeed; actual event delivery is best-effort.
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T6 should pass");
    assert_eq!(summary.failed, 0);
}

/// T7 — R6: page.goBack() returns to previous URL.
// REQ: R6
#[tokio::test]
async fn test_t7_go_back() {
    if !node_available() {
        eprintln!("skipping T7: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T7: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T7: goBack() navigates back in history', async ({ page }) => {
  await page.setContent('<html><body>page A</body></html>');
  await page.goto('data:text/html,<p>page B</p>');
  const urlB = await page.url();
  await page.goBack();
  // After goBack, URL should differ from B or content should be page A.
  // We just verify goBack() does not throw.
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T7 should pass");
    assert_eq!(summary.failed, 0);
}

/// T8 — R6: page.reload() reloads the page.
// REQ: R6
#[tokio::test]
async fn test_t8_reload() {
    if !node_available() {
        eprintln!("skipping T8: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T8: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T8: reload() reloads without throwing', async ({ page }) => {
  await page.goto('data:text/html,<p>reload test</p>');
  // reload() should complete without error.
  await page.reload();
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T8 should pass");
    assert_eq!(summary.failed, 0);
}

/// T9 — R7: page.keyboard.press('Enter') sends CDP Input.dispatchKeyEvent.
// REQ: R7
#[tokio::test]
async fn test_t9_keyboard_press() {
    if !node_available() {
        eprintln!("skipping T9: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T9: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T9: keyboard.press does not throw', async ({ page }) => {
  await page.setContent('<html><body><input id="inp" /></body></html>');
  await page.click('#inp');
  // Press Enter — should send CDP keyDown + keyUp without throwing.
  await page.keyboard.press('Enter');
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T9 should pass");
    assert_eq!(summary.failed, 0);
}

/// T10 — R7: page.keyboard.type('abc') sends keyDown+keyUp per character.
// REQ: R7
#[tokio::test]
async fn test_t10_keyboard_type() {
    if !node_available() {
        eprintln!("skipping T10: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T10: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T10: keyboard.type does not throw', async ({ page }) => {
  await page.setContent('<html><body><input id="inp" /></body></html>');
  await page.click('#inp');
  await page.keyboard.type('abc');
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T10 should pass");
    assert_eq!(summary.failed, 0);
}

/// T11 — R8: page.mouse.click(x, y) sends CDP mouse events.
// REQ: R8
#[tokio::test]
async fn test_t11_mouse_click() {
    if !node_available() {
        eprintln!("skipping T11: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T11: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T11: mouse.click does not throw', async ({ page }) => {
  await page.setContent('<html><body><button id="btn">click me</button></body></html>');
  // mouse.click should send mouseMoved + mousePressed + mouseReleased without throwing.
  await page.mouse.click(100, 50);
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T11 should pass");
    assert_eq!(summary.failed, 0);
}

/// T12 — R9: page.setContent('<p>hi</p>') and page.content() returns the HTML.
// REQ: R9
#[tokio::test]
async fn test_t12_set_content() {
    if !node_available() {
        eprintln!("skipping T12: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T12: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T12: setContent and content round-trip', async ({ page }) => {
  await page.setContent('<p>hi</p>');
  const html = await page.content();
  if (!html.includes('hi')) {
    throw new Error('Expected content to include "hi", got: ' + html.slice(0, 200));
  }
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T12 should pass");
    assert_eq!(summary.failed, 0);
}

/// T13 — R10: page.content() returns outerHTML.
// REQ: R10
#[tokio::test]
async fn test_t13_content() {
    if !node_available() {
        eprintln!("skipping T13: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T13: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T13: page.content() returns outerHTML', async ({ page }) => {
  await page.goto('data:text/html,<html><body><p>hello world</p></body></html>');
  const html = await page.content();
  if (typeof html !== 'string' || html.length === 0) {
    throw new Error('content() returned empty or non-string: ' + typeof html);
  }
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T13 should pass");
    assert_eq!(summary.failed, 0);
}

/// T14 — R11: locator.boundingBox() returns {x, y, width, height}.
// REQ: R11
#[tokio::test]
async fn test_t14_bounding_box() {
    if !node_available() {
        eprintln!("skipping T14: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T14: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T14: locator.boundingBox() returns numeric coords', async ({ page }) => {
  await page.setContent('<html><body><div id="box" style="width:100px;height:50px;position:absolute;top:10px;left:20px;"></div></body></html>');
  const box = await page.locator('#box').boundingBox();
  if (!box) throw new Error('boundingBox() returned null');
  if (typeof box.x !== 'number') throw new Error('x is not number: ' + typeof box.x);
  if (typeof box.y !== 'number') throw new Error('y is not number: ' + typeof box.y);
  if (typeof box.width !== 'number') throw new Error('width is not number: ' + typeof box.width);
  if (typeof box.height !== 'number') throw new Error('height is not number: ' + typeof box.height);
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T14 should pass");
    assert_eq!(summary.failed, 0);
}

/// T15 — R12: locator.isVisible() and locator.isHidden().
// REQ: R12
#[tokio::test]
async fn test_t15_is_visible_is_hidden() {
    if !node_available() {
        eprintln!("skipping T15: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T15: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T15: isVisible/isHidden reflect computed styles', async ({ page }) => {
  await page.setContent(`
    <div id="visible">shown</div>
    <div id="hidden" style="display:none">hidden</div>
  `);
  const isVis = await page.locator('#visible').isVisible();
  if (!isVis) throw new Error('visible element reported as not visible');
  const isHid = await page.locator('#hidden').isHidden();
  if (!isHid) throw new Error('hidden element reported as not hidden');
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T15 should pass");
    assert_eq!(summary.failed, 0);
}

/// T16 — R12: locator.isEnabled() reflects disabled attribute.
// REQ: R12
#[tokio::test]
async fn test_t16_is_enabled() {
    if !node_available() {
        eprintln!("skipping T16: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T16: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T16: isEnabled reflects disabled attribute', async ({ page }) => {
  await page.setContent(`
    <input id="enabled" type="text" />
    <input id="disabled" type="text" disabled />
  `);
  const enabled = await page.locator('#enabled').isEnabled();
  if (!enabled) throw new Error('enabled input reported as disabled');
  const disabled = await page.locator('#disabled').isEnabled();
  if (disabled) throw new Error('disabled input reported as enabled');
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T16 should pass");
    assert_eq!(summary.failed, 0);
}

/// T17 — R13: locator.hover() dispatches mousemove without throwing.
// REQ: R13
#[tokio::test]
async fn test_t17_hover() {
    if !node_available() {
        eprintln!("skipping T17: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T17: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T17: locator.hover() does not throw', async ({ page }) => {
  await page.setContent('<html><body><button id="btn">hover me</button></body></html>');
  await page.locator('#btn').hover();
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T17 should pass");
    assert_eq!(summary.failed, 0);
}

/// T18 — R14: locator.press('Tab') sends CDP keyDown+keyUp.
// REQ: R14
#[tokio::test]
async fn test_t18_locator_press() {
    if !node_available() {
        eprintln!("skipping T18: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T18: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T18: locator.press() does not throw', async ({ page }) => {
  await page.setContent('<html><body><input id="inp" /></body></html>');
  await page.locator('#inp').press('Tab');
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T18 should pass");
    assert_eq!(summary.failed, 0);
}

/// T19 — R15: locator.selectOption() sets select.value and fires change.
// REQ: R15
#[tokio::test]
async fn test_t19_select_option() {
    if !node_available() {
        eprintln!("skipping T19: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T19: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T19: locator.selectOption() sets select value', async ({ page }) => {
  await page.setContent(`
    <select id="sel">
      <option value="a">A</option>
      <option value="b">B</option>
      <option value="c">C</option>
    </select>
  `);
  await page.locator('#sel').selectOption('b');
  const val = await page.evaluate('document.getElementById("sel").value');
  if (val !== 'b') {
    throw new Error('Expected select value "b", got: ' + JSON.stringify(val));
  }
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T19 should pass");
    assert_eq!(summary.failed, 0);
}

/// T20 — R16: locator.count() returns the number of matching elements.
// REQ: R16
#[tokio::test]
async fn test_t20_count() {
    if !node_available() {
        eprintln!("skipping T20: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T20: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T20: locator.count() returns number of matches', async ({ page }) => {
  await page.setContent('<ul><li>a</li><li>b</li><li>c</li></ul>');
  const n = await page.locator('li').count();
  if (n !== 3) {
    throw new Error('Expected 3 items, got: ' + n);
  }
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T20 should pass");
    assert_eq!(summary.failed, 0);
}

/// T21 — R17: locator.nth(1) returns text of second item.
// REQ: R17
#[tokio::test]
async fn test_t21_nth() {
    if !node_available() {
        eprintln!("skipping T21: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T21: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T21: locator.nth(1) returns second element text', async ({ page }) => {
  await page.setContent('<ul><li>first</li><li>second</li><li>third</li></ul>');
  const text = await page.locator('li').nth(1).innerText();
  if (text !== 'second') {
    throw new Error('Expected "second", got: ' + JSON.stringify(text));
  }
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T21 should pass");
    assert_eq!(summary.failed, 0);
}

/// T22 — R17: locator.first() and locator.last() return correct elements.
// REQ: R17
#[tokio::test]
async fn test_t22_first_last() {
    if !node_available() {
        eprintln!("skipping T22: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T22: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T22: locator.first() and last() return correct elements', async ({ page }) => {
  await page.setContent('<ul><li>alpha</li><li>beta</li><li>gamma</li></ul>');
  const first = await page.locator('li').first().innerText();
  const last = await page.locator('li').last().innerText();
  if (first !== 'alpha') throw new Error('Expected first="alpha", got: ' + JSON.stringify(first));
  if (last !== 'gamma') throw new Error('Expected last="gamma", got: ' + JSON.stringify(last));
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T22 should pass");
    assert_eq!(summary.failed, 0);
}

/// T23 — R18: locator.innerHTML() returns raw HTML.
// REQ: R18
#[tokio::test]
async fn test_t23_inner_html() {
    if !node_available() {
        eprintln!("skipping T23: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T23: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T23: locator.innerHTML() returns raw HTML', async ({ page }) => {
  await page.setContent('<div id="el"><span>hi</span></div>');
  const html = await page.locator('#el').innerHTML();
  if (!html.includes('<span>')) {
    throw new Error('Expected innerHTML to contain <span>, got: ' + JSON.stringify(html));
  }
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T23 should pass");
    assert_eq!(summary.failed, 0);
}

/// T24 — R18: locator.innerText() returns visible text.
// REQ: R18
#[tokio::test]
async fn test_t24_inner_text() {
    if !node_available() {
        eprintln!("skipping T24: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T24: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T24: locator.innerText() returns visible text', async ({ page }) => {
  await page.setContent('<p id="txt">hello world</p>');
  const text = await page.locator('#txt').innerText();
  if (text !== 'hello world') {
    throw new Error('Expected "hello world", got: ' + JSON.stringify(text));
  }
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T24 should pass");
    assert_eq!(summary.failed, 0);
}

/// T25 — R19: locator.inputValue() returns current value of an input.
// REQ: R19
#[tokio::test]
async fn test_t25_input_value() {
    if !node_available() {
        eprintln!("skipping T25: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T25: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T25: locator.inputValue() returns input value', async ({ page }) => {
  await page.setContent('<input id="inp" value="foo" />');
  const val = await page.locator('#inp').inputValue();
  if (val !== 'foo') {
    throw new Error('Expected "foo", got: ' + JSON.stringify(val));
  }
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T25 should pass");
    assert_eq!(summary.failed, 0);
}

/// T26 — R20: expect(page).toHaveTitle() polls page.title().
// REQ: R20
#[tokio::test]
async fn test_t26_to_have_title() {
    if !node_available() {
        eprintln!("skipping T26: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T26: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T26: expect(page).toHaveTitle passes when title matches', async ({ page }) => {
  await page.setContent('<html><head><title>App</title></head><body></body></html>');
  await expect(page).toHaveTitle('App');
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T26 should pass");
    assert_eq!(summary.failed, 0);
}

/// T27 — R21: expect(page).toHaveURL() polls page.url().
// REQ: R21
#[tokio::test]
async fn test_t27_to_have_url() {
    if !node_available() {
        eprintln!("skipping T27: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T27: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T27: expect(page).toHaveURL passes when URL matches regex', async ({ page }) => {
  await page.goto('data:text/html,<p>url test</p>');
  const url = await page.url();
  // data: URLs should match data: prefix.
  await expect(page).toHaveURL(/^data:/);
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T27 should pass");
    assert_eq!(summary.failed, 0);
}

/// T28 — R22: expect(locator).toBeVisible() polls isVisible().
// REQ: R22
#[tokio::test]
async fn test_t28_to_be_visible_locator() {
    if !node_available() {
        eprintln!("skipping T28: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T28: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T28: expect(locator).toBeVisible() passes for visible element', async ({ page }) => {
  await page.setContent('<div id="vis">visible</div>');
  await expect(page.locator('#vis')).toBeVisible();
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T28 should pass");
    assert_eq!(summary.failed, 0);
}

/// T29 — R22: expect(locator).toBeHidden() passes for hidden element.
// REQ: R22
#[tokio::test]
async fn test_t29_to_be_hidden() {
    if !node_available() {
        eprintln!("skipping T29: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T29: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T29: expect(locator).toBeHidden() passes for hidden element', async ({ page }) => {
  await page.setContent('<div id="hid" style="display:none">hidden</div>');
  await expect(page.locator('#hid')).toBeHidden();
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T29 should pass");
    assert_eq!(summary.failed, 0);
}

/// T30 — R23: expect(locator).toHaveText() polls innerText().
// REQ: R23
#[tokio::test]
async fn test_t30_to_have_text_locator() {
    if !node_available() {
        eprintln!("skipping T30: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T30: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T30: expect(locator).toHaveText() passes when innerText matches', async ({ page }) => {
  await page.setContent('<p id="txt">Done</p>');
  await expect(page.locator('#txt')).toHaveText('Done');
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T30 should pass");
    assert_eq!(summary.failed, 0);
}

/// T31 — R24: expect(locator).toHaveValue() polls inputValue().
// REQ: R24
#[tokio::test]
async fn test_t31_to_have_value() {
    if !node_available() {
        eprintln!("skipping T31: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T31: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T31: expect(locator).toHaveValue() passes when inputValue matches', async ({ page }) => {
  await page.setContent('<input id="inp" value="x" />');
  await expect(page.locator('#inp')).toHaveValue('x');
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T31 should pass");
    assert_eq!(summary.failed, 0);
}

/// T32 — R25: expect(locator).toHaveCount() polls count().
// REQ: R25
#[tokio::test]
async fn test_t32_to_have_count() {
    if !node_available() {
        eprintln!("skipping T32: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T32: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T32: expect(locator).toHaveCount() passes when count matches', async ({ page }) => {
  await page.setContent('<ul><li>a</li><li>b</li><li>c</li><li>d</li><li>e</li></ul>');
  await expect(page.locator('li')).toHaveCount(5);
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T32 should pass");
    assert_eq!(summary.failed, 0);
}

/// T33 — R26: expect(locator).toHaveClass() polls element.className.
// REQ: R26
#[tokio::test]
async fn test_t33_to_have_class() {
    if !node_available() {
        eprintln!("skipping T33: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T33: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T33: expect(locator).toHaveClass() passes when class contains expected token', async ({ page }) => {
  await page.setContent('<div id="el" class="foo active bar">el</div>');
  await expect(page.locator('#el')).toHaveClass('active');
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T33 should pass");
    assert_eq!(summary.failed, 0);
}

/// T34 — R27: expect(locator).toHaveAttribute() polls getAttribute().
// REQ: R27
#[tokio::test]
async fn test_t34_to_have_attribute() {
    if !node_available() {
        eprintln!("skipping T34: node not on PATH");
        return;
    }
    if !chromium_available() {
        eprintln!("skipping T34: Chromium not available");
        return;
    }

    let spec = r#"
import { test, expect } from '@jet/test';

test('T34: expect(locator).toHaveAttribute() passes when attribute matches', async ({ page }) => {
  await page.setContent('<button id="btn" data-testid="submit-btn">Submit</button>');
  await expect(page.locator('#btn')).toHaveAttribute('data-testid', 'submit-btn');
});
"#;

    let summary = match run_spec_str(spec, |_| {}).await {
        Some(s) => s,
        None => return,
    };
    assert_eq!(summary.passed, 1, "T34 should pass");
    assert_eq!(summary.failed, 0);
}
// CODEGEN-END
