// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
// CODEGEN-BEGIN
//! High-level Page API — navigate, evaluate JS, query DOM, screenshot.

use crate::browser::cdp::CdpSession;
use crate::browser::locator::{Locator, LocatorError, LocatorOptions, SelectorExpr};
use anyhow::{Context, Result};
use serde_json::Value;

/// A browser page (tab) with a CDP session attached.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
pub struct Page {
    session: CdpSession,
    target_id: String,
    /// CDP `browserContextId` of the context that created this page.
    /// `None` for pages opened via the implicit default context — keeps the
    /// existing page-level API (50-test regression surface) unchanged.
    // @spec .aw/issues/open/enhancement-browsercontext-refactor-multi-context-isolation-fo.md#R4
    context_id: Option<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
impl Page {
    pub(crate) fn new(session: CdpSession, target_id: String) -> Self {
        Self {
            session,
            target_id,
            context_id: None,
        }
    }

    /// Construct a `Page` with an explicit `context_id`. Used by
    /// `BrowserContext::new_page()` for non-default contexts.
    // @spec .aw/issues/open/enhancement-browsercontext-refactor-multi-context-isolation-fo.md#R4
    pub(crate) fn with_context(
        session: CdpSession,
        target_id: String,
        context_id: Option<String>,
    ) -> Self {
        Self {
            session,
            target_id,
            context_id,
        }
    }

    /// The `browserContextId` this page belongs to, if any.
    pub fn context_id(&self) -> Option<&str> {
        self.context_id.as_deref()
    }

    /// Navigate to a URL and wait for the load event.
    pub async fn goto(&self, url: &str) -> Result<()> {
        self.session
            .send("Page.enable", serde_json::json!({}))
            .await?;

        self.session
            .send("Page.navigate", serde_json::json!({ "url": url }))
            .await?;

        // Wait for loadEventFired by polling — a production impl would use
        // the event stream, but this is sufficient for the initial version.
        self.wait_for_load().await?;
        Ok(())
    }

    /// Evaluate a JavaScript expression and return the result.
    pub async fn evaluate(&self, expression: &str) -> Result<Value> {
        let result = self
            .session
            .send(
                "Runtime.evaluate",
                serde_json::json!({
                    "expression": expression,
                    "returnByValue": true,
                    "awaitPromise": true,
                }),
            )
            .await?;

        if let Some(exception) = result.get("exceptionDetails") {
            let text = exception["exception"]["description"]
                .as_str()
                .unwrap_or("Unknown JS error");
            anyhow::bail!("JS evaluation error: {}", text);
        }

        Ok(result["result"]["value"].clone())
    }

    /// Register JavaScript to run before every future document script.
    ///
    /// This wraps CDP `Page.addScriptToEvaluateOnNewDocument`. Call it before
    /// [`goto`] when a test needs to install observers before app code runs.
    /// Returns Chrome's script identifier when the backend provides one.
    pub async fn add_init_script(&self, source: &str) -> Result<String> {
        let result = self
            .session
            .send(
                "Page.addScriptToEvaluateOnNewDocument",
                serde_json::json!({ "source": source }),
            )
            .await?;

        // GH #3770 — was a silent .unwrap_or("") that broke script cleanup
        // when CDP returned a wrong-shape identifier (the identifier is
        // needed to later remove the script via
        // removeScriptToEvaluateOnNewDocument).
        Ok(coerce_page_string_or_warn(
            &result["identifier"],
            "addScriptToEvaluateOnNewDocument.identifier",
        ))
    }

