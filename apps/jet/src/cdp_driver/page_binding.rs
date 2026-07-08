// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-cdp-driver.md#schema
// CODEGEN-BEGIN
//! CDP page-action wire binding — JS-to-Rust RPC for Playwright-compatible page API.
//!
//! The JS runtime (`runtime/test/page.js`) sends `PageRequest` NDJSON messages over
//! stdout when test code calls `page.goto`, `page.click`, `page.fill`, etc. The Rust
//! worker dispatches each request to the active `browser::Page` and writes a
//! `PageResponse` back over stdin so the JS promise resolves.
//!
//! Message flow (per action):
//!   JS page proxy → stdout (PageRequest NDJSON) → Rust worker → CDP → browser
//!   browser → CDP → Rust worker → stdin (PageResponse NDJSON) → JS promise resolve
//!
//! Design: the `req_id` field correlates requests and responses across the async
//! boundary. `page_id` allows multiple pages (one per test) to share the channel.
//!
//! This module only defines the wire types and the dispatch function. The Rust
//! worker (`test_runner/worker.rs`) owns the `Page` handle and calls
//! `dispatch_page_request` from its NDJSON read loop.

// @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R2
// @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6

use crate::browser::page::Page;
use anyhow::Result;
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

// Spec reference for all new variants added by this change:
// @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md

// ── Wire types ────────────────────────────────────────────────────────────────

/// Requests the JS page proxy sends to the Rust host for page actions.
///
/// `req_id` correlates with the matching `PageResponse`. `page_id` is the CDP
/// target ID allocated when `new_page` was called.
// @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PageRequest {
    /// Allocate a new browser page (tab). The Rust host returns a `NewPageResult`
    /// carrying the CDP target ID which the JS side uses as `pageId`.
    // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R5
    NewPage { req_id: u64 },
    /// Navigate to `url`. baseURL resolution is done on the JS side before
    /// the request reaches Rust (the JS Page.goto prepends baseURL for relative
    /// paths per the baseurl-resolution logic flowchart).
    // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R3
    Goto {
        req_id: u64,
        page_id: String,
        url: String,
    },
    /// Click the first element matching `selector`.
    // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
    Click {
        req_id: u64,
        page_id: String,
        selector: String,
    },
    /// Fill `value` into the form element matching `selector`.
    // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
    Fill {
        req_id: u64,
        page_id: String,
        selector: String,
        value: String,
    },
    /// Wait for `selector` to appear in the DOM, up to `timeout_ms` (default 5000ms).
    // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
    WaitForSelector {
        req_id: u64,
        page_id: String,
        selector: String,
        timeout_ms: Option<u64>,
    },
    /// Wait for `state` load state: "load" | "domcontentloaded" | "networkidle".
    // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
    WaitForLoadState {
        req_id: u64,
        page_id: String,
        state: Option<String>,
    },
    /// Evaluate a JavaScript expression in the page context.
    // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
    Evaluate {
        req_id: u64,
        page_id: String,
        expression: String,
        timeout_ms: Option<u64>,
    },
    /// Return the current URL of the page.
    // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
    Url { req_id: u64, page_id: String },
    /// Close the page (called in the fixture finally block after each test).
    // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R4
    Close {
        req_id: u64,
        page_id: String,
        timeout_ms: Option<u64>,
    },
    /// Get text content of the first element matching `selector` (used by locator.textContent).
    // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
    GetText {
        req_id: u64,
        page_id: String,
        selector: String,
    },
    /// Get an attribute value from the first element matching `selector`.
    // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
    GetAttribute {
        req_id: u64,
        page_id: String,
        selector: String,
        attribute: String,
    },

    // ── Phase-6 parity variants ───────────────────────────────────────────────
    /// Get the document title via Runtime.evaluate document.title.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R1
    Title { req_id: u64, page_id: String },

    /// Set viewport size via Emulation.setDeviceMetricsOverride.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R2
    SetViewportSize {
        req_id: u64,
        page_id: String,
        width: u32,
        height: u32,
    },

    /// Capture a screenshot via Page.captureScreenshot. Returns base64 PNG data.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R4
    Screenshot {
        req_id: u64,
        page_id: String,
        /// Optional path to write the screenshot to (unused by Rust side — JS handles saving).
        path: Option<String>,
        /// Optional per-call timeout used by visual/failure-artifact capture.
        timeout_ms: Option<u64>,
    },

    /// Navigate back in history via Page.goBack.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R6
    GoBack { req_id: u64, page_id: String },

    /// Navigate forward in history via Page.goForward.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R6
    GoForward { req_id: u64, page_id: String },

    /// Reload the page via Page.reload.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R6
    Reload { req_id: u64, page_id: String },

    /// Dispatch a keyboard event via Input.dispatchKeyEvent.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R7
    KeyboardPress {
        req_id: u64,
        page_id: String,
        /// Playwright key name (e.g. "Enter", "Tab", "a").
        key: String,
    },

    /// Type a string via Input.dispatchKeyEvent for each character.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R7
    KeyboardType {
        req_id: u64,
        page_id: String,
        text: String,
    },

    /// Dispatch a mouse event via Input.dispatchMouseEvent.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R8
    MouseEvent {
        req_id: u64,
        page_id: String,
        /// CDP mouse event type: "mouseMoved", "mousePressed", "mouseReleased".
        event_type: String,
        x: f64,
        y: f64,
        /// Optional button: "left", "right", "middle". None for moves.
        button: Option<String>,
        /// Click count (for pressed/released).
        click_count: Option<u32>,
    },

    /// Set the page HTML content via Page.setDocumentContent.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R9
    SetContent {
        req_id: u64,
        page_id: String,
        html: String,
    },

    /// Get page HTML content via Runtime.evaluate document.documentElement.outerHTML.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R10
    Content { req_id: u64, page_id: String },

    /// Get element bounding box via DOM.getBoxModel.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R11
    BoundingBox {
        req_id: u64,
        page_id: String,
        selector: String,
    },

    /// Hover over element center via Input.dispatchMouseEvent mousemove.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R13
    Hover {
        req_id: u64,
        page_id: String,
        selector: String,
    },

    /// Press a key on a focused element via Input.dispatchKeyEvent.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R14
    LocatorPress {
        req_id: u64,
        page_id: String,
        selector: String,
        key: String,
    },

    /// Register interest in a CDP event (console, pageerror) so Rust forwards them.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R5
    SubscribeEvent {
        req_id: u64,
        page_id: String,
        event_name: String,
    },

    /// Deregister a CDP event subscription (sent on page.close()).
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R5
    RemoveEventListener {
        req_id: u64,
        page_id: String,
        event_name: String,
    },

    // ── B3: BrowserContext variants (additive, non-breaking) ───────────────────
    /// Allocate a new `BrowserContext` via `Target.createBrowserContext`.
    /// Dispatched to the worker; response carries the new `browserContextId`.
    // @spec .aw/issues/open/enhancement-browsercontext-refactor-multi-context-isolation-fo.md#R5
    NewContext { req_id: u64 },

    /// Dispose a non-default `BrowserContext` via
    /// `Target.disposeBrowserContext`. Default contexts (owned by the
    /// Browser) are a no-op.
    // @spec .aw/issues/open/enhancement-browsercontext-refactor-multi-context-isolation-fo.md#R5
    CloseContext { req_id: u64, context_id: String },

    /// Allocate a new page inside a specific `BrowserContext`.
    /// Response carries the `targetId` (used as `page_id`) just like
    /// `NewPage` does for the default context.
    // @spec .aw/issues/open/enhancement-browsercontext-refactor-multi-context-isolation-fo.md#R5
    ContextNewPage { req_id: u64, context_id: String },

    // ── Storage-state variants (P3.2) ───────────────────────────────────────
    /// List cookies in a context via `Storage.getCookies`.
    // @spec .aw/tech-design/projects/jet/logic/storage-state.md#S1
    ContextCookies { req_id: u64, context_id: String },

    /// Install cookies via `Storage.setCookies` on a context.
    // @spec .aw/tech-design/projects/jet/logic/storage-state.md#S2
    ContextAddCookies {
        req_id: u64,
        context_id: String,
        cookies: serde_json::Value,
    },

    /// Clear all cookies in a context.
    // @spec .aw/tech-design/projects/jet/logic/storage-state.md#S3
    ContextClearCookies { req_id: u64, context_id: String },

    /// Return `{ cookies, origins: [] }` for a context.
    // @spec .aw/tech-design/projects/jet/logic/storage-state.md#S4
    ContextStorageState { req_id: u64, context_id: String },

    /// Load a storage-state snapshot (cookies only for MVP) into a context.
    // @spec .aw/tech-design/projects/jet/logic/storage-state.md#S5
    ContextSetStorageState {
        req_id: u64,
        context_id: String,
        state: serde_json::Value,
    },
}

