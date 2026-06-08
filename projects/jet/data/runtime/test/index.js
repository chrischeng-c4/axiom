// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-data-runtime-test.md#logic
// CODEGEN-BEGIN
// jet test worker runtime.
//
// Exposes `describe`, `test`, `expect`, and the before/after hooks as globals
// for the imported spec module, runs the collected plan, and streams NDJSON
// events back to the Rust runner via stdout.
//
// Wire format: see projects/jet/src/test_runner/wire.rs.
//
// Phase 3 additions:
// - DOM-integrated matchers (`toHaveText`, `toBeVisible`, `toMatchSnapshot`)
//   send `WireRequest` NDJSON over stdout and await `WireResponse` from stdin
//   with `req_id` correlation. Retry with 100ms polling until opts.timeout
//   (default 5000ms).
// - `test.extend(fixtures)` for flat (non-DI-graph) custom fixtures.
// @spec .aw/changes/enhancement-phase-3-dom-matchers-fixtures-for-native-test-runn/specs/enhancement-phase-3-dom-matchers-fixtures-for-native-test-runn-spec.md#R1
// @spec ...#R2
// @spec ...#R3
// @spec ...#R6
//
// Phase 5 (page-fixture auto-injection):
// - Default fixture registry pre-registers `page` as a built-in fixture backed
//   by the CDP driver via the PageRequest/PageResponse wire channel.
// - Destructure-detection: parse callback parameter names via fn.toString() to
//   detect `{page}` in test() and test.beforeEach() callbacks (no test.extend
//   call needed).
// - baseURL resolution: page.goto(relativePath) prepends opts.jetConfig.baseURL.
// @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R1
// @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R2

function makeSuite(name, parent) {
  return {
    name,
    parent,
    children: [], // nested suites
    tests: [], // { name, body, skip, only, fixtures }
    before_all: [],
    after_all: [],
    before_each: [],
    after_each: [],
  };
}

const __jet = {
  root: makeSuite("", null),
  stack: [],
  hasOnly: false,
  reqId: 0,
  pending: new Map(), // req_id -> { resolve, reject }
  currentTestTitle: null, // for toMatchSnapshot default name
  // P3.4: Active pages registered by the page fixture or browser.newContext.
  // On test failure, the runner snaps a PNG of every entry in this set so
  // the developer doesn't have to reproduce the failure to see UI state.
  // @spec .aw/tech-design/projects/jet/logic/auto-artifacts.md#A4
  activePages: new Set(),
};
__jet.stack.push(__jet.root);

// ── Page-fixture wire protocol ─────────────────────────────────────────────
// PageRequest messages flow over stdout; PageResponse messages come back over
// stdin alongside WireResponse messages. They are distinguished by `kind` tag:
// PageRequest kinds are listed in cdp_driver::page_binding::PageRequest.
// The __jet.pending map (keyed by req_id) is shared between all wire message
// types so one __sendRequest implementation serves both.

import { Page, Locator } from "./page.js";
// @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R20
import {
  toHaveTitle,
  toHaveURL,
  toBeVisibleLocator,
  toBeHidden,
  toHaveTextLocator,
  toHaveValue,
  toHaveCount,
  toHaveClass,
  toHaveAttribute,
  toBeChecked,
  toBeDisabled,
  toBeEnabled,
  toBeFocused,
  toHaveCSS,
  toHaveAccessibleName,
  toHaveRole,
  matchObject,
} from "./matchers.js";

// ── Default fixture registry ───────────────────────────────────────────────
// Pre-registers `page` as a built-in fixture. User test.extend({ page: ... })
// overrides it for tests using that extended test object. Tests that do not
// destructure `page` skip the fixture entirely (no browser launch for those).
//
// @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R1
// @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R5
const __DEFAULT_FIXTURES = {
  page: async (use, opts) => {
    // Called only when the test body destructures `page`. `opts` carries
    // jetConfig (baseURL, headless) forwarded from the worker boot script.
    const baseURL = (opts && opts.jetConfig && opts.jetConfig.baseURL) || "";
    let pg;
    try {
      // Create a new page via the PageRequest wire channel.
      pg = await __createPage(baseURL);
    } catch (err) {
      throw new Error(`browser: failed to create page — ${err?.message ?? err}`);
    }
    try {
      await use(pg);
    } finally {
      // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R4
      try {
        await pg.close();
      } catch {
        // Suppress — page may already be gone if the test crashed.
      }
    }
  },
};

// ── Destructure-detection helper ───────────────────────────────────────────
// Parse the parameter list of a callback via fn.toString() and return the set
// of names destructured from the first argument. Handles both destructure
// syntax `async ({ page }) =>` and named object `async (fixtures) =>`.
//
// @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R1
function __detectFixtureNames(fn) {
  if (typeof fn !== "function") return new Set();
  try {
    const src = fn.toString();
    // Match the first parameter of the function: (async)? (function)? name? (...)
    // We look for a destructured object pattern: `({ a, b, c })` or `{ a, b }`.
    const paramMatch = src.match(
      /^(?:async\s+)?(?:function\s*\w*\s*)?\(?\s*(\{[^)]*\})/
    );
    if (!paramMatch) return new Set();
    const destructured = paramMatch[1];
    // Extract identifiers from inside the braces.
    const names = new Set();
    for (const m of destructured.matchAll(/\b([a-zA-Z_$][a-zA-Z0-9_$]*)\b/g)) {
      names.add(m[1]);
    }
    return names;
  } catch {
    return new Set();
  }
}

// Classify a fixture function's shape. Returns one of:
//   { form: "flat" }                         — (use, opts) signature
//   { form: "advanced", deps: Set<string> }  — ({...}, use, opts) signature
//   { form: "static" }                       — non-function value
//
// Advanced form is detected when the first param is a destructured object.
// The fixture itself chooses the shape; both coexist in the same registry.
// @spec .aw/tech-design/projects/jet/logic/fixture-di.md#F2
function __fixtureShape(fn) {
  if (typeof fn !== "function") return { form: "static" };
  try {
    const src = fn.toString();
    // Strip leading keywords and optional function name.
    const head = src
      .replace(/^async\s+/, "")
      .replace(/^function\s*\w*\s*/, "")
      .trimStart();
    // Advanced form starts with "(" then "{", OR directly with "{" (arrow
    // without parens around a single destructure, though JS requires parens
    // for destructured params — include both for safety).
    const advMatch = head.match(/^\(\s*(\{[^)]*\})/);
    if (!advMatch) {
      return { form: "flat" };
    }
    const deps = new Set();
    for (const m of advMatch[1].matchAll(/\b([a-zA-Z_$][a-zA-Z0-9_$]*)\b/g)) {
      deps.add(m[1]);
    }
    return { form: "advanced", deps };
  } catch {
    return { form: "flat" };
  }
}