    /// Query a single element by CSS selector. Returns `None` if not found.
    pub async fn query_selector(&self, selector: &str) -> Result<Option<ElementHandle>> {
        let doc = self
            .session
            .send("DOM.getDocument", serde_json::json!({}))
            .await?;
        let root_node_id = doc["root"]["nodeId"]
            .as_i64()
            .context("Missing root nodeId")?;

        let result = self
            .session
            .send(
                "DOM.querySelector",
                serde_json::json!({
                    "nodeId": root_node_id,
                    "selector": selector,
                }),
            )
            .await?;

        let node_id = result["nodeId"].as_i64().unwrap_or(0);
        if node_id == 0 {
            return Ok(None);
        }

        Ok(Some(ElementHandle {
            session: self.session.clone(),
            node_id,
        }))
    }

    /// Take a screenshot of the page. Returns PNG bytes.
    pub async fn screenshot(&self) -> Result<Vec<u8>> {
        let result = self
            .session
            .send(
                "Page.captureScreenshot",
                serde_json::json!({ "format": "png" }),
            )
            .await?;

        let data = result["data"].as_str().context("Missing screenshot data")?;

        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(data)
            .context("Failed to decode screenshot base64")?;

        Ok(bytes)
    }

    /// Bring this page's target to the foreground before visual capture or input.
    pub async fn bring_to_front(&self) -> Result<()> {
        self.session
            .send("Page.bringToFront", serde_json::json!({}))
            .await?;
        Ok(())
    }

    /// Get the page title.
    pub async fn title(&self) -> Result<String> {
        let val = self.evaluate("document.title").await?;
        // GH #3770 — silent fallback masked wrong-shape CDP responses.
        Ok(coerce_page_string_or_warn(&val, "title"))
    }

    /// Get the current URL.
    pub async fn url(&self) -> Result<String> {
        let val = self.evaluate("window.location.href").await?;
        // GH #3770 — silent fallback masked wrong-shape CDP responses.
        Ok(coerce_page_string_or_warn(&val, "url"))
    }

    /// Wait for a selector to appear in the DOM (up to timeout_ms).
    pub async fn wait_for_selector(
        &self,
        selector: &str,
        timeout_ms: u64,
    ) -> Result<ElementHandle> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);

