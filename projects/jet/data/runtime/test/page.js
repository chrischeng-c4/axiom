// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-data-runtime-test.md#logic
// CODEGEN-BEGIN
// @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R2
// @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
//
// JS-side Page proxy for the Playwright-compatible page fixture.
//
// Instances of this class are injected into test bodies and beforeEach callbacks
// that destructure `page` from the fixture argument. Each Page wraps a CDP
// target (identified by `pageId`) and a request counter (`reqCounter`) shared
// with the fixture registry in index.js. Methods issue PageRequest NDJSON over
// stdout and return a Promise that resolves when the matching PageResponse
// arrives over stdin.
//
// The Rust worker (`test_runner/worker.rs`) dispatches each PageRequest to the
// active `browser::Page` via `cdp_driver::dispatch_page_request`.
//
// baseURL resolution:
//   page.goto(url) checks whether `url` starts with "/" or is a path-only
//   string (no "://"). If so, `baseURL` (set on construction) is prepended.
//   This happens entirely on the JS side so the Rust layer always receives an
//   absolute URL.
//
// Lifecycle:
//   - Constructed by the fixture registry at the start of each test.
//   - page.close() is called automatically in the fixture finally block.
//   - After close(), all method calls throw with a clear message.
//   - Event listeners registered via page.on() are cleaned up on page.close().

// @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
export class Page {
  // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R2
  constructor(pageId, sendRequest, baseURL) {
    this.__jet_page_id = pageId;
    this._send = sendRequest; // async (req) => response
    this._baseURL = baseURL || "";
    this._closed = false;
    // Event listener map: keyed by event name, value is array of handlers.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R5
    this._eventListeners = {};
    // Lazy-initialized keyboard and mouse accessor objects.
    this._keyboard = null;
    this._mouse = null;
  }

  _assertOpen() {
    if (this._closed) {
      throw new Error(
        `page.${arguments.callee?.name ?? "method"} called on a closed page — did the test leak a page reference beyond its scope?`
      );
    }
  }

  // ── Navigation ──────────────────────────────────────────────────────────────

  // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R3
  async goto(url) {
    this._assertOpen();
    // baseURL resolution: prepend base for relative paths.
    const resolved = this._resolveUrl(url);
    const res = await this._send({
      kind: "goto",
      page_id: this.__jet_page_id,
      url: resolved,
    });
    if (res.kind === "error") {
      throw new Error(res.message);
    }
  }

  // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R3
  _resolveUrl(url) {
    if (!url) return url;
    // Relative URL: starts with "/" or is scheme-less (no "://").
    const isRelative = url.startsWith("/") || !/[a-zA-Z][a-zA-Z0-9+\-.]*:\/\//.test(url);
    if (isRelative && this._baseURL) {
      return this._baseURL.replace(/\/$/, "") + (url.startsWith("/") ? url : "/" + url);
    }
    return url;
  }

  // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R6
  async goBack() {
    this._assertOpen();
    const res = await this._send({ kind: "go_back", page_id: this.__jet_page_id });
    if (res.kind === "error") throw new Error(res.message);
  }

  // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R6
  async goForward() {
    this._assertOpen();
    const res = await this._send({ kind: "go_forward", page_id: this.__jet_page_id });
    if (res.kind === "error") throw new Error(res.message);
  }

  // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R6
  async reload() {
    this._assertOpen();
    const res = await this._send({ kind: "reload", page_id: this.__jet_page_id });
    if (res.kind === "error") throw new Error(res.message);
  }

  // ── Queries ─────────────────────────────────────────────────────────────────

  // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
  async url() {
    this._assertOpen();
    const res = await this._send({ kind: "url", page_id: this.__jet_page_id });
    if (res.kind === "error") throw new Error(res.message);
    return res.value;
  }

  // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R1
  async title() {
    this._assertOpen();
    const res = await this._send({ kind: "title", page_id: this.__jet_page_id });
    if (res.kind === "error") throw new Error(res.message);
    return res.value;
  }

  // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
  async evaluate(expression) {
    this._assertOpen();
    const res = await this._send({
      kind: "evaluate",
      page_id: this.__jet_page_id,
      expression: typeof expression === "function" ? `(${expression.toString()})()` : expression,
    });
    if (res.kind === "error") throw new Error(res.message);
    return res.value;
  }

  // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R10
  async content() {
    this._assertOpen();
    const res = await this._send({ kind: "content", page_id: this.__jet_page_id });
    if (res.kind === "error") throw new Error(res.message);
    return res.value;
  }

  // ── Viewport / timing ───────────────────────────────────────────────────────

  // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R2
  async setViewportSize({ width, height }) {
    this._assertOpen();
    const res = await this._send({
      kind: "set_viewport_size",
      page_id: this.__jet_page_id,
      width,
      height,
    });
    if (res.kind === "error") throw new Error(res.message);
  }

  // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R3
  async waitForTimeout(ms) {
    // Pure-JS timeout — no CDP call needed.
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  // ── Screenshot ───────────────────────────────────────────────────────────────

  // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R4
  async screenshot(opts) {
    this._assertOpen();
    const res = await this._send({
      kind: "screenshot",
      page_id: this.__jet_page_id,
      path: (opts && opts.path) || null,
    });
    if (res.kind === "error") throw new Error(res.message);
    // res.data is base64-encoded PNG. Convert to Buffer.
    const buf = Buffer.from(res.data, "base64");
    // Playwright-style: if { path } is supplied, persist to disk from JS.
    // The Rust side currently passes `path` through but doesn't write the
    // file itself — see page_binding::PageRequest::Screenshot doc.
    // @spec .aw/tech-design/projects/jet/logic/auto-artifacts.md#A4
    if (opts && opts.path) {
      const fs = await import("node:fs/promises");
      const path = await import("node:path");
      await fs.mkdir(path.dirname(opts.path), { recursive: true });
      await fs.writeFile(opts.path, buf);
    }
    return buf;
  }

  // ── Content ──────────────────────────────────────────────────────────────────

  // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R9
  async setContent(html) {
    this._assertOpen();
    const res = await this._send({
      kind: "set_content",
      page_id: this.__jet_page_id,
      html,
    });
    if (res.kind === "error") throw new Error(res.message);
  }

  // ── Debug pause (P4.2 MVP) ───────────────────────────────────────────────
  //
  // `await page.pause()` blocks the test for up to 30 minutes or until the
  // test-level timeout cuts the run — whichever is shorter. MVP: there is
  // no interactive inspector UI; the dev is expected to already be in
  // --debug mode (headed + workers=1) so they can poke at the page by
  // hand in the Chrome window that stayed open.
  //
  // @spec .aw/tech-design/projects/jet/logic/inspector.md#I2
  async pause() {
    this._assertOpen();
    // 30 minutes — long enough for manual inspection, short enough that a
    // CI that accidentally hits this doesn't hang forever.
    await new Promise((resolve) => setTimeout(resolve, 30 * 60 * 1000));
  }

  // ── Route interception (P3.3) ────────────────────────────────────────────
  // MVP: intercept `fetch()` and `XMLHttpRequest` in the page context via a
  // JS override installed with evaluate(). The override consults
  // `window.__jetRoutes` so subsequent route() calls just mutate the list
  // without reinstalling. The installer is idempotent (gated by a flag).
  //
  // Limitations vs CDP Fetch.enable (deferred):
  // - Navigation + resource loads (images, CSS) are NOT intercepted — only
  //   fetch/XHR made by page JS.
  // - Routes are scoped to the current document; a subsequent goto()/
  //   setContent() wipes window.__jetRoutes. Call page.route() AFTER nav.
  //
  // @spec .aw/tech-design/projects/jet/logic/route-intercept.md#R1 R2 R3

  async route(urlPattern, config) {
    this._assertOpen();
    if (!config || typeof config !== "object") {
      throw new Error("page.route: config must be an object");
    }
    const isRegex = urlPattern instanceof RegExp;
    const patternJson = JSON.stringify(
      isRegex
        ? { kind: "regex", source: urlPattern.source, flags: urlPattern.flags }
        : { kind: "glob", value: String(urlPattern) },
    );
    const configJson = JSON.stringify({
      status: config.status ?? 200,
      body: config.body ?? "",
      headers: config.headers ?? {},
      contentType: config.contentType ?? null,
      abort: Boolean(config.abort),
    });
    const expr = `(function(){
      if (!window.__jetRoutesInstalled) { ${_jetRouteInstallerSrc()}; window.__jetRoutesInstalled = true; }
      window.__jetRoutes.push({ pattern: ${patternJson}, config: ${configJson} });
    })()`;
    const res = await this._send({
      kind: "evaluate",
      page_id: this.__jet_page_id,
      expression: expr,
    });
    if (res.kind === "error") throw new Error(`page.route: ${res.message}`);
  }

  async unroute(urlPattern) {
    this._assertOpen();
    const isRegex = urlPattern instanceof RegExp;
    const patternJson = JSON.stringify(
      isRegex
        ? { kind: "regex", source: urlPattern.source, flags: urlPattern.flags }
        : { kind: "glob", value: String(urlPattern) },
    );
    const expr = `(function(){
      if (!window.__jetRoutes) return 0;
      var before = window.__jetRoutes.length;
      var target = ${patternJson};
      window.__jetRoutes = window.__jetRoutes.filter(function(r){
        if (r.pattern.kind !== target.kind) return true;
        if (target.kind === 'regex') return !(r.pattern.source === target.source && r.pattern.flags === target.flags);
        return r.pattern.value !== target.value;
      });
      return before - window.__jetRoutes.length;
    })()`;
    const res = await this._send({
      kind: "evaluate",
      page_id: this.__jet_page_id,
      expression: expr,
    });
    if (res.kind === "error") throw new Error(`page.unroute: ${res.message}`);
    return typeof res.value === "number" ? res.value : 0;
  }

  async unrouteAll() {
    this._assertOpen();
    const expr = `(function(){ if (window.__jetRoutes) { var n = window.__jetRoutes.length; window.__jetRoutes = []; return n; } return 0; })()`;
    const res = await this._send({
      kind: "evaluate",
      page_id: this.__jet_page_id,
      expression: expr,
    });
    if (res.kind === "error") throw new Error(`page.unrouteAll: ${res.message}`);
    return typeof res.value === "number" ? res.value : 0;
  }

  // ── Event subscriptions ──────────────────────────────────────────────────────
  // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R5

  on(event, handler) {
    this._assertOpen();
    if (!this._eventListeners[event]) {
      this._eventListeners[event] = [];
    }
    this._eventListeners[event].push(handler);
    // The event dispatch mechanism: events from Rust arrive as PageResponse
    // messages with kind="event" on stdin. The index.js stdin reader routes
    // them to the page's event listeners via _dispatchEvent().
    // We register the interest with Rust so it forwards relevant CDP events.
    this._send({
      kind: "subscribe_event",
      page_id: this.__jet_page_id,
      event_name: event,
    }).catch(() => {
      // Subscription registration is best-effort; ignore errors.
    });
  }

  // Internal: dispatch an event to registered listeners.
  _dispatchEvent(event, payload) {
    const handlers = this._eventListeners[event] || [];
    for (const h of handlers) {
      try {
        h(payload);
      } catch {
        // Suppress handler errors — don't let them crash the test runner.
      }
    }
  }

  // ── Keyboard ─────────────────────────────────────────────────────────────────
  // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R7

  get keyboard() {
    if (!this._keyboard) {
      const send = this._send.bind(this);
      const pageId = this.__jet_page_id;
      this._keyboard = {
        async press(key) {
          const res = await send({ kind: "keyboard_press", page_id: pageId, key });
          if (res.kind === "error") throw new Error(res.message);
        },
        async type(text) {
          const res = await send({ kind: "keyboard_type", page_id: pageId, text });
          if (res.kind === "error") throw new Error(res.message);
        },
      };
    }
    return this._keyboard;
  }

  // ── Mouse ─────────────────────────────────────────────────────────────────────
  // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R8

  get mouse() {
    if (!this._mouse) {
      const send = this._send.bind(this);
      const pageId = this.__jet_page_id;
      this._mouse = {
        async click(x, y, opts) {
          const button = (opts && opts.button) || "left";
          const clickCount = (opts && opts.clickCount) || 1;
          // mouseMoved → mousePressed → mouseReleased
          const move = await send({
            kind: "mouse_event",
            page_id: pageId,
            event_type: "mouseMoved",
            x,
            y,
            button: null,
            click_count: null,
          });
          if (move.kind === "error") throw new Error(move.message);
          const press = await send({
            kind: "mouse_event",
            page_id: pageId,
            event_type: "mousePressed",
            x,
            y,
            button,
            click_count: clickCount,
          });
          if (press.kind === "error") throw new Error(press.message);
          const release = await send({
            kind: "mouse_event",
            page_id: pageId,
            event_type: "mouseReleased",
            x,
            y,
            button,
            click_count: clickCount,
          });
          if (release.kind === "error") throw new Error(release.message);
        },
        async move(x, y) {
          const res = await send({
            kind: "mouse_event",
            page_id: pageId,
            event_type: "mouseMoved",
            x,
            y,
            button: null,
            click_count: null,
          });
          if (res.kind === "error") throw new Error(res.message);
        },
        async down(opts) {
          const button = (opts && opts.button) || "left";
          const res = await send({
            kind: "mouse_event",
            page_id: pageId,
            event_type: "mousePressed",
            x: 0,
            y: 0,
            button,
            click_count: 1,
          });
          if (res.kind === "error") throw new Error(res.message);
        },
        async up(opts) {
          const button = (opts && opts.button) || "left";
          const res = await send({
            kind: "mouse_event",
            page_id: pageId,
            event_type: "mouseReleased",
            x: 0,
            y: 0,
            button,
            click_count: 1,
          });
          if (res.kind === "error") throw new Error(res.message);
        },
      };
    }
    return this._mouse;
  }

  // ── Direct element actions ───────────────────────────────────────────────────

  // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
  async click(selector) {
    this._assertOpen();
    const res = await this._send({
      kind: "click",
      page_id: this.__jet_page_id,
      selector,
    });
    if (res.kind === "error") throw new Error(res.message);
  }

  // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
  async fill(selector, value) {
    this._assertOpen();
    const res = await this._send({
      kind: "fill",
      page_id: this.__jet_page_id,
      selector,
      value,
    });
    if (res.kind === "error") throw new Error(res.message);
  }

  // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
  async waitForSelector(selector, opts) {
    this._assertOpen();
    const res = await this._send({
      kind: "wait_for_selector",
      page_id: this.__jet_page_id,
      selector,
      timeout_ms: opts && opts.timeout != null ? opts.timeout : null,
    });
    if (res.kind === "error") throw new Error(res.message);
  }

  // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
  async waitForLoadState(state) {
    this._assertOpen();
    const res = await this._send({
      kind: "wait_for_load_state",
      page_id: this.__jet_page_id,
      state: state || null,
    });
    if (res.kind === "error") throw new Error(res.message);
  }

  // ── Locator factory ──────────────────────────────────────────────────────────

  // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
  locator(selector) {
    this._assertOpen();
    return new Locator(selector, this._send, this.__jet_page_id);
  }

  // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
  getByText(text) {
    this._assertOpen();
    return new Locator("text=" + text, this._send, this.__jet_page_id);
  }

  // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R6
  getByRole(role, options) {
    this._assertOpen();
    let selector = "role=" + role;
    if (options && options.name) {
      selector += '[name="' + options.name + '"]';
    }
    return new Locator(selector, this._send, this.__jet_page_id);
  }

  // ── Lifecycle ────────────────────────────────────────────────────────────────

  // @spec .aw/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md#R4
  async close() {
    if (this._closed) return;
    this._closed = true;
    // Clean up all registered event listeners by sending remove_event_listener
    // to the Rust side so CDP subscriptions are released.
    // @spec .aw/changes/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti/specs/enhancement-page-api-parity-with-playwright-fill-gaps-in-runti-spec.md#R5
    const eventNames = Object.keys(this._eventListeners);
    for (const eventName of eventNames) {
      try {
        await this._send({
          kind: "remove_event_listener",
          page_id: this.__jet_page_id,
          event_name: eventName,
        });
      } catch {
        // Suppress errors during cleanup.
      }
    }
    this._eventListeners = {};
    try {
      await this._send({ kind: "close", page_id: this.__jet_page_id });
    } catch {
      // Suppress close errors — page may already be gone.
    }
  }
}

// ── Locator proxy ─────────────────────────────────────────────────────────────
//
// Returned by page.locator(selector), page.getByText(text), page.getByRole(role),
// and chained locator.locator(child) / .filter({hasText}) / .nth(i).
//
// Playwright-parity features added by locator-js-api:
//   - Sub-locator chaining with CSS concatenation (pure CSS) or evaluate-scope
//     (when a pseudo-selector — role= / text= — appears anywhere in the chain).
//   - filter({ hasText, hasNotText }) with string / RegExp predicates.
//   - Auto-wait FSM (Attached → Visible → Stable) before click/fill/hover/
//     press/selectOption/check/uncheck. Default 5000ms, override via
//     `opts.timeout`.
//   - NthLocator actions route through _taggedSelectorScope so
//     click/fill/hover/press target the indexed element via real CDP input
//     events (not match[0]).
//
// @spec .aw/tech-design/projects/jet/logic/locator-js-api.md
export class Locator {
  constructor(selector, sendRequest, pageId, options) {
    this._selector = selector;
    this._send = sendRequest;
    this._pageId = pageId;
    // Default per L4 / L10 — override via options.timeout or per-call opts.timeout.
    this._timeout = options && options.timeout != null ? options.timeout : 5000;
    // Each filter is one of:
    //   { kind: "scope",  child: string }
    //   { kind: "text",   hasText?: string|RegExp, hasNotText?: string|RegExp }
    this._filters = options && options.filters ? options.filters.slice() : [];
  }