// ── Page creation via wire channel ─────────────────────────────────────────
// Sends a `new_page` PageRequest (kind: "new_page") to the Rust worker which
// launches browser.new_page() and returns a page_id (CDP target ID). The JS
// Page instance wraps that ID and uses __sendRequest for all further actions.
//
// @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R5
async function __createPage(baseURL) {
  // Send a new_page request over the wire channel.
  const res = await __sendRequest({ kind: "new_page" });
  if (res.kind === "error") {
    throw new Error(`browser: ${res.message}`);
  }
  const pageId = res.page_id;
  // Wrap in a Page proxy that routes all method calls via __sendPageRequest.
  const pg = new Page(pageId, __sendPageRequest, baseURL);
  // Track for auto-artifact capture on test failure.
  // @spec .aw/tech-design/projects/jet/logic/auto-artifacts.md#A4
  __jet.activePages.add(pg);
  const origClose = pg.close.bind(pg);
  pg.close = async () => {
    __jet.activePages.delete(pg);
    return origClose();
  };
  return pg;
}

// __sendPageRequest wraps __sendRequest with the req_id correlation.
// Returns the PageResponse from stdin.
async function __sendPageRequest(req) {
  const res = await __sendRequest(req);
  return res;
}

// ── B3: BrowserContext JS wrapper ──────────────────────────────────────────
// `browser.newContext()` sends `NewContext` over the wire channel; the Rust
// worker stashes the resulting BrowserContext and returns the
// `browserContextId`. The JS wrapper routes `newPage()` via `ContextNewPage`
// and `close()` via `CloseContext`, mirroring Playwright's surface.
//
// @spec .aw/issues/open/enhancement-browsercontext-refactor-multi-context-isolation-fo.md#R7
class __JetBrowserContext {
  constructor(contextId, baseURL) {
    this.__jet_context_id = contextId;
    this.__baseURL = baseURL || "";
    this.__closed = false;
  }

  async newPage() {
    if (this.__closed) {
      throw new Error("BrowserContext: already closed");
    }
    const res = await __sendRequest({
      kind: "context_new_page",
      context_id: this.__jet_context_id,
    });
    if (res.kind === "error") {
      throw new Error(`context.newPage: ${res.message}`);
    }
    const pg = new Page(res.page_id, __sendPageRequest, this.__baseURL);
    // @spec .aw/tech-design/projects/jet/logic/auto-artifacts.md#A4
    __jet.activePages.add(pg);
    const origClose = pg.close.bind(pg);
    pg.close = async () => {
      __jet.activePages.delete(pg);
      return origClose();
    };
    return pg;
  }

  // ── Storage state (P3.2) ────────────────────────────────────────────────
  // @spec .aw/tech-design/projects/jet/logic/storage-state.md#S6

  async cookies() {
    const res = await __sendRequest({
      kind: "context_cookies",
      context_id: this.__jet_context_id,
    });
    if (res.kind === "error") throw new Error(`context.cookies: ${res.message}`);
    return Array.isArray(res.value) ? res.value : [];
  }

  async addCookies(cookies) {
    if (!Array.isArray(cookies)) {
      throw new Error("context.addCookies: expected an array of cookie objects");
    }
    const res = await __sendRequest({
      kind: "context_add_cookies",
      context_id: this.__jet_context_id,
      cookies,
    });
    if (res.kind === "error") throw new Error(`context.addCookies: ${res.message}`);
  }

  async clearCookies() {
    const res = await __sendRequest({
      kind: "context_clear_cookies",
      context_id: this.__jet_context_id,
    });
    if (res.kind === "error") throw new Error(`context.clearCookies: ${res.message}`);
  }

  // Returns `{ cookies, origins }`. If `opts.path` is supplied, the JSON is
  // also persisted to that absolute path via dynamic-imported `fs`.
  async storageState(opts) {
    const res = await __sendRequest({
      kind: "context_storage_state",
      context_id: this.__jet_context_id,
    });
    if (res.kind === "error") throw new Error(`context.storageState: ${res.message}`);
    const state = res.value || { cookies: [], origins: [] };
    if (opts && opts.path) {
      const fs = await import("node:fs/promises");
      await fs.writeFile(opts.path, JSON.stringify(state, null, 2));
    }
    return state;
  }

  async setStorageState(state) {
    const res = await __sendRequest({
      kind: "context_set_storage_state",
      context_id: this.__jet_context_id,
      state,
    });
    if (res.kind === "error") throw new Error(`context.setStorageState: ${res.message}`);
  }

  async close() {
    if (this.__closed) return;
    this.__closed = true;
    const res = await __sendRequest({
      kind: "close_context",
      context_id: this.__jet_context_id,
    });
    if (res.kind === "error") {
      throw new Error(`context.close: ${res.message}`);
    }
  }
}

// Public `browser` object exposed to spec code. Only `newContext()` is
// supported in B3 — full browser surface (newBrowserCDPSession, contexts())
// lands in later phases.
// @spec .aw/issues/open/enhancement-browsercontext-refactor-multi-context-isolation-fo.md#R7
const browser = {
  // @spec .aw/issues/open/enhancement-browsercontext-refactor-multi-context-isolation-fo.md#R7
  // @spec .aw/tech-design/projects/jet/logic/storage-state.md#S7
  async newContext(opts) {
    const baseURL = (opts && opts.baseURL) || "";
    const res = await __sendRequest({ kind: "new_context" });
    if (res.kind === "error") {
      throw new Error(`browser.newContext: ${res.message}`);
    }
    const ctx = new __JetBrowserContext(res.context_id, baseURL);
    // storageState may be a string path (load JSON) or an inline object.
    if (opts && opts.storageState != null) {
      let state = opts.storageState;
      if (typeof state === "string") {
        const fs = await import("node:fs/promises");
        const buf = await fs.readFile(state, "utf-8");
        state = JSON.parse(buf);
      }
      await ctx.setStorageState(state);
    }
    return ctx;
  },
};