        loop {
            if let Some(el) = self.query_selector(selector).await? {
                return Ok(el);
            }
            if start.elapsed() > timeout {
                anyhow::bail!(
                    "Timeout waiting for selector '{}' after {}ms",
                    selector,
                    timeout_ms
                );
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }

    /// Target ID of this page.
    pub fn target_id(&self) -> &str {
        &self.target_id
    }

    /// Access the underlying CDP session for low-level commands.
    pub fn session(&self) -> &CdpSession {
        &self.session
    }

    /// Create a [`Locator`] rooted at this page's document.
    ///
    /// Supports CSS (default), `role=<role>[name="..."]`, and `text=...` syntax.
    /// Returns an error only on selector parse failure; actual DOM queries
    /// happen lazily when an action is invoked on the Locator.
    pub fn locator(&self, selector: &str) -> Result<Locator, LocatorError> {
        let expr = SelectorExpr::parse(selector)?;
        Ok(Locator::new_root(
            self.session.clone(),
            expr,
            LocatorOptions::default(),
        ))
    }

    /// Convenience: locator by ARIA role and optional accessible name.
    pub fn get_by_role(&self, role: &str, name: Option<&str>) -> Locator {
        Locator::new_root(
            self.session.clone(),
            SelectorExpr::Role {
                role: role.to_string(),
                name: name.map(|s| s.to_string()),
            },
            LocatorOptions::default(),
        )
    }

    /// Convenience: locator by visible text (substring, case-insensitive).
    pub fn get_by_text(&self, text: &str) -> Locator {
        Locator::new_root(
            self.session.clone(),
            SelectorExpr::Text(text.to_string()),
            LocatorOptions::default(),
        )
    }

    /// Wait for the page load event.
    async fn wait_for_load(&self) -> Result<()> {
        const POLL_ITERS: u32 = 100;
        const POLL_INTERVAL_MS: u64 = 100;
        const TOTAL_TIMEOUT_MS: u64 = POLL_ITERS as u64 * POLL_INTERVAL_MS;

        // GH #3556 — remember the LAST observed readyState so a timeout
        // bail can name which of the three failure modes we hit:
        //   "loading"     → HTML never finished
        //   "interactive" → DOMContentLoaded blocked
        //    None         → evaluate never returned a string at all
        let mut last_state: Option<String> = None;
        for _ in 0..POLL_ITERS {
            let state = self.evaluate("document.readyState").await?;
            if let Some(s) = state.as_str() {
                if s == "complete" {
                    return Ok(());
                }
                last_state = Some(s.to_string());
            }
            tokio::time::sleep(std::time::Duration::from_millis(POLL_INTERVAL_MS)).await;
        }
        anyhow::bail!(format_page_load_timeout_err(
            TOTAL_TIMEOUT_MS,
            last_state.as_deref()
        ))
    }
}

/// Format the diagnostic emitted when `Page::wait_for_load` exhausts its
/// 10-second poll budget without observing `document.readyState ==
/// "complete"`.
///
/// Preserves the literal `"Timeout waiting for page load"` substring so
/// existing log-grep / monitoring rules keep firing on the same hook,
/// then appends the timeout bound and the last observed readyState plus
/// a branch-specific next-step pointer. Tagged `GH #3556`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
pub(crate) fn format_page_load_timeout_err(timeout_ms: u64, last_state: Option<&str>) -> String {
    let (state_label, hint) = match last_state {
        Some("loading") => (
            "loading".to_string(),
            "the HTML byte stream never finished — look at network / upstream server / slow CDN response",
        ),
        Some("interactive") => (
            "interactive".to_string(),
            "the HTML was parsed but `DOMContentLoaded` never fired — look for a blocking sync script or a long-running DOMContentLoaded listener in your app",
        ),
        Some(other) => (
            other.to_string(),
            "unexpected readyState — likely an unstable CDP target; restart the browser context and re-run",
        ),
        None => (
            "(no string value ever observed)".to_string(),
            "`evaluate(document.readyState)` never returned a string — the page is probably still on about:blank, navigation never started, or the CDP Runtime.evaluate is broken on this target",
        ),
    };
    format!(
        "GH #3556 Timeout waiting for page load after {timeout_ms}ms; last observed document.readyState = '{state_label}'. Next step: {hint}."
    )
}

/// Handle to a DOM element, allowing interaction.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
pub struct ElementHandle {
    session: CdpSession,
    node_id: i64,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
impl ElementHandle {
    /// Click this element.
    pub async fn click(&self) -> Result<()> {
        let box_model = self
            .session
            .send(
                "DOM.getBoxModel",
                serde_json::json!({ "nodeId": self.node_id }),
            )
            .await?;

        // Get the center of the content quad.
        let content = &box_model["model"]["content"];
        if let Some(arr) = content.as_array() {
            if arr.len() >= 4 {
                let x = (arr[0].as_f64().unwrap_or(0.0) + arr[4].as_f64().unwrap_or(0.0)) / 2.0;
                let y = (arr[1].as_f64().unwrap_or(0.0) + arr[5].as_f64().unwrap_or(0.0)) / 2.0;

                // Move mouse and click.
                self.session
                    .send(
                        "Input.dispatchMouseEvent",
                        serde_json::json!({
                            "type": "mouseMoved",
                            "x": x,
                            "y": y,
                        }),
                    )
                    .await?;
                self.session
                    .send(
                        "Input.dispatchMouseEvent",
                        serde_json::json!({
                            "type": "mousePressed",
                            "x": x,
                            "y": y,
                            "button": "left",
                            "clickCount": 1,
                        }),
                    )
                    .await?;
                self.session
                    .send(
                        "Input.dispatchMouseEvent",
                        serde_json::json!({
                            "type": "mouseReleased",
                            "x": x,
                            "y": y,
                            "button": "left",
                            "clickCount": 1,
                        }),
                    )
                    .await?;
            }
        }

        Ok(())
    }

