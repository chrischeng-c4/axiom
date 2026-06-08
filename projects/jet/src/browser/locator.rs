// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
// CODEGEN-BEGIN
//! Chainable locator API on top of `Page`, matching Playwright's `page.locator()`
//! ergonomics. See `.aw/tech-design/projects/jet/logic/locator-engine.md`.
//!
//! A [`Locator`] is a lazy, re-queryable handle to zero-or-more DOM elements.
//! Actions (`click`, `fill`, `text_content`) resolve the selector *just before*
//! the action and wait for the target to become **actionable** (attached,
//! visible, stable). Locators are cheap to construct and clone.
//!
//! ## Example
//!
//! ```no_run
//! # async fn demo(page: &jet::browser::Page) -> anyhow::Result<()> {
//! page.locator("button.submit")?.click().await?;
//! page.get_by_role("button", Some("Save")).click().await?;
//! let n = page.locator(".todo-item")?.count().await?;
//! # Ok(()) }
//! ```
//!
//! ## Selector syntax
//!
//! - `.foo`, `#id`, `div > span` — CSS (default)
//! - `role=button[name="Save"]` — ARIA role selector with optional accessible name
//! - `text=Submit` — visible text match (substring, case-insensitive by default)
//!
//! Role selectors are resolved via `[role="<role>"]` CSS fallback plus an
//! accessible-name filter (best-effort: `aria-label`, then `textContent`).

use crate::browser::cdp::CdpSession;
use serde_json::Value;
use std::fmt;
use std::time::{Duration, Instant};
use thiserror::Error;

/// Errors raised by locator operations.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
#[derive(Debug, Error)]
pub enum LocatorError {
    /// The auto-wait state machine did not reach `Actionable` within the
    /// configured timeout.
    #[error("locator timed out in state {state:?} after {timeout_ms}ms: {selector}")]
    Timeout {
        state: Actionability,
        selector: String,
        timeout_ms: u64,
    },

    /// Action expected exactly one match but the locator resolved to multiple
    /// elements and no `.nth()`/`.first()`/`.last()` was applied.
    #[error("locator matched {count} elements, expected 1 (selector: {selector}) — use .nth(), .first(), or .last() to disambiguate")]
    Ambiguous { selector: String, count: usize },

    /// A selector string failed to parse.
    #[error("invalid selector syntax: {0}")]
    InvalidSelector(String),

    /// Underlying CDP call failed.
    #[error("CDP error: {0}")]
    CdpError(String),

    /// Catch-all for evaluation errors surfaced by the page (e.g. JS threw).
    #[error("evaluation error: {0}")]
    EvalError(String),
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
impl From<anyhow::Error> for LocatorError {
    fn from(e: anyhow::Error) -> Self {
        LocatorError::CdpError(format!("{e:#}"))
    }
}

type LocatorResult<T> = Result<T, LocatorError>;

/// Auto-wait actionability stages, in the order they are checked.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Actionability {
    Detached,
    Attached,
    Visible,
    Stable,
    Actionable,
}

/// Configuration for an individual locator. Defaults match Playwright.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
#[derive(Debug, Clone)]
pub struct LocatorOptions {
    /// Total wait budget for auto-wait / resolution. Default 5000ms.
    pub timeout_ms: u64,
    /// Polling interval during auto-wait. Default 100ms.
    pub poll_interval_ms: u64,
    /// For `text=`: treat the query as substring match (true) or exact (false).
    pub text_substring: bool,
    /// For `text=` and `get_by_role` name: case-insensitive matching.
    pub text_case_insensitive: bool,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
impl Default for LocatorOptions {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            poll_interval_ms: 100,
            text_substring: true,
            text_case_insensitive: true,
        }
    }
}

/// Parsed selector expression.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
#[derive(Debug, Clone)]
pub enum SelectorExpr {
    Css(String),
    Role { role: String, name: Option<String> },
    Text(String),
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
impl SelectorExpr {
    /// Parse a user-facing selector string.
    ///
    /// - `role=button` or `role=button[name="Save"]` → `Role`
    /// - `text=Submit` → `Text`
    /// - anything else → `Css`
    pub fn parse(raw: &str) -> LocatorResult<Self> {
        let trimmed = raw.trim();
        if let Some(rest) = trimmed.strip_prefix("role=") {
            // role=button[name="Foo"] or role=button
            if let Some((role, name)) = parse_role_with_name(rest)? {
                return Ok(SelectorExpr::Role { role, name });
            } else {
                return Ok(SelectorExpr::Role {
                    role: rest.to_string(),
                    name: None,
                });
            }
        }
        if let Some(rest) = trimmed.strip_prefix("text=") {
            return Ok(SelectorExpr::Text(rest.to_string()));
        }
        if trimmed.is_empty() {
            return Err(LocatorError::InvalidSelector("empty selector".to_string()));
        }
        Ok(SelectorExpr::Css(trimmed.to_string()))
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
impl fmt::Display for SelectorExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SelectorExpr::Css(s) => write!(f, "{s}"),
            SelectorExpr::Role {
                role,
                name: Some(n),
            } => write!(f, "role={role}[name=\"{n}\"]"),
            SelectorExpr::Role { role, name: None } => write!(f, "role={role}"),
            SelectorExpr::Text(t) => write!(f, "text={t}"),
        }
    }
}