  // ── Composition ───────────────────────────────────────────────────────────

  // Sub-locator under this scope. Pure CSS on both sides → CSS concat;
  // otherwise push a "scope" filter and resolve client-side at action time.
  // @spec locator-js-api#L1 L2
  locator(childSel, opts) {
    const timeout = opts && opts.timeout != null ? opts.timeout : this._timeout;
    if (
      this._filters.length === 0 &&
      !_isPseudoSelector(this._selector) &&
      !_isPseudoSelector(childSel)
    ) {
      const combined = this._selector + " " + childSel;
      return new Locator(combined, this._send, this._pageId, { timeout });
    }
    const filters = this._filters.concat({ kind: "scope", child: childSel });
    return new Locator(this._selector, this._send, this._pageId, { timeout, filters });
  }

  // @spec locator-js-api#L3
  filter(opts) {
    const filters = this._filters.concat({
      kind: "text",
      hasText: opts && opts.hasText,
      hasNotText: opts && opts.hasNotText,
    });
    return new Locator(this._selector, this._send, this._pageId, {
      timeout: this._timeout,
      filters,
    });
  }

  // @spec locator-js-api#L8
  getByRole(role, options) {
    let child = "role=" + role;
    if (options && options.name) child += '[name="' + options.name + '"]';
    return this.locator(child);
  }

