# `projects/jet/` Layout

This file maps every top-level entry under `projects/jet/` to its role so
new readers can navigate the tree without grepping `Cargo.toml`s.

Naming conventions:

- **Directory name = its short identity** (e.g. `parity/gate`, `tools/conformance`).
- **Cargo package name = `jet-<dir-leaf>`** for every workspace crate. The
  root crate at `src/` is just `jet` (the single shipping binary).
- **`data/`** holds non-code artifacts embedded by the root crate at compile
  time (`include_str!`). Domain-scoped data (parity fixtures) lives next to
  the crates that own it under `parity/data/`.

## Top-level entries

There are nine entries. Everything else is two-deep.

```
projects/jet/
├── src/        — jet root crate (TSX → Rust transpiler + driver)
├── wasm/       — jet-wasm crate (browser-side runtime, wasm32 target)
├── parity/     — parity workspace (corpus, gate, oracle, data)
├── tools/      — auxiliary CLIs (conformance, manifest)
├── tests/      — integration tests for the root crate
├── examples/   — cargo examples for the root crate
├── assets/     — static assets shipped with builds
├── data/       — runtime shims included via include_str!
└── docs/       — project-level docs (architecture, reorg notes)
```

### The two halves of `jet`

`jet` has two architectural halves that **must** live in separate Cargo
crates because they target different `crate-type`s and different rustc
targets:

| Path | Role | crate-type | rustc target |
|------|------|-----------|--------------|
| `src/` | TSX → Rust transpiler, build orchestrator, CLI. The `jet` binary. | rlib + bin | host (native) |
| `wasm/` | Browser-side runtime that the transpiled output links against. | cdylib + rlib | `wasm32-unknown-unknown` |

They are peers, not parent/child.

### Root crate (single binary)

| Path | Role |
|------|------|
| `Cargo.toml` | Root manifest for the `jet` crate. Defines the single `jet` binary + lib re-exports. |
| `src/` | `jet` library + `jet` binary source. DDD-grouped: `e2e/`, `evidence/`, `pm_report/`, `agent/`, plus top-level modules (`runner`, `reporter`, `bundler`, `cli`, ...). |
| `tests/` | Integration tests for the `jet` crate (`cargo test -p jet`). |
| `examples/` | Cargo examples for the `jet` crate (`cargo run --example <name>`). |

### `parity/` — parity workspace

| Path | Cargo package | What it is |
|------|----------------|------------|
| `parity/corpus/` | `jet-parity-corpus` | Fixture-corpus loader + drift verifier. |
| `parity/gate/` | `jet-parity-gate` | CI gate over channel-results. |
| `parity/oracle/` | `jet-parity-oracle` | Headless DOM reference runner library. |
| `parity/data/` | _(not a crate)_ | Fixtures, manifests, schemas, ADRs that the three crates above read. |

### `tools/` — auxiliary CLIs

| Path | Cargo package | What it builds |
|------|----------------|----------------|
| `tools/conformance/` | `jet-conformance-cli` | `jet-conformance` CLI — pnpm/playwright conformance harness. |
| `tools/manifest/` | `jet-manifest-cli` | `jet-manifest` CLI — emits/validates run manifests. |

### `wasm/` — browser runtime crate

| Path | Cargo package | What it builds |
|------|----------------|----------------|
| `wasm/` | `jet-wasm` | `wasm32-unknown-unknown` build entry; produces `pkg/`. |

### Data / assets / docs

| Path | Role |
|------|------|
| `data/runtime/test/` | JS test-runner shims (`index.js`, `page.js`, `matchers.js`) embedded via `include_str!` by `src/test_runner/`. |
| `parity/data/fixtures/` | MUI corpus, parity-grid, etc. |
| `parity/data/schemas/` | JSON schemas for ax-tree, fixture regions, layout boxes. |
| `parity/data/docs/` | ADRs 001–036 + gating-manifest reference. |
| `parity/data/parity-gating.toml` | Default gate manifest. |
| `parity/data/waivers.toml` | Default waivers. |
| `assets/` | Static assets shipped with the renderer. |
| `docs/` | Project-level docs (architecture notes, reorg plan, etc.). |
| `docs/architecture/reorg-plan.md` | Historical reorg plan & success criteria. |
| `README.md` | Crate-level README. |
| `issue-loop.md` | Issue-loop working notes. |

## Quick reference

- **"Where does the `jet` binary live?"** → `src/` (root crate `Cargo.toml`).
- **"What runs in the browser?"** → `wasm/` (`jet-wasm`, sibling crate of `src/`).
- **"Where are parity fixtures?"** → `parity/data/fixtures/`.
- **"Where are the runtime test shims that get `include_str!`'d?"** → `data/runtime/test/`.
- **"Why is `parity/corpus/` next to `parity/gate/`?"** → each is its own
  crate with its own binary; `parity/data/` holds the data they read.

## Removed / consolidated

The following sibling crates were removed during the 2026-05 layout
flatten because the desktop/TUI delivery surfaces were dropped:

- `multi-target/` (`jet-multi-target`) — multi-target dispatch helpers.
- `tauri-shell/` (`jet-tauri-shell`) — Tauri desktop shell.
- `tui-renderer/` (`jet-tui-renderer`) — ratatui terminal renderer.
- `wasm-renderer-poc/` (`jet-wasm-renderer-poc`) — paint-pipeline POC.

`jet build --target` now accepts only `web`.
