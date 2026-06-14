---
id: webgpu-text-pass-encode
summary: Wire the existing GlyphInstance/text-pass building blocks into a callable encode seam. Add FrameBuilder::encode_text_pass, WebGpuRenderer::render_frame_with_text, and a glyph-instance pool slot, then route cclab_grid_wasm::renderer_bridge::renderFrameWithText through the shaper + atlas into the new two-pass entry. Issue #2191, slice of epic #1254.
fill_sections: [dependency, interaction, logic, changes, test-plan]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# WebGPU text pass — encode seam

## Dependency
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: webgpu-text-pass-encode-types
types:
  CellInstance:       { kind: struct }
  GlyphInstance:      { kind: struct }
  InstancePool:       { kind: struct }
  FrameBuilder:       { kind: struct }
  WebGpuRenderer:     { kind: struct }
  TextRunWire:        { kind: struct }
  Shaper:             { kind: service }
  GlyphAtlas:         { kind: service }
  RendererHandle:     { kind: struct }
  WebGpuTextRun:      { kind: struct }
  JetWebGpuApp:       { kind: struct }
edges:
  - { from: InstancePool,   to: CellInstance,   kind: stores,   label: "slot 0 bytes" }
  - { from: InstancePool,   to: GlyphInstance,  kind: stores,   label: "slot 1 bytes (new)" }
  - { from: FrameBuilder,   to: InstancePool,   kind: uses,     label: "upload_instance_buffer(slot, min, bytes)" }
  - { from: FrameBuilder,   to: CellInstance,   kind: encodes,  label: "encode_cell_pass" }
  - { from: FrameBuilder,   to: GlyphInstance,  kind: encodes,  label: "encode_text_pass (new)" }
  - { from: WebGpuRenderer, to: FrameBuilder,   kind: builds,   label: "begin_frame" }
  - { from: WebGpuRenderer, to: CellInstance,   kind: renders,  label: "render_frame / render_frame_clipped" }
  - { from: WebGpuRenderer, to: GlyphInstance,  kind: renders,  label: "render_frame_with_text (new)" }
  - { from: RendererHandle, to: WebGpuRenderer, kind: drives,   label: "owns" }
  - { from: RendererHandle, to: TextRunWire,    kind: parses,   label: "validate_text_runs" }
  - { from: RendererHandle, to: Shaper,         kind: invokes,  label: "shape text runs (new path)" }
  - { from: Shaper,         to: GlyphAtlas,     kind: queries,  label: "glyph cache lookup" }
  - { from: Shaper,         to: GlyphInstance,  kind: emits,    label: "shaped output" }
  - { from: JetWebGpuApp,   to: WebGpuTextRun,  kind: forwards, label: "preserved on PaintBackend" }
  - { from: JetWebGpuApp,   to: RendererHandle, kind: calls,    label: "renderFrameWithText(cells, runs)" }
---
classDiagram
    class CellInstance
    class GlyphInstance
    class InstancePool
    class FrameBuilder
    class WebGpuRenderer
    class TextRunWire
    class Shaper
    class GlyphAtlas
    class RendererHandle
    class WebGpuTextRun
    class JetWebGpuApp
    InstancePool --> CellInstance : stores slot 0
    InstancePool --> GlyphInstance : stores slot 1
    FrameBuilder --> InstancePool : uses
    FrameBuilder --> CellInstance : encodes
    FrameBuilder --> GlyphInstance : encodes
    WebGpuRenderer --> FrameBuilder : builds
    WebGpuRenderer --> CellInstance : renders
    WebGpuRenderer --> GlyphInstance : renders
    RendererHandle --> WebGpuRenderer : drives
    RendererHandle --> TextRunWire : parses
    RendererHandle --> Shaper : invokes
    Shaper --> GlyphAtlas : queries
    Shaper --> GlyphInstance : emits
    JetWebGpuApp --> WebGpuTextRun : forwards
    JetWebGpuApp --> RendererHandle : calls