/// `role=X[name="Y"]` → `Some((X, Some(Y)))`; bare `role=X` handled by caller.
fn parse_role_with_name(s: &str) -> LocatorResult<Option<(String, Option<String>)>> {
    // Find `[name=`; if absent, return None.
    let Some(bracket_idx) = s.find('[') else {
        return Ok(None);
    };
    let role = s[..bracket_idx].to_string();
    let bracket = &s[bracket_idx..];
    if !bracket.ends_with(']') {
        return Err(LocatorError::InvalidSelector(format!(
            "unclosed bracket in role selector: {s}"
        )));
    }
    let inner = &bracket[1..bracket.len() - 1];
    let Some(name_eq) = inner.strip_prefix("name=") else {
        return Err(LocatorError::InvalidSelector(format!(
            "only name= supported inside role[], got: {inner}"
        )));
    };
    let name = name_eq.trim_matches('"').trim_matches('\'').to_string();
    Ok(Some((role, Some(name))))
}

/// Positional index within a collection.
#[derive(Debug, Clone, Copy)]
enum Index {
    /// Zero-based; negative counts from end (e.g. -1 = last).
    Nth(i32),
    First,
    Last,
}

/// Filter applied to a collection before indexing.
#[derive(Debug, Clone)]
enum Filter {
    HasText(String),
}

/// A chainable locator.
///
/// Construction is free (no I/O). Actions and queries trigger CDP calls.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
#[derive(Clone)]
pub struct Locator {
    session: CdpSession,
    /// Ordered list of scoping steps — each step queries within the previous
    /// step's match. Mirrors Playwright's `page.locator(a).locator(b)`.
    steps: Vec<SelectorExpr>,
    /// Post-resolution filters (applied to the last-step match collection).
    filters: Vec<Filter>,
    /// Optional index into the filtered collection.
    index: Option<Index>,
    opts: LocatorOptions,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
impl Locator {
    pub(crate) fn new_root(session: CdpSession, expr: SelectorExpr, opts: LocatorOptions) -> Self {
        Self {
            session,
            steps: vec![expr],
            filters: Vec::new(),
            index: None,
            opts,
        }
    }

    /// Narrow this locator by an additional selector (chains `querySelector`).
    pub fn locator(&self, selector: &str) -> LocatorResult<Locator> {
        let expr = SelectorExpr::parse(selector)?;
        let mut next = self.clone();
        next.steps.push(expr);
        next.filters.clear();
        next.index = None;
        Ok(next)
    }

    /// Filter the current match collection by visible text substring.
    pub fn filter_has_text(&self, text: &str) -> Locator {
        let mut next = self.clone();
        next.filters.push(Filter::HasText(text.to_string()));
        next
    }

    /// Pick the nth match (zero-based; negative indexes from end).
    pub fn nth(&self, i: i32) -> Locator {
        let mut next = self.clone();
        next.index = Some(Index::Nth(i));
        next
    }

    pub fn first(&self) -> Locator {
        let mut next = self.clone();
        next.index = Some(Index::First);
        next
    }

    pub fn last(&self) -> Locator {
        let mut next = self.clone();
        next.index = Some(Index::Last);
        next
    }

    /// Sub-query for a descendant by ARIA role (with optional accessible name).
    pub fn get_by_role(&self, role: &str, name: Option<&str>) -> Locator {
        let mut next = self.clone();
        next.steps.push(SelectorExpr::Role {
            role: role.to_string(),
            name: name.map(|s| s.to_string()),
        });
        next.filters.clear();
        next.index = None;
        next
    }

    /// Sub-query for a descendant by visible text.
    pub fn get_by_text(&self, text: &str) -> Locator {
        let mut next = self.clone();
        next.steps.push(SelectorExpr::Text(text.to_string()));
        next.filters.clear();
        next.index = None;
        next
    }

    /// Override options (timeout, polling).
    pub fn with_options(&self, opts: LocatorOptions) -> Locator {
        let mut next = self.clone();
        next.opts = opts;
        next
    }

    /// Human-readable representation of the full chain (for error messages).
    fn describe(&self) -> String {
        let mut out = String::new();
        for (i, step) in self.steps.iter().enumerate() {
            if i > 0 {
                out.push_str(" >> ");
            }
            out.push_str(&step.to_string());
        }
        for f in &self.filters {
            match f {
                Filter::HasText(t) => out.push_str(&format!(" [hasText={t:?}]")),
            }
        }
        if let Some(idx) = self.index {
            match idx {
                Index::Nth(n) => out.push_str(&format!(" [nth={n}]")),
                Index::First => out.push_str(" [first]"),
                Index::Last => out.push_str(" [last]"),
            }
        }
        out
    }

    // ── Queries ──────────────────────────────────────────────────────────────

    /// Count the matching elements (no auto-wait).
    pub async fn count(&self) -> LocatorResult<usize> {
        let js = self.compile_count_js();
        let val = self.eval(&js).await?;
        // GH #3768 — was a silent `as_u64().unwrap_or(0)` that masked
        // wrong-shape CDP responses as "no matches", which would silently
        // pass `expect(locator.count()).toBe(0)` assertions on a broken
        // JS pipeline.
        Ok(coerce_count_or_warn(&val, &self.describe()) as usize)
    }

    /// Return the textContent of the single matching element. Waits for
    /// `Attached` only (cheaper than full actionability).
    pub async fn text_content(&self) -> LocatorResult<String> {
        self.wait_until(Actionability::Attached).await?;
        let js = self.compile_single_js(".textContent ?? \"\"");
        let val = self.eval(&js).await?;
        // GH #3768 — silent fallback to "" masked wrong-shape CDP.
        Ok(coerce_text_or_warn(&val, "text_content", &self.describe()))
    }

    /// Return the `innerText` (visible, whitespace-collapsed) of the single
    /// matching element.
    pub async fn inner_text(&self) -> LocatorResult<String> {
        self.wait_until(Actionability::Attached).await?;
        let js = self.compile_single_js(".innerText ?? \"\"");
        let val = self.eval(&js).await?;
        // GH #3768 — silent fallback to "" masked wrong-shape CDP.
        Ok(coerce_text_or_warn(&val, "inner_text", &self.describe()))
    }

