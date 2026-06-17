# Jet

## Brief

Jet is a Rust-native frontend toolchain. Its Basic track replaces the usual
frontend stack around package management, builds, dev/prod serving, native
tests, product-flow e2e, browser automation, trace, and parity evidence. Its
Advanced track sinks frontend execution into Rust/WASM and renders through
canvas/WebGPU while preserving browser-observable behavior through Jet bridges.

Primary command: `jet <command>`. In integrated cclab environments,
`cclab jet <command>` may also be available.

Agent model:

| Track | Product role | Current read |
|---|---|---|
| Basic: FE-on-DOM | Replace today's Vite/Turbopack + pnpm/npm/Bun + Playwright-style toolchain. | Green for the current local Basic gate across package manager, Browser Bridge, production build, serve, workspace, test, e2e, and trace. |
| Advanced: FE-on-WASM | Run the frontend app model inside Jet's Rust/WASM runtime and render through WebGPU/canvas. | Yellow overall. Focused evidence exists, but broad DOM-vs-WASM parity is not production-ready. |

Readiness rules for agents:

- Treat Basic and Advanced as separate readiness tracks.
- Do not use Advanced WASM progress to qualify Basic production readiness.
- Do not treat `aw capability check --project jet` as runtime proof; it
  validates README capability structure and TD refs.
- Basic gates compare function first and performance second. A faster run is
  still red when output, install tree, server behavior, browser action, or trace
  evidence differs from the oracle.
- Jet-owned fixture hydration stays separate from npm/pnpm/Playwright
  comparator evidence.

Common surfaces:

| Surface | Commands | Owns |
|---|---|---|
| Package management | `jet install`, `jet add`, `jet remove`, `jet update`, `jet audit`, `jet run`, `jet exec` | Dependency lifecycle, lockfile, workspace, registry/cache, bin scripts, lifecycle hooks. |
| Build | `jet build`, `jet build --wasm` | DOM artifacts, WASM artifacts, static assets, build metadata, target manifest. |
| Dev and serve | `jet dev`, `jet dev --proxy PATH=URL`, `jet serve`, `jet serve --wasm` | HMR/dev control plane, detached sessions, production static origin serving. |
| Browser Bridge | `jet bb ...`, `jet bb mcp`; legacy `jet browser ...` | Browser automation, semantic snapshots, ref-based actions, console/network observability, DOM/WASM capture. |
| Test/e2e/trace | `jet test`, `jet e2e`, `jet trace` | Native TS tests, product-flow e2e, replayable diagnostics, parity evidence. |

First commands:

```bash
jet install
jet build
jet dev -p 3000
jet serve
jet bb launch <url>
jet bb snapshot
jet test
jet e2e
jet trace
```

Primary verification:

```bash
projects/jet/scripts/verify-basic-dom-gates.sh
JET_BASIC_DOM_BUILD_SAMPLES=3 JET_BASIC_DOM_RUNTIME_SMOKE=required projects/jet/scripts/verify-basic-dom-gates.sh --all
projects/jet/scripts/verify-advanced-wasm-gates.sh
aw capability check --project jet --pretty
aw health --project jet
```

Source map:

| Path | Read when |
|---|---|
| `projects/jet/LAYOUT.md` | You need the repo map before editing. |
| `projects/jet/src/pkg_manager/` | Package manager behavior, lockfile, registry, store, workspace, audit/publish flows. |
| `projects/jet/src/bundler/` | Dependency graph, tree shaking, CSS bundle, minification, splitting, sourcemaps. |
| `projects/jet/src/dev_server/` | Dev server, HMR, proxy, prod static serving, watcher, prebundle/import map behavior. |
| `projects/jet/src/browser/` and `projects/jet/src/browser_cli/` | Browser Bridge driver and CLI surfaces. |
| `projects/jet/src/test_runner/`, `projects/jet/src/e2e/`, `projects/jet/src/trace/` | Native test runtime, product-flow e2e, trace artifacts. |
| `projects/jet/src/wasm_build/` and `projects/jet/wasm/` | FE-on-WASM build path and runtime crate. |
| `projects/jet/parity/` | DOM/WASM parity corpus, oracle, gates, fixtures, schemas, ADRs. |
| `projects/jet/tests/` | Product and subsystem gates. |
| `.aw/tech-design/projects/jet/specs/3779.md` | Package-manager capability/spec entrypoint. |
| `.aw/tech-design/projects/jet/logic/pkg-manager.md` | Package-manager semantic/logic entrypoint. |

