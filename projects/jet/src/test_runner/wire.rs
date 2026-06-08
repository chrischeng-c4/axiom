// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
// CODEGEN-BEGIN
//! NDJSON wire protocol between the Rust runner and the Node.js worker.
//!
//! Each message is a single line of JSON over stdin/stdout. Events flow
//! worker → runner; requests flow runner → worker (v0 uses none). See TD
//! §Wire Protocol.
//!
//! Phase 3 adds **expect RPC requests** — DOM-integrated matcher calls issued
//! by the JS worker back to the Rust host so matchers can query browser state.
//! These flow over a **second stdio channel** (stdin from worker's perspective),
//! serialised as NDJSON alongside the normal event stream.
//!
//! Phase 4b (trace) adds `TraceMode` enum and `TraceEvent` variants so the
//! runner can gate capture and forward trace events over the wire.

use serde::{Deserialize, Serialize};

// ── Trace wire types ─────────────────────────────────────────────────────────

/// Trace capture mode as understood by the wire protocol.
///
/// Mirrors `crate::trace::buffer::TraceMode` but kept in this module so the
/// wire layer has no dependency on the trace module.
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R1
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R10
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WireTraceMode {
    /// No trace capture. Zero overhead.
    #[default]
    Off,
    /// Capture and write trace for every test.
    On,
    /// Capture for all tests; only write to disk for failed tests.
    RetainOnFailure,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
impl WireTraceMode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "off" => Some(WireTraceMode::Off),
            "on" => Some(WireTraceMode::On),
            "retain-on-failure" => Some(WireTraceMode::RetainOnFailure),
            _ => None,
        }
    }

    pub fn is_active(self) -> bool {
        self != WireTraceMode::Off
    }
}

/// Trace-capture events that flow runner → buffer (not over the wire, but
/// defined here for cross-module use in Phase 4b wiring).
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R2
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WireTraceEvent {
    /// Worker reported a console message that should be captured in the trace.
    StepConsole {
        level: String,
        text: String,
        ts_ms: u64,
    },
    /// CDP Network.requestWillBeSent observed.
    NetworkRequest {
        request_id: String,
        url: String,
        method: String,
        ts_ms: u64,
    },
    /// CDP Network.responseReceived observed.
    NetworkResponse {
        request_id: String,
        status: u16,
        ts_ms: u64,
    },
}

// ── Expect RPC (worker → Rust, bidirectional) ────────────────────────────────

/// Requests the **worker** sends to the Rust host for DOM-integrated matchers.
///
/// These are distinct from `WorkerEvent` variants to allow a single-channel
/// multiplexed design: requests carry `req_id` for correlation.
// @spec .aw/changes/enhancement-phase-3-dom-matchers-fixtures-for-native-test-runn/specs/enhancement-phase-3-dom-matchers-fixtures-for-native-test-runn-spec.md#R4
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WireRequest {
    /// Query text content of the element matching `selector` on page `page_id`.
    /// Maps to `Locator::text_content()` on the Rust side.
    // @spec ...#R1
    QueryText {
        req_id: u64,
        page_id: String,
        selector: String,
    },
    /// Check whether the element matching `selector` is visible.
    /// Maps to `Locator::is_visible()` on the Rust side.
    // @spec ...#R2
    IsVisible {
        req_id: u64,
        page_id: String,
        selector: String,
    },
    /// Capture a full-page screenshot. Maps to `Page::screenshot()`.
    /// The Rust host base64-encodes the PNG bytes before sending the response.
    // @spec ...#R3
    Screenshot { req_id: u64, page_id: String },
    /// Capture a screenshot and compare against the stored baseline at
    /// `__snapshots__/<spec-slug>/<snapshot_name>.png` adjacent to the spec
    /// file. First-run writes the baseline and passes. Subsequent runs diff
    /// exact bytes. See `worker::load_or_write_snapshot`.
    // @spec ...#R3
    // @spec ...#R7
    MatchSnapshot {
        req_id: u64,
        page_id: String,
        snapshot_name: String,
    },
    /// Compare a JS-serialized text/object value against the stored baseline
    /// at `__snapshots__/<spec-slug>/<snapshot_name>.txt` adjacent to the spec
    /// file. First-run writes the baseline and passes. `--update-snapshots`
    /// overwrites on mismatch. See `worker::load_or_write_text_snapshot`.
    // @spec #2713
    MatchTextSnapshot {
        req_id: u64,
        snapshot_name: String,
        content: String,
    },
    /// Live E2E checkpoint emitted immediately after `test_start` and before
    /// fixture setup/test body execution. The Rust host may block the response
    /// until the human runner shell allows the next case.
    LiveCheckpoint {
        req_id: u64,
        test_id: String,
        title: String,
    },
}

