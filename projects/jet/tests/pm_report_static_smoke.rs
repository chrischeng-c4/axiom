// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Static PM report smoke (#2735).
//!
//! Confirms that a packaged static PM report directory is fully
//! self-contained: no Jet daemon, no dev server, no desktop shell.
//! Everything the PM shell needs lives on disk under
//! `report_dir/{index.html, *.evidence.json, *.events.jsonl,
//! artifacts/*}`, and the failure context (assertion, screenshot,
//! console, network) survives the round-trip.
//
// @spec #2621

use jet::e2e::{
    E2eArtifactRef, E2eAssertionDetail, E2eCaseEvidence, E2eConsoleEntry, E2eEvidenceBundle,
    E2eMode, E2eNetworkEntry, E2eProductStep, E2eStepContext, E2eSummary, EVIDENCE_SCHEMA_VERSION,
};
use jet::report_package::package_static_report;
use std::path::PathBuf;

fn fixture_with_artifacts() -> E2eEvidenceBundle {
    E2eEvidenceBundle {
        schema_version: EVIDENCE_SCHEMA_VERSION.to_string(),
        mode: E2eMode::Run,
        run_id: "pm-smoke".into(),
        started_at_ms: 1_700_000_000_000,
        finished_at_ms: 1_700_000_001_500,
        summary: E2eSummary {
            passed: 0,
            failed: 1,
            skipped: 0,
            duration_ms: 1_500,
            exit_code: 1,
        },
        cases: vec![E2eCaseEvidence {
            id: "case-fail".into(),
            title: "Checkout > submits order".into(),
            file: PathBuf::from("flows/checkout.case.ts"),
            outcome: "failed".into(),
            duration_ms: 1_500,
            steps: vec![E2eProductStep {
                id: "step-2".into(),
                title: "confirm order".into(),
                status: "failed".into(),
                duration_ms: 800,
                assertion: Some(E2eAssertionDetail {
                    message: "Expected order status 'paid', got 'pending'".into(),
                    stack: Some("at e2e/checkout.case.ts:60:5".into()),
                    diff: Some("- paid\n+ pending".into()),
                }),
                context: E2eStepContext {
                    screenshots: vec![E2eArtifactRef {
                        kind: "screenshot".into(),
                        path: PathBuf::from("shots/confirm-fail.png"),
                        label: Some("order confirm failure".into()),
                    }],
                    console: vec![E2eConsoleEntry {
                        level: "error".into(),
                        text: "submit failed: HTTP 502".into(),
                        ts_ms: 1_700_000_001_200,
                    }],
                    network: vec![E2eNetworkEntry {
                        request_id: "req-1".into(),
                        method: "POST".into(),
                        url: "https://api.example.com/orders".into(),
                        status: Some(502),
                        ts_start_ms: 1_700_000_001_100,
                        ts_end_ms: Some(1_700_000_001_180),
                    }],
                    ..Default::default()
                },
            }],
        }],
        artifacts: vec![],
        open_control: None,
    }
}

fn setup_source_root() -> tempfile::TempDir {
    let src = tempfile::tempdir().unwrap();
    let shots = src.path().join("shots");
    std::fs::create_dir_all(&shots).unwrap();
    std::fs::write(shots.join("confirm-fail.png"), b"fake-png").unwrap();
    src
}

#[test]
fn static_report_directory_is_self_contained() {
    // Stop condition (#2735): packaged fixture opens with no Jet
    // services — every required file lives on disk inside the
    // package.
    let src = setup_source_root();
    let out = tempfile::tempdir().unwrap();
    let pkg =
        package_static_report(&fixture_with_artifacts(), src.path(), out.path()).expect("package");

    assert!(pkg.index_html.exists(), "index.html must exist");
    assert!(pkg.bundle_json.exists(), "evidence bundle must exist");
    assert!(pkg.events_jsonl.exists(), "events jsonl must exist");
    assert!(pkg.artifact_dir.exists(), "artifacts dir must exist");
    assert!(pkg.copied_artifacts.iter().all(|a| a.absolute.exists()));
    assert!(
        pkg.missing_artifacts.is_empty(),
        "fixture artifacts must all resolve",
    );

    // All four required files are inside the same report_dir, so the
    // package is portable: zip it, move it, host it on file://.
    let parent = &pkg.report_dir;
    for path in [&pkg.index_html, &pkg.bundle_json, &pkg.events_jsonl] {
        assert!(
            path.starts_with(parent),
            "{path:?} must live under {parent:?}",
        );
    }
}