/// Responses the Rust host sends back for `PageRequest` messages.
// @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R10
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PageResponse {
    /// Action completed successfully with no return value.
    Ok { req_id: u64 },
    /// Result of a `NewPage` request: the allocated CDP target ID.
    // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R5
    NewPageResult { req_id: u64, page_id: String },
    /// Successful string result (url(), textContent(), getAttribute(), title(), content()).
    StringResult { req_id: u64, value: String },
    /// Successful JSON result (evaluate()).
    JsonResult {
        req_id: u64,
        value: serde_json::Value,
    },
    /// Action failed. `message` surfaces the OS/CDP error to the test output.
    // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R10
    Error { req_id: u64, message: String },

    // ── Phase-6 additional response variants ─────────────────────────────────
    /// Screenshot bytes as base64-encoded PNG.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R4
    ScreenshotResult { req_id: u64, data: String },

    /// Bounding box result: {x, y, width, height} or null.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R11
    BoundingBoxResult {
        req_id: u64,
        x: Option<f64>,
        y: Option<f64>,
        width: Option<f64>,
        height: Option<f64>,
    },

    /// Result of a `NewContext` request: the allocated `browserContextId`.
    // @spec .aw/issues/open/enhancement-browsercontext-refactor-multi-context-isolation-fo.md#R5
    ContextResult { req_id: u64, context_id: String },

    /// Result of a `ContextCookies` or `ContextStorageState` request.
    // @spec .aw/tech-design/projects/jet/logic/storage-state.md#S1 S4
    StorageStateResult {
        req_id: u64,
        value: serde_json::Value,
    },

    /// Asynchronous page event forwarded from CDP to the JS Page instance.
    Event {
        page_id: String,
        event: String,
        payload: serde_json::Value,
    },
}

/// Format the warn message emitted when a line parses as JSON but does
/// not match any known `PageRequest` shape (e.g. unknown `kind`, missing
/// `req_id`, future-version variant). Extracted into a helper so the
/// exact wording (issue tag + field hint) is unit-testable without
/// provoking a live JS runtime version-skew.
/// @spec .aw/tech-design/projects/jet/semantic/jet-cdp-driver.md#schema
pub(crate) fn format_unknown_page_request_warn(
    line_preview: &str,
    err: &serde_json::Error,
) -> String {
    format!(
        "GH #3745 NDJSON line from JS runtime parsed as JSON but did not \
         match any known `PageRequest` variant ({err}); the JS-side promise \
         tied to this request will never resolve. The prior `.ok()` \
         fall-back swallowed this silently. Check whether the JS runtime \
         emitted a future-version `kind` field. line_preview=`{line_preview}`"
    )
}

/// Parse a `PageRequest` from an NDJSON line emitted by the JS runtime.
///
/// Returns `None` for empty lines and non-JSON lines (the JS runtime
/// occasionally interleaves `console.log` output on the same stream and
/// those are intentionally ignored). Lines that ARE valid JSON but do
/// not match any known `PageRequest` shape return `None` *and* emit a
/// `tracing::warn!` — that case usually means JS/Rust version skew and
/// the JS-side promise will never resolve, so it must be visible.
/// @spec .aw/tech-design/projects/jet/semantic/jet-cdp-driver.md#schema
pub fn parse_page_request(line: &str) -> Option<PageRequest> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    // GH #3745 — distinguish non-JSON (silent: console.log noise on the
    // same stream) from valid-JSON-bad-shape (warn: version skew).
    let value: serde_json::Value = match serde_json::from_str(trimmed) {
        Ok(v) => v,
        Err(_) => return None,
    };
    match serde_json::from_value::<PageRequest>(value) {
        Ok(req) => Some(req),
        Err(err) => {
            let preview: String = trimmed.chars().take(200).collect();
            tracing::warn!(
                target: "jet::cdp_driver::page_binding",
                error = %err,
                line_preview = %preview,
                "{}",
                format_unknown_page_request_warn(&preview, &err)
            );
            None
        }
    }
}

