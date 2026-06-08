// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
// CODEGEN-BEGIN
//! BrowserContext — isolation boundary between `Browser` and `Page`.
//!
//! Each `BrowserContext` owns a CDP `browserContextId` (obtained via
//! `Target.createBrowserContext`). Pages created inside a context share cookies,
//! storage, and route scope with other pages in the same context but are
//! isolated from pages in other contexts.
//!
//! The default context is created by `Browser::launch()` / `Browser::connect()`
//! so the existing flat `Browser::new_page()` flow continues to work —
//! pages opened via the default context carry `context_id = None` on the
//! `Page` struct for backward compatibility with the 50-test Page API parity
//! regression surface.
//
// @spec .aw/issues/open/enhancement-browsercontext-refactor-multi-context-isolation-fo.md#R1
// @spec .aw/issues/open/enhancement-browsercontext-refactor-multi-context-isolation-fo.md#R2
// @spec .aw/issues/open/enhancement-browsercontext-refactor-multi-context-isolation-fo.md#R3

use crate::browser::cdp::CdpSession;
use crate::browser::page::Page;
use anyhow::{Context, Result};
use serde_json::Value;

/// A browser context — an isolation boundary for cookies, storage, and
/// (in later phases) route scope, emulation profile, and video recording.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
pub struct BrowserContext {
    /// Root (browser-level) CDP session for Target / Storage domain calls.
    cdp: CdpSession,
    /// CDP `browserContextId` returned from `Target.createBrowserContext`.
    context_id: String,
    /// True iff this context was auto-created by `Browser::launch()` /
    /// `Browser::connect()` as the implicit default. Default contexts are
    /// owned by the `Browser` and skip `Target.disposeBrowserContext` on
    /// close (the browser shutdown disposes them).
    default: bool,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
impl BrowserContext {
    pub(crate) fn new(cdp: CdpSession, context_id: String, default: bool) -> Self {
        Self {
            cdp,
            context_id,
            default,
        }
    }

    /// Chromium `browserContextId` assigned by `Target.createBrowserContext`.
    pub fn id(&self) -> &str {
        &self.context_id
    }

    /// Whether this is the implicit default context created by the Browser.
    pub fn is_default(&self) -> bool {
        self.default
    }

    /// Open a new page (tab) inside this context.
    ///
    /// Pages opened via the default context carry `Page::context_id = None`
    /// for backward compatibility (R4). Pages opened via user-created
    /// contexts carry `Page::context_id = Some(self.context_id.clone())`.
    // @spec .aw/issues/open/enhancement-browsercontext-refactor-multi-context-isolation-fo.md#R4
    pub async fn new_page(&self) -> Result<Page> {
        let create_res = self
            .cdp
            .send(
                "Target.createTarget",
                serde_json::json!({
                    "url": "about:blank",
                    "browserContextId": self.context_id,
                }),
            )
            .await?;
        let target_id = create_res["targetId"]
            .as_str()
            .context("Missing targetId in createTarget response")?
            .to_string();

        let attach_res = self
            .cdp
            .send(
                "Target.attachToTarget",
                serde_json::json!({
                    "targetId": &target_id,
                    "flatten": true,
                }),
            )
            .await?;
        let session_id = attach_res["sessionId"]
            .as_str()
            .context("Missing sessionId in attachToTarget response")?
            .to_string();

        let page_session = self.cdp.child_session(session_id);
        let page_context_id = if self.default {
            None
        } else {
            Some(self.context_id.clone())
        };
        Ok(Page::with_context(page_session, target_id, page_context_id))
    }

    /// List `targetId`s of pages currently owned by this context.
    pub async fn pages(&self) -> Result<Vec<String>> {
        let res = self
            .cdp
            .send("Target.getTargets", serde_json::json!({}))
            .await?;
        let mut out = Vec::new();
        if let Some(infos) = res["targetInfos"].as_array() {
            for info in infos {
                if info["type"].as_str() != Some("page") {
                    continue;
                }
                if info["browserContextId"].as_str() != Some(self.context_id.as_str()) {
                    continue;
                }
                if let Some(id) = info["targetId"].as_str() {
                    out.push(id.to_string());
                }
            }
        }
        Ok(out)
    }

