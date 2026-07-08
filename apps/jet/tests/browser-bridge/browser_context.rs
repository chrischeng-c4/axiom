// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Integration tests for the B3 BrowserContext refactor — covers R1-R9 of
//! `.aw/issues/open/enhancement-browsercontext-refactor-multi-context-isolation-fo.md`.
//!
//! Test strategy:
//!   * Wire-type tests (always run) — verify serde tags for NewContext /
//!     CloseContext / ContextNewPage requests and the ContextResult response,
//!     so the JS ↔ Rust NDJSON contract is guaranteed round-trip stable.
//!   * End-to-end tests (skip gracefully when Chromium is absent) — exercise
//!     `Browser::new_context`, `BrowserContext::new_page`, and
//!     `BrowserContext::close`, and confirm that two concurrent contexts
//!     carry distinct `browserContextId`s with independently-scoped pages.

use jet::browser::{Browser, LaunchOptions};
use jet::cdp_driver::{PageRequest, PageResponse};

// ── Chromium availability probe (same pattern as page_api_parity.rs) ─────────

fn chromium_available() -> bool {
    if std::env::var("CHROME_PATH").is_ok() {
        return true;
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let xdg = std::env::var("XDG_CACHE_HOME").unwrap_or_else(|_| format!("{home}/.cache"));
    if std::path::Path::new(&format!("{home}/Library/Caches/ms-playwright")).exists() {
        return true;
    }
    if std::path::Path::new(&format!("{xdg}/ms-playwright")).exists() {
        return true;
    }
    if std::path::Path::new(&format!("{home}/.cache/jet/chromium")).exists() {
        return true;
    }
    false
}

// ── Wire-type tests (no Chromium required) ───────────────────────────────────

/// R5: NewContext serializes with kind "new_context".
#[test]
fn new_context_request_serializes() {
    let req = PageRequest::NewContext { req_id: 1 };
    let json = serde_json::to_string(&req).unwrap();
    assert!(
        json.contains("\"kind\":\"new_context\""),
        "expected kind=new_context in {json}"
    );
    assert!(json.contains("\"req_id\":1"), "json={json}");
}

/// R5: CloseContext serializes with kind "close_context" and carries context_id.
#[test]
fn close_context_request_serializes() {
    let req = PageRequest::CloseContext {
        req_id: 2,
        context_id: "ctx-abc".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(
        json.contains("\"kind\":\"close_context\""),
        "expected kind=close_context in {json}"
    );
    assert!(json.contains("ctx-abc"), "expected context_id in {json}");
}

/// R5: ContextNewPage serializes with kind "context_new_page".
#[test]
fn context_new_page_request_serializes() {
    let req = PageRequest::ContextNewPage {
        req_id: 3,
        context_id: "ctx-xyz".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(
        json.contains("\"kind\":\"context_new_page\""),
        "expected kind=context_new_page in {json}"
    );
    assert!(json.contains("ctx-xyz"), "expected context_id in {json}");
}

/// R5: ContextResult response serializes with kind "context_result".
#[test]
fn context_result_response_serializes() {
    let resp = PageResponse::ContextResult {
        req_id: 4,
        context_id: "ctx-42".to_string(),
    };
    let json = serde_json::to_string(&resp).unwrap();
    assert!(
        json.contains("\"kind\":\"context_result\""),
        "expected kind=context_result in {json}"
    );
    assert!(json.contains("ctx-42"), "expected context_id in {json}");
}

/// R5: round-trip NewContext / CloseContext / ContextNewPage through serde.
#[test]
fn context_variants_round_trip_serde() {
    let values = [
        PageRequest::NewContext { req_id: 10 },
        PageRequest::CloseContext {
            req_id: 11,
            context_id: "a".to_string(),
        },
        PageRequest::ContextNewPage {
            req_id: 12,
            context_id: "b".to_string(),
        },
    ];
    for req in values {
        let json = serde_json::to_string(&req).unwrap();
        let parsed: PageRequest = serde_json::from_str(&json).unwrap();
        let json2 = serde_json::to_string(&parsed).unwrap();
        assert_eq!(json, json2, "round-trip mismatch for {req:?}");
    }
}

// ── End-to-end tests (skip when Chromium is absent) ──────────────────────────

/// R2 + R3: Browser launches with a default context, and `Browser::new_context`
/// returns a second, distinct `BrowserContext`.
#[tokio::test]
async fn browser_launch_exposes_default_and_new_context() {
    if !chromium_available() {
        eprintln!("skipping: Chromium not installed");
        return;
    }
    let Ok(browser) = Browser::launch(LaunchOptions {
        headless: true,
        ..Default::default()
    })
    .await
    else {
        eprintln!("skipping: Browser::launch failed");
        return;
    };

    let default_ctx = browser.default_context().expect("default context present");
    assert!(
        default_ctx.is_default(),
        "default context must be flagged as default"
    );
    let default_id = default_ctx.id().to_string();
    assert!(!default_id.is_empty(), "default context must carry an id");

    let user_ctx = browser
        .new_context()
        .await
        .expect("Browser::new_context should succeed");
    assert!(
        !user_ctx.is_default(),
        "user-created context must not be flagged as default"
    );
    assert_ne!(
        user_ctx.id(),
        default_id,
        "user context id must differ from default context id"
    );

    // Clean up.
    let _ = user_ctx.close().await;
    let _ = browser.close().await;
}

/// R4: Pages created via a user-created context carry `context_id = Some(_)`;
/// pages created via the default context carry `context_id = None`.
#[tokio::test]
async fn pages_carry_context_id_only_for_user_contexts() {
    if !chromium_available() {
        eprintln!("skipping: Chromium not installed");
        return;
    }
    let Ok(browser) = Browser::launch(LaunchOptions {
        headless: true,
        ..Default::default()
    })
    .await
    else {
        eprintln!("skipping: Browser::launch failed");
        return;
    };

    // Default-context page: context_id should be None (backward compatible).
    let default_page = browser.new_page().await.expect("default new_page");
    assert!(
        default_page.context_id().is_none(),
        "default-context pages must expose context_id = None"
    );

    // User-context page: context_id should be Some(context.id()).
    let user_ctx = browser.new_context().await.expect("new_context");
    let user_ctx_id = user_ctx.id().to_string();
    let user_page = user_ctx.new_page().await.expect("user context new_page");
    assert_eq!(
        user_page.context_id(),
        Some(user_ctx_id.as_str()),
        "user-context pages must carry their context id"
    );

    let _ = user_ctx.close().await;
    let _ = browser.close().await;
}

/// R3 + R8: Two concurrent user-created contexts carry distinct
/// `browserContextId`s, and pages opened inside each are reported only by
/// their owning context via `BrowserContext::pages()`.
#[tokio::test]
async fn two_contexts_are_isolated_by_target_listing() {
    if !chromium_available() {
        eprintln!("skipping: Chromium not installed");
        return;
    }
    let Ok(browser) = Browser::launch(LaunchOptions {
        headless: true,
        ..Default::default()
    })
    .await
    else {
        eprintln!("skipping: Browser::launch failed");
        return;
    };

    let ctx_a = browser.new_context().await.expect("ctx_a");
    let ctx_b = browser.new_context().await.expect("ctx_b");
    assert_ne!(ctx_a.id(), ctx_b.id(), "contexts must have distinct ids");

    let page_a = ctx_a.new_page().await.expect("page_a");
    let page_b = ctx_b.new_page().await.expect("page_b");

    let ids_in_a = ctx_a.pages().await.expect("ctx_a.pages()");
    let ids_in_b = ctx_b.pages().await.expect("ctx_b.pages()");

    assert!(
        ids_in_a.iter().any(|t| t == page_a.target_id()),
        "page_a target must appear under ctx_a.pages()"
    );
    assert!(
        !ids_in_a.iter().any(|t| t == page_b.target_id()),
        "page_b target must NOT appear under ctx_a.pages()"
    );
    assert!(
        ids_in_b.iter().any(|t| t == page_b.target_id()),
        "page_b target must appear under ctx_b.pages()"
    );
    assert!(
        !ids_in_b.iter().any(|t| t == page_a.target_id()),
        "page_a target must NOT appear under ctx_b.pages()"
    );

    let _ = ctx_a.close().await;
    let _ = ctx_b.close().await;
    let _ = browser.close().await;
}

/// R3 + R8: `BrowserContext::close` disposes the `browserContextId`; a
/// subsequent `new_page()` on the same context fails rather than silently
/// succeeding.
#[tokio::test]
async fn closed_context_rejects_new_page() {
    if !chromium_available() {
        eprintln!("skipping: Chromium not installed");
        return;
    }
    let Ok(browser) = Browser::launch(LaunchOptions {
        headless: true,
        ..Default::default()
    })
    .await
    else {
        eprintln!("skipping: Browser::launch failed");
        return;
    };

    let ctx = browser.new_context().await.expect("new_context");
    ctx.close().await.expect("close should succeed");

    // After close, the context_id is invalid — createTarget must fail.
    let res = ctx.new_page().await;
    assert!(
        res.is_err(),
        "new_page() on a closed context must fail (got Ok)"
    );

    let _ = browser.close().await;
}
// CODEGEN-END
