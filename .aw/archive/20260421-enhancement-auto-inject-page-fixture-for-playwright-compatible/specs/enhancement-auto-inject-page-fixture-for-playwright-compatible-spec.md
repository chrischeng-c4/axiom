---
id: enhancement-auto-inject-page-fixture-for-playwright-compatible-spec
main_spec_ref: "crates/jet/testing/page-fixture-auto-inject.md"
merge_strategy: new
fill_sections: [overview, requirements, scenarios, interaction, logic, test-plan, changes, doc]
create_complete: true
---

# Enhancement Auto Inject Page Fixture For Playwright Compatible Spec

## Overview
<!-- type: overview lang: markdown -->

Jet's native test runner (Phase 1–4) wires a CDP-backed `page` object through `test.extend()`, requiring each spec to call `test.extend({ page: ... })` before using `page` in test bodies. Playwright-style specs use plain `test('name', async ({ page }) => ...)` with no extend call. This mismatch blocked Phase 5 closeout after the `@jet/test` resolver fix.

This change adds a **default fixture registry** to `crates/jet/runtime/test/index.js` that pre-registers `page` as a built-in fixture backed by the CDP driver. The `page` fixture is injected automatically into every test body and `beforeEach` callback that destructures `page` from the fixture argument — no `test.extend()` call needed. Tests that do not destructure `page` are unaffected.

The browser process (Chromium via CDP) is launched **once per worker** at the start of `run_spec` in `crates/jet/src/test_runner/worker.rs` and closed on worker teardown. `baseURL` and `headless` from `jet.test.config.ts` are forwarded to the JS runtime via the boot script opts object, and `page.goto(relativePath)` resolves relative URLs against `baseURL`.

`test.extend({ page: ... })` continues to work: a user-supplied `page` fixture overrides the default for tests that use the extended test object. User fixtures that themselves accept `page` receive the CDP-backed default.
## Requirements
<!-- type: requirements lang: mermaid -->

```mermaid
---
id: requirements
---
requirementDiagram

  requirement R1 {
    id: R1
    text: "page fixture auto-injected into test body and beforeEach without explicit test.extend call"
    risk: low
    verifymethod: test
  }

  requirement R2 {
    id: R2
    text: "page fixture backed by native CDP driver; no Playwright package import"
    risk: low
    verifymethod: inspection
  }

  requirement R3 {
    id: R3
    text: "page.goto(relativePath) resolves against use.baseURL from active project config"
    risk: medium
    verifymethod: test
  }

  requirement R4 {
    id: R4
    text: "each test receives a fresh page instance; page is auto-closed after test body and afterEach hooks complete, pass or fail"
    risk: medium
    verifymethod: test
  }

  requirement R5 {
    id: R5
    text: "browser process shared across all tests in one worker; launched once per worker, closed on worker teardown"
    risk: medium
    verifymethod: test
  }

  requirement R6 {
    id: R6
    text: "injected page exposes: goto, locator, getByText, click, fill, waitForSelector, waitForLoadState, evaluate, url, close; coverage verified against e2e call sites"
    risk: low
    verifymethod: inspection
  }

  requirement R7 {
    id: R7
    text: "test.extend({page: userImpl}) overrides default page fixture for tests using that extended test object"
    risk: low
    verifymethod: test
  }

  requirement R8 {
    id: R8
    text: "user fixtures declared via test.extend that accept page as argument receive the CDP-backed default page"
    risk: medium
    verifymethod: test
  }

  requirement R9 {
    id: R9
    text: "fixture injection does not break existing tests that do not destructure page from the fixture argument"
    risk: low
    verifymethod: test
  }

  requirement R10 {
    id: R10
    text: "runtime surfaces clear actionable error message when CDP browser fails to launch or page cannot be created"
    risk: low
    verifymethod: test
  }
```
## Scenarios
<!-- type: scenarios lang: markdown -->

