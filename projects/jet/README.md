# jet

Rust-native replacement toolchain for modern web applications, covering package
management, development serving, production builds, test/e2e workflows, browser
trace/parity infrastructure, and WASM/multi-target execution.

Jet is intended to replace the Vite/Turbopack + pnpm/npm/Bun + Playwright-style
frontend toolchain in real projects. Production-replacement readiness is tracked
through the capability contracts below rather than asserted from feature
existence alone.

**Integrated into cclab CLI** — Use `cclab jet <command>`.

## Capability Map

This README is the canonical Jet `cap_path`. Markdown capability headings and tables below are machine-readable input for `aw capability`; YAML and legacy tables are migration input only.

Current status: `aw capability report jet --verify --json` is healthy with 8/8
capabilities verified, 23/23 required claims verified, no blockers, and
`next_action.kind = none`.

Next operating loop:

- Any new Jet product claim starts by updating the relevant
  capability table rows in this README.
- Validate structure with `aw capability check jet --json`.
- Validate product readiness with `aw capability report jet --verify --json` in
  an environment with GitHub issue access, loopback networking, browser/WASM
  prerequisites, and Cargo dependency access.
- If verification finds a product gap, create bounded follow-up work with
  `aw wi plan --project jet --json`, then run the normal WI -> TD -> CB ->
  TD merge lifecycle.
- Do not reopen generic "Jet readiness" work. A follow-up must name one
  concrete gap, fixture or real project, gate command, expected artifact or
  diagnostic, and close criteria.

## Features

### Package Manager

- **Parallel I/O** — concurrent resolution + downloads via `futures::try_join_all` + `Semaphore(16)`
- **Global content-addressable store** — `~/.jet-store/` with hardlinks to `node_modules/`
- **Lockfile fast-path** — skip resolution when `jet-lock.yaml` exists and integrity matches
- **HTTP optimizations** — abbreviated npm metadata, in-memory DashMap cache
- **Tarball extraction** — `.tgz` via `flate2` + `tar`, strips `package/` prefix
- **BFS transitive resolution** — greedy version matching with `semver::VersionReq`, conflict detection
- **Peer dependency resolution** — `peerDependencies` resolved alongside regular deps
- **Bin scripts linking** — symlink to `node_modules/.bin/` with `chmod +x`
- **Lifecycle hooks** — `preinstall` / `install` / `postinstall` via `sh -c`
- **Shasum verification** — SHA-256 integrity check on downloaded tarballs

### Bundler

- **JSX/TSX transformation** — custom Tree-sitter based (no SWC dependency)
- **TypeScript type stripping** — removes type annotations, interfaces, enums
- **Module resolution** — full Node.js algorithm with `exports` field support
- **Dependency graph** — petgraph-based with cycle detection and topological sort
- **Parallel transformation** — Rayon-based concurrent module processing
- **Single-file bundle** — `__jet__` runtime module system

### Dev Server

- **Axum HTTP server** — static file serving, SPA fallback
- **WebSocket HMR** — `/__jet_hmr` endpoint with auto-reconnect
- **File watching** — `notify` crate with smart filtering

## Commands

```bash
cclab jet init                    # Initialize a new project
cclab jet install                 # Install dependencies from package.json
cclab jet add <package> [--dev]   # Add a dependency
cclab jet remove <package>        # Remove a dependency
cclab jet dev [-p <port>]         # Start dev server with HMR
cclab jet build [-w] [-o <dir>]   # Build for production
```

## Architecture

All modules are in a single crate (`jet`):

```
projects/jet/src/
├── cli.rs                  # CLI command definitions
├── lib.rs                  # Crate root
├── bundler/                # Dependency graph + bundle generation
│   └── mod.rs              #   __jet__ runtime, module wrapping
├── dev_server/             # HTTP server + HMR
│   ├── mod.rs              #   Axum routes, WebSocket, HMR client
│   └── watcher.rs          #   File watching with notify
├── pkg_manager/            # Package management
│   ├── mod.rs              #   3-phase install orchestrator
│   ├── registry.rs         #   npm registry client + DashMap cache
│   ├── resolver.rs         #   BFS transitive + peer dep resolution
│   ├── store.rs            #   Global store, tarball extraction, bin linking
│   └── lockfile.rs         #   jet-lock.yaml v2
├── resolver/               # Module resolution (Node.js algorithm)
├── transform/              # JSX/TS transformation (Tree-sitter)
└── asset/                  # Asset processing pipeline
```

## Install Pipeline (3 Phases)

