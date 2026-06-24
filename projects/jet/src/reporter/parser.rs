// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-reporter.md#schema
// CODEGEN-BEGIN
//! NDJSON parser for the HTML reporter.
//!
//! Reads NDJSON wire protocol `testEnd` events and reconstructs `TestReport`
//! rows understood by the HTML reporter. The NDJSON format is exactly the same
//! as what the native test runner emits over the worker wire; no new protocol
//! introduced.
//!
// @spec enhancement-html-reporter-for-native-test-runner-spec#R8

use crate::reporter::html::TestRow;
use crate::test_runner::reporter::{Outcome, TestError, TestReport};
use crate::test_runner::wire::{TestOutcome, WorkerEvent};
use anyhow::{Context, Result};
use std::path::PathBuf;

/// Parse a NDJSON byte stream of `WorkerEvent` lines and reconstruct a
/// `Vec<TestReport>`.  Lines that are empty or not parseable are skipped
/// silently (matches the tolerant approach used by the live runner).
///
/// # Wire protocol
///
/// Each line must be a JSON object with `"kind": "test_end"` (snake_case as
/// emitted by the `WorkerEvent` serde tag).
///
/// # Example
///
/// ```json
/// {"kind":"test_end","id":"abc","suite":[],"name":"adds numbers","outcome":"passed","duration_ms":12,"error":null}
/// ```
// @spec enhancement-html-reporter-for-native-test-runner-spec#R8
pub fn parse_ndjson(bytes: &[u8]) -> Result<Vec<TestReport>> {
    let text = std::str::from_utf8(bytes).context("NDJSON bytes are not valid UTF-8")?;

    let mut reports = Vec::new();
    for (lineno, raw_line) in text.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        match crate::test_runner::wire::parse_line(line) {
            Some(WorkerEvent::TestEnd {
                id: _,
                suite,
                name,
                outcome,
                duration_ms,
                error,
                shard_index,
                shard_total,
                artifacts,
            }) => {
                let outcome_mapped = match outcome {
                    TestOutcome::Passed => Outcome::Passed,
                    TestOutcome::Failed => Outcome::Failed,
                    TestOutcome::Skipped => Outcome::Skipped,
                    TestOutcome::TimedOut => Outcome::TimedOut,
                };

                // Derive a synthetic file path from the suite if no explicit
                // path is present in the NDJSON (the full path is only present
                // in the Plan event; testEnd events don't carry it).
                let file = PathBuf::from("unknown.spec.ts");

                let error_mapped = error.map(|e| {
                    let source_location = e
                        .stack
                        .as_deref()
                        .and_then(crate::test_runner::reporter::SourceLocation::parse_from_stack);
                    TestError {
                        message: e.message,
                        stack: e.stack,
                        diff: e.diff,
                        source_location,
                    }
                });

                reports.push(TestReport {
                    file,
                    suite,
                    name,
                    outcome: outcome_mapped,
                    duration_ms,
                    error: error_mapped,
                    trace_path: None,
                    shard_index,
                    shard_total,
                    artifacts: artifacts.into_iter().map(PathBuf::from).collect(),
                    steps: Vec::new(),
                });
            }
            Some(_) => {
                // Non-testEnd events (Plan, TestStart, Console, Fatal) — skip.
            }
            None => {
                // Unparseable line — skip.
                let _ = lineno; // suppress unused warning
            }
        }
    }

    Ok(reports)
}

