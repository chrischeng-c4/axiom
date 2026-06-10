// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Dogfood: run `jet test` over the in-tree
//! `projects/jet/examples/jet-test-dogfood/src` corpus and assert the
//! result shape matches the agent-readable contract.
//!
//! Per #2607 the corpus stays in the `node` test environment (no DOM,
//! no fetch, no fixtures beyond beforeEach). The failure fixture is
//! intentional — we verify the JSON reporter emits a `Failed` outcome
//! with a non-empty `error.message` so agents have actionable data.

use jet::test_runner::{self, Outcome, RunnerConfig};
use std::path::PathBuf;

fn dogfood_src_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("jet-test-dogfood")
        .join("src")
}

#[tokio::test]
async fn unit_and_integration_dogfood_specs_pass() {
    if which::which("node").is_err() {
        eprintln!("skipping: node not on PATH");
        return;
    }

    let src = dogfood_src_dir();
    assert!(src.exists(), "dogfood corpus missing at {}", src.display());

    let mut cfg = RunnerConfig::default_for_root(&src).expect("config");
    cfg.reporters = vec![];
    // Only run the two pass-only specs here; the failure fixture has its
    // own test below.
    cfg.only_files = vec![
        src.join("unit.spec.ts"),
        src.join("frontend-integration.spec.ts"),
    ];

    let summary = test_runner::run(cfg).await.expect("runner completes");

    assert_eq!(
        summary.failed, 0,
        "dogfood pass-only specs must not fail: {:#?}",
        summary.reports
    );
    assert!(
        summary.passed >= 6,
        "expected at least 6 passing dogfood tests, got {} ({:?})",
        summary.passed,
        summary
            .reports
            .iter()
            .map(|r| r.name.as_str())
            .collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn failure_fixture_produces_structured_result_data() {
    if which::which("node").is_err() {
        eprintln!("skipping: node not on PATH");
        return;
    }

    let src = dogfood_src_dir();
    let fixture = src.join("failure-fixture.spec.ts");
    assert!(fixture.exists(), "failure fixture missing");

    let mut cfg = RunnerConfig::default_for_root(&src).expect("config");
    cfg.reporters = vec![];
    cfg.only_files = vec![fixture];

    let summary = test_runner::run(cfg).await.expect("runner completes");

    assert_eq!(summary.failed, 1, "failure fixture must report 1 failure");
    let failed = summary
        .reports
        .iter()
        .find(|r| r.outcome == Outcome::Failed)
        .expect("at least one failed report");

    let err = failed.error.as_ref().expect("error payload present");
    assert!(
        !err.message.trim().is_empty(),
        "agent-readable contract: error.message must be non-empty (got `{}`)",
        err.message
    );

    // Round-trip through serde_json to lock the JSON-reporter contract.
    let json = serde_json::to_value(&summary).expect("summary serializes");
    assert_eq!(
        json["schema_version"].as_str(),
        Some(jet::test_runner::reporter::SCHEMA_VERSION),
        "schema_version tag must round-trip: {json}"
    );
    let reports = json["reports"].as_array().expect("reports array");
    let failed_json = reports
        .iter()
        .find(|r| r["outcome"] == "failed")
        .expect("failed entry in JSON (snake_case)");
    assert!(
        failed_json["error"]["message"].is_string(),
        "JSON failure entry must carry error.message: {failed_json}"
    );
}
// CODEGEN-END
