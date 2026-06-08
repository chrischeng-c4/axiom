---
id: projects-jet-logic-e2e-test-infrastructure-md
fill_sections: [changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# E2E Test Infrastructure

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: ".aw/tech-design/projects/jet/logic/e2e-test-infrastructure.md"
    action: modify
    section: doc
    impl_mode: hand-written
    description: |
      Legacy Jet TD content retained as notes during AW standardization.
      Rewrite this file into semantic TD sections before promoting source to CODEGEN.
```

## Legacy notes
<!-- type: doc lang: markdown -->

# E2E Test Infrastructure

### Overview

Jet E2E coverage is organized around one unified `e2e/` directory. Grid tests
remain isolated under `e2e/grid/`; Jet build parity, dev-server, HMR, and CSS
tests share a self-contained mini-react fixture under `e2e/jet/`.

This spec replaces the old unexpected-subdir spec
`.aw/tech-design/crates/jet/e2e/e2e-test-infrastructure.md`. The active TD
contract now lives under `logic/` because crate-level spec roots only allow
`README.md` plus known top-level contract directories.

### Current State

| Area | Location | Tests |
|------|----------|-------|
| Grid RuSheet | `e2e/` | `app.spec.ts` and `cell-editing.spec.ts` |
| Jet build parity | `examples/mini-react/tests/` | `dom-snapshot.spec.ts` |
| Jet dev server | none | none |
| Jet HMR | none | none |
| Jet CSS and PostCSS | none | none |

### Target State

```text
e2e/
  grid/
    app.spec.ts
    cell-editing.spec.ts
  jet/
    src/
    package.json
    vite.config.ts
    tsconfig.json
    tests/
      build.spec.ts
      dev-server.spec.ts
      hmr.spec.ts
      css.spec.ts
  playwright.config.ts
```

### Playwright Projects

| Project | Port | Test match | Fixture |
|---------|------|------------|---------|
| `vite-build` | 4174 | `**/build.spec.ts` | `npx serve e2e/jet/dist-vite` |
| `jet-build` | 4175 | `**/build.spec.ts` | `npx serve e2e/jet/dist-jet` |
| `jet-dev` | 3000 | dev-server, HMR, and CSS specs | `cclab jet dev e2e/jet/` |

### Requirements

```mermaid
---
id: jet-e2e-test-infrastructure-requirements
entry: R1
---
requirementDiagram
    requirement R1 {
        id: R1
        text: E2E tests are grouped by product surface
        risk: medium
        verifymethod: inspection
    }
    requirement R2 {
        id: R2
        text: Playwright config exposes build and dev projects
        risk: high
        verifymethod: test
    }
    requirement R3 {
        id: R3
        text: Dev-server tests cover transform and resolution basics
        risk: high
        verifymethod: test
    }
    requirement R4 {
        id: R4
        text: HMR tests cover module CSS and overlay cycles
        risk: high
        verifymethod: test
    }
    requirement R5 {
        id: R5
        text: CSS tests cover PostCSS Tailwind and rebuild behavior
        risk: medium
        verifymethod: test
    }
    requirement R6 {
        id: R6
        text: Jet fixture is self-contained and buildable
        risk: high
        verifymethod: test
    }
```

### R1: Directory Restructure

Existing test files are moved into categorized subdirectories:

| Source | Destination | Action |
|--------|-------------|--------|
| `e2e/app.spec.ts` | `e2e/grid/app.spec.ts` | move |
| `e2e/cell-editing.spec.ts` | `e2e/grid/cell-editing.spec.ts` | move |
| `examples/mini-react/` | `e2e/jet/` | move |
| `examples/mini-react/tests/dom-snapshot.spec.ts` | `e2e/jet/tests/build.spec.ts` | move and rename |

Grid tests use relative imports only. Jet tests use relative imports within the
fixture app, so the move must not introduce cross-fixture dependencies.

### R2: Playwright Configuration

`e2e/playwright.config.ts` must define the `vite-build`, `jet-build`, and
`jet-dev` projects with project-specific `baseURL` and `testMatch` settings.
The config uses `testDir: "."`, headless execution, 30 second timeout, and zero
retries. The old `examples/mini-react/playwright.config.ts` is removed.

### R3: Dev Server Coverage

`e2e/jet/tests/dev-server.spec.ts` covers TypeScript type stripping,
`import.meta.env` injection, path alias resolution, proxy forwarding, and
Node-compatible polyfill module resolution.

### R4: HMR Coverage

`e2e/jet/tests/hmr.spec.ts` covers module updates, React Fast Refresh state
preservation, CSS HMR, and the error-overlay appear/fix/dismiss cycle. Tests
modify source files on disk and observe DOM changes plus `/__jet_hmr`
WebSocket messages.

### R5: CSS Coverage

`e2e/jet/tests/css.spec.ts` covers `@import` resolution, nested CSS, CSS custom
properties, Tailwind JIT utility emission, and dev-mode rebuilds after adding
new classes in TSX.

### R6: Fixture Integrity

`e2e/jet/` is a standalone app with `package.json`, Vite config, TypeScript
config, and source files. `npm install`, `vite build`, `cclab jet build`, and
`cclab jet dev --port 3000` must all work from that fixture.

### Scenarios

```yaml
scenarios:
  - id: S1
    requirement: R1
    given: Grid and Jet E2E tests live in the repository
    when: The E2E directories are organized
    then: Grid tests live under e2e/grid and Jet fixture tests live under e2e/jet/tests
  - id: S2
    requirement: R2
    given: Vite and Jet production builds have been generated
    when: Playwright runs vite-build and jet-build projects
    then: The shared build parity spec runs against both output directories
  - id: S3
    requirement: R3
    given: cclab jet dev serves e2e/jet on port 3000
    when: The dev-server spec runs
    then: Type stripping env injection path aliases proxying and polyfills pass without browser runtime errors
  - id: S4
    requirement: R4
    given: The Jet dev server is running and the app has user state
    when: A TSX module CSS file and syntax error are edited on disk
    then: HMR updates preserve state CSS changes apply without reload and the overlay clears after the fix
  - id: S5
    requirement: R5
    given: CSS imports Tailwind utilities custom properties and nested rules exist in the fixture
    when: The CSS spec exercises dev mode
    then: Computed styles reflect the processed CSS and new classes appear without full reload
  - id: S6
    requirement: R6
    given: e2e/jet contains the moved mini-react fixture
    when: npm install and both Vite and Jet builds run
    then: The fixture installs independently and both output directories serve functional apps
```

### Test Plan

```mermaid
---
id: jet-e2e-test-infrastructure-test-plan
entry: TP1
---
requirementDiagram
    requirement TP1 {
        id: TP1
        text: Run all Playwright projects from unified config
        risk: high
        verifymethod: test
    }
    requirement TP2 {
        id: TP2
        text: Run build parity projects after production builds
        risk: high
        verifymethod: test
    }
    requirement TP3 {
        id: TP3
        text: Run jet-dev tests with cclab jet dev webServer
        risk: high
        verifymethod: test
    }
    requirement TP4 {
        id: TP4
        text: Keep grid tests independent from Jet fixture paths
        risk: medium
        verifymethod: test
    }
```

### Execution Commands

```bash
npx playwright test --config=e2e/playwright.config.ts
npx playwright test --config=e2e/playwright.config.ts --project=vite-build --project=jet-build
npx playwright test --config=e2e/playwright.config.ts --project=jet-dev
npx playwright test --config=e2e/playwright.config.ts e2e/grid/
npx playwright test --config=e2e/playwright.config.ts --project=jet-dev e2e/jet/tests/hmr.spec.ts
```

### Prerequisites

| Project | Setup command | Port |
|---------|---------------|------|
| `vite-build` | `cd e2e/jet && npm run build:vite && npx serve dist-vite -l 4174` | 4174 |
| `jet-build` | `cd e2e/jet && cclab jet build && npx serve dist-jet -l 4175` | 4175 |
| `jet-dev` | `cd e2e/jet && cclab jet dev --port 3000` | 3000 |
| `grid` | existing grid WASM app server setup | existing |

### Pass Criteria

| Category | File | Count | Criteria |
|----------|------|-------|----------|
| Build parity Vite | `build.spec.ts` | 22 | DOM snapshot tests pass on `vite-build` |
| Build parity Jet | `build.spec.ts` | 22 | DOM snapshot tests pass on `jet-build` |
| Dev server | `dev-server.spec.ts` | 5 | Transform, env, alias, proxy, polyfill |
| HMR | `hmr.spec.ts` | 4 | Module update, Fast Refresh, CSS HMR, overlay |
| CSS | `css.spec.ts` | 3 | PostCSS, Tailwind JIT, dev rebuild |
| Grid app | `app.spec.ts` | 6 | Load, canvas, formula input, responsive, WASM, grid lines |
| Grid editing | `cell-editing.spec.ts` | 10 | Focus, typing, Enter/Escape, formula, arrows, Tab, select, double click, rapid input |

### Changes

```yaml
files:
  - path: .aw/tech-design/crates/jet/logic/e2e-test-infrastructure.md
    action: MODIFY
    impl_mode: hand-written
    desc: Move the TD out of unexpected e2e subdir and normalize section formats.
  - path: e2e/grid/app.spec.ts
    action: MOVE
    from: e2e/app.spec.ts
    impl_mode: hand-written
    desc: Move RuSheet grid app tests into e2e/grid.
  - path: e2e/grid/cell-editing.spec.ts
    action: MOVE
    from: e2e/cell-editing.spec.ts
    impl_mode: hand-written
    desc: Move grid cell-editing tests into e2e/grid.
  - path: e2e/jet/
    action: MOVE
    from: examples/mini-react/
    impl_mode: hand-written
    desc: Move the mini-react fixture app into e2e/jet.
  - path: e2e/jet/tests/build.spec.ts
    action: RENAME
    from: e2e/jet/tests/dom-snapshot.spec.ts
    impl_mode: hand-written
    desc: Rename DOM snapshot parity tests to build.spec.ts.
  - path: e2e/jet/tests/dev-server.spec.ts
    action: CREATE
    impl_mode: hand-written
    desc: Add dev-server tests for transform, env, alias, proxy, and polyfill coverage.
  - path: e2e/jet/tests/hmr.spec.ts
    action: CREATE
    impl_mode: hand-written
    desc: Add HMR tests for module, Fast Refresh, CSS, and overlay cycles.
  - path: e2e/jet/tests/css.spec.ts
    action: CREATE
    impl_mode: hand-written
    desc: Add CSS pipeline tests for PostCSS, Tailwind, and rebuild behavior.
  - path: e2e/playwright.config.ts
    action: CREATE
    impl_mode: hand-written
    desc: Add unified Playwright config with vite-build, jet-build, and jet-dev projects.
  - path: examples/mini-react/playwright.config.ts
    action: DELETE
    impl_mode: hand-written
    desc: Replace fixture-local Playwright config with the unified e2e config.
```