/// Like `parse_ndjson` but returns `TestRow` values directly, suitable for
/// use by the merger without going through `HtmlReporter::emit`.
///
/// Parses the compact row JSON produced by `row_to_ndjson_line`.
// @spec enhancement-html-reporter-for-native-test-runner-spec#R7
pub fn parse_ndjson_to_rows(bytes: &[u8]) -> Vec<TestRow> {
    // GH #3314 — invalid UTF-8 (truncated codepoint after a SIGKILL'd
    // worker, partial write, etc.) used to silently return an empty row
    // set. Recover via lossy decode so any valid lines preceding the
    // corruption still land in the merged report, and emit a warn so the
    // operator can find the corrupted shard.
    let text_buf: String;
    let text = match std::str::from_utf8(bytes) {
        Ok(t) => t,
        Err(err) => {
            tracing::warn!(
                target: "jet::reporter",
                error = %err,
                bytes_len = bytes.len(),
                valid_up_to = err.valid_up_to(),
                "GH #3314 invalid UTF-8 in NDJSON; recovering via lossy decode \
                 so any valid lines before the corruption still appear in the \
                 merged report"
            );
            text_buf = String::from_utf8_lossy(bytes).into_owned();
            text_buf.as_str()
        }
    };

    let mut rows = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match serde_json::from_str::<serde_json::Value>(line) {
            Ok(v) => {
                // GH #3763 — fields used to coerce silently to ""/0/"unknown"
                // on the wrong type. A producer that switched its NDJSON
                // schema (e.g. duration_ms as "12" string instead of number)
                // would silently appear as duration 0 in the HTML report
                // with no diagnostic. Distinguish absent (Null) from
                // wrong-shape and warn on the wrong-shape branch.
                let test_id = string_field_or_warn(&v, "test_id", "");
                let name = string_field_or_warn(&v, "name", "");
                let status = string_field_or_warn(&v, "status", "unknown");
                let duration_ms = u64_field_or_warn(&v, "duration_ms", 0);
                let file = string_field_or_warn(&v, "file", "");
                let stack_trace = optional_string_field_or_warn(&v, "stack_trace");
                let matcher_diff = optional_string_field_or_warn(&v, "matcher_diff");
                let trace_path = optional_string_field_or_warn(&v, "trace_path");

                if test_id.is_empty() {
                    continue;
                }
                rows.push(TestRow {
                    test_id,
                    name,
                    status,
                    duration_ms,
                    file,
                    stack_trace,
                    matcher_diff,
                    trace_path,
                });
            }
            Err(err) => {
                // GH #3355 — per-line JSON parse failures used to vanish
                // silently. A worker that panics mid-write can leave a
                // truncated line that drops a real test result from the
                // merged HTML report with no diagnostic. Skip the line but
                // surface the corruption so the operator can chase the
                // worker, not the report.
                let preview: String = line.chars().take(200).collect();
                tracing::warn!(
                    target: "jet::reporter",
                    error = %err,
                    line_len = line.len(),
                    line_preview = %preview,
                    "GH #3355 dropped a malformed NDJSON line while \
                     building TestRow set. Check the originating worker \
                     shard for an interrupted write or schema drift."
                );
            }
        }
    }
    rows
}