## Capabilities

From this point down is the AW-managed capability registry for Jet. Keep the
contract and work-root table schemas stable, keep enum values inside the
accepted AW vocabulary, and validate structural edits with
`aw capability check --project jet --pretty`.

Capability claims become meaningful only when their listed gate/evidence is
runnable in the current environment. Use
`aw capability report --project jet --verify` or `aw health --project jet` for
readiness questions.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Rust-Native Frontend Toolchain Replacement | #3778 | implemented | verified | smoke, conformance, corpus, negative, dogfood | ready_for_basic | The Basic all-in-one replacement gate is green across package manager, Browser Bridge, production build, serve, workspace, test, e2e, and trace. |
| Package Manager | #3779 | implemented | verified | smoke, conformance, corpus, negative | ready_for_basic | Jet owns fixture hydration and mutation gates; required isolated npm/pnpm benchmark evidence is green for the current Basic corpus. |
| Bundler And Production Build | #3782 | implemented | verified | smoke, conformance, corpus, negative | ready_for_basic | The expanded DOM production build corpus is green with required runtime smoke and Vite/Webpack comparisons. |
| Dev Server And HMR | #3780 | implemented | verified | smoke, conformance, corpus, negative, dogfood | ready | `jet dev` can replace Vite-style local development serving, HMR, browser log intake, and local API/WebSocket proxying for real projects. |
| Workspace And Task Runner | #3781 | implemented | verified | smoke, conformance, corpus, negative | ready | Jet can replace npm scripts, pnpm workspaces, and common Nx/Turborepo task-runner flows through the canonical `jet.toml` configuration surface. |
| Native Test And Product-Flow E2E | #3785 | implemented | verified | smoke, conformance, corpus, negative, dogfood | ready_for_basic | Jet native runner, reporter, product-flow e2e, and trace gates are green for the Basic production-readiness contract. |
| WASM And Multi-Target Execution | #3783 | implemented | passing | smoke, conformance, corpus, negative | partial | Jet can sink the frontend app model into WASM, render it through canvas/WebGPU, and preserve browser-observable semantics through bridges. |
| Browser, Trace, And Parity Infrastructure | #3786 | implemented | verified | smoke, conformance, corpus, negative | ready_for_basic | Jet BB is the executor for current gates, with isolated Playwright baseline evidence and trace substrate tests green. |

### Rust-Native Frontend Toolchain Replacement

ID: rust-native-frontend-toolchain
Type: DeveloperTool
Surfaces: CLI: `jet install` + `jet build` + `jet dev` + `jet test` - Aggregate frontend toolchain entrypoints for package, build, dev, and test workflows.
EC Dimensions: behavior: `projects/jet/scripts/verify-basic-dom-gates.sh --all` - Basic frontend replacement flow across package, build, dev, serve, workspace, test, e2e, and trace gates.
Root WI: #3778
Status: verified
Required Verification: smoke, conformance, corpus, negative, dogfood
Promise:
Jet is gated as an all-in-one Basic frontend replacement in dependency order:
package manager, Browser Bridge, production build, serve, workspace, test,
e2e, and trace. The current production-readiness gate is green.
Gate Inventory:
- `projects/jet/scripts/verify-basic-dom-gates.sh --all`
- projects/jet/tests/fixtures/dom-production-build

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Production replacement readiness | epic | #3778 | implemented | verified | corpus | package manager -> Browser Bridge -> build -> serve/workspace/test/e2e/trace are green in `verify-basic-dom-gates.sh --all` |
| Full Toolchain Dogfood Flow | epic | #3778 | implemented | verified | dogfood | `projects/jet/scripts/verify-basic-dom-gates.sh --all`<br>projects/jet/tests/fixtures/dom-production-build |

### Package Manager

