// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Regression harness for the `@jet/test` API compatibility corpus
//! (#2715).
//!
//! The corpus lives under `tests/fixtures/jet-test-api-compat/`. Each
//! fixture project pairs a `*.spec.js` file (which exercises a thin
//! slice of the virtual-module contract) with any baseline artifacts
//! it depends on (text snapshots, golden files, …). This harness
//! copies a fixture into a temp dir, runs `jet test` against it, and
//! asserts the recorded result envelope matches the expected
//! pass/fail/skipped counts. A regression in `@jet/test` shows up as
//! a deterministic failure pointing at the fixture that broke.
//!
//! Atomic-slice scope (#2715): guard one fixture covering hooks,
//! `expect()` matchers, and a text snapshot. Broader corpus migration
//! is deferred.
//!
//! @spec #2715

use jet::test_runner::{self, RunnerConfig};
use std::fs;
use std::path::{Path, PathBuf};

fn node_available() -> bool {
    which::which("node").is_ok()
}

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/jet-test-api-compat")
}

fn copy_dir_recursive(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir_recursive(&from, &to);
        } else {
            fs::copy(&from, &to).unwrap();
        }
    }
}

async fn run_fixture() -> test_runner::Summary {
    let tmp = tempfile::tempdir().unwrap();
    copy_dir_recursive(&fixture_root(), tmp.path());

    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    cfg.reporters = vec![];
    cfg.workers = 1;
    test_runner::run(cfg).await.expect("runner")
}

#[tokio::test]
async fn api_compat_corpus_passes_under_jet_test() {
    if !node_available() {
        return;
    }
    let summary = run_fixture().await;
    assert_eq!(
        summary.failed, 0,
        "API compat fixture regressed — failures must surface here \
         before any contract-affecting change ships: {:?}",
        summary.reports,
    );
    assert_eq!(
        summary.passed, 3,
        "expected the corpus to advertise 3 passing tests, got: {:?}",
        summary,
    );
}

#[tokio::test]
async fn api_compat_corpus_reports_name_each_guarded_behavior() {
    if !node_available() {
        return;
    }
    let summary = run_fixture().await;
    let names: Vec<&str> = summary.reports.iter().map(|r| r.name.as_str()).collect();
    // Each name encodes the behavior the fixture guards. A regression
    // dropping a behavior (e.g. snapshot matcher) shows up as a missing
    // name here, so the harness keeps pointing at the specific failure.
    assert!(
        names.contains(&"hooks run before the body and toBe matches"),
        "{names:?}",
    );
    assert!(
        names.contains(&"per-test seed is reset between tests"),
        "{names:?}",
    );
    assert!(
        names.contains(&"text snapshot matches the baseline byte-for-byte"),
        "{names:?}",
    );
}
// CODEGEN-END