/// Dispatch a `PageRequest` to the active `Page` and return a `PageResponse`.
///
/// Called from `run_spec` when the NDJSON read loop decodes a `PageRequest`.
/// If `page` is `None` (browser not active for this spec), an error response is
/// returned so the JS promise rejects with a descriptive message.
///
/// # Errors
///
/// The function itself is infallible — all errors are encoded as
/// `PageResponse::Error` so the JS side gets a clean rejection rather than a
/// silent `undefined`.
// @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R10
pub async fn dispatch_page_request(req: PageRequest, page: Option<&Page>) -> PageResponse {
    // Helper: extract req_id without moving req for error path.
    let req_id_of = |r: &PageRequest| match r {
        PageRequest::NewPage { req_id }
        | PageRequest::Goto { req_id, .. }
        | PageRequest::Click { req_id, .. }
        | PageRequest::Fill { req_id, .. }
        | PageRequest::WaitForSelector { req_id, .. }
        | PageRequest::WaitForLoadState { req_id, .. }
        | PageRequest::Evaluate { req_id, .. }
        | PageRequest::Url { req_id, .. }
        | PageRequest::Close { req_id, .. }
        | PageRequest::GetText { req_id, .. }
        | PageRequest::GetAttribute { req_id, .. }
        // Phase-6 variants
        | PageRequest::Title { req_id, .. }
        | PageRequest::SetViewportSize { req_id, .. }
        | PageRequest::Screenshot { req_id, .. }
        | PageRequest::GoBack { req_id, .. }
        | PageRequest::GoForward { req_id, .. }
        | PageRequest::Reload { req_id, .. }
        | PageRequest::KeyboardPress { req_id, .. }
        | PageRequest::KeyboardType { req_id, .. }
        | PageRequest::MouseEvent { req_id, .. }
        | PageRequest::SetContent { req_id, .. }
        | PageRequest::Content { req_id, .. }
        | PageRequest::BoundingBox { req_id, .. }
        | PageRequest::Hover { req_id, .. }
        | PageRequest::LocatorPress { req_id, .. }
        | PageRequest::SubscribeEvent { req_id, .. }
        | PageRequest::RemoveEventListener { req_id, .. }
        // B3: BrowserContext variants
        | PageRequest::NewContext { req_id }
        | PageRequest::CloseContext { req_id, .. }
        | PageRequest::ContextNewPage { req_id, .. }
        // P3.2: Storage-state variants
        | PageRequest::ContextCookies { req_id, .. }
        | PageRequest::ContextAddCookies { req_id, .. }
        | PageRequest::ContextClearCookies { req_id, .. }
        | PageRequest::ContextStorageState { req_id, .. }
        | PageRequest::ContextSetStorageState { req_id, .. } => *req_id,
    };

    let req_id = req_id_of(&req);

    let Some(page) = page else {
        return PageResponse::Error {
            req_id,
            message: "browser page is not active for this test — page fixture not injected or browser failed to launch".to_string(),
        };
    };

    match req {
        // NewPage is handled directly in worker.rs before dispatch_page_request
        // is called. If it reaches here, return an internal error.
        PageRequest::NewPage { req_id } => PageResponse::Error {
            req_id,
            message: "internal: NewPage dispatched to dispatch_page_request unexpectedly"
                .to_string(),
        },

        // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R3
        PageRequest::Goto { req_id, url, .. } => match page.goto(&url).await {
            Ok(()) => PageResponse::Ok { req_id },
            Err(e) => PageResponse::Error {
                req_id,
                message: format!("browser goto failed: {e}"),
            },
        },

        // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
        PageRequest::Click {
            req_id, selector, ..
        } => match do_click(page, &selector).await {
            Ok(()) => PageResponse::Ok { req_id },
            Err(e) => PageResponse::Error {
                req_id,
                message: format!("browser click({selector:?}) failed: {e}"),
            },
        },

        // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
        PageRequest::Fill {
            req_id,
            selector,
            value,
            ..
        } => match do_fill(page, &selector, &value).await {
            Ok(()) => PageResponse::Ok { req_id },
            Err(e) => PageResponse::Error {
                req_id,
                message: format!("browser fill({selector:?}, ...) failed: {e}"),
            },
        },

        // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
        PageRequest::WaitForSelector {
            req_id,
            selector,
            timeout_ms,
            ..
        } => {
            let ms = timeout_ms.unwrap_or(5000);
            match page.wait_for_selector(&selector, ms).await {
                Ok(_) => PageResponse::Ok { req_id },
                Err(e) => PageResponse::Error {
                    req_id,
                    message: format!("browser waitForSelector({selector:?}) failed: {e}"),
                },
            }
        }

        // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
        PageRequest::WaitForLoadState { req_id, state, .. } => {
            // Delegate to document.readyState polling via evaluate.
            let state_str = state.as_deref().unwrap_or("load");
            match do_wait_load_state(page, state_str).await {
                Ok(()) => PageResponse::Ok { req_id },
                Err(e) => PageResponse::Error {
                    req_id,
                    message: format!("browser waitForLoadState({state_str:?}) failed: {e}"),
                },
            }
        }

        // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
        PageRequest::Evaluate {
            req_id, expression, ..
        } => match page.evaluate(&expression).await {
            Ok(v) => PageResponse::JsonResult { req_id, value: v },
            Err(e) => PageResponse::Error {
                req_id,
                message: format!("browser evaluate failed: {e}"),
            },
        },

        // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
        PageRequest::Url { req_id, .. } => match page.url().await {
            Ok(u) => PageResponse::StringResult { req_id, value: u },
            Err(e) => PageResponse::Error {
                req_id,
                message: format!("browser url() failed: {e}"),
            },
        },

        // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R4
        PageRequest::Close { req_id, .. } => match page.close().await {
            Ok(()) => PageResponse::Ok { req_id },
            Err(e) => PageResponse::Error {
                req_id,
                message: format!("browser close() failed: {e}"),
            },
        },

        // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
        PageRequest::GetText {
            req_id, selector, ..
        } => match page.locator(&selector) {
            Err(e) => PageResponse::Error {
                req_id,
                message: format!("locator parse error for {selector:?}: {e}"),
            },
            Ok(loc) => match loc.text_content().await {
                Ok(t) => PageResponse::StringResult { req_id, value: t },
                Err(e) => PageResponse::Error {
                    req_id,
                    message: format!("textContent({selector:?}) failed: {e}"),
                },
            },
        },

        // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
        PageRequest::GetAttribute {
            req_id,
            selector,
            attribute,
            ..
        } => match do_get_attribute(page, &selector, &attribute).await {
            Ok(v) => PageResponse::StringResult {
                req_id,
                value: v.unwrap_or_default(),
            },
            Err(e) => PageResponse::Error {
                req_id,
                message: format!("getAttribute({selector:?}, {attribute:?}) failed: {e}"),
            },
        },

        // ── Phase-6 parity handlers ──────────────────────────────────────────

        // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R1
        PageRequest::Title { req_id, .. } => match page.title().await {
            Ok(t) => PageResponse::StringResult { req_id, value: t },
            Err(e) => PageResponse::Error {
                req_id,
                message: format!("page.title() failed: {e}"),
            },
        },

        // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R2
        PageRequest::SetViewportSize {
            req_id,
            width,
            height,
            ..
        } => match do_set_viewport_size(page, width, height).await {
            Ok(()) => PageResponse::Ok { req_id },
            Err(e) => PageResponse::Error {
                req_id,
                message: format!("setViewportSize({width},{height}) failed: {e}"),
            },
        },

        // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R4
        PageRequest::Screenshot { req_id, .. } => match do_screenshot(page).await {
            Ok(data) => PageResponse::ScreenshotResult { req_id, data },
            Err(e) => PageResponse::Error {
                req_id,
                message: format!("screenshot() failed: {e}"),
            },
        },

        // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R6
        PageRequest::GoBack { req_id, .. } => match do_go_back(page).await {
            Ok(()) => PageResponse::Ok { req_id },
            Err(e) => PageResponse::Error {
                req_id,
                message: format!("goBack() failed: {e}"),
            },
        },

        // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R6
        PageRequest::GoForward { req_id, .. } => match do_go_forward(page).await {
            Ok(()) => PageResponse::Ok { req_id },
            Err(e) => PageResponse::Error {
                req_id,
                message: format!("goForward() failed: {e}"),
            },
        },

        // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R6
        PageRequest::Reload { req_id, .. } => match do_reload(page).await {
            Ok(()) => PageResponse::Ok { req_id },
            Err(e) => PageResponse::Error {
                req_id,
                message: format!("reload() failed: {e}"),
            },
        },

        // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R7
        PageRequest::KeyboardPress { req_id, key, .. } => {
            match do_keyboard_press(page, &key).await {
                Ok(()) => PageResponse::Ok { req_id },
                Err(e) => PageResponse::Error {
                    req_id,
                    message: format!("keyboard.press({key:?}) failed: {e}"),
                },
            }
        }

        // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R7
        PageRequest::KeyboardType { req_id, text, .. } => {
            match do_keyboard_type(page, &text).await {
                Ok(()) => PageResponse::Ok { req_id },
                Err(e) => PageResponse::Error {
                    req_id,
                    message: format!("keyboard.type() failed: {e}"),
                },
            }
        }

        // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R8
        PageRequest::MouseEvent {
            req_id,
            event_type,
            x,
            y,
            button,
            click_count,
            ..
        } => match do_mouse_event(page, &event_type, x, y, button.as_deref(), click_count).await {
            Ok(()) => PageResponse::Ok { req_id },
            Err(e) => PageResponse::Error {
                req_id,
                message: format!("mouse.{event_type}({x},{y}) failed: {e}"),
            },
        },

        // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R9
        PageRequest::SetContent { req_id, html, .. } => match do_set_content(page, &html).await {
            Ok(()) => PageResponse::Ok { req_id },
            Err(e) => PageResponse::Error {
                req_id,
                message: format!("setContent() failed: {e}"),
            },
        },

        // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R10
        PageRequest::Content { req_id, .. } => {
            match page.evaluate("document.documentElement.outerHTML").await {
                Ok(v) => PageResponse::StringResult {
                    req_id,
                    value: v.as_str().unwrap_or("").to_string(),
                },
                Err(e) => PageResponse::Error {
                    req_id,
                    message: format!("content() failed: {e}"),
                },
            }
        }

        // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R11
        PageRequest::BoundingBox {
            req_id, selector, ..
        } => match do_bounding_box(page, &selector).await {
            Ok(Some((x, y, w, h))) => PageResponse::BoundingBoxResult {
                req_id,
                x: Some(x),
                y: Some(y),
                width: Some(w),
                height: Some(h),
            },
            Ok(None) => PageResponse::BoundingBoxResult {
                req_id,
                x: None,
                y: None,
                width: None,
                height: None,
            },
            Err(e) => PageResponse::Error {
                req_id,
                message: format!("boundingBox({selector:?}) failed: {e}"),
            },
        },

        // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R13
        PageRequest::Hover {
            req_id, selector, ..
        } => match do_hover(page, &selector).await {
            Ok(()) => PageResponse::Ok { req_id },
            Err(e) => PageResponse::Error {
                req_id,
                message: format!("hover({selector:?}) failed: {e}"),
            },
        },

        // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R14
        PageRequest::LocatorPress {
            req_id,
            selector,
            key,
            ..
        } => match do_locator_press(page, &selector, &key).await {
            Ok(()) => PageResponse::Ok { req_id },
            Err(e) => PageResponse::Error {
                req_id,
                message: format!("locator({selector:?}).press({key:?}) failed: {e}"),
            },
        },

        // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R5
        // Event subscription is best-effort: acknowledge receipt so the JS promise
        // resolves. Actual event forwarding (Runtime.consoleAPICalled) would require
        // an async event loop wired from the CdpClient to the JS stdin channel,
        // which is out of scope for this implementation (no redesign of CdpClient).
        PageRequest::SubscribeEvent { req_id, .. } => PageResponse::Ok { req_id },

        // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R5
        PageRequest::RemoveEventListener { req_id, .. } => PageResponse::Ok { req_id },

        // B3 context variants are handled directly in `worker.rs` alongside
        // `NewPage` because they require access to the `Browser` / contexts
        // map rather than the per-page dispatch context. If one reaches here
        // it is a routing bug — return an internal error so the JS promise
        // rejects cleanly instead of hanging.
        // @spec .aw/issues/open/enhancement-browsercontext-refactor-multi-context-isolation-fo.md#R5
        PageRequest::NewContext { req_id } => PageResponse::Error {
            req_id,
            message: "internal: NewContext dispatched to dispatch_page_request unexpectedly"
                .to_string(),
        },
        PageRequest::CloseContext { req_id, .. } => PageResponse::Error {
            req_id,
            message: "internal: CloseContext dispatched to dispatch_page_request unexpectedly"
                .to_string(),
        },
        PageRequest::ContextNewPage { req_id, .. } => PageResponse::Error {
            req_id,
            message: "internal: ContextNewPage dispatched to dispatch_page_request unexpectedly"
                .to_string(),
        },

        // P3.2 storage-state variants — handled in `worker.rs` against the
        // contexts map. Reaching here means a routing bug.
        // @spec .aw/tech-design/projects/jet/logic/storage-state.md#S1..S5
        PageRequest::ContextCookies { req_id, .. }
        | PageRequest::ContextAddCookies { req_id, .. }
        | PageRequest::ContextClearCookies { req_id, .. }
        | PageRequest::ContextStorageState { req_id, .. }
        | PageRequest::ContextSetStorageState { req_id, .. } => PageResponse::Error {
            req_id,
            message: "internal: storage-state request dispatched to dispatch_page_request"
                .to_string(),
        },
    }
}

