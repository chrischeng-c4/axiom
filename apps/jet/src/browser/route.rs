// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
// CODEGEN-BEGIN
//! Declarative page-level network interception (#1911).
//!
//! v1 is declarative-only: `page.route(pattern, descriptor)` registers a
//! pattern + outcome descriptor over the page-binding wire; matching and
//! CDP command selection both happen here, entirely on the Rust side.
//! There is no Rust→JS callback channel for route handlers — the
//! page-binding wire protocol stays JS-asks/Rust-answers, and Playwright's
//! function-handler form (`page.route(pattern, async route => {...})`) is
//! rejected by the JS façade before it ever reaches this module.
//!
//! This module is pure — no CDP session, no async I/O — so every function
//! here is directly unit-testable. `Page` (see `browser::page`) owns the
//! per-page `Vec<RouteEntry>` and the CDP `Fetch.enable` lifecycle; the
//! `Fetch.requestPaused` event pump (`test_runner::worker`) calls
//! [`decide_fetch_action`] then [`fetch_action_to_cdp`] to resolve each
//! paused request.

use serde_json::Value;

/// One registered route: a compiled URL pattern plus the outcome to apply
/// when a paused request matches it. Stored in registration order —
/// [`decide_fetch_action`] walks the list newest-first so the
/// last-registered overlapping route wins (Playwright precedence).
#[derive(Debug)]
pub struct RouteEntry {
    pub pattern: CompiledPattern,
    pub descriptor: RouteDescriptor,
}

/// A compiled Playwright-style URL-glob pattern. Built once per
/// `RouteEntry` at `page.route()` time, matched against the full request
/// URL on every `Fetch.requestPaused` event.
///
/// Built with `globset::GlobBuilder::literal_separator(true)` — the
/// codebase's other `globset` call sites (`test_runner::discovery`,
/// `css::tailwind::scanner`, ...) use the plain `Glob::new` constructor,
/// whose DEFAULT is `literal_separator(false)` (a bare `*` crosses `/`).
/// That default is wrong for URL routing: Playwright's own examples
/// (`**/*.png`, `**/api/users*`) rely on `*` stopping at a path segment
/// boundary while `**` crosses it, which is exactly what
/// `literal_separator(true)` compiles to:
///   - `?`  → `[^/]`         (single char, excludes `/`)
///   - `*`  → `[^/]*`        (zero or more, excludes `/`)
///   - `**/` (pattern start) → `(?:/?|.*/)`  (crosses `/`, optional prefix)
///   - `/**/` (mid-pattern)  → `(?:/|/.*/)`  (crosses `/`)
/// A pattern with no glob metacharacters (e.g. a literal URL) compiles to
/// an anchored literal match — i.e. exact-URL matching falls out of the
/// same glob compiler, no separate code path needed.
///
/// Known divergence from real Playwright: Playwright's own `?` token
/// matches any single character *including* `/`, whereas globset's
/// `literal_separator` mode excludes `/` for `?` (matching typical glob
/// tooling instead). `page.route()` patterns in practice overwhelmingly
/// use `*`/`**`, not bare `?`, so this is a documented, tested
/// approximation rather than a byte-for-byte Playwright glob-to-regex
/// reimplementation.
#[derive(Debug)]
pub struct CompiledPattern {
    source: String,
    matcher: globset::GlobMatcher,
}

impl CompiledPattern {
    pub fn compile(pattern: &str) -> Result<Self, String> {
        let glob = globset::GlobBuilder::new(pattern)
            .literal_separator(true)
            .build()
            .map_err(|e| format!("invalid route pattern '{pattern}': {e}"))?;
        Ok(Self {
            source: pattern.to_string(),
            matcher: glob.compile_matcher(),
        })
    }

    /// The original pattern text, as registered. `page.unroute(pattern)`
    /// removes routes by comparing against this — text equality, not
    /// glob-equivalence — matching Playwright semantics.
    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn is_match(&self, url: &str) -> bool {
        self.matcher.is_match(url)
    }
}

/// The three v1 outcome shapes a route descriptor can take. Mirrors the
/// JS-facing `{ fulfill: {...} }` / `{ abort: true|"reason" }` /
/// `{ continue: true }` object shapes 1:1 — see [`parse_route_descriptor`].
#[derive(Debug, Clone, PartialEq)]
pub enum RouteDescriptor {
    Fulfill(FulfillSpec),
    Abort(ErrorReason),
    Continue,
}

