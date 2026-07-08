// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Dogfood the e2e evidence pipeline against the Cue Artifact Studio
//! product flow fixture. Verifies that the same flow produces useful
//! evidence in both `jet e2e run` (agent/CI) and `jet e2e open` (dev
//! review) modes without depending on a live LLM or network.
//
// @spec #2615

use jet::e2e::{
    build_evidence_bundle, summary_exit_code, E2eEvidenceBundle, E2eMode, EVIDENCE_SCHEMA_VERSION,
};
use jet::test_runner::reporter::{Outcome, Summary, TestError, TestReport};
use std::path::{Path, PathBuf};

fn fixture_spec_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/jet-e2e-demo/cue-artifact-studio.spec.js")
        .canonicalize()
        .expect("cue-artifact-studio.spec.js must exist alongside the jet crate")
}

fn cue_summary_with(outcome_for_promote: Outcome) -> Summary {
    let mut promote_report = report(
        "Cue Artifact Studio",
        "promotes a work item into a published artifact",
        outcome_for_promote,
    );
    if matches!(outcome_for_promote, Outcome::Failed) {
        promote_report.error = Some(TestError {
            message: "Expected work-state to be 'shipped', got 'reviewing'".to_string(),
            stack: Some(
                "Error: Expected work-state to be 'shipped'\n    at /tmp/examples/cue-artifact-studio.spec.js:60:5"
                    .to_string(),
            ),
            diff: Some("- shipped\n+ reviewing".to_string()),
            source_location: None,
        });
    }
    Summary {
        schema_version: jet::test_runner::reporter::SCHEMA_VERSION,
        passed: if matches!(outcome_for_promote, Outcome::Passed) {
            2
        } else {
            1
        },
        failed: if matches!(outcome_for_promote, Outcome::Failed) {
            1
        } else {
            0
        },
        skipped: 0,
        duration_ms: 320,
        reports: vec![
            report(
                "Cue Artifact Studio",
                "creates a project and keeps the work item visible",
                Outcome::Passed,
            ),
            promote_report,
        ],
        coverage: None,
        browser_sessions: Vec::new(),
    }
}

fn report(suite: &str, name: &str, outcome: Outcome) -> TestReport {
    TestReport {
        file: PathBuf::from("examples/jet-e2e-demo/cue-artifact-studio.spec.js"),
        suite: vec![suite.to_string()],
        name: name.to_string(),
        outcome,
        duration_ms: 120,
        error: None,
        trace_path: None,
        shard_index: None,
        shard_total: None,
        artifacts: vec![],
        steps: Vec::new(),
    }
}

fn assert_bundle_shape(bundle: &E2eEvidenceBundle, expected_mode: E2eMode) {
    assert_eq!(bundle.schema_version, EVIDENCE_SCHEMA_VERSION);
    assert_eq!(bundle.mode, expected_mode);
    assert_eq!(bundle.cases.len(), 2, "two cue artifact studio cases");

    for case in &bundle.cases {
        assert!(
            !case.steps.is_empty(),
            "each case must carry at least one product step ({})",
            case.title
        );
        assert!(case.title.starts_with("Cue Artifact Studio"));
    }
}

#[test]
fn fixture_is_present_and_self_contained() {
    let path = fixture_spec_path();
    let body = std::fs::read_to_string(&path).expect("read fixture");
    assert!(body.contains("Cue Artifact Studio"));
    assert!(body.contains("creates a project"));
    assert!(body.contains("promotes a work item"));
    // No live LLM / network dependence — page.setContent only.
    assert!(
        !body.contains("page.goto("),
        "fixture must not navigate to external URLs"
    );
    assert!(
        !body.contains("fetch("),
        "fixture must not depend on live network"
    );
}

#[test]
fn run_mode_evidence_carries_product_steps_for_cue_flow() {
    let summary = cue_summary_with(Outcome::Passed);
    let bundle = build_evidence_bundle(
        E2eMode::Run,
        summary,
        1_700_000_000_000,
        1_700_000_000_320,
        None,
        None,
    );
    assert_bundle_shape(&bundle, E2eMode::Run);
    assert_eq!(bundle.summary.passed, 2);
    assert_eq!(bundle.summary.failed, 0);
    assert_eq!(summary_exit_code(&bundle), 0);
}

#[test]
fn open_mode_evidence_inspects_same_flow() {
    let summary = cue_summary_with(Outcome::Passed);
    let bundle = build_evidence_bundle(
        E2eMode::Open,
        summary,
        1_700_000_000_000,
        1_700_000_000_320,
        None,
        None,
    );
    assert_bundle_shape(&bundle, E2eMode::Open);
    // Open-mode evidence is consumable by the read-only review surface
    // without a live runner attached.
    assert!(
        bundle.open_control.is_none(),
        "no open_control wired in this fixture"
    );
}

#[test]
fn failure_path_carries_assertion_context() {
    let summary = cue_summary_with(Outcome::Failed);
    let bundle = build_evidence_bundle(E2eMode::Run, summary, 1, 321, None, None);
    assert_eq!(bundle.summary.failed, 1);
    assert_eq!(summary_exit_code(&bundle), 1);

    let failed = bundle
        .cases
        .iter()
        .find(|c| c.outcome == "failed")
        .expect("failed case present");
    let step = failed.steps.first().expect("at least one product step");
    let assertion = step
        .assertion
        .as_ref()
        .expect("failure has assertion context");
    assert!(assertion.message.contains("work-state"));
    assert!(
        assertion.diff.as_deref().unwrap_or("").contains("shipped"),
        "diff carries expected vs actual"
    );
}

#[test]
fn fixture_path_resolves_relative_to_jet_crate() {
    let path = fixture_spec_path();
    assert!(path.is_absolute());
    assert!(
        path.starts_with(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .parent()
                .unwrap()
        ),
        "fixture lives in the same checkout: {}",
        path.display()
    );
}
// CODEGEN-END