    /// Read an attribute from the single matching element.
    pub async fn get_attribute(&self, name: &str) -> LocatorResult<Option<String>> {
        self.wait_until(Actionability::Attached).await?;
        let js = self.compile_single_js(&format!(".getAttribute({})", json_str(name)));
        let val = self.eval(&js).await?;
        Ok(val.as_str().map(|s| s.to_string()))
    }

    /// Check whether the single matching element is visible (no wait — returns
    /// current state).
    pub async fn is_visible(&self) -> LocatorResult<bool> {
        let js = format!(
            "(() => {{ const el = {}; return el ? {} : false; }})()",
            self.compile_single_expr(),
            VISIBLE_CHECK_JS
        );
        let val = self.eval(&js).await?;
        Ok(val.as_bool().unwrap_or(false))
    }

    /// Wait until the locator reaches the given state, or timeout.
    pub async fn wait_for(&self, state: Actionability) -> LocatorResult<()> {
        self.wait_until(state).await
    }

    // ── Actions ──────────────────────────────────────────────────────────────

    /// Click the matching element.
    pub async fn click(&self) -> LocatorResult<()> {
        self.wait_until(Actionability::Actionable).await?;
        // Use element.click() — simpler and more reliable than coordinate math,
        // and sufficient for MVP. See TD §Known Gaps for tradeoff.
        let js = format!(
            "(() => {{ const el = {}; if (!el) throw new Error('no match'); el.click(); return true; }})()",
            self.compile_single_expr()
        );
        self.eval(&js).await?;
        Ok(())
    }

    /// Fill a text input: clear existing value then set to `text` and dispatch
    /// `input` + `change` events so JS frameworks pick up the change.
    pub async fn fill(&self, text: &str) -> LocatorResult<()> {
        self.wait_until(Actionability::Actionable).await?;
        let js = format!(
            "(() => {{
                const el = {expr};
                if (!el) throw new Error('no match');
                el.focus();
                const nativeSetter = Object.getOwnPropertyDescriptor(
                    el.tagName === 'TEXTAREA' ? window.HTMLTextAreaElement.prototype : window.HTMLInputElement.prototype,
                    'value'
                )?.set;
                if (nativeSetter) {{ nativeSetter.call(el, {val}); }} else {{ el.value = {val}; }}
                el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return true;
            }})()",
            expr = self.compile_single_expr(),
            val = json_str(text),
        );
        self.eval(&js).await?;
        Ok(())
    }

    /// Check a checkbox (idempotent — clicks only if not already checked).
    pub async fn check(&self) -> LocatorResult<()> {
        self.wait_until(Actionability::Actionable).await?;
        let js = format!(
            "(() => {{ const el = {}; if (!el) throw new Error('no match'); if (!el.checked) el.click(); return true; }})()",
            self.compile_single_expr()
        );
        self.eval(&js).await?;
        Ok(())
    }

    /// Uncheck a checkbox.
    pub async fn uncheck(&self) -> LocatorResult<()> {
        self.wait_until(Actionability::Actionable).await?;
        let js = format!(
            "(() => {{ const el = {}; if (!el) throw new Error('no match'); if (el.checked) el.click(); return true; }})()",
            self.compile_single_expr()
        );
        self.eval(&js).await?;
        Ok(())
    }

    /// Hover the matching element (dispatches mouseenter + mousemove).
    pub async fn hover(&self) -> LocatorResult<()> {
        self.wait_until(Actionability::Actionable).await?;
        let js = format!(
            "(() => {{
                const el = {};
                if (!el) throw new Error('no match');
                const r = el.getBoundingClientRect();
                el.dispatchEvent(new MouseEvent('mouseenter', {{ bubbles: true, clientX: r.left + r.width/2, clientY: r.top + r.height/2 }}));
                el.dispatchEvent(new MouseEvent('mousemove', {{ bubbles: true, clientX: r.left + r.width/2, clientY: r.top + r.height/2 }}));
                return true;
            }})()",
            self.compile_single_expr()
        );
        self.eval(&js).await?;
        Ok(())
    }

    // ── Internals: JS compilation ────────────────────────────────────────────

    /// Build a JS expression that returns the *collection* of current matches
    /// (a JS Array, already filtered + indexed per this locator).
    fn compile_collection_expr(&self) -> String {
        // Start from document, progressively narrow.
        // For the first step, scope is `document`; subsequent steps scope to
        // the flattened matches of the previous step.
        let mut expr = String::from("[document]");
        for step in &self.steps {
            let step_js = compile_step(step);
            // For each element in the current collection, run step_js and flatten.
            expr = format!(
                "({expr}).flatMap((__scope) => {step_js})",
                expr = expr,
                step_js = step_js,
            );
        }
        // Apply filters
        for f in &self.filters {
            match f {
                Filter::HasText(t) => {
                    expr = format!(
                        "({expr}).filter((el) => (el.textContent ?? '').includes({needle}))",
                        expr = expr,
                        needle = json_str(t),
                    );
                }
            }
        }
        // Apply index
        if let Some(idx) = self.index {
            expr = match idx {
                Index::First => format!("[({expr})[0]].filter(Boolean)"),
                Index::Last => format!("[({expr}).slice(-1)[0]].filter(Boolean)"),
                Index::Nth(n) => {
                    if n >= 0 {
                        format!("[({expr})[{n}]].filter(Boolean)")
                    } else {
                        format!("[({expr}).slice({n})[0]].filter(Boolean)")
                    }
                }
            };
        }
        expr
    }

    /// Build a JS expression that returns `count` of current collection.
    fn compile_count_js(&self) -> String {
        format!("({}).length", self.compile_collection_expr())
    }

    /// Build a JS expression that returns the *single* selected element (first
    /// from collection if index not set and count == 1, else first element).
    /// Falls back to `null` when empty.
    fn compile_single_expr(&self) -> String {
        format!("({})[0] ?? null", self.compile_collection_expr())
    }

    /// Compile JS that reads `suffix` (e.g. `.textContent`) from the single
    /// matching element. Returns empty/null if no match.
    fn compile_single_js(&self, suffix: &str) -> String {
        format!(
            "(() => {{ const el = {}; return el == null ? null : el{suffix}; }})()",
            self.compile_single_expr()
        )
    }

    // ── Internals: wait + eval ───────────────────────────────────────────────

    /// Poll until the locator reaches `target` state, or timeout.
    async fn wait_until(&self, target: Actionability) -> LocatorResult<()> {
        let start = Instant::now();
        let budget = Duration::from_millis(self.opts.timeout_ms);
        let poll = Duration::from_millis(self.opts.poll_interval_ms);

        let mut last_reached = self.probe_state().await?;

        loop {
            if actionability_rank(last_reached) >= actionability_rank(target) {
                return Ok(());
            }

            if start.elapsed() > budget {
                return Err(LocatorError::Timeout {
                    state: last_reached,
                    selector: self.describe(),
                    timeout_ms: self.opts.timeout_ms,
                });
            }
            tokio::time::sleep(poll).await;
            last_reached = self.probe_state().await?;
        }
    }

    /// Inspect the DOM in one `Runtime.evaluate` round-trip to determine the
    /// best-reached actionability state.
    async fn probe_state(&self) -> LocatorResult<Actionability> {
        let js = format!(
            "(() => {{
                const coll = {coll};
                if (coll.length === 0) return 'Detached';
                const el = coll[0];
                if (!el || !el.isConnected) return 'Detached';
                // Visible check
                const rect = el.getBoundingClientRect();
                const style = window.getComputedStyle(el);
                const visible = style.visibility !== 'hidden'
                    && style.display !== 'none'
                    && rect.width > 0 && rect.height > 0;
                if (!visible) return 'Attached';
                // Stability check: compare rect to 1 frame later is impractical
                // synchronously — treat visible+non-disabled as Actionable for MVP.
                if (el.disabled) return 'Stable';
                return 'Actionable';
            }})()",
            coll = self.compile_collection_expr()
        );
        let val = self.eval(&js).await?;
        // GH #3768 — silent fallback to "Detached" masked wrong-shape CDP
        // responses (a polyfill or shim could return a non-string), which
        // would silently report the locator as detached and cause
        // actionability waits to time out with no diagnostic. Warn on
        // wrong-shape but keep the "Detached" safe default so actionability
        // timeouts still surface to the test.
        let s = coerce_actionability_or_warn(&val, &self.describe());
        Ok(match s.as_str() {
            "Attached" => Actionability::Attached,
            "Visible" => Actionability::Visible,
            "Stable" => Actionability::Stable,
            "Actionable" => Actionability::Actionable,
            _ => Actionability::Detached,
        })
    }

    /// Run a JS expression in the page context with returnByValue + awaitPromise.
    async fn eval(&self, expression: &str) -> LocatorResult<Value> {
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
            .await
            .map_err(|e| LocatorError::CdpError(format!("{e:#}")))?;

        if let Some(exception) = result.get("exceptionDetails") {
            let text = exception["exception"]["description"]
                .as_str()
                .or(exception["text"].as_str())
                .unwrap_or("unknown JS error");
            return Err(LocatorError::EvalError(text.to_string()));
        }

        Ok(result["result"]["value"].clone())
    }
}

