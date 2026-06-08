// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! End-to-end smoke test for the native `jet test` runner.
//!
//! Builds a tempdir containing a `.spec.ts` file, runs the runner, and
//! asserts the summary reflects the expected pass/fail/skip counts.

use jet::test_runner::{self, Outcome, RunnerConfig};
use std::fs;

#[tokio::test]
async fn runs_basic_spec_and_reports_pass_fail_skip() {
    // Skip silently if node isn't on PATH — this test needs a real node runtime.
    if which::which("node").is_err() {
        eprintln!("skipping: node not on PATH");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let spec = tmp.path().join("basic.spec.ts");
    fs::write(
        &spec,
        r#"
describe("math", () => {
  test("adds", () => {
    expect(1 + 1).toBe(2);
  });
  test("fails on purpose", () => {
    expect(1).toBe(2);
  });
  test.skip("skipped", () => {
    expect(true).toBe(false);
  });
});

test("contains", () => {
  expect("hello world").toContain("world");
});
"#,
    )
    .unwrap();

    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    // Silence json reporter for a cleaner test.
    cfg.reporters = vec![];

    let summary = test_runner::run(cfg).await.expect("runner should complete");

    assert_eq!(summary.passed, 2, "expected 2 passing tests");
    assert_eq!(summary.failed, 1, "expected 1 failing test");
    assert_eq!(summary.skipped, 1, "expected 1 skipped test");

    let has_adds = summary
        .reports
        .iter()
        .any(|r| r.name == "adds" && r.outcome == Outcome::Passed);
    assert!(has_adds, "expected `adds` to pass");

    let has_fail = summary
        .reports
        .iter()
        .any(|r| r.name == "fails on purpose" && r.outcome == Outcome::Failed);
    assert!(has_fail, "expected `fails on purpose` to fail");
}

// @spec #2605 — @jet/test unit-test surface: hooks + fixtures + new matchers
#[tokio::test]
async fn unit_test_surface_hooks_fixtures_and_matchers() {
    if which::which("node").is_err() {
        eprintln!("skipping: node not on PATH");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let spec = tmp.path().join("unit.spec.ts");
    fs::write(
        &spec,
        r#"
let state = { hits: 0, things: [] as number[] };

beforeEach(() => { state.hits += 1; state.things = [1, 2, 3]; });
afterEach(() => { state.things = []; });

describe("comparators", () => {
  test("toBeGreaterThan + toBeLessThan", () => {
    expect(state.hits).toBeGreaterThan(0);
    expect(state.hits).toBeLessThan(1000);
  });
  test("toBeCloseTo", () => {
    expect(0.1 + 0.2).toBeCloseTo(0.3, 5);
  });
});

describe("nullish", () => {
  test("toBeNull / toBeUndefined / toBeDefined / toBeNaN", () => {
    expect(null).toBeNull();
    expect(undefined).toBeUndefined();
    expect(0).toBeDefined();
    expect(NaN).toBeNaN();
  });
});

describe("collections", () => {
  test("toHaveLength", () => {
    expect(state.things).toHaveLength(3);
    expect("abc").toHaveLength(3);
  });
  test("toHaveProperty", () => {
    expect({ a: { b: 1 } }).toHaveProperty("a.b", 1);
    expect({ x: 1 }).toHaveProperty("x");
  });
});

describe("throw", () => {
  test("toThrow with regex + class", () => {
    expect(() => { throw new Error("boom!"); }).toThrow(/boom/);
    expect(() => { throw new TypeError("nope"); }).toThrow(TypeError);
  });
});

describe("not chain", () => {
  test("negates pass into fail and back", () => {
    expect(1).not.toBe(2);
    expect("abc").not.toContain("z");
    expect([]).not.toHaveLength(1);
  });
});

const useFixture = test.extend({
  cart: async ({}, use) => {
    await use({ items: ["apple"] });
  },
});
useFixture("custom fixture is injected by name", ({ cart }) => {
  expect(cart.items).toHaveLength(1);
  expect(cart.items[0]).toBe("apple");
});
"#,
    )
    .unwrap();

    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    cfg.reporters = vec![];
    let summary = test_runner::run(cfg).await.expect("runner should complete");
    assert_eq!(
        summary.failed, 0,
        "all matcher/hook/fixture cases must pass; reports = {:#?}",
        summary.reports
    );
    assert!(
        summary.passed >= 8,
        "expected at least 8 passing tests, got {} ({:?})",
        summary.passed,
        summary
            .reports
            .iter()
            .map(|r| r.name.as_str())
            .collect::<Vec<_>>()
    );
}

// @spec #2608 — @jet/test virtual module contract: introspection + tripwires
#[tokio::test]
async fn jet_test_contract_introspection_and_tripwires() {
    if which::which("node").is_err() {
        eprintln!("skipping: node not on PATH");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let spec = tmp.path().join("contract.spec.ts");
    fs::write(
        &spec,
        r#"
import { describe, test, expect, __JET_TEST_CONTRACT, vi, jest, mock, fail } from "@jet/test";

describe("@jet/test contract", () => {
  test("supported names are present in __JET_TEST_CONTRACT", () => {
    for (const name of ["describe", "test", "expect", "beforeEach", "afterEach", "beforeAll", "afterAll", "Page", "browser"]) {
      expect(__JET_TEST_CONTRACT).toContain(name);
    }
  });

  test("vi tripwire throws a jet-owned diagnostic", () => {
    expect(() => vi.fn()).toThrow(/@jet\/test:\s*`vi`\s+is not part of the @jet\/test contract/);
  });

  test("jest tripwire throws a jet-owned diagnostic", () => {
    expect(() => jest.fn()).toThrow(/@jet\/test:\s*`jest`/);
  });

  test("mock tripwire throws on call", () => {
    expect(() => mock()).toThrow(/@jet\/test:\s*`mock`/);
  });

  test("fail tripwire throws on call", () => {
    expect(() => fail("nope")).toThrow(/@jet\/test:\s*`fail`/);
  });
});
"#,
    )
    .unwrap();

    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    cfg.reporters = vec![];
    let summary = test_runner::run(cfg).await.expect("runner should complete");
    assert_eq!(
        summary.failed, 0,
        "contract introspection + tripwire cases must pass; reports = {:#?}",
        summary.reports
    );
    assert!(
        summary.passed >= 5,
        "expected 5 passing contract tests, got {}",
        summary.passed
    );
}
// CODEGEN-END
