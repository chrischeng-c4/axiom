// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Integration tests for fixture DI advanced form (P3.5).
//!
//! Spec: `.aw/tech-design/projects/jet/logic/fixture-di.md`.

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
    let spec_path = tmp.path().join("fixture_di.spec.js");
    fs::write(&spec_path, spec).unwrap();
    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    cfg.reporters = vec![];
    cfg.workers = 1;
    Some(test_runner::run(cfg).await.expect("runner"))
}

// ── FD1: advanced fixture depends on flat fixture ────────────────────────────

#[tokio::test]
async fn fd1_advanced_depends_on_flat() {
    if !node_available() {
        return;
    }
    let spec = r#"
import { test } from '@jet/test';

const t = test.extend({
  // flat form
  greeting: async (use) => {
    await use('hello');
  },
  // advanced form — destructures greeting
  shout: async ({ greeting }, use) => {
    await use(greeting.toUpperCase());
  },
});

t('FD1: advanced reads flat value', async ({ shout }) => {
  if (shout !== 'HELLO') throw new Error('shout=' + shout);
});
"#;
    let s = run_spec(spec).await.unwrap();
    assert_eq!(s.passed, 1, "{:?}", s);
    assert_eq!(s.failed, 0);
}

// ── FD2: 2-deep advanced chain ──────────────────────────────────────────────

#[tokio::test]
async fn fd2_two_deep_chain() {
    if !node_available() {
        return;
    }
    let spec = r#"
import { test } from '@jet/test';

const t = test.extend({
  a: async (use) => use(1),
  b: async ({ a }, use) => use(a + 10),
  c: async ({ b }, use) => use(b + 100),
});

t('FD2: 2-deep DFS', async ({ c }) => {
  if (c !== 111) throw new Error('c=' + c);
});
"#;
    let s = run_spec(spec).await.unwrap();
    assert_eq!(s.passed, 1);
    assert_eq!(s.failed, 0);
}

// ── FD3: shared dep resolved once ────────────────────────────────────────────

#[tokio::test]
async fn fd3_shared_dep_resolved_once() {
    if !node_available() {
        return;
    }
    let spec = r#"
import { test } from '@jet/test';

// This would-be mutable is scoped via a module-level counter bound to
// the test. Under the DI resolver, `counter` must resolve exactly once
// per test, so `x` and `y` see the same increment.
let resolved = 0;

const t = test.extend({
  counter: async (use) => {
    resolved += 1;
    await use(resolved);
  },
  x: async ({ counter }, use) => use('x' + counter),
  y: async ({ counter }, use) => use('y' + counter),
});

t('FD3: shared counter resolved once', async ({ x, y }) => {
  // x and y both read the same counter value (last `resolved` bump).
  const xn = Number(x.slice(1));
  const yn = Number(y.slice(1));
  if (xn !== yn) throw new Error('counter resolved twice: x=' + x + ', y=' + y);
  if (resolved !== 1) throw new Error('resolved=' + resolved + ', expected 1');
});
"#;
    let s = run_spec(spec).await.unwrap();
    assert_eq!(s.passed, 1, "{:?}", s);
    assert_eq!(s.failed, 0);
}

// ── FD4: cycle throws ────────────────────────────────────────────────────────

#[tokio::test]
async fn fd4_cycle_detected() {
    if !node_available() {
        return;
    }
    let spec = r#"
import { test } from '@jet/test';

const t = test.extend({
  a: async ({ b }, use) => { await use(b + 1); },
  b: async ({ a }, use) => { await use(a + 1); },
});

t('FD4: cycle fails fast', async ({ a }) => {
  throw new Error('should not reach test body, saw a=' + a);
});
"#;
    let s = run_spec(spec).await.unwrap();
    assert_eq!(s.failed, 1);
    let msg = s.reports[0]
        .error
        .as_ref()
        .map(|e| e.message.clone())
        .unwrap_or_default();
    assert!(msg.contains("cycle"), "expected cycle error, got: {msg}");
}

// ── FD5: undefined dep ──────────────────────────────────────────────────────

#[tokio::test]
async fn fd5_undefined_dep() {
    if !node_available() {
        return;
    }
    let spec = r#"
import { test } from '@jet/test';

const t = test.extend({
  a: async ({ nope }, use) => { await use(nope); },
});

t('FD5: missing dep fails', async ({ a }) => {
  throw new Error('should not reach test body, saw a=' + a);
});
"#;
    let s = run_spec(spec).await.unwrap();
    assert_eq!(s.failed, 1);
    let msg = s.reports[0]
        .error
        .as_ref()
        .map(|e| e.message.clone())
        .unwrap_or_default();
    assert!(
        msg.contains("nope") && (msg.contains("not defined") || msg.contains("undefined")),
        "expected 'not defined' for 'nope', got: {msg}"
    );
}

// ── FD6: flat fixture regression guard ──────────────────────────────────────

#[tokio::test]
async fn fd6_flat_still_works() {
    if !node_available() {
        return;
    }
    let spec = r#"
import { test } from '@jet/test';

const t = test.extend({
  flat: async (use) => { await use('F'); },
});

t('FD6: flat fixture', async ({ flat }) => {
  if (flat !== 'F') throw new Error('flat=' + flat);
});
"#;
    let s = run_spec(spec).await.unwrap();
    assert_eq!(s.passed, 1);
    assert_eq!(s.failed, 0);
}
// CODEGEN-END