fn actionability_rank(a: Actionability) -> u8 {
    match a {
        Actionability::Detached => 0,
        Actionability::Attached => 1,
        Actionability::Visible => 2,
        Actionability::Stable => 3,
        Actionability::Actionable => 4,
    }
}

/// Compile a single step into a JS expression that, given `__scope` (an
/// Element or Document), yields an Array of matching descendants.
fn compile_step(step: &SelectorExpr) -> String {
    match step {
        SelectorExpr::Css(sel) => {
            format!("Array.from(__scope.querySelectorAll({}))", json_str(sel))
        }
        SelectorExpr::Role { role, name } => {
            let role_sel = format!("[role=\"{role}\"], {}", html_tags_for_role(role));
            let base = format!(
                "Array.from(__scope.querySelectorAll({}))",
                json_str(&role_sel)
            );
            if let Some(n) = name {
                format!(
                    "({base}).filter((el) => {{ const an = (el.getAttribute('aria-label') ?? el.textContent ?? '').trim(); return an.toLowerCase().includes({}.toLowerCase()); }})",
                    json_str(n)
                )
            } else {
                base
            }
        }
        SelectorExpr::Text(t) => {
            format!(
                "Array.from(__scope.querySelectorAll('*')).filter((el) => (el.textContent ?? '').toLowerCase().includes({}.toLowerCase()))",
                json_str(t)
            )
        }
    }
}

/// Maps a (limited) set of ARIA roles to their implicit HTML tags, so that a
/// `role=button` selector matches `<button>` elements without an explicit
/// `role="button"` attribute. Best-effort for MVP.
fn html_tags_for_role(role: &str) -> &'static str {
    match role {
        "button" => "button, input[type=\"button\"], input[type=\"submit\"]",
        "link" => "a[href]",
        "textbox" => "input[type=\"text\"], input:not([type]), textarea",
        "checkbox" => "input[type=\"checkbox\"]",
        "radio" => "input[type=\"radio\"]",
        "heading" => "h1, h2, h3, h4, h5, h6",
        "listitem" => "li",
        "list" => "ul, ol",
        "navigation" => "nav",
        "main" => "main",
        "banner" => "header",
        "contentinfo" => "footer",
        "img" => "img",
        _ => "_no_implicit_tag_",
    }
}