ID: package-manager
Type: DeveloperTool
Surfaces: CLI: `jet install` + `jet add` + `jet remove` + `jet update` - Package lifecycle commands that own dependency and lockfile behavior.
EC Dimensions: behavior: `cargo test -p jet --lib pkg_manager -- --nocapture` - Package lifecycle, lockfile, workspace, registry, and negative-path conformance.
Root WI: #3779
Status: verified
Required Verification: smoke, conformance, corpus, negative
Promise:
Jet owns fixture hydration, mutation, workspace, and frozen-lockfile checks;
isolated npm/pnpm benchmark evidence is green for the current Basic corpus.
Gate Inventory:
- `cargo test -p jet --lib pkg_manager -- --nocapture`
- `projects/jet/scripts/compare-pkg-management.mjs --baseline-tools npm,pnpm --require-baselines`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Package manager readiness | epic | #3779 | implemented | verified | corpus | `projects/jet/scripts/compare-pkg-management.mjs` |
| Package Manager Lockfile Parity | epic | #3779 | implemented | verified | conformance | `cargo test -p jet --lib pkg_manager::lockfile -- --nocapture`<br>projects/jet/fixtures/pkg-manager/lockfile |
| Package Manager Workspace Parity | epic | #3779 | implemented | verified | conformance | `cargo test -p jet --lib pkg_manager::workspace -- --nocapture`<br>projects/jet/fixtures/pkg-manager/workspace |
| Package Manager Registry Integrity | epic | #3779 | implemented | verified | negative | `cargo test -p jet --lib pkg_manager -- --nocapture`<br>projects/jet/fixtures/pkg-manager/registry |

### Bundler And Production Build

ID: bundler-production-build
Type: DeveloperTool
Surfaces: CLI: `jet build` + `jet build --wasm` - Production and WASM build command surface.
EC Dimensions: behavior: `projects/jet/scripts/compare-dom-build-corpus.mjs --runtime-smoke required --build-samples 3` - DOM production build corpus, runtime smoke, and Vite/Webpack comparison behavior.
Root WI: #3782
Status: verified
Required Verification: smoke, conformance, corpus, negative
Promise:
Jet production build replacement is green after package manager and Browser
Bridge gates. The expanded DOM production build corpus has green static checks,
runtime smoke, and performance/size comparisons for the current fixture set.
Gate Inventory:
- `projects/jet/scripts/compare-dom-build-corpus.mjs --runtime-smoke required --build-samples 3`
- projects/jet/tests/fixtures/dom-production-build

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Bundler production readiness | epic | #3782 | implemented | verified | corpus | DOM production build corpus is green with required runtime smoke and Vite/Webpack comparisons |
| Production Bundle Output Parity | epic | #3782 | implemented | verified | conformance | `cargo test -p jet --lib bundler -- --nocapture`<br>projects/jet/fixtures/bundler/production |
| Transform Resolver Parity | epic | #3782 | implemented | verified | corpus | `cargo test -p jet --lib transform -- --nocapture`<br>projects/jet/fixtures/bundler/transform-resolver |
| Asset Sourcemap Negative Paths | epic | #3782 | implemented | verified | negative | `cargo test -p jet --lib asset -- --nocapture`<br>projects/jet/fixtures/bundler/assets |

### Dev Server And HMR