/// A resolved `route.fulfill({...})` outcome. `body` and `content_type`
/// are already resolved by [`parse_route_descriptor`] — a `json` input
/// field has been serialized into `body` and defaulted `content_type` to
/// `application/json` by the time a `FulfillSpec` exists.
#[derive(Debug, Clone, PartialEq)]
pub struct FulfillSpec {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub content_type: Option<String>,
    pub body: String,
}

/// CDP `Network.ErrorReason` — the enum `Fetch.failRequest` expects. Names
/// are PascalCase on the wire (Chromium's own casing), while Playwright's
/// public `route.abort(errorCode)` argument uses lowercase names (e.g.
/// `"connectionrefused"`). [`ErrorReason::from_playwright_name`] maps
/// between the two, defaulting unrecognized names to `Failed` — the same
/// default CDP documents for a bare `route.abort()` call with no reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorReason {
    Failed,
    Aborted,
    TimedOut,
    AccessDenied,
    ConnectionClosed,
    ConnectionReset,
    ConnectionRefused,
    ConnectionAborted,
    ConnectionFailed,
    NameNotResolved,
    InternetDisconnected,
    AddressUnreachable,
    BlockedByClient,
    BlockedByResponse,
}

impl ErrorReason {
    /// CDP `Network.ErrorReason` wire value (PascalCase).
    pub fn as_cdp_str(&self) -> &'static str {
        match self {
            Self::Failed => "Failed",
            Self::Aborted => "Aborted",
            Self::TimedOut => "TimedOut",
            Self::AccessDenied => "AccessDenied",
            Self::ConnectionClosed => "ConnectionClosed",
            Self::ConnectionReset => "ConnectionReset",
            Self::ConnectionRefused => "ConnectionRefused",
            Self::ConnectionAborted => "ConnectionAborted",
            Self::ConnectionFailed => "ConnectionFailed",
            Self::NameNotResolved => "NameNotResolved",
            Self::InternetDisconnected => "InternetDisconnected",
            Self::AddressUnreachable => "AddressUnreachable",
            Self::BlockedByClient => "BlockedByClient",
            Self::BlockedByResponse => "BlockedByResponse",
        }
    }

    /// Map a Playwright-style abort reason name (case-insensitive, e.g.
    /// `"failed"`, `"timedout"`, `"connectionrefused"`) to the CDP enum.
    /// Unrecognized names (including a bare `true`, which has no name at
    /// all) fall back to `Failed`.
    pub fn from_playwright_name(name: &str) -> Self {
        match name.to_ascii_lowercase().as_str() {
            "aborted" => Self::Aborted,
            "timedout" => Self::TimedOut,
            "accessdenied" => Self::AccessDenied,
            "connectionclosed" => Self::ConnectionClosed,
            "connectionreset" => Self::ConnectionReset,
            "connectionrefused" => Self::ConnectionRefused,
            "connectionaborted" => Self::ConnectionAborted,
            "connectionfailed" => Self::ConnectionFailed,
            "namenotresolved" => Self::NameNotResolved,
            "internetdisconnected" => Self::InternetDisconnected,
            "addressunreachable" => Self::AddressUnreachable,
            "blockedbyclient" => Self::BlockedByClient,
            "blockedbyresponse" => Self::BlockedByResponse,
            _ => Self::Failed,
        }
    }
}

/// Parse a raw wire-protocol descriptor `Value` (the JS façade sends the
/// `{ fulfill: {...} } | { abort: ... } | { continue: true }` object
/// as-is) into a typed [`RouteDescriptor`]. Exactly one of the three keys
/// must be present.
pub fn parse_route_descriptor(value: &Value) -> Result<RouteDescriptor, String> {
    let obj = value.as_object().ok_or_else(|| {
        "route descriptor must be an object with one of: fulfill, abort, continue".to_string()
    })?;

    let has_fulfill = obj.contains_key("fulfill");
    let has_abort = obj.contains_key("abort");
    let has_continue = obj.contains_key("continue");

    match (has_fulfill, has_abort, has_continue) {
        (true, false, false) => parse_fulfill(&obj["fulfill"]).map(RouteDescriptor::Fulfill),
        (false, true, false) => Ok(RouteDescriptor::Abort(parse_abort(&obj["abort"]))),
        (false, false, true) => Ok(RouteDescriptor::Continue),
        (false, false, false) => {
            Err("route descriptor must set one of: fulfill, abort, continue".to_string())
        }
        _ => Err("route descriptor must set exactly one of: fulfill, abort, continue — got more than one".to_string()),
    }
}