```

## Interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: webgpu-text-pass-encode-flow
actors:
  - { id: jet, kind: actor }
  - { id: handle, kind: participant }
  - { id: shaper, kind: participant }
  - { id: atlas, kind: participant }
  - { id: renderer, kind: participant }
  - { id: frame, kind: participant }
  - { id: gpu, kind: system }
messages:
  - { from: jet,      to: handle,   name: "renderFrameWithText(cells, runs)" }
  - { from: handle,   to: handle,   name: "validate_text_runs(runs)", returns: "count" }
  - { from: handle,   to: shaper,   name: "shape(runs)", returns: "glyphs" }
  - { from: shaper,   to: atlas,    name: "glyph cache lookup", returns: "uv rects" }
  - { from: handle,   to: renderer, name: "render_frame_with_text(cells, glyphs)" }
  - { from: renderer, to: frame,    name: "begin_frame()", returns: "FrameBuilder" }
  - { from: renderer, to: frame,    name: "encode_cell_pass(cells, Some(clear))" }
  - { from: renderer, to: frame,    name: "encode_text_pass(glyphs, None)" }
  - { from: frame,    to: gpu,      name: "submit (one queue.submit per frame)" }
  - { from: frame,    to: gpu,      name: "present surface" }
  - { from: handle,   to: handle,   name: "observe_text_glyph_count(glyphs.len())" }
---
sequenceDiagram
    actor jet
    participant handle
    participant shaper
    participant atlas
    participant renderer
    participant frame
    participant gpu
    jet ->> handle: renderFrameWithText(cells, runs)
    handle ->> handle: validate_text_runs(runs)
    handle ->> shaper: shape(runs)
    shaper ->> atlas: glyph cache lookup
    shaper -->> handle: glyphs
    handle ->> renderer: render_frame_with_text(cells, glyphs)
    renderer ->> frame: begin_frame()
    renderer ->> frame: encode_cell_pass(cells, Some(clear))
    renderer ->> frame: encode_text_pass(glyphs, None)
    frame ->> gpu: submit
    frame ->> gpu: present
    handle ->> handle: observe_text_glyph_count(glyphs.len())
```

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: webgpu-text-pass-encode-orchestration
entry: enter
nodes:
  enter:      { kind: start,    label: "render_frame_with_text(cells, glyphs)" }
  acquire:    { kind: process,  label: "begin_frame -> FrameBuilder" }
  cell_pass:  { kind: process,  label: "encode_cell_pass(cells, Some(clear))" }
  glyphs_q:   { kind: decision, label: "glyphs empty?" }
  text_noop:  { kind: process,  label: "encode_text_pass(empty, None) = no-op" }
  upload:     { kind: process,  label: "instance pool slot 1 upload" }
  text_pass:  { kind: process,  label: "encode_text_pass with load op" }
  commit:     { kind: process,  label: "commit -> submit + present" }
  observe:    { kind: process,  label: "observe_text_glyph_count" }
  done:       { kind: terminal, label: "frame complete" }
edges:
  - { from: enter,     to: acquire }
  - { from: acquire,   to: cell_pass }
  - { from: cell_pass, to: glyphs_q }
  - { from: glyphs_q,  to: text_noop, label: "yes" }
  - { from: glyphs_q,  to: upload,    label: "no" }
  - { from: upload,    to: text_pass }
  - { from: text_noop, to: commit }
  - { from: text_pass, to: commit }
  - { from: commit,    to: observe }
  - { from: observe,   to: done }