ID: dev-server-hmr
Type: DeveloperTool
Surfaces: CLI: `jet dev` + `jet dev --proxy PATH=URL` + `jet serve` - Dev server control plane, proxy/HMR entrypoints, and production static serving surface.; UI: `http://localhost:<port>` - Connected browser client for HMR, browser log intake, and local app inspection.
EC Dimensions: behavior: `cargo test -p jet --lib dev_server -- --nocapture` - Local serving, HMR, proxy, browser-log intake, and production static serving conformance.
Root WI: #3780
Status: verified
Required Verification: smoke, conformance, corpus, negative, dogfood
Promise:
`jet dev` can replace Vite-style local development serving and HMR for real
projects. Dev mode prioritizes a connected browser client, HMR, browser log
intake, and dev-only reverse proxy rules from `[dev.proxy]` in `jet.toml` or
repeatable `--proxy PATH=URL` CLI overrides. `jet serve` is a separate
Kubernetes/GKE static frontend data plane behind a load balancer, with
nginx-class static serving behavior and a hot path tuned for low memory-copy
overhead and high RPS; it does not own TLS termination, public virtual hosts,
cert management, WAF/CDN, or cross-service ingress routing. Current local proof
includes prod static serving versus nginx with first-byte p95 ratio `0.803` and
throughput ratio `1.164`.
Gate Inventory:
- projects/jet/fixtures/dev-server/basic-hmr
- projects/jet/fixtures/dev-server/react-refresh/state-preserved
- projects/jet/fixtures/dev-server/prebundle-importmap
- `cargo test -p jet --lib dev_server -- --nocapture`
- `cargo test -p jet --lib dev_server::proxy -- --nocapture`
- `cargo test -p jet --lib cli::e2e_command_contract_tests -- --nocapture`
- `projects/jet/scripts/compare-prod-static-serve.mjs --jet-bin target/release/jet`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Dev server replacement readiness | epic | #3780 | implemented | verified | dogfood | jet dev is the client-connected dev control plane; jet serve is the static data plane |
| Dev Server Local Serving Hmr | epic | #3780 | implemented | verified | conformance | `cargo test -p jet --lib dev_server -- --nocapture`<br>projects/jet/fixtures/dev-server/basic-hmr |
| Dev Server Proxy Contract | epic | #3780 | implemented | verified | conformance | `cargo test -p jet --lib dev_server::proxy -- --nocapture` |
| Dev Server Cli Contract | epic | #3780 | implemented | verified | conformance | `cargo test -p jet --lib cli::e2e_command_contract_tests -- --nocapture` |
| React Refresh State Preserved | epic | #3780 | implemented | verified | conformance | `cargo test -p jet --lib dev_server::hmr -- --nocapture`<br>projects/jet/fixtures/dev-server/react-refresh/state-preserved |
| Prebundle Importmap Parity | epic | #3780 | implemented | verified | corpus | `cargo test -p jet --lib dev_server::prebundle -- --nocapture`<br>projects/jet/fixtures/dev-server/prebundle-importmap |

### Workspace And Task Runner

ID: workspace-task-runner
Type: DeveloperTool
Surfaces: CLI: `jet run` + `jet exec` - Workspace script and binary execution surface.
EC Dimensions: behavior: `cargo test -p jet --lib task_runner -- --nocapture` - Workspace script execution, graph cache, package selection, and task-runner parity behavior.
Root WI: #3781
Status: verified
Required Verification: smoke, conformance, corpus, negative
Promise:
Jet workspace/task-runner replacement remains part of the package-management
replacement track before build claims. The canonical project configuration file
is `jet.toml`, and the active schema artifact is `schemas/jet.schema.json`.
Gate Inventory:
- projects/jet/fixtures/task-runner/graph-cache
- projects/jet/fixtures/workspace/package-selection
- projects/jet/fixtures/task-runner/nx
- `cargo test -p jet --lib task_runner -- --nocapture`
- `cargo test -p jet --lib task_runner::config::tests -- --nocapture`
- `cargo run -p jet -- config schema --check`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Workspace task runner readiness | epic | #3781 | implemented | verified | corpus | package/workspace replacement track plus canonical jet.toml config, lint, and schemas/jet.schema.json schema artifact |
| Task Runner Graph Cache | epic | #3781 | implemented | verified | conformance | `cargo test -p jet --lib task_runner -- --nocapture`<br>projects/jet/fixtures/task-runner/graph-cache |
| Workspace Package Selection | epic | #3781 | implemented | verified | conformance | `cargo test -p jet --lib pkg_manager::workspace -- --nocapture`<br>projects/jet/fixtures/workspace/package-selection |
| Nx Graph Parity | epic | #3781 | implemented | verified | corpus | `cargo test -p jet --lib pkg_manager::nx -- --nocapture`<br>projects/jet/fixtures/task-runner/nx |

### Native Test And Product-Flow E2E