  getByText(text) {
    return this.locator("text=" + text);
  }

  // @spec locator-js-api#L6 L9
  nth(index) {
    return new NthLocator(this._selector, this._send, this._pageId, index, {
      timeout: this._timeout,
      filters: this._filters,
    });
  }

  first() { return this.nth(0); }
  last() { return this.nth(-1); }

  // ── Selector resolution ───────────────────────────────────────────────────

  // Returns true if the chain requires client-side resolution (filters set or
  // pseudo selector present). CDP-direct actions (click/fill/hover/press) can
  // only consume a raw CSS selector, so we must fall back to _taggedSelectorScope.
  _needsClientResolve() {
    return this._filters.length > 0 || _isPseudoSelector(this._selector);
  }

  // Build a JS expression that returns the first element in the resolved set,
  // or null. Honours scope/text filters.
  _resolveFirstExpr() {
    return `(${_collectMatchesSrc(this._selector, this._filters)})()[0] || null`;
  }

  // Build a JS expression that returns the resolved NodeList length.
  _resolveCountExpr() {
    return `(${_collectMatchesSrc(this._selector, this._filters)})().length`;
  }

  // ── Actionability / auto-wait ─────────────────────────────────────────────

  // Polls until the target is actionable. Poll interval 50ms, shared budget.
  // @spec locator-js-api#L4 L5
  async _waitForActionable(opts) {
    const timeout = opts && opts.timeout != null ? opts.timeout : this._timeout;
    const deadline = Date.now() + timeout;
    // Attached
    while (Date.now() < deadline) {
      const r = await this._send({
        kind: "evaluate",
        page_id: this._pageId,
        expression: `(function(){var el=${this._resolveFirstExpr()};return !!el;})()`,
      });
      if (r.kind !== "error" && r.value) break;
      await _sleep(50);
    }
    if (Date.now() >= deadline) {
      throw new Error(`locator ${this._selector}: timeout ${timeout}ms waiting for Attached`);
    }
    // Visible
    while (Date.now() < deadline) {
      const expr = `(function(){var el=${this._resolveFirstExpr()};${_visibilityCheckSrc}})()`;
      const r = await this._send({
        kind: "evaluate",
        page_id: this._pageId,
        expression: expr,
      });
      if (r.kind !== "error" && r.value) break;
      await _sleep(50);
    }
    if (Date.now() >= deadline) {
      throw new Error(`locator ${this._selector}: timeout ${timeout}ms waiting for Visible`);
    }
    // Stable — two consecutive equal rects, ≥50ms apart.
    let prev = null;
    while (Date.now() < deadline) {
      const expr = `(function(){var el=${this._resolveFirstExpr()};if(!el)return null;var r=el.getBoundingClientRect();return [r.x,r.y,r.width,r.height];})()`;
      const r = await this._send({
        kind: "evaluate",
        page_id: this._pageId,
        expression: expr,
      });
      const cur = r.kind !== "error" ? r.value : null;
      if (prev && cur && prev.length === 4 &&
          prev[0] === cur[0] && prev[1] === cur[1] &&
          prev[2] === cur[2] && prev[3] === cur[3]) {
        return;
      }
      prev = cur;
      await _sleep(50);
    }
    throw new Error(`locator ${this._selector}: timeout ${timeout}ms waiting for Stable`);
  }

  // Tag the resolved element with a unique data attribute, run fn with the
  // synthetic selector, then strip the attribute in `finally`. Used by
  // NthLocator and client-resolve paths to route CDP-direct actions.
  async _taggedSelectorScope(fn) {
    const token = _uuid();
    const tagExpr = `(function(){var el=${this._resolveFirstExpr()};if(!el)return false;el.setAttribute('data-__jet_nth__', ${JSON.stringify(token)});return true;})()`;
    const tagRes = await this._send({
      kind: "evaluate",
      page_id: this._pageId,
      expression: tagExpr,
    });
    if (tagRes.kind === "error" || !tagRes.value) {
      throw new Error(`locator ${this._selector}: could not resolve element to tag`);
    }
    const unique = `[data-__jet_nth__="${token}"]`;
    try {
      return await fn(unique);
    } finally {
      await this._send({
        kind: "evaluate",
        page_id: this._pageId,
        expression: `(function(){var el=document.querySelector(${JSON.stringify(unique)});if(el)el.removeAttribute('data-__jet_nth__');})()`,
      }).catch(() => {});
    }
  }