    /// Type text into this element (focuses it first).
    pub async fn type_text(&self, text: &str) -> Result<()> {
        self.session
            .send("DOM.focus", serde_json::json!({ "nodeId": self.node_id }))
            .await?;

        for ch in text.chars() {
            self.session
                .send(
                    "Input.dispatchKeyEvent",
                    serde_json::json!({
                        "type": "keyDown",
                        "text": ch.to_string(),
                    }),
                )
                .await?;
            self.session
                .send(
                    "Input.dispatchKeyEvent",
                    serde_json::json!({
                        "type": "keyUp",
                        "text": ch.to_string(),
                    }),
                )
                .await?;
        }

        Ok(())
    }

    /// Get the text content of this element.
    pub async fn text_content(&self) -> Result<String> {
        let result = self
            .session
            .send(
                "Runtime.callFunctionOn",
                serde_json::json!({
                    "functionDeclaration": "function() { return this.textContent; }",
                    "objectId": self.resolve_object_id().await?,
                    "returnByValue": true,
                }),
            )
            .await?;

        // GH #3770 — silent fallback masked wrong-shape CDP responses
        // for ElementHandle::text_content (sibling to Locator family).
        Ok(coerce_page_string_or_warn(
            &result["result"]["value"],
            "ElementHandle::text_content",
        ))
    }

    /// Resolve this DOM node to a Runtime object ID.
    async fn resolve_object_id(&self) -> Result<String> {
        let result = self
            .session
            .send(
                "DOM.resolveNode",
                serde_json::json!({ "nodeId": self.node_id }),
            )
            .await?;

        result["object"]["objectId"]
            .as_str()
            .map(|s| s.to_string())
            .context("Failed to resolve node to object")
    }
}

/// GH #3770 — name a CDP JSON value's shape for diagnostics. Sibling of
/// `locator_value_kind` in browser/locator.rs (GH #3768) — kept local to
/// page.rs so the two warn families stay independently auditable.
fn page_value_kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// GH #3770 — coerce a CDP value that the protocol expects to be a
/// string. Pass strings through, treat null as silent "", and warn on
/// wrong-shape so operators can chase polyfill / CDP shim drift.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
pub(crate) fn coerce_page_string_or_warn(val: &Value, op: &str) -> String {
    match val {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => {
            let kind = page_value_kind(other);
            tracing::warn!(
                target: "jet::browser::page",
                op = %op,
                actual_type = %kind,
                "{}",
                format_page_string_shape_warn(op, kind)
            );
            String::new()
        }
    }
}

/// GH #3770 — diagnostic for a wrong-shape CDP string response.
/// Operators grep for "GH #3770" to chase polyfill / CDP shim drift.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
pub(crate) fn format_page_string_shape_warn(op: &str, actual_type: &str) -> String {
    format!(
        "GH #3770 jet page op `{op}` got wrong-shape CDP response \
         (expected string or null, got {actual_type}); coerced to \"\". \
         Check for a JS polyfill or CDP shim that altered the response \
         envelope."
    )
}

#[cfg(test)]
mod gh3770_page_shape_warn_tests {
    //! GH #3770 — Page::title, Page::url,
    //! addScriptToEvaluateOnNewDocument.identifier, and
    //! ElementHandle::text_content used silent fallbacks
    //! (`.as_str().unwrap_or("")`) that masked wrong-shape CDP responses
    //! as empty strings. Tests cover each shape branch +
    //! helper-name discoverability + sibling-distinctness vs. GH #3768
    //! Locator family.

    use super::*;
    use serde_json::{json, Value};

    /// GH #3770 — valid string passes through.
    #[test]
    fn gh3770_string_passes_through() {
        assert_eq!(
            coerce_page_string_or_warn(&json!("Example"), "title"),
            "Example"
        );
        assert_eq!(
            coerce_page_string_or_warn(&json!("https://example.com/"), "url"),
            "https://example.com/"
        );
    }