/// Responses the Rust host sends back for `WireRequest` messages.
// @spec .aw/changes/enhancement-phase-3-dom-matchers-fixtures-for-native-test-runn/specs/enhancement-phase-3-dom-matchers-fixtures-for-native-test-runn-spec.md#R4
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WireResponse {
    /// Successful reply to `QueryText`.
    TextResult { req_id: u64, text: String },
    /// Successful reply to `IsVisible`.
    VisibleResult { req_id: u64, visible: bool },
    /// Successful reply to `Screenshot`. `data` is base64-encoded PNG.
    ScreenshotResult { req_id: u64, data: String },
    /// Successful reply to `MatchSnapshot` (baseline written or bytes matched).
    SnapshotResult { req_id: u64 },
    /// Successful reply to `LiveCheckpoint`.
    LiveCheckpointResult { req_id: u64 },
    /// Error reply for any request (matcher_diff carries actual/expected on failure).
    Error {
        req_id: u64,
        message: String,
        /// Optional structured diff for `matcher_diff` reporter display.
        /// Format: `{ "actual": "...", "expected": "..." }` JSON string.
        // @spec ...#R8
        matcher_diff: Option<MatcherDiff>,
    },
}

/// Structured diff returned on DOM matcher failure so the reporter can display
/// a meaningful diff line.
// @spec .aw/changes/enhancement-phase-3-dom-matchers-fixtures-for-native-test-runn/specs/enhancement-phase-3-dom-matchers-fixtures-for-native-test-runn-spec.md#R8
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatcherDiff {
    pub actual: String,
    pub expected: String,
}

/// Parse a `WireRequest` NDJSON line from the worker.
///
/// Returns `None` if the line is empty, not JSON, or a JSON object that
/// does not look like a tagged `WireRequest`. Lines that look like a
/// tagged `WireRequest` but fail typed deserialization (well-formed JSON
/// with a `kind` discriminator but a wrong-shaped field) emit a
/// `tracing::warn!` with the "GH #3759" tag and return `None`, so the
/// caller's passthrough behaviour for stdout is preserved while the
/// protocol-shape error is no longer silent.
/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
pub fn parse_request(line: &str) -> Option<WireRequest> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    // Step 1: structural parse. Non-JSON or non-object lines are stdout
    // (intentionally silent — see `read_events` passthrough).
    let value: serde_json::Value = serde_json::from_str(trimmed).ok()?;
    if !value_has_kind(&value) {
        return None;
    }
    // Step 2: typed parse. The discriminator says this is a tagged
    // WireRequest but the rest of the shape is wrong — warn loudly,
    // unless the `kind` doesn't match any known variant (that case is
    // most likely stdout that happens to be JSON with a `kind` field).
    match serde_json::from_value::<WireRequest>(value) {
        Ok(req) => Some(req),
        Err(err) => {
            if is_unknown_variant_error(&err) {
                return None;
            }
            tracing::warn!(
                target: "jet::test_runner::wire",
                "{}",
                format_wire_request_shape_warn(trimmed, &err)
            );
            None
        }
    }
}

