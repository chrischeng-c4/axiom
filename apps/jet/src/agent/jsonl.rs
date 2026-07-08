// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-agent.md#schema
// CODEGEN-BEGIN
//! Agent-readable JSONL event stream contract over the evidence bundle (#2872).
//!
//! Both `jet test` and `jet e2e` already write an append-only
//! `events.jsonl` next to a finalised `manifest.json` via
//! [`crate::evidence_writer::EvidenceWriter`]. This module is the
//! *agent-facing* surface: a stable read-side contract that documents
//! the one-object-per-line shape, exposes a parser, and proves the
//! final `RunFinished` envelope agrees with the per-event tally.
//!
//! ```text
//! <bundle-root>/events.jsonl   # one EvidenceEvent per line, append order
//! <bundle-root>/manifest.json  # finalised BundleManifest
//! ```
//!
//! Agents do not have to parse the manifest to obtain a summary —
//! [`read_stream`] + [`derive_summary`] reconstructs the same counts
//! the producer recorded in `RunFinished`. [`validate_summary_matches_finished`]
//! is the consistency check the spec calls out.
//!
//! @spec #2872

use crate::evidence_writer::{EvidenceEvent, EVENTS_FILE_NAME};
use anyhow::{anyhow, bail, Context, Result};
use std::path::Path;

/// Replayed counts derived from the per-event outcomes in the stream.
/// `duration_ms` is the sum of per-result durations (not wall time).
/// @spec .aw/tech-design/projects/jet/semantic/jet-agent.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReplaySummary {
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub duration_ms: u64,
}

/// Read an `events.jsonl` stream from the bundle root, parsing one
/// `EvidenceEvent` per line. Blank lines are skipped; any malformed
/// line fails loud with the 1-based line number so a producer bug is
/// easy to localise.
/// @spec .aw/tech-design/projects/jet/semantic/jet-agent.md#schema
pub fn read_stream(root: impl AsRef<Path>) -> Result<Vec<EvidenceEvent>> {
    let path = root.as_ref().join(EVENTS_FILE_NAME);
    let body = std::fs::read_to_string(&path)
        .with_context(|| format!("reading event stream at {}", path.display()))?;
    let mut events = Vec::new();
    for (idx, line) in body.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let event: EvidenceEvent = serde_json::from_str(line)
            .with_context(|| format!("parsing event stream line {}", idx + 1))?;
        events.push(event);
    }
    Ok(events)
}

/// Derive a replay summary from the per-event outcomes. Recognised
/// outcome strings are `"passed"`, `"failed"`, `"skipped"`; any other
/// outcome counts toward `failed` so the tally never silently loses a
/// non-passing result.
/// @spec .aw/tech-design/projects/jet/semantic/jet-agent.md#schema
pub fn derive_summary(events: &[EvidenceEvent]) -> ReplaySummary {
    let mut s = ReplaySummary::default();
    for e in events {
        let (outcome, duration_ms) = match e {
            EvidenceEvent::TestResult {
                outcome,
                duration_ms,
                ..
            }
            | EvidenceEvent::CaseResult {
                outcome,
                duration_ms,
                ..
            } => (outcome.as_str(), *duration_ms),
            _ => continue,
        };
        match outcome {
            "passed" => s.passed += 1,
            "skipped" => s.skipped += 1,
            _ => s.failed += 1,
        }
        s.duration_ms = s.duration_ms.saturating_add(duration_ms);
    }
    s
}

/// Extract the terminal `RunFinished` event. Returns an error when the
/// stream has none — an unfinished run cannot be summarised by an
/// agent.
/// @spec .aw/tech-design/projects/jet/semantic/jet-agent.md#schema
pub fn finished_summary(events: &[EvidenceEvent]) -> Result<ReplaySummary> {
    events
        .iter()
        .rev()
        .find_map(|e| match e {
            EvidenceEvent::RunFinished {
                passed,
                failed,
                skipped,
                duration_ms,
            } => Some(ReplaySummary {
                passed: *passed,
                failed: *failed,
                skipped: *skipped,
                duration_ms: *duration_ms,
            }),
            _ => None,
        })
        .ok_or_else(|| anyhow!("event stream has no RunFinished record"))
}

