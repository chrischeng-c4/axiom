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

  test("jest compatibility exposes mocks and the shared global", () => {
    const addOne = jest.fn((value) => value + 1);
    expect(addOne(2)).toBe(3);
    expect(addOne.mock.calls.length).toBe(1);
    expect(globalThis.jest).toBe(jest);
    jest.mock("example", () => ({ answer: 42 }));
    expect(jest.requireMock("example").answer).toBe(42);
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

#[tokio::test]
async fn jest_each_and_describe_each_expand_rows() {
    if which::which("node").is_err() {
        eprintln!("skipping: node not on PATH");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::write(
        tmp.path().join("src/jest-helper.ts"),
        r#"export const importedSpy = jest.fn((value: string) => value.toUpperCase());"#,
    )
    .unwrap();
    fs::write(
        tmp.path().join("jest-each.test.js"),
        r#"
import { importedSpy } from "./src/jest-helper";

it.each([
  [1, 2, 3],
  [2, 3, 5],
])("%i + %i = %i", (left, right, total) => {
  expect(left + right).toBe(total);
  expect(importedSpy("jet")).toBe("JET");
});

describe.each([["first", 1], ["second", 2]])("case %s", (label, value) => {
  test("keeps row value " + label, () => {
    expect(value).toBeGreaterThan(0);
  });
});
"#,
    )
    .unwrap();

    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    cfg.reporters = vec![];
    cfg.workers = 1;
    let summary = test_runner::run(cfg).await.expect("runner should complete");
    assert_eq!(
        summary.failed, 0,
        "Jest table compatibility must pass: {:#?}",
        summary.reports
    );
    assert_eq!(summary.passed, 4, "expected one test per table row");
}

#[tokio::test]
async fn jet_test_colon_virtual_module_resolves_for_ts_and_js_specs() {
    if which::which("node").is_err() {
        eprintln!("skipping: node not on PATH");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    fs::write(
        tmp.path().join("virtual-ts.spec.ts"),
        r#"
import { test, expect } from "jet:test";

test("TS spec can import Jet's virtual test module", () => {
  const label: string = "jet:test";
  expect(label).toBe("jet:test");
});
"#,
    )
    .unwrap();
    fs::write(
        tmp.path().join("virtual-js.spec.js"),
        r#"
import { test, expect } from "jet:test";

test("JS spec can import Jet's virtual test module", () => {
  expect([1, 2, 3]).toHaveLength(3);
});
"#,
    )
    .unwrap();

    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    cfg.reporters = vec![];
    let summary = test_runner::run(cfg).await.expect("runner should complete");
    assert_eq!(
        summary.failed, 0,
        "`jet:test` must resolve to Jet's built-in runtime without npm install; reports = {:#?}",
        summary.reports
    );
    assert_eq!(
        summary.passed, 2,
        "expected both virtual-module specs to pass"
    );
}

#[tokio::test]
async fn relative_imports_from_specs_resolve_against_original_spec_directory() {
    if which::which("node").is_err() {
        eprintln!("skipping: node not on PATH");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::write(
        tmp.path().join("src/sum.ts"),
        r#"
export function sum(a: number, b: number): number {
  return a + b;
}
"#,
    )
    .unwrap();
    fs::write(
        tmp.path().join("ext-js.test.ts"),
        r#"
import { test, expect } from "@jet/test";
import { sum } from "./src/sum.js";

test("relative .js specifier can target TS source", () => {
  expect(sum(1, 2)).toBe(3);
});
"#,
    )
    .unwrap();
    fs::write(
        tmp.path().join("ext-ts.test.ts"),
        r#"
import { test, expect } from "@jet/test";
import { sum } from "./src/sum.ts";

test("relative .ts specifier resolves from the original spec dir", () => {
  expect(sum(2, 3)).toBe(5);
});
"#,
    )
    .unwrap();
    fs::write(
        tmp.path().join("extless.test.ts"),
        r#"
import { test, expect } from "@jet/test";
import { sum } from "./src/sum";

test("extensionless relative specifier resolves TS source", () => {
  expect(sum(4, 5)).toBe(9);
});
"#,
    )
    .unwrap();

    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    cfg.reporters = vec![];
    let summary = test_runner::run(cfg).await.expect("runner should complete");
    assert_eq!(
        summary.failed, 0,
        "relative imports should load from the original spec directory; reports = {:#?}",
        summary.reports
    );
    assert_eq!(summary.passed, 3, "all relative import specs should pass");
}

#[tokio::test]
async fn test_worker_resolves_project_packages_and_disables_dev_refresh() {
    if which::which("node").is_err() {
        eprintln!("skipping: node not on PATH");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let node_modules = tmp.path().join("node_modules");
    let demo_dep = node_modules.join("demo-dep");
    let react = node_modules.join("react");
    fs::create_dir_all(&demo_dep).unwrap();
    fs::create_dir_all(&react).unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::write(
        demo_dep.join("package.json"),
        r#"{"name":"demo-dep","type":"module","exports":"./index.js"}"#,
    )
    .unwrap();
    fs::write(demo_dep.join("index.js"), "export const answer = 42;\n").unwrap();
    fs::write(
        react.join("package.json"),
        r#"{"name":"react","type":"module","exports":{"./jsx-runtime":"./jsx-runtime.js"}}"#,
    )
    .unwrap();
    fs::write(
        react.join("jsx-runtime.js"),
        r#"export const Fragment = Symbol.for("fragment");
export const jsx = (tag, props) => ({ tag, props });
export const jsxs = jsx;
"#,
    )
    .unwrap();
    fs::write(
        tmp.path().join("src/Panel.tsx"),
        r#"export function Panel() { return <div>panel</div>; }"#,
    )
    .unwrap();
    fs::write(
        tmp.path().join("packages-and-tsx.test.ts"),
        r#"
import { test, expect } from "@jet/test";
import { answer } from "demo-dep";
import { Panel } from "./src/Panel";

test("resolves workspace packages without dev-only refresh imports", () => {
  expect(answer).toBe(42);
  expect(typeof Panel).toBe("function");
});
"#,
    )
    .unwrap();

    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    cfg.reporters = vec![];
    cfg.workers = 1;
    let summary = test_runner::run(cfg).await.expect("runner should complete");
    assert_eq!(
        summary.failed, 0,
        "project packages and TSX imports must run in Node: {:#?}",
        summary.reports
    );
    assert_eq!(summary.passed, 1);
}

#[tokio::test]
async fn test_worker_strips_class_field_ts_syntax_in_imported_modules() {
    if which::which("node").is_err() {
        eprintln!("skipping: node not on PATH");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::write(
        tmp.path().join("src/model.ts"),
        r#"
interface Labelled { label?: string; }
export class Model implements Labelled {
  public label?: string;
  id!: string;
  constructor() { this.id = "model"; }
}
"#,
    )
    .unwrap();
    fs::write(
        tmp.path().join("class-fields.test.ts"),
        r#"
import { test, expect } from "@jet/test";
import { Model } from "./src/model";

test("imports TypeScript class fields without raw TS syntax", () => {
  const model = new Model();
  expect(model.id).toBe("model");
  expect(model.label).toBeUndefined();
});
"#,
    )
    .unwrap();

    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    cfg.reporters = vec![];
    cfg.workers = 1;
    let summary = test_runner::run(cfg).await.expect("runner should complete");
    assert_eq!(
        summary.failed, 0,
        "imported TypeScript class syntax must be stripped: {:#?}",
        summary.reports
    );
    assert_eq!(summary.passed, 1);
}

#[tokio::test]
async fn test_worker_rewrites_dynamic_directory_imports_to_index_modules() {
    if which::which("node").is_err() {
        eprintln!("skipping: node not on PATH");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let directory = tmp.path().join("src/dynamic");
    fs::create_dir_all(&directory).unwrap();
    fs::write(
        directory.join("index.ts"),
        "export const answer: number = 84;\n",
    )
    .unwrap();
    fs::write(
        directory.join("index.test.ts"),
        r#"
import { test, expect } from "@jet/test";

test("loads the sibling directory index dynamically", async () => {
  const { answer } = await import("./");
  expect(answer).toBe(84);
});
"#,
    )
    .unwrap();

    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    cfg.reporters = vec![];
    cfg.workers = 1;
    let summary = test_runner::run(cfg).await.expect("runner should complete");
    assert_eq!(
        summary.failed, 0,
        "dynamic directory imports must resolve their index module: {:#?}",
        summary.reports
    );
    assert_eq!(summary.passed, 1);
}

#[tokio::test]
async fn test_worker_preserves_commonjs_spec_location_and_require() {
    if which::which("node").is_err() {
        eprintln!("skipping: node not on PATH");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    fs::write(
        tmp.path().join("local.cjs"),
        "module.exports = { answer: 42 };\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("commonjs.test.js"),
        r#"
const path = require("node:path");
const local = require("./local.cjs");

test("CommonJS globals and require stay native", () => {
  expect(path.basename("/tmp/jet")).toBe("jet");
  expect(__dirname).toBeTruthy();
  expect(local.answer).toBe(42);
});
"#,
    )
    .unwrap();

    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    cfg.reporters = vec![];
    cfg.workers = 1;
    let summary = test_runner::run(cfg).await.expect("runner should complete");
    assert_eq!(
        summary.failed, 0,
        "CommonJS specs must execute at their original path: {:#?}",
        summary.reports
    );
    assert_eq!(summary.passed, 1);
}

#[tokio::test]
async fn jest_compatibility_supports_mock_timer_and_expect_extensions() {
    if which::which("node").is_err() {
        eprintln!("skipping: node not on PATH");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    fs::write(
        tmp.path().join("actual.cjs"),
        r#"
globalThis.__jetRequireActualLoads = (globalThis.__jetRequireActualLoads ?? 0) + 1;
module.exports = {
  loads: globalThis.__jetRequireActualLoads,
  token: "actual",
};
"#,
    )
    .unwrap();
    fs::write(
        tmp.path().join("jest-compat-surface.test.ts"),
        r#"
import { expect, jest, test } from "@jet/test";

expect.extend({
  toBeDivisibleBy(received, divisor) {
    const pass = typeof received === "number" && received % divisor === 0;
    return {
      pass,
      message: () => `expected ${received} to be divisible by ${divisor}`,
    };
  },
});

test("keeps Jest mock, timer, actual-module, and expect helpers meaningful", () => {
  const subject = { multiply(value) { return value * 2; } };
  const spy = jest.spyOn(subject, "multiply");
  expect(subject.multiply(3)).toBe(6);
  spy.mockReturnValue(9);
  expect(subject.multiply(3)).toBe(9);
  expect(jest.mocked(spy)).toBe(spy);
  jest.restoreAllMocks();
  expect(subject.multiply(3)).toBe(6);
  expect(jest.isMockFunction(subject.multiply)).toBeFalsy();

  jest.mock("./actual.cjs", () => ({ token: "mock" }));
  expect(jest.requireMock("./actual.cjs").token).toBe("mock");
  const firstActual = jest.requireActual("./actual.cjs");
  expect(firstActual.token).toBe("actual");
  jest.resetModules();
  const secondActual = jest.requireActual("./actual.cjs");
  expect(secondActual.loads).toBe(firstActual.loads + 1);
  expect(jest.requireMock("./actual.cjs").token).toBe("mock");

  jest.useFakeTimers();
  const fired = jest.fn();
  const interval = setInterval(() => fired("interval"), 5);
  setTimeout(() => fired("timeout"), 10);
  expect(jest.getTimerCount()).toBe(2);
  jest.advanceTimersByTime(5);
  expect(fired.mock.calls).toHaveLength(1);
  clearInterval(interval);
  jest.advanceTimersByTime(5);
  expect(fired.mock.calls).toHaveLength(2);
  setTimeout(() => fired("cleared"), 1);
  jest.clearAllTimers();
  jest.runAllTimers();
  expect(fired.mock.calls).toHaveLength(2);
  jest.useRealTimers();

  expect(12).toBeDivisibleBy(3);
  expect(12).not.toBeDivisibleBy(5);
  expect({ label: "jet native runner" }).toEqual({
    label: expect.stringContaining("native"),
  });
});
"#,
    )
    .unwrap();

    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    cfg.reporters = vec![];
    cfg.workers = 1;
    let summary = test_runner::run(cfg).await.expect("runner should complete");
    assert_eq!(
        summary.failed, 0,
        "Jest-compatible extensions must execute with real semantics: {:#?}",
        summary.reports
    );
    assert_eq!(summary.passed, 1);
}

#[tokio::test]
async fn test_worker_strips_inline_type_exports_from_transitive_barrels() {
    if which::which("node").is_err() {
        eprintln!("skipping: node not on PATH");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("values.ts"),
        r#"
export const runtimeValue = 42;
export type RuntimeValueShape = { value: number };
export type AllTypeOnly = { label: string };
"#,
    )
    .unwrap();
    fs::write(
        src.join("barrel.ts"),
        r#"
export {
  runtimeValue,
  type RuntimeValueShape,
} from "./values.ts";
export { type AllTypeOnly } from "./values.ts";
"#,
    )
    .unwrap();
    fs::write(
        tmp.path().join("inline-type-export.test.ts"),
        r#"
import { test, expect } from "@jet/test";
import { runtimeValue } from "./src/barrel.ts";

test("loads a transitive barrel with inline type exports", () => {
  expect(runtimeValue).toBe(42);
});
"#,
    )
    .unwrap();

    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    cfg.reporters = vec![];
    cfg.workers = 1;
    let summary = test_runner::run(cfg).await.expect("runner should complete");
    assert_eq!(
        summary.failed, 0,
        "transitive inline type exports must not reach Node: {:#?}",
        summary.reports
    );
    assert_eq!(summary.passed, 1);
}

#[tokio::test]
async fn test_worker_strips_complex_destructured_parameter_types() {
    if which::which("node").is_err() {
        eprintln!("skipping: node not on PATH");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("destructured-parameters.tsx"),
        r#"
type DraftEditorFormatOptions = { isNullable?: boolean };
type EditorState = { value: string; nullable: boolean };

export const formatFromString = (
  value: string,
  { isNullable = true }: DraftEditorFormatOptions = {},
): EditorState | null => ({ value, nullable: isNullable });

type PlatformConfigContextProps = { variableText: string };
type Menu = { name: string };

export const nameSeatalkBotCol = ({
  record,
  variableText,
  value,
}: Pick<PlatformConfigContextProps, 'variableText'> & {
  record: Menu;
  value: string;
}) => `${record.name}:${variableText}:${value}`;
"#,
    )
    .unwrap();
    fs::write(
        tmp.path().join("destructured-parameters.test.tsx"),
        r#"
import { test, expect } from "@jet/test";
import { formatFromString, nameSeatalkBotCol } from "./src/destructured-parameters.tsx";

test("loads complex destructured parameter annotations", () => {
  expect(formatFromString("value")).toEqual({ value: "value", nullable: true });
  expect(formatFromString("value", { isNullable: false })).toEqual({ value: "value", nullable: false });
  expect(nameSeatalkBotCol({
    record: { name: "bot" },
    variableText: "label",
    value: "value",
  })).toBe("bot:label:value");
});
"#,
    )
    .unwrap();

    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    cfg.reporters = vec![];
    cfg.workers = 1;
    let summary = test_runner::run(cfg).await.expect("runner should complete");
    assert_eq!(
        summary.failed, 0,
        "complex destructured parameter annotations must be fully stripped: {:#?}",
        summary.reports
    );
    assert_eq!(summary.passed, 1);
}

#[tokio::test]
async fn test_worker_resolves_extensionless_legacy_package_subpaths_and_indexes() {
    if which::which("node").is_err() {
        eprintln!("skipping: node not on PATH");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let node_modules = tmp.path().join("node_modules");
    let legacy_cjs = node_modules.join("legacy-cjs");
    let scoped_legacy = node_modules.join("@scope").join("legacy-utils");
    let partial_exports = node_modules.join("exports-without-subpath");
    let relative_index = tmp.path().join("src").join("relative-index");
    fs::create_dir_all(legacy_cjs.join("nested")).unwrap();
    fs::create_dir_all(scoped_legacy.join("dist")).unwrap();
    fs::create_dir_all(&partial_exports).unwrap();
    fs::create_dir_all(&relative_index).unwrap();

    fs::write(
        legacy_cjs.join("package.json"),
        r#"{"name":"legacy-cjs","main":"./index.js"}"#,
    )
    .unwrap();
    fs::write(legacy_cjs.join("index.js"), "module.exports = {};\n").unwrap();
    fs::write(
        legacy_cjs.join("camelCase.js"),
        "module.exports = value => String(value).replace(/-([a-z])/g, (_, ch) => ch.toUpperCase());\n",
    )
    .unwrap();
    fs::write(
        legacy_cjs.join("nested/index.js"),
        "module.exports = \"nested-index\";\n",
    )
    .unwrap();

    fs::write(
        scoped_legacy.join("package.json"),
        r#"{"name":"@scope/legacy-utils","main":"./index.js"}"#,
    )
    .unwrap();
    fs::write(
        scoped_legacy.join("dist/label.js"),
        "module.exports = \"scoped-deep-import\";\n",
    )
    .unwrap();

    fs::write(
        partial_exports.join("package.json"),
        r#"{"name":"exports-without-subpath","type":"module","exports":{".":"./index.js"}}"#,
    )
    .unwrap();
    fs::write(
        partial_exports.join("index.js"),
        "export const root = true;\n",
    )
    .unwrap();
    fs::write(
        partial_exports.join("legacy.js"),
        "export const legacy = \"unexported-subpath\";\n",
    )
    .unwrap();
    fs::write(
        relative_index.join("index.ts"),
        "export default \"relative-index\";\n",
    )
    .unwrap();

    fs::write(
        tmp.path().join("legacy-subpaths.test.ts"),
        r#"
import { test, expect } from "@jet/test";
import camelCase from "legacy-cjs/camelCase";
import nestedIndex from "legacy-cjs/nested";
import scopedLabel from "@scope/legacy-utils/dist/label";
import { legacy } from "exports-without-subpath/legacy";
import relativeIndex from "./src/relative-index";
import path from "path";

test("resolves legacy files and directory indexes without extensions", () => {
  expect(camelCase("jet-test")).toBe("jetTest");
  expect(nestedIndex).toBe("nested-index");
  expect(scopedLabel).toBe("scoped-deep-import");
  expect(legacy).toBe("unexported-subpath");
  expect(relativeIndex).toBe("relative-index");
  expect(path.basename("/tmp/jet")).toBe("jet");
});
"#,
    )
    .unwrap();

    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    cfg.reporters = vec![];
    cfg.workers = 1;
    let summary = test_runner::run(cfg).await.expect("runner should complete");
    assert_eq!(
        summary.failed, 0,
        "legacy bare package subpaths and index directories must resolve: {:#?}",
        summary.reports
    );
    assert_eq!(summary.passed, 1);
}

#[tokio::test]
async fn test_worker_stubs_static_assets_from_source_and_package_barrels() {
    if which::which("node").is_err() {
        eprintln!("skipping: node not on PATH");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let source_assets = tmp.path().join("src");
    let package_assets = tmp.path().join("node_modules").join("assets-pkg");
    fs::create_dir_all(&source_assets).unwrap();
    fs::create_dir_all(package_assets.join("images")).unwrap();
    fs::write(
        package_assets.join("package.json"),
        r#"{"name":"assets-pkg","type":"module","main":"./index.js"}"#,
    )
    .unwrap();
    fs::write(
        package_assets.join("index.js"),
        "export { default as logo } from \"./images/logo.svg\";\nexport { default as avatar } from \"./images/avatar.jpeg\";\n",
    )
    .unwrap();
    fs::write(
        package_assets.join("images/logo.svg"),
        r#"<svg viewBox="0 0 1 1"><path d="M0 0h1v1H0z"/></svg>"#,
    )
    .unwrap();
    fs::write(package_assets.join("images/avatar.jpeg"), b"jpeg-bytes").unwrap();
    fs::write(
        source_assets.join("direct.svg"),
        r#"<svg viewBox="0 0 1 1"><path d="M0 0h1v1H0z"/></svg>"#,
    )
    .unwrap();

    fs::write(
        tmp.path().join("static-assets.test.ts"),
        r#"
import { test, expect } from "@jet/test";
import { logo, avatar } from "assets-pkg";
import directLogo from "./src/direct.svg";
import packageLogo from "assets-pkg/images/logo.svg";

test("loads raw static assets as deterministic URL strings", () => {
  expect(typeof logo).toBe("string");
  expect(typeof avatar).toBe("string");
  expect(typeof directLogo).toBe("string");
  expect(typeof packageLogo).toBe("string");
  expect(logo.endsWith("logo.svg")).toBe(true);
  expect(avatar.endsWith("avatar.jpeg")).toBe(true);
  expect(directLogo.endsWith("direct.svg")).toBe(true);
  expect(packageLogo.endsWith("logo.svg")).toBe(true);
});
"#,
    )
    .unwrap();

    let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
    cfg.reporters = vec![];
    cfg.workers = 1;
    let summary = test_runner::run(cfg).await.expect("runner should complete");
    assert_eq!(
        summary.failed, 0,
        "source assets and installed-package asset barrels must be test-safe: {:#?}",
        summary.reports
    );
    assert_eq!(summary.passed, 1);
}
// CODEGEN-END
