---
id: add-production-jet-build-regression-coverage
summary: "Add a production Jet build regression gate that installs, builds, and browser-loads a representative fixture."
fill_sections: [scenarios, mindmap, state-machine, interaction, logic, dependency, db-model, schema, rest-api, rpc-api, async-api, cli, wireframe, component, design-token, config, manifest, runtime-image, deployment, unit-test, e2e-test, changes]
---

# Add Production Jet Build Regression Coverage

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: production_build_regression_gate_runs_full_jet_lifecycle
    given: "A representative Jet fixture imports React, MUI/Emotion, CJS dependencies, ESM package subpaths, extensionless package directories, CSS/style injection, and static assets."
    when: "The regression test runs `jet install --frozen-lockfile`, `jet build`, serves the dist output from the test harness, and browser-loads it through Jet browser tooling."
    then: "The test proves the production bundle boots in a browser-visible surface instead of only compiling unit-level code."
  - id: build_failure_names_exact_phase
    given: "The fixture install, production build, or browser boot fails."
    when: "The regression gate records failure evidence."
    then: "The output names the failing phase and persists command logs plus browser artifacts needed for triage."
  - id: hermetic_fixture_host
    given: "The regression fixture needs to serve production output."
    when: "The browser smoke runs."
    then: "The test uses an in-process Rust static server plus Jet browser tooling and does not depend on an external Python/http-server workaround."
```
## Mindmap
<!-- type: mindmap lang: mermaid -->

```mermaid
---
id: jet-production-build-regression-gate-map
---
mindmap
  root((Production build regression gate))
    Fixture
      React
      MUI Emotion
      CJS and ESM deps
      package subpaths
      styles and assets
    Lifecycle
      jet install frozen lockfile
      jet build
      Jet-hosted browser load
    Evidence
      command logs
      build artifacts
      console output
      screenshot or DOM text
    Diagnostics
      install phase
      build phase
      browser boot phase
```
## State Machine
<!-- type: state-machine lang: mermaid -->

```mermaid
---
id: jet-production-build-regression-state
initial: prepare_fixture
nodes:
  prepare_fixture: { kind: initial, label: "Prepare representative fixture" }
  install: { kind: normal, label: "Run jet install --frozen-lockfile" }
  build: { kind: normal, label: "Run jet build" }
  serve: { kind: normal, label: "Serve built output with Jet-owned tooling" }
  browser_load: { kind: normal, label: "Browser-load built app" }
  capture_failure: { kind: normal, label: "Persist phase-specific artifacts" }
  pass: { kind: terminal, label: "Regression gate passed" }
edges:
  - { from: prepare_fixture, to: install, event: "fixture ready" }
  - { from: install, to: build, event: "install passed" }
  - { from: build, to: serve, event: "build passed" }
  - { from: serve, to: browser_load, event: "server ready" }
  - { from: browser_load, to: pass, event: "boot proof observed" }
  - { from: install, to: capture_failure, event: "install failed" }
  - { from: build, to: capture_failure, event: "build failed" }
  - { from: serve, to: capture_failure, event: "serve failed" }
  - { from: browser_load, to: capture_failure, event: "browser failed" }
---
stateDiagram-v2
    [*] --> prepare_fixture
    prepare_fixture --> install
    install --> build: install passed
    build --> serve: build passed
    serve --> browser_load: server ready
    browser_load --> pass: boot proof observed
    install --> capture_failure: install failed
    build --> capture_failure: build failed
    serve --> capture_failure: serve failed
    browser_load --> capture_failure: browser failed
