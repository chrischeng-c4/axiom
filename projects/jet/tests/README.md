# Jet integration tests — verification blocks

Each folder is one verification block tied to a replacement claim from the
capability map in `projects/jet/README.md`. Cargo target names stay flat
(`cargo test -p jet --test <name>` keeps working); the grouping lives in the
folder layout and the `[[test]]` entries in `Cargo.toml`.

| Block | Replacement claim | Block README |
|---|---|---|
| `pkg-mgmt/` | `jet install/add/remove/update/audit` fully replaces npm/pnpm and is faster | `pkg-mgmt/README.md` |
| `browser-bridge/` | `jet bb` fully replaces Playwright, exposed as CLI and MCP for agents | `browser-bridge/README.md` |
| `build/` | `jet build` output matches Vite/Webpack, bundle size is not larger, and the build is faster | `build/README.md` |
| `test-runner/` | `jet test` replaces Vitest/Jest with a Jet-owned TS test runtime (no npm package) | `test-runner/README.md` |
| `task-runner/` | Jet workspace task execution replaces Nx/Turbo-style runners | `task-runner/README.md` |
| `wasm/` | Advanced FE-on-WASM runtime matches the FE-on-DOM oracle | `wasm/README.md` |

Shared infrastructure stays at the tests root:

- `common/` — shared harness (browser launch, react oracle, canvas spy,
  snapshot assertions). Tests in block folders import it via
  `#[path = "../common/mod.rs"] mod common;`.
- `fixtures/` — fixture apps and corpora. Paths are referenced by gate scripts
  (`projects/jet/scripts/*.mjs|sh`) and must not move without updating them.
- `__snapshots__/` — JSON snapshots resolved from `CARGO_MANIFEST_DIR`.

Run one block locally by folder:

```bash
# names are listed per block in Cargo.toml [[test]] groups, e.g.
cargo test -p jet --test workspace_protocol
```

The cross-tool comparison gates (npm/pnpm benchmarks, Playwright baseline,
Vite/Webpack corpus) live in `projects/jet/scripts/` and are documented per
block.
