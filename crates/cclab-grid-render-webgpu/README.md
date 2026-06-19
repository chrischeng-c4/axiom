# Cclab Grid Render Webgpu

## Brief

Cclab Grid Render Webgpu is the native WebGPU rendering runtime for cclab grid
surfaces.

It owns the renderer lifecycle, backend selection, frame/cell-rect pipeline,
viewport/DPR/MSAA state, text glyph pipeline, diagnostics, device-loss recovery,
GPU memory accounting, and headless smoke readback harness. The code has broad
unit coverage, but the configured full `cargo test -p cclab-grid-render-webgpu`
gate is currently blocked in this checkout because the local cap memory policy
kills it before integration/doc-test completion.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| WebGPU Renderer Runtime | - | implemented | blocked | smoke | not_ready | renderer/headless tests start and unit assertions pass until local cap kills the full gate before completion |
| Text Glyph Rendering Pipeline | - | implemented | blocked | smoke | not_ready | text/glyph units run but the configured full gate is blocked by local memory pressure |
| Diagnostics Recovery And Resource Accounting | - | implemented | blocked | smoke | not_ready | diagnostics/resource units run but the configured full gate is blocked by local memory pressure |

### WebGPU Renderer Runtime

ID: webgpu-renderer-runtime
Type: RuntimeTool
Surfaces:
- Rust API: `cclab_grid_render_webgpu::{WebGpuRenderer, FrameBuilder}` - WebGPU renderer lifecycle and surface/frame pipeline.
- Rust API: `cclab_grid_render_webgpu::headless::{request_smoke_adapter, HeadlessSmokeRenderer}` - surface-less smoke renderer and pixel readback harness.
EC Dimensions: behavior: `cargo test -p cclab-grid-render-webgpu` - renderer, backend, pipeline, viewport, DPR, instance pool, frame loop, screenshot, and headless smoke coverage; currently blocked because the local cap memory policy kills the full gate before integration/doc-test completion
Root WI: -
Status: blocked
Required Verification: smoke
Promise:
Cclab Grid Render Webgpu provides the native WebGPU rendering runtime for grid surfaces, including backend selection, device/queue/surface ownership, frame orchestration, cell-rect rendering, viewport/DPR/MSAA state, and a headless smoke-readback harness.
Gate Inventory: `cargo test -p cclab-grid-render-webgpu`; `cargo test -p cclab-grid-render-webgpu --lib`; `cargo test -p cclab-grid-render-webgpu --test headless_smoke`; crates/cclab-grid-render-webgpu/tests/headless_smoke.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Renderer runtime smoke contract | epic | - | implemented | blocked | smoke | `cargo test -p cclab-grid-render-webgpu`; `cargo test -p cclab-grid-render-webgpu --lib`; `cargo test -p cclab-grid-render-webgpu --test headless_smoke`; crates/cclab-grid-render-webgpu/tests/headless_smoke.rs |
| Headless render readback contract | epic | - | implemented | blocked | smoke | `cargo test -p cclab-grid-render-webgpu --test headless_smoke` is killed by local cap memory pressure before completion |

### Text Glyph Rendering Pipeline

ID: text-glyph-rendering-pipeline
Type: RuntimeTool
Surfaces: Rust API: `cclab_grid_render_webgpu::{font_db, font_face, glyph_raster, glyph_cache, glyph_atlas, glyph_atlas_upload, text_pass, shaper}` - text/glyph rendering data plane
EC Dimensions: behavior: `cargo test -p cclab-grid-render-webgpu` - font loading/indexing, font face validation, glyph raster/cache/atlas descriptors and upload validation, text pass WGSL/layout, and shaping value objects; currently blocked because the local cap memory policy kills the full gate before integration/doc-test completion
Root WI: -
Status: blocked
Required Verification: smoke
Promise:
Cclab Grid Render Webgpu owns the text rendering data plane for grid cells: font lookup, font face validation, glyph bitmap rasterization, glyph cache/atlas descriptors, atlas upload validation, text-pass WGSL/layout, and glyph shaping value objects.
Gate Inventory: `cargo test -p cclab-grid-render-webgpu`; crates/cclab-grid-render-webgpu/src/font_db.rs; crates/cclab-grid-render-webgpu/src/text_pass.rs; crates/cclab-grid-render-webgpu/docs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Text glyph pipeline smoke contract | epic | - | implemented | blocked | smoke | `cargo test -p cclab-grid-render-webgpu`; crates/cclab-grid-render-webgpu/src/font_db.rs; crates/cclab-grid-render-webgpu/src/text_pass.rs; crates/cclab-grid-render-webgpu/docs |

### Diagnostics Recovery And Resource Accounting

ID: diagnostics-recovery-and-resource-accounting
Type: RuntimeTool
Surfaces: Rust API: `cclab_grid_render_webgpu::{validation, tracing_setup, gpu_memory, frame_timing, lost_context, viewport_clamp}` - diagnostics, recovery, and resource-accounting helpers
EC Dimensions: behavior: `cargo test -p cclab-grid-render-webgpu` - validation flags/log bridge, tracing setup, device-loss status/recovery errors, GPU memory math, frame timing, viewport clamp, and screenshot/readback invariants; currently blocked because the local cap memory policy kills the full gate before integration/doc-test completion
Root WI: -
Status: blocked
Required Verification: smoke
Promise:
Cclab Grid Render Webgpu exposes renderer diagnostics and recovery helpers for wgpu validation signal, tracing spans, device-loss observation/recovery, GPU memory estimation, frame timing, viewport clamping, and screenshot/readback invariants.
Gate Inventory: `cargo test -p cclab-grid-render-webgpu`; crates/cclab-grid-render-webgpu/src/validation.rs; crates/cclab-grid-render-webgpu/src/gpu_memory.rs; crates/cclab-grid-render-webgpu/src/lost_context.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Diagnostics and recovery smoke contract | epic | - | implemented | blocked | smoke | `cargo test -p cclab-grid-render-webgpu`; crates/cclab-grid-render-webgpu/src/validation.rs; crates/cclab-grid-render-webgpu/src/gpu_memory.rs; crates/cclab-grid-render-webgpu/src/lost_context.rs |
