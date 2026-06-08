// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Integration test for the product.step timeline API.
//!
//! Acceptance criteria from #2617:
//!   - A dogfood E2E case uses named product steps.
//!   - Evidence contains ordered step records.
//!   - Open mode timeline can consume the same records.
//
// @spec #2617

use jet::e2e::{
    events_for_bundle, E2eAssertionDetail, E2eCaseEvidence, E2eEvidenceBundle, E2eEvidenceEvent,
    E2eMode, E2eProductStep, E2eStepContext, E2eSummary, EVIDENCE_SCHEMA_VERSION,
};
use std::path::PathBuf;

fn cue_step(id: &str, title: &str, status: &str, duration_ms: u64) -> E2eProductStep {
    E2eProductStep {
        id: id.to_string(),
        title: title.to_string(),
        status: status.to_string(),
        duration_ms,
        assertion: None,
        context: E2eStepContext::default(),
    }
}

fn cue_artifact_studio_bundle(mode: E2eMode) -> E2eEvidenceBundle {
    E2eEvidenceBundle {
        schema_version: EVIDENCE_SCHEMA_VERSION.to_string(),
        mode,
        run_id: "cue-studio-run".to_string(),
        started_at_ms: 100,
        finished_at_ms: 900,
        summary: E2eSummary {
            passed: 1,
            failed: 1,
            skipped: 0,
            duration_ms: 800,
            exit_code: 1,
        },
        cases: vec![
            E2eCaseEvidence {
                id: "case-0001".to_string(),
                title: "Cue Artifact Studio > creates a project".to_string(),
                file: PathBuf::from("examples/jet-e2e-demo/cue-artifact-studio.spec.js"),
                outcome: "passed".to_string(),
                duration_ms: 300,
                steps: vec![
                    cue_step("step-0001", "product.step: open studio", "passed", 100),
                    cue_step("step-0002", "product.step: create project", "passed", 100),
                    cue_step(
                        "step-0003",
                        "product.step: verify work item visible",
                        "passed",
                        100,
                    ),
                ],
            },
            E2eCaseEvidence {
                id: "case-0002".to_string(),
                title: "Cue Artifact Studio > promotes a work item".to_string(),
                file: PathBuf::from("examples/jet-e2e-demo/cue-artifact-studio.spec.js"),
                outcome: "failed".to_string(),
                duration_ms: 500,
                steps: vec![
                    cue_step("step-0001", "product.step: open studio", "passed", 80),
                    cue_step(
                        "step-0002",
                        "product.step: promote work item",
                        "passed",
                        120,
                    ),
                    E2eProductStep {
                        id: "step-0003".to_string(),
                        title: "product.step: publish artifact".to_string(),
                        status: "failed".to_string(),
                        duration_ms: 300,
                        assertion: Some(E2eAssertionDetail {
                            message: "Expected work-state 'shipped', got 'reviewing'".to_string(),
                            stack: None,
                            diff: Some("- shipped\n+ reviewing".to_string()),
                        }),
                        context: E2eStepContext::default(),
                    },
                ],
            },
        ],
        artifacts: vec![],
        open_control: None,
    }
}

#[test]
fn dogfood_case_uses_named_product_steps() {
    let bundle = cue_artifact_studio_bundle(E2eMode::Run);
    for case in &bundle.cases {
        assert!(
            case.steps.len() >= 2,
            "case '{}' must carry multiple product steps",
            case.title
        );
        for step in &case.steps {
            assert!(
                step.title.starts_with("product.step:"),
                "every step title is a product-language label, got {:?}",
                step.title,
            );
        }
    }
}

#[test]
fn evidence_carries_ordered_step_records() {
    let bundle = cue_artifact_studio_bundle(E2eMode::Run);

    let case2 = &bundle.cases[1];
    let titles: Vec<&str> = case2.steps.iter().map(|s| s.title.as_str()).collect();
    assert_eq!(
        titles,
        vec![
            "product.step: open studio",
            "product.step: promote work item",
            "product.step: publish artifact",
        ],
        "step records preserve script order",
    );
    assert_eq!(case2.steps.last().unwrap().status, "failed");
}

#[test]
fn open_mode_timeline_consumes_same_step_records_as_run_mode() {
    let run_events: Vec<&'static str> =
        events_for_bundle(&cue_artifact_studio_bundle(E2eMode::Run))
            .iter()
            .map(kind)
            .collect();
    let open_events: Vec<&'static str> =
        events_for_bundle(&cue_artifact_studio_bundle(E2eMode::Open))
            .iter()
            .map(kind)
            .collect();
    assert_eq!(run_events, open_events);

    let step_starts = run_events.iter().filter(|k| **k == "step_started").count();
    let step_finishes = run_events.iter().filter(|k| **k == "step_finished").count();
    assert_eq!(step_starts, 6, "3 + 3 steps across both cases");
    assert_eq!(step_finishes, 6, "every start has a matching finish");
}

#[test]
fn step_events_carry_start_end_duration_and_failure_context() {
    let bundle = cue_artifact_studio_bundle(E2eMode::Run);
    let events = events_for_bundle(&bundle);

    let mut saw_start = false;
    let mut saw_finish_with_duration = false;
    let mut saw_failed_with_assertion = false;
    for ev in &events {
        match ev {
            E2eEvidenceEvent::StepStarted { ts_ms, .. } => {
                assert!(*ts_ms >= bundle.started_at_ms);
                saw_start = true;
            }
            E2eEvidenceEvent::StepFinished {
                status,
                duration_ms,
                ts_ms,
                assertion,
                ..
            } => {
                assert!(*ts_ms >= bundle.started_at_ms);
                if *duration_ms > 0 {
                    saw_finish_with_duration = true;
                }
                if status == "failed" {
                    let a = assertion.as_ref().expect("failure carries assertion");
                    assert!(a.message.contains("work-state"));
                    assert!(a.diff.as_deref().unwrap().contains("shipped"));
                    saw_failed_with_assertion = true;
                }
            }
            _ => {}
        }
    }
    assert!(saw_start, "at least one StepStarted event");
    assert!(saw_finish_with_duration, "StepFinished carries duration");
    assert!(
        saw_failed_with_assertion,
        "failure path carries assertion context",
    );
}

fn kind(e: &E2eEvidenceEvent) -> &'static str {
    match e {
        E2eEvidenceEvent::RunStarted { .. } => "run_started",
        E2eEvidenceEvent::StepStarted { .. } => "step_started",
        E2eEvidenceEvent::StepFinished { .. } => "step_finished",
        E2eEvidenceEvent::CaseFinished { .. } => "case_finished",
        E2eEvidenceEvent::RunFinished { .. } => "run_finished",
    }
}
// CODEGEN-END