function __emit(event) {
  process.stdout.write(JSON.stringify(event) + "\n");
}

// ── Stdin NDJSON reader for WireResponse messages ──────────────────────────
// The Rust runner pipes responses for DOM-matcher RPC calls back over stdin.
// Each response carries a `req_id` that correlates with the originating
// request. See projects/jet/src/test_runner/wire.rs::WireResponse.
// @spec ...#R4
let __stdinBuf = "";
process.stdin.on("data", (chunk) => {
  __stdinBuf += chunk.toString("utf-8");
  let nl;
  while ((nl = __stdinBuf.indexOf("\n")) !== -1) {
    const line = __stdinBuf.slice(0, nl).trim();
    __stdinBuf = __stdinBuf.slice(nl + 1);
    if (!line) continue;
    let msg;
    try {
      msg = JSON.parse(line);
    } catch {
      continue;
    }
    const pending = __jet.pending.get(msg.req_id);
    if (!pending) continue;
    __jet.pending.delete(msg.req_id);
    if (msg.kind === "error") {
      pending.reject(msg);
    } else {
      pending.resolve(msg);
    }
  }
});
process.stdin.on("error", () => {}); // worker survives if stdin closes
process.stdin.resume();

function __sendRequest(req) {
  const req_id = ++__jet.reqId;
  const body = { ...req, req_id };
  return new Promise((resolve, reject) => {
    __jet.pending.set(req_id, { resolve, reject });
    process.stdout.write(JSON.stringify(body) + "\n");
  });
}

function __sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

// ── Auto-artifacts on failure (P3.4) ───────────────────────────────────────
// Snap a PNG of every active page into
// `<artifactsDir>/<sanitized-test-name>/page-<n>.png` and return the
// absolute paths. Best-effort — the caller swallows any throw.
//
// @spec .aw/tech-design/projects/jet/logic/auto-artifacts.md#A4 A5
async function __captureFailureArtifacts(testName, artifactsDir) {
  if (!artifactsDir) return [];
  const fs = await import("node:fs/promises");
  const path = await import("node:path");
  const slug = String(testName)
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 80) || "test";
  const dir = path.join(artifactsDir, slug);
  await fs.mkdir(dir, { recursive: true });
  const out = [];
  let i = 0;
  for (const pg of __jet.activePages) {
    i += 1;
    const file = path.join(dir, `page-${i}.png`);
    try {
      await pg.screenshot({ path: file });
      out.push(file);
    } catch {
      // Page may have already been torn down — skip silently.
    }
  }
  return out;
}

function current() {
  return __jet.stack[__jet.stack.length - 1];
}

function describe(name, body) {
  const suite = makeSuite(name, current());
  current().children.push(suite);
  __jet.stack.push(suite);
  try {
    body();
  } finally {
    __jet.stack.pop();
  }
}

function test(name, body) {
  current().tests.push({ name, body, skip: false, only: false, fixtures: null });
}
test.skip = (name, body) => {
  current().tests.push({ name, body, skip: true, only: false, fixtures: null });
};
test.only = (name, body) => {
  current().tests.push({ name, body, skip: false, only: true, fixtures: null });
  __jet.hasOnly = true;
};

// ── test.extend(fixtures) — flat + DI-graph fixture API ──────────────────
// Returns a new `test` function bound to the given fixtures. Each fixture
// may be:
//   - Flat form:     async (use, opts) => { ... await use(value); ... }
//   - Advanced form: async ({dep1, dep2}, use, opts) => { ... }
// In the advanced form, `{dep1, dep2}` destructures previously-resolved
// fixture values. The runtime topologically resolves dependencies per-test
// and detects cycles at fixture-build time.
//
// @spec ...#R6
// @spec .aw/tech-design/projects/jet/logic/fixture-di.md#F1
test.extend = (fixtures) => {
  const boundTest = (name, body) => {
    current().tests.push({
      name,
      body,
      skip: false,
      only: false,
      fixtures,
    });
  };
  boundTest.skip = (name, body) => {
    current().tests.push({
      name,
      body,
      skip: true,
      only: false,
      fixtures,
    });
  };
  boundTest.only = (name, body) => {
    current().tests.push({
      name,
      body,
      skip: false,
      only: true,
      fixtures,
    });
    __jet.hasOnly = true;
  };
  boundTest.extend = (extra) => test.extend({ ...fixtures, ...extra });
  return boundTest;
};

function beforeAll(fn) {
  current().before_all.push(fn);
}
function afterAll(fn) {
  current().after_all.push(fn);
}
function beforeEach(fn) {
  current().before_each.push(fn);
}
function afterEach(fn) {
  current().after_each.push(fn);
}

// Playwright-compatible surface: expose suite builders as methods on `test`
// so specs can write `test.describe(...)` / `test.beforeEach(...)` without
// relying on implicit globals. The standalone globals are still set by
// __jetRun for code that uses them directly.
test.describe = describe;
test.beforeAll = beforeAll;
test.afterAll = afterAll;
test.beforeEach = beforeEach;
test.afterEach = afterEach;

// ── expect() + matchers ─────────────────────────────────────────────────────

class AssertionError extends Error {
  constructor(message, diff) {
    super(message);
    this.name = "AssertionError";
    this.diff = diff;
  }
}

function expect(actual) {
  const obj = __expectBase(actual);
  // @spec #2605 — `expect(x).not.toBe(y)` and friends. Wraps every
  // function-valued matcher so a thrown AssertionError becomes a pass and
  // a clean return becomes a thrown negated AssertionError.
  obj.not = __negate(obj, actual);
  return obj;
}

function __negate(obj, actual) {
  const negated = {};
  for (const [name, fn] of Object.entries(obj)) {
    if (typeof fn !== "function") continue;
    negated[name] = function (...args) {
      let threw = false;
      let result;
      try {
        result = fn.apply(obj, args);
      } catch (e) {
        if (e && e.name === "AssertionError") {
          threw = true;
        } else {
          throw e;
        }
      }
      // Async matchers return promises — handle the negation there too.
      if (result && typeof result.then === "function") {
        return result.then(
          () => {
            throw new AssertionError(
              `Expected not.${name} to fail, but it passed on ${display(actual)}`,
            );
          },
          (err) => {
            if (err && err.name === "AssertionError") return; // expected failure
            throw err;
          },
        );
      }
      if (!threw) {
        throw new AssertionError(
          `Expected not.${name} to fail, but it passed on ${display(actual)}`,
        );
      }
    };
  }
  return negated;
}