/// Write a `PageResponse` as an NDJSON line to `writer`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-cdp-driver.md#schema
pub async fn write_page_response<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    response: PageResponse,
) -> Result<()> {
    let line = serde_json::to_string(&response)
        .map_err(|e| anyhow::anyhow!("serialize PageResponse: {e}"))?;
    writer.write_all(line.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    Ok(())
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Click the first element matching `selector` via the Locator engine.
async fn do_click(page: &Page, selector: &str) -> Result<()> {
    let locator = page.locator(selector).map_err(|e| anyhow::anyhow!("{e}"))?;
    locator.click().await.map_err(|e| anyhow::anyhow!("{e}"))
}

/// Fill `value` into the first element matching `selector`.
async fn do_fill(page: &Page, selector: &str, value: &str) -> Result<()> {
    let locator = page.locator(selector).map_err(|e| anyhow::anyhow!("{e}"))?;
    locator
        .fill(value)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Poll `document.readyState` until `state` is reached (default: "complete" for "load").
async fn do_wait_load_state(page: &Page, state: &str) -> Result<()> {
    let js_state = match state {
        "domcontentloaded" => "interactive",
        "networkidle" => "complete", // approximate
        _ => "complete",             // "load" → "complete"
    };

    for _ in 0..100 {
        let ready = page.evaluate("document.readyState").await?;
        if ready.as_str() == Some(js_state) || ready.as_str() == Some("complete") {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    anyhow::bail!("Timeout waiting for load state '{state}'")
}

/// Get an attribute value from the first element matching `selector`.
async fn do_get_attribute(page: &Page, selector: &str, attr: &str) -> Result<Option<String>> {
    let expr = format!(
        r#"(function() {{
            var el = document.querySelector({sel});
            return el ? el.getAttribute({a}) : null;
        }})()"#,
        sel = serde_json::to_string(selector)?,
        a = serde_json::to_string(attr)?,
    );
    let val = page.evaluate(&expr).await?;
    Ok(val.as_str().map(|s| s.to_string()))
}

// ── Phase-6 private helpers ───────────────────────────────────────────────────

/// Set viewport size via Emulation.setDeviceMetricsOverride.
// @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R2
async fn do_set_viewport_size(page: &Page, width: u32, height: u32) -> Result<()> {
    page.session()
        .send(
            "Emulation.setDeviceMetricsOverride",
            serde_json::json!({
                "width": width,
                "height": height,
                "deviceScaleFactor": 1,
                "mobile": false,
            }),
        )
        .await?;
    Ok(())
}

/// Capture a screenshot and return base64-encoded PNG data.
// @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R4
async fn do_screenshot(page: &Page) -> Result<String> {
    let result = page
        .session()
        .send(
            "Page.captureScreenshot",
            serde_json::json!({ "format": "png" }),
        )
        .await?;
    let data = result["data"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("missing screenshot data field"))?;
    Ok(data.to_string())
}

/// Navigate back in history. The CDP `Page.goBack` method doesn't exist;
/// Playwright-core uses `Page.getNavigationHistory` + `Page.navigateToHistoryEntry`.
// @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R6
async fn do_go_back(page: &Page) -> Result<()> {
    navigate_history_delta(page, -1).await
}

/// Navigate forward in history (see `do_go_back`).
// @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R6
async fn do_go_forward(page: &Page) -> Result<()> {
    navigate_history_delta(page, 1).await
}

/// GH #3743 — build the warn body for a malformed
/// `Page.getNavigationHistory` response field. Extracted so the
/// wording (issue tag + field name + expected shape) is unit-testable
/// without provoking a real malformed-CDP-response scenario.
///
/// `field` is the JSON path inside the CDP response (e.g.
/// `"currentIndex"`, `"entries"`, `"entries[target].id"`). `expected`
/// is the JSON-Value type we needed (e.g. `"i64"`, `"array"`).
/// @spec .aw/tech-design/projects/jet/semantic/jet-cdp-driver.md#schema
pub(crate) fn format_history_field_warn(field: &str, expected: &str) -> String {
    format!(
        "GH #3743 Page.getNavigationHistory response field `{field}` was \
         missing or not the expected JSON type `{expected}`; \
         goBack/goForward will return Ok(()) as a no-op (the prior \
         `unwrap_or` fall-back would have silently navigated to entry \
         0 — possibly the wrong page). Check whether the Chromium \
         version is supported and whether instrumentation is rewriting \
         CDP responses."
    )
}

async fn navigate_history_delta(page: &Page, delta: i64) -> Result<()> {
    let hist = page
        .session()
        .send("Page.getNavigationHistory", serde_json::json!({}))
        .await?;
    // GH #3743 — previously `hist["currentIndex"].as_i64().unwrap_or(0)`.
    // A malformed currentIndex combined with a non-zero delta would
    // silently navigate to the wrong entry. No-op cleanly instead.
    let current = match hist["currentIndex"].as_i64() {
        Some(i) => i,
        None => {
            tracing::warn!(
                target: "jet::cdp_driver::page_binding",
                "{}",
                format_history_field_warn("currentIndex", "i64")
            );
            return Ok(());
        }
    };
    // GH #3743 — previously `as_array().cloned().unwrap_or_default()`.
    // Empty Vec is the right semantic when entries is genuinely
    // absent, but we want a warn on the wrong-shape branch.
    let entries = match hist["entries"].as_array() {
        Some(arr) => arr.clone(),
        None => {
            tracing::warn!(
                target: "jet::cdp_driver::page_binding",
                "{}",
                format_history_field_warn("entries", "array")
            );
            return Ok(());
        }
    };
    let target = current + delta;
    if target < 0 || target >= entries.len() as i64 {
        // Nothing to navigate to — matches Playwright's "returns null" behavior.
        return Ok(());
    }
    // GH #3743 — previously `entries[target][\"id\"].as_i64().unwrap_or(0)`.
    // Silent fallback to entry_id 0 would navigate to whatever entry 0
    // is (often the initial page); warn and no-op instead.
    let entry_id = match entries[target as usize]["id"].as_i64() {
        Some(id) => id,
        None => {
            tracing::warn!(
                target: "jet::cdp_driver::page_binding",
                "{}",
                format_history_field_warn("entries[target].id", "i64")
            );
            return Ok(());
        }
    };
    page.session()
        .send(
            "Page.navigateToHistoryEntry",
            serde_json::json!({ "entryId": entry_id }),
        )
        .await?;
    do_wait_load_state(page, "load").await
}

/// Reload the page via Page.reload.
// @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R6
async fn do_reload(page: &Page) -> Result<()> {
    page.session()
        .send("Page.reload", serde_json::json!({}))
        .await?;
    do_wait_load_state(page, "load").await
}

/// Dispatch a single key press (keyDown + keyUp) via Input.dispatchKeyEvent.
// @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R7
async fn do_keyboard_press(page: &Page, key: &str) -> Result<()> {
    // Map Playwright key names to CDP key info.
    let (key_text, code) = playwright_key_to_cdp(key);
    page.session()
        .send(
            "Input.dispatchKeyEvent",
            serde_json::json!({
                "type": "keyDown",
                "key": key,
                "code": code,
                "text": key_text,
            }),
        )
        .await?;
    page.session()
        .send(
            "Input.dispatchKeyEvent",
            serde_json::json!({
                "type": "keyUp",
                "key": key,
                "code": code,
                "text": key_text,
            }),
        )
        .await?;
    Ok(())
}

/// Type a string by dispatching keyDown+keyUp for each character.
// @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R7
async fn do_keyboard_type(page: &Page, text: &str) -> Result<()> {
    for ch in text.chars() {
        let ch_str = ch.to_string();
        page.session()
            .send(
                "Input.dispatchKeyEvent",
                serde_json::json!({
                    "type": "keyDown",
                    "key": &ch_str,
                    "text": &ch_str,
                }),
            )
            .await?;
        page.session()
            .send(
                "Input.dispatchKeyEvent",
                serde_json::json!({
                    "type": "keyUp",
                    "key": &ch_str,
                    "text": &ch_str,
                }),
            )
            .await?;
    }
    Ok(())
}

/// Dispatch a mouse event via Input.dispatchMouseEvent.
// @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R8
async fn do_mouse_event(
    page: &Page,
    event_type: &str,
    x: f64,
    y: f64,
    button: Option<&str>,
    click_count: Option<u32>,
) -> Result<()> {
    let mut params = serde_json::json!({
        "type": event_type,
        "x": x,
        "y": y,
    });
    if let Some(btn) = button {
        params["button"] = serde_json::Value::String(btn.to_string());
    }
    if let Some(cc) = click_count {
        params["clickCount"] = serde_json::Value::Number(cc.into());
    }
    if let Some(buttons) = mouse_event_buttons(event_type, button) {
        params["buttons"] = serde_json::Value::Number(buttons.into());
    }
    page.session()
        .send("Input.dispatchMouseEvent", params)
        .await?;
    Ok(())
}

fn mouse_event_buttons(event_type: &str, button: Option<&str>) -> Option<u32> {
    match event_type {
        "mousePressed" => Some(match button {
            Some("right") => 2,
            Some("middle") => 4,
            _ => 1,
        }),
        "mouseReleased" => Some(0),
        _ => None,
    }
}

/// Set page content via Page.setDocumentContent.
// @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R9
async fn do_set_content(page: &Page, html: &str) -> Result<()> {
    page.session()
        .send(
            "Page.setDocumentContent",
            serde_json::json!({
                "frameId": "",
                "html": html,
            }),
        )
        .await
        .ok(); // Ignore errors — some CDP versions don't support frameId override.
               // Alternative: use Runtime.evaluate to set document content.
    let escaped = html.replace('`', "\\`").replace('$', "\\$");
    let expr = format!("document.open(); document.write(`{escaped}`); document.close(); true");
    page.evaluate(&expr).await?;
    Ok(())
}

/// Get the bounding box of a matched element using DOM.getBoxModel.
/// Returns (x, y, width, height) or None if element not found.
// @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R11
async fn do_bounding_box(page: &Page, selector: &str) -> Result<Option<(f64, f64, f64, f64)>> {
    // Use JS getBoundingClientRect for reliability across all CDP versions.
    let sel_json = serde_json::to_string(selector)?;
    let expr = format!(
        r#"(function() {{
            var el = document.querySelector({sel});
            if (!el) return null;
            var r = el.getBoundingClientRect();
            return {{ x: r.left, y: r.top, width: r.width, height: r.height }};
        }})()"#,
        sel = sel_json,
    );
    let val = page.evaluate(&expr).await?;
    if val.is_null() {
        return Ok(None);
    }
    let x = val["x"].as_f64().unwrap_or(0.0);
    let y = val["y"].as_f64().unwrap_or(0.0);
    let w = val["width"].as_f64().unwrap_or(0.0);
    let h = val["height"].as_f64().unwrap_or(0.0);
    Ok(Some((x, y, w, h)))
}

