# ui-runtime

## Brief

`ui-runtime` is the renderer-neutral component runtime above `surface`: hooks,
fiber storage, mount, flush, and update scheduling over surface elements.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Renderer-Neutral Component Runtime | - | implemented | verified | smoke | ready | hooks, fiber storage, mount, flush, and scheduling |

### Renderer-Neutral Component Runtime

ID: renderer-neutral-component-runtime
Type: DeveloperTool
Root WI: -
Status: verified
Surfaces: Rust API: `cclab_ui_runtime`.
EC Dimensions: behavior: `cargo test -p cclab-ui-runtime` - runtime hook and scheduling coverage
Required Verification: smoke
Promise:
Renderers can run reusable component logic over surface elements without
coupling to a browser or native UI backend.
Gate Inventory: `cargo test -p cclab-ui-runtime`; libs/ui-runtime/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| renderer-neutral-component-runtime-contract | epic | - | implemented | verified | smoke | `cargo test -p cclab-ui-runtime`; libs/ui-runtime/src/lib.rs |