---
flowchart TD
    enter([render_frame_with_text]) --> acquire[begin_frame]
    acquire --> cell_pass[encode_cell_pass with clear]
    cell_pass --> glyphs_q{glyphs empty?}
    glyphs_q -->|yes| text_noop[encode_text_pass no-op]
    glyphs_q -->|no| upload[pool slot 1 upload]
    upload --> text_pass[encode_text_pass load op]
    text_noop --> commit[commit submit+present]
    text_pass --> commit
    commit --> observe[observe_text_glyph_count]
    observe --> done([frame complete])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: crates/cclab-grid-render-webgpu/src/frame.rs
    action: modify
    section: dependency
    impl_mode: hand-written
    description: Add FrameBuilder::encode_text_pass(glyphs, clear) that reuses the text_pass bind-group layout + WGSL pipeline, uploads GlyphInstance bytes into instance-pool slot 1 when non-empty, and begins one render pass with the requested load op (Load when clear is None, Clear(c) otherwise). Mirrors encode_cell_pass empty/clear semantics and shares the same encoder under the one-submit invariant. New regression tests cover empty+None no-op and non-empty load-op draw call count.
  - path: crates/cclab-grid-render-webgpu/src/lib.rs
    action: modify
    section: dependency
    impl_mode: hand-written
    description: Add WebGpuRenderer::render_frame_with_text(cells, glyphs) that calls begin_frame once, encodes cell pass with Some(clear), encodes text pass with None, and commits exactly one submission. Leaves render_frame and render_frame_clipped byte-equivalent. Adds tracing span "frame.text_pass" nested under the existing frame span for parity with cell_pass.
  - path: crates/cclab-grid-render-webgpu/src/instance_pool.rs
    action: modify
    section: dependency
    impl_mode: hand-written
    description: Allow pool slot 1 to be reserved and grown independently of slot 0. The grow rule (min-size, returns reusable wgpu::Buffer handle) is identical to slot 0; slot 0 cell-pass uploads must not invalidate slot 1's buffer (and vice versa). Add a unit test that uploads to slot 0 then slot 1 then slot 0 again and asserts the slot 1 handle stays valid.
  - path: crates/cclab-grid-render-webgpu/src/text_pass.rs
    action: modify
    section: dependency
    impl_mode: hand-written
    description: Expose a stable pub fn that returns the text-pass render pipeline (currently constructed inline in tests). FrameBuilder::encode_text_pass calls into it. No WGSL or BGL changes.
  - path: crates/cclab-grid-wasm/src/renderer_bridge.rs
    action: modify
    section: dependency
    impl_mode: hand-written
    description: Extend RendererHandle::render_frame_with_text so it (a) parses TextRunWire as before, (b) shapes each run via the crate's shaper module against the active font_db + glyph_atlas, (c) collects GlyphInstance rows into a Vec, (d) calls renderer.render_frame_with_text(cells, glyphs) instead of the current cell-only fallback, (e) records last_text_glyph_count on BridgeState. Adds observe_text_glyph_count helper alongside observe_text_runs.
  - path: projects/jet/wasm/src/react/webgpu_app.rs
    action: modify
    section: dependency
    impl_mode: hand-written
    description: Extend window.__jet_webgpu_status with lastTextGlyphCount mirroring lastTextRunCount; do not remove or rename any existing field. bridgeMode flips from "text-runs" to "text-glyphs" once the new path is wired.
  - path: projects/jet/tests/wasm/wasm_build_end_to_end.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: In webgpu_renderer_reports_runtime_status_when_available, after the existing text-run assertion, add an assertion that window.__jet_webgpu_status.lastTextGlyphCount > 0 when navigator.gpu is present; keep the existing should_skip_env gate so hosts without WebGPU still skip cleanly.
  - path: .aw/tech-design/projects/jet/logic/wasm-renderer-webgpu-backend.md
    action: modify
    section: dependency
    impl_mode: hand-written
    description: Update the requirements matrix to reflect that W11 now flows glyphs into the lower renderer; add cross-references to the new spec for the encode seam. No backend-bridge contract changes.
  - path: .aw/tech-design/crates/cclab-grid-wasm/logic/renderer_bridge.md
    action: modify
    section: dependency
    impl_mode: hand-written
    description: Note that renderFrameWithText now shapes runs into GlyphInstance rows and forwards them to the lower renderer; reference the new spec for the wire-to-glyph plumbing.
  - path: ".aw/tech-design/projects/jet/specs/webgpu-text-pass-encode.md"
    action: verify
    section: interaction
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

  - path: ".aw/tech-design/projects/jet/specs/webgpu-text-pass-encode.md"
    action: verify
    section: logic
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```

## Test Plan
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: webgpu-text-pass-encode-verification
requirements:
  encode_text_pass_basic:    { id: T1, text: "encode_text_pass draws a render pass with text pipeline + slot 1 upload", kind: functional, risk: high,   verify: test }
  encode_text_pass_noop:     { id: T2, text: "empty glyphs + clear None encodes nothing",                              kind: functional, risk: medium, verify: test }
  render_frame_with_text:    { id: T3, text: "render_frame_with_text composes cell then text with one submit",         kind: functional, risk: high,   verify: test }
  pool_slot_independence:    { id: T4, text: "slot 0 + slot 1 grow independently and do not clobber each other",       kind: functional, risk: high,   verify: test }
  legacy_byte_equivalence:   { id: T5, text: "render_frame and render_frame_clipped behavior unchanged",               kind: regression, risk: medium, verify: test }
  bridge_shapes_runs:        { id: T6, text: "renderer bridge produces GlyphInstance rows from TextRunWire payloads",  kind: functional, risk: high,   verify: test }
  bridge_observes_count:     { id: T7, text: "bridge state records last_text_glyph_count after shape + submit",        kind: functional, risk: medium, verify: test }
  chromium_runtime_smoke:    { id: T8, text: "wasm e2e asserts lastTextGlyphCount > 0 when navigator.gpu present",     kind: functional, risk: high,   verify: test }
  coverage_stays_managed:    { id: T9, text: "aw standardize managed next reports 100 percent for touched scopes",  kind: tooling,    risk: low,    verify: tool }
elements:
  test_encode_text_pass_draws_when_non_empty:     { kind: test, type: "rs/#[test]" }
  test_encode_text_pass_noop_on_empty_and_none:   { kind: test, type: "rs/#[test]" }
  test_render_frame_with_text_single_submit:      { kind: test, type: "rs/#[test]" }
  test_instance_pool_slot_independence:           { kind: test, type: "rs/#[test]" }
  test_render_frame_unchanged_against_baseline:   { kind: test, type: "rs/#[test]" }
  test_bridge_shapes_runs_into_glyph_instances:   { kind: test, type: "rs/#[test]" }
  test_bridge_observes_last_text_glyph_count:     { kind: test, type: "rs/#[test]" }
  test_webgpu_renderer_reports_glyph_count:       { kind: test, type: "rs/integration" }
  check_managed_coverage_after_slice:             { kind: check, type: "shell/score-standardize" }
relations:
  - { from: test_encode_text_pass_draws_when_non_empty,   verifies: encode_text_pass_basic }
  - { from: test_encode_text_pass_noop_on_empty_and_none, verifies: encode_text_pass_noop }
  - { from: test_render_frame_with_text_single_submit,    verifies: render_frame_with_text }
  - { from: test_instance_pool_slot_independence,         verifies: pool_slot_independence }
  - { from: test_render_frame_unchanged_against_baseline, verifies: legacy_byte_equivalence }
  - { from: test_bridge_shapes_runs_into_glyph_instances, verifies: bridge_shapes_runs }
  - { from: test_bridge_observes_last_text_glyph_count,   verifies: bridge_observes_count }
  - { from: test_webgpu_renderer_reports_glyph_count,     verifies: chromium_runtime_smoke }
  - { from: check_managed_coverage_after_slice,           verifies: coverage_stays_managed }
---
requirementDiagram
    requirement encode_text_pass_basic    { id: T1 text: encode_text_pass draws pass risk: high   verifymethod: test }
    requirement encode_text_pass_noop     { id: T2 text: empty glyphs no op            risk: medium verifymethod: test }
    requirement render_frame_with_text    { id: T3 text: two pass single submit         risk: high   verifymethod: test }
    requirement pool_slot_independence    { id: T4 text: slot 0 and slot 1 isolated     risk: high   verifymethod: test }
    requirement legacy_byte_equivalence   { id: T5 text: legacy paths unchanged         risk: medium verifymethod: test }
    requirement bridge_shapes_runs        { id: T6 text: bridge shapes runs to glyphs   risk: high   verifymethod: test }
    requirement bridge_observes_count     { id: T7 text: bridge records glyph count     risk: medium verifymethod: test }
    requirement chromium_runtime_smoke    { id: T8 text: e2e asserts glyph count        risk: high   verifymethod: test }
    requirement coverage_stays_managed    { id: T9 text: managed coverage 100 pct       risk: low    verifymethod: tool }
    element test_encode_text_pass_draws_when_non_empty
    element test_encode_text_pass_noop_on_empty_and_none
    element test_render_frame_with_text_single_submit
    element test_instance_pool_slot_independence
    element test_render_frame_unchanged_against_baseline
    element test_bridge_shapes_runs_into_glyph_instances
    element test_bridge_observes_last_text_glyph_count
    element test_webgpu_renderer_reports_glyph_count
    element check_managed_coverage_after_slice
    test_encode_text_pass_draws_when_non_empty - verifies -> encode_text_pass_basic
    test_encode_text_pass_noop_on_empty_and_none - verifies -> encode_text_pass_noop
    test_render_frame_with_text_single_submit - verifies -> render_frame_with_text
    test_instance_pool_slot_independence - verifies -> pool_slot_independence
    test_render_frame_unchanged_against_baseline - verifies -> legacy_byte_equivalence
    test_bridge_shapes_runs_into_glyph_instances - verifies -> bridge_shapes_runs
    test_bridge_observes_last_text_glyph_count - verifies -> bridge_observes_count
    test_webgpu_renderer_reports_glyph_count - verifies -> chromium_runtime_smoke
    check_managed_coverage_after_slice - verifies -> coverage_stays_managed
```

