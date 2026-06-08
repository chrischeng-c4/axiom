// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! PM report acceptance — packaged static report hides dev controls,
//! preserves enough failure context to be useful to non-developers,
//! and works from a plain static directory without `jet e2e open`.
//
// @spec #2621

use jet::e2e::{
    E2eArtifactRef, E2eAssertionDetail, E2eCaseEvidence, E2eEvidenceBundle, E2eMode,
    E2eProductStep, E2eStepContext, E2eSummary, EVIDENCE_SCHEMA_VERSION,
};
use jet::report_package::package_static_report;
use std::path::PathBuf;

fn pm_fixture_bundle() -> E2eEvidenceBundle {
    E2eEvidenceBundle {
        schema_version: EVIDENCE_SCHEMA_VERSION.to_string(),
        mode: E2eMode::Run,
        run_id: "pm-fixture".to_string(),
        started_at_ms: 1_700_000_000_000,
        finished_at_ms: 1_700_000_001_500,
        summary: E2eSummary {
            passed: 1,
            failed: 1,
            skipped: 0,
            duration_ms: 1_500,
            exit_code: 1,
        },
        cases: vec![
            E2eCaseEvidence {
                id: "case-0001".to_string(),
                title: "Cue Artifact Studio > creates a project".to_string(),
                file: PathBuf::from("examples/jet-e2e-demo/cue-artifact-studio.spec.js"),
                outcome: "passed".to_string(),
                duration_ms: 400,
                steps: vec![E2eProductStep {
                    id: "step-0001".to_string(),
                    title: "open studio".to_string(),
                    status: "passed".to_string(),
                    duration_ms: 200,
                    assertion: None,
                    context: E2eStepContext::default(),
                }],
            },
            E2eCaseEvidence {
                id: "case-0002".to_string(),
                title: "Cue Artifact Studio > promotes a work item".to_string(),
                file: PathBuf::from("examples/jet-e2e-demo/cue-artifact-studio.spec.js"),
                outcome: "failed".to_string(),
                duration_ms: 1_100,
                steps: vec![E2eProductStep {
                    id: "step-0001".to_string(),
                    title: "publish artifact".to_string(),
                    status: "failed".to_string(),
                    duration_ms: 900,
                    assertion: Some(E2eAssertionDetail {
                        message: "Expected work-state 'shipped', got 'reviewing'".to_string(),
                        stack: Some("at e2e/cue.spec.js:60:5".to_string()),
                        diff: Some("- shipped\n+ reviewing".to_string()),
                    }),
                    context: E2eStepContext {
                        screenshots: vec![E2eArtifactRef {
                            kind: "screenshot".to_string(),
                            path: PathBuf::from("shots/promote.png"),
                            label: Some("publish failure".to_string()),
                        }],
                        ..Default::default()
                    },
                }],
            },
        ],
        artifacts: vec![],
        open_control: None,
    }
}

fn setup_source_with_screenshot() -> tempfile::TempDir {
    let src = tempfile::tempdir().unwrap();
    let shots = src.path().join("shots");
    std::fs::create_dir_all(&shots).unwrap();
    std::fs::write(shots.join("promote.png"), b"fake-png-bytes").unwrap();
    src
}

#[test]
fn pm_report_hides_pause_next_replay_dev_controls() {
    // Acceptance: a PM-facing report must not include any control that
    // implies local-runner ownership. Buttons are visually present (the
    // DOM is shared between modes) but the `disabled` attribute and the
    // `read-only` mode flag together neutralise them.
    let src = setup_source_with_screenshot();
    let out = tempfile::tempdir().unwrap();
    let pkg = package_static_report(&pm_fixture_bundle(), src.path(), out.path()).expect("package");
    let html = std::fs::read_to_string(&pkg.index_html).unwrap();

    // Mode flag.
    assert!(
        html.contains("data-mode=\"pm-report\""),
        "report html must declare pm-report mode",
    );
    // No live-control endpoint embedded.
    assert!(
        !html.contains("/api/live-control"),
        "report must not embed a live-control endpoint",
    );
    // CSS hides the dev toolbar and command log in PM mode (controls
    // are not just disabled, they are visually absent for non-devs).
    assert!(
        html.contains("body[data-mode=\"pm-report\"] .toolbar"),
        "pm-report mode must hide the dev toolbar via CSS",
    );
    assert!(
        html.contains("body[data-mode=\"pm-report\"] #commands"),
        "pm-report mode must hide the dev command log via CSS",
    );

    // Every control button is disabled.
    for needle in [
        "data-action=\"pause\"",
        "data-action=\"next\"",
        "data-action=\"replay\"",
    ] {
        if let Some(idx) = html.find(needle) {
            // Inspect the surrounding button tag to confirm `disabled`
            // is present. The renderer attaches the attribute inside
            // the same tag.
            let tag_start = html[..idx].rfind('<').unwrap();
            let tag_end = html[idx..].find('>').unwrap() + idx;
            let tag = &html[tag_start..=tag_end];
            assert!(
                tag.contains("disabled"),
                "control {needle} must be disabled in PM report, got tag: {tag}",
            );
        }
    }
}

