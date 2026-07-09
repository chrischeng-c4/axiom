// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Regression tests for jet issue #1534: hanging-fixture timeout.
//!
//! Prior to the fix, the per-test timeout (`opts.timeoutMs`) only wrapped
//! `t.body(fixtureArg)`. Any custom or built-in fixture that never resolved
//! (e.g. the `page` fixture failing to launch a browser) would hang the
//! worker indefinitely and `--timeout` had no effect. The fix wraps the
//! entire fixture-setup + body pipeline in a single `withTimeout` race.
//!
//! These tests construct a deliberately hanging fixture via the public
//! `test.extend` API and assert the runner reports `TimedOut` quickly
//! rather than hanging past the harness deadline.

use jet::test_runner::{self, Outcome, RunnerConfig};
use std::fs;
use std::time::{Duration, Instant};

fn node_available() -> bool {
    which::which("node").is_ok()
}

#[tokio::test]
async fn hanging_fixture_terminates_with_timeout() {
    if !node_available() {
        eprintln!("skipping: node not on PATH");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let spec = tmp.path().join("hanging_fixture.spec.js");
    // The `slow` fixture never calls `use(...)`, so prior to the fix, the
    // resolver awaits the `donePromise` forever and the test never reaches
    // the body-level `withTimeout`. After the fix, the entire pipeline
    // races against `opts.timeoutMs`.
    fs::write(
        &spec,
        r#"
import { test } from '@jet/test';

const t = test.extend({
  slow: async (use) => {
    await new Promise(() => {}); // never resolves
    await use('unreachable');
  },
});

t('hangs in fixture setup', async ({ slow }) => {
  // Body must never be reached — fixture hangs first.
  throw new Error('body should not run, got slow=' + slow);
});
"#,
    )
    .unwrap();

    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    cfg.reporters = vec![];
    cfg.workers = 1;
    cfg.timeout_ms = 1_500; // short, so the test terminates fast

    let started = Instant::now();
    let summary = test_runner::run(cfg)
        .await
        .expect("runner should complete instead of hanging");
    let elapsed = started.elapsed();

    // The deadline + small overhead must not exceed a generous wall clock
    // budget. Without the fix this run hangs until the harness kills it
    // (typically minutes).
    assert!(
        elapsed < Duration::from_secs(30),
        "runner did not honour timeout (elapsed={elapsed:?})"
    );

    assert_eq!(summary.passed, 0, "no test should pass: {:?}", summary);
    assert_eq!(
        summary.failed, 1,
        "hanging test must count as failed (timed out)"
    );

    let timed_out = summary
        .reports
        .iter()
        .find(|r| r.name == "hangs in fixture setup")
        .expect("report present");
    assert_eq!(
        timed_out.outcome,
        Outcome::TimedOut,
        "fixture hang must surface as TimedOut, got {:?}",
        timed_out.outcome
    );

    let err_message = timed_out
        .error
        .as_ref()
        .map(|e| e.message.as_str())
        .unwrap_or("");
    assert!(
        err_message.to_lowercase().contains("timed out"),
        "error message should mention timeout, got: {err_message}"
    );
}

#[tokio::test]
async fn fixture_finishing_within_timeout_still_passes() {
    // Sanity guard: the timeout-around-fixture change must not break the
    // common case where the fixture resolves promptly and the body runs.
    if !node_available() {
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let spec = tmp.path().join("fast_fixture.spec.js");
    fs::write(
        &spec,
        r#"
import { test } from '@jet/test';

const t = test.extend({
  greeting: async (use) => {
    await use('hello');
  },
});

t('fast fixture is fine', async ({ greeting }) => {
  if (greeting !== 'hello') throw new Error('greeting=' + greeting);
});
"#,
    )
    .unwrap();

    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    cfg.reporters = vec![];
    cfg.workers = 1;
    cfg.timeout_ms = 5_000;

    let summary = test_runner::run(cfg).await.expect("runner");
    assert_eq!(summary.passed, 1, "{:?}", summary);
    assert_eq!(summary.failed, 0);
}
// CODEGEN-END