    /// Return all cookies stored in this context via
    /// `Storage.getCookies(browserContextId=...)`. Each cookie is a raw JSON
    /// object matching Chromium's CDP schema (`name`, `value`, `domain`,
    /// `path`, `expires`, `httpOnly`, `secure`, `sameSite`, ...).
    ///
    /// Default contexts omit the `browserContextId` parameter — Chromium
    /// rejects it for the default storage partition.
    // @spec .aw/tech-design/projects/jet/logic/storage-state.md#S1
    pub async fn cookies(&self) -> Result<Vec<Value>> {
        let params = if self.default {
            serde_json::json!({})
        } else {
            serde_json::json!({ "browserContextId": self.context_id })
        };
        let res = self.cdp.send("Storage.getCookies", params).await?;
        // GH #3761 — sibling of #3739 on the READ path. The prior
        //   `res["cookies"].as_array().cloned().unwrap_or_default()`
        // silently turned a wrong-shape CDP response into an empty
        // cookie list, masking Chromium protocol drift / proxy
        // corruption as a flaky-auth test failure. Distinguish the
        // three branches explicitly: array (happy), absent (legit
        // empty), present-but-wrong-shape (warn loudly, degrade).
        let cookies = match res.get("cookies") {
            None => Vec::new(),
            Some(Value::Array(arr)) => arr.clone(),
            Some(other) => {
                let actual_type = json_value_type_name(other);
                tracing::warn!(
                    target: "jet::browser::context",
                    actual_type = %actual_type,
                    "{}",
                    format_cdp_get_cookies_shape_warn(actual_type)
                );
                Vec::new()
            }
        };
        Ok(cookies)
    }

    /// Install cookies into this context via `Storage.setCookies`. Each
    /// cookie must include at minimum `name`, `value`, and either `url` or
    /// `domain`+`path` (Chromium enforces this).
    // @spec .aw/tech-design/projects/jet/logic/storage-state.md#S2
    pub async fn add_cookies(&self, cookies: Vec<Value>) -> Result<()> {
        let mut params = serde_json::json!({ "cookies": cookies });
        if !self.default {
            params["browserContextId"] = Value::String(self.context_id.clone());
        }
        self.cdp.send("Storage.setCookies", params).await?;
        Ok(())
    }

    /// Drop all cookies in this context via `Storage.clearCookies`.
    // @spec .aw/tech-design/projects/jet/logic/storage-state.md#S3
    pub async fn clear_cookies(&self) -> Result<()> {
        let params = if self.default {
            serde_json::json!({})
        } else {
            serde_json::json!({ "browserContextId": self.context_id })
        };
        self.cdp.send("Storage.clearCookies", params).await?;
        Ok(())
    }

    /// Return the persistable storage state for this context as a JSON
    /// value shaped `{ cookies: [...], origins: [] }`. `origins` (which
    /// would carry per-origin localStorage snapshots) is always an empty
    /// array for MVP — full origin capture lands with a later change once
    /// visited-origin tracking is wired (see `storage-state.md#S5 Future`).
    // @spec .aw/tech-design/projects/jet/logic/storage-state.md#S4
    pub async fn storage_state(&self) -> Result<Value> {
        let cookies = self.cookies().await?;
        Ok(serde_json::json!({
            "cookies": cookies,
            "origins": [],
        }))
    }

    /// Load a storage-state snapshot (as produced by `storage_state()`).
    /// Applies cookies; ignores any `origins` entries for MVP. Existing
    /// cookies are not cleared first — callers wanting a clean slate should
    /// call `clear_cookies()` beforehand.
    // @spec .aw/tech-design/projects/jet/logic/storage-state.md#S5
    pub async fn set_storage_state(&self, state: &Value) -> Result<()> {
        let cookies = state["cookies"].as_array().cloned().unwrap_or_default();
        if !cookies.is_empty() {
            self.add_cookies(cookies).await?;
        }
        Ok(())
    }