```yaml
- id: S1
  requirement: R1
  given: a test file contains `test('name', async ({ page }) => ...)` with no test.extend call
  when: jet executes the spec file
  then: page is defined and is a CDP-backed Page instance; no TypeError about undefined

- id: S2
  requirement: R2
  given: the runtime fixture registry is initialized
  when: the page fixture is resolved
  then: the page object is constructed from CdpSession/Page types in crates/jet/src/cdp_driver/; no require/import of @playwright/test or playwright-core is present in the bundle

- id: S3
  requirement: R3
  given: jet.test.config.ts has `use: { baseURL: 'http://localhost:4200' }` for the active project
  when: page.goto('/dashboard') is called inside a test body
  then: the CDP driver navigates to 'http://localhost:4200/dashboard'

- id: S4
  requirement: R4
  given: two sequential tests each destructure page
  when: both tests run in the same worker
  then: each test gets a distinct Page instance; the first page is closed before the second test begins; page from test-1 is inaccessible in test-2

- id: S5
  requirement: R4
  given: a test body throws an unhandled error after calling page.goto
  when: jet catches the error and runs afterEach hooks
  then: page.close() is still called; no dangling page remains open in the browser

- id: S6
  requirement: R5
  given: a worker is assigned ten test cases
  when: all ten tests run sequentially
  then: exactly one Browser instance is launched at worker start and one Browser.close() is called at worker teardown; no additional browser processes are spawned

- id: S7
  requirement: R6
  given: a test calls page.locator('.btn').click()
  when: the locator action is executed
  then: the CDP driver dispatches a click via the locator engine; the action completes without error

- id: S8
  requirement: R7
  given: a spec calls `const myTest = test.extend({ page: async ({}, use) => { ... await use(customPage); } })`
  when: myTest('name', async ({ page }) => ...) runs
  then: page inside the callback is the customPage supplied by the user fixture, not the CDP default

- id: S9
  requirement: R8
  given: a user fixture declared via test.extend accepts `{ page }` as its first argument
  when: the user fixture is resolved
  then: the page argument is the CDP-backed default page instance

- id: S10
  requirement: R9
  given: a test is defined as `test('name', async () => ...)` with no fixture argument
  when: jet executes the test
  then: the test runs normally; no page is created or closed for this test; no error is thrown

- id: S11
  requirement: R10
  given: the Chromium binary is missing or the CDP port is already in use
  when: the worker attempts to launch the browser
  then: jet emits an error message containing the word 'browser' and the underlying OS error; the test is marked failed with that message, not a silent undefined crash
```
## Mindmap
<!-- type: mindmap lang: mermaid -->
<!-- TODO: Use Mermaid Plus mindmap (YAML frontmatter inside mermaid block).
```mermaid
---
id: mindmap
---
mindmap
  root((System))
    Component A
    Component B
```
-->

## State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO: Use Mermaid Plus stateDiagram-v2 (YAML frontmatter inside mermaid block).
```mermaid
---
id: state-machine
initial: idle
---
stateDiagram-v2
    [*] --> idle
```
-->

## Interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: page-fixture-lifecycle
actors:
  - id: TestRunner
    kind: system
    label: "jet worker (Rust)"
  - id: FixtureRegistry
    kind: participant
    label: "fixture registry (JS)"
  - id: CdpDriver
    kind: system
    label: "cdp_driver (Rust)"
  - id: Chromium
    kind: system
    label: "Chromium process"
  - id: TestBody
    kind: participant
    label: "test body callback (JS)"
messages:
  - from: TestRunner
    to: CdpDriver
    name: browser_launch
    label: "once per worker: Browser::launch(headless, port)"
  - from: CdpDriver
    to: Chromium
    name: spawn_chromium
    label: "spawn Chromium with --remote-debugging-port"
  - from: Chromium
    to: CdpDriver
    name: cdp_ready
    returns: BrowserHandle
  - from: CdpDriver
    to: TestRunner
    name: browser_handle
    returns: Browser
  - from: TestRunner
    to: FixtureRegistry
    name: run_test
    label: "per test: pass Browser + baseURL"
  - from: FixtureRegistry
    to: CdpDriver
    name: new_page
    label: "Browser::new_page()"
  - from: CdpDriver
    to: FixtureRegistry
    name: page_handle
    returns: Page
  - from: FixtureRegistry
    to: TestBody
    name: inject_page
    label: "destructure-detected fixture arg {page}"
  - from: TestBody
    to: FixtureRegistry
    name: test_complete
    label: "resolve or reject"
  - from: FixtureRegistry
    to: CdpDriver
    name: page_close
    label: "Page::close() in finally block"
  - from: TestRunner
    to: CdpDriver
    name: browser_close
    label: "worker teardown: Browser::close()"
  - from: CdpDriver
    to: Chromium
    name: kill_process