/// GH #3763 — name the JSON value kind for diagnostics.
///
/// Used by [`string_field_or_warn`], [`u64_field_or_warn`], and
/// [`optional_string_field_or_warn`] to distinguish absent (`Null`) from
/// wrong-shape values when parsing NDJSON `TestRow` lines.
fn json_value_kind(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

/// GH #3763 — diagnostic message for a wrong-shape string field in a
/// `TestRow` NDJSON line. Operators grep for "GH #3763" to chase producer
/// schema drift.
/// @spec .aw/tech-design/projects/jet/semantic/jet-reporter.md#schema
pub(crate) fn format_reporter_parser_string_field_warn(field: &str, actual_type: &str) -> String {
    format!(
        "GH #3763 NDJSON TestRow field `{field}` has wrong shape \
         (expected string, got {actual_type}); coerced to default. \
         Check the producing worker for schema drift."
    )
}

/// GH #3763 — diagnostic message for a wrong-shape numeric field in a
/// `TestRow` NDJSON line.
/// @spec .aw/tech-design/projects/jet/semantic/jet-reporter.md#schema
pub(crate) fn format_reporter_parser_u64_field_warn(field: &str, actual_type: &str) -> String {
    format!(
        "GH #3763 NDJSON TestRow field `{field}` has wrong shape \
         (expected unsigned integer, got {actual_type}); coerced to 0. \
         Check the producing worker for schema drift."
    )
}

/// GH #3763 — diagnostic message for a wrong-shape optional string field
/// in a `TestRow` NDJSON line.
/// @spec .aw/tech-design/projects/jet/semantic/jet-reporter.md#schema
pub(crate) fn format_reporter_parser_optional_string_field_warn(
    field: &str,
    actual_type: &str,
) -> String {
    format!(
        "GH #3763 NDJSON TestRow field `{field}` has wrong shape \
         (expected string or null, got {actual_type}); treated as None. \
         Check the producing worker for schema drift."
    )
}

/// GH #3763 — read a required string field, warning on wrong shape and
/// staying silent on absent/null. Returns `default` for both absent and
/// wrong-shape so callers retain the prior behavior.
fn string_field_or_warn(v: &serde_json::Value, field: &str, default: &str) -> String {
    match v.get(field) {
        None | Some(serde_json::Value::Null) => default.to_string(),
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(other) => {
            let actual_type = json_value_kind(other);
            tracing::warn!(
                target: "jet::reporter",
                field = %field,
                actual_type = %actual_type,
                "{}",
                format_reporter_parser_string_field_warn(field, actual_type)
            );
            default.to_string()
        }
    }
}

/// GH #3763 — read a required `u64` field, warning on wrong shape and
/// staying silent on absent/null.
fn u64_field_or_warn(v: &serde_json::Value, field: &str, default: u64) -> u64 {
    match v.get(field) {
        None | Some(serde_json::Value::Null) => default,
        Some(serde_json::Value::Number(n)) => n.as_u64().unwrap_or_else(|| {
            let actual_type = if n.is_i64() {
                "negative integer"
            } else {
                "non-integer number"
            };
            tracing::warn!(
                target: "jet::reporter",
                field = %field,
                actual_type = %actual_type,
                "{}",
                format_reporter_parser_u64_field_warn(field, actual_type)
            );
            default
        }),
        Some(other) => {
            let actual_type = json_value_kind(other);
            tracing::warn!(
                target: "jet::reporter",
                field = %field,
                actual_type = %actual_type,
                "{}",
                format_reporter_parser_u64_field_warn(field, actual_type)
            );
            default
        }
    }
}

/// GH #3763 — read an optional string field, warning on wrong shape and
/// staying silent on absent/null.
fn optional_string_field_or_warn(v: &serde_json::Value, field: &str) -> Option<String> {
    match v.get(field) {
        None | Some(serde_json::Value::Null) => None,
        Some(serde_json::Value::String(s)) => Some(s.clone()),
        Some(other) => {
            let actual_type = json_value_kind(other);
            tracing::warn!(
                target: "jet::reporter",
                field = %field,
                actual_type = %actual_type,
                "{}",
                format_reporter_parser_optional_string_field_warn(field, actual_type)
            );
            None
        }
    }
}

#[cfg(test)]
mod gh3314_tests {
    use super::*;

    /// GH #3314 — happy path: valid NDJSON bytes round-trip.
    #[test]
    fn parse_ndjson_to_rows_valid_utf8_extracts_rows() {
        let line = r#"{"test_id":"a","name":"a","status":"pass","duration_ms":1,"file":"a.ts"}"#;
        let rows = parse_ndjson_to_rows(line.as_bytes());
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].test_id, "a");
    }

    /// GH #3314 — corrupted UTF-8 tail (e.g. SIGKILL'd worker mid-write)
    /// must NOT silently drop the entire shard. The valid prefix lines
    /// must be recovered via lossy decode + a warn.
    #[test]
    fn parse_ndjson_to_rows_corrupt_utf8_recovers_valid_prefix() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(
            br#"{"test_id":"a","name":"a","status":"pass","duration_ms":1,"file":"a.ts"}"#,
        );
        bytes.push(b'\n');
        // Truncated UTF-8 codepoint at the tail — would have been "好"
        // but only the first two of three bytes are present.
        bytes.push(0xE5);
        bytes.push(0xA5);

        let rows = parse_ndjson_to_rows(&bytes);
        assert_eq!(
            rows.len(),
            1,
            "valid prefix line must survive corruption at the tail"
        );
        assert_eq!(rows[0].test_id, "a");
    }

    // ── GH #3355 per-line JSON parse swallow ─────────────────────────────

    /// GH #3355 — mixed valid + malformed lines: valid rows must be
    /// recovered; the malformed line is dropped (and a warn is emitted).
    #[test]
    fn gh3355_parse_ndjson_to_rows_mixed_valid_and_malformed() {
        let payload = concat!(
            r#"{"test_id":"a","name":"a","status":"pass","duration_ms":1,"file":"a.ts"}"#,
            "\n",
            "{this-is-not-json\n",
            r#"{"test_id":"b","name":"b","status":"fail","duration_ms":2,"file":"b.ts"}"#,
            "\n",
        );
        let rows = parse_ndjson_to_rows(payload.as_bytes());
        assert_eq!(
            rows.len(),
            2,
            "valid rows must survive a single malformed line"
        );
        let ids: Vec<_> = rows.iter().map(|r| r.test_id.as_str()).collect();
        assert_eq!(ids, vec!["a", "b"]);
    }

    /// GH #3355 — fully-malformed shard yields zero rows without panic.
    #[test]
    fn gh3355_parse_ndjson_to_rows_all_malformed_returns_empty() {
        let payload = "not json\n{also not json\n";
        let rows = parse_ndjson_to_rows(payload.as_bytes());
        assert!(
            rows.is_empty(),
            "all-malformed input must return no rows: {:?}",
            rows
        );
    }

    /// GH #3355 — blank lines and pure-whitespace lines must remain a
    /// silent skip (no warn worthiness), and not be counted as parse
    /// failures. This guards against a regression where the warn fires on
    /// trivial whitespace.
    #[test]
    fn gh3355_parse_ndjson_to_rows_blank_lines_skipped_silently() {
        let payload = concat!(
            "\n",
            "   \n",
            r#"{"test_id":"a","name":"a","status":"pass","duration_ms":1,"file":"a.ts"}"#,
            "\n",
            "\n",
        );
        let rows = parse_ndjson_to_rows(payload.as_bytes());
        assert_eq!(rows.len(), 1, "only the one real row should land");
    }
}

