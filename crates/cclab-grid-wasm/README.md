# Cclab Grid Wasm

## Brief

Cclab Grid Wasm is the wasm-bindgen bridge that exposes the Rust spreadsheet
engine and grid renderer integration points to JavaScript.

It owns the JavaScript-facing `SpreadsheetEngine`, JSON/typed-array data
contracts, renderer bridge state, text-run planning, RAF visibility state
machine, resize debounce logic, and viewport packing helpers. The configured
host gate validates these Rust-side wasm-bindgen contracts; browser packaging,
WebGPU canvas rendering, RAF, and ResizeObserver journeys remain separate
browser/e2e coverage.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| JavaScript Spreadsheet Engine API | - | implemented | passing | conformance | not_ready | host tests cover the wasm-bindgen engine API and data contracts; browser package/e2e gate remains separate |
| WebGPU Renderer Bridge | - | implemented | passing | smoke | not_ready | host tests cover bridge invariants and text planning; browser WebGPU e2e gate remains separate |
| Browser Frame Loop And Resize Bridge | - | implemented | passing | smoke | not_ready | host tests cover state machines; live RAF/ResizeObserver integration gate remains separate |

### JavaScript Spreadsheet Engine API

ID: javascript-spreadsheet-engine-api
Type: DeveloperTool
Surfaces: WASM/JS API: `SpreadsheetEngine` - JavaScript-facing workbook, cell, formula, formatting, validation, selection, history, serialization, and viewport buffer API
EC Dimensions: behavior: `cargo test -p cclab-grid-wasm` - host-portable wasm-bindgen API invariants, formula recalculation, validation, formatting serialization, persistence JSON, viewport packing, and bug-fix regressions; browser package/e2e verification is not covered by this gate
Root WI: -
Status: confirmed
Required Verification: conformance
Promise:
Cclab Grid Wasm exposes the spreadsheet engine to JavaScript with workbook/cell mutation, formula dependency recalculation, validation, formatting, selection/history behavior, JSON serialization, and zero-copy viewport buffers.
Gate Inventory: `cargo test -p cclab-grid-wasm`; crates/cclab-grid-wasm/src/api.rs; crates/cclab-grid-wasm/src/viewport.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| JavaScript spreadsheet engine contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-grid-wasm`; crates/cclab-grid-wasm/src/api.rs; crates/cclab-grid-wasm/src/viewport.rs |

### WebGPU Renderer Bridge

ID: webgpu-renderer-bridge
Type: DeveloperTool
Surfaces: WASM/JS API: `init_renderer(canvas, width, height, dpr)` and `RendererHandle` - Promise-based renderer initialization, packed cell buffer rendering, text-run payload validation, resize, and destroy lifecycle
EC Dimensions: behavior: `cargo test -p cclab-grid-wasm` - host-portable bridge state, packed cell buffer stride validation, text-run payload validation, placeholder glyph planning, atlas planning, and resize/render counters; browser WebGPU e2e verification is not covered by this gate
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Cclab Grid Wasm bridges JavaScript grid views to the native WebGPU renderer through an async renderer initializer, explicit renderer handle lifecycle, packed Float32 cell buffers, text-run validation/planning, resize propagation, and teardown.
Gate Inventory: `cargo test -p cclab-grid-wasm`; crates/cclab-grid-wasm/src/renderer_bridge.rs; crates/cclab-grid-wasm/docs/renderer-bridge-slice-4m.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| WebGPU renderer bridge smoke contract | epic | - | implemented | passing | smoke | `cargo test -p cclab-grid-wasm`; crates/cclab-grid-wasm/src/renderer_bridge.rs; crates/cclab-grid-wasm/docs/renderer-bridge-slice-4m.md |

### Browser Frame Loop And Resize Bridge

ID: browser-frame-loop-and-resize-bridge
Type: DeveloperTool
Surfaces: WASM/JS API: `WasmView`, `LoopController`, `ResizeDebouncer`, `ViewportBuffer` - browser scheduling, visibility, resize debounce, and typed-array viewport bridge
EC Dimensions: behavior: `cargo test -p cclab-grid-wasm` - RAF loop state transitions, visibility pause/resume, resize debounce coalescing/clamping, and viewport format packing; browser RAF/ResizeObserver integration is not covered by this host gate
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Cclab Grid Wasm provides browser-facing frame-loop and resize state machines for RAF scheduling, off-screen pause/resume, resize coalescing, DPR clamping, and typed-array viewport data transfer.
Gate Inventory: `cargo test -p cclab-grid-wasm`; crates/cclab-grid-wasm/src/frame_loop.rs; crates/cclab-grid-wasm/src/resize_debouncer.rs; crates/cclab-grid-wasm/src/viewport.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Browser frame loop and resize smoke contract | epic | - | implemented | passing | smoke | `cargo test -p cclab-grid-wasm`; crates/cclab-grid-wasm/src/frame_loop.rs; crates/cclab-grid-wasm/src/resize_debouncer.rs; crates/cclab-grid-wasm/src/viewport.rs |
