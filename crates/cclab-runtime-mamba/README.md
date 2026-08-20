# Cclab Runtime Mamba

## Brief

Cclab Runtime Mamba is the Mamba native binding for async runtime primitives.

It registers the `cclab.runtime` module through the shared Mamba registry and
exposes native-call entrypoints for blocking sleep, no-op task spawn handles,
gather stub acceptance, and a prototype HTTP `serve` bridge. The current
`serve` path has smoke-verified route-table mapping into Axum stub handlers;
full Mamba coroutine dispatch from Axum is outside the current binding surface.

## Capabilities

A promise with no gate under it is not claimed.

### Capability Index

| Capability | Root WI | Notes |
|---|---:|---|
| Mamba Async Runtime Binding | - | sleep/spawn/gather ABI and `serve` route-table bridge are behavior-smoke verified |

### Mamba Async Runtime Binding

Cclab Runtime Mamba exposes async runtime primitives to Mamba scripts through
the `cclab.runtime` native module, including blocking sleep, no-op task
spawning handles, gather stub acceptance, and a prototype `serve` bridge from
Mamba route tables to Axum stub handlers.

- Root WI: none; this capability predates the tracker.
- Surfaces: Mamba module: `cclab.runtime`; Native ABI: `mb_runtime_sleep`,
  `mb_runtime_spawn`, `mb_runtime_gather`, `mb_runtime_serve`; Rust module
  registrar: `RuntimeMambaModule`
- Gate — behavior: `cargo test -p cclab-runtime-mamba` - sleep/spawn/gather ABI
  and Axum route-table bridge smoke
- Gate: `cargo test -p cclab-runtime-mamba`
- Source: `crates/cclab-runtime-mamba/src/lib.rs`,
  `crates/cclab-runtime-mamba/src/methods.rs`,
  `crates/cclab-runtime-mamba/src/types.rs`,
  `crates/cclab-runtime-mamba/tests/methods_test.rs`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| Mamba sleep spawn and gather ABI contract | epic | - | `cargo test -p cclab-runtime-mamba`; crates/cclab-runtime-mamba/src/lib.rs; crates/cclab-runtime-mamba/src/methods.rs; crates/cclab-runtime-mamba/src/types.rs; crates/cclab-runtime-mamba/tests/methods_test.rs |
| Mamba HTTP serve route-table bridge | epic | - | `cargo test -p cclab-runtime-mamba`; crates/cclab-runtime-mamba/src/methods.rs |