---
sequenceDiagram
    TestRunner->>CdpDriver: browser_launch (once per worker)
    CdpDriver->>Chromium: spawn Chromium --remote-debugging-port
    Chromium-->>CdpDriver: CDP ready
    CdpDriver-->>TestRunner: Browser handle
    loop per test
        TestRunner->>FixtureRegistry: run_test(Browser, baseURL)
        FixtureRegistry->>CdpDriver: Browser::new_page()
        CdpDriver-->>FixtureRegistry: Page handle
        FixtureRegistry->>TestBody: inject {page} into callback
        TestBody-->>FixtureRegistry: test complete (pass/fail)
        FixtureRegistry->>CdpDriver: Page::close()
    end
    TestRunner->>CdpDriver: Browser::close() on worker teardown
    CdpDriver->>Chromium: terminate process
```
## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: fixture-injection-logic
entry: resolve_fixtures
nodes:
  resolve_fixtures: { kind: start, label: "resolve fixture args for test callback" }
  parse_params: { kind: process, label: "parse callback parameter names via fn.toString()" }
  has_page_param: { kind: decision, label: "destructures 'page'?" }
  check_user_override: { kind: decision, label: "user test.extend defines 'page'?" }
  use_user_page: { kind: process, label: "use user-provided page fixture" }
  create_cdp_page: { kind: process, label: "Browser::new_page() via wire channel" }
  baseurl_check: { kind: decision, label: "goto arg is relative URL?" }
  resolve_baseurl: { kind: process, label: "prepend baseURL from worker config" }
  pass_absolute: { kind: process, label: "pass URL as-is to CDP" }
  inject_and_run: { kind: process, label: "inject resolved fixtures; invoke test callback" }
  close_page: { kind: process, label: "Page::close() in finally" }
  no_page_needed: { kind: process, label: "skip page fixture; run test with remaining fixtures" }
  done: { kind: terminal, label: "test complete" }
edges:
  - from: resolve_fixtures
    to: parse_params
  - from: parse_params
    to: has_page_param
  - from: has_page_param
    to: check_user_override
    label: "yes"
  - from: has_page_param
    to: no_page_needed
    label: "no"
  - from: check_user_override
    to: use_user_page
    label: "yes"
  - from: check_user_override
    to: create_cdp_page
    label: "no"
  - from: use_user_page
    to: inject_and_run
  - from: create_cdp_page
    to: inject_and_run
  - from: inject_and_run
    to: close_page
  - from: close_page
    to: done
  - from: no_page_needed
    to: done
---
flowchart TD
    resolve_fixtures([resolve fixture args]) --> parse_params[parse callback params via fn.toString]
    parse_params --> has_page_param{destructures page?}
    has_page_param -->|yes| check_user_override{user test.extend defines page?}
    has_page_param -->|no| no_page_needed[skip page fixture]
    check_user_override -->|yes| use_user_page[use user-provided page fixture]
    check_user_override -->|no| create_cdp_page[Browser::new_page via wire channel]
    use_user_page --> inject_and_run[inject fixtures; invoke test callback]
    create_cdp_page --> inject_and_run
    inject_and_run --> close_page[Page::close in finally]
    close_page --> done([test complete])
    no_page_needed --> done
```

BaseURL resolution sub-logic (called from `create_cdp_page` path when `page.goto` is invoked):

```mermaid
---
id: baseurl-resolution
entry: goto_called
nodes:
  goto_called: { kind: start, label: "page.goto(url) called" }
  is_relative: { kind: decision, label: "url starts with / or is path-only?" }
  prepend_base: { kind: process, label: "url = baseURL + url" }
  pass_through: { kind: process, label: "url unchanged" }
  cdp_navigate: { kind: terminal, label: "CDP Page.navigate(url)" }
edges:
  - from: goto_called
    to: is_relative
  - from: is_relative
    to: prepend_base
    label: "yes"
  - from: is_relative
    to: pass_through
    label: "no"
  - from: prepend_base
    to: cdp_navigate
  - from: pass_through
    to: cdp_navigate
---
flowchart TD
    goto_called([page.goto url called]) --> is_relative{url is relative?}
    is_relative -->|yes| prepend_base[url = baseURL + url]
    is_relative -->|no| pass_through[url unchanged]
    prepend_base --> cdp_navigate([CDP Page.navigate url])
    pass_through --> cdp_navigate
```
## Test Plan
<!-- type: test-plan lang: markdown -->

