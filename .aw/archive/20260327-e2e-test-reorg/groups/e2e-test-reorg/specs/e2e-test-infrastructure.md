---
id: e2e-test-infrastructure
main_spec_ref: "crates/cclab-jet/e2e/e2e-test-infrastructure.md"
merge_strategy: new
fill_sections: [overview, requirements, scenarios, test-plan, changes]
filled_sections: [overview, requirements, scenarios, test-plan, changes]
create_complete: true
---

# E2e Test Infrastructure

## Overview

<!-- type: overview lang: markdown -->

Reorganize E2E test infrastructure for cclab-jet into a unified `e2e/` directory with Playwright projects isolating build-parity and dev-server test scenarios.

### Current State

| Area | Location | Tests |
|------|----------|-------|
| Grid (RuSheet) | `e2e/` | app.spec.ts (6 tests), cell-editing.spec.ts (10 tests) |
| Jet build parity | `examples/mini-react/tests/` | dom-snapshot.spec.ts (22 tests) |
| Jet dev server | — | None |
| Jet HMR | — | None |
| Jet CSS/PostCSS | — | None |

### Target State

```
e2e/
├── grid/
│   ├── app.spec.ts            # RuSheet canvas, WASM init, responsive
│   └── cell-editing.spec.ts   # Formula input, cell navigation, edit mode
├── jet/
│   ├── src/                   # mini-react TodoMVC fixture app
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   └── tests/
│       ├── build.spec.ts      # DOM parity: vite-build vs jet-build (renamed from dom-snapshot)
│       ├── dev-server.spec.ts # TS stripping, import.meta.env, path alias, proxy, polyfills
│       ├── hmr.spec.ts        # HMR module updates, React Fast Refresh state preservation
│       └── css.spec.ts        # PostCSS pipeline, Tailwind CSS JIT in dev mode
└── playwright.config.ts       # 3 projects: vite-build, jet-build, jet-dev
```

### Playwright Projects

| Project | Port | testMatch | Fixture |
|---------|------|-----------|----------|
| vite-build | 4174 | `**/build.spec.ts` | `npx serve e2e/jet/dist-vite` |
| jet-build | 4175 | `**/build.spec.ts` | `npx serve e2e/jet/dist-jet` |
| jet-dev | 3000 | `**/dev-server.spec.ts, **/hmr.spec.ts, **/css.spec.ts` | `cclab jet dev e2e/jet/` |

### Constraints

- `examples/jet/` and `examples/react-bench/` stay in `examples/` (different purpose: demo/benchmark, not test fixture)
- All jet E2E tests share the single mini-react TodoMVC app as fixture
- Grid tests have no fixture dependency on jet — separate subdirectory
## Requirements

<!-- type: requirements lang: markdown -->

### R1: Directory Restructure

Move existing test files into categorized subdirectories:

| Source | Destination | Action |
|--------|-------------|--------|
| `e2e/app.spec.ts` | `e2e/grid/app.spec.ts` | move |
| `e2e/cell-editing.spec.ts` | `e2e/grid/cell-editing.spec.ts` | move |
| `examples/mini-react/` (entire directory) | `e2e/jet/` | move |
| `examples/mini-react/tests/dom-snapshot.spec.ts` | `e2e/jet/tests/build.spec.ts` | move + rename |

No import path changes required — grid tests use relative imports only. Jet tests use relative imports within the fixture app.

### R2: Playwright Configuration

Create unified `e2e/playwright.config.ts` with three projects:

```typescript
projects: [
  {
    name: "vite-build",
    use: { baseURL: "http://localhost:4174" },
    testMatch: "**/build.spec.ts",
  },
  {
    name: "jet-build",
    use: { baseURL: "http://localhost:4175" },
    testMatch: "**/build.spec.ts",
  },
  {
    name: "jet-dev",
    use: { baseURL: "http://localhost:3000" },
    testMatch: ["**/dev-server.spec.ts", "**/hmr.spec.ts", "**/css.spec.ts"],
  },
]
```

- `testDir`: `"."` (covers both `grid/` and `jet/tests/`)
- Remove `examples/mini-react/playwright.config.ts` after migration
- Shared settings: `headless: true`, `timeout: 30_000`, `retries: 0`

### R3: dev-server.spec.ts

New test file at `e2e/jet/tests/dev-server.spec.ts` covering:

| Test Case | Validates | Assertion |
|-----------|-----------|----------|
| TS type stripping | `import type {}` removed at serve time | No `Unexpected token` in console |
| `import.meta.env` | Env vars injected by dev server | `import.meta.env.DEV === true`, `import.meta.env.MODE === "development"` |
| Path alias resolution | `@/components/...` via tsconfig paths | Component renders without 404 |
| Proxy forwarding | API requests proxied per config | Response from proxy target, not 404 |
| Node.js polyfills | `crypto`, `buffer`, `path` available | Polyfill modules resolve, no runtime errors |