# Reviews

### Review 1
**Verdict:** approved

_2026-05-15T17:24Z · score-td-reviewer_

- [Dependency] classDiagram + Mermaid Plus YAML is well-formed; types/edges cleanly capture the new `FrameBuilder::encode_text_pass` and `WebGpuRenderer::render_frame_with_text` seams plus InstancePool slot 1, and the JetWebGpuApp → RendererHandle → Shaper/GlyphAtlas wiring matches the existing renderer_bridge.rs surface (verified `validate_text_runs`, `observe_text_runs`, `last_text_run_count`, `TextRunWire` all exist).
- [Interaction] sequenceDiagram correctly threads `renderFrameWithText(cells, runs)` → `validate_text_runs` → `shape` → `render_frame_with_text` → two passes → one submit → `observe_text_glyph_count`; lines up with the Logic flowchart and the issue's R3/R6/R7.
- [Logic] flowchart accurately encodes the empty-glyph no-op branch (R2) and the load-op text pass over a cleared cell pass (R1, R3); cell pass uses `Some(clear)` and text pass uses `None`, matching `frame.rs` (`encode_cell_pass` already accepts the same shape).
- [Changes] All nine file paths exist on disk; descriptions cite real, current symbol names (`FrameBuilder::encode_cell_pass` at frame.rs:80, `render_frame`/`render_frame_clipped`/`begin_frame` in lib.rs, slot-indexed `InstancePool::get_or_grow`, `RendererHandle::render_frame_with_text` shim in renderer_bridge.rs, `__jet_webgpu_status` keys in webgpu_app.rs, `webgpu_renderer_reports_runtime_status_when_available` in tests). The two TD cross-reference specs (`wasm-renderer-webgpu-backend.md`, `crates/cclab-grid-wasm/logic/renderer_bridge.md`) both exist.
- [Test Plan] requirementDiagram + YAML are well-formed; T1–T9 each trace to exactly one element via `relations:`, kinds/risk/verify are consistent, and the chromium_runtime_smoke (T8) ↔ `wasm_build_end_to_end::webgpu_renderer_reports_runtime_status_when_available` mapping is concrete and implementable.
