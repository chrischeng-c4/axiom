// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-data-runtime-test.md#logic
// CODEGEN-BEGIN
// jet test worker runtime.
//
// Exposes `describe`, `test`, `expect`, and the before/after hooks as globals
// for the imported spec module, runs the collected plan, and streams NDJSON
// events back to the Rust runner via stdout.
//
// Wire format: see apps/jet/src/test_runner/wire.rs.
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
  currentTestId: null,
  currentStepSeq: 0,
  currentStepStack: [],
  // P3.4: Active pages registered by the page fixture or browser.newContext.
  // On test failure, the runner snaps a PNG of every entry in this set so
  // the developer doesn't have to reproduce the failure to see UI state.
  // @spec .aw/tech-design/projects/jet/logic/auto-artifacts.md#A4
  activePages: new Set(),
  pagesById: new Map(),
};
__jet.stack.push(__jet.root);

// ── Page-fixture wire protocol ─────────────────────────────────────────────
// PageRequest messages flow over stdout; PageResponse messages come back over
// stdin alongside WireResponse messages. They are distinguished by `kind` tag:
// PageRequest kinds are listed in cdp_driver::page_binding::PageRequest.
// The __jet.pending map (keyed by req_id) is shared between all wire message
// types so one __sendRequest implementation serves both.

