# surface

## Brief

`surface` is the renderer-neutral UI element model shared by Jet WASM, native
desktop readers, renderers, and parity tools.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Renderer-Neutral UI Surface Model | - | implemented | verified | smoke | ready | serializable element, props, callback, and snapshot model |

### Renderer-Neutral UI Surface Model

ID: renderer-neutral-ui-surface-model
Type: DeveloperTool
Root WI: -
Status: verified
Surfaces: Rust API: `cclab_surface`.
EC Dimensions: behavior: `cargo test -p cclab-surface` - surface model and snapshot coverage
Required Verification: smoke
Promise:
UI runtimes and renderers can exchange deterministic renderer-neutral surface
trees without depending on a specific frontend backend.
Gate Inventory: `cargo test -p cclab-surface`; libs/surface/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| renderer-neutral-ui-surface-model-contract | epic | - | implemented | verified | smoke | `cargo test -p cclab-surface`; libs/surface/src/lib.rs |
