# jet

Rust-native replacement toolchain for modern web applications, covering package
management, development serving, production builds, native test/e2e workflows,
browser trace/parity infrastructure, and FE-on-WASM execution.

Jet is intended to replace the Vite/Turbopack + pnpm/npm/Bun +
Playwright-style frontend toolchain in real projects. It is organized as two
layered toolchains:

- **Basic: FE-on-DOM toolchain** — package manager, bundler, production build,
  dev server, test/e2e runner, browser automation, trace, and parity evidence
  for ordinary browser DOM applications.
- **Advanced: FE-on-WASM toolchain** — a new product category built on top of
  the Basic toolchain. It sinks the frontend app model into WASM: TS/TSX
  logic, CSS styling/layout inputs, and the HTML-like host tree are represented
  inside Jet's Rust/WASM runtime, then painted through canvas/WebGPU without
  rendering app widgets as DOM nodes.

The Advanced toolchain is DOM-oracle first: if FE-on-WASM behavior differs from
the Basic FE-on-DOM oracle for the same app and gesture, the gap belongs to
Jet's WASM runtime, renderer, or browser bridge.

**Primary CLI** — Use `jet <command>`. The `cclab jet <command>` wrapper may
also be available in integrated cclab environments.

## Capability Map

This README is the canonical Jet `cap_path`. Markdown capability headings and tables below are machine-readable input for `aw capability`; YAML and legacy tables are migration input only.

This README defines the capabilities Jet is meant to provide. Verification and
format linting are tracked separately; do not treat this document as a narrow
implementation checklist. When the target product surface expands, update the
capability definitions here first, then align tests, specs, and work items.

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

## Current Capability Status

This section is the product status dashboard. It is intentionally stricter than
the work-root Capability Index: `green` means a runnable gate exists and the
named scope is verified; `yellow` means the implementation exists but the
oracle, fixture breadth, performance comparison, or replayable evidence is not
complete; `red` means a known required behavior fails; `gray` means the gate was
not run in the current environment.

Last local status refresh: 2026-06-12.

| Layer | Overall status | Local proof | Current interpretation |
|---|---|---|---|
| Basic FE-on-DOM | `green` for the current production-readiness gate across package management, Browser Bridge, production build, serve, workspace, test, e2e, and trace | `JET_BASIC_DOM_BUILD_SAMPLES=3 JET_BASIC_DOM_RUNTIME_SMOKE=required projects/jet/scripts/verify-basic-dom-gates.sh --all` is green locally. Evidence files under `/tmp/jet-basic-dom-gate/` include `pkg-management-compare.json` (`basic.install.replacement` green), `browser-bridge-replacement.json` (`basic.browser-bridge.replacement` green), `basic-build-corpus.json` (`basic.build.production.corpus` green), and `prod-static-serve.json` (`basic.serve.prod-static` green). `jet serve --prod` beats the nginx container baseline in the local static-origin gate: first-byte p95 ratio `0.803`, throughput ratio `1.164`. | Basic is production-ready for the current FE-on-DOM replacement contract. Advanced FE-on-WASM remains a separate readiness track and must not be used to qualify or dilute the Basic production claim. |
| Advanced FE-on-WASM | `yellow` overall | `projects/jet/scripts/verify-advanced-wasm-gates.sh` defines the current gate set for WASM build config, `jet-wasm`, WebGPU default output, WebGPU visual probe, library lowering fixtures, MUI/AntD DOM/WASM parity, and DOM oracle parity skeleton. | The path is implemented enough for focused WebGPU/WASM fixtures and Browser Bridge evidence, but it is not production-ready until typed frontend IR, TS/CSS/HTML lowering, layout, text, selection, clipboard, scroll, context menu, a11y, and post-load performance rows have replayable DOM-vs-WASM traces. |

Basic command-family status:

| Command family | Status | Required gate / evidence | Remaining gap before full green |
|---|---|---|---|
| `jet install/add/remove/update/audit` | `green` for local package-manager correctness, fixture hydration, executable bins, mutation behavior, workspace linking, workspace frozen-lockfile drift rejection, and required npm/pnpm baseline benchmark thresholds | `cargo test -p jet --lib pkg_manager -- --nocapture` plus `JET_BASIC_DOM_PACKAGE_BASELINES=npm,pnpm JET_BASIC_DOM_REQUIRE_BASELINES=1 projects/jet/scripts/verify-basic-dom-gates.sh --phase package` | Extend the Jet-owned package gate into downloaded bytes, cache hit evidence, lifecycle output breadth, and broader workspace layouts. npm may run only as a host comparator against isolated benchmark copies; pnpm comparator runs from a temporary tool root provisioned by `jet install`. npm/pnpm artifacts remain oracle evidence, not the fixture management path. |
| `jet build` | `green` for the expanded DOM production build corpus; intentionally third after package and Browser Bridge gates | `JET_BASIC_DOM_BUILD_SAMPLES=3 JET_BASIC_DOM_RUNTIME_SMOKE=required projects/jet/scripts/verify-basic-dom-gates.sh --all`; `basic-build-corpus.json` is green across react-bench, DOM production assets, MUI, AntD, Tailwind, and styled-components fixtures with runtime smoke required | Current tightest visual-library slice remains green: MUI duration ratio `1.184` and gzip ratio `1.041` against Vite; AntD duration ratio `0.736` and gzip ratio `0.608` against Vite. Continue broadening fixture coverage, but there is no red Basic production-build blocker in the current gate. |
| `jet dev` / `jet serve --prod` | `green` for local serve/HMR and nginx-class production static-origin replacement | `cargo test -p jet --lib dev_server -- --nocapture` through `verify-basic-dom-gates.sh`; `compare-prod-static-serve.mjs` writes `/tmp/jet-basic-dom-gate/prod-static-serve.json` and checks startup, first-byte latency, throughput, cache headers, SPA fallback, conditional/range requests, health/readiness, shutdown, and logs | Dev serving is the client-connected control plane for HMR, browser log intake, and proxying local API/WebSocket backends. `jet serve --prod` is the data plane: optimize memory footprint, response-copy cost, and RPS as an app-container static origin behind a load balancer; it does not own TLS termination, public virtual hosts, cert management, WAF/CDN, or cross-service ingress routing. |
| `jet bb` / `jet browser` | `green` for detached agent-first core gesture automation plus isolated Playwright baseline comparison; `yellow` for full Playwright/Cypress replacement surface | `cargo test -p jet --lib browser -- --nocapture`; `JET_BASIC_DOM_BROWSER_BASELINES=playwright JET_BASIC_DOM_REQUIRE_BROWSER_BASELINES=1 projects/jet/scripts/verify-basic-dom-gates.sh --phase browser`; production comparator drives runtime smoke through `jet bb launch/eval/shutdown` | Semantic snapshot/ref interaction (`snapshot/click/fill/type/hover/select/check` by ref or locator selector), navigation (`goto/back/forward/reload/resize/wait`), and console/fetch observability (`console/requests`) now ship on both `jet bb` and `jet bb mcp`. Next: tabs/multi-session, storage state + route interception on the bb surface, richer failure diagnostics, trace replay hooks, and WASM browser-API bridge coverage before broadening build claims. |
| `jet test` | `green` for the Basic native runner/reporter gate | `cargo test -p jet --lib test_runner -- --nocapture`; `cargo test -p jet --lib reporter -- --nocapture` through `verify-basic-dom-gates.sh --all` | Continue broadening Vitest/Jest comparison fixtures and dogfood coverage, but the current Basic gate is green. |
| `jet e2e` | `green` for the Basic product-flow e2e gate | `cargo test -p jet --lib e2e -- --nocapture` through `verify-basic-dom-gates.sh --all` | Add larger real-app fixtures and DOM/WASM matrix support as follow-up breadth, not as a current Basic production blocker. |
| `jet trace` | `green` for the Basic trace evidence gate | `cargo test -p jet --lib trace -- --nocapture` through `verify-basic-dom-gates.sh --all` | Add richer trace replay and comparison evidence across network/console, DOM snapshots, WASM capture, screenshots, and pHash deltas as follow-up breadth. |

Advanced WASM contract-family status:

| Contract family | Status | Current evidence | Required next evidence |
|---|---|---|---|
| Typed frontend IR and WASM build | `yellow` | `cargo test -p jet --lib wasm_build:: -- --nocapture`; `cargo test -p jet --test wasm_build_end_to_end wasm_build_selects_webgpu_scaffold_by_default -- --nocapture` | Typed TS/HTML/CSS frontend IR snapshots proving type facts, host tree intent, CSS IR, and resource graph survive before target emission. |
| Type-to-Rust lowering | `yellow` | `cargo test -p jet --test tsx_to_rust_imports -- --nocapture` plus focused TSX-to-Rust tests | More library and app fixtures covering object/array/union/state/event payload shapes and memory layout decisions. |
| Wrapper shell and browser bridge | `yellow` | `jet bb capture --surface dom`, `jet bb capture --surface wasm`, Browser Bridge unit gates, MUI/AntD visual fixture gates | Prove wrapper JS is only a loader/bridge and that app semantics live in WASM for supported fixtures. |
| WebGPU renderer and visual probe | `yellow` | `cargo test -p jet --test wasm_build_end_to_end webgpu_renderer_reports_runtime_status_and_visual_probe_when_available -- --nocapture`; MUI/AntD visual regression gates | Broaden to replayable layout/paint/text traces, screenshot/pHash thresholds, glyph atlas evidence, and large-table steady-state rendering. |
| Layout, table, text, and fonts | `yellow` | Renderer layout/paint/text tests under `projects/jet/wasm/tests/` and grid renderer crates | DOM oracle traces for table geometry, text runs, clipping, glyph counts, and font policy with explicit tolerances. |
| Pointer, wheel, scroll, and overlay scrollbars | `yellow` | Browser Bridge input commands and WASM renderer/runtime tests | DOM-vs-WASM traces showing hit targets, scroll offsets, clamping, scrollbar visible/idle-hidden states, thumb geometry, and repaint coalescing. |
| Selection and clipboard | `yellow` | WASM selection renderer source/tests are present in the worktree | DOM-vs-WASM drag/key/copy traces proving selected range, copied TSV/text, visual highlight, and clipboard status. |
| Forms, focus, keyboard, and context menu | `yellow` | Focused controlled-input and event pipeline tests exist; context menu policy is documented | Replayable traces for active target, keyboard dispatch, controlled values, checked state, default context menu policy, and app override behavior. |
| Accessibility evidence | `yellow` | Browser/DOM capture surfaces exist | Role/name/state snapshots or explicit unsupported markers for each supported widget family. |
| Runtime performance after load | `yellow` | Frame timing and renderer performance hooks exist in grid/WebGPU crates | Post-load budgets for scroll, selection, input-to-frame latency, CPU/GPU frame timing, frame count, and coalescing counters. |

## Toolchain Capability Model

Jet provides one base system and one advanced system.

| Toolchain | Role | Capability contract |
|---|---|---|
| Basic: FE-on-DOM | Replacement for today's frontend toolchain | Install dependencies, resolve modules, bundle/build DOM apps, serve HMR dev sessions, run native tests and product-flow e2e, drive browsers without Playwright, capture trace/parity evidence |
| Advanced: FE-on-WASM | WebGPU/WASM frontend runtime on top of Basic | Sink TS/TSX, CSS, and HTML-like app structure into WASM, render through canvas/WebGPU, emulate the browser behavior subset Jet supports, and match the Basic DOM oracle for equivalent apps and gestures |

Execution priority is Basic first. Jet must complete the FE-on-DOM toolchain as
a credible replacement for today's market-standard stack before Advanced
FE-on-WASM claims can count as product-ready. Advanced work may reuse and
stress-test Basic components, but it must not redefine Basic completion or hide
gaps in package management, build, dev server, browser automation, test/e2e, or
trace evidence.

Within Basic, the replacement order is strict:

1. Package management first: fixtures are hydrated and checked by `jet install`
   from `jet-lock.yaml`; npm/pnpm/Bun are external oracles and benchmarks, not
   fixture managers.
2. Browser automation second: runtime gates use `jet bb`/`jet browser`;
   Playwright/Cypress-style behavior is the replacement target, not the
   executor.
3. Production build third: only after package management and Browser Bridge are
   first-class gates does `jet build` compete with Vite/Webpack on correctness,
   speed, and bundle size.

The current phase-gate state is: phase 1 package replacement is green for the
current corpus, phase 2 Browser Bridge core actions plus the isolated Playwright
baseline are green for the current gesture contract, phase 3 production build is
green for the expanded DOM corpus, and the serve/workspace/test/e2e/trace gates
are green in `verify-basic-dom-gates.sh --all`. Jet-owned fixture hydration stays
separate from incumbent benchmarks: `compare-pkg-management.mjs` defaults to required npm/pnpm isolated
baselines and may run comparator package managers only inside benchmark copies,
never as the source fixture manager. Phase 2 applies the same rule to browser
automation: `jet bb` is the executor and Playwright/Cypress are comparison
targets. Package-manager baselines may use host `npm` only against isolated
benchmark copies; the pnpm comparator is installed into a temporary tool root
by `jet install`.
Browser Bridge baselines may use Playwright only from an explicit
`JET_PLAYWRIGHT_PACKAGE_ROOT` or from a temporary tool root provisioned by
`jet install`; repo-root Playwright resolution is intentionally ignored. No gate
may use `npm ci` or Playwright as Jet's fixture/runtime path. Phase 3 build work
stays behind those replacement contracts.

Five command families make those two toolchains work:

| Family | Target command | Current command | Owns | Capability contract |
|---|---|---|---|---|
| Package management | `jet install/add/remove/update/audit` | same | Dependency lifecycle | Mature package install, lockfile, workspace, registry, bin, lifecycle, and integrity behavior |
| Build | `jet build [--wasm]` | same | Artifact creation | DOM builds, WASM builds, static assets, `dist/jet-target.json`, reproducible build metadata |
| Dev | `jet dev [--proxy PATH=URL]`; `jet dev --wasm` | same | Local development serving | Foreground dev control plane with hot reload, HMR, browser client connection, console/log intake, reverse proxy rules, and test/debug hooks. DOM dev proxy rules come from `[dev.proxy]` in `jet.toml` and repeatable `--proxy` CLI overrides; `jet dev --wasm` keeps its separate WASM loop until its proxy path is wired. |
| Serve | `jet serve [--prod] [--wasm]` | same; dev mode delegates to the dev serving surface for now | Detached and production serving | Agent-first serving returns machine-readable URL/session metadata. With `--prod`: production static serving suitable to replace a tuned nginx-style frontend deployment. Without `--prod`: detached dev serving compatibility. With `--wasm`: serve the Advanced FE-on-WASM target. |
| Browser Bridge | `jet bb ...` | same; legacy `jet browser ...` remains | Browser automation and WASM bridge | Agent-first browser control. Default launch mode is headless and detached, returning a controllable browser/session handle. Explicit attach/headful modes are for human inspection. Owns click/drag/key/wheel/eval/screenshot/capture, DOM oracle capture, WASM runtime capture, and browser API bridge into FE-on-WASM |
| Test, e2e, trace | `jet test`, `jet e2e`, `jet trace` | same | Verification and evidence | Basic validates TS/DOM toolchain behavior with a Jet-owned TS test runtime that is auto-available under `jet test` and does not need an npm package. Advanced reuses the same surfaces for WASM runtime checks, DOM-oracle parity, Browser Bridge evidence, trace replay, and diagnostics. |

The dependency direction is intentionally one-way:

```text
jet build
  -> emits Basic DOM or Advanced WASM artifacts and manifest

jet serve
  -> invokes build when needed and serves the selected toolchain target
  -> defaults to a detached agent-managed server session

jet bb
  -> controls a live browser target and observes Basic DOM or Advanced WASM surfaces
  -> defaults to a headless detached agent-managed browser session
  -> does not own build semantics
```

`jet build` has shared frontend analysis and two target-specific build paths:

```text
jet build                    (FE-on-DOM)
  TSX + other FE inputs
    -> typed TS/HTML/CSS frontend IR
    -> JS/HTML/CSS artifacts

jet build --wasm             (FE-on-WASM)
  TSX + other FE inputs
    -> typed TS/HTML/CSS frontend IR
    -> Rust/WASM runtime artifacts + thin wrapper HTML/CSS/JS assets
```

The shared typed `TS/HTML/CSS frontend IR` is the contract boundary between
source analysis and target emission. It preserves app logic, TypeScript type
information, host tree intent, style inputs, resource references, and public
asset semantics before the build target is selected. The DOM target may erase
TypeScript types while emitting browser-native `JS/HTML/CSS`. The WASM target
must not route through type-erased JS as its canonical input; it consumes the
typed frontend IR so Rust/WASM emission can choose concrete scalar, struct,
arena, and memory-layout representations. Wrapper JS is allowed only as a host
loader/bridge artifact, not as the app semantics carrier.

Build evidence is split along that boundary:

| Stage | Applies to | Required evidence | Failure meaning |
|---|---|---|---|
| Typed frontend analysis | `jet build`, `jet build --wasm` | Typed TS graph, HTML host tree IR, CSS IR, module/resource graph, type facts, diagnostics | Jet misunderstood the source app before any target-specific output. |
| DOM emission | `jet build` | JS/HTML/CSS artifacts, DOM asset manifest, sourcemaps, runtime smoke, Vite/Webpack comparison | Jet's FE-on-DOM build output differs from the market baseline. |
| WASM typed emission | `jet build --wasm` | Type-to-Rust mapping, generated Rust/WASM module metadata, memory-layout plan, WASM imports/exports, wrapper manifest | Jet lost type information or emitted a WASM runtime that cannot represent the typed frontend IR correctly. |

The same source fixture should be able to record both DOM and WASM evidence, but
the success criteria are target-specific. A DOM build is allowed to erase
TypeScript types after analysis. A WASM build is not.

`jet bb` has two responsibilities:

1. Replace Playwright/Cypress-style browser operation for Jet tests.
2. Bridge browser-facing APIs into the FE-on-WASM runtime so agents and e2e
   tests can use ordinary browser concepts against a canvas/WebGPU app.

