---
id: projects-jet-logic-wasm-renderer-webgpu-backend-md
fill_sections: [changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# WebGPU backend bridge

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: ".aw/tech-design/projects/jet/logic/wasm-renderer-webgpu-backend.md"
    action: modify
    section: doc
    impl_mode: hand-written
    description: |
      Legacy Jet TD content retained as notes during AW standardization.
      Rewrite this file into semantic TD sections before promoting source to CODEGEN.
```

## Legacy notes
<!-- type: doc lang: markdown -->

# WebGPU backend bridge

### Overview

Parent issue: #1254, WasmView renderer and grid WebGPU migration.

Jet's wasm renderer already has the right internal split:
`Element -> LayoutTree -> PaintOp -> PaintBackend`. The WebGPU path
must therefore plug into `PaintBackend` instead of bypassing layout or
creating a second renderer model. The first bridge lowers paint ops that
match the existing workspace WebGPU primitive contract into
`cclab_grid_render_webgpu::cell_rect::CellInstance`.

This slice is deliberately adapter-free. It is a deterministic frame
planner that can be tested on the host. The actual browser/native GPU
adapter, surface, frame loop, resize, device loss, and text passes remain
owned by `cclab-grid-render-webgpu` and `cclab-grid-wasm`.

The browser app bridge is `wasm32`-only: it calls
`cclab_grid_wasm::init_renderer`, waits for the returned Promise, and sends
each repaint through `RendererHandle.renderFrame(Float32Array)`.

`jet build --wasm` exposes this as a config-level renderer choice. The
default remains the existing canvas app. Setting `[wasm].renderer =
"web-gpu"` makes the generated entrypoint call `react::webgpu_app::run`
and makes the scaffolded Cargo project request `jet-wasm/webgpu-app`
instead of `jet-wasm/canvas-app`.

### Requirements

```mermaid
---
id: jet-wasm-renderer-webgpu-backend-requirements
entry: W1
---
requirementDiagram
    requirement W1 { id: W1 text: WebGPU backend implements PaintBackend risk: high verifymethod: test }
    requirement W2 { id: W2 text: FillRect lowers to shared CellInstance risk: high verifymethod: test }
    requirement W3 { id: W3 text: unsupported paint ops are reported risk: medium verifymethod: test }
    requirement W12 { id: W12 text: StrokeRect lowers to four CellInstance edge strips risk: medium verifymethod: test }
    requirement W4 { id: W4 text: backend selection wording comes from grid WebGPU runtime risk: medium verifymethod: test }
    requirement W5 { id: W5 text: frame plan exports grid wasm Float32Array wire layout risk: high verifymethod: test }
    requirement W6 { id: W6 text: webgpu app bridge reuses cclab grid wasm renderer handle risk: high verifymethod: wasm check }
    requirement W7 { id: W7 text: wasm config selects canvas or webgpu generated entrypoint risk: high verifymethod: test }
    requirement W8 { id: W8 text: webgpu renderer selection survives real wasm-pack scaffold build risk: high verifymethod: test }
    requirement W9 { id: W9 text: webgpu browser bridge exposes init and frame status for e2e diagnosis risk: medium verifymethod: wasm check }
    requirement W10 { id: W10 text: webgpu browser smoke verifies runtime status when navigator.gpu is available risk: high verifymethod: test }
```

| id | Requirement | Verifies |
|----|-------------|----------|
| W1 | `WebGpuBackend` implements `PaintBackend` and stores the latest frame plan. | `cargo test -p jet-wasm --features webgpu` |
| W2 | `PaintOp::FillRect` maps to `CellInstance { pos_px, size_px, color }`, with RGBA normalized to `0..=1`. | unit test |
| W3 | `Text` and clip ops are not silently dropped; the frame plan records an unsupported marker for each. (`StrokeRect` is now lowered — see W12.) | unit test |
| W4 | User-facing backend description is delegated to `cclab_grid_render_webgpu::backend`, keeping web = `BROWSER_WEBGPU` and native = `PRIMARY`. | unit test |
| W5 | `WebGpuFramePlan` can export the packed `Float32Array` wire format consumed by `cclab-grid-wasm::RendererHandle.renderFrame`: 8 f32 values per cell in `pos.xy, size.xy, rgba` order. | unit test |
| W6 | `react::webgpu_app::run` initializes the shared grid WebGPU renderer and drives first paint + click repaint through `renderFrame`. | `cargo check -p jet-wasm --target wasm32-unknown-unknown --features webgpu-app` |
| W7 | `[wasm].renderer = "web-gpu"` selects the WebGPU app entrypoint and scaffolded `jet-wasm/webgpu-app` feature set; omitting it preserves the canvas app default. | `cargo test -p jet wasm_build` |
| W8 | A temporary `renderer = "web-gpu"` app builds through `wasm-pack`, emits `jet-target.json` with `jet-wasm/webgpu-app`, and leaves the generated scaffold on the WebGPU entrypoint. | `cargo test -p jet --test wasm_build_end_to_end webgpu_renderer_build_selects_webgpu_scaffold` |
| W9 | The browser bridge writes `window.__jet_webgpu_status` with phase, frame count, last cell count, last text-run count, unsupported count, and error state so WebGPU e2e can distinguish adapter failure from Jet paint lowering. | `cargo check -p jet-wasm --target wasm32-unknown-unknown --features webgpu-app` |
| W10 | A Chromium smoke launches the generated WebGPU app, skips only when `navigator.gpu` is absent, and otherwise requires a non-error status using the `renderFrameWithText` bridge with at least one frame, lowered cell, and planned text run. | `cargo test -p jet --test wasm_build_end_to_end webgpu_renderer_reports_runtime_status_when_available` |
| W11 | The grid wasm bridge accepts a structured `renderFrameWithText(cells, textRuns)` wire payload, validates text runs, and preserves the text-run count while the lower renderer remains cell-only. | `cargo test -p cclab-grid-wasm renderer_bridge` |
| W12 | `PaintOp::StrokeRect { rect, color, width }` lowers to up to four `CellInstance` edge strips (top + bottom span full horizontal extent; left + right span only the middle, no corner overlap); degenerate inputs emit zero cells. See [`wasm-renderer-webgpu-strokerect.md`](./wasm-renderer-webgpu-strokerect.md). | unit test |

### Interfaces

```rust
pub struct WebGpuFramePlan {
    pub cells: Vec<cclab_grid_render_webgpu::cell_rect::CellInstance>,
    pub text_runs: Vec<WebGpuTextRun>,
    pub unsupported: Vec<WebGpuUnsupportedOp>,
}

impl WebGpuFramePlan {
    pub fn to_packed_f32(&self) -> Vec<f32>;
}

pub struct WebGpuTextRun {
    pub origin_px: [f32; 2],
    pub content: String,
    pub font_family: String,
    pub font_size_px: f32,
    pub font_weight: u16,
    pub color: [f32; 4],
}

pub enum WebGpuUnsupportedOp {
    // StrokeRect variant removed (W12): strokes now lower to four CellInstances.
    Text,
    Clip,
}

pub struct WebGpuBackend;

impl PaintBackend for WebGpuBackend {
    fn execute(&mut self, ops: &[PaintOp]);
}

pub mod react::webgpu_app {
    pub fn run(canvas_id: &str, component: Component) -> Result<js_sys::Promise, JsValue>;
}

pub struct JetWebGpuApp;

impl JetWebGpuApp {
    pub fn destroy(self) -> Result<(), JsValue>;
    pub fn status(&self) -> JsValue;
}

pub mod cclab_grid_wasm_bridge {
    pub fn renderFrameWithText(cells: Float32Array, textRuns: Array<TextRunWire>) -> Result<(), JsValue>;
}

pub enum WasmRenderer {
    Canvas,
    WebGpu,
}
```

### Changes

```yaml
changes:
  - path: projects/jet/wasm/Cargo.toml
    kind: update
    section: doc
    impl_mode: hand-written
    summary: Add a webgpu feature with an optional cclab-grid-render-webgpu dependency.
  - path: projects/jet/wasm/src/renderer/mod.rs
    kind: update
    section: doc
    impl_mode: hand-written
    summary: Export the WebGPU backend when the webgpu feature is enabled.
  - path: projects/jet/wasm/src/renderer/webgpu.rs
    kind: create
    section: doc
    impl_mode: hand-written
    summary: Lower FillRect paint ops into shared WebGPU CellInstance records, export packed bridge data, and report unsupported ops.
  - path: projects/jet/wasm/src/react/webgpu_app.rs
    kind: create
    section: doc
    impl_mode: hand-written
    summary: Add a wasm32-only React WebGPU browser app bridge over cclab-grid-wasm RendererHandle.
  - path: projects/jet/src/wasm_build/config.rs
    kind: update
    section: doc
    impl_mode: hand-written
    summary: Add a typed [wasm].renderer config key with canvas as the default and web-gpu as the WebGPU selector.
  - path: projects/jet/src/wasm_build/mod.rs
    kind: update
    section: doc
    impl_mode: hand-written
    summary: Thread the renderer selector into generated wasm entrypoint code, scaffolded jet-wasm features, and jet-target.json cargo feature metadata.
  - path: projects/jet/tests/wasm_build_end_to_end.rs
    kind: update
    section: doc
    impl_mode: hand-written
    summary: Add wasm-pack scaffold and browser runtime smokes for renderer = "web-gpu".
  - path: projects/jet/wasm/src/react/webgpu_app.rs
    kind: update
    section: doc
    impl_mode: hand-written
    summary: Expose browser-side WebGPU status on window.__jet_webgpu_status and JetWebGpuApp::status for runtime/e2e diagnostics.
  - path: projects/jet/wasm/src/renderer/webgpu.rs
    kind: update
    section: doc
    impl_mode: hand-written
    summary: Preserve structured text runs in the WebGPU frame plan while still reporting them unsupported until the wasm bridge exposes a text wire format.
  - path: crates/cclab-grid-wasm/src/renderer_bridge.rs
    kind: update
    section: doc
    impl_mode: hand-written
    summary: Add renderFrameWithText text-run wire validation and preserve the latest text-run count for the upcoming text pass.
  - path: projects/jet/wasm/src/renderer/webgpu.rs
    kind: update
    section: doc
    impl_mode: hand-written
    summary: Lower PaintOp::StrokeRect to four CellInstance edge strips (top + bottom + left + right with no corner overlap); remove the WebGpuUnsupportedOp::StrokeRect variant. See [strokerect slice doc](./wasm-renderer-webgpu-strokerect.md).
  - path: crates/cclab-grid-wasm/src/renderer_bridge.rs
    kind: update
    section: doc
    impl_mode: hand-written
    summary: |
      Slice #2191. After validating runs, plan placeholder GlyphInstances
      (one per char, mono-advance font_size_px * 0.6, full-atlas uv, run
      color), call WebGpuRenderer::render_frame_with_text(cells, glyphs)
      so cell pass + text pass share a single submit, observe the glyph
      count onto BridgeState, and expose lastTextGlyphCount() as a
      wasm-bindgen getter so the browser e2e (T8) can tell
      encode-fired-empty from encode-fired-with-glyphs.
  - path: projects/jet/wasm/src/react/webgpu_app.rs
    kind: update
    section: doc
    impl_mode: hand-written
    summary: |
      Slice #2191. Add lastTextGlyphCount to window.__jet_webgpu_status by
      reading RendererHandle.lastTextGlyphCount() via JS reflection on
      each repaint; falls back to 0 on older bridge builds.
  - path: projects/jet/tests/wasm_build_end_to_end.rs
    kind: update
    section: doc
    impl_mode: hand-written
    summary: |
      Slice #2191. T8 — assert window.__jet_webgpu_status.lastTextGlyphCount
      >= 1 so the browser smoke proves the text pass was encoded with at
      least one glyph (not merely that it didn't crash on empty runs).
```
