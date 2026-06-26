---
id: jet-complete-package-management-replacement-gate-before-build
fill_sections: [scenarios, mindmap, state-machine, interaction, logic, dependency, db-model, schema, rest-api, rpc-api, async-api, cli, wireframe, component, design-token, config, manifest, runtime-image, deployment, unit-test, e2e-test, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "WI #4160: Jet package management must fully replace pnpm before Browser Bridge expansion and DOM production build claims."
---

# Complete Package Management Replacement Gate Before Build

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: S1
    requirement: R1
    title: Jet-only source fixture hydration
    given: DOM production fixture source trees contain package.json and jet-lock.yaml
    when: compare-pkg-management runs with hydration enabled
    then: source fixtures are hydrated only by jet install --frozen-lockfile and lockfiles remain unchanged
  - id: S2
    requirement: R2
    title: npm and pnpm are isolated baselines only
    given: npm and pnpm baselines are requested
    when: package comparator executes incumbent package managers
    then: npm and pnpm commands run only in temporary benchmark copies and are excluded from Jet executor commands
  - id: S3
    requirement: R3
    title: Jet install reports cold warm and fast-path evidence
    given: a package fixture is copied for Jet benchmarking
    when: Jet performs cold install followed by warm frozen install
    then: evidence records cold duration, warm duration, installed bytes, and cache/fast-path signals proving warm behavior is not a blind reinstall
  - id: S4
    requirement: R4
    title: Package gate covers workspace and lifecycle/bin-heavy layouts
    given: package-management contract fixtures include workspace and lifecycle/bin-heavy packages
    when: the package phase gate runs
    then: Jet proves workspace linking, frozen drift rejection, lifecycle/bin behavior, and package surface preservation
  - id: S5
    requirement: R5
    title: Basic DOM phase order remains package then browser then build
    given: CI and verify-basic-dom-gates orchestrate the Basic DOM gate
    when: the gate is inspected or run
    then: package-management remains phase 1, Browser Bridge remains phase 2, and DOM production build remains dependent phase 3
  - id: S6
    requirement: R2
    title: npm ci is never accepted as a Jet package path
    given: package scripts, fixtures, and CI are scanned
    when: package evidence is generated
    then: no npm ci command is present anywhere in package evidence or Jet executor policy
```
## Mindmap
<!-- type: mindmap lang: mermaid -->

```mermaid
---
id: jet-package-management-replacement-gate
---
mindmap
  root((Jet package management replacement))
    "Jet executor"
      "source fixtures use jet install --frozen-lockfile"
      "lockfiles remain unchanged"
      "mutation contract uses jet add update audit remove"
    "Incumbent baselines"
      "npm benchmark copy only"
      "pnpm provisioned by jet install"
      "no npm ci"
    "Install maturity evidence"
      "cold install duration"
      "warm install duration"
      "installed bytes"
      "cache hit or fast-path signal"
    "Fixture breadth"
      "DOM production libraries"
      "workspace protocol"
      "lifecycle and bin-heavy packages"
    "Gate ordering"
      "phase 1 package"
      "phase 2 Browser Bridge"
      "phase 3 production build"
```
## State Machine
<!-- type: state-machine lang: mermaid -->

```mermaid
---
id: jet-package-gate-state-machine
initial: start
nodes:
  start: { kind: initial, label: "Start package gate" }
  hydrate_source_fixtures: { kind: normal, label: "Hydrate source fixtures with Jet" }
  inspect_jet_hydration: { kind: normal, label: "Inspect Jet hydration state" }
  run_incumbent_baselines: { kind: normal, label: "Run isolated npm/pnpm baselines" }
  run_jet_benchmark_copy: { kind: normal, label: "Run Jet cold/warm benchmark copy" }
  collect_install_maturity: { kind: normal, label: "Collect bytes/cache/fast-path evidence" }
  run_mutation_contract: { kind: normal, label: "Run mutation contract" }
  run_workspace_contract: { kind: normal, label: "Run workspace contract" }
  evaluate_executor_policy: { kind: normal, label: "Evaluate executor policy" }
  evaluate_phase_one_checks: { kind: normal, label: "Evaluate phase-one checks" }
  green: { kind: terminal, label: "Green package gate" }
  red: { kind: terminal, label: "Red package gate" }
edges:
  - { from: start, to: hydrate_source_fixtures, event: "gate starts" }
  - { from: hydrate_source_fixtures, to: inspect_jet_hydration, event: "source installs finish" }
  - { from: inspect_jet_hydration, to: run_incumbent_baselines, event: "Jet hydration is inspectable" }
  - { from: run_incumbent_baselines, to: run_jet_benchmark_copy, event: "baselines finish" }
  - { from: run_jet_benchmark_copy, to: collect_install_maturity, event: "Jet benchmark finishes" }
  - { from: collect_install_maturity, to: run_mutation_contract, event: "maturity evidence collected" }
  - { from: run_mutation_contract, to: run_workspace_contract, event: "mutation contract finishes" }
  - { from: run_workspace_contract, to: evaluate_executor_policy, event: "workspace contract finishes" }
  - { from: evaluate_executor_policy, to: evaluate_phase_one_checks, event: "policy evidence ready" }
  - { from: evaluate_phase_one_checks, to: green, event: "all required checks pass" }
  - { from: evaluate_phase_one_checks, to: red, event: "any required check fails" }
---
stateDiagram-v2
    [*] --> hydrate_source_fixtures
    hydrate_source_fixtures --> inspect_jet_hydration
    inspect_jet_hydration --> run_incumbent_baselines
    run_incumbent_baselines --> run_jet_benchmark_copy
    run_jet_benchmark_copy --> collect_install_maturity
    collect_install_maturity --> run_mutation_contract
    run_mutation_contract --> run_workspace_contract
    run_workspace_contract --> evaluate_executor_policy
    evaluate_executor_policy --> evaluate_phase_one_checks
    evaluate_phase_one_checks --> green : all required checks pass
    evaluate_phase_one_checks --> red : any required check fails
```
## Interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: jet-package-gate-interaction
actors:
  - { id: gate, kind: system }
  - { id: comparator, kind: system }
  - { id: jet, kind: system }
  - { id: baseline, kind: system }
  - { id: evidence, kind: system }
messages:
  - { from: gate, to: comparator, name: "Run phase package", returns: "package gate result" }
  - { from: comparator, to: jet, name: "Hydrate source fixture", returns: "install output and node_modules state" }
  - { from: comparator, to: baseline, name: "Run isolated npm/pnpm baselines", returns: "baseline timing and disk stats" }
  - { from: comparator, to: jet, name: "Run Jet cold/warm benchmark installs", returns: "duration, bytes, cache or fast-path signals" }
  - { from: comparator, to: evidence, name: "Write phase-one checks", returns: "pkg-management-compare.json" }
---
sequenceDiagram
    participant Gate as verify-basic-dom-gates.sh
    participant Comparator as compare-pkg-management.mjs
    participant Jet as jet install
    participant Baseline as npm/pnpm baseline copies
    participant Evidence as pkg-management-compare.json

    Gate->>Comparator: run phase package
    Comparator->>Jet: hydrate source fixture with frozen lockfile
    Jet-->>Comparator: install output and node_modules state
    Comparator->>Baseline: run npm/pnpm only in temp copies
    Baseline-->>Comparator: baseline timing and disk stats
    Comparator->>Jet: run cold and warm Jet benchmark installs
    Jet-->>Comparator: duration, bytes, cache or fast-path signals
    Comparator->>Comparator: run mutation and workspace contracts
    Comparator->>Evidence: write phase-1 checks and failures
    Evidence-->>Gate: green or red gate result
```
## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-package-gate-logic
entry: start
nodes:
  start: { kind: start, label: "Start package comparator" }
  hydrate: { kind: process, label: "Run Jet frozen install on source fixture" }
  inspect: { kind: process, label: "Inspect direct deps, bin links, layout residue, and lock hash" }
  baseline: { kind: process, label: "Run npm/pnpm in isolated copies" }
  benchmark: { kind: process, label: "Run Jet cold/warm benchmark copy" }
  maturity: { kind: process, label: "Extract bytes/cache/fast-path evidence" }
  contracts: { kind: process, label: "Run mutation and workspace contracts" }
  policy: { kind: decision, label: "Any forbidden package manager executor or npm ci?" }
  required: { kind: decision, label: "All required checks green?" }
  green: { kind: terminal, label: "Green evidence" }
  red: { kind: terminal, label: "Red evidence" }
edges:
  - { from: start, to: hydrate }
  - { from: hydrate, to: inspect }
  - { from: inspect, to: baseline }
  - { from: baseline, to: benchmark }
  - { from: benchmark, to: maturity }
  - { from: maturity, to: contracts }
  - { from: contracts, to: policy }
  - { from: policy, to: red, label: "yes" }
  - { from: policy, to: required, label: "no" }
  - { from: required, to: green, label: "yes" }
  - { from: required, to: red, label: "no" }
---
flowchart TD
    start([Start package comparator]) --> hydrate[Run Jet frozen install on source fixture]
    hydrate --> inspect[Inspect direct deps, bin links, layout residue, and lock hash]
    inspect --> baseline[Run npm/pnpm in isolated copies]
    baseline --> benchmark[Run Jet cold/warm benchmark copy]
    benchmark --> maturity[Extract bytes/cache/fast-path evidence]
    maturity --> contracts[Run mutation and workspace contracts]
    contracts --> policy{Any forbidden package manager executor or npm ci?}
    policy -->|yes| red([Red evidence])
    policy -->|no| required{All required checks green?}
    required -->|yes| green([Green evidence])
    required -->|no| red
```
## Dependency
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: jet-package-gate-dependency
---
classDiagram
    class VerifyBasicDomGates {
      phase package
    }
    class ComparePkgManagement {
      inspectFixture()
      runJetInstallBenchmark()
      runMutationContract()
      runWorkspaceContract()
    }
    class JetCli {
      install
      add
      update
      audit
      remove
    }
    class IncumbentBaselines {
      npmTempCopy
      pnpmProvisionedByJet
    }
    class PackageEvidence {
      pkg_management_compare_json
      phaseOneChecks
    }
    class BasicDomCi {
      packageManagementJob
      browserBridgeJob
      domProductionBuildJob
    }
    VerifyBasicDomGates --> ComparePkgManagement
    ComparePkgManagement --> JetCli
    ComparePkgManagement --> IncumbentBaselines
    ComparePkgManagement --> PackageEvidence
    BasicDomCi --> VerifyBasicDomGates
```
## Data Model
<!-- type: db-model lang: mermaid -->

```mermaid
---
id: jet-package-gate-data-model
---
erDiagram
    PACKAGE_GATE_EVIDENCE {
      string contract_id
      string result
      int phase
    }
    FIXTURE_EVIDENCE {
      string fixture
      string result
      int direct_dependency_count
      int bin_link_count
    }
    INSTALL_MATURITY {
      float cold_install_ms
      float warm_install_ms
      int installed_bytes
      string cache_signal
    }
    BASELINE_BENCHMARK {
      string tool
      string command_policy
      string result
    }
    PACKAGE_GATE_EVIDENCE ||--o{ FIXTURE_EVIDENCE : includes
    FIXTURE_EVIDENCE ||--|| INSTALL_MATURITY : records
    FIXTURE_EVIDENCE ||--o{ BASELINE_BENCHMARK : compares
```
## Schema
<!-- type: schema lang: yaml -->

```yaml
package_gate_evidence:
  contract_id: basic.install.replacement
  phase: 1
  checks:
    - no_npm_pnpm_yarn_bun_executor_commands
    - no_npm_ci_anywhere
    - required_baseline_benchmarks_green
    - required_baseline_performance_green
    - install_maturity_evidence_present
    - package_contract_fixture_breadth_present
  fixture:
    required_fields:
      - fixture
      - result
      - commands
      - jet_install_maturity
      - baseline_benchmarks
  jet_install_maturity:
    required_fields:
      - cold_install_ms
      - warm_install_ms
      - installed_bytes
      - cache_or_fast_path_signal
```
## REST API
<!-- type: rest-api lang: yaml -->

```yaml
not_applicable:
  reason: "The package-management replacement gate is a local CLI/evidence contract and does not introduce HTTP REST endpoints."
```
## RPC API
<!-- type: rpc-api lang: yaml -->

```yaml
not_applicable:
  reason: "The package-management replacement gate does not introduce RPC contracts."
```
## Async API
<!-- type: async-api lang: yaml -->

```yaml
not_applicable:
  reason: "The package-management replacement gate does not introduce pub-sub, queue, or WebSocket contracts."
```
## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: package_phase_gate
    command: "projects/jet/scripts/verify-basic-dom-gates.sh --phase package"
    verifies:
      - "Jet package-management replacement phase only"
      - "npm/pnpm remain isolated baselines"
      - "package evidence includes maturity and fixture breadth checks"
  - name: package_comparator
    command: "node projects/jet/scripts/compare-pkg-management.mjs --jet-bin target/release/jet"
    verifies:
      - "source fixture hydration"
      - "Jet benchmark installs"
      - "mutation and workspace contracts"
```
## Wireframe
<!-- type: wireframe lang: yaml -->

```yaml
not_applicable:
  reason: "The package-management replacement gate is CLI and JSON evidence only; it does not introduce UI."
```
## Component
<!-- type: component lang: yaml -->

```yaml
not_applicable:
  reason: "The package-management replacement gate does not introduce frontend components."
```
## Design Token
<!-- type: design-token lang: yaml -->

```yaml
not_applicable:
  reason: "The package-management replacement gate does not introduce design tokens."
```
## Config
<!-- type: config lang: yaml -->

```yaml
config_surfaces:
  - name: JET_BASIC_DOM_PHASES
    role: "allows selecting package phase without build"
  - name: JET_BASIC_DOM_PACKAGE_BASELINES
    role: "selects npm/pnpm isolated baseline tools"
  - name: JET_BASIC_DOM_REQUIRE_BASELINES
    role: "keeps baselines blocking when required"
  - name: JET_BASIC_DOM_COMMAND_TIMEOUT_MS
    role: "bounds install and comparator child commands"
```
## Manifest
<!-- type: manifest lang: yaml -->

```yaml
manifest_surfaces:
  - package.json
  - jet-lock.yaml
  - package-lock.json
  - pnpm-lock.yaml
  - .github/workflows/jet-basic-dom.yml
policy:
  source_fixture_manager: "jet install --frozen-lockfile"
  incumbent_managers: "read-only oracle and isolated benchmark only"
```
## Runtime Image
<!-- type: runtime-image lang: yaml -->

```yaml
not_applicable:
  reason: "The package-management replacement gate does not introduce container or runtime images."
```
## Deployment
<!-- type: deployment lang: yaml -->

```yaml
not_applicable:
  reason: "The package-management replacement gate updates local/CI verification only and does not introduce deployment manifests."
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: jet-package-gate-unit-test
requirements:
  R1:
    text: "Package comparator records Jet install maturity evidence."
    risk: high
    verify: unit
  R2:
    text: "Package comparator keeps npm/pnpm out of executor commands."
    risk: high
    verify: unit
  R3:
    text: "Package gate proves workspace and lifecycle/bin-heavy contracts."
    risk: high
    verify: command
---
requirementDiagram
requirement R1 {
  id: R1
  text: "Install maturity evidence"
  risk: High
  verifymethod: Test
}
requirement R2 {
  id: R2
  text: "Baseline isolation"
  risk: High
  verifymethod: Test
}
requirement R3 {
  id: R3
  text: "Package fixture breadth"
  risk: High
  verifymethod: Test
}
```
## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: package_phase_gate
    name: "Basic DOM package-management replacement gate"
    command: "projects/jet/scripts/verify-basic-dom-gates.sh --phase package"
    verifies:
      - "Jet source fixture hydration"
      - "npm/pnpm baseline isolation"
      - "install maturity evidence"
      - "workspace and lifecycle/bin-heavy package contract breadth"
      - "no npm ci"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/scripts/compare-pkg-management.mjs"
    action: modify
    section: cli
    description: |
      Add phase-1 package-management replacement evidence: cold/warm Jet
      install maturity, installed bytes, cache-or-fast-path signals,
      workspace contract, mutation contract, bin-heavy fixture breadth, and
      incumbent package-manager executor guards.
    impl_mode: hand-written
  - path: "projects/jet/src/pkg_manager/store.rs"
    action: modify
    section: logic
    description: |
      Fix package store cache validation and tarball root normalization so
      scoped packages such as @types/react hydrate root node_modules links with
      a real package.json instead of a symlink to a nested top-level directory.
      Generate fixture-local Node bin shims so package CLIs such as webpack can
      resolve sibling CLI packages from the project node_modules graph instead
      of from the global Jet store realpath. These are required for the phase-1
      fixture contract to pass from a clean dependency tree rather than relying
      on pre-existing node_modules.
    impl_mode: hand-written
  - path: "projects/jet/src/pkg_manager/mod.rs"
    action: modify
    section: logic
    description: |
      Make workspace installs honor frozen lockfile deps-hash drift checks and
      make jet remove prune its lockfile/node_modules/bin-shim surface so the
      mutation and workspace package-manager contracts can replace pnpm's
      install lifecycle guarantees.
    impl_mode: hand-written
  - path: "projects/jet/src/pkg_manager/lockfile.rs"
    action: modify
    section: unit-test
    description: |
      Align lockfile hydration test fixtures with the stricter store invariant
      that a package cache hit must include a package.json at the store root.
    impl_mode: hand-written
  - path: "projects/jet/scripts/verify-basic-dom-gates.sh"
    action: modify
    section: cli
    description: |
      Keep Basic DOM gates explicitly phased as package first, Browser Bridge
      second, and production build third. Route phase 1 through the
      package-management replacement comparator.
    impl_mode: hand-written
  - path: "projects/jet/scripts/verify-browser-bridge-replacement.mjs"
    action: create
    section: cli
    description: |
      Provide the phase-2 Browser Bridge replacement gate referenced by the
      Basic DOM phase orchestrator and CI ordering. This keeps phase 2 explicit
      without allowing Playwright to become the Jet executor.
    impl_mode: hand-written
  - path: "projects/jet/scripts/compare-dom-build-corpus.mjs"
    action: create
    section: cli
    description: |
      Provide the phase-3 DOM production build corpus comparator referenced by
      the Basic DOM phase orchestrator and CI ordering. This gate remains behind
      package-management and Browser Bridge replacement.
    impl_mode: hand-written
  - path: "projects/jet/tests/fixtures/dom-production-build"
    action: create
    section: manifest
    description: |
      Add the tests-fixture corpus used by package-management and later
      production-build gates, including React bench, MUI, AntD, Tailwind, and
      styled-components app shapes with Jet lockfiles and baseline oracle
      metadata where applicable.
    impl_mode: hand-written
  - path: ".github/workflows/jet-basic-dom.yml"
    action: modify
    section: deployment
    description: |
      Mirror the Basic DOM phase order in CI so package-management replacement
      gates run before Browser Bridge and production build comparisons.
    impl_mode: hand-written
  - path: "projects/jet/README.md"
    action: modify
    section: manifest
    description: |
      Document that Basic replacement order is package management, then Browser
      Bridge, then production build, and that npm/pnpm/Playwright are isolated
      baselines rather than Jet executors.
    impl_mode: hand-written
  - path: "projects/jet/scripts/compare-pkg-management.mjs"
    action: annotate
    section: scenarios
    description: |
      Own the Jet-only package fixture hydration, isolated baseline, install
      maturity, fixture breadth, phase-order, and npm-ci exclusion scenarios.
    impl_mode: hand-written
  - path: "projects/jet/scripts/compare-pkg-management.mjs"
    action: annotate
    section: mindmap
    description: |
      Own the package-management replacement decomposition across Jet executor,
      incumbent baselines, install evidence, fixture breadth, and gate ordering.
    impl_mode: hand-written
  - path: "projects/jet/scripts/compare-pkg-management.mjs"
    action: annotate
    section: state-machine
    description: |
      Own the package gate state progression from source fixture hydration
      through benchmark evidence, contracts, policy checks, and green/red result.
    impl_mode: hand-written
  - path: "projects/jet/scripts/compare-pkg-management.mjs"
    action: annotate
    section: interaction
    description: |
      Own the interaction between the Basic DOM gate, package comparator, Jet
      install, isolated baselines, and emitted JSON evidence.
    impl_mode: hand-written
  - path: "projects/jet/scripts/compare-pkg-management.mjs"
    action: annotate
    section: dependency
    description: |
      Own the dependency graph connecting the phase gate, comparator, Jet CLI,
      incumbent baselines, package evidence, and CI ordering.
    impl_mode: hand-written
  - path: "projects/jet/scripts/compare-pkg-management.mjs"
    action: annotate
    section: db-model
    description: |
      Own the local evidence data model for package gate results, fixture
      evidence, install maturity, and baseline benchmark records.
    impl_mode: hand-written
  - path: "projects/jet/scripts/compare-pkg-management.mjs"
    action: annotate
    section: schema
    description: |
      Own the package gate evidence schema, including required checks,
      per-fixture fields, and Jet install maturity fields.
    impl_mode: hand-written
  - path: "projects/jet/scripts/compare-pkg-management.mjs"
    action: annotate
    section: config
    description: |
      Own the package gate environment knobs for phase selection, baseline tool
      selection, required baselines, and command timeouts.
    impl_mode: hand-written
  - path: "projects/jet/README.md"
    action: annotate
    section: rest-api
    description: |
      Record that this package-management replacement gate does not introduce a
      REST API surface.
    impl_mode: hand-written
  - path: "projects/jet/README.md"
    action: annotate
    section: rpc-api
    description: |
      Record that this package-management replacement gate does not introduce an
      RPC surface.
    impl_mode: hand-written
  - path: "projects/jet/README.md"
    action: annotate
    section: async-api
    description: |
      Record that this package-management replacement gate does not introduce a
      pub-sub, queue, or WebSocket surface.
    impl_mode: hand-written
  - path: "projects/jet/README.md"
    action: annotate
    section: wireframe
    description: |
      Record that this package-management replacement gate is CLI and JSON
      evidence only, not an interactive UI flow.
    impl_mode: hand-written
  - path: "projects/jet/README.md"
    action: annotate
    section: component
    description: |
      Record that this package-management replacement gate does not introduce a
      frontend component surface.
    impl_mode: hand-written
  - path: "projects/jet/README.md"
    action: annotate
    section: design-token
    description: |
      Record that this package-management replacement gate does not introduce
      design tokens.
    impl_mode: hand-written
  - path: "projects/jet/README.md"
    action: annotate
    section: runtime-image
    description: |
      Record that this package-management replacement gate does not introduce a
      runtime image surface.
    impl_mode: hand-written
```

# Reviews

### Review 1
**Verdict:** approved

- [scenarios] Covers the phase-1 pivot explicitly: Jet owns source hydration, npm/pnpm are isolated baselines, and build remains phase 3.
- [schema/logic] Evidence requirements are machine-checkable: cold/warm duration, installed bytes, cache-or-fast-path signal, forbidden executor commands, and npm ci absence.
- [cli/e2e-test] The hard verification command is concrete: `projects/jet/scripts/verify-basic-dom-gates.sh --phase package`.
- [scope] API, UI, deployment, runtime-image, and WASM/build behavior are correctly out of scope for this phase-1 package-management change.

# Reviews

### Review 1
**Verdict:** approved

- [scenarios] Covers Jet-only fixture hydration, isolated npm/pnpm baselines, cold/warm install maturity evidence, workspace/lifecycle/bin-heavy fixture breadth, phase ordering, and npm ci exclusion.
- [schema] Defines machine-checkable JSON evidence for phase 1, including `install_maturity_evidence_present`, `package_contract_fixture_breadth_present`, and per-fixture `jet_install_maturity`.
- [cli] Keeps `verify-basic-dom-gates.sh --phase package` as the phase-1 entry point and keeps production build claims behind package and Browser Bridge gates.
- [unit-test] Requires pkg-manager unit coverage and script-level scans to reject npm/pnpm executor drift and npm ci regressions.
- [e2e-test] Requires the full package gate to produce green evidence across React bench, MUI, AntD, Tailwind, and styled-components fixtures with incumbent package managers used only as isolated baselines.
