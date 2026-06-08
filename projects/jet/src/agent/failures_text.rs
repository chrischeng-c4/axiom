// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-agent.md#schema
// CODEGEN-BEGIN
//! Compact failures-only text adapter for agents (#2873).
//!
//! This is an **adapter over evidence**: it consumes an
//! [`EvidenceEvent`] stream produced by `jet test` / `jet e2e` and
//! emits a tight, ASCII-only block that an agent can read in a single
//! glance. Every failure occupies one line and carries enough metadata
//! to identify the failing item and a copy/paste rerun command when one
//! is attached (#2874 lands the rerun-hint field; this adapter degrades
//! cleanly until then).
//!
//! Output shape:
//!
//! ```text
//! FAIL Suite > A > flaky_test    (12ms)
//! FAIL e2e: checkout flow         (250ms)
//! 2 failed of 4 (1 passed, 1 skipped)
//! ```
//!
//! Passing runs return an explicit `OK` line so a caller can tell
//! "empty output" (broken pipe) apart from "no failures" (clean run).
//!
//! @spec #2873

use crate::evidence_writer::EvidenceEvent;
use std::fmt::Write;

/// Render a compact, failures-only text block from an event stream.
///
/// Skipped and passing items are folded into the trailing counts line
/// only — the agent sees the failures itemised. When the run has no
/// failures, the output is a single `OK` line so the consumer can
/// tell "nothing failed" apart from an empty stream.
/// @spec .aw/tech-design/projects/jet/semantic/jet-agent.md#schema
pub fn render_failures(events: &[EvidenceEvent]) -> String {
    let mut out = String::new();
    let mut passed = 0u32;
    let mut failed = 0u32;
    let mut skipped = 0u32;
    let mut retry_attempts = 0u32;

    for e in events {
        match e {
            EvidenceEvent::TestResult {
                suite,
                name,
                outcome,
                duration_ms,
            } => match outcome.as_str() {
                "passed" => passed += 1,
                "skipped" => skipped += 1,
                _ => {
                    failed += 1;
                    let suite_path = if suite.is_empty() {
                        String::new()
                    } else {
                        format!("{} > ", suite.join(" > "))
                    };
                    let _ = writeln!(out, "FAIL {suite_path}{name}    ({duration_ms}ms)");
                }
            },
            EvidenceEvent::CaseResult {
                case,
                outcome,
                duration_ms,
            } => match outcome.as_str() {
                "passed" => passed += 1,
                "skipped" => skipped += 1,
                _ => {
                    failed += 1;
                    let _ = writeln!(out, "FAIL e2e: {case}         ({duration_ms}ms)");
                }
            },
            // Per-attempt retry records (#2868) — counted into the
            // trailing summary so triage can see the cost of flake.
            EvidenceEvent::TestRetry { .. } | EvidenceEvent::CaseRetry { .. } => {
                retry_attempts += 1;
            }
            _ => {}
        }
    }

    if failed == 0 && retry_attempts == 0 {
        return "OK\n".to_string();
    }
    if failed == 0 {
        // Clean run modulo flake — surface the retry cost explicitly.
        let _ = writeln!(
            out,
            "OK (with {retry_attempts} retry attempt{})",
            plural(retry_attempts)
        );
        return out;
    }

    let total = passed + failed + skipped;
    let retry_suffix = if retry_attempts > 0 {
        format!(", {retry_attempts} retry attempt{}", plural(retry_attempts))
    } else {
        String::new()
    };
    let _ = writeln!(
        out,
        "{failed} failed of {total} ({passed} passed, {skipped} skipped{retry_suffix})"
    );
    out
}