```mermaid
---
id: test-plan
---
requirementDiagram

  element T1 {
    type: "Test"
    docref: "cargo test -p jet --test page_fixture_auto_inject -- test_page_fixture_auto_injected_into_test_body"
  }

  element T2 {
    type: "Test"
    docref: "cargo test -p jet --test page_fixture_auto_inject -- test_page_auto_closed_after_test"
  }

  element T3 {
    type: "Test"
    docref: "cargo test -p jet --test page_fixture_auto_inject -- test_page_auto_closed_on_test_failure"
  }

  element T4 {
    type: "Test"
    docref: "cargo test -p jet --test page_fixture_auto_inject -- test_browser_shared_across_tests_in_worker"
  }

  element T5 {
    type: "Test"
    docref: "cargo test -p jet --test page_fixture_auto_inject -- test_baseurl_resolution_relative_path"
  }

  element T6 {
    type: "Test"
    docref: "cargo test -p jet --test page_fixture_auto_inject -- test_user_extend_page_overrides_default"
  }

  element T7 {
    type: "Test"
    docref: "cargo test -p jet --test page_fixture_auto_inject -- test_user_fixture_receives_cdp_page_as_dependency"
  }

  element T8 {
    type: "Test"
    docref: "cargo test -p jet --test page_fixture_auto_inject -- test_no_page_no_injection"
  }

  element T9 {
    type: "Test"
    docref: "cargo test -p jet --test page_fixture_auto_inject -- test_cdp_launch_failure_error_message"
  }

  element T10 {
    type: "Test"
    docref: "cargo test -p jet --lib test_runner (regression)"
  }

  element T11 {
    type: "Test"
    docref: "jet test /tmp/page_fixture_smoke.spec.ts (e2e smoke)"
  }

  T1 - verifies -> R1
  T2 - verifies -> R4
  T3 - verifies -> R4
  T4 - verifies -> R5
  T5 - verifies -> R3
  T6 - verifies -> R7
  T7 - verifies -> R8
  T8 - verifies -> R9
  T9 - verifies -> R10
  T10 - verifies -> R9
  T11 - verifies -> R1
  T11 - verifies -> R6
```

Concrete test commands:

| ID | Command | Scope |
|----|---------|-------|
| T1–T9 | `cargo test -p jet --test page_fixture_auto_inject` | new integration test file `crates/jet/tests/page_fixture_auto_inject.rs` |
| T10 | `cargo test -p jet --lib test_runner` | existing unit tests; regression gate |
| T11 | `jet test /tmp/page_fixture_smoke.spec.ts` | e2e smoke against live Chromium; spec contains `test('smoke', async ({ page }) => { await page.goto('/'); })` |
## Changes
<!-- type: changes lang: yaml -->