function __expectBase(actual) {
  return {
    toBe(expected) {
      if (!Object.is(actual, expected)) {
        throw new AssertionError(
          `Expected ${display(actual)} to be ${display(expected)}`,
          `- ${display(expected)}\n+ ${display(actual)}`,
        );
      }
    },
    toEqual(expected) {
      if (!deepEqual(actual, expected)) {
        throw new AssertionError(
          `Expected deep equal:\n  expected: ${display(expected)}\n    actual: ${display(actual)}`,
          `- ${display(expected)}\n+ ${display(actual)}`,
        );
      }
    },
    toBeTruthy() {
      if (!actual) {
        throw new AssertionError(
          `Expected truthy, got ${display(actual)}`,
        );
      }
    },
    toContain(needle) {
      const ok =
        (typeof actual === "string" && actual.includes(needle)) ||
        (Array.isArray(actual) && actual.includes(needle));
      if (!ok) {
        throw new AssertionError(
          `Expected ${display(actual)} to contain ${display(needle)}`,
        );
      }
    },
    toMatch(pattern) {
      const re = pattern instanceof RegExp ? pattern : new RegExp(String(pattern));
      if (typeof actual !== "string" || !re.test(actual)) {
        throw new AssertionError(`Expected ${display(actual)} to match ${re}`);
      }
    },

    // ── #2605 — Vitest/Jest-parity unit-test matchers ────────────────────
    //
    // Synchronous, structured-failure matchers for the common unit-test
    // surface. Each throws AssertionError with a one-line diff so the
    // reporter can render a clean failure block.
    toBeFalsy() {
      if (actual) {
        throw new AssertionError(`Expected falsy, got ${display(actual)}`);
      }
    },
    toBeNull() {
      if (actual !== null) {
        throw new AssertionError(`Expected null, got ${display(actual)}`);
      }
    },
    toBeUndefined() {
      if (actual !== undefined) {
        throw new AssertionError(`Expected undefined, got ${display(actual)}`);
      }
    },
    toBeDefined() {
      if (actual === undefined) {
        throw new AssertionError(`Expected defined value, got undefined`);
      }
    },
    toBeNaN() {
      if (typeof actual !== "number" || !Number.isNaN(actual)) {
        throw new AssertionError(`Expected NaN, got ${display(actual)}`);
      }
    },
    toBeGreaterThan(n) {
      if (!(typeof actual === "number" && actual > n)) {
        throw new AssertionError(
          `Expected ${display(actual)} to be > ${display(n)}`,
        );
      }
    },
    toBeGreaterThanOrEqual(n) {
      if (!(typeof actual === "number" && actual >= n)) {
        throw new AssertionError(
          `Expected ${display(actual)} to be >= ${display(n)}`,
        );
      }
    },
    toBeLessThan(n) {
      if (!(typeof actual === "number" && actual < n)) {
        throw new AssertionError(
          `Expected ${display(actual)} to be < ${display(n)}`,
        );
      }
    },
    toBeLessThanOrEqual(n) {
      if (!(typeof actual === "number" && actual <= n)) {
        throw new AssertionError(
          `Expected ${display(actual)} to be <= ${display(n)}`,
        );
      }
    },
    toBeCloseTo(n, digits) {
      const precision = digits ?? 2;
      const epsilon = Math.pow(10, -precision) / 2;
      if (
        typeof actual !== "number" ||
        Math.abs(actual - n) >= epsilon
      ) {
        throw new AssertionError(
          `Expected ${display(actual)} to be within ${epsilon} of ${display(n)}`,
        );
      }
    },
    toHaveLength(n) {
      const len = actual == null ? undefined : actual.length;
      if (len !== n) {
        throw new AssertionError(
          `Expected length ${display(n)}, got ${display(len)} on ${display(actual)}`,
        );
      }
    },
    toHaveProperty(path, value) {
      const keys = Array.isArray(path) ? path : String(path).split(".");
      let cur = actual;
      for (const k of keys) {
        if (cur == null || !(k in cur)) {
          throw new AssertionError(
            `Expected ${display(actual)} to have property ${display(path)}`,
          );
        }
        cur = cur[k];
      }
      if (arguments.length > 1 && !deepEqual(cur, value)) {
        throw new AssertionError(
          `Expected property ${display(path)} to equal ${display(value)}, got ${display(cur)}`,
          `- ${display(value)}\n+ ${display(cur)}`,
        );
      }
    },
    toThrow(expected) {
      if (typeof actual !== "function") {
        throw new AssertionError(
          `toThrow: expected a function, got ${display(actual)}`,
        );
      }
      let caught;
      try {
        actual();
      } catch (e) {
        caught = e;
      }
      if (!caught) {
        throw new AssertionError(`Expected function to throw, but it did not`);
      }
      if (expected === undefined) return;
      const msg = caught && caught.message ? caught.message : String(caught);
      if (expected instanceof RegExp) {
        if (!expected.test(msg)) {
          throw new AssertionError(
            `Expected thrown message to match ${expected}, got ${display(msg)}`,
          );
        }
      } else if (typeof expected === "string") {
        if (!msg.includes(expected)) {
          throw new AssertionError(
            `Expected thrown message to contain ${display(expected)}, got ${display(msg)}`,
          );
        }
      } else if (typeof expected === "function" && !(caught instanceof expected)) {
        throw new AssertionError(
          `Expected thrown error to be instance of ${expected.name}, got ${display(caught)}`,
        );
      }
    },

    // ── Phase-6 polling matchers (matchers.js) ───────────────────────────
    // Dispatched by argument type: page matchers route to Page methods;
    // locator matchers route to Locator methods.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R20

    // toHaveTitle: page-only matcher.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R20
    async toHaveTitle(expected, opts) {
      if (!actual || !actual.__jet_page_id) {
        throw new Error("toHaveTitle: expected a Page object (with __jet_page_id)");
      }
      return toHaveTitle(actual, expected, opts);
    },

    // toHaveURL: page-only matcher.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R21
    async toHaveURL(expected, opts) {
      if (!actual || !actual.__jet_page_id) {
        throw new Error("toHaveURL: expected a Page object (with __jet_page_id)");
      }
      return toHaveURL(actual, expected, opts);
    },

    // toBeVisible (locator-backed, new form): dispatch to Locator.isVisible().
    // The old toBeVisible(selector, opts) form with a string argument remains below.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R22
    async toBeVisible(selectorOrOpts, opts) {
      if (actual instanceof Locator) {
        // New locator-backed form: expect(locator).toBeVisible(opts?)
        return toBeVisibleLocator(actual, selectorOrOpts);
      }
      // Old page-selector form: expect(page).toBeVisible(selector, opts)
      const options = opts ?? {};
      const pageId = (actual && actual.__jet_page_id) ?? "default";
      const timeout = options.timeout ?? 5000;
      const start = Date.now();
      let lastError = null;
      while (true) {
        try {
          const res = await __sendRequest({
            kind: "is_visible",
            page_id: pageId,
            selector: selectorOrOpts,
          });
          if (res.visible) return;
        } catch (err) {
          lastError = err;
        }
        if (Date.now() - start >= timeout) {
          const msg = lastError
            ? `toBeVisible(${JSON.stringify(selectorOrOpts)}): ${lastError.message ?? String(lastError)}`
            : `Expected ${selectorOrOpts} to be visible within ${timeout}ms`;
          throw new AssertionError(msg);
        }
        await __sleep(100);
      }
    },

    // toBeHidden: locator-only matcher.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R22
    async toBeHidden(opts) {
      if (!(actual instanceof Locator)) {
        throw new Error("toBeHidden: expected a Locator object");
      }
      return toBeHidden(actual, opts);
    },

    // toHaveValue: locator-only matcher.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R24
    async toHaveValue(expected, opts) {
      if (!(actual instanceof Locator)) {
        throw new Error("toHaveValue: expected a Locator object");
      }
      return toHaveValue(actual, expected, opts);
    },

    // toHaveCount: locator-only matcher.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R25
    async toHaveCount(expected, opts) {
      if (!(actual instanceof Locator)) {
        throw new Error("toHaveCount: expected a Locator object");
      }
      return toHaveCount(actual, expected, opts);
    },

    // toHaveClass: locator-only matcher.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R26
    async toHaveClass(expected, opts) {
      if (!(actual instanceof Locator)) {
        throw new Error("toHaveClass: expected a Locator object");
      }
      return toHaveClass(actual, expected, opts);
    },

    // toHaveAttribute: locator-only matcher.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R27
    async toHaveAttribute(name, expected, opts) {
      if (!(actual instanceof Locator)) {
        throw new Error("toHaveAttribute: expected a Locator object");
      }
      return toHaveAttribute(actual, name, expected, opts);
    },

    // ── DOM-integrated matchers (Phase 3 + Phase 6 locator dispatch) ─────
    // toHaveText dispatches by argument type:
    //   - If actual is a Locator → locator-backed (innerText polling).
    //   - Otherwise → page-selector-based (query_text WireRequest, backward compat).
    // @spec ...#R1
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R23
    async toHaveText(selector, expected, opts) {
      if (actual instanceof Locator) {
        // Locator-backed: selector is actually the expected text, expected is opts.
        return toHaveTextLocator(actual, selector, expected);
      }
      // Page-selector form (Phase 3 backward compat): actual is a page object.
      const options = opts ?? {};
      const pageId = (actual && actual.__jet_page_id) ?? "default";
      const timeout = options.timeout ?? 5000;
      const start = Date.now();
      let lastText = null;
      let lastError = null;
      while (true) {
        try {
          const res = await __sendRequest({
            kind: "query_text",
            page_id: pageId,
            selector,
          });
          lastText = res.text;
          if (__textMatches(res.text, expected)) return;
        } catch (err) {
          lastError = err;
        }
        if (Date.now() - start >= timeout) {
          const msg = lastError
            ? `toHaveText(${JSON.stringify(selector)}): ${lastError.message ?? String(lastError)}`
            : `Expected ${selector} to have text ${display(expected)}, got ${display(lastText)}`;
          throw new AssertionError(
            msg,
            `- expected: ${display(expected)}\n+ actual:   ${display(lastText)}`,
          );
        }
        await __sleep(100);
      }
    },
    // ── Phase 5 matchers: element-state + CSS + a11y + value ────────────

    // @spec matchers-state-value-a11y#M1
    async toBeChecked(opts) {
      if (!(actual instanceof Locator)) throw new Error("toBeChecked: expected a Locator");
      return toBeChecked(actual, opts);
    },

    // @spec matchers-state-value-a11y#M2
    async toBeDisabled(opts) {
      if (!(actual instanceof Locator)) throw new Error("toBeDisabled: expected a Locator");
      return toBeDisabled(actual, opts);
    },

    // @spec matchers-state-value-a11y#M3
    async toBeEnabled(opts) {
      if (!(actual instanceof Locator)) throw new Error("toBeEnabled: expected a Locator");
      return toBeEnabled(actual, opts);
    },

    // @spec matchers-state-value-a11y#M4
    async toBeFocused(opts) {
      if (!(actual instanceof Locator)) throw new Error("toBeFocused: expected a Locator");
      return toBeFocused(actual, opts);
    },

    // @spec matchers-state-value-a11y#M5
    async toHaveCSS(name, expected, opts) {
      if (!(actual instanceof Locator)) throw new Error("toHaveCSS: expected a Locator");
      return toHaveCSS(actual, name, expected, opts);
    },

    // @spec matchers-state-value-a11y#M6
    async toHaveAccessibleName(expected, opts) {
      if (!(actual instanceof Locator)) throw new Error("toHaveAccessibleName: expected a Locator");
      return toHaveAccessibleName(actual, expected, opts);
    },

    // @spec matchers-state-value-a11y#M7
    async toHaveRole(expected, opts) {
      if (!(actual instanceof Locator)) throw new Error("toHaveRole: expected a Locator");
      return toHaveRole(actual, expected, opts);
    },

    // @spec matchers-state-value-a11y#M8
    toMatchObject(expected) {
      if (!matchObject(actual, expected)) {
        throw new AssertionError(
          `Expected value to match object:\n  expected: ${display(expected)}\n    actual: ${display(actual)}`,
          `- ${display(expected)}\n+ ${display(actual)}`,
        );
      }
    },

    // @spec ...#R3
    // @spec ...#R7
    // @spec ...#R8
    async toMatchSnapshot(name) {
      const pageId = (actual && actual.__jet_page_id) ?? "default";
      const snapshotName = name ?? __jet.currentTestTitle ?? "snapshot";
      try {
        await __sendRequest({
          kind: "match_snapshot",
          page_id: pageId,
          snapshot_name: snapshotName,
        });
        // Pass on success (either wrote new baseline or bytes matched).
      } catch (err) {
        const diff = err && err.matcher_diff
          ? `- expected: ${err.matcher_diff.expected}\n+ actual:   ${err.matcher_diff.actual}`
          : null;
        throw new AssertionError(
          `toMatchSnapshot(${JSON.stringify(snapshotName)}): ${err?.message ?? String(err)}`,
          diff,
        );
      }
    },

    // @spec #2713
    // expect(value).toMatchTextSnapshot(name?) — compares the serialized
    // form of `value` against a text baseline at
    // `<spec-dir>/__snapshots__/<spec-slug>/<name>.txt`.
    //
    // - Strings are compared verbatim.
    // - All other values are serialized with `JSON.stringify(value, null, 2)`
    //   using a stable key order so the baseline is diff-friendly.
    // - First run (or `--update-snapshots`) writes the baseline and passes.
    // - Mismatch fails with a unified line-by-line diff.
    async toMatchTextSnapshot(name) {
      const snapshotName = name ?? __jet.currentTestTitle ?? "snapshot";
      const serialized = __serializeForTextSnapshot(actual);
      try {
        await __sendRequest({
          kind: "match_text_snapshot",
          snapshot_name: snapshotName,
          content: serialized,
        });
      } catch (err) {
        const diff = err && err.matcher_diff
          ? __formatTextSnapshotDiff(
              err.matcher_diff.expected,
              err.matcher_diff.actual,
            )
          : null;
        throw new AssertionError(
          `toMatchTextSnapshot(${JSON.stringify(snapshotName)}): ${err?.message ?? String(err)}`,
          diff,
        );
      }
    },

    // @spec .aw/tech-design/projects/jet/logic/to-have-screenshot.md#S1
    // expect(page).toHaveScreenshot(name?, opts?) — visual regression on a
    // byte-exact baseline.
    //
    // First run: captures the PNG and writes the baseline. Pass.
    // Subsequent runs: captures again, compares bytes. Mismatch → fail with
    // a diff note ("PNG N bytes vs baseline M bytes").
    //
    // `--update-snapshots` overrides: always rewrite the baseline and pass.
    async toHaveScreenshot(name, _opts) {
      if (!actual || !actual.__jet_page_id) {
        throw new Error("toHaveScreenshot: expected a Page object");
      }
      // Option (a): name omitted — use currentTestTitle. Option (b): first arg
      // is opts object (Playwright shorthand `expect(page).toHaveScreenshot({...})`).
      let snapshotName = name;
      if (snapshotName && typeof snapshotName === "object") {
        snapshotName = undefined;
      }
      snapshotName = snapshotName ?? __jet.currentTestTitle ?? "screenshot";
      try {
        await __sendRequest({
          kind: "match_snapshot",
          page_id: actual.__jet_page_id,
          snapshot_name: snapshotName,
        });
      } catch (err) {
        const diff = err && err.matcher_diff
          ? `- expected: ${err.matcher_diff.expected}\n+ actual:   ${err.matcher_diff.actual}`
          : null;
        throw new AssertionError(
          `toHaveScreenshot(${JSON.stringify(snapshotName)}): ${err?.message ?? String(err)}`,
          diff,
        );
      }
    },
  };
}