```
## Interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: jet-production-build-regression-interaction
actors:
  - { id: test_runner, kind: actor }
  - { id: fixture, kind: participant }
  - { id: jet_cli, kind: system }
  - { id: browser_smoke, kind: participant }
  - { id: artifact_store, kind: participant }
messages:
  - { from: test_runner, to: fixture, name: "copy representative production fixture", returns: "isolated temp project" }
  - { from: test_runner, to: jet_cli, name: "jet install --frozen-lockfile", returns: "install log" }
  - { from: test_runner, to: jet_cli, name: "jet build", returns: "dist output and build log" }
  - { from: test_runner, to: browser_smoke, name: "load built output", returns: "text/state/console evidence" }
  - { from: test_runner, to: artifact_store, name: "persist failure evidence", returns: "actionable artifact paths" }
---
sequenceDiagram
    actor test_runner
    participant fixture
    participant jet_cli
    participant browser_smoke
    participant artifact_store
    test_runner->>fixture: copy representative production fixture
    fixture-->>test_runner: isolated temp project
    test_runner->>jet_cli: jet install --frozen-lockfile
    jet_cli-->>test_runner: install log
    test_runner->>jet_cli: jet build
    jet_cli-->>test_runner: dist output and build log
    test_runner->>browser_smoke: load built output
    browser_smoke-->>test_runner: text/state/console evidence
    test_runner->>artifact_store: persist failure evidence
    artifact_store-->>test_runner: actionable artifact paths
```
## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-production-build-regression-logic
entry: start
nodes:
  start: { kind: start, label: "start regression gate" }
  fixture: { kind: process, label: "prepare production fixture temp project" }
  install: { kind: process, label: "run jet install --frozen-lockfile" }
  install_ok: { kind: decision, label: "install passed" }
  build: { kind: process, label: "run jet build" }
  build_ok: { kind: decision, label: "build passed" }
  browser: { kind: process, label: "browser-load dist output" }
  browser_ok: { kind: decision, label: "boot proof observed" }
  artifacts: { kind: process, label: "persist phase-specific evidence" }
  pass: { kind: terminal, label: "gate pass" }
  fail: { kind: terminal, label: "gate fail with named phase" }
edges:
  - { from: start, to: fixture }
  - { from: fixture, to: install }
  - { from: install, to: install_ok }
  - { from: install_ok, to: build, label: "yes" }
  - { from: install_ok, to: artifacts, label: "no" }
  - { from: build, to: build_ok }
  - { from: build_ok, to: browser, label: "yes" }
  - { from: build_ok, to: artifacts, label: "no" }
  - { from: browser, to: browser_ok }
  - { from: browser_ok, to: pass, label: "yes" }
  - { from: browser_ok, to: artifacts, label: "no" }
  - { from: artifacts, to: fail }
---
flowchart TD
    start([start regression gate]) --> fixture[prepare production fixture temp project]
    fixture --> install[run jet install --frozen-lockfile]
    install --> install_ok{install passed}
    install_ok -- yes --> build[run jet build]
    install_ok -- no --> artifacts[persist phase-specific evidence]
    build --> build_ok{build passed}
    build_ok -- yes --> browser[browser-load dist output]
    build_ok -- no --> artifacts
    browser --> browser_ok{boot proof observed}
    browser_ok -- yes --> pass([gate pass])
    browser_ok -- no --> artifacts
    artifacts --> fail([gate fail with named phase])
```
## Dependency
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: jet-production-build-regression-dependency
---
classDiagram
    class ProductionBuildFixture {
      package_json
      jet_lock
      src_entry
      style_asset_inputs
    }
    class JetInstallPhase {
      frozen_lockfile
      package_store
      install_log
    }
    class JetBuildPhase {
      production_bundle
      dist_index
      build_log
    }
    class BrowserBootPhase {
      loaded_url
      console_events
      visible_text
      screenshot
    }
    class RegressionEvidence {
      phase
      command_log
      browser_artifacts
    }
    ProductionBuildFixture --> JetInstallPhase
    JetInstallPhase --> JetBuildPhase
    JetBuildPhase --> BrowserBootPhase
    JetInstallPhase --> RegressionEvidence
    JetBuildPhase --> RegressionEvidence
    BrowserBootPhase --> RegressionEvidence
```
## Data Model
<!-- type: db-model lang: mermaid -->

```mermaid
---
id: jet-production-build-regression-data-model
not_applicable:
  reason: "The production build regression gate does not introduce persistent database tables or storage models."
---
erDiagram
    NO_DATABASE_MODEL {
      string reason
    }
```
## Schema
<!-- type: schema lang: yaml -->

```yaml
production_build_regression_evidence:
  phase: "install|build|serve|browser"
  fixture: "fixture name or temp project path"
  command:
    argv: "command that ran"
    exit_code: "integer or null"
    stdout_path: "path to captured stdout"
    stderr_path: "path to captured stderr"
  browser:
    url: "loaded production output URL"
    console_log_path: "path to browser console log"
    screenshot_path: "path to failure screenshot when available"
    visible_text: "text proof used by the assertion"
  artifacts:
    dist_path: "built output path"
    report_dir: "failure artifact directory"
```
## REST API
<!-- type: rest-api lang: yaml -->

```yaml
not_applicable:
  reason: "The production build regression gate does not introduce HTTP REST endpoints."
```
## RPC API
<!-- type: rpc-api lang: yaml -->

```yaml
not_applicable:
  reason: "The production build regression gate does not introduce RPC methods or service contracts."
```
## Async API
<!-- type: async-api lang: yaml -->

```yaml
not_applicable:
  reason: "The production build regression gate does not introduce pub-sub, WebSocket, or background protocol contracts."
```
## CLI
<!-- type: cli lang: yaml -->