/// Messages the **worker** sends to the runner.
/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorkerEvent {
    /// The worker has loaded the spec file and collected its test plan.
    Plan {
        file: String,
        tests: Vec<TestDescriptor>,
    },
    /// A test is about to run.
    TestStart {
        id: String,
        suite: Vec<String>,
        name: String,
    },
    /// A test finished (pass, fail, or skipped).
    TestEnd {
        id: String,
        suite: Vec<String>,
        name: String,
        outcome: TestOutcome,
        duration_ms: u64,
        error: Option<TestError>,
        /// 1-indexed shard number when `--shard=i/N` is active. `null` in serial runs.
        // @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R8
        #[serde(default, skip_serializing_if = "Option::is_none")]
        shard_index: Option<u32>,
        /// Total shard count N when `--shard=i/N` is active. `null` in serial runs.
        // @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R8
        #[serde(default, skip_serializing_if = "Option::is_none")]
        shard_total: Option<u32>,
        /// Absolute paths to artifacts captured when the test failed —
        /// screenshots today, traces/videos later. Empty for passing tests.
        // @spec .aw/tech-design/projects/jet/logic/auto-artifacts.md#A1
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        artifacts: Vec<String>,
    },
    /// A `console.log`/`console.error` line from the spec file.
    Console {
        stream: ConsoleStream,
        message: String,
    },
    /// Fatal worker error — the test plan could not be collected or the
    /// worker crashed mid-run.
    Fatal { message: String },
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestDescriptor {
    pub id: String,
    pub suite: Vec<String>,
    pub name: String,
    pub skip: bool,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TestOutcome {
    Passed,
    Failed,
    Skipped,
    TimedOut,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConsoleStream {
    Stdout,
    Stderr,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestError {
    pub message: String,
    pub stack: Option<String>,
    /// Optional matcher diff (e.g. `toBe` failure: expected vs actual).
    pub diff: Option<String>,
}

/// Parse a single NDJSON line. Returns `None` if the line is empty or not
/// recognisable (stray stdout from the spec leaks through as `Console` via a
/// passthrough wrapper — see `worker::read_events`).
///
/// GH #3759 — lines that look like a tagged `WorkerEvent` (well-formed JSON
/// object with a `kind` discriminator) but fail typed deserialization now
/// emit a `tracing::warn!` carrying the issue tag, the offending preview,
/// and the serde error, so a protocol-shape bug in the worker is no longer
/// silently dropped. The function still returns `None` so the line is
/// routed through the same passthrough path as legitimate stdout.
/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
pub fn parse_line(line: &str) -> Option<WorkerEvent> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    // Step 1: structural parse. Non-JSON or non-object lines are stdout
    // (intentionally silent — see `read_events` passthrough).
    let value: serde_json::Value = serde_json::from_str(trimmed).ok()?;
    if !value_has_kind(&value) {
        return None;
    }
    // Step 2: typed parse. The discriminator says this is a tagged
    // WorkerEvent but the rest of the shape is wrong — warn loudly,
    // unless the `kind` doesn't match any known variant (that case is
    // most likely stdout that happens to be JSON with a `kind` field).
    match serde_json::from_value::<WorkerEvent>(value) {
        Ok(event) => Some(event),
        Err(err) => {
            if is_unknown_variant_error(&err) {
                return None;
            }
            tracing::warn!(
                target: "jet::test_runner::wire",
                "{}",
                format_worker_event_shape_warn(trimmed, &err)
            );
            None
        }
    }
}

/// True when the serde error reports an unknown-variant failure
/// (i.e. `kind` value didn't match any known variant name). Such
/// lines are most likely spec stdout that happens to be a JSON object
/// with a `kind` field, so we keep returning `None` silently rather
/// than spamming the log for every rogue stdout payload.
/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
pub(crate) fn is_unknown_variant_error(err: &serde_json::Error) -> bool {
    err.to_string().starts_with("unknown variant")
}

/// True if `value` is a JSON object carrying a string `kind` field.
/// Stdout from the spec is overwhelmingly *not* a JSON object with
/// `kind`, so this filter cheaply separates legitimate protocol lines
/// (`kind`-tagged) from genuine console output. Lines that *are*
/// tagged but have a wrong-shape field downstream get warn'd.
fn value_has_kind(value: &serde_json::Value) -> bool {
    matches!(
        value.as_object().and_then(|m| m.get("kind")),
        Some(serde_json::Value::String(_))
    )
}

/// Format the `tracing::warn!` payload for a well-formed JSON object
/// that looked like a `WorkerEvent` but failed typed deserialization.
/// The "GH #3759" tag pins the warning to the originating issue; the
/// preview is truncated to 200 bytes so a runaway payload doesn't
/// drown the log.
/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
pub(crate) fn format_worker_event_shape_warn(line: &str, err: &serde_json::Error) -> String {
    format!(
        "GH #3759: worker event has known `kind` discriminator but \
         wrong shape (refusing silent drop of a protocol message); \
         err={err}; line_preview={:?}",
        truncate_preview(line, 200)
    )
}

/// Sibling of [`format_worker_event_shape_warn`] for the runner→worker
/// `WireRequest` direction. Carries the same "GH #3759" tag.
/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
pub(crate) fn format_wire_request_shape_warn(line: &str, err: &serde_json::Error) -> String {
    format!(
        "GH #3759: wire request has known `kind` discriminator but \
         wrong shape (refusing silent drop of a protocol message); \
         err={err}; line_preview={:?}",
        truncate_preview(line, 200)
    )
}

fn truncate_preview(line: &str, max_bytes: usize) -> &str {
    if line.len() <= max_bytes {
        line
    } else {
        // Walk to the largest char boundary ≤ max_bytes so we never
        // slice in the middle of a UTF-8 sequence.
        let mut idx = max_bytes;
        while idx > 0 && !line.is_char_boundary(idx) {
            idx -= 1;
        }
        &line[..idx]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_plan() {
        let ev = WorkerEvent::Plan {
            file: "a.spec.ts".into(),
            tests: vec![TestDescriptor {
                id: "0".into(),
                suite: vec!["math".into()],
                name: "adds".into(),
                skip: false,
            }],
        };
        let s = serde_json::to_string(&ev).unwrap();
        let back: WorkerEvent = serde_json::from_str(&s).unwrap();
        match back {
            WorkerEvent::Plan { tests, .. } => {
                assert_eq!(tests.len(), 1);
                assert_eq!(tests[0].name, "adds");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn round_trip_test_end_with_error() {
        let ev = WorkerEvent::TestEnd {
            id: "1".into(),
            suite: vec![],
            name: "fails".into(),
            outcome: TestOutcome::Failed,
            duration_ms: 42,
            error: Some(TestError {
                message: "Expected 1 to be 2".into(),
                stack: Some("at foo.js:3".into()),
                diff: Some("-1\n+2".into()),
            }),
            shard_index: None,
            shard_total: None,
            artifacts: Vec::new(),
        };
        let s = serde_json::to_string(&ev).unwrap();
        let back: WorkerEvent = serde_json::from_str(&s).unwrap();
        match back {
            WorkerEvent::TestEnd { outcome, error, .. } => {
                assert_eq!(outcome, TestOutcome::Failed);
                assert!(error.is_some());
                assert_eq!(error.unwrap().diff.as_deref(), Some("-1\n+2"));
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn empty_line_is_none() {
        assert!(parse_line("").is_none());
        assert!(parse_line("   ").is_none());
    }

    #[test]
    fn non_json_line_is_none() {
        assert!(parse_line("not json").is_none());
        assert!(parse_line("{incomplete").is_none());
    }

    #[test]
    fn unknown_kind_is_none() {
        assert!(parse_line("{\"kind\": \"mystery\"}").is_none());
    }

    /// GH #3094 — `WireTraceMode::from_str` must return `None` for any input
    /// that isn't one of the three documented values. The CLI relies on
    /// `None` to produce a hard error instead of silently falling back to
    /// `Off` (which is the opposite of what the user asked for). If this
    /// contract regresses — e.g. someone adds an implicit `_ => Off`
    /// fallback — the CLI's error path goes dead and the silent-swallow
    /// returns. Pin the contract here.
    #[test]
    fn wire_trace_mode_from_str_rejects_unknown_values() {
        assert_eq!(WireTraceMode::from_str("off"), Some(WireTraceMode::Off));
        assert_eq!(WireTraceMode::from_str("on"), Some(WireTraceMode::On));
        assert_eq!(
            WireTraceMode::from_str("retain-on-failure"),
            Some(WireTraceMode::RetainOnFailure)
        );

        // Common typos / unknown values must return None, NOT silently
        // coerce to Off.
        assert_eq!(WireTraceMode::from_str("verbose"), None);
        assert_eq!(WireTraceMode::from_str("retain-on-failed"), None);
        assert_eq!(WireTraceMode::from_str("ON"), None, "case-sensitive");
        assert_eq!(WireTraceMode::from_str(""), None);
        assert_eq!(WireTraceMode::from_str("retain"), None);
    }
}

/// GH #3759 — silent shape-mismatch drops in `wire::parse_line` /
/// `wire::parse_request`. The fix is a two-step parse: structural
/// `from_str::<Value>` (silent on non-JSON / no `kind`) followed by a
/// typed `from_value::<…>` whose non-unknown-variant errors warn via
/// the new family helpers. These tests pin the helper contract and
/// the new behaviour for both directions.
#[cfg(test)]
mod gh3759_wire_shape_warn_tests {
    use super::*;

    #[test]
    fn gh3759_worker_event_shape_warn_contains_tag_err_and_preview() {
        let err: serde_json::Error = serde_json::from_str::<u32>("\"x\"").unwrap_err();
        let msg = format_worker_event_shape_warn("{\"kind\":\"test_end\"}", &err);
        assert!(msg.contains("GH #3759"), "tag missing: {msg}");
        assert!(msg.contains("wrong shape"), "shape anchor missing: {msg}");
        assert!(msg.contains("line_preview"), "preview key missing: {msg}");
        assert!(msg.contains("test_end"), "preview value missing: {msg}");
    }

    #[test]
    fn gh3759_wire_request_shape_warn_distinct_from_worker_event_warn() {
        let err: serde_json::Error = serde_json::from_str::<u32>("\"x\"").unwrap_err();
        let we = format_worker_event_shape_warn("{}", &err);
        let wr = format_wire_request_shape_warn("{}", &err);
        assert!(we.contains("worker event"), "worker anchor: {we}");
        assert!(wr.contains("wire request"), "request anchor: {wr}");
        assert_ne!(we, wr);
    }

    #[test]
    fn gh3759_both_warns_carry_same_issue_tag() {
        let err: serde_json::Error = serde_json::from_str::<u32>("\"x\"").unwrap_err();
        let we = format_worker_event_shape_warn("{}", &err);
        let wr = format_wire_request_shape_warn("{}", &err);
        assert!(we.contains("GH #3759"));
        assert!(wr.contains("GH #3759"));
    }

    #[test]
    fn gh3759_warn_is_deterministic_for_same_inputs() {
        let err: serde_json::Error = serde_json::from_str::<u32>("\"x\"").unwrap_err();
        let a = format_worker_event_shape_warn("line", &err);
        let b = format_worker_event_shape_warn("line", &err);
        assert_eq!(a, b);
    }

    #[test]
    fn gh3759_parse_line_still_silent_for_non_json() {
        assert!(parse_line("not json").is_none());
        assert!(parse_line("{incomplete").is_none());
    }

    #[test]
    fn gh3759_parse_line_still_silent_for_json_without_kind() {
        // A plain JSON object without `kind` is most likely stdout and
        // must stay silent (no warn).
        assert!(parse_line("{\"hello\":\"world\"}").is_none());
        assert!(parse_line("[1,2,3]").is_none());
        assert!(parse_line("42").is_none());
    }

    #[test]
    fn gh3759_parse_line_still_silent_for_unknown_kind() {
        // Discriminator that doesn't match any known variant — still
        // silent because unknown-variant is most likely stdout that
        // happens to have a `kind` field.
        assert!(parse_line("{\"kind\":\"mystery\"}").is_none());
    }

    #[test]
    fn gh3759_parse_line_returns_some_for_well_formed_known_variant() {
        // Happy-path regression: a real WorkerEvent still round-trips.
        let line = r#"{"kind":"console","stream":"stdout","message":"hi"}"#;
        let event = parse_line(line).expect("console event must parse");
        match event {
            WorkerEvent::Console { message, .. } => assert_eq!(message, "hi"),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn gh3759_parse_line_returns_none_for_known_kind_with_bad_shape() {
        // Known discriminator (`test_start`) but `id` is a number, not
        // a string — this is the bug class #3759 catches. parse_line
        // still returns None (so the caller's passthrough behaviour is
        // unchanged), but the warn path is exercised.
        let line = r#"{"kind":"test_start","id":42,"suite":[],"name":"x"}"#;
        assert!(parse_line(line).is_none());
    }

    #[test]
    fn gh3759_parse_request_returns_some_for_well_formed_request() {
        let line = r##"{"kind":"query_text","req_id":1,"page_id":"p","selector":"#x"}"##;
        let req = parse_request(line).expect("query_text request must parse");
        match req {
            WireRequest::QueryText { req_id, .. } => assert_eq!(req_id, 1),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn gh3759_parse_request_returns_none_for_known_kind_with_bad_shape() {
        // `req_id` should be u64 not string — exact same silent-drop
        // family as parse_line.
        let line = r##"{"kind":"query_text","req_id":"oops","page_id":"p","selector":"#x"}"##;
        assert!(parse_request(line).is_none());
    }

    #[test]
    fn gh3759_value_has_kind_filter() {
        let v: serde_json::Value = serde_json::from_str("{\"kind\":\"plan\"}").unwrap();
        assert!(value_has_kind(&v));
        let v: serde_json::Value = serde_json::from_str("{\"other\":1}").unwrap();
        assert!(!value_has_kind(&v));
        let v: serde_json::Value = serde_json::from_str("{\"kind\":1}").unwrap();
        assert!(
            !value_has_kind(&v),
            "non-string kind must not trigger warn path"
        );
        let v: serde_json::Value = serde_json::from_str("[1,2]").unwrap();
        assert!(!value_has_kind(&v));
    }

    #[test]
    fn gh3759_unknown_variant_detector_matches_serde_message() {
        // Force serde to emit an "unknown variant" error for
        // WorkerEvent and confirm our detector matches it.
        let err = serde_json::from_str::<WorkerEvent>("{\"kind\":\"mystery\"}").unwrap_err();
        assert!(
            is_unknown_variant_error(&err),
            "detector must match unknown-variant: {err}"
        );
        // And a wrong-shape error must NOT be detected as unknown-variant.
        let err = serde_json::from_str::<WorkerEvent>(
            "{\"kind\":\"test_start\",\"id\":1,\"suite\":[],\"name\":\"x\"}",
        )
        .unwrap_err();
        assert!(
            !is_unknown_variant_error(&err),
            "wrong-shape must not be detected as unknown-variant: {err}"
        );
    }

    #[test]
    fn gh3759_truncate_preview_respects_char_boundaries() {
        // 2-byte UTF-8 char at the boundary — must not slice mid-codepoint.
        let s = "aé"; // 'a' (1B) + 'é' (2B) = 3 bytes total
        assert_eq!(truncate_preview(s, 100), "aé");
        assert_eq!(truncate_preview(s, 1), "a");
        // Cutting in the middle of 'é' must back off to the codepoint
        // boundary (so we get "a", not a partial UTF-8 sequence).
        assert_eq!(truncate_preview(s, 2), "a");
    }
}
// CODEGEN-END