/// Spec acceptance: the final summary agrees with the finalized envelope.
/// Compares pass/fail/skipped counts. `duration_ms` is intentionally not
/// asserted — the producer records wall time, the replay sums per-result
/// time; the two need not match.
/// @spec .aw/tech-design/projects/jet/semantic/jet-agent.md#schema
pub fn validate_summary_matches_finished(events: &[EvidenceEvent]) -> Result<()> {
    let replayed = derive_summary(events);
    let finished = finished_summary(events)?;
    if replayed.passed != finished.passed
        || replayed.failed != finished.failed
        || replayed.skipped != finished.skipped
    {
        bail!(
            "RunFinished disagrees with per-event tally: \
             finished={{passed:{},failed:{},skipped:{}}} \
             replayed={{passed:{},failed:{},skipped:{}}}",
            finished.passed,
            finished.failed,
            finished.skipped,
            replayed.passed,
            replayed.failed,
            replayed.skipped,
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence_bundle::{BundleCommand, BundleEnvironment, BundleManifest};
    use crate::evidence_writer::EvidenceWriter;
    use tempfile::TempDir;

    fn manifest(cmd: BundleCommand) -> BundleManifest {
        BundleManifest::new(
            "run-jsonl",
            cmd,
            "jet",
            "cafef00d",
            BundleEnvironment {
                os: "darwin".into(),
                runner_version: "0.3.51".into(),
                ci: None,
                node_version: None,
            },
        )
    }

    #[test]
    fn jsonl_fixture_is_one_object_per_line_and_summary_agrees() {
        let tmp = TempDir::new().unwrap();
        let mut w = EvidenceWriter::open(tmp.path(), manifest(BundleCommand::Test)).unwrap();
        w.run_started("run-jsonl", "jet test").unwrap();
        w.test_result(vec!["Suite".into()], "a", "passed", 5)
            .unwrap();
        w.test_result(vec!["Suite".into()], "b", "failed", 7)
            .unwrap();
        w.test_result(vec!["Suite".into()], "c", "skipped", 0)
            .unwrap();
        w.run_finished(1, 1, 1, 20).unwrap();
        let handle = w.finalize().unwrap();

        let events = read_stream(handle.root()).unwrap();
        assert_eq!(events.len(), 5);

        // Per-line JSON validity: every line is a parseable object.
        let raw = std::fs::read_to_string(handle.root().join(EVENTS_FILE_NAME)).unwrap();
        for (i, line) in raw.lines().enumerate() {
            let v: serde_json::Value = serde_json::from_str(line)
                .unwrap_or_else(|e| panic!("line {} not valid JSON: {e}: {line}", i + 1));
            assert!(v.is_object(), "line {} is not a JSON object: {line}", i + 1);
        }

        validate_summary_matches_finished(&events).unwrap();
    }

    #[test]
    fn jsonl_fixture_works_for_e2e_case_results() {
        let tmp = TempDir::new().unwrap();
        let mut w = EvidenceWriter::open(tmp.path(), manifest(BundleCommand::E2e)).unwrap();
        w.run_started("run-jsonl", "jet e2e run").unwrap();
        w.case_result("login flow", "passed", 100).unwrap();
        w.case_result("checkout flow", "failed", 250).unwrap();
        w.run_finished(1, 1, 0, 360).unwrap();
        let handle = w.finalize().unwrap();

        let events = read_stream(handle.root()).unwrap();
        validate_summary_matches_finished(&events).unwrap();
        let replayed = derive_summary(&events);
        assert_eq!(replayed.passed, 1);
        assert_eq!(replayed.failed, 1);
        assert_eq!(replayed.skipped, 0);
    }

    #[test]
    fn unknown_outcome_strings_count_as_failed() {
        // Producers may emit framework-specific outcomes like "errored"
        // or "timed_out"; the tally must not silently drop them.
        let events = vec![
            EvidenceEvent::TestResult {
                suite: vec![],
                name: "x".into(),
                outcome: "errored".into(),
                duration_ms: 1,
            },
            EvidenceEvent::RunFinished {
                passed: 0,
                failed: 1,
                skipped: 0,
                duration_ms: 1,
            },
        ];
        validate_summary_matches_finished(&events).unwrap();
    }

    #[test]
    fn missing_run_finished_event_is_an_error() {
        let events = vec![EvidenceEvent::TestResult {
            suite: vec![],
            name: "x".into(),
            outcome: "passed".into(),
            duration_ms: 1,
        }];
        let err = format!("{:#}", finished_summary(&events).unwrap_err());
        assert!(err.contains("no RunFinished"), "{err}");
    }

    #[test]
    fn mismatched_finished_counts_fail_loud() {
        let events = vec![
            EvidenceEvent::TestResult {
                suite: vec![],
                name: "x".into(),
                outcome: "passed".into(),
                duration_ms: 1,
            },
            EvidenceEvent::RunFinished {
                passed: 0,
                failed: 0,
                skipped: 0,
                duration_ms: 1,
            },
        ];
        let err = format!(
            "{:#}",
            validate_summary_matches_finished(&events).unwrap_err()
        );
        assert!(err.contains("disagrees"), "{err}");
    }

    #[test]
    fn malformed_line_reports_one_based_line_number() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join(EVENTS_FILE_NAME);
        std::fs::write(
            &path,
            r#"{"kind":"run_started","run_id":"r","command":"jet test"}
not json
"#,
        )
        .unwrap();
        let err = format!("{:#}", read_stream(tmp.path()).unwrap_err());
        assert!(err.contains("line 2"), "{err}");
    }

    #[test]
    fn read_stream_is_fast_for_large_event_streams() {
        // Perf gate: parsing 10_000 events stays well under 250ms.
        let tmp = TempDir::new().unwrap();
        let mut w = EvidenceWriter::open(tmp.path(), manifest(BundleCommand::Test)).unwrap();
        for i in 0..10_000 {
            w.test_result(vec!["S".into()], format!("t{i}"), "passed", 1)
                .unwrap();
        }
        w.run_finished(10_000, 0, 0, 10_000).unwrap();
        let handle = w.finalize().unwrap();

        let start = std::time::Instant::now();
        let events = read_stream(handle.root()).unwrap();
        let elapsed = start.elapsed();
        assert_eq!(events.len(), 10_001);
        assert!(
            elapsed < std::time::Duration::from_millis(250),
            "read_stream too slow on 10k events: {elapsed:?}",
        );
        validate_summary_matches_finished(&events).unwrap();
    }
}
// CODEGEN-END