```yaml
commands_under_test:
  - command: "jet install --frozen-lockfile"
    phase: install
    expectation: "hydrates the fixture dependencies from the lockfile without mutating package metadata"
  - command: "jet build"
    phase: build
    expectation: "produces production dist output for the representative fixture"
  - command: "cargo test -p jet production_build_regression -- --nocapture"
    phase: verification
    expectation: "runs the full install/build/browser-load regression gate with actionable failure artifacts"
```
## Wireframe
<!-- type: wireframe lang: yaml -->

```yaml
not_applicable:
  reason: "The production build regression gate is a Rust/CLI test artifact and does not introduce a UI layout."
```
## Component
<!-- type: component lang: yaml -->

```yaml
not_applicable:
  reason: "The change adds a regression fixture/test, not reusable UI components."
```
## Design Token
<!-- type: design-token lang: yaml -->

```yaml
not_applicable:
  reason: "The regression gate does not add or modify visual design tokens."
```
## Config
<!-- type: config lang: yaml -->

```yaml
config_surfaces:
  - path: "fixture jet.config.toml or default Jet build config"
    purpose: "exercise production build defaults and output directory behavior"
  - path: "fixture package manager lock/config inputs"
    purpose: "prove `jet install --frozen-lockfile` can prepare the build without package metadata mutation"
```
## Manifest
<!-- type: manifest lang: yaml -->

```yaml
fixture_manifests:
  - path: "package.json"
    covers:
      - "React and MUI/Emotion dependencies"
      - "CJS dependency imported by the app"
      - "ESM package subpath import"
      - "extensionless package directory import"
  - path: "jet lockfile"
    covers:
      - "frozen install reproducibility before production build"
```
## Runtime Image
<!-- type: runtime-image lang: yaml -->

```yaml
not_applicable:
  reason: "The regression gate does not define or build container/runtime images."
```
## Deployment
<!-- type: deployment lang: yaml -->

```yaml
not_applicable:
  reason: "The regression gate validates local Jet production output and does not introduce deployment manifests or rollout steps."
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: jet-production-build-regression-unit-test
requirements:
  R1:
    text: "The regression helper records install/build/browser phases separately."
    risk: high
    verify: unit
  R2:
    text: "Failure evidence names the failing phase and artifact directory."
    risk: high
    verify: unit
---
requirementDiagram
requirement R1 {
  id: R1
  text: "Phase-specific regression helper"
  risk: High
  verifymethod: Test
}
requirement R2 {
  id: R2
  text: "Actionable failure evidence"
  risk: High
  verifymethod: Test
}
```
## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: production_build_fixture_boots_in_browser
    name: "Production build fixture boots in browser"
    command: "cargo test -p jet production_build_regression -- --nocapture"
    fixture: "representative React/MUI production fixture"
    verifies:
      - "jet install --frozen-lockfile succeeds"
      - "jet build produces dist output"
      - "built output browser-loads with expected visible text and no boot console errors"
      - "failure artifacts identify install, build, serve, or browser phase"
```

# Reviews

### Review 1
**Verdict:** approved

- [scenarios] Contract directly matches #4128 acceptance: frozen install, production build, browser boot, and actionable artifacts.
- [cli/e2e-test] Verification command is concrete and machine-checkable.
- [schema/logic] Failure evidence schema and phase routing are enough for implementation.
- [scope] No API, UI, deployment, or runtime-image work is introduced.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/jet/tests/build/production_build_regression.rs
    action: create
    section: e2e-test
    impl_mode: hand-written
    description: "Add the production build regression integration test that prepares the fixture, runs Jet install/build, browser-loads dist output, and emits phase-specific artifacts."
  - path: projects/jet/tests/fixtures/production-build-regression/package.json
    action: create
    section: manifest
    impl_mode: hand-written
    description: "Define the representative React/MUI production fixture dependency manifest."
  - path: projects/jet/tests/fixtures/production-build-regression/src/main.tsx
    action: create
    section: manifest
    impl_mode: hand-written
    description: "Add fixture source importing React, MUI/Emotion, package subpaths, CJS/ESM shapes, styles, and assets."
  - path: projects/jet/tests/fixtures/production-build-regression/src/style.css
    action: create
    section: manifest
    impl_mode: hand-written
    description: "Add fixture style input used to prove production CSS/style injection survives build output boot."
  - path: projects/jet/tests/fixtures/production-build-regression/src/message.cjs
    action: create
    section: manifest
    impl_mode: hand-written
    description: "Add local CJS fixture module used by the production build regression app."
```
