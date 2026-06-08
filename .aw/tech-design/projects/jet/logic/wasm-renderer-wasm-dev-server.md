---
id: projects-jet-logic-wasm-renderer-wasm-dev-server-md
fill_sections: [changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# jet — `jet dev --wasm` (WASM development loop)

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: ".aw/tech-design/projects/jet/logic/wasm-renderer-wasm-dev-server.md"
    action: modify
    section: doc
    impl_mode: hand-written
    description: |
      Legacy Jet TD content retained as notes during AW standardization.
      Rewrite this file into semantic TD sections before promoting source to CODEGEN.
```

## Legacy notes
<!-- type: doc lang: markdown -->

# jet — `jet dev --wasm` (WASM development loop)

### Overview

`jet dev --wasm` is a single-command development loop for the
TSX → WASM pipeline, separate from the pre-existing JS-bundle
`jet dev` path. It runs an initial `wasm_build::build_with_profile`,
serves `dist/` as static files over axum, and watches `src/**/*.tsx`
+ `jet.config.toml` to rebuild on change.

Live browser reload over WebSocket is out of scope for this cut —
the user presses Cmd-R after "rebuild ok" prints. A WS channel
would plug into `boot.js` but is deferred (M5 in
`logic/wasm-renderer-architecture.md`'s phased map).

### Why a separate module

`crates/jet/src/dev_server/` is ~7,700 lines of infrastructure
tailored to the JS bundler path — HMR, module graph, import maps,
prebundle, proxies. None of it applies to WASM output (one `.wasm`
binary + one JS glue file + one HTML host). Reusing that module
would have been massive over-engineering.

`crates/jet/src/wasm_dev/mod.rs` is ~260 lines and does exactly
three things: (1) build, (2) serve, (3) watch-and-rebuild. Feature
overlap with the JS-bundle dev server is effectively zero — they
share only `axum` and `notify` as third-party deps, which are
workspace-level already.

### CLI surface

```yaml
$schema: https://json-schema.org/draft/2020-12/schema
title: jet dev (wasm mode)
description: >
  Arguments added to the pre-existing `jet dev` subcommand for
  WASM mode. `--wasm` short-circuits the JS-bundle path entirely;
  `--host` / `--port` are shared between both modes.
type: object
properties:
  wasm:
    type: boolean
    description: Enable WASM mode — build via [wasm] section of jet.config.toml.
    default: false
  debug:
    type: boolean
    description: >
      Dev-profile wasm-pack build (DWARF retained) + jet-wasm
      `debug` feature on (window.__jet_debug lives). Only
      meaningful with `--wasm`.
    default: false
  host:
    type: string
    default: 127.0.0.1
  port:
    type: integer
    default: 3000
```

### Request routing

```yaml
$schema: https://json-schema.org/draft/2020-12/schema
title: wasm_dev routes
description: >
  Intentionally tiny surface. All responses use `cache-control: no-store`
  so a refresh always picks up the latest rebuild.
type: object
properties:
  "GET /":
    description: Serve dist/index.html
    response:
      content-type: text/html; charset=utf-8
  "GET /{*path}":
    description: >
      Serve dist/<path>. `..` components are rejected (404) — not
      `path.canonicalize()` + ancestor check, but the simpler
      rule that any `..` in the request path is disallowed.
    response:
      content-type: |
        per file ext:
          .html → text/html; charset=utf-8
          .js / .mjs → application/javascript; charset=utf-8
          .wasm → application/wasm
          .css → text/css; charset=utf-8
          .json / .map → application/json; charset=utf-8
          .svg → image/svg+xml
          .png → image/png
          .jpg / .jpeg → image/jpeg
          .ico → image/x-icon
          default → application/octet-stream
      headers:
        cache-control: no-store
    errors:
      404: File missing OR path contained `..`.
```

### Boot → serve state machine

```mermaid
---
id: jet-wasm-dev-server-state
initial: InitialBuild
---
stateDiagram-v2
    [*] --> InitialBuild: jet dev --wasm [--debug]
    InitialBuild --> BuildFailed: wasm_build::build_with_profile() err
    BuildFailed --> [*]: exit 1 (fatal — nothing to serve)
    InitialBuild --> BindSocket
    BindSocket --> StartWatcher: TcpListener on host:port
    StartWatcher --> Serve
    Serve --> Rebuild: src/**/*.tsx or jet.config.toml changed<br/>(150 ms debounce)
    Rebuild --> Serve: build ok — log "refresh the browser"
    Rebuild --> Serve: build failed — log error, keep serving last-good dist
    Serve --> Shutdown: Ctrl-C
    Shutdown --> [*]: axum graceful shutdown
```

Key property: **a failed rebuild doesn't kill the dev server.**
Users see the compile error in their terminal, keep editing, hit
save again. `dist/` retains the last successful build until a
rebuild succeeds.

### Watcher scope + debounce

- Only two paths are watched:
  - `<root>/src/` (recursive).
  - `<root>/jet.config.toml` (non-recursive).
- Watching the repo root would trigger constantly on the rebuild's
  output in `<root>/.jet/wasm-build/` and create an infinite loop.
- Extension filter inside the raw-event handler rejects everything
  except `.tsx` / `.ts` / `.toml`. A `.DS_Store` change doesn't
  wake the builder.
- Debounce: 150 ms of silence after the last event in a burst
  before the rebuild fires. Editors that save-then-rename produce
  multi-event bursts; debouncing avoids doubled builds.

### Profile plumbing

`DevOptions { debug: bool }` is the only new CLI-controlled knob.
Internally:

```
--debug → wasm_build::Profile::Dev
  → scaffold_cargo_project writes [jet-wasm = features = [
      "react", "canvas", "canvas-app", "debug"
     ]]
  → run_wasm_pack uses --dev (not --release) → DWARF retained
  → on startup: print "install C/C++ DevTools for WebAssembly in
     Chromium for Rust source stepping"
```

Release-profile dev mode (`jet dev --wasm` without `--debug`) works
too — just builds faster, strips symbols, and `window.__jet_debug`
is undefined. Useful for iterating on layout / styling without the
debug overhead.

### Error modes

| Failure | Behaviour |
|---|---|
| `wasm_build::build_with_profile` fails on startup | Fatal. Exit 1 with anyhow context. Nothing to serve. |
| Rebuild fails after a user edit | Logged to stderr. Server keeps running. Last-good `dist/` stays deployed. |
| Port in use | `TcpListener::bind` error bubbles up; suggested remedy: `--port N`. |
| `src/` missing at startup | Watcher init skips the path silently; only `jet.config.toml` gets watched. Edge case — production apps always have `src/`. |
| File-watcher event channel saturates | `tokio::sync::mpsc::unbounded_channel` — no backpressure today. If it ever saturates, the pattern is broken somewhere (typically a watch target deep inside a build output dir). |

### Follow-ups

- **WebSocket live-reload** — `boot.js` opens a WS back to the
  dev server; rebuild success broadcasts a reload message. About
  100 lines of axum WS handler + 20 lines in `boot.js`. M2.5.
- **HMR (module replacement)** — significantly harder; requires
  WASM module refresh without losing `MountHandle` state. Probably
  never worth it — full reload is fast enough for the WASM hot
  path.
- **Custom host page** — today we always emit a canned
  `index.html` with `#jet-canvas`. A `<root>/public/index.html`
  override would let users bring their own shell.

### Cross-references

- `interfaces/wasm-renderer/debug-bridge.md` — `--debug` unlocks the bridge this server
  ships.
- `tools/wasm-renderer-browser-cli.md` — typical companion for debugging the served
  app.
- `logic/wasm-renderer-transpiler.md` — profile argument threads through
  `wasm_build::build_with_profile` down to the transpiler's
  annotation source-file setting.
- Code: `crates/jet/src/wasm_dev/mod.rs`.