```yaml
_sdd:
  id: changes

changes:
  - id: C1
    path: crates/jet/runtime/test/index.js
    action: edit
    description: >
      Default fixture registry: register 'page' as built-in fixture at module init (lines ~118–158 around existing test.extend block).
      Destructure-detection: parse callback parameter names via fn.toString() to detect '{page}' in test() and test.beforeEach() callbacks.
      If detected and no user 'page' fixture exists in the merged fixture map, inject CDP-backed page from the default registry.
      baseURL resolution: wrap page.goto so relative URLs are prepended with baseURL received from worker boot opts.
      Inject logic at fixture-argument build site (lines ~420–490).
    refs:
      - $ref: "#fixture-injection-logic"
      - $ref: "#baseurl-resolution"

  - id: C2
    path: crates/jet/src/test_runner/worker.rs
    action: edit
    description: >
      Wire baseURL and headless from the active project RunnerConfig into the JS worker boot script.
      Pass as a JSON blob on the opts object (e.g. opts.jetConfig = { baseURL, headless }) so the fixture registry can read them at startup.
      Launch Browser::launch(headless, port) once before running any test; store the handle in a thread-local or worker-scoped variable.
      Pass the browser handle reference to the fixture registry via the wire channel.
      On worker teardown (after all tests), call Browser::close().
    refs:
      - $ref: "#page-fixture-lifecycle"

  - id: C3
    path: crates/jet/src/test_runner/runner_config.rs
    action: edit
    description: >
      Add fields to RunnerConfig (or ProjectConfig nested struct) if absent:
        base_url: Option<String>
        headless: bool (default true)
      Read from jet.test.config.ts project.use.baseURL and project.use.headless during config parse.

  - id: C4
    path: crates/jet/src/cdp_driver/page_binding.rs
    action: create
    description: >
      New file. JS-exposed wrapper that adapts CdpSession/Page to the Playwright-compatible API subset required by R6.
      Exposes via wire channel: goto(url), locator(selector), getByText(text), click(selector), fill(selector, value),
      waitForSelector(selector, opts), waitForLoadState(state), evaluate(expr), url(), close().
      goto implementation delegates baseURL resolution to the caller (JS side prepends before calling Rust).
      Each method maps to existing CdpSession CDP commands; no Playwright dependency.
    refs:
      - $ref: "#page-fixture-lifecycle"

  - id: C5
    path: crates/jet/runtime/test/page.js
    action: create
    description: >
      New file. JS-side Page class that proxies each Playwright-compatible method to the Rust CDP wire channel.
      Constructor accepts a pageId (CDP target ID) and a wireChannel reference.
      Methods: goto(url), locator(selector), getByText(text), click(selector), fill(selector, value),
      waitForSelector(selector, opts), waitForLoadState(state), evaluate(expr), url(), close().
      locator() and getByText() return a Locator proxy object with: click(), fill(), waitFor(), textContent(), getAttribute().
      Imported by the fixture registry in index.js.

  - id: C6
    path: crates/jet/tests/page_fixture_auto_inject.rs
    action: create
    description: >
      New integration test file. Contains #[test] blocks (T1–T9):
        test_page_fixture_auto_injected_into_test_body — runs a minimal spec string via the worker, asserts page is defined
        test_page_auto_closed_after_test — asserts page target is gone after test completes (query CDP targets)
        test_page_auto_closed_on_test_failure — same as above but test body panics
        test_browser_shared_across_tests_in_worker — asserts browser process count stays 1 across 3 sequential tests
        test_baseurl_resolution_relative_path — asserts goto('/path') results in CDP navigate to baseURL+path
        test_user_extend_page_overrides_default — asserts user fixture page identity differs from CDP default
        test_user_fixture_receives_cdp_page_as_dependency — asserts user fixture arg page is CdpPage instance
        test_no_page_no_injection — asserts no browser launch when test does not destructure page
        test_cdp_launch_failure_error_message — kills Chromium binary; asserts error message contains 'browser'
    refs:
      - $ref: "#test-plan"

  - id: C7
    path: jet.test.config.ts
    action: edit
    description: >
      Verify that each project block in use: { baseURL, headless } is present and correctly parsed.
      No new fields needed if they exist; add baseURL to any project block that lacks it so the plumbing test (T5) has a value to read.

  - id: C8
    path: .score/tech_design/crates/jet/testing/test-runner.md
    action: edit
    description: >
      Add section 'Default Fixture Registry' under Requirements: document built-in page fixture, destructure-detection logic, test.extend override precedence.
      Add section 'Page Fixture Lifecycle' under Runner Architecture: fresh-per-test creation, auto-close after afterEach, error surface for launch failure.
      Update T6 requirement row to reflect auto-injection without explicit test.extend.

  - id: C9
    path: .score/tech_design/crates/jet/testing/browser-driver.md
    action: edit
    description: >
      Add 'baseURL Resolution' section: document how use.baseURL is threaded from RunnerConfig through worker wire protocol to Page::goto.
      Add 'Worker Lifecycle' note: Browser owned by worker process, closed on worker shutdown.

  - id: C10
    path: .score/tech_design/crates/jet/testing/locator-engine.md
    action: edit
    description: >
      Add getByText to Design Contract table (currently missing; required by R6).
      Add 'JS Bridge' section: document how locator actions are forwarded from JS fixture proxy to Rust Locator struct over the wire channel.

  - id: C11
    path: .score/tech_design/crates/jet/testing/worker-pool.md
    action: edit
    description: >
      Add note under 'Per-worker browser isolation': the single Browser instance per worker is the same instance that backs the auto-injected page fixture; no second browser is launched for fixture setup.

  - id: C12
    path: .score/tech_design/crates/jet/e2e/e2e-test-infrastructure.md
    action: edit
    description: >
      Add 'page API Call-Site Inventory' table listing every page.* method called across e2e/jet/tests/*.spec.ts.
      Annotate which calls are covered by R6 and flag any gaps for follow-on issues.
```
## Doc
<!-- type: doc lang: markdown -->