import { Page, Locator } from "./page.js";
import { createRequire } from "node:module";
import { resolve as resolvePath } from "node:path";
// @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R20
import {
  toHaveTitle,
  toHaveURL,
  toBeVisibleLocator,
  toBeHidden,
  toHaveTextLocator,
  toContainTextLocator,
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
  setDefaultAssertionTimeout,
  DEFAULT_ASSERTION_TIMEOUT_MS,
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
  __jet.pagesById.set(pageId, pg);
  // Track for auto-artifact capture on test failure.
  // @spec .aw/tech-design/projects/jet/logic/auto-artifacts.md#A4
  __jet.activePages.add(pg);
  const origClose = pg.close.bind(pg);
  pg.close = async () => {
    __jet.activePages.delete(pg);
    __jet.pagesById.delete(pageId);
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
    __jet.pagesById.set(res.page_id, pg);
    // @spec .aw/tech-design/projects/jet/logic/auto-artifacts.md#A4
    __jet.activePages.add(pg);
    const origClose = pg.close.bind(pg);
    pg.close = async () => {
      __jet.activePages.delete(pg);
      __jet.pagesById.delete(res.page_id);
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
// request. See apps/jet/src/test_runner/wire.rs::WireResponse.
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
    if (msg.kind === "event") {
      const page = __jet.pagesById.get(msg.page_id);
      if (page) {
        page._dispatchEvent(msg.event, msg.payload);
      }
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
  // Runner polling must remain on the real event loop even when a spec opts
  // into fake timers.
  return new Promise((resolve) => __nativeTimers.setTimeout(resolve, ms));
}

// ── Auto-artifacts on failure (P3.4) ───────────────────────────────────────
// Snap a PNG of every active page into
// `<artifactsDir>/<sanitized-test-name>/page-<n>.png` and return the
// absolute paths. Best-effort — the caller swallows any throw.
//
// @spec .aw/tech-design/projects/jet/logic/auto-artifacts.md#A4 A5
const DEFAULT_FAILURE_SCREENSHOT_TIMEOUT_MS = 5000;

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
      await pg.screenshot({ path: file, timeout: DEFAULT_FAILURE_SCREENSHOT_TIMEOUT_MS });
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

function eachRowValues(row) {
  return Array.isArray(row) ? row : [row];
}

function formatEachTitle(name, values, index) {
  let valueIndex = 0;
  const title = String(name).replace(/%[sdifjoO%]/g, (token) => {
    if (token === "%%") return "%";
    const value = values[valueIndex++];
    switch (token) {
      case "%d":
      case "%i":
        return String(Number.parseInt(value, 10));
      case "%f":
        return String(Number(value));
      case "%j":
        try {
          return JSON.stringify(value);
        } catch {
          return "[Circular]";
        }
      case "%o":
      case "%O":
        return typeof value === "string" ? value : JSON.stringify(value);
      case "%s":
      default:
        return String(value);
    }
  });
  return title.replace(/\$#/g, String(index));
}

function makeEach(register) {
  return (table) => {
    if (!Array.isArray(table)) {
      throw new TypeError("test.each/describe.each expects an array of rows");
    }
    return (name, body) => {
      if (typeof body !== "function") {
        throw new TypeError("test.each/describe.each requires a callback");
      }
      table.forEach((row, index) => {
        const values = eachRowValues(row);
        register(formatEachTitle(name, values, index), () => body(...values));
      });
    };
  };
}

test.skip = (name, body) => {
  current().tests.push({ name, body, skip: true, only: false, fixtures: null });
};
test.only = (name, body) => {
  current().tests.push({ name, body, skip: false, only: true, fixtures: null });
  __jet.hasOnly = true;
};
test.step = async (name, body) => {
  const title = String(name ?? "step");
  if (!__jet.currentTestId) {
    return await body();
  }
  __jet.currentStepSeq += 1;
  const step_id = `step-${String(__jet.currentStepSeq).padStart(4, "0")}`;
  const parent_step_id =
    __jet.currentStepStack[__jet.currentStepStack.length - 1] ?? null;
  const started = __realNow();
  __emit({
    kind: "step_start",
    test_id: __jet.currentTestId,
    step_id,
    title,
    parent_step_id,
  });
  __jet.currentStepStack.push(step_id);
  try {
    const result = await body();
    __emit({
      kind: "step_end",
      test_id: __jet.currentTestId,
      step_id,
      title,
      parent_step_id,
      outcome: "passed",
      duration_ms: __realNow() - started,
      error: null,
    });
    return result;
  } catch (err) {
    __emit({
      kind: "step_end",
      test_id: __jet.currentTestId,
      step_id,
      title,
      parent_step_id,
      outcome: "failed",
      duration_ms: __realNow() - started,
      error: toWireError(err, "step"),
    });
    throw err;
  } finally {
    __jet.currentStepStack.pop();
  }
};
test.each = makeEach(test);
test.skip.each = makeEach(test.skip);
test.only.each = makeEach(test.only);

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
  boundTest.step = test.step;
  boundTest.each = makeEach(boundTest);
  boundTest.skip.each = makeEach(boundTest.skip);
  boundTest.only.each = makeEach(boundTest.only);
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

describe.each = makeEach(describe);

const __jestMockFunctions = new Set();
const __jestSpies = new Set();
const __jestModuleMocks = new Map();
const __jestRequiredModuleIds = new Set();
let __jestNodeRequire = null;

// Keep the exact native constructor and clock separate from the test-facing
// fake clock. The runner itself relies on these for deadlines and telemetry.
const __nativeDate = globalThis.Date;
const __nativeDateNow = __nativeDate.now.bind(__nativeDate);

function __realNow() {
  return __nativeDateNow();
}

// The test worker is native ESM, but Jest's `requireActual` contract is a
// synchronous CommonJS API. Keep a project/spec-relative require just for
// that explicit escape hatch. Static ESM imports still belong to Node's ESM
// graph and deliberately cannot be intercepted or reset from this runtime.
function __setJestRequireForSpec(file) {
  const relativeFile =
    typeof file === "string" && file.length > 0
      ? file
      : "__jet_test_runtime__.cjs";
  __jestNodeRequire = createRequire(resolvePath(process.cwd(), relativeFile));
}

function __getJestNodeRequire() {
  if (__jestNodeRequire == null) {
    __setJestRequireForSpec(null);
  }
  return __jestNodeRequire;
}

function __trackJestRequiredModuleTree(entryId) {
  const nodeRequire = __getJestNodeRequire();
  const cache = nodeRequire.cache;
  if (!cache) {
    __jestRequiredModuleIds.add(entryId);
    return;
  }

  const pending = [entryId];
  const seen = new Set();
  while (pending.length > 0) {
    const moduleId = pending.pop();
    if (!moduleId || seen.has(moduleId)) continue;
    seen.add(moduleId);
    __jestRequiredModuleIds.add(moduleId);
    const cached = cache[moduleId];
    for (const child of cached?.children ?? []) {
      if (child?.id) pending.push(child.id);
    }
  }
}

function __resetJestModuleRegistry() {
  const cache = __getJestNodeRequire().cache;
  if (cache) {
    for (const moduleId of __jestRequiredModuleIds) {
      delete cache[moduleId];
    }
  }
  __jestRequiredModuleIds.clear();
}

const __nativeTimers = Object.freeze({
  setTimeout: globalThis.setTimeout.bind(globalThis),
  clearTimeout: globalThis.clearTimeout.bind(globalThis),
  setInterval: globalThis.setInterval.bind(globalThis),
  clearInterval: globalThis.clearInterval.bind(globalThis),
  setImmediate:
    typeof globalThis.setImmediate === "function"
      ? globalThis.setImmediate.bind(globalThis)
      : null,
  clearImmediate:
    typeof globalThis.clearImmediate === "function"
      ? globalThis.clearImmediate.bind(globalThis)
      : null,
});

// Deliberately small fake-timer clock. Test code sees its scheduled callbacks
// and Date clock, while the runner keeps using the captured native clock.
const __fakeTimers = {
  enabled: false,
  now: 0,
  nextId: 1,
  timers: new Map(),
};

function __fakeDate(...args) {
  if (new.target) {
    return args.length === 0
      ? new __nativeDate(__fakeTimers.now)
      : new __nativeDate(...args);
  }
  return new __nativeDate(__fakeTimers.now).toString();
}

// Preserve Date's static helpers and native Date instances while substituting
// only the fake clock-facing constructor behavior.
Object.setPrototypeOf(__fakeDate, __nativeDate);
__fakeDate.prototype = __nativeDate.prototype;
__fakeDate.now = () => __fakeTimers.now;

function __timerDelay(value) {
  const numeric = Number(value);
  return Number.isFinite(numeric) && numeric > 0 ? numeric : 0;
}

function __scheduleFakeTimer(callback, delay, args, intervalMs = null) {
  if (typeof callback !== "function") {
    throw new TypeError("fake timers require a function callback");
  }
  const id = __fakeTimers.nextId++;
  __fakeTimers.timers.set(id, {
    id,
    callback,
    args,
    due: __fakeTimers.now + __timerDelay(delay),
    intervalMs,
  });
  return id;
}

function __clearFakeTimer(id, clearNative) {
  if (__fakeTimers.timers.delete(id)) return;
  // The runner may have installed a native timeout before a test switches to
  // fake timers. Delegate unknown handles so its own watchdog still cleans up.
  clearNative(id);
}

function __nextFakeTimer(maxDue = Number.POSITIVE_INFINITY) {
  let next = null;
  for (const timer of __fakeTimers.timers.values()) {
    if (timer.due > maxDue) continue;
    if (
      next == null ||
      timer.due < next.due ||
      (timer.due === next.due && timer.id < next.id)
    ) {
      next = timer;
    }
  }
  return next;
}

function __runFakeTimer(timer) {
  if (!__fakeTimers.timers.has(timer.id)) return;
  if (timer.intervalMs == null) {
    __fakeTimers.timers.delete(timer.id);
  } else {
    // Requeue before invoking the callback so clearInterval() inside it wins.
    timer.due += timer.intervalMs;
  }
  return timer.callback(...timer.args);
}

function __requireFakeTimers(api) {
  if (!__fakeTimers.enabled) {
    throw new Error(`${api} requires jest.useFakeTimers()`);
  }
}

function __advanceFakeTimersBy(ms) {
  __requireFakeTimers("jest.advanceTimersByTime");
  const target = __fakeTimers.now + __timerDelay(ms);
  let timer;
  while ((timer = __nextFakeTimer(target)) != null) {
    __fakeTimers.now = timer.due;
    __runFakeTimer(timer);
  }
  __fakeTimers.now = target;
}

async function __advanceFakeTimersByAsync(ms) {
  __requireFakeTimers("jest.advanceTimersByTimeAsync");
  const target = __fakeTimers.now + __timerDelay(ms);
  let timer;
  while ((timer = __nextFakeTimer(target)) != null) {
    __fakeTimers.now = timer.due;
    // Unlike the synchronous API, preserve a callback's async completion so
    // callers do not observe its promise as an unhandled background task.
    await __runFakeTimer(timer);
  }
  __fakeTimers.now = target;
}

function __setFakeSystemTime(value) {
  __requireFakeTimers("jest.setSystemTime");
  const now = value === undefined
    ? __realNow()
    : value instanceof __nativeDate
      ? value.getTime()
      : Number(value);
  if (!Number.isFinite(now)) {
    throw new TypeError("jest.setSystemTime expects a valid Date or timestamp");
  }
  __fakeTimers.now = now;
}

function __runAllFakeTimers() {
  __requireFakeTimers("jest.runAllTimers");
  let runs = 0;
  let timer;
  while ((timer = __nextFakeTimer()) != null) {
    if (runs++ >= 100_000) {
      throw new Error("jest.runAllTimers exceeded 100000 scheduled callbacks");
    }
    __fakeTimers.now = timer.due;
    __runFakeTimer(timer);
  }
}

function __runOnlyPendingFakeTimers() {
  __requireFakeTimers("jest.runOnlyPendingTimers");
  const pending = [...__fakeTimers.timers.values()].sort(
    (left, right) => left.due - right.due || left.id - right.id,
  );
  for (const timer of pending) {
    if (!__fakeTimers.timers.has(timer.id)) continue;
    __fakeTimers.now = timer.due;
    __runFakeTimer(timer);
  }
}

function __installFakeTimers() {
  __fakeTimers.enabled = true;
  __fakeTimers.now = __realNow();
  __fakeTimers.nextId = 1;
  __fakeTimers.timers.clear();

  globalThis.setTimeout = (callback, delay, ...args) =>
    __scheduleFakeTimer(callback, delay, args);
  globalThis.clearTimeout = (id) =>
    __clearFakeTimer(id, __nativeTimers.clearTimeout);
  globalThis.setInterval = (callback, delay, ...args) =>
    __scheduleFakeTimer(callback, delay, args, Math.max(1, __timerDelay(delay)));
  globalThis.clearInterval = (id) =>
    __clearFakeTimer(id, __nativeTimers.clearInterval);
  if (__nativeTimers.setImmediate) {
    globalThis.setImmediate = (callback, ...args) =>
      __scheduleFakeTimer(callback, 0, args);
  }
  if (__nativeTimers.clearImmediate) {
    globalThis.clearImmediate = (id) =>
      __clearFakeTimer(id, __nativeTimers.clearImmediate);
  }
  globalThis.Date = __fakeDate;
}

function __restoreRealTimers() {
  if (!__fakeTimers.enabled) return;
  globalThis.setTimeout = __nativeTimers.setTimeout;
  globalThis.clearTimeout = __nativeTimers.clearTimeout;
  globalThis.setInterval = __nativeTimers.setInterval;
  globalThis.clearInterval = __nativeTimers.clearInterval;
  if (__nativeTimers.setImmediate) globalThis.setImmediate = __nativeTimers.setImmediate;
  if (__nativeTimers.clearImmediate) globalThis.clearImmediate = __nativeTimers.clearImmediate;
  globalThis.Date = __nativeDate;
  __fakeTimers.enabled = false;
  __fakeTimers.now = 0;
  __fakeTimers.timers.clear();
}

function makeJestMock(implementation) {
  let defaultImplementation =
    typeof implementation === "function" ? implementation : () => undefined;
  const onceImplementations = [];

  const mock = function (...args) {
    const impl = onceImplementations.length > 0 ? onceImplementations.shift() : defaultImplementation;
    mock.mock.calls.push(args);
    mock.mock.contexts.push(this);
    mock.mock.instances.push(new.target ? this : undefined);
    try {
      const value = impl.apply(this, args);
      mock.mock.results.push({ type: "return", value });
      return value;
    } catch (value) {
      mock.mock.results.push({ type: "throw", value });
      throw value;
    }
  };

  mock.mock = { calls: [], contexts: [], instances: [], results: [] };
  mock._isMockFunction = true;
  mock.mockImplementation = (...args) => {
    if (args.length === 0) {
      defaultImplementation = () => undefined;
      return mock;
    }
    const [next] = args;
    if (typeof next !== "function") throw new TypeError("jest.fn mockImplementation expects a function");
    defaultImplementation = next;
    return mock;
  };
  mock.mockImplementationOnce = (next) => {
    if (typeof next !== "function") throw new TypeError("jest.fn mockImplementationOnce expects a function");
    onceImplementations.push(next);
    return mock;
  };
  mock.mockReturnValue = (value) => mock.mockImplementation(() => value);
  mock.mockReturnValueOnce = (value) => mock.mockImplementationOnce(() => value);
  mock.mockResolvedValue = (value) => mock.mockImplementation(() => Promise.resolve(value));
  mock.mockResolvedValueOnce = (value) => mock.mockImplementationOnce(() => Promise.resolve(value));
  mock.mockRejectedValue = (value) => mock.mockImplementation(() => Promise.reject(value));
  mock.mockRejectedValueOnce = (value) => mock.mockImplementationOnce(() => Promise.reject(value));
  mock.mockClear = () => {
    mock.mock.calls.length = 0;
    mock.mock.contexts.length = 0;
    mock.mock.instances.length = 0;
    mock.mock.results.length = 0;
    return mock;
  };
  mock.mockReset = () => {
    mock.mockClear();
    onceImplementations.length = 0;
    defaultImplementation = () => undefined;
    return mock;
  };

  __jestMockFunctions.add(mock);
  return mock;
}

const jest = {
  fn: makeJestMock,
  isMockFunction(value) {
    return Boolean(value && value._isMockFunction === true);
  },
  spyOn(object, property) {
    if (object == null || typeof object[property] !== "function") {
      throw new TypeError("jest.spyOn expects an existing function property");
    }
    const original = object[property];
    const ownDescriptor = Object.getOwnPropertyDescriptor(object, property);
    const mock = makeJestMock(function (...args) {
      return original.apply(this, args);
    });
    let restored = false;
    mock.mockRestore = () => {
      if (restored) return mock;
      restored = true;
      mock.mockReset();
      if (ownDescriptor) {
        Object.defineProperty(object, property, ownDescriptor);
      } else {
        delete object[property];
      }
      __jestSpies.delete(mock);
      return mock;
    };
    try {
      object[property] = mock;
    } catch (err) {
      throw new TypeError(`jest.spyOn could not replace ${String(property)}: ${err?.message ?? err}`);
    }
    if (object[property] !== mock) {
      throw new TypeError(`jest.spyOn could not replace ${String(property)}`);
    }
    __jestSpies.add(mock);
    return mock;
  },
  // Factories are retained for explicit `jest.requireMock()` consumers. ESM
  // static-import interception is intentionally not implied by this registry.
  mock(moduleName, factory) {
    if (typeof moduleName !== "string") {
      throw new TypeError("jest.mock expects a module name string");
    }
    if (factory !== undefined && typeof factory !== "function") {
      throw new TypeError("jest.mock factory must be a function");
    }
    __jestModuleMocks.set(moduleName, factory ? factory() : {});
    return jest;
  },
  unmock(moduleName) {
    __jestModuleMocks.delete(moduleName);
    return jest;
  },
  requireMock(moduleName) {
    if (!__jestModuleMocks.has(moduleName)) {
      throw new Error(`jest.requireMock: no mock registered for ${moduleName}`);
    }
    return __jestModuleMocks.get(moduleName);
  },
  requireActual(moduleName) {
    if (typeof moduleName !== "string") {
      throw new TypeError("jest.requireActual expects a module name string");
    }
    const nodeRequire = __getJestNodeRequire();
    try {
      const resolved = nodeRequire.resolve(moduleName);
      const actual = nodeRequire(moduleName);
      __trackJestRequiredModuleTree(resolved);
      return actual;
    } catch (err) {
      const esmHint = err?.code === "ERR_REQUIRE_ESM"
        ? " Jet's ESM runner cannot synchronously require an ESM-only module; use `await import(...)` for that module."
        : "";
      throw new Error(
        `jest.requireActual(${JSON.stringify(moduleName)}) failed: ${err?.message ?? err}.${esmHint}`,
      );
    }
  },
  // Jest's `mocked()` is a TypeScript narrowing helper; its runtime contract
  // is identity, so callers keep the real mock object and its call history.
  mocked(value) {
    return value;
  },
  clearAllMocks() {
    __jestMockFunctions.forEach((mock) => mock.mockClear());
  },
  resetAllMocks() {
    __jestMockFunctions.forEach((mock) => mock.mockReset());
  },
  restoreAllMocks() {
    for (const mock of [...__jestSpies]) {
      mock.mockRestore();
    }
  },
  // Native ESM static imports are instantiated before test code runs and
  // cannot be re-evaluated without a loader-level cache bust. This resets the
  // CJS modules loaded through requireActual(), which is the registry this
  // runtime owns, while intentionally preserving explicit jest.mock factories.
  resetModules() {
    __resetJestModuleRegistry();
    return jest;
  },
  useFakeTimers() {
    __installFakeTimers();
    return jest;
  },
  useRealTimers() {
    __restoreRealTimers();
    return jest;
  },
  clearAllTimers() {
    if (__fakeTimers.enabled) {
      __fakeTimers.timers.clear();
    }
  },
  getTimerCount() {
    return __fakeTimers.enabled ? __fakeTimers.timers.size : 0;
  },
  advanceTimersByTime(ms) {
    __advanceFakeTimersBy(ms);
  },
  advanceTimersByTimeAsync(ms) {
    return __advanceFakeTimersByAsync(ms);
  },
  setSystemTime(value) {
    __setFakeSystemTime(value);
  },
  runAllTimers() {
    __runAllFakeTimers();
  },
  runOnlyPendingTimers() {
    __runOnlyPendingFakeTimers();
  },
};

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

const __expectCustomMatchers = new Map();

function expect(actual) {
  const obj = __expectBase(actual);
  // @spec #2605 — `expect(x).not.toBe(y)` and friends. Wraps every
  // function-valued matcher so a thrown AssertionError becomes a pass and
  // a clean return becomes a thrown negated AssertionError.
  obj.not = __negate(obj, actual);
  return obj;
}

// Jest-compatible extension point for project-local matchers. Matchers return
// `{ pass, message }` (or a Promise of that shape); an assertion failure stays
// a Jet AssertionError so normal reporting and `.not` semantics still work.
expect.extend = (matchers) => {
  if (matchers == null || typeof matchers !== "object") {
    throw new TypeError("expect.extend expects an object of matcher functions");
  }
  for (const [name, matcher] of Object.entries(matchers)) {
    if (typeof matcher !== "function") {
      throw new TypeError(`expect.extend matcher ${name} must be a function`);
    }
    __expectCustomMatchers.set(name, matcher);
  }
};

// Asymmetric matchers consumed by deepEqual below, so these work in nested
// `toEqual` structures exactly where Jest callers use them.
expect.stringContaining = (expected) => {
  if (typeof expected !== "string") {
    throw new TypeError("expect.stringContaining expects a string");
  }
  return Object.freeze({
    asymmetricMatch(value) {
      return typeof value === "string" && value.includes(expected);
    },
    toString() {
      return "StringContaining";
    },
    toAsymmetricMatcher() {
      return `StringContaining ${JSON.stringify(expected)}`;
    },
  });
};

expect.stringMatching = (expected) => {
  if (!(expected instanceof RegExp) && typeof expected !== "string") {
    throw new TypeError("expect.stringMatching expects a string or RegExp");
  }
  const matcher = expected instanceof RegExp ? expected : new RegExp(expected);
  return Object.freeze({
    asymmetricMatch(value) {
      if (typeof value !== "string") return false;
      // Global and sticky regexes retain lastIndex across `test()` calls.
      // Reset it so an asymmetric matcher stays deterministic in deep equals.
      matcher.lastIndex = 0;
      const matched = matcher.test(value);
      matcher.lastIndex = 0;
      return matched;
    },
    toString() {
      return "StringMatching";
    },
    toAsymmetricMatcher() {
      return `StringMatching ${matcher}`;
    },
  });
};

expect.any = (expected) => {
  if (typeof expected !== "function") {
    throw new TypeError("expect.any expects a constructor");
  }
  return Object.freeze({
    asymmetricMatch(value) {
      if (expected === String) return typeof value === "string" || value instanceof String;
      if (expected === Number) return typeof value === "number" || value instanceof Number;
      if (expected === Boolean) return typeof value === "boolean" || value instanceof Boolean;
      if (expected === Function) return typeof value === "function";
      if (expected === Object) return value != null && (typeof value === "object" || typeof value === "function");
      if (expected === Array) return Array.isArray(value);
      if (expected === BigInt) return typeof value === "bigint";
      if (expected === Symbol) return typeof value === "symbol";
      return value instanceof expected;
    },
    toString() {
      return "Any";
    },
    toAsymmetricMatcher() {
      return `Any<${expected.name || "anonymous"}>`;
    },
  });
};

expect.objectContaining = (expected) => {
  if (expected == null || typeof expected !== "object") {
    throw new TypeError("expect.objectContaining expects an object");
  }
  const entries = Object.entries(expected);
  return Object.freeze({
    asymmetricMatch(value) {
      if (value == null || typeof value !== "object") return false;
      return entries.every(
        ([key, expectedValue]) =>
          Object.prototype.hasOwnProperty.call(value, key) &&
          deepEqual(value[key], expectedValue),
      );
    },
    toString() {
      return "ObjectContaining";
    },
    toAsymmetricMatcher() {
      return `ObjectContaining ${display(expected)}`;
    },
  });
};

function __customMatcherContext(isNot) {
  return {
    isNot,
    equals: deepEqual,
    utils: {
      printExpected: display,
      printReceived: display,
      matcherHint(name) {
        return `expect${isNot ? ".not" : ""}.${name}`;
      },
    },
  };
}

function __customMatcherFailure(name, result, isNot, actual) {
  let message;
  try {
    message = typeof result?.message === "function"
      ? result.message()
      : result?.message;
  } catch (err) {
    message = `expect.${name} failed while building its message: ${err?.message ?? err}`;
  }
  return new AssertionError(
    typeof message === "string" && message.length > 0
      ? message
      : `Expected ${display(actual)} ${isNot ? "not " : ""}to satisfy custom matcher ${name}`,
  );
}

function __runCustomMatcher(name, matcher, actual, args, isNot) {
  const verify = (result) => {
    if (result == null || typeof result.pass !== "boolean") {
      throw new TypeError(
        `expect.extend matcher ${name} must return { pass: boolean, message?: string | function }`,
      );
    }
    if (isNot ? result.pass : !result.pass) {
      throw __customMatcherFailure(name, result, isNot, actual);
    }
  };
  const result = matcher.call(__customMatcherContext(isNot), actual, ...args);
  return result && typeof result.then === "function" ? result.then(verify) : verify(result);
}

function __negate(obj, actual) {
  const negated = {};
  for (const [name, fn] of Object.entries(obj)) {
    if (typeof fn !== "function") continue;
    const customMatcher = __expectCustomMatchers.get(name);
    if (customMatcher) {
      negated[name] = (...args) =>
        __runCustomMatcher(name, customMatcher, actual, args, true);
      continue;
    }
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
  const matchers = {
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
      const timeout = options.timeout ?? DEFAULT_ASSERTION_TIMEOUT_MS;
      const start = __realNow();
      let lastVisible = null;
      let lastError = null;
      while (true) {
        try {
          const res = await __sendRequest({
            kind: "is_visible",
            page_id: pageId,
            selector: selectorOrOpts,
          });
          lastVisible = res.visible;
          if (res.visible) return;
        } catch (err) {
          lastError = err;
        }
        if (__realNow() - start >= timeout) {
          const msg = lastError
            ? `toBeVisible(${JSON.stringify(selectorOrOpts)}): ${lastError.message ?? String(lastError)}`
            : `Expected ${selectorOrOpts} to be visible within ${timeout}ms, got ${JSON.stringify(lastVisible)}`;
          throw new AssertionError(
            msg,
            `- expected: true\n+ actual:   ${JSON.stringify(lastVisible)}`,
          );
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
      const timeout = options.timeout ?? DEFAULT_ASSERTION_TIMEOUT_MS;
      const start = __realNow();
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
        if (__realNow() - start >= timeout) {
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

    async toContainText(expected, opts) {
      if (!(actual instanceof Locator)) {
        throw new Error("toContainText: expected a Locator object");
      }
      return toContainTextLocator(actual, expected, opts);
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

  for (const [name, matcher] of __expectCustomMatchers) {
    matchers[name] = (...args) =>
      __runCustomMatcher(name, matcher, actual, args, false);
  }
  return matchers;
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
  if (b && typeof b.asymmetricMatch === "function") {
    return Boolean(b.asymmetricMatch(a));
  }
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
  globalThis.jest = jest;
  globalThis.beforeAll = beforeAll;
  globalThis.afterAll = afterAll;
  globalThis.beforeEach = beforeEach;
  globalThis.afterEach = afterEach;
  // Thread the resolved assertion poll timeout (RunnerConfig.expect_timeout_ms
  // — CLI --expect-timeout or jet.toml, default 5000ms) into matchers.js so
  // every locator/page matcher's default (absent a per-call opts.timeout)
  // reflects the run's configured value. #1908
  setDefaultAssertionTimeout(opts && opts.jetConfig && opts.jetConfig.expectTimeoutMs);
  // Anchor `jest.requireActual("./relative")` at the source spec path even
  // though ESM source is emitted into the worker's temporary module graph.
  __setJestRequireForSpec(opts.file);
  globalThis.require = __getJestNodeRequire();

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

    const started = __realNow();
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
      __jet.currentTestId = id;
      __jet.currentStepSeq = 0;
      __jet.currentStepStack = [];
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
        __jet.currentTestId = null;
        __jet.currentStepSeq = 0;
        __jet.currentStepStack = [];
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
      duration_ms: __realNow() - started,
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
    handle = __nativeTimers.setTimeout(() => {
      const e = new Error("timeout");
      e.__jet_timeout = true;
      reject(e);
    }, ms);
  });
  return Promise.race([Promise.resolve(promise), timeout]).finally(() =>
    __nativeTimers.clearTimeout(handle),
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
  jest,
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
  "jest",
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
      `See apps/jet/data/runtime/test/CONTRACT.md.`
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