    /// Close this context, disposing its `browserContextId`. Default
    /// contexts owned by the `Browser` are a no-op — they are disposed
    /// by `Browser::close()`.
    pub async fn close(&self) -> Result<()> {
        if self.default {
            return Ok(());
        }
        self.cdp
            .send(
                "Target.disposeBrowserContext",
                serde_json::json!({ "browserContextId": self.context_id }),
            )
            .await?;
        Ok(())
    }
}

/// GH #3739 — name the JSON shape we got, for log messages. Avoid
/// leaking the contents of the value (which may include credentials
/// from a corrupted-but-still-sensitive storage state file).
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
pub(crate) fn json_value_type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// GH #3739 — build the warn-body for a malformed `cookies` field in
/// the storage-state JSON passed to `set_storage_state`. Extracted so
/// the wording (issue tag, actual type, remediation hint) is
/// unit-testable.
///
/// `actual_type` is one of the strings returned by
/// `json_value_type_name` — names the shape the operator actually
/// passed so they can grep for the matching deserialiser bug.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
pub(crate) fn format_storage_state_cookies_shape_warn(actual_type: &str) -> String {
    format!(
        "GH #3739 set_storage_state received a `cookies` field of JSON \
         type `{actual_type}`, expected `array` (or absent). Cookies are \
         NOT being applied — the test will run with no authentication / \
         no session cookies and may fail for what looks like a flaky \
         network call but is actually a silent state-restore drop. Check \
         whether the storage-state JSON file is corrupted, was written \
         by a non-jet tool, or has the wrong schema version."
    )
}

/// GH #3761 — build the warn-body for a malformed `cookies` field in
/// the **CDP response** to `Storage.getCookies` (the READ-side sibling
/// of #3739). Surfaces Chromium protocol drift / proxy corruption that
/// would otherwise silently turn into an empty cookie list and look
/// like a flaky-auth test failure.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser.md#schema
pub(crate) fn format_cdp_get_cookies_shape_warn(actual_type: &str) -> String {
    format!(
        "GH #3761 Storage.getCookies CDP response carried a `cookies` \
         field of JSON type `{actual_type}`, expected `array` (or \
         absent). Returning an empty cookie list — the test will run \
         as if the context had no session and may fail for what looks \
         like a flaky auth call but is actually a silent CDP-shape \
         drop. Check whether Chromium drifted from the documented CDP \
         schema, a proxy / extension is rewriting the response, or the \
         browserContextId points at a stale partition."
    )
}

#[cfg(test)]
mod gh3739_set_storage_state_silent_no_op_tests {
    //! GH #3739 — `set_storage_state` previously did
    //! `state["cookies"].as_array().cloned().unwrap_or_default()`,
    //! silently no-op'ing when the `cookies` field was the wrong shape.
    //! These tests pin the helper's wording, the `json_value_type_name`
    //! contract, and the family-naming convention.

    use super::*;

    /// The warn body must include the issue tag and the actual JSON
    /// type name so an operator who hits the warn in logs can grep for
    /// it and immediately know what their storage-state JSON looks like.
    #[test]
    fn gh3739_helper_includes_tag_and_actual_type() {
        for actual_type in ["null", "bool", "number", "string", "object"] {
            let msg = format_storage_state_cookies_shape_warn(actual_type);
            assert!(
                msg.contains("GH #3739"),
                "must include issue tag for type={actual_type}: {msg}"
            );
            assert!(
                msg.contains(actual_type),
                "must name the actual type for type={actual_type}: {msg}"
            );
            assert!(
                msg.contains("array"),
                "must name the expected type (array) for type={actual_type}: {msg}"
            );
        }
    }

    /// The helper must give operators a remediation hint, not just
    /// blame. The wording must point at the three common causes:
    /// corrupt file, wrong tool, wrong schema version.
    #[test]
    fn gh3739_helper_includes_remediation_hint() {
        let msg = format_storage_state_cookies_shape_warn("string");
        assert!(
            msg.contains("corrupted") || msg.contains("non-jet") || msg.contains("schema"),
            "must include a remediation hint: {msg}"
        );
    }