Both `jet serve` and `jet bb` are agent-first surfaces. Their defaults should
optimize for unattended Codex/agent runs: start in the background, return
structured session metadata, keep logs/artifacts addressable, and avoid
requiring a visible browser window or foreground terminal. Human-facing
foreground, attached, or headful modes are explicit inspection modes, not the
default workflow.

`jet test`, `jet e2e`, and `jet trace` are also split by toolchain layer:

| Surface | Basic: FE-on-DOM | Advanced: FE-on-WASM |
|---|---|---|
| `jet test` | Fast local TS test runner for source, modules, transforms, components, and contracts. Provides Jet's built-in TS test API as a virtual module such as `jet:test` during test resolution, without adding a dependency to `package.json` or publishing an npm package. Browser is not started by default. | Runs WASM runtime, lowering, renderer, and bridge-contract tests with the same Jet test API. Uses target/matrix options when a test must compare DOM behavior with WASM behavior. |
| `jet e2e` | Product-flow runner over the Basic DOM app. Orchestrates `jet serve` and `jet bb`, drives browser gestures, and records behavior evidence. | Product-flow parity runner. Executes the same flows against DOM oracle and WASM target, then compares screenshots/pHash, selection, clipboard, input, focus, scroll, and runtime capture. |
| `jet trace` | Evidence substrate for test/e2e runs: test report, server logs, console, network, HMR events, DOM snapshots, screenshots, and replay metadata. | Adds WASM debug tree, WebGPU/runtime status, paint/layout/text artifacts, Browser Bridge metadata, pHash comparison, and DOM-vs-WASM diff diagnostics. |

The second responsibility is why WASM cannot be "just pixels." The Advanced
runtime needs a practical browser-behavior subset inside WASM: layout, text,
selection, focus, clipboard, scroll, input, accessibility evidence, and event
semantics where Jet claims support. The host DOM should stay a thin wrapper and
API anchor; the app's behavior and visual tree should live in WASM. For parity
work, `jet bb capture --surface dom` is the DOM oracle and `jet bb capture
--surface wasm` is the FE-on-WASM observation bundle.

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
- **DOM TypeScript erasure** — the FE-on-DOM build path strips or lowers
  TypeScript-only syntax while emitting browser-native JavaScript
- **WASM typed lowering** — the FE-on-WASM build path preserves TypeScript type
  information in the frontend IR so Rust/WASM codegen can specialize data
  representation, memory layout, and runtime dispatch
- **Module resolution** — full Node.js algorithm with `exports` field support
- **Dependency graph** — petgraph-based with cycle detection and topological sort
- **Parallel transformation** — Rayon-based concurrent module processing
- **Single-file bundle** — `__jet__` runtime module system
- **Frontend IR normalization** — TS/TSX, HTML-like host structure, CSS inputs,
  type facts, and frontend assets normalize into a shared typed TS/HTML/CSS
  frontend IR before target-specific emission
- **Target emitters** — the default DOM emitter produces JS/HTML/CSS artifacts;
  the WASM emitter lowers the same frontend IR into Rust/WASM runtime artifacts
  plus a thin wrapper shell

### Serve

- **Axum HTTP server** — static file serving, SPA fallback
- **WebSocket HMR** — `/__jet_hmr` endpoint with auto-reconnect
- **File watching** — `notify` crate with smart filtering
- **Development serving** — target command `jet dev` starts full foreground dev
  serving with hot reload, HMR, browser client connection, console/log intake,
  proxy, and test/debug hooks
- **Agent-first default** — serving starts as a detached/background session and
  returns machine-readable URL/session metadata for follow-up commands
- **Production serving** — target command `jet serve --prod` serves production
  artifacts as a Kubernetes/GKE static frontend origin server behind a load
  balancer
- **WASM serving** — target command `jet serve --wasm` builds and serves the
  Advanced FE-on-WASM target with runtime inspection hooks

#### Dev Proxy

`jet dev` supports reverse proxy rules for API, webhook, MCP, and WebSocket
backends during DOM local development:

```toml
[dev.proxy]
"/api" = "http://127.0.0.1:3200"
"/mcp" = "http://127.0.0.1:3210"
```

One-off CLI overrides are repeatable and override matching config prefixes:

```bash
jet dev --proxy /api=http://127.0.0.1:3200 --proxy /events=http://127.0.0.1:3300
```

Proxy matching is path-segment aware, so `/api` matches `/api/users` but not
`/apidocs`. The dev proxy preserves HTTP methods and headers, streams SSE, and
tunnels WebSocket connections; it is intentionally a dev-only control-plane
feature, not part of the `jet serve --prod` static data plane.

#### Production Serving Boundary

`jet serve --prod` is scoped as the app-container origin for frontend static
assets in a Kubernetes cluster, not as the public edge proxy. In a GKE-style
deployment, the external load balancer, Ingress, Gateway, CDN, or service mesh
owns TLS termination, certificate rotation, public virtual-host/SNI routing,
cross-service path routing, WAF/auth policy, and HTTP/2 or HTTP/3 edge
negotiation.

Jet owns the behavior that remains inside the pod: fast startup, predictable
bind/port behavior, health/readiness and graceful shutdown, static file serving
from `dist/`, content-type correctness, HTML no-cache policy, immutable cache
policy for hashed assets, SPA fallback, 404 behavior, conditional requests,
range requests for large assets, structured access/error logs, low CPU/memory
overhead, and nginx-class latency/throughput for the same static artifact set.
The comparison target is an nginx container serving the same `dist/` as an
origin, under the same cluster/LB shape; it is not nginx's full edge feature set.

### Browser Bridge

- **Browser automation** — launch/attach to Chromium and drive pointer,
  keyboard, wheel, eval, and screenshot operations through Jet-owned CDP code
- **Agent-first default** — launch is headless and detached by default, returning
  a browser/session handle that follow-up `jet bb` commands can target
- **Semantic snapshot + refs** — `jet bb snapshot` prints a role-annotated tree
  of the live DOM and mints element refs (`e1`, `e2`, …); `click`, `fill`,
  `type`, `hover`, `select`, and `check`/`uncheck` accept a ref or a locator
  selector (CSS, `text=…`, `role=…[name="…"]`), mirroring playwright-mcp's
  snapshot/ref interaction model
- **Navigation** — `goto`, `back`, `forward`, `reload`, `resize`, and `wait`
  (selector / text / delay) against the attached session
- **Page observability** — launch installs init-script rings for console
  output, page errors, and fetch/XHR activity; `jet bb console` and
  `jet bb requests` read (and optionally drain) them from any later command
- **DOM oracle capture** — inspect live DOM targets without Playwright
- **WASM observation bundle** — inspect `window.__jet_debug`, WebGPU status,
  layout tree, element tree, paint ops, hooks, and build manifest
- **Parity diagnostics** — capture comparable screenshots, pHash probes,
  runtime metadata, and browser-semantics evidence
- **MCP server** — `jet bb mcp` serves the full `bb` surface (gestures,
  semantic actions, navigation, observability, capture) as MCP stdio tools
- **Command target** — `jet bb ...`; legacy-compatible `jet browser ...`
  remains available during migration

### Test, E2E, And Trace

- **Built-in TS test API** — `jet test` provides Jet's own TS testing runtime
  during resolution, for example through a virtual module such as `jet:test`.
  Projects should not need to install a Jest/Vitest-style npm package just to
  write Jet-native tests.
- **Basic test layer** — fast unit/module/component/contract tests for the
  FE-on-DOM toolchain; no browser is launched unless the test explicitly asks
  for browser-backed behavior
- **Advanced test layer** — WASM runtime, lowering, renderer, and bridge
  contract tests reuse the same test API instead of creating a separate WASM
  testing package
- **Basic e2e layer** — `jet e2e` orchestrates `jet serve` and `jet bb` for
  product-flow tests against the DOM oracle
- **Advanced e2e layer** — the same flows run as DOM-vs-WASM parity matrices
  when the target includes FE-on-WASM
- **Trace substrate** — `jet trace` is not a second test runner; it stores,
  inspects, replays, and compares evidence produced by `jet test`, `jet e2e`,
  `jet serve`, and `jet bb`

### FE-on-WASM Runtime

- **Wrapper-only host DOM** — the app target uses a host wrapper, bridge
  scripts, and `<canvas id="jet-canvas">`; app widgets are not rendered as
  ordinary DOM nodes
- **WASM-owned browser model** — supported TS/TSX behavior, CSS/layout inputs,
  and HTML-like host nodes are represented inside WASM rather than delegated to
  the browser DOM
- **WebGPU renderer** — layout/paint/text lower to WebGPU-friendly primitives
  and glyph atlas text rendering
- **Browser semantics bridge** — selection, focus, clipboard, scroll, input,
  and accessibility must remain observable through Jet browser APIs even when
  the visible app is WebGPU-backed
- **DOM-oracle parity** — FE-on-WASM must match FE-on-DOM observable behavior
  for the same fixture and gesture

## Commands