    /// GH #3770 — null is silent (legitimate "no value" semantics).
    #[test]
    fn gh3770_null_returns_empty_silent() {
        assert_eq!(coerce_page_string_or_warn(&Value::Null, "title"), "");
        assert_eq!(coerce_page_string_or_warn(&Value::Null, "url"), "");
    }

    /// GH #3770 — wrong-shape values (number, bool, array, object)
    /// return "" with a warn.
    #[test]
    fn gh3770_wrong_shape_returns_empty() {
        assert_eq!(coerce_page_string_or_warn(&json!(42), "title"), "");
        assert_eq!(coerce_page_string_or_warn(&json!(true), "url"), "");
        assert_eq!(coerce_page_string_or_warn(&json!([1, 2]), "title"), "");
        assert_eq!(coerce_page_string_or_warn(&json!({"x": 1}), "url"), "");
    }

    /// GH #3770 — empty string is treated as a legitimate empty value
    /// (passes through).
    #[test]
    fn gh3770_empty_string_passes_through() {
        assert_eq!(coerce_page_string_or_warn(&json!(""), "title"), "");
    }

    /// GH #3770 — issue-tag discoverability. Operators grep "GH #3770".
    #[test]
    fn gh3770_helper_includes_issue_tag() {
        assert!(format_page_string_shape_warn("title", "number").contains("GH #3770"));
    }

    /// GH #3770 — helper message records both the operation name and
    /// the actual JS type so operators can act without rerunning with
    /// debug logging.
    #[test]
    fn gh3770_helper_records_op_and_actual_type() {
        let msg = format_page_string_shape_warn("url", "number");
        assert!(msg.contains("`url`"));
        assert!(msg.contains("number"));
        assert!(msg.contains("expected string or null"));
    }

    /// GH #3770 — sibling-distinctness vs. the GH #3768 Locator
    /// family. Both families use shape-warn naming, but the messages
    /// disambiguate by tag and (for page) the operation label.
    #[test]
    fn gh3770_warn_distinct_from_gh3768_locator_family() {
        let msg = format_page_string_shape_warn("title", "number");
        assert!(msg.contains("GH #3770"));
        assert!(!msg.contains("GH #3768"));
        // Page family's message mentions "page op" so it's grep-able
        // separately from the locator family.
        assert!(msg.contains("page op"));
    }

    /// GH #3770 — page_value_kind covers every serde_json variant.
    #[test]
    fn gh3770_value_kind_covers_all_variants() {
        assert_eq!(page_value_kind(&Value::Null), "null");
        assert_eq!(page_value_kind(&Value::Bool(true)), "bool");
        assert_eq!(page_value_kind(&json!(1)), "number");
        assert_eq!(page_value_kind(&json!("x")), "string");
        assert_eq!(page_value_kind(&json!([])), "array");
        assert_eq!(page_value_kind(&json!({})), "object");
    }

    /// GH #3770 — the call sites for title / url /
    /// addScriptToEvaluateOnNewDocument.identifier / text_content all
    /// route through the same helper, so a single bug fix protects all
    /// four. Drive the helper with each call site's op label.
    #[test]
    fn gh3770_all_callsite_op_labels_produce_distinct_warns() {
        let title_msg = format_page_string_shape_warn("title", "number");
        let url_msg = format_page_string_shape_warn("url", "number");
        let script_msg =
            format_page_string_shape_warn("addScriptToEvaluateOnNewDocument.identifier", "number");
        let text_msg = format_page_string_shape_warn("ElementHandle::text_content", "number");

        assert!(title_msg.contains("`title`"));
        assert!(url_msg.contains("`url`"));
        assert!(script_msg.contains("addScriptToEvaluateOnNewDocument.identifier"));
        assert!(text_msg.contains("ElementHandle::text_content"));

        // The four messages must be pairwise distinct so an operator
        // grepping for one call site doesn't collide with another.
        let msgs = [&title_msg, &url_msg, &script_msg, &text_msg];
        for i in 0..msgs.len() {
            for j in (i + 1)..msgs.len() {
                assert_ne!(msgs[i], msgs[j]);
            }
        }
    }