  // ── Actions (auto-waited) ─────────────────────────────────────────────────

  async click(opts) {
    await this._waitForActionable(opts);
    if (this._needsClientResolve()) {
      await this._taggedSelectorScope(async (sel) => {
        const res = await this._send({ kind: "click", page_id: this._pageId, selector: sel });
        if (res.kind === "error") throw new Error(res.message);
      });
      return;
    }
    const res = await this._send({
      kind: "click",
      page_id: this._pageId,
      selector: this._selector,
    });
    if (res.kind === "error") throw new Error(res.message);
  }

  async fill(value, opts) {
    await this._waitForActionable(opts);
    if (this._needsClientResolve()) {
      await this._taggedSelectorScope(async (sel) => {
        const res = await this._send({ kind: "fill", page_id: this._pageId, selector: sel, value });
        if (res.kind === "error") throw new Error(res.message);
      });
      return;
    }
    const res = await this._send({
      kind: "fill",
      page_id: this._pageId,
      selector: this._selector,
      value,
    });
    if (res.kind === "error") throw new Error(res.message);
  }

  async hover(opts) {
    await this._waitForActionable(opts);
    if (this._needsClientResolve()) {
      await this._taggedSelectorScope(async (sel) => {
        const res = await this._send({ kind: "hover", page_id: this._pageId, selector: sel });
        if (res.kind === "error") throw new Error(res.message);
      });
      return;
    }
    const res = await this._send({
      kind: "hover",
      page_id: this._pageId,
      selector: this._selector,
    });
    if (res.kind === "error") throw new Error(res.message);
  }

  async press(key, opts) {
    await this._waitForActionable(opts);
    if (this._needsClientResolve()) {
      await this._taggedSelectorScope(async (sel) => {
        const res = await this._send({ kind: "locator_press", page_id: this._pageId, selector: sel, key });
        if (res.kind === "error") throw new Error(res.message);
      });
      return;
    }
    const res = await this._send({
      kind: "locator_press",
      page_id: this._pageId,
      selector: this._selector,
      key,
    });
    if (res.kind === "error") throw new Error(res.message);
  }

  async selectOption(value, opts) {
    await this._waitForActionable(opts);
    const val = JSON.stringify(value);
    const expr = `(function() {
      var el = ${this._resolveFirstExpr()};
      if (!el) throw new Error('selectOption: element not found');
      el.value = ${val};
      el.dispatchEvent(new Event('change', { bubbles: true }));
      return el.value;
    })()`;
    const res = await this._send({
      kind: "evaluate",
      page_id: this._pageId,
      expression: expr,
    });
    if (res.kind === "error") throw new Error(res.message);
  }

  async check(opts) {
    await this._waitForActionable(opts);
    const expr = `(function(){var el=${this._resolveFirstExpr()};if(!el)throw new Error('check: element not found');if(!el.checked){el.checked=true;el.dispatchEvent(new Event('change',{bubbles:true}));}return el.checked;})()`;
    const res = await this._send({ kind: "evaluate", page_id: this._pageId, expression: expr });
    if (res.kind === "error") throw new Error(res.message);
  }

  async uncheck(opts) {
    await this._waitForActionable(opts);
    const expr = `(function(){var el=${this._resolveFirstExpr()};if(!el)throw new Error('uncheck: element not found');if(el.checked){el.checked=false;el.dispatchEvent(new Event('change',{bubbles:true}));}return el.checked;})()`;
    const res = await this._send({ kind: "evaluate", page_id: this._pageId, expression: expr });
    if (res.kind === "error") throw new Error(res.message);
  }

  // ── Reads (lightweight Attached-only wait not required by Playwright parity) ─

  async waitFor(opts) {
    // Honour client-resolve path by polling via evaluate.
    if (this._needsClientResolve()) {
      const timeout = opts && opts.timeout != null ? opts.timeout : this._timeout;
      const deadline = Date.now() + timeout;
      while (Date.now() < deadline) {
        const r = await this._send({
          kind: "evaluate",
          page_id: this._pageId,
          expression: `(function(){return !!${this._resolveFirstExpr()};})()`,
        });
        if (r.kind !== "error" && r.value) return;
        await _sleep(50);
      }
      throw new Error(`locator ${this._selector}: timeout ${timeout}ms waiting for Attached`);
    }
    const res = await this._send({
      kind: "wait_for_selector",
      page_id: this._pageId,
      selector: this._selector,
      timeout_ms: opts && opts.timeout != null ? opts.timeout : null,
    });
    if (res.kind === "error") throw new Error(res.message);
  }

  async textContent() {
    if (this._needsClientResolve()) {
      const expr = `(function(){var el=${this._resolveFirstExpr()};return el?el.textContent:null;})()`;
      const res = await this._send({ kind: "evaluate", page_id: this._pageId, expression: expr });
      if (res.kind === "error") throw new Error(res.message);
      return res.value != null ? String(res.value) : null;
    }
    const res = await this._send({
      kind: "get_text",
      page_id: this._pageId,
      selector: this._selector,
    });
    if (res.kind === "error") throw new Error(res.message);
    return res.value;
  }