```bash
jet init                          # Initialize a new project
jet install                       # Install dependencies from package.json
jet add <package> [--dev]         # Add a dependency
jet remove <package>              # Remove a dependency
jet run [script]                  # Run package scripts/tasks
jet exec <cmd>                    # Run with node_modules/.bin on PATH
jet serve                         # Detached FE-on-DOM dev server compatibility
jet serve --prod                  # Detached production static server replacement
jet serve --wasm                  # Detached FE-on-WASM server with debug bridge
jet serve shutdown                # Stop the active detached Jet serve session
jet dev [-p <port>]               # Foreground FE-on-DOM dev server
jet dev --proxy /api=http://127.0.0.1:3200
jet dev --wasm --debug            # Legacy foreground FE-on-WASM dev server
jet dev shutdown -p <port>        # Legacy stop command for a Jet dev server
jet build                         # Build production DOM target
jet build --wasm                  # Build production FE-on-WASM target
jet test                          # Basic: Jet-native TS tests with built-in test API
jet e2e                           # Basic/Advanced: product-flow and parity e2e
jet trace                         # Basic/Advanced: inspect/replay/compare trace evidence
jet bb launch <url>               # Headless detached Browser Bridge session
jet bb snapshot                   # Ref-annotated semantic DOM snapshot (e1, e2, …)
jet bb click e7                   # Act on snapshot refs or selectors (fill/type/hover/select too)
jet bb goto <url>                 # Navigate; back/forward/reload/resize/wait also available
jet bb console                    # Console/error history; jet bb requests for fetch/XHR
jet bb drag ...                   # Drive browser input without Playwright
jet bb capture --surface dom      # Capture DOM oracle evidence
jet bb capture --surface wasm     # Capture WASM runtime evidence
jet bb mcp                        # Serve the whole bb surface as MCP stdio tools
jet browser ...                   # Legacy-compatible Browser Bridge surface
```

## Architecture

Jet spans the CLI crate, the WASM runtime crate, and shared WebGPU/grid crates:

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
├── wasm_build/             # Frontend source -> Rust/WASM scaffold + dist artifacts
├── browser/                # Chromium/CDP browser driver
├── browser_cli/            # jet browser launch/input/capture/debug surface
├── e2e/                    # Jet-native product-flow e2e runner
└── asset/                  # Asset processing pipeline

projects/jet/wasm/          # jet-wasm runtime crate
crates/cclab-grid-wasm/     # WebGPU browser bridge
crates/cclab-grid-render-webgpu/
                            # Shared WebGPU renderer primitives
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

Use the capability gate inventory and verification contracts below as the
source of truth for readiness. A test result is meaningful only when it names
the contract it evaluated, the oracle it compared against, the fixture or trace
it ran, and the artifact where evidence was recorded.

Jet readiness reports use this result vocabulary:

| Result | Meaning | Production interpretation |
|---|---|---|
| `green` | The gate ran in the required environment and every required assertion passed. | Counts toward verified readiness. |
| `red` | The gate ran and found a Jet behavior, artifact, parity, or performance failure. | Blocks the owning capability until fixed or explicitly rescoped. |
| `yellow` | The gate is implemented but partial, threshold-only, smoke-only, or missing a required comparison dimension. | Does not count as verified readiness for that contract. |
| `gray` | The gate could not run because of environment, auth, browser, network, or fixture prerequisites. | Does not prove readiness; rerun in a valid environment. |

For quick local inventory checks, run:

```bash
cargo test -p jet -- --list
```

### Basic Toolchain Benchmark Contract

Basic FE-on-DOM is compared against mature external systems. These are product
benchmarks, not microbenchmarks. Each run must record cold and warm timings,
artifact size, machine metadata, dependency cache state, command stdout/stderr,
and the exact fixture revision.

Current Basic gate order:

```bash
# Default local goal gate: phase 1 package management, then phase 2 Browser Bridge
# with isolated npm/pnpm and Playwright baselines required.
projects/jet/scripts/verify-basic-dom-gates.sh

# Full phase-1 replacement benchmark; npm/pnpm baselines are required by default.
projects/jet/scripts/verify-basic-dom-gates.sh --phase package

# Full phase-2 replacement benchmark; Playwright baseline is required by default.
projects/jet/scripts/verify-basic-dom-gates.sh --phase browser

# Third-stage build gate, only after package and Browser Bridge are green.
projects/jet/scripts/verify-basic-dom-gates.sh --phase build
```

The `jet-basic-dom` CI workflow mirrors this as three dependent jobs:
`1 package management replacement` -> `2 browser bridge replacement` ->
`3 DOM production build comparison`. The third job is allowed to run only after
the first two replacement contracts pass, so a build comparison can never stand
in for package-manager or browser-automation replacement.

Third-stage production build comparison:

```bash
projects/jet/scripts/verify-basic-dom-gates.sh --phase build
```

This compares the DOM production build corpus across `jet build`, `vite build`,
and `webpack --mode production`-equivalent output. The corpus currently covers
the interactive React bench plus CSS side-effect import, production define
replacement, public asset copying, and CSS-linked HTML output. It records static
artifact function checks, JS syntax checks, browser runtime fixture assertions,
Jet vs Vite runtime trace parity, and performance evidence. The Basic FE-on-DOM
gate runs this comparator with `--runtime-smoke required --build-samples 3`, so
a runtime parity miss or red best-of-N build comparison is a red build gate, not
a benchmark note.

| Jet surface | External baseline | Required comparison | Green condition | Red condition |
|---|---|---|---|---|
| `jet install` | `npm`, `pnpm` | Cold install time, warm install time, lockfile determinism, `node_modules` correctness, downloaded bytes, disk bytes, lifecycle/bin behavior | Jet is correct and within the accepted time/space envelope for the fixture class. The fixture dependency root is created by Jet package management. | Install output differs semantically, lockfile is unstable, lifecycle/bin behavior diverges, or time/space exceeds the accepted envelope. |
| `jet build` | Vite, Webpack | Cold build time, warm rebuild time where supported, JS/CSS asset count, raw/gzip/brotli bundle size, sourcemap presence, public asset output, runtime smoke | Jet output runs correctly and build time/size are within the accepted envelope for the fixture class. | Runtime output differs, required artifacts are missing, or time/size exceeds the accepted envelope. |
| `jet dev` / `jet serve --prod` | Vite dev server, Node static server, nginx container as static origin | Dev startup time, HMR latency, proxy behavior, static first-byte latency, static throughput, cache headers, SPA fallback, production startup time, health/readiness, graceful shutdown, conditional/range requests, access/error logs | Jet dev/prod serving matches required behavior and is within the accepted latency/throughput envelope. For `jet serve --prod`, nginx parity means static-origin behavior inside a Kubernetes pod behind a load balancer. | HMR/proxy/static behavior diverges, prod serving cannot replace the static-origin baseline fixture, latency/throughput exceeds the accepted envelope, or Jet starts owning edge concerns such as TLS termination, public virtual hosts, or cross-service load-balancer routing. |
| `jet test/e2e/trace` | Vitest/Jest plus Playwright/Cypress-style flows | Test startup, watchless run time, browser action fidelity, trace completeness, failure diagnostics, artifact replayability | Jet can run the fixture with equivalent assertions and better or accepted diagnostics/evidence. Browser actions are driven by `jet bb`. | Assertions cannot be represented, browser actions diverge, or trace artifacts are insufficient to reproduce the failure. |

Basic benchmark gates must emit machine-readable summaries. A benchmark is
`yellow`, not `green`, if it only prints timings without comparing against a
named baseline command and threshold. Thresholds are fixture-class specific and
belong in gate inventories or trace metadata, not in ad hoc test prose.

#### Basic Functional And Performance Gate Shape

Basic gates compare function first and performance second. Performance is not
allowed to hide a functional gap: if the generated app, install tree, server
behavior, or test behavior differs from the baseline oracle, the result is
`red` even when Jet is faster.

Each Basic gate runs one Jet command and one or more baseline commands against
the same fixture revision. Baseline commands are treated as external oracles for
functionality and as comparison points for performance. The minimum valid
result shape is:

```json
{
  "contract_id": "basic.build.production",
  "result": "red",
  "fixture": "projects/jet/fixtures/basic/react-production",
  "jet": {
    "command": "jet build",
    "duration_ms": 420,
    "artifacts": {
      "raw_bytes": 123456,
      "gzip_bytes": 34567,
      "brotli_bytes": 30123
    }
  },
  "baselines": [
    {
      "name": "vite",
      "command": "vite build",
      "duration_ms": 390,
      "artifacts": {
        "raw_bytes": 120000,
        "gzip_bytes": 34000,
        "brotli_bytes": 29900
      }
    }
  ],
  "functional": {
    "result": "green",
    "checks": ["runtime-smoke", "asset-manifest", "sourcemap"]
  },
  "performance": {
    "result": "red",
    "threshold": "duration_ratio <= 1.00 and gzip_ratio <= 1.05",
    "duration_ratio": 1.08,
    "gzip_ratio": 1.02
  },
  "evidence": "path/to/basic-build-bench.json"
}
```

The Basic contract inventory is:

Build functional equivalence is layered. It is not byte-for-byte equality,
because hashed filenames, helper ordering, minifier output, and chunk splitting
can legitimately differ across tools. It is also not screenshot equality;
screenshots only prove a narrow rendered state after the build has already
passed lower-level artifact checks. The `basic.build.production` oracle must
record these checks for the FE-on-DOM path. It does not define
`jet build --wasm` success:

| Layer | What is compared | Green condition |
|---|---|---|
| Build manifest | Entry points, emitted files, asset types, public base paths, preload/modulepreload hints, CSS links, sourcemap policy | The app has the same externally loadable entry contract and required artifacts. Filename hashes are normalized before comparison. |
| HTML shell contract | Emitted HTML files, `script`/`link`/`base` tags, module/nomodule policy, preload/modulepreload order, CSP-relevant attributes, root mount node, public-path rewrites | The browser entry shell loads the same app target with equivalent resource ordering and required attributes. Whitespace, generated hashes, and tool-owned comments are normalized. |
| CSS bundle contract | Parsed CSS rules, cascade layer order, media/supports/container queries, imports, font-face rules, CSS modules mappings, URL references, extracted/inlined CSS policy | The shipped style contract is semantically equivalent for supported CSS features, and every referenced font/image/source asset resolves correctly. Formatting, hash names, and minifier ordering noise are normalized. |
| Module graph | Resolved module ids, import/export edges, dynamic imports, externalized deps, side-effect ordering groups | Required reachable modules and chunk boundaries are semantically compatible with the baseline. Exact chunk names and helper module names are normalized. |
| AST/IR shape | Parsed JS, HTML, and CSS output after formatting, hash, sourcemap comment, and bundler helper normalization | Application-level imports, exports, declarations, call sites, JSX/DOM creation intent, HTML resource loading intent, CSS rules, and asset references match the expected normalized shape. |
| Sourcemap contract | Source list, sourceRoot, mappings parseability, sourcesContent policy, generated file references | Devtools can map shipped code back to the expected source files without broken references. |
| Static asset contract | Copied public files, transformed asset references, content type, hash/cache policy inputs | Every referenced asset is present, loadable, and has the expected transformed or copied bytes. |
| Runtime smoke | Browser loads the built app and runs fixture assertions through DOM APIs | The built artifact behaves the same for deterministic fixture assertions. This may include screenshots as evidence, but screenshots are not the primary build oracle. |

AST/IR comparison should be structural and normalized, not textual. The harness
should parse emitted JS, HTML, and CSS; remove tool-owned noise; canonicalize
identifier names for generated helpers; preserve user-facing identifiers,
strings, selectors, and resource attributes; and compare the semantic shape that
a browser or downstream tool observes. A build gate is `yellow` when it has only
runtime screenshots without manifest, HTML shell, CSS bundle, graph, AST/IR,
sourcemap, and asset evidence.

| Contract ID | Jet command | Baseline command(s) | Functional oracle | Performance metrics |
|---|---|---|---|---|
| `basic.install.lockfile` | `jet install` | npm/pnpm lockfile oracle; isolated npm/pnpm benchmark evidence when explicitly run | Resolved package versions, lock determinism, integrity, peer dependency behavior | Cold time, warm time, downloaded bytes, cache hits, disk bytes |
| `basic.install.node-modules` | `jet install` | npm/pnpm `node_modules` oracle; isolated npm/pnpm benchmark evidence when explicitly run | `node_modules` package tree, bin links, lifecycle output, script executability | Cold time, warm time, file count, hardlink/symlink count, disk bytes |
| `basic.build.production` | `jet build` | `vite build`, `webpack --mode production` | Runtime smoke, asset graph, manifest, HTML shell, CSS bundle, sourcemap, public assets | Cold build time, warm build time where supported, raw/gzip/brotli bytes, asset count |
| `basic.build.incremental` | `jet build` or watchless rebuild harness | Vite/Webpack rebuild harness | Changed module reflected correctly; unaffected outputs stable | Single-file edit rebuild time, changed artifact count |
| `basic.serve.dev-start` | `jet dev` | `vite --host 127.0.0.1` | Serves app, SPA fallback, module graph, error overlay/diagnostic surface where supported | Startup-to-ready time, first request latency |
| `basic.serve.hmr` | `jet dev` | Vite dev server | HMR update applies without full reload where supported; state preservation fixture passes | Edit-to-browser-update latency, reload count |
| `basic.serve.prod-static` | `jet serve --prod` | Node static server, nginx container as static origin behind the same load-balancer shape | Static assets, content types, cache headers, compression/precompressed asset policy, SPA fallback, 404 behavior, conditional requests, range requests, health/readiness, graceful shutdown, structured logs. TLS termination, public virtual hosts/SNI, cert management, WAF/auth policy, CDN, and cross-service routing are owned by GKE/LB/Ingress/Gateway and are out of Jet scope. | Startup time, first-byte latency, throughput, p95 latency, CPU/memory envelope |
| `basic.test.runner` | `jet test` | Vitest/Jest fixture command | Same pass/fail assertions, snapshots where supported, reporter semantics | Test discovery time, run time, reporter artifact bytes |
| `basic.e2e.browser` | `jet e2e` / `jet bb` | Playwright/Cypress-style fixture oracle | Same product-flow assertions and browser actions; replayable failure evidence; no Playwright executor in Jet-owned gates | Browser startup time, action latency, trace artifact size |
| `basic.trace.artifacts` | `jet trace` | Playwright trace / custom fixture baseline | Replays or explains the same failure with enough logs/screenshots/network/state | Capture overhead, artifact size, replay time |

Initial green thresholds are intentionally explicit and can be tightened per
fixture class:

| Surface | Initial performance green threshold |
|---|---|
| Install | Jet functional parity is green and median cold/warm time is not slower than the fastest baseline by more than `25%`; disk bytes are not larger than the smallest correct baseline by more than `15%`. |
| Build | Jet functional parity is green, median cold build time is not slower than the fastest baseline by more than `25%`, and gzip output is not larger than the smallest correct baseline by more than `5%`. |
| Dev serve | Jet functional parity is green, startup-to-ready is not slower than Vite by more than `25%`, and HMR edit-to-update p95 is not slower than Vite by more than `25%`. |
| Prod serve | Jet functional parity is green, first-byte p95 is no more than `10%` slower than the configured nginx static-origin baseline, throughput is at least `90%` of that baseline, and CPU/memory stay within the accepted fixture envelope. |
| Test/e2e/trace | Jet functional parity is green, run time is within `25%` of the baseline, and trace artifacts are complete enough to replay or diagnose the seeded failure. |

These thresholds are not claims that Jet is currently green. They define how a
gate decides green/red once the benchmark harness emits data. If a fixture lacks
one of the required baseline commands, that run is `gray`; if it omits a metric
or only compares against Jet's previous run, that run is `yellow`.

### Advanced WASM Behavior Contract

Advanced FE-on-WASM uses the Basic FE-on-DOM toolchain as the external behavior
oracle where the browser platform is deterministic. Where native browser or OS
policy is intentionally variable, Jet defines an explicit cross-platform
policy. The source hierarchy is:

```text
WPT behavior > web spec algorithm > ARIA/APG widget convention
> Chrome/Safari/Firefox observation > Jet cross-platform policy
```

The WASM contract is trace-driven. A trace contains setup, input events,
animation frames, controlled time advances, semantic assertions, screenshot or
pHash assertions when useful, and captured evidence from both surfaces. One
trace should run against DOM and WASM unless the contract is WASM-internal.

The `jet build --wasm` functional oracle is typed-lowering first. It is not a
JS bundle comparison and it must not accept a type-erased JS bundle as the
canonical source for WASM generation. It must record these checks:

| Layer | What is compared | Green condition |
|---|---|---|
| Typed frontend IR | Type facts, inferred/declared TS shapes, host tree IR, CSS IR, resource graph, diagnostics | The typed IR matches the source fixture contract and preserves every type fact the WASM emitter relies on. |
| Type-to-Rust mapping | TS primitives, object/tuple/array shapes, discriminated unions, component state, event payloads, resource handles | The generated Rust representation is explicit, stable, and uses concrete scalar/struct/arena layouts where supported. |
| WASM module contract | Exports/imports, initialization hooks, memory sections, debug metadata, panic/error surface | The module can be loaded by the Jet wrapper and exposes the runtime hooks required by `jet bb` and `jet e2e`. |
| Wrapper shell contract | Thin HTML/CSS/JS loader, canvas/WebGPU attachment, asset preload policy, bridge bootstrap | Wrapper JS only loads and bridges the WASM runtime; app semantics remain in WASM-owned state and code. |
| Runtime parity smoke | DOM oracle trace compared with WASM trace after load | Supported user-visible behavior matches DOM for the fixture; screenshot/pHash may support the result but cannot replace typed-lowering evidence. |