function __textMatches(actual, expected) {
  if (expected instanceof RegExp) return typeof actual === "string" && expected.test(actual);
  if (typeof expected === "string") return actual === expected;
  return Object.is(actual, expected);
}

// @spec #2713 — deterministic snapshot serialization. Strings are
// verbatim; everything else round-trips through JSON.stringify with a
// stable key order so the on-disk baseline is diff-friendly across
// repeated runs and across workers.
function __serializeForTextSnapshot(value) {
  if (typeof value === "string") return value;
  const sortKeysReplacer = (_key, val) => {
    if (val && typeof val === "object" && !Array.isArray(val)) {
      const sorted = {};
      for (const k of Object.keys(val).sort()) sorted[k] = val[k];
      return sorted;
    }
    return val;
  };
  try {
    return JSON.stringify(value, sortKeysReplacer, 2);
  } catch (err) {
    throw new Error(
      `toMatchTextSnapshot: value is not serialisable (${err?.message ?? err})`,
    );
  }
}

// @spec #2713 — format a unified-ish line diff for a text snapshot
// mismatch. Marks every expected line with `-` and every actual line
// with `+`. Compact, copy-pasteable, no diff-library dependency.
function __formatTextSnapshotDiff(expected, actual) {
  const expLines = String(expected).split("\n");
  const actLines = String(actual).split("\n");
  const out = [];
  for (const line of expLines) out.push(`- ${line}`);
  for (const line of actLines) out.push(`+ ${line}`);
  return out.join("\n");
}