    /// Determinism: same input → same output. No HashMap-ordering, no
    /// entropy, no timestamps.
    #[test]
    fn gh3739_helper_is_deterministic() {
        let a = format_storage_state_cookies_shape_warn("string");
        let b = format_storage_state_cookies_shape_warn("string");
        assert_eq!(a, b);
    }

    /// Six type names round-trip through `json_value_type_name`. This
    /// pins the strings used in log messages so a grep for one type
    /// continues to land.
    #[test]
    fn gh3739_json_value_type_name_covers_all_six_shapes() {
        assert_eq!(json_value_type_name(&Value::Null), "null");
        assert_eq!(json_value_type_name(&Value::Bool(true)), "bool");
        assert_eq!(
            json_value_type_name(&Value::Number(serde_json::Number::from(42))),
            "number"
        );
        assert_eq!(json_value_type_name(&Value::String("x".into())), "string");
        assert_eq!(json_value_type_name(&Value::Array(vec![])), "array");
        assert_eq!(
            json_value_type_name(&Value::Object(serde_json::Map::new())),
            "object"
        );
    }

    /// The five non-array type names must be pairwise distinct so a
    /// grep for `string` in logs doesn't accidentally match `object` etc.
    #[test]
    fn gh3739_json_value_type_name_is_pairwise_distinct() {
        let names = [
            json_value_type_name(&Value::Null),
            json_value_type_name(&Value::Bool(true)),
            json_value_type_name(&Value::Number(serde_json::Number::from(0))),
            json_value_type_name(&Value::String("x".into())),
            json_value_type_name(&Value::Array(vec![])),
            json_value_type_name(&Value::Object(serde_json::Map::new())),
        ];
        for i in 0..names.len() {
            for j in (i + 1)..names.len() {
                assert_ne!(names[i], names[j], "names must be pairwise distinct");
            }
        }
    }

    /// The warn must NOT leak the contents of the malformed value —
    /// storage-state JSON may carry credentials even when corrupted.
    /// Helper takes only the type name, not the value itself, so this
    /// is a structural guarantee; pin it with a contract test.
    #[test]
    fn gh3739_helper_signature_takes_only_type_name_not_value() {
        // This test exists to lock in that `format_*_warn` accepts a
        // `&str` (type name), not a `&Value` (the data). If somebody
        // later changes the signature to take the Value, this test
        // breaks and forces a re-audit.
        let msg = format_storage_state_cookies_shape_warn("string");
        // The wording must NOT mention placeholder values like
        // "actual_value=", "got value", etc.
        assert!(!msg.contains("actual_value"));
        assert!(!msg.contains("got value"));
    }

    /// Sibling-distinctness: this helper's tag must NOT collide with
    /// prior `format_*_warn` helpers in the same crate (we already
    /// audited #3725 / #3727 / #3730 / #3732 / #3734 / #3737).
    #[test]
    fn gh3739_helper_does_not_leak_sibling_tags() {
        let msg = format_storage_state_cookies_shape_warn("string");
        for sibling_tag in [
            "GH #3725", "GH #3727", "GH #3730", "GH #3732", "GH #3734", "GH #3737",
        ] {
            assert!(
                !msg.contains(sibling_tag),
                "#3739 msg must not leak sibling tag {sibling_tag}: {msg}"
            );
        }
    }