| Contract family | Source of truth | Observable evidence | Example green check |
|---|---|---|---|
| Layout and style | WPT/spec where available; DOM oracle for supported fixture shapes | DOM rects, WASM layout tree, paint ops, screenshot/pHash | Same supported table cells, text boxes, clipping, and visible geometry within tolerance. |
| Pointer and wheel events | Pointer/UI Events plus DOM oracle | Browser Bridge input log, event dispatch log, state transition trace | Same target cell/element, same prevented/default behavior, same resulting state. |
| Scroll containers | CSS Overflow/CSSOM View for offsets; Jet policy for overlay scrollbar visibility | `scrollTop`, `scrollLeft`, max offsets, thumb rects, visibility state, frame timestamps | Offsets clamp to `[0, max]`; thumb position follows the formula; overlay scrollbar appears during scrolling and hides after the configured idle window. |
| Selection and clipboard | Selection API/Clipboard API plus DOM oracle and Jet policy for canvas selection | Selection range, selected cells/text, copied TSV/text, clipboard write status | Same selected range and copied payload for the same drag/key trace. |
| Focus and keyboard | HTML/UI Events/ARIA/APG plus DOM oracle | Active element/focus proxy, key events, app state, accessibility evidence | Same focus target, key handling, and state changes for supported widgets. |
| Text and fonts | CSS text/font specs plus Jet font policy | Text runs, glyph count, atlas status, screenshot/pHash, copied text | Same visible text content and acceptable glyph geometry for the configured font policy. |
| Accessibility evidence | ARIA/APG and Jet bridge policy | Role/name/state snapshot or declared unsupported marker | Supported widgets expose comparable role/name/state; unsupported surfaces fail explicitly. |
| Runtime performance | Jet benchmark policy | Post-load repaint CPU/GPU time, frame count, coalescing counters, input-to-frame latency | After first render, scroll/selection/input stay within the fixture-class steady-state budget. |

#### Advanced WASM Browser Subset Inventory

The Advanced runtime is large because it emulates a browser subset inside WASM.
Each row below is a contract target. Test reports should key results by
`contract_id` so a run can say exactly which rows are `green`, `red`, `yellow`,
or `gray`.

| Contract ID | Surface | Contract | Oracle / policy | Required evidence |
|---|---|---|---|---|
| `wasm.event.raf` | Event loop | `requestAnimationFrame` schedules paint work once per frame and coalesces repeated state/input updates. | DOM oracle plus Jet RAF policy | Input trace, RAF frame log, repaint request/coalescing counters |
| `wasm.event.microtask` | Event loop | Promise/microtask ordering observable by supported app code matches DOM for supported cases. | HTML event loop / DOM oracle | Trace with task, microtask, state flush, and rendered state |
| `wasm.event.timer` | Event loop | Timers used by supported UI behavior fire through a controllable virtual clock in tests. | Jet virtual-clock policy | Trace with `advanceTime(ms)`, timer fire log, final state |
| `wasm.state.flush` | React-like runtime | State updates batch and flush at the same observable boundaries as the Basic DOM oracle for supported hooks. | DOM oracle | Hook state log, rendered state, event/RAF boundary evidence |
| `wasm.host.node-tree` | Host tree | Supported HTML-like nodes, ids, classes, attrs, ARIA attrs, and text children exist in the WASM host model. | DOM oracle for supported nodes | DOM capture, WASM element/layout/debug tree, diff output |
| `wasm.host.event-target` | Host tree | Hit testing maps pointer coordinates to the same supported host target as the DOM oracle. | DOM oracle | Pointer trace, target id/tag/cell coordinate, prevented/default status |
| `wasm.layout.block` | Layout/CSS | Supported block flow, margin, padding, border, width, height, and max-width geometry match within tolerance. | CSS specs/WPT where available; DOM oracle | DOM rects, WASM layout tree, geometry diff |
| `wasm.layout.table` | Layout/CSS | Supported `table/tbody/tr/td` geometry, row stacking, cell sizing, clipping, and text placement match within tolerance. | DOM oracle plus Jet table subset | DOM rects, WASM layout tree, visible cell inventory |
| `wasm.layout.flex` | Layout/CSS | Supported flex direction, alignment, justify, and sizing behavior matches within tolerance. | CSS Flexbox/WPT; DOM oracle | DOM rects, WASM layout tree, geometry diff |
| `wasm.layout.overflow` | Layout/CSS | Overflow clips children and exposes scrollable content extent for supported containers. | CSS Overflow/CSSOM View | Clip rects, content extent, max scroll offsets |
| `wasm.scroll.offset` | Scroll | `scrollTop`/`scrollLeft` clamp to `[0, max]`; wheel and keyboard scroll update offsets deterministically. | CSSOM View plus Jet policy | Wheel/key trace, offset sequence, max offset evidence |
| `wasm.scroll.overlay-scrollbar` | Scroll | Overlay scrollbar is hidden while idle, visible during scroll, hides after the idle window, and uses deterministic thumb geometry. | Jet cross-platform policy | Virtual-clock trace, visibility state, thumb rect, paint ops |
| `wasm.paint.z-order-clip` | Paint/WebGPU | Paint order, clipping, background, border, and selection overlays match the supported DOM visual order. | DOM oracle plus Jet paint policy | Paint op stream, screenshot/pHash, visual diff |
| `wasm.paint.text-glyphs` | Paint/WebGPU | Text runs lower to glyph atlas output with stable visible text and acceptable glyph geometry. | CSS font/text specs plus Jet font policy | Text runs, glyph count, atlas status, screenshot/pHash |
| `wasm.input.pointer-click` | Input | Click/down/up sequences dispatch to the same supported target and produce the same app state. | Pointer/UI Events; DOM oracle | Browser Bridge event log, target, state diff |
| `wasm.input.pointer-drag` | Input | Drag sequences preserve capture/selection semantics for supported table/canvas interactions. | DOM oracle plus Jet policy | Drag trace, selected range, pointer state, repaint counters |
| `wasm.input.wheel` | Input | Wheel events scroll the same supported container, prevent default when appropriate, and coalesce repaint work. | UI Events/CSSOM View; DOM oracle | Wheel trace, target container, offset, coalescing counters |
| `wasm.input.keyboard` | Input | Supported key events update focus, selection, forms, and shortcuts like copy with DOM-equivalent observable results. | UI Events/HTML/ARIA; DOM oracle | Key trace, focus state, selection/form state, clipboard state |
| `wasm.contextmenu.default` | Context menu | Right-click dispatches to the WASM logical target and shows Jet's default semantic menu for that target when app code does not override it. | Jet cross-platform policy plus DOM target oracle | Contextmenu trace, logical target, default menu item list, enabled/disabled state |
| `wasm.contextmenu.override` | Context menu | If app code cancels `contextmenu` or renders a custom menu, Jet suppresses the default semantic menu and reports the app-owned menu state. | DOM event semantics plus Jet policy | Contextmenu trace, `preventDefault` status, app menu evidence, suppressed default flag |
| `wasm.focus.active` | Focus | WASM exposes a focus proxy equivalent to DOM active element for supported focusable widgets. | HTML focus model; DOM oracle | Active target, tab order trace, focus/blur event log |
| `wasm.selection.cell-range` | Selection | Table/spreadsheet drag selection tracks anchor/focus/range and highlights the same supported cells. | DOM oracle plus Jet canvas selection policy | Selection range, visible selected cells, paint ops, screenshot/pHash |
| `wasm.clipboard.copy` | Clipboard | Copy writes the same text/TSV payload and exposes write success/failure state. | Clipboard API plus Jet policy | Selected payload, clipboard write status, copied text evidence |
| `wasm.form.input` | Forms | Controlled text input value, selection range where supported, composition limits, and change events match DOM for supported cases. | HTML forms; DOM oracle | Input trace, value/state diff, event log |
| `wasm.form.checkbox-radio` | Forms | Checked state, click/key toggles, and change events match DOM for supported checkbox/radio cases. | HTML forms; DOM oracle | Pointer/key trace, checked state, event log |
| `wasm.form.textarea-select` | Forms | Supported textarea/select value and change behavior match DOM or fail as explicitly unsupported. | HTML forms; DOM oracle | Value diff, event log, unsupported marker when out of scope |
| `wasm.a11y.role-name-state` | Accessibility | Supported widgets expose comparable role, name, value, selected, checked, focused, and disabled state through Jet evidence. | ARIA/APG plus Jet bridge policy | A11y snapshot or explicit unsupported marker |
| `wasm.api.browser-bridge` | Browser bridge | `jet bb` can drive and observe WASM through browser-like APIs without Playwright-only escape hatches. | Jet Browser Bridge policy | Command transcript, session metadata, capture bundle |
| `wasm.trace.replay` | Trace | A recorded DOM/WASM parity trace can replay deterministically and preserve enough evidence to debug red rows. | Jet trace policy | Trace JSON, artifact manifest, replay result, diagnostics |
| `wasm.visual.phash` | Visual parity | Screenshot/pHash is available as an auxiliary visual check for supported fixtures. | DOM oracle plus Jet tolerance policy | DOM/WASM screenshots, pHash distance, visual diff |
| `wasm.perf.post-load` | Runtime performance | After first render, steady-state scroll/selection/input meet the fixture-class frame and latency budgets. | Jet benchmark policy | CPU/GPU timing, input-to-frame latency, frame count, coalescing counters |

Each advanced gate should emit a structured result shaped like:

```json
{
  "contract_id": "wasm.scroll.overlay-scrollbar",
  "result": "red",
  "oracle": "jet-policy",
  "fixture": "examples/mui-visual-demo",
  "trace": "large-table-scrollbar-idle-hide",
  "evidence": "path/to/trace-artifact.json",
  "reason": "scrollbar remained visible after idle timeout"
}
```

Rows with no implemented trace are `yellow` rather than `green`. Rows that the
current runtime does not intend to support must be listed as explicit
unsupported evidence; silent absence is a `red` result once a contract row
exists.

