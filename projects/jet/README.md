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

Canonical field-style capability contracts below are machine-readable input for `aw capability`; YAML and legacy tables are migration input only.

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
| Library Build And Package Publishing | #168 | implemented | verified | conformance | partial | `jet build --lib` (ESM+CJS, externalized deps/peerDeps, multi-entry), preserve-modules ESM/CJS output, `.d.ts` emission, and `jet publish --build` with metadata validation + private-registry (`.npmrc` scoped) e2e all shipped and tested (A1-A3 merged). `partial`: IIFE lib output, class-member `.d.ts` reduction, and some CJS re-export edge cases are TODO follow-ups. |
| Component Workbench (Stories) | #169 | implemented | verified | conformance | ready_for_basic | CSF `*.stories.tsx` discovery, the Stories dev manager + isolated preview, preview HMR, and a prop-type-derived Controls panel (B1-B3 + B2b). The earlier `partial` follow-ups have since shipped: hook-state-preserving React Refresh (#196), `node_modules` bare-import resolution for dev + static (#197), generic/cross-file/intersection prop-type controls (#198), CSF2 `Template.bind`/re-exported stories/spread args (#199), and `jet stories build` static export (#190). CSF-compatible, no Storybook runtime. |

### Rust-Native Frontend Toolchain Replacement

ID: rust-native-frontend-toolchain
Root WI: #3778
Status: verified
Type: DeveloperTool
Required Verification: smoke, conformance, corpus, negative, dogfood
Promise:
Jet is gated as an all-in-one Basic frontend replacement in dependency order:
package manager, Browser Bridge, production build, serve, workspace, test,
e2e, trace, and stack-aware API client codegen. `jet codegen openapi`
resolves generated output from CLI flags, `jet.toml` `[codegen.openapi]`,
and `package.json` dependencies so the hook runtime (React Query or SWR) and
fetch/axios runtime selection match the project tech stack. See
`docs/openapi-codegen.md` for the hook/runtime matrix, axios version support,
and injecting a pre-configured `AxiosInstance`. The current
production-readiness gate is green.
Gate Inventory:
- `projects/jet/scripts/verify-basic-dom-gates.sh --all`
- projects/jet/tests/fixtures/dom-production-build
- `cargo test -p jet --test openapi_golden`
Surfaces:
- CLI: `jet install` + `jet build` + `jet dev` + `jet test` - Aggregate frontend toolchain entrypoints for package, build, dev, and test workflows.
EC Dimensions:
- behavior: `projects/jet/scripts/verify-basic-dom-gates.sh --all` - Basic frontend replacement flow across package, build, dev, serve, workspace, test, e2e, and trace gates.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Production replacement readiness | epic | #3778 | implemented | verified | corpus | package manager -> Browser Bridge -> build -> serve/workspace/test/e2e/trace are green in `projects/jet/scripts/verify-basic-dom-gates.sh --all` |
| Full Toolchain Dogfood Flow | epic | #3778 | implemented | verified | dogfood | `projects/jet/scripts/verify-basic-dom-gates.sh --all`<br>projects/jet/tests/fixtures/dom-production-build |
| Stack-Aware OpenAPI Codegen | change | #3778 | implemented | verified | conformance | `cargo test -p jet --test openapi_golden` — jet codegen openapi resolves stack/http/hooks from CLI flags, jet.toml, and package.json |

### Package Manager

ID: package-manager
Root WI: #3779
Status: verified
Type: DeveloperTool
Required Verification: smoke, conformance, corpus, negative
Promise:
Jet owns fixture hydration, mutation, workspace, and frozen-lockfile checks;
isolated npm/pnpm benchmark evidence is green for the current Basic corpus.
Gate Inventory:
- `cargo test -p jet --lib pkg_manager -- --nocapture`
- `node projects/jet/scripts/compare-pkg-management.mjs --baseline-tools npm,pnpm --require-baselines`
Surfaces:
- CLI: `jet install` + `jet add` + `jet remove` + `jet update` - Package lifecycle commands that own dependency and lockfile behavior.
EC Dimensions:
- behavior: `cargo test -p jet --lib pkg_manager -- --nocapture` - Package lifecycle, lockfile, workspace, registry, and negative-path conformance.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Package manager readiness | epic | #3779 | implemented | verified | corpus | `node projects/jet/scripts/compare-pkg-management.mjs` |
| Package Manager Lockfile Parity | epic | #3779 | implemented | verified | conformance | `cargo test -p jet --lib pkg_manager::lockfile -- --nocapture`<br>projects/jet/fixtures/pkg-manager/lockfile |
| Package Manager Workspace Parity | epic | #3779 | implemented | verified | conformance | `cargo test -p jet --lib pkg_manager::workspace -- --nocapture`<br>projects/jet/fixtures/pkg-manager/workspace |
| Package Manager Registry Integrity | epic | #3779 | implemented | verified | negative | `cargo test -p jet --lib pkg_manager -- --nocapture`<br>projects/jet/fixtures/pkg-manager/registry |