function display(v) {
  try {
    if (typeof v === "string") return JSON.stringify(v);
    if (typeof v === "function") return `[Function ${v.name || "anon"}]`;
    if (v === undefined) return "undefined";
    if (v === null) return "null";
    return JSON.stringify(v);
  } catch {
    return String(v);
  }
}

function deepEqual(a, b) {
  if (Object.is(a, b)) return true;
  if (typeof a !== typeof b) return false;
  if (a == null || b == null) return false;
  if (Array.isArray(a) !== Array.isArray(b)) return false;
  if (Array.isArray(a)) {
    if (a.length !== b.length) return false;
    for (let i = 0; i < a.length; i++) if (!deepEqual(a[i], b[i])) return false;
    return true;
  }
  if (typeof a === "object") {
    const ak = Object.keys(a);
    const bk = Object.keys(b);
    if (ak.length !== bk.length) return false;
    for (const k of ak) if (!deepEqual(a[k], b[k])) return false;
    return true;
  }
  return false;
}

// ── Runner entry point ──────────────────────────────────────────────────────

export async function __jetRun(opts) {
  globalThis.describe = describe;
  globalThis.test = test;
  globalThis.it = test;
  globalThis.expect = expect;
  globalThis.beforeAll = beforeAll;
  globalThis.afterAll = afterAll;
  globalThis.beforeEach = beforeEach;
  globalThis.afterEach = afterEach;

  try {
    await import(opts.specUrl);
  } catch (err) {
    __emit({
      kind: "fatal",
      message: `spec import failed: ${err?.stack ?? err?.message ?? err}`,
    });
    process.exit(2);
  }

  // Collect the plan (flat list of test descriptors).
  const tests = [];
  let nextId = 0;
  function collect(suite, path) {
    const here = suite.name ? [...path, suite.name] : path;
    for (const t of suite.tests) {
      tests.push({
        id: String(nextId++),
        suite: here,
        name: t.name,
        skip: t.skip,
      });
    }
    for (const child of suite.children) collect(child, here);
  }
  collect(__jet.root, []);

  __emit({ kind: "plan", file: opts.file, tests });

  const grep = opts.grep;
  let idCounter = 0;
  await runSuite(__jet.root, [], opts, grep, () => String(idCounter++));

  await new Promise((r) => process.stdout.write("", () => r()));
  process.exit(0);
}

