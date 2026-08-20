# ui-runtime

## Brief

`ui-runtime` is the renderer-neutral component runtime above `surface`: hooks,
fiber storage, mount, flush, and update scheduling over surface elements.

## Capabilities

A promise with no gate under it is not claimed.

### Capability Index

| Capability | Root WI | Notes |
|---|---:|---|
| Renderer-Neutral Component Runtime | - | hooks, fiber storage, mount, flush, and scheduling |

### Renderer-Neutral Component Runtime

Renderers can run reusable component logic over surface elements without
coupling to a browser or native UI backend.

- Root WI: none; this capability predates the tracker.
- Surfaces: Rust API: `cclab_ui_runtime`.
- Gate — behavior: `cargo test -p cclab-ui-runtime` - runtime hook and
  scheduling coverage
- Gate: `cargo test -p cclab-ui-runtime`
- Source: `libs/ui-runtime/src/lib.rs`
- Evidence: `cargo test -p cclab-ui-runtime`; libs/ui-runtime/src/lib.rs