  async getAttribute(name) {
    if (this._needsClientResolve()) {
      const n = JSON.stringify(name);
      const expr = `(function(){var el=${this._resolveFirstExpr()};return el?el.getAttribute(${n}):null;})()`;
      const res = await this._send({ kind: "evaluate", page_id: this._pageId, expression: expr });
      if (res.kind === "error") throw new Error(res.message);
      return res.value != null ? String(res.value) : null;
    }
    const res = await this._send({
      kind: "get_attribute",
      page_id: this._pageId,
      selector: this._selector,
      attribute: name,
    });
    if (res.kind === "error") throw new Error(res.message);
    return res.value;
  }

  async boundingBox() {
    if (this._needsClientResolve()) {
      const expr = `(function(){var el=${this._resolveFirstExpr()};if(!el)return null;var r=el.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height};})()`;
      const res = await this._send({ kind: "evaluate", page_id: this._pageId, expression: expr });
      if (res.kind === "error") throw new Error(res.message);
      return res.value || null;
    }
    const res = await this._send({
      kind: "bounding_box",
      page_id: this._pageId,
      selector: this._selector,
    });
    if (res.kind === "error") throw new Error(res.message);
    if (res.x == null) return null;
    return { x: res.x, y: res.y, width: res.width, height: res.height };
  }

  async isVisible() {
    const expr = `(function(){var el=${this._resolveFirstExpr()};${_visibilityCheckSrc}})()`;
    const res = await this._send({ kind: "evaluate", page_id: this._pageId, expression: expr });
    if (res.kind === "error") return false;
    return Boolean(res.value);
  }

  async isHidden() {
    return !(await this.isVisible());
  }

  async isEnabled() {
    const expr = `(function(){var el=${this._resolveFirstExpr()};if(!el)return false;return !el.disabled;})()`;
    const res = await this._send({ kind: "evaluate", page_id: this._pageId, expression: expr });
    if (res.kind === "error") return false;
    return Boolean(res.value);
  }

  async isDisabled() {
    return !(await this.isEnabled());
  }

  async isChecked() {
    const expr = `(function(){var el=${this._resolveFirstExpr()};if(!el)return false;return !!el.checked;})()`;
    const res = await this._send({ kind: "evaluate", page_id: this._pageId, expression: expr });
    if (res.kind === "error") return false;
    return Boolean(res.value);
  }

  async isFocused() {
    const expr = `(function(){var el=${this._resolveFirstExpr()};if(!el)return false;return el===document.activeElement;})()`;
    const res = await this._send({ kind: "evaluate", page_id: this._pageId, expression: expr });
    if (res.kind === "error") return false;
    return Boolean(res.value);
  }

  // Returns the resolved value of `window.getComputedStyle(el)[name]`, or null
  // if the element is missing. Name is a camelCase CSS property.
  async computedStyle(name) {
    const n = JSON.stringify(name);
    const expr = `(function(){var el=${this._resolveFirstExpr()};if(!el)return null;return window.getComputedStyle(el)[${n}];})()`;
    const res = await this._send({ kind: "evaluate", page_id: this._pageId, expression: expr });
    if (res.kind === "error") throw new Error(res.message);
    return res.value != null ? String(res.value) : null;
  }

  // Returns the accessible name, computed per the Playwright rule:
  // aria-label > aria-labelledby text > innerText > alt/title. Good-enough
  // subset; full AccName spec is not replicated.
  async accessibleName() {
    const expr = `(function(){
      var el = ${this._resolveFirstExpr()};
      if (!el) return null;
      var al = el.getAttribute('aria-label');
      if (al) return al;
      var alby = el.getAttribute('aria-labelledby');
      if (alby) {
        var parts = alby.split(/\\s+/).map(function(id){ var n=document.getElementById(id); return n ? (n.innerText||n.textContent||'') : ''; });
        var joined = parts.join(' ').trim();
        if (joined) return joined;
      }
      if (el.tagName === 'INPUT' && (el.type === 'submit' || el.type === 'button')) {
        return el.value || '';
      }
      if (el.tagName === 'IMG') {
        return el.alt || el.title || '';
      }
      var t = (el.innerText || el.textContent || '').trim();
      if (t) return t;
      return el.title || '';
    })()`;
    const res = await this._send({ kind: "evaluate", page_id: this._pageId, expression: expr });
    if (res.kind === "error") throw new Error(res.message);
    return res.value != null ? String(res.value) : null;
  }

  // Returns the ARIA role — explicit `role` attribute if present, else the
  // implicit role derived from tagName+type (common cases only).
  async role() {
    const expr = `(function(){
      var el = ${this._resolveFirstExpr()};
      if (!el) return null;
      var r = el.getAttribute('role');
      if (r) return r;
      var tag = el.tagName;
      if (tag === 'BUTTON') return 'button';
      if (tag === 'A' && el.hasAttribute('href')) return 'link';
      if (tag === 'INPUT') {
        var t = (el.type || '').toLowerCase();
        if (t === 'checkbox') return 'checkbox';
        if (t === 'radio') return 'radio';
        if (t === 'submit' || t === 'button' || t === 'reset') return 'button';
        if (t === 'range') return 'slider';
        return 'textbox';
      }
      if (tag === 'SELECT') return 'combobox';
      if (tag === 'TEXTAREA') return 'textbox';
      if (tag === 'NAV') return 'navigation';
      if (tag === 'MAIN') return 'main';
      if (tag === 'HEADER') return 'banner';
      if (tag === 'FOOTER') return 'contentinfo';
      if (tag === 'H1' || tag === 'H2' || tag === 'H3' || tag === 'H4' || tag === 'H5' || tag === 'H6') return 'heading';
      if (tag === 'UL' || tag === 'OL') return 'list';
      if (tag === 'LI') return 'listitem';
      if (tag === 'IMG') return el.getAttribute('alt') ? 'img' : 'presentation';
      return '';
    })()`;
    const res = await this._send({ kind: "evaluate", page_id: this._pageId, expression: expr });
    if (res.kind === "error") throw new Error(res.message);
    return res.value != null ? String(res.value) : null;
  }