async function runSuite(suite, parentPath, opts, grep, nextId) {
  const path = suite.name ? [...parentPath, suite.name] : parentPath;

  for (const hook of suite.before_all) {
    try {
      await hook();
    } catch (err) {
      __emit({
        kind: "fatal",
        message: `beforeAll threw: ${err?.stack ?? err?.message ?? err}`,
      });
      return;
    }
  }

  for (const t of suite.tests) {
    const id = nextId();
    const fullName = [...path, t.name].join(" > ");

    if (t.skip || (__jet.hasOnly && !t.only) || (grep && !grep.test(fullName))) {
      __emit({
        kind: "test_end",
        id,
        suite: path,
        name: t.name,
        outcome: "skipped",
        duration_ms: 0,
        error: null,
      });
      continue;
    }

    __emit({ kind: "test_start", id, suite: path, name: t.name });
    if (opts.liveControl) {
      await __sendRequest({
        kind: "live_checkpoint",
        test_id: id,
        title: fullName,
      });
    }

    const started = Date.now();
    let outcome = "passed";
    let error = null;
    // P3.4: artifact paths captured on failure (screenshots today).
    // @spec .aw/tech-design/projects/jet/logic/auto-artifacts.md#A1
    const artifacts = [];

    // Apply every enclosing suite's beforeEach (outer → inner).
    const chain = ancestorChain(suite);
    for (const s of chain) {
      for (const hook of s.before_each) {
        try {
          await hook();
        } catch (err) {
          outcome = "failed";
          error = toWireError(err, "beforeEach");
          break;
        }
      }
      if (outcome !== "passed") break;
    }

    let fixtureCleanups = [];

    if (outcome === "passed") {
      __jet.currentTestTitle = t.name;
      try {
        // Build fixture argument. Merge default fixtures (page) with user
        // test.extend fixtures (user fixtures take precedence). Only resolve
        // fixtures whose names appear in the test body's destructured param.
        //
        // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R1
        // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R7
        // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R9

        // Merged fixture map: defaults overridden by user-supplied fixtures.
        // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R7
        const mergedFixtures = { ...__DEFAULT_FIXTURES, ...(t.fixtures || {}) };

        // Detect which fixture names the test body actually destructures.
        // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R1
        const neededNames = __detectFixtureNames(t.body);

        // Only build a fixtureArg if any fixture name appears in the body.
        // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R9
        const fixtureKeysNeeded = Object.keys(mergedFixtures).filter(
          (k) => neededNames.has(k)
        );

        // Per-test timeout covers BOTH fixture setup AND the test body.
        //
        // Issue #1534: previously only `t.body(fixtureArg)` was wrapped in
        // `withTimeout`, so a fixture (e.g. the built-in `page` fixture
        // launching a browser) that never resolved would hang the worker
        // indefinitely and `--timeout` was ineffective. We now race the
        // entire setup-and-run pipeline against a single deadline so any
        // hang in fixture initialization terminates with a clear error.
        const runWithFixtures = async () => {
          let fixtureArg = undefined;
          if (fixtureKeysNeeded.length > 0) {
            fixtureArg = {};
            // Resolve fixtures via topological walk: each fixture's deps are
            // resolved before it runs. Cycles throw. Flat fixtures (no deps)
            // resolve in arbitrary order within their cohort.
            // @spec .aw/tech-design/projects/jet/logic/fixture-di.md#F3 F4
            const resolving = new Set();
            const resolvedMap = new Map();
            const resolveFixture = async (key) => {
              if (resolvedMap.has(key)) return resolvedMap.get(key);
              if (resolving.has(key)) {
                throw new Error(
                  `Fixture DI cycle detected involving "${key}"`
                );
              }
              resolving.add(key);
              const fn = mergedFixtures[key];
              if (fn === undefined) {
                throw new Error(`Fixture "${key}" is not defined`);
              }
              if (typeof fn !== "function") {
                resolving.delete(key);
                resolvedMap.set(key, fn);
                return fn;
              }
              const shape = __fixtureShape(fn);
              const deps = {};
              if (shape.form === "advanced") {
                for (const depName of shape.deps) {
                  deps[depName] = await resolveFixture(depName);
                }
              }
              let resolved;
              let useDone;
              let cleanupDone;
              const donePromise = new Promise((r) => { useDone = r; });
              const cleanupPromise = new Promise((r) => { cleanupDone = r; });
              const useFn = async (value) => {
                resolved = value;
                useDone();
                await cleanupPromise;
              };
              const fixturePromise =
                shape.form === "advanced"
                  ? fn(deps, useFn, opts).catch((err) => err)
                  : fn(useFn, opts).catch((err) => err);
              await donePromise;
              fixtureCleanups.push(async () => {
                cleanupDone();
                const maybeErr = await fixturePromise;
                if (maybeErr instanceof Error) throw maybeErr;
              });
              resolving.delete(key);
              resolvedMap.set(key, resolved);
              return resolved;
            };
            for (const key of fixtureKeysNeeded) {
              fixtureArg[key] = await resolveFixture(key);
            }
          }
          await t.body(fixtureArg);
        };
        await withTimeout(runWithFixtures(), opts.timeoutMs);
      } catch (err) {
        if (err && err.__jet_timeout) {
          outcome = "timed_out";
          error = {
            message: `Test timed out after ${opts.timeoutMs}ms`,
            stack: null,
            diff: null,
          };
        } else {
          outcome = "failed";
          error = toWireError(err, "test");
        }
      } finally {
        // P3.4: capture screenshots of every active page before fixture
        // teardown closes them. Best-effort — we never let an artifact
        // failure mask the original test failure.
        // @spec .aw/tech-design/projects/jet/logic/auto-artifacts.md#A4
        if (
          (outcome === "failed" || outcome === "timed_out") &&
          opts.autoArtifacts &&
          __jet.activePages.size > 0
        ) {
          try {
            const captured = await __captureFailureArtifacts(
              t.name,
              opts.artifactsDir,
            );
            artifacts.push(...captured);
          } catch {
            // ignore — never overwrite the original test error
          }
        }
        // Run fixture cleanups in reverse order.
        for (const cleanup of [...fixtureCleanups].reverse()) {
          try {
            await cleanup();
          } catch (err) {
            if (outcome === "passed") {
              outcome = "failed";
              error = toWireError(err, "fixture-cleanup");
            }
          }
        }
        __jet.currentTestTitle = null;
      }
    }

    // afterEach runs in reverse order (inner → outer).
    for (const s of [...chain].reverse()) {
      for (const hook of s.after_each) {
        try {
          await hook();
        } catch (err) {
          if (outcome === "passed") {
            outcome = "failed";
            error = toWireError(err, "afterEach");
          }
        }
      }
    }

    __emit({
      kind: "test_end",
      id,
      suite: path,
      name: t.name,
      outcome,
      duration_ms: Date.now() - started,
      error,
      artifacts,
    });
  }

  for (const child of suite.children) {
    await runSuite(child, path, opts, grep, nextId);
  }

  for (const hook of suite.after_all) {
    try {
      await hook();
    } catch (err) {
      __emit({
        kind: "fatal",
        message: `afterAll threw: ${err?.stack ?? err?.message ?? err}`,
      });
    }
  }
}