### Bundler And Production Build

ID: bundler-production-build
Root WI: #3782
Status: verified
Type: DeveloperTool
Required Verification: smoke, conformance, corpus, negative
Promise:
Jet production build replacement is green after package manager and Browser
Bridge gates. The expanded DOM production build corpus has green static checks,
runtime smoke, and performance/size comparisons for the current fixture set.
Gate Inventory:
- `node projects/jet/scripts/compare-dom-build-corpus.mjs --runtime-smoke required --build-samples 3`
- projects/jet/tests/fixtures/dom-production-build
Surfaces:
- CLI: `jet build` + `jet build --wasm` - Production and WASM build command surface.
EC Dimensions:
- behavior: `projects/jet/scripts/compare-dom-build-corpus.mjs --runtime-smoke required --build-samples 3` - DOM production build corpus, runtime smoke, and Vite/Webpack comparison behavior.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Bundler production readiness | epic | #3782 | implemented | verified | corpus | `node projects/jet/scripts/compare-dom-build-corpus.mjs --runtime-smoke required --build-samples 3` |
| Production Bundle Output Parity | epic | #3782 | implemented | verified | conformance | `cargo test -p jet --lib bundler -- --nocapture`<br>projects/jet/fixtures/bundler/production |
| Transform Resolver Parity | epic | #3782 | implemented | verified | corpus | `cargo test -p jet --lib transform -- --nocapture`<br>projects/jet/fixtures/bundler/transform-resolver |
| Asset Sourcemap Negative Paths | epic | #3782 | implemented | verified | negative | `cargo test -p jet --lib asset -- --nocapture`<br>projects/jet/fixtures/bundler/assets |
| SCSS / Sass Compilation | change | #204 | implemented | verified | conformance | `cargo test -p jet --lib css::scss` — grass-based (pure-Rust, no C deps) SCSS/Sass to CSS: nesting, variables, use/import partials, mixins; fed into the CSS pipeline before minify |

### Dev Server And HMR

ID: dev-server-hmr
Root WI: #3780
Status: verified
Type: DeveloperTool
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
Surfaces:
- CLI: `jet dev` + `jet dev --proxy PATH=URL` + `jet serve` - Dev server control plane, proxy/HMR entrypoints, and production static serving surface.
- UI: `http://localhost:<port>` - Connected browser client for HMR, browser log intake, and local app inspection.
EC Dimensions:
- behavior: `cargo test -p jet --lib dev_server -- --nocapture` - Local serving, HMR, proxy, browser-log intake, and production static serving conformance.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Dev server replacement readiness | epic | #3780 | implemented | verified | dogfood | `projects/jet/scripts/verify-basic-dom-gates.sh --phase serve` |
| Dev Server Local Serving Hmr | epic | #3780 | implemented | verified | conformance | `cargo test -p jet --lib dev_server -- --nocapture`<br>projects/jet/fixtures/dev-server/basic-hmr |
| Dev Server Proxy Contract | epic | #3780 | implemented | verified | conformance | `cargo test -p jet --lib dev_server::proxy -- --nocapture` |
| Dev Server Cli Contract | epic | #3780 | implemented | verified | conformance | `cargo test -p jet --lib cli::e2e_command_contract_tests -- --nocapture` |
| React Refresh State Preserved | epic | #3780 | implemented | verified | conformance | `cargo test -p jet --lib dev_server::hmr -- --nocapture`<br>projects/jet/fixtures/dev-server/react-refresh/state-preserved |
| Prebundle Importmap Parity | epic | #3780 | implemented | verified | corpus | `cargo test -p jet --lib dev_server::prebundle -- --nocapture`<br>projects/jet/fixtures/dev-server/prebundle-importmap |