  async count() {
    const expr = this._needsClientResolve()
      ? this._resolveCountExpr()
      : `document.querySelectorAll(${JSON.stringify(this._selector)}).length`;
    const res = await this._send({ kind: "evaluate", page_id: this._pageId, expression: expr });
    if (res.kind === "error") throw new Error(res.message);
    return typeof res.value === "number" ? res.value : 0;
  }

  async innerHTML() {
    const expr = `(function(){var el=${this._resolveFirstExpr()};return el?el.innerHTML:null;})()`;
    const res = await this._send({ kind: "evaluate", page_id: this._pageId, expression: expr });
    if (res.kind === "error") throw new Error(res.message);
    return res.value != null ? String(res.value) : null;
  }

  async innerText() {
    const expr = `(function(){var el=${this._resolveFirstExpr()};return el?el.innerText:null;})()`;
    const res = await this._send({ kind: "evaluate", page_id: this._pageId, expression: expr });
    if (res.kind === "error") throw new Error(res.message);
    return res.value != null ? String(res.value) : null;
  }

  async inputValue() {
    const expr = `(function(){var el=${this._resolveFirstExpr()};return el?el.value:null;})()`;
    const res = await this._send({ kind: "evaluate", page_id: this._pageId, expression: expr });
    if (res.kind === "error") throw new Error(res.message);
    return res.value != null ? String(res.value) : null;
  }
}

// ── NthLocator ────────────────────────────────────────────────────────────────
//
// Scopes actions and reads to the element at index `_index` within the
// resolved match set. For CDP-direct actions (click/fill/hover/press), it
// routes through `_taggedSelectorScope` so the real CDP input event targets
// the indexed element — not match[0].
//
// @spec locator-js-api#L6 L7
class NthLocator extends Locator {
  constructor(selector, sendRequest, pageId, index, options) {
    super(selector, sendRequest, pageId, options);
    this._index = index;
  }

  // Override: apply nth selection *after* the parent's match collection.
  _resolveFirstExpr() {
    const matches = `(${_collectMatchesSrc(this._selector, this._filters)})()`;
    if (this._index === -1) {
      return `(function(){var els=${matches};return els[els.length-1]||null;})()`;
    }
    return `(function(){var els=${matches};return els[${this._index}]||null;})()`;
  }

  _resolveCountExpr() {
    // Count of a nth-locator is 1 if the element exists, 0 otherwise —
    // matches Playwright semantics.
    return `(${this._resolveFirstExpr()} ? 1 : 0)`;
  }