    /// GH #3770 — the helper-name family convention
    /// (`coerce_page_string_or_warn` + `format_page_string_shape_warn`)
    /// is discoverable. Assert via use-site so a rename trips this test.
    #[test]
    fn gh3770_helper_naming_convention_discoverable() {
        let _ = coerce_page_string_or_warn(&Value::Null, "x");
        let _ = format_page_string_shape_warn("x", "number");
    }
}

#[cfg(test)]
mod gh3556_tests {
    use super::*;

    /// GH #3556 — the timeout error must include the GH tag, the literal
    /// "Timeout waiting for page load" substring (so existing log-grep
    /// rules keep firing), the timeout-ms bound, and the observed-state
    /// label.
    #[test]
    fn gh3556_page_load_timeout_err_names_tag_legacy_substring_timeout_and_state() {
        let msg = format_page_load_timeout_err(10_000, Some("loading"));

        assert!(msg.contains("GH #3556"), "must tag GH #3556, got: {msg}");
        assert!(
            msg.contains("Timeout waiting for page load"),
            "must preserve legacy substring for log-grep compat, got: {msg}"
        );
        assert!(
            msg.contains("10000"),
            "must name the timeout-ms bound, got: {msg}"
        );
        assert!(
            msg.contains("'loading'"),
            "must name the observed readyState, got: {msg}"
        );
    }

    /// GH #3556 — each of the four readyState branches (`loading`,
    /// `interactive`, `None`, other) must produce a DISTINCT next-step
    /// hint so the dev can match the symptom to the root-cause class.
    #[test]
    fn gh3556_page_load_timeout_err_branches_to_distinct_next_steps() {
        let loading = format_page_load_timeout_err(10_000, Some("loading"));
        let interactive = format_page_load_timeout_err(10_000, Some("interactive"));
        let none = format_page_load_timeout_err(10_000, None);
        let other = format_page_load_timeout_err(10_000, Some("uninitialized"));

        // Branch-specific keywords.
        assert!(
            loading.contains("HTML byte stream") || loading.contains("network"),
            "loading branch must point at network/HTML transport, got: {loading}"
        );
        assert!(
            interactive.contains("DOMContentLoaded") || interactive.contains("blocking sync script"),
            "interactive branch must point at DOMContentLoaded / blocking script, got: {interactive}"
        );
        assert!(
            none.contains("about:blank") || none.contains("navigation never started"),
            "None branch must point at about:blank / nav-not-started, got: {none}"
        );
        assert!(
            other.contains("unexpected readyState") || other.contains("unstable CDP target"),
            "unknown-state branch must point at CDP instability, got: {other}"
        );

        // Cross-distinctness: each pair differs.
        assert_ne!(loading, interactive, "loading and interactive must differ");
        assert_ne!(loading, none, "loading and none must differ");
        assert_ne!(interactive, none, "interactive and none must differ");
    }

    /// GH #3556 — the legacy bare bail substring "Timeout waiting for
    /// page load" must remain even on the None / unknown-state branches,
    /// because production log-grep / monitoring rules look for that
    /// literal regardless of the suffix.
    #[test]
    fn gh3556_page_load_timeout_err_preserves_legacy_substring_on_all_branches() {
        for last in [
            Some("loading"),
            Some("interactive"),
            Some("uninitialized"),
            None,
        ] {
            let msg = format_page_load_timeout_err(10_000, last);
            assert!(
                msg.contains("Timeout waiting for page load"),
                "branch {last:?} dropped the legacy substring; would break log-grep. Got: {msg}"
            );
        }
    }
}
// CODEGEN-END