Prerequisite: `cclab jet dev` running on `e2e/jet/` fixture (port 3000).

### R4: hmr.spec.ts

New test file at `e2e/jet/tests/hmr.spec.ts` covering:

| Test Case | Validates | Assertion |
|-----------|-----------|----------|
| Module HMR update | Edit `.tsx` → component re-renders | DOM updated, no `window.location.reload()` |
| React Fast Refresh | Component state preserved after edit | Todo list items survive component re-render |
| CSS HMR | Edit `.css` → styles update | Computed style changes, no page reload |
| Error overlay | Syntax error → overlay; fix → dismiss | Overlay element appears/disappears |

Mechanism: Set state markers via `page.evaluate()`, modify source files via Node.js `fs`, observe DOM changes and WebSocket messages on `/__jet_hmr`.

### R5: css.spec.ts

New test file at `e2e/jet/tests/css.spec.ts` covering:

| Test Case | Validates | Assertion |
|-----------|-----------|----------|
| PostCSS pipeline | `@import` resolution, nested CSS, custom properties | Styles applied correctly in computed style |
| Tailwind JIT | Utility classes in TSX produce correct styles | `getComputedStyle()` matches expected values |
| Dev mode rebuild | Add new Tailwind class → CSS updates | New styles appear without full reload |

### R6: Test Fixture Integrity

- `e2e/jet/` is a self-contained app: `package.json`, `vite.config.ts`, `tsconfig.json`, `src/`
- `npm install` in `e2e/jet/` must succeed independently
- `vite build` produces functional output in `dist-vite/`
- `cclab jet build` produces functional output in `dist-jet/`
- `cclab jet dev` serves the app on port 3000
- Existing 22 DOM snapshot tests pass without modification after rename
## Scenarios

<!-- type: scenarios lang: markdown -->

### S1: Build Parity (vite-build + jet-build projects)

Validates Vite and Jet produce functionally equivalent build output.

1. Build: `cd e2e/jet && npm run build:vite` and `cclab jet build`
2. Serve: `npx serve dist-vite -l 4174` and `npx serve dist-jet -l 4175`
3. Run: `npx playwright test --config=e2e/playwright.config.ts --project=vite-build --project=jet-build`
4. Assert: All 22 DOM snapshot tests pass on both projects with identical behavior

### S2: Dev Server Basics (jet-dev project)

1. Start: `cclab jet dev` on `e2e/jet/` (port 3000)
2. Run: `npx playwright test --config=e2e/playwright.config.ts --project=jet-dev --grep dev-server`
3. Assert:
   - Page loads without `Unexpected token` errors (TS stripped)
   - `import.meta.env.DEV === true` accessible in browser
   - `@/components/Header` resolves (path alias)
   - Proxy endpoint returns upstream response
   - `crypto.randomUUID()` executes (Node.js polyfill)

### S3: HMR Cycle (jet-dev project)

1. Start: `cclab jet dev` on `e2e/jet/` (port 3000)
2. Navigate to app, add a todo item ("HMR test") to establish state
3. Modify `src/components/TodoItem.tsx` on disk (change text or class)
4. Assert: Component re-renders, todo "HMR test" still in list (React Fast Refresh)
5. Modify `src/style.css` on disk (change a color)
6. Assert: Computed style updates, no full page reload
7. Introduce syntax error in `src/app.tsx`
8. Assert: Error overlay appears in browser
9. Fix syntax error
10. Assert: Error overlay dismissed, app functional

### S4: CSS Pipeline (jet-dev project)

1. Start: `cclab jet dev` on `e2e/jet/` (port 3000)
2. Assert: Page renders with correct Tailwind utility styles (e.g., `bg-blue-500` → computed `background-color`)
3. Assert: `@import` in CSS resolves (imported styles applied)
4. Assert: CSS custom properties (`var(--color-primary)`) resolve
5. Add new Tailwind class to a component TSX file
6. Assert: New styles appear in browser without full reload

### S5: Grid Tests Isolation

1. Grid tests in `e2e/grid/` run independently (no jet fixture dependency)
2. Serve grid WASM app on configured port
3. Run: `npx playwright test --config=e2e/playwright.config.ts e2e/grid/`
4. Assert: All 16 existing tests pass (6 from app.spec.ts + 10 from cell-editing.spec.ts)
5. Assert: No test references jet fixture paths or ports
## Diagrams

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- TODO -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- TODO -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- TODO -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- TODO -->

## API Spec

### REST API
<!-- type: rest-api lang: yaml -->
<!-- TODO -->

### RPC API
<!-- type: rpc-api lang: json -->
<!-- TODO -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- TODO -->

### CLI
<!-- type: cli lang: yaml -->
<!-- TODO -->

