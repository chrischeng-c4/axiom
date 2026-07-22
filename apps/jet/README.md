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
  validates capability structure and TD refs in `CAPABILITIES.md`.
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
apps/jet/scripts/verify-basic-dom-gates.sh
JET_BASIC_DOM_BUILD_SAMPLES=3 JET_BASIC_DOM_RUNTIME_SMOKE=required apps/jet/scripts/verify-basic-dom-gates.sh --all
apps/jet/scripts/verify-advanced-wasm-gates.sh
aw capability check --project jet --pretty
aw health --project jet
```

Source map:

| Path | Read when |
|---|---|
| `apps/jet/CAPABILITIES.md` | Machine-readable capability contracts, promise guarantees, and verification gates. |
| `apps/jet/docs/architecture/layout.md` | You need the repo map before editing. |
| `apps/jet/src/pkg_manager/` | Package manager behavior, lockfile, registry, store, workspace, audit/publish flows. |
| `apps/jet/src/bundler/` | Dependency graph, tree shaking, CSS bundle, minification, splitting, sourcemaps. |
| `apps/jet/src/dev_server/` | Dev server, HMR, proxy, prod static serving, watcher, prebundle/import map behavior. |
| `apps/jet/src/browser/` and `apps/jet/src/browser_cli/` | Browser Bridge driver and CLI surfaces. |
| `apps/jet/src/test_runner/`, `apps/jet/src/e2e/`, `apps/jet/src/trace/` | Native test runtime, product-flow e2e, trace artifacts. |
| `apps/jet/src/wasm_build/` and `apps/jet/wasm/` | FE-on-WASM build path and runtime crate. |
| `apps/jet/parity/` | DOM/WASM parity corpus, oracle, gates, fixtures, schemas, ADRs. |
| `apps/jet/tests/` | Product and subsystem gates. |
| `.aw/tech-design/projects/jet/specs/3779.md` | Package-manager capability/spec entrypoint. |
| `.aw/tech-design/projects/jet/logic/pkg-manager.md` | Package-manager semantic/logic entrypoint. |

## Capabilities

The canonical field-style capability index, subsystem contracts, and verification gate inventories for Jet are maintained in [CAPABILITIES.md](./CAPABILITIES.md).
