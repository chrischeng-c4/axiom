// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-src.md#schema
// CODEGEN-BEGIN
//! CI summary projection over a finalised evidence bundle (#2719).
//!
//! This is an **adapter over evidence**, not a runner-specific code
//! path: it never invokes a runner, never opens a browser, and works
//! after the bundle is finalised. Pair it with [`EvidenceWriter`] +
//! [`BundleHandle`] to project the same bundle into a CI-friendly
//! markdown summary that GitHub Actions can fold into a job-summary
//! step.
//!
//! @spec #2719

use crate::evidence_bundle::BundleHandle;
use crate::evidence_writer::EvidenceEvent;

/// Render a concise markdown summary from a finalised bundle handle
/// plus the event stream that produced it.
///
/// The output is deterministic and stable across invocations on the
/// same evidence — CI scrapers can rely on the section order and the
/// "Failed tests" list shape. Tests/cases that did not fail are
/// rolled into the counts line; only failures are itemised.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub fn render_summary(handle: &BundleHandle, events: &[EvidenceEvent]) -> String {
    let manifest = handle.manifest();
    let mut out = String::new();

    out.push_str("# jet test summary\n\n");
    out.push_str(&format!("- Project: `{}`\n", manifest.project));
    out.push_str(&format!("- Commit: `{}`\n", manifest.commit));
    out.push_str(&format!("- Run id: `{}`\n", manifest.run_id));
    out.push_str(&format!(
        "- Command: `{}`\n",
        match manifest.command {
            crate::evidence_bundle::BundleCommand::Test => "jet test",
            crate::evidence_bundle::BundleCommand::E2e => "jet e2e",
        }
    ));

    let mut counts = (0u32, 0u32, 0u32, 0u64); // (passed, failed, skipped, duration_ms)
    for event in events {
        if let EvidenceEvent::RunFinished {
            passed,
            failed,
            skipped,
            duration_ms,
        } = event
        {
            counts = (*passed, *failed, *skipped, *duration_ms);
        }
    }
    out.push_str(&format!(
        "\n**Results:** {} passed, {} failed, {} skipped — {}ms total\n",
        counts.0, counts.1, counts.2, counts.3,
    ));

    let failures: Vec<&EvidenceEvent> = events
        .iter()
        .filter(|e| match e {
            EvidenceEvent::TestResult { outcome, .. }
            | EvidenceEvent::CaseResult { outcome, .. } => {
                outcome != "passed" && outcome != "skipped"
            }
            _ => false,
        })
        .collect();

    if failures.is_empty() {
        out.push_str("\nNo failures.\n");
    } else {
        out.push_str("\n## Failed tests\n\n");
        for failure in failures {
            let line = match failure {
                EvidenceEvent::TestResult {
                    suite,
                    name,
                    outcome,
                    duration_ms,
                } => {
                    let suite_path = if suite.is_empty() {
                        String::new()
                    } else {
                        format!("{} > ", suite.join(" > "))
                    };
                    format!("- **{outcome}** `{suite_path}{name}` ({duration_ms}ms)\n",)
                }
                EvidenceEvent::CaseResult {
                    case,
                    outcome,
                    duration_ms,
                } => format!("- **{outcome}** `{case}` ({duration_ms}ms)\n"),
                _ => String::new(),
            };
            out.push_str(&line);
        }
    }

    let artifacts = &manifest.artifacts;
    if !artifacts.is_empty() {
        out.push_str("\n## Artifacts\n\n");
        for a in artifacts {
            out.push_str(&format!(
                "- `{}` ({}) — `{}`\n",
                a.id,
                a.kind,
                a.path.display()
            ));
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence_bundle::{BundleCommand, BundleEnvironment, BundleManifest};
    use crate::evidence_writer::EvidenceWriter;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn manifest_for(command: BundleCommand) -> BundleManifest {
        BundleManifest::new(
            "run-fixture",
            command,
            "jet",
            "abc1234",
            BundleEnvironment {
                os: "darwin".into(),
                runner_version: "0.3.48".into(),
                ci: Some("github".into()),
                node_version: Some("v20.10.0".into()),
            },
        )
    }

    #[test]
    fn render_summary_emits_concise_markdown_for_a_finalised_bundle() {
        let tmp = TempDir::new().unwrap();
        let mut w = EvidenceWriter::open(tmp.path(), manifest_for(BundleCommand::Test)).unwrap();
        w.run_started("run-fixture", "jet test").unwrap();
        w.test_result(vec!["Outer".into()], "passes", "passed", 5)
            .unwrap();
        w.test_result(vec!["Outer".into()], "boom", "failed", 12)
            .unwrap();
        w.run_finished(1, 1, 0, 20).unwrap();
        let handle = w.finalize().unwrap();

        let events = vec![
            EvidenceEvent::RunStarted {
                run_id: "run-fixture".into(),
                command: "jet test".into(),
            },
            EvidenceEvent::TestResult {
                suite: vec!["Outer".into()],
                name: "passes".into(),
                outcome: "passed".into(),
                duration_ms: 5,
            },
            EvidenceEvent::TestResult {
                suite: vec!["Outer".into()],
                name: "boom".into(),
                outcome: "failed".into(),
                duration_ms: 12,
            },
            EvidenceEvent::RunFinished {
                passed: 1,
                failed: 1,
                skipped: 0,
                duration_ms: 20,
            },
        ];

        let md = render_summary(&handle, &events);
        // Header is stable, project + commit appear, counts are rendered.
        assert!(md.starts_with("# jet test summary\n"), "{md}");
        assert!(md.contains("Project: `jet`"), "{md}");
        assert!(md.contains("Commit: `abc1234`"), "{md}");
        assert!(md.contains("Run id: `run-fixture`"), "{md}");
        assert!(
            md.contains("**Results:** 1 passed, 1 failed, 0 skipped — 20ms total"),
            "{md}"
        );
        // Only the failure is itemised (passing test stays in the counts).
        assert!(md.contains("## Failed tests"), "{md}");
        assert!(md.contains("`Outer > boom`"), "{md}");
        assert!(!md.contains("`Outer > passes`"), "{md}");
    }

    #[test]
    fn render_summary_says_no_failures_when_clean() {
        let tmp = TempDir::new().unwrap();
        let mut w = EvidenceWriter::open(tmp.path(), manifest_for(BundleCommand::Test)).unwrap();
        w.test_result(vec![], "ok", "passed", 1).unwrap();
        w.run_finished(1, 0, 0, 1).unwrap();
        let handle = w.finalize().unwrap();

        let events = vec![
            EvidenceEvent::TestResult {
                suite: vec![],
                name: "ok".into(),
                outcome: "passed".into(),
                duration_ms: 1,
            },
            EvidenceEvent::RunFinished {
                passed: 1,
                failed: 0,
                skipped: 0,
                duration_ms: 1,
            },
        ];

        let md = render_summary(&handle, &events);
        assert!(md.contains("No failures."), "{md}");
        assert!(!md.contains("## Failed tests"), "{md}");
    }

    #[test]
    fn render_summary_groups_e2e_case_failures_under_same_heading() {
        let tmp = TempDir::new().unwrap();
        let mut w = EvidenceWriter::open(tmp.path(), manifest_for(BundleCommand::E2e)).unwrap();
        w.case_result("login flow", "failed", 200).unwrap();
        w.run_finished(0, 1, 0, 200).unwrap();
        let handle = w.finalize().unwrap();

        let events = vec![
            EvidenceEvent::CaseResult {
                case: "login flow".into(),
                outcome: "failed".into(),
                duration_ms: 200,
            },
            EvidenceEvent::RunFinished {
                passed: 0,
                failed: 1,
                skipped: 0,
                duration_ms: 200,
            },
        ];

        let md = render_summary(&handle, &events);
        assert!(md.contains("Command: `jet e2e`"), "{md}");
        assert!(md.contains("## Failed tests"), "{md}");
        assert!(md.contains("`login flow`"), "{md}");
    }

    #[test]
    fn render_summary_is_fast_for_large_failure_lists() {
        // Perf gate: rendering 1000 failures must stay well under 100ms.
        let tmp = TempDir::new().unwrap();
        let mut w = EvidenceWriter::open(tmp.path(), manifest_for(BundleCommand::Test)).unwrap();
        for i in 0..1000 {
            w.test_result(vec!["Suite".into()], &format!("case_{i}"), "failed", 1)
                .unwrap();
        }
        w.run_finished(0, 1000, 0, 1000).unwrap();
        let handle = w.finalize().unwrap();

        let mut events: Vec<EvidenceEvent> = (0..1000)
            .map(|i| EvidenceEvent::TestResult {
                suite: vec!["Suite".into()],
                name: format!("case_{i}"),
                outcome: "failed".into(),
                duration_ms: 1,
            })
            .collect();
        events.push(EvidenceEvent::RunFinished {
            passed: 0,
            failed: 1000,
            skipped: 0,
            duration_ms: 1000,
        });

        let start = std::time::Instant::now();
        let md = render_summary(&handle, &events);
        let elapsed = start.elapsed();
        assert!(
            elapsed < std::time::Duration::from_millis(100),
            "render_summary too slow on 1000 failures: {elapsed:?}",
        );
        assert!(md.contains("1000 failed"), "{md}");
    }

    #[test]
    fn render_summary_lists_registered_artifacts() {
        let tmp = TempDir::new().unwrap();
        let mut w = EvidenceWriter::open(tmp.path(), manifest_for(BundleCommand::Test)).unwrap();
        w.register_artifact(
            "page-1",
            "screenshot",
            PathBuf::from("artifacts/page-1.png"),
        )
        .unwrap();
        w.run_finished(0, 0, 0, 0).unwrap();
        let handle = w.finalize().unwrap();

        let md = render_summary(
            &handle,
            &[EvidenceEvent::RunFinished {
                passed: 0,
                failed: 0,
                skipped: 0,
                duration_ms: 0,
            }],
        );
        assert!(md.contains("## Artifacts"), "{md}");
        assert!(md.contains("`page-1`"), "{md}");
        assert!(md.contains("`artifacts/page-1.png`"), "{md}");
    }
}
// CODEGEN-END