Canvas context menus are a policy boundary. The browser's native menu only sees
the host `<canvas>`, so Jet must hit-test the WASM logical tree before deciding
what the user right-clicked. Jet does not need to clone Chrome or macOS menu UI.
It must expose a deterministic default semantic menu for supported logical
targets, such as copy for selected cells/text, edit actions for editable
controls, and link actions for supported links. App code wins when it cancels
the `contextmenu` event or renders its own menu; in that case the Jet default
menu must not appear.

Native overlay scrollbars are a good example of the policy boundary. Browser
and OS settings can change whether a DOM scrollbar is always visible, overlay
only, or hidden until scroll. Jet therefore does not blindly copy host OS
scrollbar visibility. For WASM, the contract is: no scrollbar when content does
not overflow; hidden while idle; visible during wheel/drag/keyboard scrolling;
hidden again after the configured idle window; offsets always clamp; thumb
geometry is deterministic.

Advanced gates are `red` when WASM differs from the Basic oracle or Jet policy,
even if the current WASM implementation is internally consistent. They are
`yellow` when they only prove a screenshot, a single runtime counter, or one
manual fixture path without replayable trace evidence.

## Basic Oracle, Advanced WASM, And Browser Bridge

Jet treats the Basic FE-on-DOM toolchain as the external behavior oracle for
the Advanced FE-on-WASM toolchain. The two execution paths are allowed to differ
internally: Basic uses real browser DOM nodes and React runtime semantics;
Advanced owns the app's TS/TSX behavior, CSS/layout inputs, and HTML-like host
tree inside WASM, renders through WebGPU, and exposes browser semantics through
Jet-managed bridges. Those implementation differences must not change what a
user, test, or agent can observe.

Parity means Advanced FE-on-WASM must match the Basic FE-on-DOM oracle for:

- rendered text, layout, visual state, and screenshot/pHash evidence;
- pointer, keyboard, wheel, focus, and scroll behavior;
- selection and clipboard behavior, including browser `Selection` observable
  state where the DOM oracle exposes it;
- controlled input and form-control behavior;
- library fixture behavior for supported component surfaces such as MUI and
  AntD;
- Browser Bridge capture output across DOM and WASM surfaces.

The focused WASM and Browser Bridge gates are:

```bash
cargo test -p jet --test wasm_build_end_to_end wasm_build_selects_webgpu_scaffold_by_default -- --nocapture
cargo test -p jet --test wasm_build_end_to_end webgpu_renderer_reports_runtime_status_and_visual_probe_when_available -- --nocapture
cargo test -p jet --test tsx_to_rust_imports -- --nocapture
cargo test -p jet --test mui_visual_regression -- --nocapture
```

The real-package visual regression fixtures are `examples/mui-visual-demo` and
`examples/antd-visual-demo`. They are intentionally outside the simplified unit
gates and exercise both FE-on-DOM and FE-on-WASM with real MUI/AntD source plus
`jet bb capture --surface dom`, `jet bb capture --surface wasm`, and
`jet bb screenshot` CLI evidence:

```bash
cd examples/mui-visual-demo
jet install
cd ../..
cargo test -p jet --test mui_visual_regression -- --nocapture

cd examples/antd-visual-demo
jet install
cd ../..
cargo test -p jet --test mui_visual_regression -- --nocapture
```

The fixture is managed through Jet end to end. Primary vocabulary is
`jet install`, `jet dev`, `jet serve --prod`, `jet serve --wasm`, `jet bb
launch`, `jet bb capture`, `jet bb screenshot`, and shutdown through the
serving layer. `jet dev` owns the local client-connected control plane;
`jet serve --prod` owns the production static data plane. Legacy
`jet browser ...` commands remain compatibility surfaces while tests and
fixtures migrate to the primary command names.

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
| Rust-Native Frontend Toolchain Replacement | #3778 | implemented | verified | smoke, conformance, corpus, negative, dogfood | ready_for_basic | The Basic all-in-one replacement gate is green across package manager, Browser Bridge, production build, serve, workspace, test, e2e, and trace. |
| Package Manager | #3779 | implemented | verified | smoke, conformance, corpus, negative | ready_for_basic | Jet owns fixture hydration and mutation gates; required isolated npm/pnpm benchmark evidence is green for the current Basic corpus. |
| Bundler And Production Build | #3782 | implemented | verified | smoke, conformance, corpus, negative | ready_for_basic | The expanded DOM production build corpus is green with required runtime smoke and Vite/Webpack comparisons. |
| Dev Server And HMR | #3780 | implemented | verified | smoke, conformance, corpus, negative, dogfood | ready | `jet dev` can replace Vite-style local development serving, HMR, browser log intake, and local API/WebSocket proxying for real projects. |
| Workspace And Task Runner | #3781 | implemented | verified | smoke, conformance, corpus, negative | ready | Jet can replace npm scripts, pnpm workspaces, and common Nx/Turborepo task-runner flows through the canonical `jet.toml` configuration surface. |
| Native Test And Product-Flow E2E | #3785 | implemented | verified | smoke, conformance, corpus, negative, dogfood | ready_for_basic | Jet native runner, reporter, product-flow e2e, and trace gates are green for the Basic production-readiness contract. |
| WASM And Multi-Target Execution | #3783 | implemented | passing | smoke, conformance, corpus, negative | partial | Jet can sink the frontend app model into WASM, render it through canvas/WebGPU, and preserve browser-observable semantics through bridges. |
| Browser, Trace, And Parity Infrastructure | #3786 | implemented | verified | smoke, conformance, corpus, negative | ready_for_basic | Jet BB is the executor for current gates, with isolated Playwright baseline evidence and trace substrate tests green. |
| Library Build And Package Publishing | #168 | implemented | verified | conformance | partial | `jet build --lib` (ESM+CJS, externalized deps/peerDeps, multi-entry), `.d.ts` emission, and `jet publish --build` with metadata validation + private-registry (`.npmrc` scoped) e2e all shipped and tested (A1-A3 merged). `partial`: `preserve_modules`/IIFE lib output, class-member `.d.ts` reduction, and some CJS re-export edge cases are TODO follow-ups. |
| Component Workbench (Stories) | #169 | implemented | verified | conformance | partial | CSF `*.stories.tsx` discovery, `jet stories` dev manager + isolated preview, preview HMR, and a prop-type-derived Controls panel all shipped and tested (B1-B3 + B2b merged). `partial`: full hook-state-preserving React refresh, bare-import resolution beyond React, and generic/cross-file prop types are TODO follow-ups; static export (B4) deferred to phase 2. |

## Rust-Native Frontend Toolchain Replacement

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| rust-native-frontend-toolchain | #3778 | verified | Jet is gated as an all-in-one Basic frontend replacement in dependency order: package manager, Browser Bridge, production build, serve, workspace, test, e2e, and trace. The current production-readiness gate is green. | smoke, conformance, corpus, negative, dogfood | `projects/jet/scripts/verify-basic-dom-gates.sh --all`<br>projects/jet/tests/fixtures/dom-production-build |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Production replacement readiness | epic | #3778 | implemented | verified | corpus | package manager -> Browser Bridge -> build -> serve/workspace/test/e2e/trace are green in `verify-basic-dom-gates.sh --all` |
| Full Toolchain Dogfood Flow | epic | #3778 | implemented | verified | dogfood | `projects/jet/scripts/verify-basic-dom-gates.sh --all`<br>projects/jet/tests/fixtures/dom-production-build |

### Package Manager

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| package-manager | #3779 | verified | Jet owns fixture hydration, mutation, workspace, and frozen-lockfile checks; isolated npm/pnpm benchmark evidence is green for the current Basic corpus. | smoke, conformance, corpus, negative | `cargo test -p jet --lib pkg_manager -- --nocapture`<br>`projects/jet/scripts/compare-pkg-management.mjs --baseline-tools npm,pnpm --require-baselines` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Package manager readiness | epic | #3779 | implemented | verified | corpus | `projects/jet/scripts/compare-pkg-management.mjs` |
| Package Manager Lockfile Parity | epic | #3779 | implemented | verified | conformance | `cargo test -p jet --lib pkg_manager::lockfile -- --nocapture`<br>projects/jet/fixtures/pkg-manager/lockfile |
| Package Manager Workspace Parity | epic | #3779 | implemented | verified | conformance | `cargo test -p jet --lib pkg_manager::workspace -- --nocapture`<br>projects/jet/fixtures/pkg-manager/workspace |
| Package Manager Registry Integrity | epic | #3779 | implemented | verified | negative | `cargo test -p jet --lib pkg_manager -- --nocapture`<br>projects/jet/fixtures/pkg-manager/registry |

### Bundler And Production Build

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| bundler-production-build | #3782 | verified | Jet production build replacement is green after package manager and Browser Bridge gates. The expanded DOM production build corpus has green static checks, runtime smoke, and performance/size comparisons for the current fixture set. | smoke, conformance, corpus, negative | `projects/jet/scripts/compare-dom-build-corpus.mjs --runtime-smoke required --build-samples 3`<br>projects/jet/tests/fixtures/dom-production-build |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Bundler production readiness | epic | #3782 | implemented | verified | corpus | DOM production build corpus is green with required runtime smoke and Vite/Webpack comparisons |
| Production Bundle Output Parity | epic | #3782 | implemented | verified | conformance | `cargo test -p jet --lib bundler -- --nocapture`<br>projects/jet/fixtures/bundler/production |
| Transform Resolver Parity | epic | #3782 | implemented | verified | corpus | `cargo test -p jet --lib transform -- --nocapture`<br>projects/jet/fixtures/bundler/transform-resolver |
| Asset Sourcemap Negative Paths | epic | #3782 | implemented | verified | negative | `cargo test -p jet --lib asset -- --nocapture`<br>projects/jet/fixtures/bundler/assets |