  // NthLocator always needs client resolve (can't compile an nth CSS).
  _needsClientResolve() { return true; }
}

// ── Private helpers ───────────────────────────────────────────────────────────

function _isPseudoSelector(sel) {
  return typeof sel === "string" && (sel.startsWith("role=") || sel.startsWith("text="));
}

// Visibility check source, reused by isVisible + auto-wait.
// Expects an `el` variable to be in scope (set by the enclosing factory).
const _visibilityCheckSrc = `
if (!el) return false;
var s = window.getComputedStyle(el);
if (s.visibility === 'hidden' || s.display === 'none') return false;
if (el.offsetParent === null && el.tagName !== 'BODY' && el.tagName !== 'HTML') return false;
var r = el.getBoundingClientRect();
return r.width > 0 && r.height > 0;
`;

// Emit a JS source string for a zero-arg function that returns an Array of
// Elements matching `selector` after applying all `filters`.
//
// Selector syntax supported inside filter scopes and at the top level:
//   role=<role>[name="<n>"]   (maps to [role=<role>][aria-label^="<n>"] as a
//                              best-effort CSS fallback — same rule as the
//                              existing Page.getByRole)
//   text=<substring>          (maps to innerText includes filter)
//   <css>                     (querySelectorAll)
function _collectMatchesSrc(selector, filters) {
  const lines = [];
  lines.push("function() {");
  lines.push("  var __jet_match = function(root, sel) {");
  lines.push("    if (typeof sel !== 'string') return [];");
  lines.push("    if (sel.indexOf('role=') === 0) {");
  lines.push("      var body = sel.substring(5);");
  lines.push("      var role = body; var name = null;");
  lines.push("      var m = body.match(/^([A-Za-z]+)\\[name=\"(.*)\"\\]$/);");
  lines.push("      if (m) { role = m[1]; name = m[2]; }");
  lines.push("      var arr = Array.prototype.slice.call(root.querySelectorAll('[role=\"' + role + '\"], ' + role));");
  lines.push("      if (name) arr = arr.filter(function(e){ var n = e.getAttribute('aria-label') || e.innerText || ''; return n.indexOf(name) !== -1; });");
  lines.push("      return arr;");
  lines.push("    }");
  lines.push("    if (sel.indexOf('text=') === 0) {");
  lines.push("      var t = sel.substring(5);");
  lines.push("      return Array.prototype.slice.call(root.querySelectorAll('*')).filter(function(e){ return (e.innerText || '').indexOf(t) !== -1; });");
  lines.push("    }");
  lines.push("    return Array.prototype.slice.call(root.querySelectorAll(sel));");
  lines.push("  };");
  lines.push(`  var current = __jet_match(document, ${JSON.stringify(selector)});`);
  for (const f of filters) {
    if (f.kind === "scope") {
      lines.push(`  current = current.flatMap(function(parent){ return __jet_match(parent, ${JSON.stringify(f.child)}); });`);
    } else if (f.kind === "text") {
      if (f.hasText != null) {
        if (f.hasText instanceof RegExp) {
          lines.push(`  { var __re = ${_regexLiteral(f.hasText)}; current = current.filter(function(e){ return __re.test(e.innerText || ''); }); }`);
        } else {
          lines.push(`  { var __sub = ${JSON.stringify(String(f.hasText))}; current = current.filter(function(e){ return (e.innerText || '').indexOf(__sub) !== -1; }); }`);
        }
      }
      if (f.hasNotText != null) {
        if (f.hasNotText instanceof RegExp) {
          lines.push(`  { var __re2 = ${_regexLiteral(f.hasNotText)}; current = current.filter(function(e){ return !__re2.test(e.innerText || ''); }); }`);
        } else {
          lines.push(`  { var __sub2 = ${JSON.stringify(String(f.hasNotText))}; current = current.filter(function(e){ return (e.innerText || '').indexOf(__sub2) === -1; }); }`);
        }
      }
    }
  }
  lines.push("  return current;");
  lines.push("}");
  return lines.join("\n");
}

function _regexLiteral(re) {
  // Turn a RegExp into its JS source literal form.
  return "new RegExp(" + JSON.stringify(re.source) + "," + JSON.stringify(re.flags) + ")";
}

function _uuid() {
  // Short non-cryptographic token — fine for DOM scoping.
  return "j" + Math.random().toString(36).slice(2, 10) + Date.now().toString(36);
}

function _sleep(ms) {
  return new Promise(function (resolve) { setTimeout(resolve, ms); });
}

// Returns the JS source of the fetch/XHR installer, evaluated inside the
// page. The installer defines window.__jetRoutes (if not present) and
// overrides window.fetch + XMLHttpRequest to consult the list. Idempotent —
// page.route() gates the call with window.__jetRoutesInstalled.
//
// @spec .aw/tech-design/projects/jet/logic/route-intercept.md#R4
function _jetRouteInstallerSrc() {
  return `
    window.__jetRoutes = window.__jetRoutes || [];
    var __matchPattern = function(url, pat){
      if (pat.kind === 'regex') return new RegExp(pat.source, pat.flags).test(url);
      var g = pat.value;
      // Expand ** → .* and * → [^/]* for glob matching. Escape other regex meta.
      var re = '^' + g.replace(/[.+^\\\${}()|[\\]\\\\]/g, '\\\\$&')
        .replace(/\\*\\*/g, '\\u0001').replace(/\\*/g, '[^/]*').replace(/\\u0001/g, '.*') + '$';
      return new RegExp(re).test(url);
    };
    var __fulfillBodyToBuffer = function(body){
      if (body == null) return '';
      if (typeof body === 'string') return body;
      try { return JSON.stringify(body); } catch (e) { return String(body); }
    };
    var __origFetch = window.fetch.bind(window);
    window.fetch = function(input, init){
      var url = typeof input === 'string' ? input : (input && input.url) || '';
      for (var i = 0; i < window.__jetRoutes.length; i++) {
        var r = window.__jetRoutes[i];
        if (!__matchPattern(url, r.pattern)) continue;
        var c = r.config;
        if (c.abort) {
          return Promise.reject(new TypeError('net::ERR_FAILED (jet route aborted)'));
        }
        var headers = Object.assign({}, c.headers || {});
        if (c.contentType && !Object.keys(headers).map(function(k){return k.toLowerCase();}).includes('content-type')) {
          headers['content-type'] = c.contentType;
        }
        return Promise.resolve(new Response(__fulfillBodyToBuffer(c.body), {
          status: c.status || 200,
          headers: headers,
        }));
      }
      return __origFetch(input, init);
    };
    // Minimal XHR override — matches by URL, supports static fulfill/abort.
    var __OrigXHR = window.XMLHttpRequest;
    function __JetXHR(){
      this.__orig = new __OrigXHR();
      this.__listeners = {};
      this.readyState = 0; this.status = 0; this.responseText = ''; this.response = '';
      var self = this;
      ['load','loadend','error','abort','readystatechange'].forEach(function(evt){
        self.__orig.addEventListener(evt, function(){ self.__dispatch(evt); });
      });
    }
    __JetXHR.prototype.addEventListener = function(evt, fn){
      (this.__listeners[evt] = this.__listeners[evt] || []).push(fn);
    };
    __JetXHR.prototype.__dispatch = function(evt){
      this.readyState = this.__orig.readyState;
      this.status = this.__orig.status;
      this.responseText = this.__orig.responseText;
      this.response = this.__orig.response;
      (this.__listeners[evt] || []).forEach(function(fn){ try { fn.call(this); } catch (e) {} }, this);
      var h = this['on' + evt];
      if (typeof h === 'function') { try { h.call(this); } catch (e) {} }
    };
    __JetXHR.prototype.open = function(method, url){
      this.__method = method;
      this.__url = url;
      for (var i = 0; i < window.__jetRoutes.length; i++) {
        var r = window.__jetRoutes[i];
        if (__matchPattern(url, r.pattern)) { this.__mock = r.config; return; }
      }
      this.__orig.open.apply(this.__orig, arguments);
    };
    __JetXHR.prototype.setRequestHeader = function(k, v){
      if (this.__mock) return;
      this.__orig.setRequestHeader(k, v);
    };
    __JetXHR.prototype.send = function(body){
      var self = this;
      if (this.__mock) {
        var c = self.__mock;
        setTimeout(function(){
          if (c.abort) {
            self.readyState = 4; self.status = 0; self.responseText = ''; self.response = '';
            (self.__listeners.error || []).forEach(function(fn){ try { fn.call(self); } catch(e){} });
            if (typeof self.onerror === 'function') { try { self.onerror(); } catch(e){} }
            return;
          }
          self.readyState = 4; self.status = c.status || 200;
          self.responseText = __fulfillBodyToBuffer(c.body);
          self.response = self.responseText;
          ['readystatechange', 'load', 'loadend'].forEach(function(evt){
            (self.__listeners[evt] || []).forEach(function(fn){ try { fn.call(self); } catch(e){} });
            var h = self['on' + evt];
            if (typeof h === 'function') { try { h.call(self); } catch(e){} }
          });
        }, 0);
        return;
      }
      this.__orig.send(body);
    };
    window.XMLHttpRequest = __JetXHR;
  `;
}
// CODEGEN-END