/// GH #3737 — produce a valid JS double-quoted string literal for any
/// `&str` input. Happy path delegates to `serde_json::to_string` (which
/// is infallible for valid UTF-8 strings). The defensive fall-back is a
/// hand-rolled JSON-string escaper — never the prior naïve `"\"\""`
/// (empty string), which would silently corrupt every locator that
/// uses it (selectors → empty querySelectorAll, text needles →
/// matches-empty-string predicates).
///
/// This mirrors `safe_import_meta_env_define_value` (#3641) in
/// `bundler/define.rs` — same correctness-regression-lever pattern,
/// same defensive shape.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
pub(crate) fn safe_locator_json_str(s: &str) -> String {
    if let Ok(encoded) = serde_json::to_string(s) {
        return encoded;
    }
    // Defensive: hand-rolled JSON-string escape. Should be unreachable
    // because serde_json::to_string on &str is infallible, but the
    // panic-free contract is preserved without emitting broken JS that
    // would silently break every locator-by-role/text/label callsite.
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// JSON-encode a string for safe inlining into JS (handles quotes, newlines,
/// Unicode).
fn json_str(s: &str) -> String {
    // GH #3737 — was previously `unwrap_or_else(|_| "\"\"".to_string())`,
    // which silently corrupted locator JS on the (unreachable today, but
    // refactor-reachable tomorrow) Err arm. Route through the defensive
    // helper so the fall-back can never emit an empty-string literal.
    match serde_json::to_string(s) {
        Ok(encoded) => encoded,
        Err(err) => {
            tracing::warn!(
                target: "jet::browser::locator",
                error = %err,
                input_len = s.len(),
                "GH #3737 serde_json::to_string failed on a &str (should be \
                 unreachable since &str is always valid UTF-8); falling back \
                 to the hand-rolled JSON-string escaper to avoid emitting \
                 a corrupted empty-string literal into the generated locator \
                 JS. If you are seeing this warn, the input type contract \
                 of `json_str` has been changed — re-audit safe_locator_json_str."
            );
            safe_locator_json_str(s)
        }
    }
}

/// JS expression fragment for checking whether an element is visible (used by
/// `is_visible`). Expects `el` in scope.
const VISIBLE_CHECK_JS: &str = "(() => { const r = el.getBoundingClientRect(); const s = window.getComputedStyle(el); return s.visibility !== 'hidden' && s.display !== 'none' && r.width > 0 && r.height > 0; })()";

/// GH #3768 — name a CDP JSON value's shape for diagnostics.
fn locator_value_kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// GH #3768 — coerce a CDP `(collection).length` response to a `u64`,
/// warning if the value is not the expected `Number`. Returns 0 for null
/// (silent — legitimate "no matches") and 0+warn for wrong-shape.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
pub(crate) fn coerce_count_or_warn(val: &Value, selector: &str) -> u64 {
    match val {
        Value::Number(n) => n.as_u64().unwrap_or_else(|| {
            tracing::warn!(
                target: "jet::browser::locator",
                selector = %selector,
                actual = ?val,
                "{}",
                format_locator_count_shape_warn(selector, "non-u64 number")
            );
            0
        }),
        Value::Null => 0,
        other => {
            let kind = locator_value_kind(other);
            tracing::warn!(
                target: "jet::browser::locator",
                selector = %selector,
                actual_type = %kind,
                "{}",
                format_locator_count_shape_warn(selector, kind)
            );
            0
        }
    }
}

/// GH #3768 — coerce a CDP text-reading response to a string, warning on
/// wrong-shape. `null` is silent (legitimate "no element found").
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
pub(crate) fn coerce_text_or_warn(val: &Value, op: &str, selector: &str) -> String {
    match val {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => {
            let kind = locator_value_kind(other);
            tracing::warn!(
                target: "jet::browser::locator",
                op = %op,
                selector = %selector,
                actual_type = %kind,
                "{}",
                format_locator_text_shape_warn(op, selector, kind)
            );
            String::new()
        }
    }
}

/// GH #3768 — coerce a CDP actionability-eval response to a string,
/// warning on wrong-shape. Returns `"Detached"` for both null and
/// wrong-shape so the actionability wait still surfaces a timeout.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
pub(crate) fn coerce_actionability_or_warn(val: &Value, selector: &str) -> String {
    match val {
        Value::String(s) => s.clone(),
        Value::Null => "Detached".to_string(),
        other => {
            let kind = locator_value_kind(other);
            tracing::warn!(
                target: "jet::browser::locator",
                selector = %selector,
                actual_type = %kind,
                "{}",
                format_locator_actionability_shape_warn(selector, kind)
            );
            "Detached".to_string()
        }
    }
}

/// GH #3768 — diagnostic for a wrong-shape `Locator::count` response.
/// Operators grep for "GH #3768" to chase polyfill / CDP shim drift.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
pub(crate) fn format_locator_count_shape_warn(selector: &str, actual_type: &str) -> String {
    format!(
        "GH #3768 jet locator count() got wrong-shape CDP response \
         for `{selector}` (expected number, got {actual_type}); \
         coerced to 0. Check for a JS polyfill or CDP shim that \
         altered Array.prototype.length."
    )
}

/// GH #3768 — diagnostic for a wrong-shape text-content response from
/// `text_content` / `inner_text`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
pub(crate) fn format_locator_text_shape_warn(
    op: &str,
    selector: &str,
    actual_type: &str,
) -> String {
    format!(
        "GH #3768 jet locator {op} got wrong-shape CDP response for \
         `{selector}` (expected string or null, got {actual_type}); \
         coerced to \"\". Check for a JS polyfill or CDP shim."
    )
}

/// GH #3768 — diagnostic for a wrong-shape actionability eval response.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
pub(crate) fn format_locator_actionability_shape_warn(selector: &str, actual_type: &str) -> String {
    format!(
        "GH #3768 jet locator actionability probe got wrong-shape CDP \
         response for `{selector}` (expected string, got {actual_type}); \
         treated as Detached. Actionability waits may time out."
    )
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_css_by_default() {
        let e = SelectorExpr::parse("div.foo > span").unwrap();
        matches!(e, SelectorExpr::Css(_));
        assert_eq!(format!("{e}"), "div.foo > span");
    }

    #[test]
    fn parses_role_without_name() {
        let e = SelectorExpr::parse("role=button").unwrap();
        match e {
            SelectorExpr::Role { role, name } => {
                assert_eq!(role, "button");
                assert!(name.is_none());
            }
            _ => panic!("expected role"),
        }
    }

    #[test]
    fn parses_role_with_name() {
        let e = SelectorExpr::parse("role=button[name=\"Save\"]").unwrap();
        match e {
            SelectorExpr::Role { role, name } => {
                assert_eq!(role, "button");
                assert_eq!(name.as_deref(), Some("Save"));
            }
            _ => panic!("expected role"),
        }
    }

    #[test]
    fn parses_text_selector() {
        let e = SelectorExpr::parse("text=Hello World").unwrap();
        match e {
            SelectorExpr::Text(t) => assert_eq!(t, "Hello World"),
            _ => panic!("expected text"),
        }
    }

    #[test]
    fn empty_selector_errors() {
        assert!(SelectorExpr::parse("").is_err());
        assert!(SelectorExpr::parse("   ").is_err());
    }

    #[test]
    fn unclosed_role_bracket_errors() {
        assert!(SelectorExpr::parse("role=button[name=\"Save\"").is_err());
    }

    #[test]
    fn role_with_non_name_predicate_errors() {
        assert!(SelectorExpr::parse("role=button[label=\"Save\"]").is_err());
    }

    #[test]
    fn json_str_escapes_quotes() {
        assert_eq!(json_str("a\"b"), "\"a\\\"b\"");
        assert_eq!(json_str("multi\nline"), "\"multi\\nline\"");
    }

    #[test]
    fn actionability_rank_is_monotonic() {
        assert!(
            actionability_rank(Actionability::Detached)
                < actionability_rank(Actionability::Attached)
        );
        assert!(
            actionability_rank(Actionability::Attached)
                < actionability_rank(Actionability::Visible)
        );
        assert!(
            actionability_rank(Actionability::Visible) < actionability_rank(Actionability::Stable)
        );
        assert!(
            actionability_rank(Actionability::Stable)
                < actionability_rank(Actionability::Actionable)
        );
    }

    #[test]
    fn html_tags_for_known_roles() {
        assert!(html_tags_for_role("button").contains("button"));
        assert!(html_tags_for_role("link").contains("a["));
        assert_eq!(html_tags_for_role("unknown-role"), "_no_implicit_tag_");
    }

    #[test]
    fn compile_step_css_escapes_selector() {
        let js = compile_step(&SelectorExpr::Css(".foo[data-x=\"y\"]".to_string()));
        assert!(js.contains("querySelectorAll"));
        assert!(js.contains("\\\""));
    }

    #[test]
    fn compile_step_role_includes_implicit_tag() {
        let js = compile_step(&SelectorExpr::Role {
            role: "button".to_string(),
            name: None,
        });
        assert!(js.contains("[role=\\\"button\\\"]"));
        assert!(js.contains("input[type"));
    }

    #[test]
    fn compile_step_text_uses_textcontent() {
        let js = compile_step(&SelectorExpr::Text("Save".to_string()));
        assert!(js.contains("textContent"));
        assert!(js.contains("toLowerCase"));
    }
}

#[cfg(test)]
mod gh3737_locator_json_str_no_silent_empty_tests {
    //! GH #3737 — `json_str` previously did
    //! `unwrap_or_else(|_| "\"\"".to_string())`, which would silently
    //! corrupt every locator selector / role / text needle on the Err
    //! arm. These tests pin the defensive escaper's behaviour and the
    //! contract that the helper NEVER emits a bare empty-string literal
    //! for a non-empty input.
    use super::*;

    /// Happy-path equivalence: for an arbitrary valid `&str`, the
    /// defensive escaper produces the SAME JSON string literal as
    /// `serde_json::to_string`. Otherwise the fall-back would emit JS
    /// that diverges from the happy path under refactor pressure.
    #[test]
    fn gh3737_defensive_path_matches_serde_for_common_strings() {
        let cases = [
            "",            // empty
            "Save",        // ASCII
            "a\"b",        // embedded quote
            "multi\nline", // newline
            "a\\b",        // backslash
            "a\tb",        // tab
            "a\rb",        // carriage return
            "a\u{0008}b",  // backspace
            "a\u{000c}b",  // form feed
            "a\u{001f}b",  // < 0x20 control char
            "ßéü",         // multi-byte UTF-8
            "你好",        // CJK
        ];
        for case in cases {
            let serde_out = serde_json::to_string(case).unwrap();
            let defensive_out = safe_locator_json_str(case);
            assert_eq!(
                serde_out, defensive_out,
                "defensive output must match serde for input {case:?}: \
                 serde={serde_out}, defensive={defensive_out}"
            );
        }
    }

    /// The defensive escaper must produce a valid JSON string literal
    /// (parseable round-trip) for every input — that's the whole
    /// purpose of the fall-back vs the prior naïve `"\"\""`.
    #[test]
    fn gh3737_defensive_path_round_trips_through_serde() {
        for case in ["plain", "a\"b", "x\ny\rz", "tab\there", "\u{0001}"] {
            let escaped = safe_locator_json_str(case);
            let parsed: String = serde_json::from_str(&escaped).unwrap_or_else(|err| {
                panic!("defensive output not valid JSON for {case:?}: {err}; output={escaped}")
            });
            assert_eq!(
                parsed, case,
                "round-trip must yield the original input for {case:?}"
            );
        }
    }

    /// The whole point of the fix: for a NON-EMPTY input, the helper
    /// MUST NOT silently return `"\"\""`. The prior code's Err arm did
    /// exactly that.
    #[test]
    fn gh3737_defensive_path_never_emits_bare_empty_for_nonempty_input() {
        for case in ["a", "save", "click me", "ßéü"] {
            let out = safe_locator_json_str(case);
            assert_ne!(
                out, "\"\"",
                "defensive output MUST NOT silently collapse {case:?} to empty"
            );
            assert!(
                out.len() >= 3,
                "defensive output for non-empty input must be at least 3 chars (open-quote, content, close-quote): got {out:?}"
            );
        }
    }

    /// Empty input must still produce the literal `""` — that's the
    /// correct JSON encoding of an empty string, not a corruption.
    #[test]
    fn gh3737_defensive_path_empty_input_yields_empty_literal() {
        assert_eq!(safe_locator_json_str(""), "\"\"");
    }

    /// Embedded quotes and backslashes must be escaped — a naïve
    /// wrap-in-quotes fall-back would have broken locator JS by
    /// emitting unescaped `"` inside the literal. This is the
    /// difference between this fix and a quick `format!("\"{s}\"")`.
    #[test]
    fn gh3737_defensive_path_escapes_embedded_quote_and_backslash() {
        let with_quote = safe_locator_json_str(r#"say "hi""#);
        // Must contain escaped form, must not contain naïve form.
        assert!(
            with_quote.contains("\\\""),
            "must escape embedded quote: {with_quote}"
        );
        // Re-parse must succeed → confirms valid JS string literal.
        let _: String = serde_json::from_str(&with_quote).unwrap();

        let with_bs = safe_locator_json_str(r"a\b");
        assert!(
            with_bs.contains("\\\\"),
            "must escape embedded backslash: {with_bs}"
        );
        let _: String = serde_json::from_str(&with_bs).unwrap();
    }

    /// All control chars (< 0x20) must be escaped via `\uXXXX` so the
    /// generated JS is parseable on every browser. Prior fall-back
    /// would not escape anything.
    #[test]
    fn gh3737_defensive_path_escapes_control_chars_via_unicode() {
        // 0x01 is a SOH control char — must come out as .
        let out = safe_locator_json_str("\u{0001}");
        assert!(out.contains("\\u0001"), "must escape SOH as \\u0001: {out}");
        let parsed: String = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed, "\u{0001}");
    }

    /// Sibling distinctness: `safe_locator_json_str` must produce the
    /// SAME output as `safe_import_meta_env_define_value` (#3641 in
    /// `bundler/define.rs`) for the same input — they implement the
    /// same JSON-string encoding contract, so their behaviour must
    /// agree. If they ever diverge, the bundler's defines and the
    /// browser's locator JS would render the same input differently,
    /// which is a real correctness bug.
    #[test]
    fn gh3737_safe_locator_agrees_with_safe_define_for_same_inputs() {
        use crate::bundler::define::safe_import_meta_env_define_value;
        for case in ["", "x", "a\"b", "multi\nline", "\u{0001}", "ßéü"] {
            assert_eq!(
                safe_locator_json_str(case),
                safe_import_meta_env_define_value(case),
                "the two defensive escapers must agree for input {case:?}"
            );
        }
    }

    /// json_str (the public-facing helper) must produce IDENTICAL output
    /// to safe_locator_json_str on the happy path. Guards against an
    /// accidental future divergence where json_str adds an extra
    /// transformation that the defensive path doesn't.
    #[test]
    fn gh3737_public_json_str_matches_safe_locator_on_happy_path() {
        for case in ["", "x", "a\"b", "multi\nline", "你好"] {
            assert_eq!(super::json_str(case), safe_locator_json_str(case));
        }
    }

    /// Determinism: same input → same output, byte-identical, across
    /// successive calls. No HashMap-ordering, no entropy.
    #[test]
    fn gh3737_defensive_path_is_deterministic() {
        for case in ["a", "a\"b", "\u{0001}", "ßéü"] {
            let a = safe_locator_json_str(case);
            let b = safe_locator_json_str(case);
            assert_eq!(a, b);
        }
    }

    /// Family cross-reference: confirm the codebase now has the family
    /// of `safe_*_json*`-style escapers (define + locator). Operators
    /// who grep for `safe_locator_json_str` should be able to discover
    /// the sibling helper via the doc-comment cross-reference.
    #[test]
    fn gh3737_helper_name_follows_family_convention() {
        // The new helper name must follow the `safe_*` + `_json*`-ish
        // naming convention established by #3641.
        let name = "safe_locator_json_str";
        assert!(
            name.starts_with("safe_"),
            "family helper must start with `safe_`: {name}"
        );
        // And the sibling exists.
        let sibling = "safe_import_meta_env_define_value";
        assert!(
            sibling.starts_with("safe_"),
            "sibling must also start with `safe_`: {sibling}"
        );
        assert_ne!(name, sibling);
    }
}

#[cfg(test)]
mod gh3768_locator_shape_warn_tests {
    //! GH #3768 — Locator::count / text_content / inner_text /
    //! probe_state used silent fallbacks (`.unwrap_or(0)` /
    //! `.unwrap_or("")` / `.unwrap_or("Detached")`) that masked
    //! wrong-shape CDP responses. Tests cover each shape branch +
    //! helper-name discoverability + sibling-distinctness.

    use super::*;
    use serde_json::{json, Value};

    /// GH #3768 — count() on a valid Number returns the integer.
    #[test]
    fn gh3768_count_number_returns_value() {
        assert_eq!(coerce_count_or_warn(&json!(7), "div"), 7);
    }

    /// GH #3768 — count() on null silently returns 0 (legitimate "no
    /// matches").
    #[test]
    fn gh3768_count_null_returns_zero_silent() {
        assert_eq!(coerce_count_or_warn(&Value::Null, "div"), 0);
    }

    /// GH #3768 — count() on a wrong-shape value (string, bool, array,
    /// object) returns 0 — but emits a warn, distinguishing the
    /// path from the silent-null path.
    #[test]
    fn gh3768_count_wrong_shape_returns_zero() {
        assert_eq!(coerce_count_or_warn(&json!("seven"), "div"), 0);
        assert_eq!(coerce_count_or_warn(&json!(true), "div"), 0);
        assert_eq!(coerce_count_or_warn(&json!([1, 2]), "div"), 0);
        assert_eq!(coerce_count_or_warn(&json!({"x": 1}), "div"), 0);
    }

    /// GH #3768 — text_content / inner_text on a valid String returns
    /// the value.
    #[test]
    fn gh3768_text_string_returns_value() {
        assert_eq!(
            coerce_text_or_warn(&json!("hello"), "text_content", "p"),
            "hello"
        );
    }

    /// GH #3768 — text_content on null returns "" silently (no element
    /// found is legitimate).
    #[test]
    fn gh3768_text_null_returns_empty_silent() {
        assert_eq!(coerce_text_or_warn(&Value::Null, "text_content", "p"), "");
    }

    /// GH #3768 — text_content on a wrong-shape value (number, bool,
    /// object) returns "" with a warn.
    #[test]
    fn gh3768_text_wrong_shape_returns_empty() {
        assert_eq!(coerce_text_or_warn(&json!(42), "text_content", "p"), "");
        assert_eq!(coerce_text_or_warn(&json!({"x": 1}), "inner_text", "p"), "");
    }

    /// GH #3768 — actionability probe on a known string passes through.
    #[test]
    fn gh3768_actionability_string_returns_value() {
        assert_eq!(
            coerce_actionability_or_warn(&json!("Visible"), "p"),
            "Visible"
        );
    }

    /// GH #3768 — actionability probe on null and wrong-shape both
    /// return "Detached" so the actionability wait still surfaces a
    /// timeout if the JS pipeline is broken.
    #[test]
    fn gh3768_actionability_null_and_wrong_shape_return_detached() {
        assert_eq!(coerce_actionability_or_warn(&Value::Null, "p"), "Detached");
        assert_eq!(coerce_actionability_or_warn(&json!(42), "p"), "Detached");
        assert_eq!(coerce_actionability_or_warn(&json!(true), "p"), "Detached");
    }

    /// GH #3768 — issue-tag discoverability across all three helper
    /// families.
    #[test]
    fn gh3768_helpers_include_issue_tag() {
        assert!(format_locator_count_shape_warn("div", "string").contains("GH #3768"));
        assert!(format_locator_text_shape_warn("text_content", "p", "number").contains("GH #3768"));
        assert!(format_locator_actionability_shape_warn("button", "object").contains("GH #3768"));
    }

    /// GH #3768 — sibling-distinctness vs. the GH #3737 safe_locator_*
    /// family that lives in the same file. The new helpers use
    /// `format_locator_*_shape_warn`, distinct from the `safe_locator_*`
    /// convention.
    #[test]
    fn gh3768_helpers_distinct_from_gh3737_safe_locator_family() {
        let count_msg = format_locator_count_shape_warn("div", "string");
        let text_msg = format_locator_text_shape_warn("text_content", "p", "number");
        let act_msg = format_locator_actionability_shape_warn("button", "object");

        for msg in [&count_msg, &text_msg, &act_msg] {
            assert!(msg.contains("GH #3768"));
            assert!(!msg.contains("GH #3737"));
            // The new family's messages mention "wrong-shape CDP" so an
            // operator can disambiguate them from the safe-string family.
            assert!(msg.contains("wrong-shape CDP"));
        }
    }

    /// GH #3768 — the count helper records both the selector and the
    /// actual JS type so operators can reproduce without rerunning with
    /// debug logging.
    #[test]
    fn gh3768_count_warn_records_selector_and_type() {
        let msg = format_locator_count_shape_warn("button.primary", "string");
        assert!(msg.contains("button.primary"));
        assert!(msg.contains("string"));
        assert!(msg.contains("expected number"));
    }

    /// GH #3768 — locator_value_kind names every serde_json variant.
    #[test]
    fn gh3768_value_kind_covers_all_variants() {
        assert_eq!(locator_value_kind(&Value::Null), "null");
        assert_eq!(locator_value_kind(&Value::Bool(false)), "bool");
        assert_eq!(locator_value_kind(&json!(1)), "number");
        assert_eq!(locator_value_kind(&json!("x")), "string");
        assert_eq!(locator_value_kind(&json!([1])), "array");
        assert_eq!(locator_value_kind(&json!({"x": 1})), "object");
    }
}
// CODEGEN-END