ID: native-test-product-flow-e2e
Type: DeveloperTool
Surfaces: CLI: `jet test` + `jet e2e` - Native test runner and product-flow e2e surface.; WebAppE2E: `jet e2e` - Browser-driven product-flow verification for frontend behavior across app and API boundaries.
EC Dimensions: behavior: `jet e2e` - Browser-driven product-flow verification across frontend behavior and app/API boundaries.
Root WI: #3785
Status: verified
Required Verification: smoke, conformance, corpus, negative, dogfood
Promise:
Jet native tests, reporter artifacts, product-flow e2e, and trace evidence are
green in the Basic production-readiness gate.
Gate Inventory:
- projects/jet/fixtures/test-runner/native
- projects/jet/fixtures/test-runner/reporters
- projects/jet/fixtures/e2e/product-flow
- projects/jet/fixtures/e2e/trace-replay
- `cargo test -p jet --lib test_runner -- --nocapture`
- `cargo test -p jet --lib reporter -- --nocapture`
- `cargo test -p jet --lib e2e -- --nocapture`
- `cargo test -p jet --lib trace -- --nocapture`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Native test runner readiness | epic | #3785 | implemented | verified | none | - |
| Product flow e2e readiness | epic | #3784 | implemented | verified | dogfood | Browser Bridge replacement gate plus `cargo test -p jet --lib e2e -- --nocapture` |
| Native Test Runner Core | epic | #3785 | implemented | verified | conformance | `cargo test -p jet --lib test_runner -- --nocapture` |
| Built In Ts Test Runtime | epic | #3785 | implemented | verified | conformance | `cargo test -p jet --lib test_runner -- --nocapture`<br>projects/jet/fixtures/test-runner/native |
| Reporter Artifacts | epic | #3785 | implemented | verified | negative | `cargo test -p jet --lib reporter -- --nocapture`<br>projects/jet/fixtures/test-runner/reporters |
| Product Flow E2e Review | epic | #3785 | implemented | verified | dogfood | `cargo test -p jet --lib e2e -- --nocapture`<br>projects/jet/fixtures/e2e/product-flow |
| Trace Replay Evidence | epic | #3785 | implemented | verified | conformance | `cargo test -p jet --lib trace -- --nocapture`<br>projects/jet/fixtures/e2e/trace-replay |

### WASM And Multi-Target Execution

ID: wasm-multi-target
Type: DeveloperTool
Surfaces: CLI: `jet build --wasm` - WASM build target surface.
EC Dimensions: behavior: `projects/jet/scripts/verify-advanced-wasm-gates.sh` - WASM build, runtime subset, renderer target, and DOM/WASM parity behavior gates.
Root WI: #3783
Status: auditing
Required Verification: smoke, conformance, corpus, negative
Promise:
Jet can sink the frontend app model into WASM only after Basic package
management, Browser Bridge, and DOM production build contracts are stable enough
to reuse.
Gate Inventory:
- projects/jet/fixtures/wasm/build-dev
- projects/jet/fixtures/wasm/runtime-subset
- projects/jet/fixtures/wasm/renderer-targets
- `projects/jet/scripts/verify-advanced-wasm-gates.sh`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Wasm multi target readiness | epic | #3783 | implemented | planned | none | Basic phase 1 -> phase 2 -> phase 3 |
| Wasm Build Dev Core | epic | #3783 | implemented | verified | conformance | `cargo test -p jet --lib wasm_build:: -- --nocapture`<br>projects/jet/fixtures/wasm/build-dev |
| Wasm Runtime Subset | epic | #3783 | implemented | verified | corpus | `cargo test -p jet-wasm -- --nocapture`<br>projects/jet/fixtures/wasm/runtime-subset |
| Renderer Target Output | epic | #3783 | implemented | verified | conformance | `cargo test -p jet-wasm renderer -- --nocapture`<br>projects/jet/fixtures/wasm/renderer-targets |
| WebGPU WASM Build Default | change | #3783 | implemented | verified | conformance | `cargo test -p jet --test wasm_build_end_to_end wasm_build_selects_webgpu_scaffold_by_default -- --nocapture` |
| WebGPU Large Table Smoke | change | #3783 | implemented | verified | conformance | `cargo test -p jet --test wasm_build_end_to_end webgpu_renderer_reports_runtime_status_and_visual_probe_when_available -- --nocapture` |
| DOM Renderer Controlled Input Parity | change | #4004 | implemented | verified | conformance | `cargo test -p jet --test react_dom_oracle_conformance dom_renderer_controlled_input_parity -- --nocapture` |
| DOM Renderer Controlled Textarea Parity | change | #4015 | implemented | verified | conformance | `cargo test -p jet --test react_dom_oracle_conformance dom_renderer_controlled_textarea_parity -- --nocapture` |
| Library WASM Lowering Fixtures | change | #4072 | implemented | verified | conformance | `cargo test -p jet --test tsx_to_rust_imports -- --nocapture`<br>projects/jet/parity/data/fixtures/libraries |
| Library DOM/WASM Parity Fixtures | change | #4072 | implemented | verified | conformance | `cargo test -p jet --test react_dom_oracle_conformance library_dom_wasm_parity -- --nocapture`<br>projects/jet/parity/data/fixtures/libraries |
| MUI Visual Table DOM/WASM Parity | change | #3783 | implemented | verified | conformance | `cargo test -p jet --test mui_visual_regression mui_visual_fixture_renders_on_react_dom_and_jet_wasm -- --nocapture`<br>Browser Bridge CLI capture/screenshot evidence<br>examples/mui-visual-demo |
| AntD Visual Table DOM/WASM Parity | change | #3783 | implemented | verified | conformance | `cargo test -p jet --test mui_visual_regression antd_visual_fixture_renders_on_react_dom_and_jet_wasm -- --nocapture`<br>Browser Bridge CLI capture/screenshot evidence<br>examples/antd-visual-demo |
### Browser, Trace, And Parity Infrastructure