#[cfg(test)]
mod gh3763_wrong_shape_warn_tests {
    use super::*;

    /// GH #3763 — wrong-shape `name` (number instead of string) yields an
    /// empty name without dropping the row; warn is emitted.
    #[test]
    fn gh3763_wrong_shape_name_field_coerced_with_warn() {
        let line = r#"{"test_id":"a","name":42,"status":"pass","duration_ms":1,"file":"a.ts"}"#;
        let rows = parse_ndjson_to_rows(line.as_bytes());
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].test_id, "a");
        assert_eq!(rows[0].name, "");
    }

    /// GH #3763 — wrong-shape `status` (boolean) coerces to "unknown".
    #[test]
    fn gh3763_wrong_shape_status_field_coerced_to_unknown() {
        let line = r#"{"test_id":"a","name":"a","status":true,"duration_ms":1,"file":"a.ts"}"#;
        let rows = parse_ndjson_to_rows(line.as_bytes());
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].status, "unknown");
    }

    /// GH #3763 — wrong-shape `duration_ms` (string instead of number)
    /// coerces to 0.
    #[test]
    fn gh3763_wrong_shape_duration_ms_coerced_to_zero() {
        let line = r#"{"test_id":"a","name":"a","status":"pass","duration_ms":"12","file":"a.ts"}"#;
        let rows = parse_ndjson_to_rows(line.as_bytes());
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].duration_ms, 0);
    }

    /// GH #3763 — wrong-shape `file` (array) coerces to empty string.
    #[test]
    fn gh3763_wrong_shape_file_field_coerced_to_empty() {
        let line = r#"{"test_id":"a","name":"a","status":"pass","duration_ms":1,"file":["a.ts"]}"#;
        let rows = parse_ndjson_to_rows(line.as_bytes());
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].file, "");
    }

    /// GH #3763 — wrong-shape `stack_trace` (number) is treated as None
    /// rather than panicking or coercing the number.
    #[test]
    fn gh3763_wrong_shape_optional_field_treated_as_none() {
        let line = r#"{"test_id":"a","name":"a","status":"pass","duration_ms":1,"file":"a.ts","stack_trace":42}"#;
        let rows = parse_ndjson_to_rows(line.as_bytes());
        assert_eq!(rows.len(), 1);
        assert!(rows[0].stack_trace.is_none());
    }

    /// GH #3763 — absent fields stay silent (no warn), and the row still
    /// builds with defaults. Distinguishes the absent path from the
    /// wrong-shape path.
    #[test]
    fn gh3763_absent_optional_field_silent() {
        let line = r#"{"test_id":"a","name":"a","status":"pass","duration_ms":1,"file":"a.ts"}"#;
        let rows = parse_ndjson_to_rows(line.as_bytes());
        assert_eq!(rows.len(), 1);
        assert!(rows[0].stack_trace.is_none());
        assert!(rows[0].matcher_diff.is_none());
        assert!(rows[0].trace_path.is_none());
    }

    /// GH #3763 — explicit-null fields stay silent (treated identically
    /// to absent).
    #[test]
    fn gh3763_explicit_null_optional_field_silent() {
        let line = r#"{"test_id":"a","name":"a","status":"pass","duration_ms":1,"file":"a.ts","stack_trace":null,"trace_path":null}"#;
        let rows = parse_ndjson_to_rows(line.as_bytes());
        assert_eq!(rows.len(), 1);
        assert!(rows[0].stack_trace.is_none());
        assert!(rows[0].trace_path.is_none());
    }

    /// GH #3763 — wrong-shape `test_id` still triggers the existing
    /// empty-id skip, but the warn now flags the producer instead of
    /// silently dropping the row.
    #[test]
    fn gh3763_wrong_shape_test_id_skipped_but_diagnosed() {
        let line = r#"{"test_id":42,"name":"a","status":"pass","duration_ms":1,"file":"a.ts"}"#;
        let rows = parse_ndjson_to_rows(line.as_bytes());
        assert!(
            rows.is_empty(),
            "wrong-shape test_id falls into the empty-id skip path"
        );
    }

    /// GH #3763 — issue-tag discoverability. Operators grep for "GH #3763"
    /// to chase producer schema drift; the helper messages must include
    /// the tag.
    #[test]
    fn gh3763_helpers_include_issue_tag() {
        assert!(format_reporter_parser_string_field_warn("name", "number").contains("GH #3763"));
        assert!(
            format_reporter_parser_u64_field_warn("duration_ms", "string").contains("GH #3763")
        );
        assert!(
            format_reporter_parser_optional_string_field_warn("stack_trace", "number")
                .contains("GH #3763")
        );
    }

    /// GH #3763 — siblings must be distinguishable from the GH #3355
    /// (per-line JSON parse) warn so operators can tell schema drift from
    /// a corrupted line.
    #[test]
    fn gh3763_warn_distinct_from_gh3355() {
        let shape = format_reporter_parser_string_field_warn("name", "number");
        assert!(shape.contains("GH #3763"));
        assert!(!shape.contains("GH #3355"));
    }

    /// GH #3763 — discoverable helper names (`format_reporter_parser_*`)
    /// must include `shape` so the warn family is grep-able with the same
    /// convention as sibling shape warns elsewhere in the codebase.
    #[test]
    fn gh3763_helpers_mention_wrong_shape_in_message() {
        let msg = format_reporter_parser_string_field_warn("name", "number");
        assert!(msg.contains("wrong shape"));
        assert!(msg.contains("`name`"));
        assert!(msg.contains("number"));
    }

    /// GH #3763 — message records both the expected and actual types so
    /// the operator can act without re-running with debug logging.
    #[test]
    fn gh3763_u64_warn_message_records_expected_and_actual() {
        let msg = format_reporter_parser_u64_field_warn("duration_ms", "string");
        assert!(msg.contains("expected unsigned integer"));
        assert!(msg.contains("got string"));
        assert!(msg.contains("duration_ms"));
    }

    /// GH #3763 — json_value_kind exposes a complete name for each
    /// serde_json value variant.
    #[test]
    fn gh3763_json_value_kind_covers_all_variants() {
        assert_eq!(json_value_kind(&serde_json::Value::Null), "null");
        assert_eq!(json_value_kind(&serde_json::Value::Bool(true)), "bool");
        assert_eq!(
            json_value_kind(&serde_json::Value::Number(serde_json::Number::from(7))),
            "number"
        );
        assert_eq!(
            json_value_kind(&serde_json::Value::String("x".into())),
            "string"
        );
        assert_eq!(
            json_value_kind(&serde_json::Value::Array(Vec::new())),
            "array"
        );
        assert_eq!(
            json_value_kind(&serde_json::Value::Object(serde_json::Map::new())),
            "object"
        );
    }
}
// CODEGEN-END