fn parse_fulfill(value: &Value) -> Result<FulfillSpec, String> {
    let status = value
        .get("status")
        .and_then(Value::as_u64)
        .map(|v| v as u16)
        .unwrap_or(200);

    let mut headers: Vec<(String, String)> = Vec::new();
    if let Some(h) = value.get("headers").and_then(Value::as_object) {
        for (k, v) in h {
            if let Some(s) = v.as_str() {
                headers.push((k.clone(), s.to_string()));
            }
        }
    }
    let content_type = value
        .get("contentType")
        .and_then(Value::as_str)
        .map(str::to_string);

    // `json` is a convenience over `body`: serialize it and default the
    // content-type to application/json (an explicit `contentType` still
    // wins). Mirrors Playwright's `route.fulfill({ json })`.
    if let Some(json_val) = value.get("json") {
        let body = serde_json::to_string(json_val)
            .map_err(|e| format!("route fulfill: json field must serialize — {e}"))?;
        let content_type = content_type.or_else(|| Some("application/json".to_string()));
        return Ok(FulfillSpec {
            status,
            headers,
            content_type,
            body,
        });
    }

    let body = value
        .get("body")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    Ok(FulfillSpec {
        status,
        headers,
        content_type,
        body,
    })
}

fn parse_abort(value: &Value) -> ErrorReason {
    match value.as_str() {
        Some(reason) => ErrorReason::from_playwright_name(reason),
        None => ErrorReason::Failed,
    }
}

/// What to do with a paused request, once a route (or the lack of one)
/// has been decided. Owned/self-contained — no reference back into the
/// page's route list — so it can cross an `.await` point cheaply.
#[derive(Debug, Clone, PartialEq)]
pub enum FetchAction {
    Continue,
    Fulfill(FulfillSpec),
    Abort(ErrorReason),
}

/// Decide what to do with a paused request given the page's currently
/// registered routes, in registration order. Walks newest-first so the
/// **last-registered** matching route wins on overlapping patterns
/// (Playwright precedence: "the last registered route can always
/// override all the routes that were registered before it"). A request
/// matching no route continues untouched.
pub fn decide_fetch_action(routes: &[RouteEntry], request_url: &str) -> FetchAction {
    for entry in routes.iter().rev() {
        if entry.pattern.is_match(request_url) {
            return match &entry.descriptor {
                RouteDescriptor::Fulfill(spec) => FetchAction::Fulfill(spec.clone()),
                RouteDescriptor::Abort(reason) => FetchAction::Abort(*reason),
                RouteDescriptor::Continue => FetchAction::Continue,
            };
        }
    }
    FetchAction::Continue
}