#[test]
fn static_report_html_embeds_no_runner_endpoints() {
    // Stop condition (#2735): no run/pause/replay controls visible —
    // and no live-runner endpoint at all, even hidden.
    let src = setup_source_root();
    let out = tempfile::tempdir().unwrap();
    let pkg =
        package_static_report(&fixture_with_artifacts(), src.path(), out.path()).expect("package");
    let html = std::fs::read_to_string(&pkg.index_html).unwrap();

    // PM mode flag is present.
    assert!(html.contains("data-mode=\"pm-report\""));
    // No WebSocket, no live API, no localhost runner ports.
    assert!(!html.contains("ws://"));
    assert!(!html.contains("/api/live-control"));
    assert!(!html.contains("http://localhost:"));
    // Control buttons (if rendered for layout parity) must carry
    // `disabled`.
    for needle in [
        "data-action=\"pause\"",
        "data-action=\"next\"",
        "data-action=\"replay\"",
    ] {
        if let Some(idx) = html.find(needle) {
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
fn static_report_renders_failure_artifacts_from_relative_paths() {
    // Stop condition (#2735): failed-step screenshot/log/network
    // artifacts render from static paths.
    let src = setup_source_root();
    let out = tempfile::tempdir().unwrap();
    let pkg =
        package_static_report(&fixture_with_artifacts(), src.path(), out.path()).expect("package");

    let bundle_text = std::fs::read_to_string(&pkg.bundle_json).unwrap();
    let parsed: E2eEvidenceBundle = serde_json::from_str(&bundle_text).unwrap();
    let case = &parsed.cases[0];
    let step = &case.steps[0];

    // Assertion message + diff + stack survive the round-trip.
    let assertion = step.assertion.as_ref().expect("assertion preserved");
    assert!(assertion.message.contains("Expected order status"));
    assert!(assertion.diff.as_deref().unwrap().contains("+ pending"));

    // Console + network entries survive — these are what the PM
    // failure-detail tabs render.
    assert_eq!(step.context.console.len(), 1);
    assert_eq!(step.context.console[0].text, "submit failed: HTTP 502");
    assert_eq!(step.context.network.len(), 1);
    assert_eq!(step.context.network[0].status, Some(502));

    // Screenshot artifact path is relative and resolves under the
    // report dir, so the browser can render it via file://.
    let shot = &step.context.screenshots[0];
    assert!(!shot.path.is_absolute());
    let resolved = pkg.report_dir.join(&shot.path);
    assert!(resolved.exists(), "screenshot must resolve from report dir");
}

#[test]
fn static_report_can_load_without_running_jet_services() {
    // Stop condition (#2735): browser opens packaged fixture and
    // shows failure context. We can't drive a real browser from a
    // unit test, but we can prove the full data path the browser
    // would take: read index.html + evidence JSON + screenshot bytes
    // off disk with no helper process.
    let src = setup_source_root();
    let out = tempfile::tempdir().unwrap();
    let pkg =
        package_static_report(&fixture_with_artifacts(), src.path(), out.path()).expect("package");

    // 1. Read index.html: contains the failure title and message.
    let html = std::fs::read_to_string(&pkg.index_html).unwrap();
    assert!(html.contains("submits order"), "case title in HTML");
    assert!(
        html.contains("Expected order status"),
        "failure message in HTML",
    );

    // 2. Read evidence JSON: parses + carries failure summary.
    let bundle_text = std::fs::read_to_string(&pkg.bundle_json).unwrap();
    let parsed: E2eEvidenceBundle = serde_json::from_str(&bundle_text).unwrap();
    assert_eq!(parsed.summary.failed, 1);

    // 3. Read events JSONL: at least one event per step.
    let events_text = std::fs::read_to_string(&pkg.events_jsonl).unwrap();
    assert!(events_text.lines().count() > 0);

    // 4. Read screenshot bytes: artifact is reachable via the
    //    relative path embedded in the JSON.
    let shot = &parsed.cases[0].steps[0].context.screenshots[0];
    let bytes = std::fs::read(pkg.report_dir.join(&shot.path)).unwrap();
    assert_eq!(bytes, b"fake-png");
}
// CODEGEN-END