### Workspace And Task Runner

ID: workspace-task-runner
Root WI: #3781
Status: verified
Type: DeveloperTool
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
Surfaces:
- CLI: `jet run` + `jet exec` - Workspace script and binary execution surface.
EC Dimensions:
- behavior: `cargo test -p jet --lib task_runner -- --nocapture` - Workspace script execution, graph cache, package selection, and task-runner parity behavior.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Workspace task runner readiness | epic | #3781 | implemented | verified | corpus | `cargo run -p jet -- config schema --check` |
| Task Runner Graph Cache | epic | #3781 | implemented | verified | conformance | `cargo test -p jet --lib task_runner -- --nocapture`<br>projects/jet/fixtures/task-runner/graph-cache |
| Workspace Package Selection | epic | #3781 | implemented | verified | conformance | `cargo test -p jet --lib pkg_manager::workspace -- --nocapture`<br>projects/jet/fixtures/workspace/package-selection |
| Nx Graph Parity | epic | #3781 | implemented | verified | corpus | `cargo test -p jet --lib pkg_manager::nx -- --nocapture`<br>projects/jet/fixtures/task-runner/nx |

### Native Test And Product-Flow E2E

ID: native-test-product-flow-e2e
Root WI: #3785
Status: verified
Type: DeveloperTool
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
Surfaces:
- CLI: `jet test` + `jet e2e` - Native test runner and product-flow e2e surface.
- WebAppE2E: `jet e2e` - Browser-driven product-flow verification for frontend behavior across app and API boundaries.
EC Dimensions:
- behavior: `jet e2e` - Browser-driven product-flow verification across frontend behavior and app/API boundaries.

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
Root WI: #3783
Status: auditing
Type: DeveloperTool
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
Surfaces:
- CLI: `jet build --wasm` - WASM build target surface.
EC Dimensions:
- behavior: `projects/jet/scripts/verify-advanced-wasm-gates.sh` - WASM build, runtime subset, renderer target, and DOM/WASM parity behavior gates.

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
Root WI: #3786
Status: auditing
Type: DeveloperTool
Required Verification: smoke, conformance, corpus, negative
Promise:
Jet Browser Bridge, trace, and parity diagnostics are the second Basic
replacement gate and the evidence substrate for later DOM/WASM parity.
Gate Inventory:
- `node projects/jet/scripts/verify-browser-bridge-replacement.mjs --jet-bin target/release/jet`
- projects/jet/fixtures/trace/artifacts
- projects/jet/fixtures/browser/automation-diagnostics
- projects/jet/parity/**
Surfaces:
- CLI: `jet bb` + `jet trace` - Browser Bridge and trace diagnostic surface.
EC Dimensions:
- behavior: `projects/jet/scripts/verify-browser-bridge-replacement.mjs` - Browser Bridge automation, trace evidence, and DOM/WASM parity corpus behavior.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Browser trace parity readiness | epic | #3786 | implemented | passing | corpus | `node projects/jet/scripts/verify-browser-bridge-replacement.mjs` |
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

### Library Build And Package Publishing

ID: library-build-publishing
Root WI: #168
Status: confirmed
Type: DeveloperTool
Required Verification: smoke, conformance, corpus, negative
Promise:
jet builds publishable npm packages in library mode (ESM + optional CJS, externalized dependencies/peerDependencies, multi-entry from package.json `exports`), emits `.d.ts` type declarations, and `jet publish --build` builds + validates package metadata (`exports`/`main`/`module`/`types`) before publishing to public or private (GitLab/Verdaccio/Nexus) registries via `.npmrc` scoped-registry auth. App-mode `jet build` is unchanged.
Gate Inventory:
- `cargo test -p jet --test library_build`
- `cargo test -p jet --test library_dts`
- `cargo test -p jet --test library_publish_e2e`
- `cargo test -p jet --lib bundler::lib_build bundler::dts`
- `cargo test -p jet --lib pkg_manager::publish`
Surfaces:
- CLI: `jet build --lib` + `jet publish --build` - Library package build, metadata validation, and registry publishing surface.
EC Dimensions:
- behavior: `cargo test -p jet --test library_publish_e2e` - Library build, declaration output, package metadata validation, and publish/private-registry conformance.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Library publishing readiness | epic | #168 | implemented | verified | conformance | `cargo test -p jet --test library_publish_e2e` — A1 library build, A2 declaration emission, and A3 publish/private-registry hardening are merged |
| Library Build Mode | change | #170 | implemented | verified | conformance | `cargo test -p jet --test library_build` — ESM+CJS, externalized deps/peerDeps, multi-entry, preserve-modules ESM/CJS (IIFE TODO) |
| Type Declaration Emission | change | #171 | implemented | verified | conformance | `cargo test -p jet --test library_dts` — declaration files per entry plus package types field (isolatedDeclarations) |
| Publish And Private Registry | change | #172 | implemented | verified | conformance | `cargo test -p jet --test library_publish_e2e` — build + metadata validate; in-process mock-registry publish/install round-trip |
| Library CSS Cascade-Merge | change | #205 | implemented | verified | conformance | `cargo test -p jet --lib bundler::css_bundle` — cascade-ordered CSS merge across entries plus raw asset copy in library builds |

### Component Workbench (Stories)

ID: component-workbench
Root WI: #169
Status: confirmed
Type: DeveloperTool
Required Verification: smoke, conformance, corpus, negative
Promise:
jet discovers and parses CSF `*.stories.tsx` (default-export meta + named-export stories), serves a jet-native manager UI (sidebar, isolated preview, toolbar) with HMR, and derives a live Controls panel from component prop types + `argTypes`. CSF/CSF2-compatible with no Storybook runtime dependency, with hook-state-preserving React Refresh, `node_modules` bare-import resolution, generic/cross-file prop-type controls, and a static `jet stories build` export (all shipped).
Gate Inventory:
- `cargo test -p jet --test csf_discovery`
- `cargo test -p jet --test manager`
- `cargo test -p jet --test preview_hmr`
- `cargo test -p jet --test controls`
- `cargo test -p jet --test stories_build`
Surfaces:
- CLI: `jet stories` + `jet stories build` - Component workbench dev server and static export entrypoints.
- UI: `jet stories` manager + preview - Sidebar, isolated story preview, toolbar, HMR, and controls panel surface.
EC Dimensions:
- behavior: `cargo test -p jet --test stories_build` - Static workbench export, story preview modules, and relative URL behavior.
- behavior: `cargo test -p jet --test manager` - Manager UI routing, story listing, isolated preview, and bare-import resolution behavior.
- behavior: `cargo test -p jet --test controls` - Prop-type-derived controls and live arg edit behavior.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Component workbench readiness | epic | #169 | implemented | verified | conformance | `cargo test -p jet --test stories_build` |
| CSF Story Discovery | change | #173 | implemented | verified | conformance | `cargo test -p jet --test csf_discovery` — glob + CSF3 meta/named-story parse into a story index |
| Stories Dev Manager | change | #174 | implemented | verified | conformance | `cargo test -p jet --test manager` — Stories dev command path, manager UI, and isolated per-story preview |
| Stories Preview HMR | change | #176 | implemented | verified | conformance | `cargo test -p jet --test preview_hmr` — watcher + WS, preview re-render/reload, manager untouched |
| Stories Controls Panel | change | #175 | implemented | verified | conformance | `cargo test -p jet --test controls` — prop-type-inferred controls plus argTypes override; live arg edits re-render the preview |
| Stories Static Export | change | #190 | implemented | verified | conformance | `cargo test -p jet --test stories_build` — jet stories build emits a static, server-less workbench with manager, per-story previews, transformed modules, and relative URLs |
| Hook-State-Preserving Refresh | change | #196 | implemented | verified | conformance | `cargo test -p jet --test preview_hmr` — React Refresh preserves useState/hook state across preview edits |
| Stories Bare-Import Resolution | change | #197 | implemented | verified | conformance | `cargo test -p jet --test manager` — node_modules bare-import resolution for stories dev and static export |
| Generic / Cross-File Prop Controls | change | #198 | implemented | verified | conformance | `cargo test -p jet --test controls` — controls inferred from generic, cross-file, and intersection prop types |
| CSF2 Template.bind + Re-Exports | change | #199 | implemented | verified | conformance | `cargo test -p jet --test csf_discovery` — CSF2 Template.bind, re-exported stories, and spread-args discovery |
