# surface

## Brief

`surface` is the renderer-neutral UI element model shared by Jet WASM, native
desktop readers, renderers, and parity tools.

## Capabilities

A promise with no gate under it is not claimed.

### Capability Index

| Capability | Root WI | Notes |
|---|---:|---|
| Renderer-Neutral UI Surface Model | - | serializable element, props, callback, and snapshot model |

### Renderer-Neutral UI Surface Model

UI runtimes and renderers can exchange deterministic renderer-neutral surface
trees without depending on a specific frontend backend.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `cclab_surface`.
- Gate — behavior: `cargo test -p cclab-surface` - surface model and snapshot
  coverage
- Gate: `cargo test -p cclab-surface`
- Source: `libs/surface/src/lib.rs`
- Evidence: `cargo test -p cclab-surface`; libs/surface/src/lib.rs
