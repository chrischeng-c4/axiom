// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Per-test fixture lifecycle contract for `@jet/test` (#2711).
//!
//! Atomic slice: define fixture lifetime rules and add one per-test
//! fixture example. Stop condition — per-test setup/teardown passes and
//! leaks no state.
//!
//! Non-goals (deferred): worker isolation matrix and project-scoped
//! fixtures. Those land via #2715 + later slices.
//!
//! @spec projects/jet/data/runtime/test/CONTRACT.md#fixture-lifecycle-2711

use jet::test_runner::{self, RunnerConfig};
use std::fs;

fn node_available() -> bool {
    which::which("node").is_ok()
}

async fn run_spec(spec: &str) -> Option<test_runner::Summary> {
    if !node_available() {
        return None;
    }
    let tmp = tempfile::tempdir().unwrap();
    let spec_path = tmp.path().join("fixture_lifecycle.spec.js");
    fs::write(&spec_path, spec).unwrap();
    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    cfg.reporters = vec![];
    cfg.workers = 1;
    Some(test_runner::run(cfg).await.expect("runner"))
}

// ── L1: per-test setup runs once per test, teardown runs after body ──────────

#[tokio::test]
async fn l1_per_test_setup_and_teardown_run_for_each_test() {
    if !node_available() {
        return;
    }
    // Both tests share a module-level counter so we can observe how many
    // times the fixture runs setup vs teardown across the file. Per-test
    // lifetime means: two setups, two teardowns, no cross-test reuse.
    let spec = r#"
import { test, expect } from '@jet/test';

let setups = 0;
let teardowns = 0;

const t = test.extend({
  scratch: async (use) => {
    setups += 1;
    const value = { id: setups };
    await use(value);
    teardowns += 1;
  },
});

t('L1a: first test sees setup #1', async ({ scratch }) => {
  expect(scratch.id).toBe(1);
  expect(setups).toBe(1);
  expect(teardowns).toBe(0);
});

t('L1b: second test sees a fresh setup #2', async ({ scratch }) => {
  expect(scratch.id).toBe(2);
  expect(setups).toBe(2);
  // teardown ran for L1a before L1b's body — the contract guarantees this.
  expect(teardowns).toBe(1);
});
"#;
    let s = run_spec(spec).await.unwrap();
    assert_eq!(s.passed, 2, "{:?}", s);
    assert_eq!(s.failed, 0);
}

// ── L2: state does not leak across tests ─────────────────────────────────────

#[tokio::test]
async fn l2_per_test_fixture_state_does_not_leak() {
    if !node_available() {
        return;
    }
    // The fixture returns a fresh object every test. The first test
    // mutates it; the second test must see the un-mutated default.
    let spec = r#"
import { test, expect } from '@jet/test';

const t = test.extend({
  bag: async (use) => {
    await use({ items: [] });
  },
});

t('L2a: pushes into bag', async ({ bag }) => {
  bag.items.push('a');
  bag.items.push('b');
  expect(bag.items.length).toBe(2);
});

t('L2b: bag starts empty for the next test', async ({ bag }) => {
  expect(bag.items.length).toBe(0);
});
"#;
    let s = run_spec(spec).await.unwrap();
    assert_eq!(s.passed, 2, "{:?}", s);
    assert_eq!(s.failed, 0);
}

// ── L3: teardown runs in reverse order of resolution ─────────────────────────

#[tokio::test]
async fn l3_teardown_runs_in_reverse_order() {
    if !node_available() {
        return;
    }
    // outer depends on inner — inner resolves first, so inner cleans up
    // last. Recording each phase in `events` lets us assert the order.
    let spec = r#"
import { test, expect } from '@jet/test';

const events = [];

const t = test.extend({
  inner: async (use) => {
    events.push('inner.setup');
    await use('inner');
    events.push('inner.teardown');
  },
  outer: async ({ inner }, use) => {
    events.push('outer.setup');
    await use(inner + '+outer');
    events.push('outer.teardown');
  },
});

t('L3: setup inner→outer, teardown outer→inner', async ({ outer }) => {
  expect(outer).toBe('inner+outer');
});

// A second test inspects the recorded event sequence after the first
// finished its teardown. Per-test reverse-order cleanup means:
//   inner.setup, outer.setup, outer.teardown, inner.teardown
t('L3b: event sequence is documented order', async () => {
  expect(events).toEqual([
    'inner.setup',
    'outer.setup',
    'outer.teardown',
    'inner.teardown',
  ]);
});
"#;
    let s = run_spec(spec).await.unwrap();
    assert_eq!(s.passed, 2, "{:?}", s);
    assert_eq!(s.failed, 0);
}

// ── L4: cleanup throwing surfaces as a fixture-cleanup failure ───────────────

#[tokio::test]
async fn l4_cleanup_failure_becomes_test_failure() {
    if !node_available() {
        return;
    }
    // The fixture passes through `use(...)` cleanly so the body runs to
    // completion. The post-`use` work then throws, which by contract
    // turns the otherwise-passing test into a failure with a
    // `fixture-cleanup` source label on the wire error.
    let spec = r#"
import { test } from '@jet/test';

const t = test.extend({
  flaky: async (use) => {
    await use('ok');
    throw new Error('teardown blew up');
  },
});

t('L4: body passes but cleanup throws', async ({ flaky }) => {
  if (flaky !== 'ok') throw new Error('flaky=' + flaky);
});
"#;
    let s = run_spec(spec).await.unwrap();
    assert_eq!(s.passed, 0, "cleanup failure must flip outcome: {:?}", s);
    assert_eq!(s.failed, 1);
    let report = s
        .reports
        .iter()
        .find(|r| r.name == "L4: body passes but cleanup throws")
        .expect("report present");
    let err = report
        .error
        .as_ref()
        .map(|e| e.message.as_str())
        .unwrap_or("");
    assert!(
        err.contains("teardown blew up"),
        "expected cleanup error in message, got: {err}",
    );
}
// CODEGEN-END