fn plural(n: u32) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev_test(name: &str, outcome: &str, ms: u64) -> EvidenceEvent {
        EvidenceEvent::TestResult {
            suite: vec!["Suite".into(), "A".into()],
            name: name.into(),
            outcome: outcome.into(),
            duration_ms: ms,
        }
    }

    fn ev_case(case: &str, outcome: &str, ms: u64) -> EvidenceEvent {
        EvidenceEvent::CaseResult {
            case: case.into(),
            outcome: outcome.into(),
            duration_ms: ms,
        }
    }

    #[test]
    fn failing_fixture_emits_concise_failure_block() {
        let events = vec![
            ev_test("ok", "passed", 1),
            ev_test("boom", "failed", 12),
            ev_case("checkout flow", "failed", 250),
            ev_test("skipme", "skipped", 0),
            EvidenceEvent::RunFinished {
                passed: 1,
                failed: 2,
                skipped: 1,
                duration_ms: 263,
            },
        ];
        let txt = render_failures(&events);
        // Failing items are itemised, passing/skipped are not.
        assert!(txt.contains("FAIL Suite > A > boom"), "{txt}");
        assert!(txt.contains("FAIL e2e: checkout flow"), "{txt}");
        assert!(!txt.contains("ok"), "{txt}"); // passing test stays out
        assert!(!txt.contains("skipme"), "{txt}"); // skipped stays out
                                                   // Trailing counts line agrees with what's in the stream.
        assert!(txt.contains("2 failed of 4 (1 passed, 1 skipped)"), "{txt}");
    }

    #[test]
    fn passing_run_emits_explicit_ok() {
        // Spec: passing runs produce an empty or explicitly no-failures
        // output. We pick "explicit OK" so a broken pipe is distinguishable.
        let events = vec![
            ev_test("a", "passed", 1),
            ev_test("b", "passed", 2),
            EvidenceEvent::RunFinished {
                passed: 2,
                failed: 0,
                skipped: 0,
                duration_ms: 3,
            },
        ];
        assert_eq!(render_failures(&events), "OK\n");
    }

    #[test]
    fn unknown_outcomes_are_treated_as_failures() {
        // Producers may emit framework-specific outcomes like "errored"
        // or "timed_out"; the adapter must surface them, not drop them.
        let events = vec![
            ev_test("hang", "timed_out", 30000),
            EvidenceEvent::RunFinished {
                passed: 0,
                failed: 1,
                skipped: 0,
                duration_ms: 30000,
            },
        ];
        let txt = render_failures(&events);
        assert!(txt.contains("FAIL Suite > A > hang"), "{txt}");
        assert!(txt.contains("(30000ms)"), "{txt}");
    }

    #[test]
    fn retry_attempts_are_summarised_in_counts_line() {
        // @spec #2868 — failures.txt must surface the retry count so
        // an agent can spot a flake-heavy run from one glance.
        let events = vec![
            // Final-fail with two failed retry attempts before the terminal record.
            EvidenceEvent::TestRetry {
                suite: vec!["Suite".into(), "A".into()],
                name: "always_bad".into(),
                attempt: 1,
                outcome: "failed".into(),
                duration_ms: 5,
            },
            EvidenceEvent::TestRetry {
                suite: vec!["Suite".into(), "A".into()],
                name: "always_bad".into(),
                attempt: 2,
                outcome: "failed".into(),
                duration_ms: 6,
            },
            EvidenceEvent::TestResult {
                suite: vec!["Suite".into(), "A".into()],
                name: "always_bad".into(),
                outcome: "failed".into(),
                duration_ms: 7,
            },
            EvidenceEvent::RunFinished {
                passed: 0,
                failed: 1,
                skipped: 0,
                duration_ms: 18,
            },
        ];
        let txt = render_failures(&events);
        assert!(txt.contains("FAIL Suite > A > always_bad"), "{txt}");
        assert!(
            txt.contains("1 failed of 1 (0 passed, 0 skipped, 2 retry attempts)"),
            "retry count must appear in counts line: {txt}",
        );
    }

    #[test]
    fn retry_pass_only_emits_ok_with_retry_count() {
        // @spec #2868 — a flaky test that eventually passed must not
        // show up as FAIL, but the retry cost must still be visible.
        let events = vec![
            EvidenceEvent::TestRetry {
                suite: vec!["Suite".into()],
                name: "flaky".into(),
                attempt: 1,
                outcome: "failed".into(),
                duration_ms: 4,
            },
            EvidenceEvent::TestResult {
                suite: vec!["Suite".into()],
                name: "flaky".into(),
                outcome: "passed".into(),
                duration_ms: 3,
            },
            EvidenceEvent::RunFinished {
                passed: 1,
                failed: 0,
                skipped: 0,
                duration_ms: 7,
            },
        ];
        assert_eq!(render_failures(&events), "OK (with 1 retry attempt)\n");
    }

    #[test]
    fn rendering_is_fast_for_large_failure_lists() {
        // Perf gate: 1000 failures must render well under 50ms — agents
        // read this synchronously from a pipe.
        let mut events: Vec<EvidenceEvent> = (0..1000)
            .map(|i| ev_test(&format!("case_{i}"), "failed", 1))
            .collect();
        events.push(EvidenceEvent::RunFinished {
            passed: 0,
            failed: 1000,
            skipped: 0,
            duration_ms: 1000,
        });
        let start = std::time::Instant::now();
        let txt = render_failures(&events);
        let elapsed = start.elapsed();
        assert!(
            elapsed < std::time::Duration::from_millis(50),
            "render_failures too slow on 1000 failures: {elapsed:?}",
        );
        assert!(txt.contains("1000 failed of 1000"), "{txt}");
    }
}
// CODEGEN-END
