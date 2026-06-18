# Cclab Runtime Mamba

## Brief

Cclab Runtime Mamba is the Mamba native binding for async runtime primitives.

It registers the `cclab.runtime` module through the shared Mamba registry and
exposes native-call entrypoints for blocking sleep, no-op task spawn handles,
gather stub acceptance, and a prototype HTTP `serve` bridge. The current
`serve` path maps Mamba route tables to Axum stub handlers; full Mamba coroutine
dispatch from Axum is outside the current binding surface.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Mamba Async Runtime Binding | - | partial | planned | smoke | not_ready | sleep/spawn/gather are tested; `serve` route-table bridge still needs live server verification |

### Mamba Async Runtime Binding

ID: mamba-async-runtime-binding
Type: DeveloperTool
Surfaces: Mamba module: `cclab.runtime`; Native ABI: `mb_runtime_sleep`, `mb_runtime_spawn`, `mb_runtime_gather`, `mb_runtime_serve`; Rust module registrar: `RuntimeMambaModule`
EC Dimensions: behavior: `cargo test -p cclab-runtime-mamba`
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Cclab Runtime Mamba exposes async runtime primitives to Mamba scripts through the `cclab.runtime` native module, including blocking sleep, no-op task spawning handles, gather stub acceptance, and a prototype `serve` bridge from Mamba route tables to Axum stub handlers.
Gate Inventory: `cargo test -p cclab-runtime-mamba`; crates/cclab-runtime-mamba/src/lib.rs; crates/cclab-runtime-mamba/src/methods.rs; crates/cclab-runtime-mamba/src/types.rs; crates/cclab-runtime-mamba/tests/methods_test.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Mamba sleep spawn and gather ABI contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-runtime-mamba`; crates/cclab-runtime-mamba/src/lib.rs; crates/cclab-runtime-mamba/src/methods.rs; crates/cclab-runtime-mamba/src/types.rs; crates/cclab-runtime-mamba/tests/methods_test.rs |
| Mamba HTTP serve route-table bridge | epic | - | implemented | planned | smoke | crates/cclab-runtime-mamba/src/methods.rs |