/// Translate a decided [`FetchAction`] into the CDP Fetch-domain command
/// (method name + params) that resolves the paused request. All three
/// CDP methods key off `requestId` from the triggering
/// `Fetch.requestPaused` event. `Fetch.fulfillRequest`'s `body` must be
/// base64-encoded per the CDP schema.
pub fn fetch_action_to_cdp(action: &FetchAction, request_id: &str) -> (&'static str, Value) {
    match action {
        FetchAction::Continue => (
            "Fetch.continueRequest",
            serde_json::json!({ "requestId": request_id }),
        ),
        FetchAction::Abort(reason) => (
            "Fetch.failRequest",
            serde_json::json!({
                "requestId": request_id,
                "errorReason": reason.as_cdp_str(),
            }),
        ),
        FetchAction::Fulfill(spec) => {
            use base64::Engine;
            let mut headers: Vec<Value> = Vec::new();
            let mut has_content_type = false;
            for (name, value) in &spec.headers {
                if name.eq_ignore_ascii_case("content-type") {
                    has_content_type = true;
                }
                headers.push(serde_json::json!({ "name": name, "value": value }));
            }
            if !has_content_type {
                if let Some(ct) = &spec.content_type {
                    headers.push(serde_json::json!({ "name": "content-type", "value": ct }));
                }
            }
            let body_b64 = base64::engine::general_purpose::STANDARD.encode(spec.body.as_bytes());
            (
                "Fetch.fulfillRequest",
                serde_json::json!({
                    "requestId": request_id,
                    "responseCode": spec.status,
                    "responseHeaders": headers,
                    "body": body_b64,
                }),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── CompiledPattern / glob semantics ──────────────────────────────

    #[test]
    fn glob_star_does_not_cross_path_boundary() {
        let p = CompiledPattern::compile("**/api/users*").unwrap();
        assert!(p.is_match("http://x.test/api/users"));
        assert!(p.is_match("http://x.test/api/users?id=1"));
        // A `*` after `users` must not cross a `/` — `users/nested` has an
        // extra path segment, so this must NOT match the WI's own example
        // pattern `**/api/users*`.
        assert!(!p.is_match("http://x.test/api/users/nested"));
    }

    #[test]
    fn glob_double_star_crosses_any_prefix() {
        let p = CompiledPattern::compile("**/*.png").unwrap();
        assert!(p.is_match("http://x.test/a.png"));
        assert!(p.is_match("http://x.test/a/b/c.png"));
        assert!(p.is_match("/a/b/c.png"));
        assert!(!p.is_match("http://x.test/a.png.bak"));
        assert!(!p.is_match("http://x.test/a.jpg"));
    }

    #[test]
    fn glob_pattern_with_no_meta_chars_is_exact_match() {
        let p = CompiledPattern::compile("http://x.test/exact").unwrap();
        assert!(p.is_match("http://x.test/exact"));
        assert!(!p.is_match("http://x.test/exact/more"));
        assert!(!p.is_match("http://x.test/exactly"));
    }

    #[test]
    fn compiled_pattern_source_round_trips() {
        let p = CompiledPattern::compile("**/api/**").unwrap();
        assert_eq!(p.source(), "**/api/**");
    }

    #[test]
    fn invalid_glob_pattern_reports_source_in_error() {
        // Unbalanced class bracket — globset rejects this at compile time.
        let err = CompiledPattern::compile("**/[abc").unwrap_err();
        assert!(
            err.contains("**/[abc"),
            "error should name the pattern: {err}"
        );
    }

    // ── parse_route_descriptor ─────────────────────────────────────────

    #[test]
    fn parse_fulfill_defaults_status_200_and_empty_body() {
        let d = parse_route_descriptor(&serde_json::json!({ "fulfill": {} })).unwrap();
        assert_eq!(
            d,
            RouteDescriptor::Fulfill(FulfillSpec {
                status: 200,
                headers: vec![],
                content_type: None,
                body: String::new(),
            })
        );
    }

    #[test]
    fn parse_fulfill_reads_status_headers_body_content_type() {
        let d = parse_route_descriptor(&serde_json::json!({
            "fulfill": {
                "status": 201,
                "headers": { "x-custom": "1" },
                "contentType": "text/plain",
                "body": "hello",
            }
        }))
        .unwrap();
        assert_eq!(
            d,
            RouteDescriptor::Fulfill(FulfillSpec {
                status: 201,
                headers: vec![("x-custom".to_string(), "1".to_string())],
                content_type: Some("text/plain".to_string()),
                body: "hello".to_string(),
            })
        );
    }

    #[test]
    fn parse_fulfill_json_field_serializes_body_and_defaults_content_type() {
        let d =
            parse_route_descriptor(&serde_json::json!({ "fulfill": { "json": { "ok": true } } }))
                .unwrap();
        let RouteDescriptor::Fulfill(spec) = d else {
            panic!("expected Fulfill");
        };
        assert_eq!(spec.body, r#"{"ok":true}"#);
        assert_eq!(spec.content_type.as_deref(), Some("application/json"));
    }

    #[test]
    fn parse_fulfill_json_field_respects_explicit_content_type() {
        let d = parse_route_descriptor(&serde_json::json!({
            "fulfill": { "json": { "ok": true }, "contentType": "application/vnd.api+json" }
        }))
        .unwrap();
        let RouteDescriptor::Fulfill(spec) = d else {
            panic!("expected Fulfill");
        };
        assert_eq!(
            spec.content_type.as_deref(),
            Some("application/vnd.api+json")
        );
    }

    #[test]
    fn parse_abort_true_maps_to_failed() {
        let d = parse_route_descriptor(&serde_json::json!({ "abort": true })).unwrap();
        assert_eq!(d, RouteDescriptor::Abort(ErrorReason::Failed));
    }

    #[test]
    fn parse_abort_named_reason_maps_case_insensitively() {
        let d =
            parse_route_descriptor(&serde_json::json!({ "abort": "ConnectionRefused" })).unwrap();
        assert_eq!(d, RouteDescriptor::Abort(ErrorReason::ConnectionRefused));
    }

    #[test]
    fn parse_abort_unknown_reason_falls_back_to_failed() {
        let d =
            parse_route_descriptor(&serde_json::json!({ "abort": "not-a-real-reason" })).unwrap();
        assert_eq!(d, RouteDescriptor::Abort(ErrorReason::Failed));
    }

    #[test]
    fn parse_continue_true() {
        let d = parse_route_descriptor(&serde_json::json!({ "continue": true })).unwrap();
        assert_eq!(d, RouteDescriptor::Continue);
    }

    #[test]
    fn parse_descriptor_rejects_non_object() {
        let err = parse_route_descriptor(&serde_json::json!("not-an-object")).unwrap_err();
        assert!(err.contains("object"), "{err}");
    }

    #[test]
    fn parse_descriptor_rejects_empty_object() {
        let err = parse_route_descriptor(&serde_json::json!({})).unwrap_err();
        assert!(
            err.contains("fulfill, abort, continue"),
            "must name the three valid forms: {err}"
        );
    }

    #[test]
    fn parse_descriptor_rejects_multiple_keys() {
        let err = parse_route_descriptor(&serde_json::json!({ "fulfill": {}, "abort": true }))
            .unwrap_err();
        assert!(err.contains("exactly one"), "{err}");
    }

    // ── decide_fetch_action precedence ─────────────────────────────────

    fn entry(pattern: &str, descriptor: RouteDescriptor) -> RouteEntry {
        RouteEntry {
            pattern: CompiledPattern::compile(pattern).unwrap(),
            descriptor,
        }
    }

    #[test]
    fn no_routes_continues() {
        let action = decide_fetch_action(&[], "http://x.test/anything");
        assert_eq!(action, FetchAction::Continue);
    }

    #[test]
    fn no_matching_route_continues() {
        let routes = vec![entry("**/api/**", RouteDescriptor::Continue)];
        let action = decide_fetch_action(&routes, "http://x.test/other");
        assert_eq!(action, FetchAction::Continue);
    }

    #[test]
    fn single_matching_route_applies() {
        let spec = FulfillSpec {
            status: 200,
            headers: vec![],
            content_type: None,
            body: "hi".to_string(),
        };
        let routes = vec![entry("**/api/**", RouteDescriptor::Fulfill(spec.clone()))];
        let action = decide_fetch_action(&routes, "http://x.test/api/x");
        assert_eq!(action, FetchAction::Fulfill(spec));
    }

    #[test]
    fn last_registered_overlapping_route_wins() {
        // WI #1911 architecture decision: Playwright precedence — the
        // LAST registered route wins on overlap, not the first.
        let first = FulfillSpec {
            status: 200,
            headers: vec![],
            content_type: None,
            body: "FIRST".to_string(),
        };
        let second = FulfillSpec {
            status: 200,
            headers: vec![],
            content_type: None,
            body: "SECOND".to_string(),
        };
        let routes = vec![
            entry("**/api/**", RouteDescriptor::Fulfill(first)),
            entry("**/api/**", RouteDescriptor::Fulfill(second.clone())),
        ];
        let action = decide_fetch_action(&routes, "http://x.test/api/x");
        assert_eq!(action, FetchAction::Fulfill(second));
    }

    #[test]
    fn later_continue_route_carves_out_exception_from_earlier_broad_block() {
        // A later, more specific `{ continue: true }` route can override
        // an earlier broad abort — exactly the use case `route.continue()`
        // exists for in Playwright even though it's also the default.
        let routes = vec![
            entry("**/api/**", RouteDescriptor::Abort(ErrorReason::Failed)),
            entry("**/api/health", RouteDescriptor::Continue),
        ];
        let action = decide_fetch_action(&routes, "http://x.test/api/health");
        assert_eq!(action, FetchAction::Continue);

        let blocked = decide_fetch_action(&routes, "http://x.test/api/orders");
        assert_eq!(blocked, FetchAction::Abort(ErrorReason::Failed));
    }

    // ── fetch_action_to_cdp ─────────────────────────────────────────────

    #[test]
    fn continue_action_maps_to_continue_request() {
        let (method, params) = fetch_action_to_cdp(&FetchAction::Continue, "req-1");
        assert_eq!(method, "Fetch.continueRequest");
        assert_eq!(params["requestId"], "req-1");
    }

    #[test]
    fn abort_action_maps_to_fail_request_with_cdp_error_reason() {
        let (method, params) =
            fetch_action_to_cdp(&FetchAction::Abort(ErrorReason::ConnectionRefused), "req-2");
        assert_eq!(method, "Fetch.failRequest");
        assert_eq!(params["requestId"], "req-2");
        assert_eq!(params["errorReason"], "ConnectionRefused");
    }

    #[test]
    fn fulfill_action_maps_to_fulfill_request_with_base64_body() {
        use base64::Engine;
        let spec = FulfillSpec {
            status: 201,
            headers: vec![("x-custom".to_string(), "yes".to_string())],
            content_type: Some("application/json".to_string()),
            body: r#"{"ok":true}"#.to_string(),
        };
        let (method, params) = fetch_action_to_cdp(&FetchAction::Fulfill(spec), "req-3");
        assert_eq!(method, "Fetch.fulfillRequest");
        assert_eq!(params["requestId"], "req-3");
        assert_eq!(params["responseCode"], 201);
        let headers = params["responseHeaders"].as_array().unwrap();
        assert!(headers
            .iter()
            .any(|h| h["name"] == "x-custom" && h["value"] == "yes"));
        assert!(headers
            .iter()
            .any(|h| h["name"] == "content-type" && h["value"] == "application/json"));
        let body = params["body"].as_str().unwrap();
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(body)
            .unwrap();
        assert_eq!(decoded, br#"{"ok":true}"#);
    }

    #[test]
    fn fulfill_action_does_not_duplicate_explicit_content_type_header() {
        // If the caller already set a `content-type` in `headers`
        // (any-case), the derived `content_type` (from `contentType` or a
        // `json` shortcut) must not add a second header.
        let spec = FulfillSpec {
            status: 200,
            headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
            content_type: Some("application/json".to_string()),
            body: "hi".to_string(),
        };
        let (_, params) = fetch_action_to_cdp(&FetchAction::Fulfill(spec), "req-4");
        let headers = params["responseHeaders"].as_array().unwrap();
        let ct_count = headers
            .iter()
            .filter(|h| {
                h["name"]
                    .as_str()
                    .unwrap()
                    .eq_ignore_ascii_case("content-type")
            })
            .count();
        assert_eq!(ct_count, 1, "must not duplicate content-type: {headers:?}");
        assert_eq!(headers[0]["value"], "text/plain", "explicit header wins");
    }

    #[test]
    fn fulfill_action_default_status_is_200() {
        let spec = FulfillSpec {
            status: 200,
            headers: vec![],
            content_type: None,
            body: String::new(),
        };
        let (_, params) = fetch_action_to_cdp(&FetchAction::Fulfill(spec), "req-5");
        assert_eq!(params["responseCode"], 200);
    }

    // ── ErrorReason mapping ──────────────────────────────────────────────

    #[test]
    fn error_reason_playwright_names_are_case_insensitive() {
        assert_eq!(
            ErrorReason::from_playwright_name("timedout"),
            ErrorReason::TimedOut
        );
        assert_eq!(
            ErrorReason::from_playwright_name("TIMEDOUT"),
            ErrorReason::TimedOut
        );
        assert_eq!(
            ErrorReason::from_playwright_name("TimedOut"),
            ErrorReason::TimedOut
        );
    }

    #[test]
    fn error_reason_all_variants_round_trip_through_cdp_str() {
        // Pin the exact PascalCase wire values CDP's Network.ErrorReason
        // enum expects — a typo here silently breaks every abort() call.
        let cases = [
            (ErrorReason::Failed, "Failed"),
            (ErrorReason::Aborted, "Aborted"),
            (ErrorReason::TimedOut, "TimedOut"),
            (ErrorReason::AccessDenied, "AccessDenied"),
            (ErrorReason::ConnectionClosed, "ConnectionClosed"),
            (ErrorReason::ConnectionReset, "ConnectionReset"),
            (ErrorReason::ConnectionRefused, "ConnectionRefused"),
            (ErrorReason::ConnectionAborted, "ConnectionAborted"),
            (ErrorReason::ConnectionFailed, "ConnectionFailed"),
            (ErrorReason::NameNotResolved, "NameNotResolved"),
            (ErrorReason::InternetDisconnected, "InternetDisconnected"),
            (ErrorReason::AddressUnreachable, "AddressUnreachable"),
            (ErrorReason::BlockedByClient, "BlockedByClient"),
            (ErrorReason::BlockedByResponse, "BlockedByResponse"),
        ];
        for (variant, expected) in cases {
            assert_eq!(variant.as_cdp_str(), expected);
        }
    }

    #[test]
    fn error_reason_unknown_name_defaults_to_failed() {
        assert_eq!(
            ErrorReason::from_playwright_name("totally-made-up"),
            ErrorReason::Failed
        );
    }
}
// CODEGEN-END