function ancestorChain(suite) {
  const chain = [];
  let cur = suite;
  while (cur) {
    chain.unshift(cur);
    cur = cur.parent;
  }
  return chain;
}

function withTimeout(promise, ms) {
  let handle;
  const timeout = new Promise((_, reject) => {
    handle = setTimeout(() => {
      const e = new Error("timeout");
      e.__jet_timeout = true;
      reject(e);
    }, ms);
  });
  return Promise.race([Promise.resolve(promise), timeout]).finally(() =>
    clearTimeout(handle),
  );
}

function toWireError(err, source) {
  return {
    message: `[${source}] ${err?.message ?? String(err)}`,
    stack: err?.stack ?? null,
    diff: err?.diff ?? null,
  };
}

// ── Public named exports for `@jet/test` bare specifier ────────────────────
// Specs migrated off `@playwright/test` (Phase 5b) import these as named
// exports. `Page` is re-exported from ./page.js (the CDP-backed implementation
// imported above) so `import { Page }` in specs resolves to the live class.
export {
  describe,
  test,
  expect,
  beforeAll,
  afterAll,
  beforeEach,
  afterEach,
  Page,
  browser,
};

// ── Virtual-module contract (#2608) ────────────────────────────────────────
// Source-of-truth list of supported names exposed by `@jet/test`. Specs that
// want to introspect the contract can `import { __JET_TEST_CONTRACT } from
// "@jet/test"` and check membership at runtime.
export const __JET_TEST_CONTRACT = Object.freeze([
  "describe",
  "test",
  "expect",
  "beforeAll",
  "afterAll",
  "beforeEach",
  "afterEach",
  "Page",
  "browser",
  "__JET_TEST_CONTRACT",
]);

// Tripwire for symbols that are commonly reached for in Jest/Vitest/Jasmine
// codebases but are NOT part of the @jet/test contract. Importing the name
// succeeds (ESM static binding) so legacy code keeps parsing; the first
// runtime access throws a Jet-owned diagnostic that names the gap and points
// the user at the supported alternative.
function __jetUnsupported(symbol, alt) {
  const altText = alt ? ` Use ${alt} instead.` : "";
  const supported = __JET_TEST_CONTRACT.filter((n) => !n.startsWith("__")).join(", ");
  const e = new Error(
    `@jet/test: \`${symbol}\` is not part of the @jet/test contract. ` +
      `Supported: ${supported}.${altText} ` +
      `See projects/jet/data/runtime/test/CONTRACT.md.`
  );
  e.name = "JetTestUnsupportedError";
  return e;
}

function __makeTripwire(symbol, alt) {
  const throwIt = () => {
    throw __jetUnsupported(symbol, alt);
  };
  return new Proxy(throwIt, {
    get(_target, prop) {
      // Allow trivial debugging coercions so a stray `console.log(vi)` does
      // not throw before the spec ever calls into the tripwire.
      if (
        prop === Symbol.toPrimitive ||
        prop === Symbol.toStringTag ||
        prop === "toString" ||
        prop === "inspect" ||
        prop === "valueOf"
      ) {
        return () => `[jet-unsupported ${symbol}]`;
      }
      throw __jetUnsupported(symbol, alt);
    },
    apply() {
      throwIt();
    },
  });
}

export const jest = __makeTripwire(
  "jest",
  "`@jet/test` (describe/test/expect) and the post-#2605 matchers"
);
export const vi = __makeTripwire(
  "vi",
  "`@jet/test` (describe/test/expect) and the post-#2605 matchers"
);
export const vitest = __makeTripwire("vitest", "`@jet/test`");
export const mock = __makeTripwire(
  "mock",
  "manual fakes — built-in mocking is not in the @jet/test contract yet"
);
export const fail = __makeTripwire(
  "fail",
  "`throw new Error(...)` inside the test body, or an `expect(...)` assertion"
);
// CODEGEN-END