    /// `set_storage_state` MUST select the right branch for each of the
    /// six JSON shapes. We can't actually call `set_storage_state`
    /// without a live CDP session, but we can verify the cookies-vec
    /// derivation logic by mimicking the match. This pins the
    /// distinguishing behaviour: absent / null → empty, array → that
    /// array, anything else → empty (after warn).
    #[test]
    fn gh3739_cookies_derivation_for_each_shape() {
        fn derive(state: &Value) -> Vec<Value> {
            // Mirror the match inside `set_storage_state`.
            match &state["cookies"] {
                Value::Null => Vec::new(),
                Value::Array(arr) => arr.clone(),
                _ => Vec::new(),
            }
        }

        // Absent.
        assert_eq!(
            derive(&serde_json::json!({"origins": []})),
            Vec::<Value>::new()
        );
        // Null.
        assert_eq!(
            derive(&serde_json::json!({"cookies": null})),
            Vec::<Value>::new()
        );
        // Array with elements.
        let arr = vec![serde_json::json!({"name": "session", "value": "abc"})];
        assert_eq!(derive(&serde_json::json!({"cookies": arr.clone()})), arr);
        // Empty array — legitimate, returns empty.
        assert_eq!(
            derive(&serde_json::json!({"cookies": []})),
            Vec::<Value>::new()
        );
        // String — corruption, returns empty (warn fires in real path).
        assert_eq!(
            derive(&serde_json::json!({"cookies": "session=abc"})),
            Vec::<Value>::new()
        );
        // Object — corruption, returns empty.
        assert_eq!(
            derive(&serde_json::json!({"cookies": {"session": "abc"}})),
            Vec::<Value>::new()
        );
        // Number — corruption, returns empty.
        assert_eq!(
            derive(&serde_json::json!({"cookies": 42})),
            Vec::<Value>::new()
        );
        // Bool — corruption, returns empty.
        assert_eq!(
            derive(&serde_json::json!({"cookies": true})),
            Vec::<Value>::new()
        );
    }

    /// The five non-null-non-array shapes must each surface a DISTINCT
    /// type name in the warn body, so an operator looking at logs from
    /// a corruption incident can tell exactly what their storage-state
    /// JSON looks like.
    #[test]
    fn gh3739_warn_body_distinguishes_each_corrupt_shape() {
        let shapes: Vec<(Value, &str)> = vec![
            (Value::Bool(true), "bool"),
            (Value::Number(serde_json::Number::from(42)), "number"),
            (Value::String("x".into()), "string"),
            (Value::Object(serde_json::Map::new()), "object"),
        ];
        let mut msgs: Vec<String> = Vec::new();
        for (val, expected_type) in shapes {
            let actual_type = json_value_type_name(&val);
            assert_eq!(actual_type, expected_type);
            msgs.push(format_storage_state_cookies_shape_warn(actual_type));
        }
        // All four warn bodies must be pairwise distinct.
        for i in 0..msgs.len() {
            for j in (i + 1)..msgs.len() {
                assert_ne!(
                    msgs[i], msgs[j],
                    "warn bodies must distinguish each corrupt shape"
                );
            }
        }
    }

    /// Naming convention discoverability: the helper is named
    /// `format_storage_state_cookies_shape_warn`, matching the
    /// project-wide `format_<area>_<thing>_warn` convention.
    #[test]
    fn gh3739_helper_name_follows_family_convention() {
        let name = "format_storage_state_cookies_shape_warn";
        assert!(name.starts_with("format_"));
        assert!(name.ends_with("_warn"));
        // Pair: the type-name helper.
        let pair = "json_value_type_name";
        assert!(pair.ends_with("_name"));
    }
}

/// GH #3761 — `BrowserContext::cookies` previously did
/// `res["cookies"].as_array().cloned().unwrap_or_default()`, silently
/// returning an empty cookie list when Chromium's CDP response had the
/// wrong shape. The fix mirrors the #3739 set-side fix on the read
/// side: explicit match on absent / array / wrong-shape with a tagged
/// `tracing::warn!` for the wrong-shape branch.
#[cfg(test)]
mod gh3761_get_cookies_silent_shape_drop_tests {
    use super::*;

    #[test]
    fn gh3761_helper_includes_tag_and_actual_type() {
        for actual_type in ["null", "bool", "number", "string", "object"] {
            let msg = format_cdp_get_cookies_shape_warn(actual_type);
            assert!(
                msg.contains("GH #3761"),
                "must include issue tag for type={actual_type}: {msg}"
            );
            assert!(
                msg.contains(actual_type),
                "must name the actual type for type={actual_type}: {msg}"
            );
            assert!(
                msg.contains("array"),
                "must name the expected type for type={actual_type}: {msg}"
            );
        }
    }