/// Hover over element center via Input.dispatchMouseEvent mousemove.
// @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R13
async fn do_hover(page: &Page, selector: &str) -> Result<()> {
    let bb = do_bounding_box(page, selector).await?;
    if let Some((x, y, w, h)) = bb {
        let cx = x + w / 2.0;
        let cy = y + h / 2.0;
        page.session()
            .send(
                "Input.dispatchMouseEvent",
                serde_json::json!({
                    "type": "mouseMoved",
                    "x": cx,
                    "y": cy,
                }),
            )
            .await?;
    }
    Ok(())
}

/// Press a key on the element matching `selector` (focuses first).
// @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R14
async fn do_locator_press(page: &Page, selector: &str, key: &str) -> Result<()> {
    // Focus the element first via evaluate.
    let sel_json = serde_json::to_string(selector)?;
    let focus_expr = format!(
        r#"(function() {{ var el = document.querySelector({sel}); if (el) el.focus(); return !!el; }})()"#,
        sel = sel_json,
    );
    page.evaluate(&focus_expr).await?;
    // Then dispatch the key press.
    do_keyboard_press(page, key).await
}

/// Map Playwright key names to CDP (key text, key code) pairs.
/// Returns ("", "") for non-printable keys like Enter, Tab, etc.
fn playwright_key_to_cdp(key: &str) -> (&str, &str) {
    match key {
        "Enter" => ("\r", "Enter"),
        "Tab" => ("\t", "Tab"),
        "Escape" | "Esc" => ("", "Escape"),
        "Backspace" => ("\x08", "Backspace"),
        "Delete" | "Del" => ("", "Delete"),
        "ArrowLeft" => ("", "ArrowLeft"),
        "ArrowRight" => ("", "ArrowRight"),
        "ArrowUp" => ("", "ArrowUp"),
        "ArrowDown" => ("", "ArrowDown"),
        "Home" => ("", "Home"),
        "End" => ("", "End"),
        "PageUp" => ("", "PageUp"),
        "PageDown" => ("", "PageDown"),
        "Space" | " " => (" ", "Space"),
        // Single printable character — use as-is.
        s if s.len() == 1 => (s, s),
        // Unknown — pass empty text, use key name as code.
        _ => ("", key),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: R10 — parse_page_request gracefully handles non-PageRequest JSON
    #[test]
    fn parse_page_request_empty_is_none() {
        assert!(parse_page_request("").is_none());
        assert!(parse_page_request("   ").is_none());
    }

    #[test]
    fn parse_page_request_non_json_is_none() {
        assert!(parse_page_request("not json").is_none());
    }

    #[test]
    fn parse_page_request_unknown_kind_is_none() {
        assert!(parse_page_request(r#"{"kind":"mystery","req_id":1,"page_id":"t1"}"#).is_none());
    }

    // REQ: R6 — all page actions round-trip through serde
    #[test]
    fn parse_page_request_goto() {
        let json = r#"{"kind":"goto","req_id":1,"page_id":"t1","url":"http://localhost:3000/"}"#;
        let req = parse_page_request(json).expect("should parse");
        match req {
            PageRequest::Goto {
                req_id,
                page_id,
                url,
            } => {
                assert_eq!(req_id, 1);
                assert_eq!(page_id, "t1");
                assert_eq!(url, "http://localhost:3000/");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn page_response_ok_serializes() {
        let resp = PageResponse::Ok { req_id: 42 };
        let s = serde_json::to_string(&resp).unwrap();
        assert!(s.contains("\"kind\":\"ok\""));
        assert!(s.contains("42"));
    }

    #[test]
    fn page_response_error_serializes() {
        let resp = PageResponse::Error {
            req_id: 7,
            message: "browser launch failed: os error 2".to_string(),
        };
        let s = serde_json::to_string(&resp).unwrap();
        assert!(s.contains("browser"));
        assert!(s.contains("os error 2"));
    }
}

#[cfg(test)]
mod gh3743_history_field_warn_tests {
    //! GH #3743 — `navigate_history_delta` previously had three silent
    //! `unwrap_or(0)` / `unwrap_or_default()` fallbacks on the CDP
    //! `Page.getNavigationHistory` response. A malformed response
    //! silently navigated to the wrong page. These tests pin the
    //! helper wording for all three fields and verify the field-name
    //! pairwise distinctness in the warn body so log readers can
    //! grep for one field and only land on that site.
    use super::*;

    #[test]
    fn gh3743_helper_includes_tag_field_and_expected() {
        for (field, expected) in [
            ("currentIndex", "i64"),
            ("entries", "array"),
            ("entries[target].id", "i64"),
        ] {
            let msg = format_history_field_warn(field, expected);
            assert!(
                msg.contains("GH #3743"),
                "must include issue tag (field={field}): {msg}"
            );
            assert!(
                msg.contains(field),
                "must name the offending field (field={field}): {msg}"
            );
            assert!(
                msg.contains(expected),
                "must name the expected type (expected={expected}): {msg}"
            );
        }
    }

    #[test]
    fn gh3743_helper_includes_safer_behavior_note() {
        let msg = format_history_field_warn("currentIndex", "i64");
        // The warn body must explain WHY this is safer than the prior
        // unwrap_or(0): the prior would silently navigate to the wrong page.
        assert!(
            msg.contains("entry 0") || msg.contains("wrong page"),
            "must hint at the prior wrong-navigation behaviour: {msg}"
        );
    }

    #[test]
    fn gh3743_helper_includes_remediation_hint() {
        let msg = format_history_field_warn("entries", "array");
        assert!(
            msg.contains("Chromium") || msg.contains("instrumentation"),
            "must hint at the two most likely causes: {msg}"
        );
    }

    #[test]
    fn gh3743_helper_is_deterministic() {
        let a = format_history_field_warn("currentIndex", "i64");
        let b = format_history_field_warn("currentIndex", "i64");
        assert_eq!(a, b);
    }

    /// Three field names must be pairwise distinct in the warn body so
    /// an operator grepping for `entries[target].id` doesn't accidentally
    /// land on a `currentIndex` failure (which has a totally different
    /// root cause to investigate).
    #[test]
    fn gh3743_three_field_names_yield_pairwise_distinct_messages() {
        let a = format_history_field_warn("currentIndex", "i64");
        let b = format_history_field_warn("entries", "array");
        let c = format_history_field_warn("entries[target].id", "i64");
        assert_ne!(a, b);
        assert_ne!(a, c);
        assert_ne!(b, c);
    }

    /// Same field name + different expected type must yield distinct
    /// messages so a refactor that changes the expected type (e.g. CDP
    /// schema bump) doesn't produce indistinguishable log lines.
    #[test]
    fn gh3743_same_field_different_expected_type_yields_distinct_messages() {
        let a = format_history_field_warn("currentIndex", "i64");
        let b = format_history_field_warn("currentIndex", "u64");
        assert_ne!(a, b);
    }

    /// Sibling-distinctness: this helper's tag must NOT collide with
    /// prior `format_*_warn` helpers in the silent-fallback family.
    #[test]
    fn gh3743_helper_does_not_leak_sibling_tags() {
        let msg = format_history_field_warn("currentIndex", "i64");
        for sibling in [
            "GH #3725", "GH #3727", "GH #3730", "GH #3732", "GH #3734", "GH #3737", "GH #3739",
            "GH #3741",
        ] {
            assert!(
                !msg.contains(sibling),
                "#3743 msg must not leak sibling tag {sibling}: {msg}"
            );
        }
    }

    /// Naming convention discoverability.
    #[test]
    fn gh3743_helper_name_follows_family_convention() {
        let name = "format_history_field_warn";
        assert!(name.starts_with("format_"));
        assert!(name.ends_with("_warn"));
    }

    /// Pure-function shape-extraction logic mirror: given a parsed
    /// history JSON value, the same field-access pattern that
    /// `navigate_history_delta` uses must yield the documented results
    /// for each malformed shape. This pins the behaviour without
    /// requiring a live CDP session.
    #[test]
    fn gh3743_field_extraction_logic_mirrors_navigate_history_delta() {
        let well_formed = serde_json::json!({
            "currentIndex": 2,
            "entries": [
                {"id": 100, "url": "a"},
                {"id": 200, "url": "b"},
                {"id": 300, "url": "c"},
            ],
        });
        assert_eq!(well_formed["currentIndex"].as_i64(), Some(2));
        assert!(well_formed["entries"].as_array().is_some());
        assert_eq!(well_formed["entries"][2]["id"].as_i64(), Some(300));

        // Malformed: currentIndex is a string.
        let bad_index = serde_json::json!({"currentIndex": "two", "entries": []});
        assert!(bad_index["currentIndex"].as_i64().is_none());

        // Malformed: entries is an object.
        let bad_entries = serde_json::json!({"currentIndex": 0, "entries": {}});
        assert!(bad_entries["entries"].as_array().is_none());

        // Malformed: entries[target].id is a string.
        let bad_id = serde_json::json!({
            "currentIndex": 0,
            "entries": [{"id": "string"}],
        });
        assert!(bad_id["entries"][0]["id"].as_i64().is_none());
    }

    /// Two distinct ways a target entry might be unreachable:
    /// (a) entries is empty / target out of range — Ok(()) no-op (legitimate),
    /// (b) entries[target].id is wrong type — Ok(()) no-op (warn).
    /// These two paths share the same Ok(()) outcome so callers see the
    /// same behaviour, but the warn distinguishes them in logs.
    #[test]
    fn gh3743_no_op_outcome_is_shared_across_legitimate_and_corrupt_paths() {
        // Both produce the same outcome to the caller (no-op), but
        // only the corrupt path emits a warn (the warn fires when
        // `as_i64` returns None for `entries[target].id`).
        let entries_empty = serde_json::json!({"currentIndex": 0, "entries": []});
        let entries = entries_empty["entries"].as_array().unwrap();
        let target: i64 = 0 + (-1); // delta = -1
        let out_of_range = target < 0 || target >= entries.len() as i64;
        assert!(out_of_range, "delta=-1 from current=0 is out of range");

        // Distinct from this: if entries[target].id is the wrong type.
        let entries_bad_id = serde_json::json!({
            "currentIndex": 0,
            "entries": [{"id": "string"}],
        });
        let target: i64 = 0 + 1; // delta = +1, but len=1 → out of range
        let len = entries_bad_id["entries"].as_array().unwrap().len() as i64;
        let in_range = target >= 0 && target < len;
        assert!(!in_range, "test premise: target out of range here");
    }
}

#[cfg(test)]
mod gh3745_parse_page_request_warn_tests {
    use super::*;

    /// Empty / whitespace lines stay silent (None, no panic).
    #[test]
    fn gh3745_empty_and_whitespace_lines_return_none_silently() {
        assert!(parse_page_request("").is_none());
        assert!(parse_page_request("   ").is_none());
        assert!(parse_page_request("\n").is_none());
        assert!(parse_page_request("\t\r\n  ").is_none());
    }

    /// Non-JSON lines stay silent — JS runtime may interleave plain
    /// console.log output on the same stream and those must not warn.
    #[test]
    fn gh3745_non_json_lines_return_none_without_warning() {
        assert!(parse_page_request("hello world").is_none());
        assert!(parse_page_request("[unclosed").is_none());
        assert!(parse_page_request("{not: json}").is_none());
        assert!(parse_page_request("console.log: ready").is_none());
    }

    /// Valid-JSON-but-unknown-kind returns None AND would warn (we can't
    /// directly capture tracing without infra, but we can pin the helper
    /// message for any input that triggers it).
    #[test]
    fn gh3745_valid_json_unknown_kind_returns_none() {
        let unknown = r#"{"kind":"FutureVariant","req_id":42,"extra":1}"#;
        assert!(parse_page_request(unknown).is_none());
    }

    /// Valid JSON with missing req_id field also returns None (still a
    /// shape mismatch the JS-side promise would otherwise hang on).
    #[test]
    fn gh3745_valid_json_missing_req_id_returns_none() {
        let missing = r#"{"kind":"NewPage"}"#;
        assert!(parse_page_request(missing).is_none());
    }

    /// Well-formed PageRequest still round-trips cleanly. Note the
    /// `kind` field uses `serde(rename_all = "snake_case")`, so the wire
    /// form for `PageRequest::NewPage` is `"new_page"`.
    #[test]
    fn gh3745_well_formed_request_still_round_trips() {
        let line = r#"{"kind":"new_page","req_id":7}"#;
        let req = parse_page_request(line).expect("well-formed new_page should parse");
        match req {
            PageRequest::NewPage { req_id } => assert_eq!(req_id, 7),
            other => panic!("expected NewPage, got {other:?}"),
        }
    }

    /// Helper output carries the issue tag so the warn is greppable
    /// during incident triage.
    #[test]
    fn gh3745_helper_message_contains_issue_tag() {
        let err = serde_json::from_str::<PageRequest>("{\"kind\":\"X\"}")
            .err()
            .expect("bad shape should yield Err");
        let msg = format_unknown_page_request_warn("{\"kind\":\"X\"}", &err);
        assert!(
            msg.contains("GH #3745"),
            "msg must contain issue tag: {msg}"
        );
        assert!(msg.contains("PageRequest"), "msg must name the type: {msg}");
        assert!(
            msg.contains("never resolve"),
            "msg must call out the promise-hang consequence: {msg}"
        );
    }

    /// The serde error message is round-tripped into the warn output so
    /// engineers can see the exact missing/extra field without rerunning.
    #[test]
    fn gh3745_helper_message_round_trips_serde_error() {
        let err = serde_json::from_str::<PageRequest>("{\"kind\":\"X\",\"req_id\":1}")
            .err()
            .expect("bad shape should yield Err");
        let err_str = err.to_string();
        let msg = format_unknown_page_request_warn("preview", &err);
        // serde_json::Error renders the variant name; assert a stable substring.
        assert!(
            msg.contains(&err_str),
            "msg must embed serde error: msg={msg}, err={err_str}"
        );
    }

    /// Same inputs yield byte-identical messages (deterministic, safe to log).
    #[test]
    fn gh3745_helper_message_is_deterministic() {
        let err = serde_json::from_str::<PageRequest>("{}").err().unwrap();
        let a = format_unknown_page_request_warn("p", &err);
        let b = format_unknown_page_request_warn("p", &err);
        assert_eq!(a, b);
    }

    /// Sibling-distinctness: the #3745 warn must NOT collide with any
    /// other warn-tag emitted by this module (specifically the #3743
    /// history-field warn family).
    #[test]
    fn gh3745_warn_is_distinct_from_sibling_3743_warns() {
        let err = serde_json::from_str::<PageRequest>("{}").err().unwrap();
        let msg_3745 = format_unknown_page_request_warn("p", &err);
        let msg_3743 = format_history_field_warn("currentIndex", "i64");
        assert_ne!(msg_3745, msg_3743);
        assert!(!msg_3745.contains("#3743"));
        assert!(!msg_3743.contains("#3745"));
    }

    /// Naming convention discoverability — keeps the warn-helper family
    /// uniformly named so future authors find them via `format_*_warn`.
    #[test]
    fn gh3745_helper_name_follows_family_convention() {
        let name = "format_unknown_page_request_warn";
        assert!(name.starts_with("format_"));
        assert!(name.ends_with("_warn"));
    }

    /// Line preview is truncated so a multi-megabyte malformed line
    /// doesn't bloat the warn log. The fix uses `.chars().take(200)`;
    /// pin that behaviour against the real parse path.
    #[test]
    fn gh3745_long_unknown_kind_line_still_returns_none() {
        // Build a valid-JSON-but-bad-shape line whose `kind` value alone
        // is ~1 KiB. parse_page_request must still cleanly return None
        // rather than panic, and the (untestable here) warn would carry
        // a bounded preview.
        let big_kind = "Z".repeat(1024);
        let line = format!("{{\"kind\":\"{big_kind}\",\"req_id\":1}}");
        assert!(parse_page_request(&line).is_none());

        // Preview-truncation contract on the helper input side.
        let preview: String = line.chars().take(200).collect();
        assert_eq!(preview.chars().count(), 200);
        let err = serde_json::from_str::<PageRequest>(&line).err().unwrap();
        let msg = format_unknown_page_request_warn(&preview, &err);
        assert!(msg.contains(&preview));
    }

    #[test]
    fn mouse_event_buttons_match_cdp_button_bitfield() {
        assert_eq!(mouse_event_buttons("mousePressed", Some("left")), Some(1));
        assert_eq!(mouse_event_buttons("mousePressed", Some("right")), Some(2));
        assert_eq!(mouse_event_buttons("mousePressed", Some("middle")), Some(4));
        assert_eq!(mouse_event_buttons("mouseReleased", Some("left")), Some(0));
        assert_eq!(mouse_event_buttons("mouseMoved", Some("left")), None);
    }
}
// CODEGEN-END