### page fixture (auto-injected)

Jet auto-injects a `page` fixture into every test body and `test.beforeEach` callback that destructures `page` from the fixture argument. No `test.extend()` call is required.

```ts
import { test, expect } from '@jet/test';

test('navigates to home', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('h1')).toHaveText('Welcome');
});
```

### Available methods

| Method | Playwright equivalent |
|--------|----------------------|
| `page.goto(url)` | `page.goto` — relative URLs resolved against `use.baseURL` |
| `page.locator(selector)` | `page.locator` |
| `page.getByText(text)` | `page.getByText` |
| `page.click(selector)` | `page.click` |
| `page.fill(selector, value)` | `page.fill` |
| `page.waitForSelector(selector)` | `page.waitForSelector` |
| `page.waitForLoadState(state)` | `page.waitForLoadState` |
| `page.evaluate(expr)` | `page.evaluate` |
| `page.url()` | `page.url` |
| `page.close()` | `page.close` — called automatically after each test |

### Lifecycle

- Browser: launched once per worker process at startup; closed on worker teardown.
- Page: created fresh for each test; closed automatically after the test body and all `afterEach` hooks complete, whether the test passes or fails.

### Overriding the default page

To replace the default CDP-backed `page` with a custom implementation, use `test.extend`:

```ts
const myTest = test.extend({
  page: async ({}, use) => {
    const p = await createCustomPage();
    await use(p);
    await p.close();
  },
});

myTest('uses custom page', async ({ page }) => { ... });
```

Tests that do not destructure `page` are unaffected; no browser is launched for those tests.

### Configuration

Set `baseURL` in `jet.test.config.ts`:

```ts
export default defineConfig({
  projects: [{
    name: 'local',
    use: { baseURL: 'http://localhost:4200', headless: true },
  }],
});
```

# Reviews

## Review: reviewer (Iteration 1)

**Change ID**: enhancement-auto-inject-page-fixture-for-playwright-compatible

**Verdict**: APPROVED

### Summary

Spec is implementation-ready. Overview explains the Phase 1-4 mismatch (test.extend required vs plain destructure), the default fixture registry solution, and browser-per-worker + baseURL wiring. Requirements are a Mermaid requirementDiagram with R1-R10 mapped 1:1 from the issue body; each has id/text/risk/verifymethod. Scenarios S1-S11 cover R1-R10 with R4 getting both pass and fail paths (S4+S5). Interaction sequenceDiagram shows the TestRunner -> FixtureRegistry -> CdpDriver -> Chromium -> TestBody flow including browser launch (once per worker), page create per test, auto-close in finally, and worker teardown. Logic has two flowcharts: fixture-injection-logic (fn.toString param detection -> user override check -> CDP page create -> inject -> close) and baseurl-resolution (relative URL detection -> prepend baseURL -> CDP navigate). Test Plan is a Mermaid requirementDiagram with T1-T11 each referencing a concrete #[test] function in crates/jet/tests/page_fixture_auto_inject.rs, plus a command table including `cargo test -p jet --test page_fixture_auto_inject`. Changes C1-C12 enumerate concrete files: C1 runtime/test/index.js (fixture registry + injection), C2 worker.rs (browser launch + baseURL wire), C3 runner_config.rs (new fields), C4 cdp_driver/page_binding.rs (NEW), C5 runtime/test/page.js (NEW), C6 tests/page_fixture_auto_inject.rs (NEW), C7 jet.test.config.ts verify, C8-C12 spec edits for 5 existing specs under .score/tech_design/crates/jet/. Doc provides user-facing reference with API subset table, lifecycle notes, and test.extend override pattern. Test Plan -> Changes cross-check: C6 declares the test file holding T1-T11, no hard-reject risk. Irrelevant sections (REST API, Async API, CLI, Config, Wireframe, Component, Design Token, Dependencies, Data Model, RPC API, Schema) were pruned by mainthread to eliminate format-priority violations; frontmatter fill_sections/filled_sections lists only the 8 sections actually filled.

### Issues

No issues found.