ID: browser-trace-parity
Type: DeveloperTool
Surfaces: CLI: `jet bb` + `jet trace` - Browser Bridge and trace diagnostic surface.
EC Dimensions: behavior: `projects/jet/scripts/verify-browser-bridge-replacement.mjs` - Browser Bridge automation, trace evidence, and DOM/WASM parity corpus behavior.
Root WI: #3786
Status: auditing
Required Verification: smoke, conformance, corpus, negative
Promise:
Jet Browser Bridge, trace, and parity diagnostics are the second Basic
replacement gate and the evidence substrate for later DOM/WASM parity.
Gate Inventory:
- `projects/jet/scripts/verify-browser-bridge-replacement.mjs --jet-bin target/release/jet`
- projects/jet/fixtures/trace/artifacts
- projects/jet/fixtures/browser/automation-diagnostics
- projects/jet/parity/**

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Browser trace parity readiness | epic | #3786 | implemented | passing | corpus | `projects/jet/scripts/verify-browser-bridge-replacement.mjs` |
| Trace Evidence Artifacts | epic | #3786 | implemented | verified | conformance | `cargo test -p jet --lib trace -- --nocapture`<br>projects/jet/fixtures/trace/artifacts |
| Browser Automation Diagnostics | epic | #3786 | implemented | verified | negative | `cargo test -p jet --lib browser -- --nocapture`<br>projects/jet/fixtures/browser/automation-diagnostics |
| Parity Corpus Gates | epic | #3786 | implemented | verified | corpus | `projects/jet/scripts/verify-parity-oracle-gate.sh`<br>projects/jet/parity/** |
| WebGPU WASM Build Default | change | #3783 | implemented | verified | conformance | `cargo test -p jet --test wasm_build_end_to_end wasm_build_selects_webgpu_scaffold_by_default -- --nocapture` |
| WebGPU Large Table Smoke | change | #3783 | implemented | verified | conformance | `cargo test -p jet --test wasm_build_end_to_end webgpu_renderer_reports_runtime_status_and_visual_probe_when_available -- --nocapture` |
| DOM Renderer Controlled Input Parity | change | #4004 | implemented | verified | conformance | `cargo test -p jet --test react_dom_oracle_conformance dom_renderer_controlled_input_parity -- --nocapture` |
| DOM Renderer Controlled Textarea Parity | change | #4015 | implemented | verified | conformance | `cargo test -p jet --test react_dom_oracle_conformance dom_renderer_controlled_textarea_parity -- --nocapture` |
| Library WASM Lowering Fixtures | change | #4072 | implemented | verified | conformance | `cargo test -p jet --test tsx_to_rust_imports -- --nocapture`<br>projects/jet/parity/data/fixtures/libraries |
| Library DOM/WASM Parity Fixtures | change | #4072 | implemented | verified | conformance | `cargo test -p jet --test react_dom_oracle_conformance library_dom_wasm_parity -- --nocapture`<br>projects/jet/parity/data/fixtures/libraries |
| MUI Visual Table DOM/WASM Parity | change | #3783 | implemented | verified | conformance | `cargo test -p jet --test mui_visual_regression mui_visual_fixture_renders_on_react_dom_and_jet_wasm -- --nocapture`<br>Browser Bridge CLI capture/screenshot evidence<br>examples/mui-visual-demo |
| AntD Visual Table DOM/WASM Parity | change | #3783 | implemented | verified | conformance | `cargo test -p jet --test mui_visual_regression antd_visual_fixture_renders_on_react_dom_and_jet_wasm -- --nocapture`<br>Browser Bridge CLI capture/screenshot evidence<br>examples/antd-visual-demo |