#[test]
fn pm_report_preserves_failure_screenshot_assertion_and_step_context() {
    // Acceptance: failure context survives packaging so non-developers
    // can read the assertion message, see the diff, and inspect the
    // captured screenshot.
    let src = setup_source_with_screenshot();
    let out = tempfile::tempdir().unwrap();
    let pkg = package_static_report(&pm_fixture_bundle(), src.path(), out.path()).expect("package");

    // Embedded bundle on disk carries the diff and the message.
    let bundle_text = std::fs::read_to_string(&pkg.bundle_json).unwrap();
    assert!(bundle_text.contains("Expected work-state"));
    assert!(bundle_text.contains("- shipped"));
    assert!(bundle_text.contains("publish artifact"));

    // Index HTML inlines bundle JSON for the read-only adapter, so the
    // same details show up there as well.
    let html = std::fs::read_to_string(&pkg.index_html).unwrap();
    assert!(html.contains("Expected work-state"));
    assert!(html.contains("publish artifact"));

    // Screenshot copied and reachable via the embedded relative path.
    assert_eq!(pkg.copied_artifacts.len(), 1);
    let copied = &pkg.copied_artifacts[0];
    assert!(copied.absolute.exists());
    assert!(copied.relative.starts_with("artifacts"));
    let resolved = pkg.report_dir.join(&copied.relative);
    assert!(resolved.exists(), "embedded relative path resolves on disk");
}

#[test]
fn pm_report_works_from_static_files_without_runner_process() {
    // Acceptance: the report directory is self-contained and the HTML
    // can be opened from disk (file://) without any helper process.
    let src = setup_source_with_screenshot();
    let out = tempfile::tempdir().unwrap();
    let pkg = package_static_report(&pm_fixture_bundle(), src.path(), out.path()).expect("package");

    // All paths in the packaged bundle are relative — the package can
    // be moved or zipped and the report still resolves its assets.
    let bundle_text = std::fs::read_to_string(&pkg.bundle_json).unwrap();
    let parsed: E2eEvidenceBundle = serde_json::from_str(&bundle_text).unwrap();
    for case in &parsed.cases {
        for step in &case.steps {
            for shot in &step.context.screenshots {
                assert!(
                    !shot.path.is_absolute(),
                    "screenshot path in packaged bundle must be relative: {:?}",
                    shot.path,
                );
            }
        }
    }

    // The HTML does not require localhost/HTTP — it works off
    // file:// origin. We assert this by checking nothing inside the
    // page makes a live-runner fetch (the read-only mode flag and the
    // missing /api endpoint are the contract).
    let html = std::fs::read_to_string(&pkg.index_html).unwrap();
    assert!(html.contains("data-mode=\"pm-report\""));
    assert!(!html.contains("ws://"));
    assert!(!html.contains("/api/live-control"));
}

#[test]
fn pm_report_carries_no_open_control_protocol() {
    // Open-mode control protocol is the "I am the live runner" handle.
    // It must not appear in a PM report bundle.
    let src = setup_source_with_screenshot();
    let out = tempfile::tempdir().unwrap();
    let pkg = package_static_report(&pm_fixture_bundle(), src.path(), out.path()).expect("package");
    let bundle_text = std::fs::read_to_string(&pkg.bundle_json).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&bundle_text).unwrap();
    assert!(
        parsed.get("open_control").is_none() || parsed["open_control"] == serde_json::Value::Null,
        "PM report bundle must not carry open_control protocol, got: {:?}",
        parsed.get("open_control"),
    );
}
// CODEGEN-END
