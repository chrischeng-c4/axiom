// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! `expect(value).toMatchTextSnapshot(name?)` — text/object snapshot
//! matcher contract (#2713).
//!
//! Atomic slice: prove mismatch output and update policy for one stable
//! string/object fixture. Non-goals: golden artifacts, visual diffing.
//!
//! @spec #2713

use jet::test_runner::{self, RunnerConfig};
use std::fs;

fn node_available() -> bool {
    which::which("node").is_ok()
}

async fn run_spec_with(
    spec: &str,
    update_snapshots: bool,
    seed_baseline: Option<(&str, &str)>,
) -> Option<(test_runner::Summary, tempfile::TempDir)> {
    if !node_available() {
        return None;
    }
    let tmp = tempfile::tempdir().unwrap();
    let spec_path = tmp.path().join("text_snapshot.spec.js");
    fs::write(&spec_path, spec).unwrap();

    if let Some((name, content)) = seed_baseline {
        // Mirror the Rust-side path layout: __snapshots__/<spec-slug>/<name>.txt.
        // spec_slug for "text_snapshot.spec.js" stem "text_snapshot.spec".
        let snap_dir = tmp.path().join("__snapshots__").join("text-snapshot-spec");
        fs::create_dir_all(&snap_dir).unwrap();
        fs::write(snap_dir.join(format!("{name}.txt")), format!("{content}\n")).unwrap();
    }

    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    cfg.reporters = vec![];
    cfg.workers = 1;
    cfg.update_snapshots = update_snapshots;
    let summary = test_runner::run(cfg).await.expect("runner");
    Some((summary, tmp))
}

// ── T1: first-run writes baseline + passes ───────────────────────────────────

#[tokio::test]
async fn t1_first_run_writes_baseline_and_passes() {
    if !node_available() {
        return;
    }
    let spec = r#"
import { test, expect } from '@jet/test';

test('T1: writes new text snapshot baseline', async () => {
  await expect('hello world').toMatchTextSnapshot('greeting');
});
"#;
    let (summary, tmp) = run_spec_with(spec, false, None).await.unwrap();
    assert_eq!(summary.passed, 1, "{:?}", summary);
    assert_eq!(summary.failed, 0);

    let baseline = tmp
        .path()
        .join("__snapshots__")
        .join("text-snapshot-spec")
        .join("greeting.txt");
    assert!(baseline.exists(), "baseline must be written");
    let content = fs::read_to_string(&baseline).unwrap();
    assert_eq!(content, "hello world\n");
}

// ── T2: matching baseline passes silently ────────────────────────────────────

#[tokio::test]
async fn t2_matching_baseline_passes() {
    if !node_available() {
        return;
    }
    let spec = r#"
import { test, expect } from '@jet/test';

test('T2: matches stored baseline', async () => {
  await expect('stable').toMatchTextSnapshot('greeting');
});
"#;
    let (summary, _tmp) = run_spec_with(spec, false, Some(("greeting", "stable")))
        .await
        .unwrap();
    assert_eq!(summary.passed, 1, "{:?}", summary);
    assert_eq!(summary.failed, 0);
}

// ── T3: mismatch fails with diff and keeps baseline untouched ────────────────

#[tokio::test]
async fn t3_mismatch_fails_with_diff_and_preserves_baseline() {
    if !node_available() {
        return;
    }
    let spec = r#"
import { test, expect } from '@jet/test';

test('T3: mismatch is a failure', async () => {
  await expect('drift').toMatchTextSnapshot('greeting');
});
"#;
    let (summary, tmp) = run_spec_with(spec, false, Some(("greeting", "stable")))
        .await
        .unwrap();
    assert_eq!(summary.passed, 0, "{:?}", summary);
    assert_eq!(summary.failed, 1);

    let report = summary
        .reports
        .iter()
        .find(|r| r.name == "T3: mismatch is a failure")
        .expect("report present");
    let err = report.error.as_ref().expect("error present");
    assert!(
        err.message.contains("text snapshot mismatch"),
        "expected mismatch error, got: {}",
        err.message,
    );
    // Diff payload carries both sides.
    let diff = err.diff.as_deref().unwrap_or("");
    assert!(diff.contains("- stable"), "diff missing expected: {diff}");
    assert!(diff.contains("+ drift"), "diff missing actual: {diff}");

    // Baseline must not be silently overwritten on mismatch.
    let baseline = tmp
        .path()
        .join("__snapshots__")
        .join("text-snapshot-spec")
        .join("greeting.txt");
    assert_eq!(fs::read_to_string(&baseline).unwrap(), "stable\n");
}

// ── T4: --update-snapshots rewrites the baseline and passes ──────────────────

#[tokio::test]
async fn t4_update_snapshots_overwrites_baseline_and_passes() {
    if !node_available() {
        return;
    }
    let spec = r#"
import { test, expect } from '@jet/test';

test('T4: update rewrites baseline', async () => {
  await expect('refreshed').toMatchTextSnapshot('greeting');
});
"#;
    let (summary, tmp) = run_spec_with(spec, true, Some(("greeting", "stale")))
        .await
        .unwrap();
    assert_eq!(summary.passed, 1, "{:?}", summary);
    assert_eq!(summary.failed, 0);

    let baseline = tmp
        .path()
        .join("__snapshots__")
        .join("text-snapshot-spec")
        .join("greeting.txt");
    assert_eq!(fs::read_to_string(&baseline).unwrap(), "refreshed\n");
}

// ── T5: objects serialize with stable key order ──────────────────────────────

#[tokio::test]
async fn t5_object_snapshot_uses_stable_key_order() {
    if !node_available() {
        return;
    }
    // Same logical object, different literal key order. Both serialize
    // to the same baseline thanks to stable-key JSON.stringify.
    let spec = r#"
import { test, expect } from '@jet/test';

test('T5a: writes baseline from one key order', async () => {
  await expect({ name: 'jet', kind: 'runner' }).toMatchTextSnapshot('shape');
});

test('T5b: matches the same baseline from a different key order', async () => {
  await expect({ kind: 'runner', name: 'jet' }).toMatchTextSnapshot('shape');
});
"#;
    let (summary, tmp) = run_spec_with(spec, false, None).await.unwrap();
    assert_eq!(summary.passed, 2, "{:?}", summary);
    assert_eq!(summary.failed, 0);

    let baseline = tmp
        .path()
        .join("__snapshots__")
        .join("text-snapshot-spec")
        .join("shape.txt");
    let content = fs::read_to_string(&baseline).unwrap();
    // Keys sorted alphabetically: kind then name.
    assert!(content.contains("\"kind\": \"runner\""), "{content}");
    assert!(content.contains("\"name\": \"jet\""), "{content}");
    // Stable order means kind comes first.
    let kind_idx = content.find("\"kind\"").unwrap();
    let name_idx = content.find("\"name\"").unwrap();
    assert!(kind_idx < name_idx, "key order not stable:\n{content}");
}
// CODEGEN-END