### Dev Server And HMR

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| dev-server-hmr | #3780 | verified | `jet dev` can replace Vite-style local development serving and HMR for real projects. Dev mode prioritizes a connected browser client, HMR, browser log intake, and dev-only reverse proxy rules from `[dev.proxy]` in `jet.toml` or repeatable `--proxy PATH=URL` CLI overrides. `jet serve --prod` is a separate Kubernetes/GKE static frontend data plane behind a load balancer, with nginx-class static serving behavior and a hot path tuned for low memory-copy overhead and high RPS; it does not own TLS termination, public virtual hosts, cert management, WAF/CDN, or cross-service ingress routing. Current local proof includes prod static serving versus nginx with first-byte p95 ratio `0.803` and throughput ratio `1.164`. | smoke, conformance, corpus, negative, dogfood | projects/jet/fixtures/dev-server/basic-hmr<br>projects/jet/fixtures/dev-server/react-refresh/state-preserved<br>projects/jet/fixtures/dev-server/prebundle-importmap<br>`cargo test -p jet --lib dev_server -- --nocapture`<br>`cargo test -p jet --lib dev_server::proxy -- --nocapture`<br>`cargo test -p jet --lib cli::e2e_command_contract_tests -- --nocapture`<br>`projects/jet/scripts/compare-prod-static-serve.mjs --jet-bin target/release/jet` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Dev server replacement readiness | epic | #3780 | implemented | verified | dogfood | jet dev is the client-connected dev control plane; jet serve --prod is the static data plane |
| Dev Server Local Serving Hmr | epic | #3780 | implemented | verified | conformance | `cargo test -p jet --lib dev_server -- --nocapture`<br>projects/jet/fixtures/dev-server/basic-hmr |
| Dev Server Proxy Contract | epic | #3780 | implemented | verified | conformance | `cargo test -p jet --lib dev_server::proxy -- --nocapture` |
| Dev Server Cli Contract | epic | #3780 | implemented | verified | conformance | `cargo test -p jet --lib cli::e2e_command_contract_tests -- --nocapture` |
| React Refresh State Preserved | epic | #3780 | implemented | verified | conformance | `cargo test -p jet --lib dev_server::hmr -- --nocapture`<br>projects/jet/fixtures/dev-server/react-refresh/state-preserved |
| Prebundle Importmap Parity | epic | #3780 | implemented | verified | corpus | `cargo test -p jet --lib dev_server::prebundle -- --nocapture`<br>projects/jet/fixtures/dev-server/prebundle-importmap |

### Workspace And Task Runner

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| workspace-task-runner | #3781 | verified | Jet workspace/task-runner replacement remains part of the package-management replacement track before build claims. The canonical project configuration file is `jet.toml`, and the active schema artifact is `schemas/jet.schema.json`. | smoke, conformance, corpus, negative | projects/jet/fixtures/task-runner/graph-cache<br>projects/jet/fixtures/workspace/package-selection<br>projects/jet/fixtures/task-runner/nx<br>`cargo test -p jet --lib task_runner -- --nocapture`<br>`cargo test -p jet --lib task_runner::config::tests -- --nocapture`<br>`cargo run -p jet -- config schema --check` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Workspace task runner readiness | epic | #3781 | implemented | verified | corpus | package/workspace replacement track plus canonical jet.toml config, lint, and schemas/jet.schema.json schema artifact |
| Task Runner Graph Cache | epic | #3781 | implemented | verified | conformance | `cargo test -p jet --lib task_runner -- --nocapture`<br>projects/jet/fixtures/task-runner/graph-cache |
| Workspace Package Selection | epic | #3781 | implemented | verified | conformance | `cargo test -p jet --lib pkg_manager::workspace -- --nocapture`<br>projects/jet/fixtures/workspace/package-selection |
| Nx Graph Parity | epic | #3781 | implemented | verified | corpus | `cargo test -p jet --lib pkg_manager::nx -- --nocapture`<br>projects/jet/fixtures/task-runner/nx |

### Native Test And Product-Flow E2E

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| native-test-product-flow-e2e | #3785 | verified | Jet native tests, reporter artifacts, product-flow e2e, and trace evidence are green in the Basic production-readiness gate. | smoke, conformance, corpus, negative, dogfood | projects/jet/fixtures/test-runner/native<br>projects/jet/fixtures/test-runner/reporters<br>projects/jet/fixtures/e2e/product-flow<br>projects/jet/fixtures/e2e/trace-replay<br>`cargo test -p jet --lib test_runner -- --nocapture`<br>`cargo test -p jet --lib reporter -- --nocapture`<br>`cargo test -p jet --lib e2e -- --nocapture`<br>`cargo test -p jet --lib trace -- --nocapture` |

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

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| wasm-multi-target | #3783 | auditing | Jet can sink the frontend app model into WASM only after Basic package management, Browser Bridge, and DOM production build contracts are stable enough to reuse. | smoke, conformance, corpus, negative | projects/jet/fixtures/wasm/build-dev<br>projects/jet/fixtures/wasm/runtime-subset<br>projects/jet/fixtures/wasm/renderer-targets<br>`projects/jet/scripts/verify-advanced-wasm-gates.sh` |

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

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| browser-trace-parity | #3786 | auditing | Jet Browser Bridge, trace, and parity diagnostics are the second Basic replacement gate and the evidence substrate for later DOM/WASM parity. | smoke, conformance, corpus, negative | `projects/jet/scripts/verify-browser-bridge-replacement.mjs --jet-bin target/release/jet`<br>projects/jet/fixtures/trace/artifacts<br>projects/jet/fixtures/browser/automation-diagnostics<br>projects/jet/parity/** |

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

### Library Build And Package Publishing

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| library-build-publishing | #168 | implemented | jet builds publishable npm packages in library mode (ESM + optional CJS, externalized dependencies/peerDependencies, multi-entry from package.json `exports`), emits `.d.ts` type declarations, and `jet publish --build` builds + validates package metadata (`exports`/`main`/`module`/`types`) before publishing to public or private (GitLab/Verdaccio/Nexus) registries via `.npmrc` scoped-registry auth. App-mode `jet build` is unchanged. | smoke, conformance, corpus, negative | `cargo test -p jet --test library_build`<br>`cargo test -p jet --test library_dts`<br>`cargo test -p jet --test library_publish_e2e`<br>`cargo test -p jet --lib bundler::lib_build bundler::dts`<br>`cargo test -p jet --lib pkg_manager::publish` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Library publishing readiness | epic | #168 | implemented | verified | conformance | A1 `jet build --lib` -> A2 `.d.ts` emission -> A3 publish + private-registry hardening (all merged) |
| Library Build Mode | change | #170 | implemented | verified | conformance | `cargo test -p jet --test library_build` — ESM+CJS, externalized deps/peerDeps, multi-entry (preserve-modules/IIFE TODO) |
| Type Declaration Emission | change | #171 | implemented | verified | conformance | `cargo test -p jet --test library_dts` — `.d.ts` per entry + `types` field (isolatedDeclarations) |
| Publish And Private Registry | change | #172 | implemented | verified | conformance | `cargo test -p jet --test library_publish_e2e` — build + metadata validate; in-process mock-registry publish/install round-trip |

### Component Workbench (Stories)

| ID | Root WI | Status | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|
| component-workbench | #169 | implemented | jet discovers and parses CSF `*.stories.tsx` (default-export meta + named-export stories), serves a jet-native manager UI (sidebar, isolated preview, toolbar) with HMR, and derives a live Controls panel from component prop types + `argTypes`. CSF-compatible with no Storybook runtime dependency; `jet stories build` static export is deferred to phase 2. | smoke, conformance, corpus, negative | `cargo test -p jet --test csf_discovery`<br>`cargo test -p jet --test manager`<br>`cargo test -p jet --test preview_hmr`<br>`cargo test -p jet --test controls`<br>`cargo test -p jet --lib stories` |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Component workbench readiness | epic | #169 | implemented | verified | conformance | B1 CSF discovery -> B2 `jet stories` manager -> B2b preview HMR -> B3 controls (all merged) |
| CSF Story Discovery | change | #173 | implemented | verified | conformance | `cargo test -p jet --test csf_discovery` — glob + CSF3 meta/named-story parse into a story index |
| Stories Dev Manager | change | #174 | implemented | verified | conformance | `cargo test -p jet --test manager` — `jet stories` command + manager UI + isolated per-story preview |
| Stories Preview HMR | change | #176 | implemented | verified | conformance | `cargo test -p jet --test preview_hmr` — watcher + WS, preview re-render/reload, manager untouched (full hook-state refresh TODO) |
| Stories Controls Panel | change | #175 | implemented | verified | conformance | `cargo test -p jet --test controls` — prop-type-inferred controls + `argTypes` override; live arg edits re-render the preview |