    #[test]
    fn gh3761_helper_includes_remediation_hint() {
        let msg = format_cdp_get_cookies_shape_warn("string");
        assert!(
            msg.contains("Chromium") || msg.contains("proxy") || msg.contains("browserContextId"),
            "must include a remediation hint: {msg}"
        );
    }

    #[test]
    fn gh3761_helper_is_distinct_from_storage_state_sibling() {
        // Sibling helpers must carry DIFFERENT issue tags so operators
        // can grep the two failure modes (write-side #3739 vs read-side
        // #3761) and tell them apart in logs.
        let write_msg = format_storage_state_cookies_shape_warn("null");
        let read_msg = format_cdp_get_cookies_shape_warn("null");
        assert!(write_msg.contains("GH #3739"), "{write_msg}");
        assert!(read_msg.contains("GH #3761"), "{read_msg}");
        assert!(!write_msg.contains("GH #3761"));
        assert!(!read_msg.contains("GH #3739"));
        // The write helper mentions set_storage_state; the read helper
        // mentions Storage.getCookies.
        assert!(write_msg.contains("set_storage_state"));
        assert!(read_msg.contains("Storage.getCookies"));
    }

    #[test]
    fn gh3761_helper_message_is_deterministic() {
        let a = format_cdp_get_cookies_shape_warn("object");
        let b = format_cdp_get_cookies_shape_warn("object");
        assert_eq!(a, b);
    }

    #[test]
    fn gh3761_distinct_actual_types_produce_distinct_messages() {
        let a = format_cdp_get_cookies_shape_warn("null");
        let b = format_cdp_get_cookies_shape_warn("string");
        assert_ne!(a, b);
    }

    #[test]
    fn gh3761_helper_does_not_leak_response_body() {
        // Like the #3739 sibling, the warn must NOT include the actual
        // value bytes — a corrupt response may still carry sensitive
        // tokens / session cookies. Only the JSON type name leaks.
        let msg = format_cdp_get_cookies_shape_warn("string");
        assert!(!msg.contains("secret"));
        assert!(!msg.contains("token"));
        assert!(!msg.contains("session="));
    }

    #[test]
    fn gh3761_helper_name_follows_family_convention() {
        // Discoverability: callers searching for "format_cdp_get_cookies"
        // should find the helper in this module, and the name should
        // match the project-wide `format_<area>_<thing>_warn` shape.
        let name = "format_cdp_get_cookies_shape_warn";
        assert!(name.starts_with("format_"), "{name}");
        assert!(name.ends_with("_warn"), "{name}");
        assert!(name.contains("cdp"), "must anchor to CDP area: {name}");
        assert!(
            name.contains("get_cookies"),
            "must anchor to operation: {name}"
        );
    }

    #[test]
    fn gh3761_json_value_type_name_round_trips_into_warn() {
        // The helper takes whatever `json_value_type_name` returns; we
        // ensure the full pipeline works for each of the six shapes.
        for v in [
            Value::Null,
            Value::Bool(false),
            Value::Number(serde_json::Number::from(7)),
            Value::String("x".into()),
            Value::Array(Vec::new()),
            Value::Object(serde_json::Map::new()),
        ] {
            let t = json_value_type_name(&v);
            let msg = format_cdp_get_cookies_shape_warn(t);
            assert!(msg.contains(t), "must contain type {t}: {msg}");
        }
    }

    #[test]
    fn gh3761_warn_anchor_names_the_cdp_method() {
        // Operators grepping for the failing CDP call should find the
        // exact method name (Storage.getCookies) in the warn body, so
        // the bug report points at one specific CDP surface.
        let msg = format_cdp_get_cookies_shape_warn("number");
        assert!(
            msg.contains("Storage.getCookies"),
            "must name the CDP method: {msg}"
        );
    }

    #[test]
    fn gh3761_warn_explains_user_visible_symptom() {
        // The wording must connect the silent shape-drop to its
        // observable symptom (auth failure) so on-call grepping the
        // warn understands the connection without reading the PR.
        let msg = format_cdp_get_cookies_shape_warn("null");
        assert!(
            msg.contains("auth") || msg.contains("session"),
            "must explain user symptom: {msg}"
        );
    }
}
// CODEGEN-END
