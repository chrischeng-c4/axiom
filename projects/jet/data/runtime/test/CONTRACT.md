# `@jet/test` virtual module contract

`@jet/test` is a **virtual module** — there is no npm install and no npm
publish. The runtime is shipped as part of the `jet` binary and installed
per-worker into a temp `node_modules/@jet/test/` shim. Specs import the
bare specifier and Node's standard ESM resolver finds the shim.

This document defines the contract that specs may rely on. Anything not
listed here is **not part of the contract** and should not be imported by
specs.

## Resolution boundary

| Usage site         | Resolves to                                              |
|--------------------|----------------------------------------------------------|
| `jet test` spec    | `projects/jet/data/runtime/test/index.js` (per-worker shim)   |
| `jet e2e` case     | Same module instance — shared runtime per worker         |
| `@playwright/test` | Compat shim re-exports everything from `@jet/test`       |

`Page` is re-exported from the CDP-backed implementation in
`projects/jet/data/runtime/test/page.js`. The browser singleton (`browser`) is
constructed lazily on the first `new_page` request.

## Supported exports

All names in `__JET_TEST_CONTRACT` are stable. Specs may rely on these
without an npm publish.

| Name             | Shape                              | Notes                                |
|------------------|------------------------------------|--------------------------------------|
| `describe`       | `(name, fn) => void`               | Nested describes supported           |
| `test`           | `(name, fn) => void` + `.skip` / `.extend({...})` | Async + sync fn bodies |
| `expect`         | `(actual) => Matchers` + `.not`     | Matchers listed in #2605; text snapshot via `toMatchTextSnapshot` (#2713) |
| `beforeAll`      | `(fn) => void`                     | Once per spec file                   |
| `afterAll`       | `(fn) => void`                     | Once per spec file                   |
| `beforeEach`     | `(fn) => void`                     | Per-test                             |
| `afterEach`      | `(fn) => void`                     | Per-test                             |
| `Page`           | class                              | CDP-backed Playwright-compat surface |
| `browser`        | lazy `Browser` singleton           | Constructed on first `new_page`      |
| `__JET_TEST_CONTRACT` | `readonly string[]`           | Self-introspection list              |

## Unsupported exports (tripwires)

The following names are exported as **tripwires**: importing the name
succeeds (ESM static binding), but the first runtime access throws a
`JetTestUnsupportedError` carrying a Jet-owned diagnostic. This catches
migrations from Jest/Vitest/Jasmine without forcing the user through
Node's generic `does not provide an export named` error.

| Tripwire | Recommended alternative                                                    |
|----------|----------------------------------------------------------------------------|
| `jest`   | `@jet/test` (`describe` / `test` / `expect`) + the matchers landed in #2605 |
| `vi`     | `@jet/test` + the matchers landed in #2605                                 |
| `vitest` | `@jet/test`                                                                |
| `mock`   | Manual fakes — built-in mocking is not in the contract yet                 |
| `fail`   | `throw new Error(...)` inside the test body, or an `expect(...)` assertion |

Anything else (`fc`, `chai`, `sinon`, ...) currently fails with Node's
default `does not provide an export named` error. Add tripwires here as
real migration friction shows up; do not invent a tripwire for a name
that has never been requested.

## Fixture lifecycle (#2711)

Fixtures declared with `test.extend({ ... })` are **per-test** in the current
contract. Each test owns its own resolution and cleanup chain.

| Phase        | Order                                                                |
|--------------|----------------------------------------------------------------------|
| Setup        | Topological — each fixture's deps resolve before its own body runs   |
| Body         | After every needed fixture has called `use(value)`                   |
| Teardown     | Reverse order of resolution — the last fixture to resolve cleans up first |

Contract guarantees a spec may rely on:

- **Per-test isolation** — every test in a file gets a fresh fixture
  instance. Mutating a value returned from `use(...)` in one test never
  affects the value the next test receives.
- **Reverse-order teardown** — fixture cleanups run in the inverse of
  resolution order so a fixture that depends on another can still see its
  dep during cleanup.
- **Cleanup-failure surfacing** — if a test would otherwise pass but a
  cleanup throws, the test fails with a `fixture-cleanup` source label.
  If the test was already failing, the original failure is preserved.
- **Timeout covers setup + body** — the per-test timeout wraps the entire
  setup-and-run pipeline, so a hanging fixture terminates with the same
  `Test timed out after Nms` error as a hanging body (issue #1534).

Per-file and per-project fixture lifetimes are **not** in the contract
yet — track #2715 (worker isolation) and follow-ups before relying on
shared fixture state across tests.

## Stability rules

- Names in **Supported exports** are append-only between minor releases —
  removing or renaming requires a deprecation cycle.
- **Tripwires** may be promoted to a supported export by removing the
  proxy and adding a real implementation, with a CHANGELOG entry.
- The contract list is the source of truth: any breaking change to
  `__JET_TEST_CONTRACT` is a breaking change.