```
Phase 1 — Parallel (Semaphore=16):
  Read package.json → Check jet-lock.yaml → BFS Resolve
  → Download tarballs → Verify shasum → Extract to ~/.jet-store/
  → Hardlink to node_modules/

Phase 2 — Sequential:
  Link bin scripts → node_modules/.bin/ (symlink + chmod +x)

Phase 3 — Sequential:
  Run lifecycle hooks (preinstall → install → postinstall)

→ Write jet-lock.yaml
```

## Tests

```
66 tests passing

pkg_manager:   28 tests (resolver, lockfile, store, registry, mod)
bundler:       14 tests
transform:     13 tests
resolver:      11 tests
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `tree-sitter` | JS/TS parsing (shared with cclab-prism) |
| `petgraph` | Dependency graph |
| `axum` | Dev server |
| `notify` | File watching |
| `reqwest` | HTTP client (npm registry) |
| `semver` | Version resolution |
| `dashmap` | Concurrent metadata cache |
| `flate2` + `tar` | Tarball extraction |
| `sha2` | Shasum verification |
| `futures` | Parallel async I/O |

## Spec

Full spec with JSON Schema, OpenAPI, and Mermaid diagrams: `.aw/tech-design/jet/pkg-manager.md`

## Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Rust-Native Frontend Toolchain Replacement | #3778 | implemented | verified | smoke, conformance, corpus, negative, dogfood | ready | Jet can replace the Vite/Turbopack + pnpm/npm/Bun + Playwright-style frontend toolchain for real projects. |
| Package Manager | #3779 | implemented | verified | smoke, conformance, corpus, negative | ready | Jet can replace npm/pnpm/Bun package-management flows for real projects. |
| Bundler And Production Build | #3782 | implemented | verified | smoke, conformance, corpus, negative | ready | Jet can produce production frontend builds with Vite/Webpack-class behavior for real projects. |
| Dev Server And HMR | #3780 | implemented | verified | smoke, conformance, corpus, negative, dogfood | ready | Jet can replace Vite-style local development serving and HMR for real projects. |
| Workspace And Task Runner | #3781 | implemented | verified | smoke, conformance, corpus, negative | ready | Jet can replace npm scripts, pnpm workspaces, and common Nx/Turborepo task-runner flows. |
| Native Test And Product-Flow E2E | #3785 | implemented | verified | smoke, conformance, corpus, negative, dogfood | ready | Jet can replace Jest/Vitest plus Playwright/Cypress-style product-flow testing for real projects. |
| WASM And Multi-Target Execution | #3783 | implemented | verified | smoke, conformance, corpus, negative | ready | Jet can support WASM and multi-target frontend execution paths. |
| Browser, Trace, And Parity Infrastructure | #3786 | implemented | verified | smoke, conformance, corpus, negative | ready | Jet can produce inspectable browser, trace, and parity evidence for replacement readiness. |

## Rust-Native Frontend Toolchain Replacement

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| rust-native-frontend-toolchain | #3778 | verified | Jet can replace the Vite/Turbopack + pnpm/npm/Bun + Playwright-style frontend toolchain for real projects. | smoke, conformance, corpus, negative, dogfood | projects/jet/fixtures/dogfood/full-toolchain<br>`cargo test -p jet` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Production replacement readiness | epic | #3778 | implemented | verified | none | - |
| Full Toolchain Dogfood Flow | epic | #3778 | implemented | verified | dogfood | `cargo test -p jet`<br>projects/jet/fixtures/dogfood/full-toolchain |

### Package Manager

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| package-manager | #3779 | verified | Jet can replace npm/pnpm/Bun package-management flows for real projects. | smoke, conformance, corpus, negative | projects/jet/fixtures/pkg-manager/lockfile<br>projects/jet/fixtures/pkg-manager/workspace<br>projects/jet/fixtures/pkg-manager/registry<br>`cargo test -p jet pkg_manager::lockfile -- --nocapture` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Package manager readiness | epic | #3779 | implemented | verified | none | - |
| Package Manager Lockfile Parity | epic | #3779 | implemented | verified | conformance | `cargo test -p jet pkg_manager::lockfile -- --nocapture`<br>projects/jet/fixtures/pkg-manager/lockfile |
| Package Manager Workspace Parity | epic | #3779 | implemented | verified | conformance | `cargo test -p jet pkg_manager::workspace -- --nocapture`<br>projects/jet/fixtures/pkg-manager/workspace |
| Package Manager Registry Integrity | epic | #3779 | implemented | verified | negative | `cargo test -p jet pkg_manager -- --nocapture`<br>projects/jet/fixtures/pkg-manager/registry |

### Bundler And Production Build

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| bundler-production-build | #3782 | verified | Jet can produce production frontend builds with Vite/Webpack-class behavior for real projects. | smoke, conformance, corpus, negative | projects/jet/fixtures/bundler/production<br>projects/jet/fixtures/bundler/transform-resolver<br>projects/jet/fixtures/bundler/assets<br>`cargo test -p jet bundler -- --nocapture` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Bundler production readiness | epic | #3782 | implemented | verified | none | - |
| Production Bundle Output Parity | epic | #3782 | implemented | verified | conformance | `cargo test -p jet bundler -- --nocapture`<br>projects/jet/fixtures/bundler/production |
| Transform Resolver Parity | epic | #3782 | implemented | verified | corpus | `cargo test -p jet transform -- --nocapture`<br>projects/jet/fixtures/bundler/transform-resolver |
| Asset Sourcemap Negative Paths | epic | #3782 | implemented | verified | negative | `cargo test -p jet asset -- --nocapture`<br>projects/jet/fixtures/bundler/assets |

### Dev Server And HMR

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| dev-server-hmr | #3780 | verified | Jet can replace Vite-style local development serving and HMR for real projects. | smoke, conformance, corpus, negative, dogfood | projects/jet/fixtures/dev-server/basic-hmr<br>projects/jet/fixtures/dev-server/react-refresh/state-preserved<br>projects/jet/fixtures/dev-server/prebundle-importmap<br>`cargo test -p jet --lib dev_server -- --nocapture` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Dev server replacement readiness | epic | #3780 | implemented | verified | none | - |
| Dev Server Local Serving Hmr | epic | #3780 | implemented | verified | conformance | `cargo test -p jet dev_server -- --nocapture`<br>projects/jet/fixtures/dev-server/basic-hmr |
| React Refresh State Preserved | epic | #3780 | implemented | verified | conformance | `cargo test -p jet dev_server::hmr -- --nocapture`<br>projects/jet/fixtures/dev-server/react-refresh/state-preserved |
| Prebundle Importmap Parity | epic | #3780 | implemented | verified | corpus | `cargo test -p jet dev_server::prebundle -- --nocapture`<br>projects/jet/fixtures/dev-server/prebundle-importmap |

### Workspace And Task Runner

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| workspace-task-runner | #3781 | verified | Jet can replace npm scripts, pnpm workspaces, and common Nx/Turborepo task-runner flows. | smoke, conformance, corpus, negative | projects/jet/fixtures/task-runner/graph-cache<br>projects/jet/fixtures/workspace/package-selection<br>projects/jet/fixtures/task-runner/nx<br>`cargo test -p jet task_runner -- --nocapture` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Workspace task runner readiness | epic | #3781 | implemented | verified | none | - |
| Task Runner Graph Cache | epic | #3781 | implemented | verified | conformance | `cargo test -p jet task_runner -- --nocapture`<br>projects/jet/fixtures/task-runner/graph-cache |
| Workspace Package Selection | epic | #3781 | implemented | verified | conformance | `cargo test -p jet pkg_manager::workspace -- --nocapture`<br>projects/jet/fixtures/workspace/package-selection |
| Nx Graph Parity | epic | #3781 | implemented | verified | corpus | `cargo test -p jet pkg_manager::nx -- --nocapture`<br>projects/jet/fixtures/task-runner/nx |

### Native Test And Product-Flow E2E

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| native-test-product-flow-e2e | #3785 | verified | Jet can replace Jest/Vitest plus Playwright/Cypress-style product-flow testing for real projects. | smoke, conformance, corpus, negative, dogfood | projects/jet/fixtures/test-runner/native<br>projects/jet/fixtures/test-runner/reporters<br>projects/jet/fixtures/e2e/product-flow<br>projects/jet/fixtures/e2e/trace-replay<br>`cargo test -p jet test_runner -- --nocapture`<br>`cargo test -p jet e2e -- --nocapture` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Native test runner readiness | epic | #3785 | implemented | verified | none | - |
| Product flow e2e readiness | epic | #3784 | implemented | verified | none | - |
| Native Test Runner Core | epic | #3785 | implemented | verified | conformance | `cargo test -p jet test_runner -- --nocapture`<br>projects/jet/fixtures/test-runner/native |
| Reporter Artifacts | epic | #3785 | implemented | verified | negative | `cargo test -p jet reporter -- --nocapture`<br>projects/jet/fixtures/test-runner/reporters |
| Product Flow E2e Review | epic | #3785 | implemented | verified | dogfood | `cargo test -p jet e2e -- --nocapture`<br>projects/jet/fixtures/e2e/product-flow |
| Trace Replay Evidence | epic | #3785 | implemented | verified | conformance | `cargo test -p jet trace -- --nocapture`<br>projects/jet/fixtures/e2e/trace-replay |

### WASM And Multi-Target Execution

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| wasm-multi-target | #3783 | verified | Jet can support WASM and multi-target frontend execution paths. | smoke, conformance, corpus, negative | projects/jet/fixtures/wasm/build-dev<br>projects/jet/fixtures/wasm/runtime-subset<br>projects/jet/fixtures/wasm/renderer-targets<br>`cargo test -p jet wasm -- --nocapture` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Wasm multi target readiness | epic | #3783 | implemented | verified | none | - |
| Wasm Build Dev Core | epic | #3783 | implemented | verified | conformance | `cargo test -p jet wasm -- --nocapture`<br>projects/jet/fixtures/wasm/build-dev |
| Wasm Runtime Subset | epic | #3783 | implemented | verified | corpus | `cargo test -p jet-wasm -- --nocapture`<br>projects/jet/fixtures/wasm/runtime-subset |
| Renderer Target Output | epic | #3783 | implemented | verified | conformance | `cargo test -p jet-wasm renderer -- --nocapture`<br>projects/jet/fixtures/wasm/renderer-targets |
| Dom Renderer Controlled Input Parity | change | #4004 | implemented | verified | conformance | `cargo test -p jet --test react_dom_oracle_conformance dom_renderer_controlled_input_parity -- --nocapture`<br>projects/jet/tests/react_dom_oracle_conformance.rs |
| Dom Renderer Controlled Textarea Parity | change | #4015 | implemented | verified | conformance | `cargo test -p jet --test react_dom_oracle_conformance dom_renderer_controlled_textarea_parity -- --nocapture`<br>projects/jet/tests/react_dom_oracle_conformance.rs |
| Library Dom/Wasm Parity Fixtures | change | #4041 | implemented | verified | conformance | `cargo test -p jet --test react_dom_oracle_conformance library_dom_wasm_parity -- --nocapture`<br>projects/jet/parity/data/fixtures/libraries |
| Library Form-Control Dom/Wasm Parity | change | #4072 | implemented | verified | conformance | `cargo test -p jet --test react_dom_oracle_conformance library_dom_wasm_parity -- --nocapture`<br>projects/jet/parity/data/fixtures/libraries |

### Browser, Trace, And Parity Infrastructure

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| browser-trace-parity | #3786 | verified | Jet can produce inspectable browser, trace, and parity evidence for replacement readiness. | smoke, conformance, corpus, negative | projects/jet/fixtures/trace/artifacts<br>projects/jet/fixtures/browser/automation-diagnostics<br>projects/jet/parity/**<br>`cargo test -p jet trace -- --nocapture` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Browser trace parity readiness | epic | #3786 | implemented | verified | none | - |
| Trace Evidence Artifacts | epic | #3786 | implemented | verified | conformance | `cargo test -p jet trace -- --nocapture`<br>projects/jet/fixtures/trace/artifacts |
| Browser Automation Diagnostics | epic | #3786 | implemented | verified | negative | `cargo test -p jet browser -- --nocapture`<br>projects/jet/fixtures/browser/automation-diagnostics |
| Parity Corpus Gates | epic | #3786 | implemented | verified | corpus | `cargo test -p jet parity -- --nocapture`<br>projects/jet/parity/** |
| Dom Renderer Controlled Input Parity | change | #4004 | implemented | verified | conformance | `cargo test -p jet --test react_dom_oracle_conformance dom_renderer_controlled_input_parity -- --nocapture`<br>projects/jet/tests/react_dom_oracle_conformance.rs |
| Dom Renderer Controlled Textarea Parity | change | #4015 | implemented | verified | conformance | `cargo test -p jet --test react_dom_oracle_conformance dom_renderer_controlled_textarea_parity -- --nocapture`<br>projects/jet/tests/react_dom_oracle_conformance.rs |
| Library Dom/Wasm Parity Fixtures | change | #4041 | implemented | verified | conformance | `cargo test -p jet --test react_dom_oracle_conformance library_dom_wasm_parity -- --nocapture`<br>projects/jet/parity/data/fixtures/libraries |
| Library Form-Control Dom/Wasm Parity | change | #4072 | implemented | verified | conformance | `cargo test -p jet --test react_dom_oracle_conformance library_dom_wasm_parity -- --nocapture`<br>projects/jet/parity/data/fixtures/libraries |