### Schema
<!-- type: schema lang: json -->
<!-- TODO -->

### Config
<!-- type: config lang: json -->
<!-- TODO -->

## Test Plan

<!-- type: test-plan lang: markdown -->

### Execution

```bash
# All tests
npx playwright test --config=e2e/playwright.config.ts

# Build parity only
npx playwright test --config=e2e/playwright.config.ts --project=vite-build --project=jet-build

# Dev server only (requires cclab jet dev running)
npx playwright test --config=e2e/playwright.config.ts --project=jet-dev

# Grid only
npx playwright test --config=e2e/playwright.config.ts e2e/grid/

# Single test file
npx playwright test --config=e2e/playwright.config.ts --project=jet-dev e2e/jet/tests/hmr.spec.ts
```

### Prerequisites

| Project | Setup Command | Port |
|---------|---------------|------|
| vite-build | `cd e2e/jet && npm run build:vite && npx serve dist-vite -l 4174` | 4174 |
| jet-build | `cd e2e/jet && cclab jet build && npx serve dist-jet -l 4175` | 4175 |
| jet-dev | `cd e2e/jet && cclab jet dev --port 3000` | 3000 |
| grid | Serve grid WASM app (existing setup) | existing |

### Pass Criteria

| Category | File | Count | Criteria |
|----------|------|-------|----------|
| Build parity (vite) | build.spec.ts | 22 | All DOM snapshot tests pass on vite-build |
| Build parity (jet) | build.spec.ts | 22 | All DOM snapshot tests pass on jet-build |
| Dev server | dev-server.spec.ts | 5 | TS strip, env vars, path alias, proxy, polyfills |
| HMR | hmr.spec.ts | 4 | Module update, Fast Refresh, CSS HMR, error overlay |
| CSS | css.spec.ts | 3 | PostCSS pipeline, Tailwind JIT, dev rebuild |
| Grid app | app.spec.ts | 6 | Load, canvas, formula input, responsive, WASM, grid lines |
| Grid editing | cell-editing.spec.ts | 10 | Focus, typing, Enter/Escape, formula, arrows, Tab, select, dblclick, rapid |
| **Total** | | **72** | |

### CI Integration

- Build parity tests: run after `vite build` + `cclab jet build` (no dev server needed)
- Dev server tests: require `cclab jet dev` process (use Playwright `webServer` config to auto-start)
- Grid tests: run against deployed WASM app
- Parallelization: build-parity and grid tests can run concurrently; dev-server tests are sequential (shared port 3000)
## Changes

<!-- type: changes lang: yaml -->

```yaml
files:
  # Grid test moves
  - path: e2e/grid/app.spec.ts
    action: MOVE
    from: e2e/app.spec.ts
    desc: Move RuSheet grid tests into e2e/grid/ subdirectory
  - path: e2e/grid/cell-editing.spec.ts
    action: MOVE
    from: e2e/cell-editing.spec.ts
    desc: Move cell editing tests into e2e/grid/ subdirectory

  # Jet fixture move
  - path: e2e/jet/
    action: MOVE
    from: examples/mini-react/
    desc: Move entire mini-react fixture app (src/, package.json, configs, dist-*/) into e2e/jet/

  # Rename
  - path: e2e/jet/tests/build.spec.ts
    action: RENAME
    from: e2e/jet/tests/dom-snapshot.spec.ts
    desc: Rename dom-snapshot.spec.ts to build.spec.ts (tests build output parity)

  # New test files
  - path: e2e/jet/tests/dev-server.spec.ts
    action: CREATE
    desc: "E2E tests for jet dev server: TS stripping, import.meta.env, path alias, proxy, Node.js polyfills (5 tests)"
  - path: e2e/jet/tests/hmr.spec.ts
    action: CREATE
    desc: "E2E tests for HMR: module updates, React Fast Refresh state preservation, CSS HMR, error overlay (4 tests)"
  - path: e2e/jet/tests/css.spec.ts
    action: CREATE
    desc: "E2E tests for CSS pipeline: PostCSS, Tailwind JIT, dev mode rebuild (3 tests)"

  # Playwright config
  - path: e2e/playwright.config.ts
    action: CREATE
    desc: "Unified Playwright config with 3 projects: vite-build (port 4174), jet-build (port 4175), jet-dev (port 3000)"

  # Cleanup
  - path: examples/mini-react/playwright.config.ts
    action: DELETE
    desc: Replaced by e2e/playwright.config.ts
  - path: examples/mini-react/
    action: DELETE
    desc: Entire directory removed after move to e2e/jet/
```
## Wireframe
<!-- type: wireframe lang: yaml -->

<!-- TODO -->

## Component
<!-- type: component lang: json -->

<!-- TODO -->

## Design Token
<!-- type: design-token lang: json -->

<!-- TODO -->

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->

# Reviews
