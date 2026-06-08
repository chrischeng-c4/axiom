---
id: projects-jet-logic-wasm-renderer-webgpu-strokerect-md
fill_sections: [changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# WebGPU backend — StrokeRect lowering

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: ".aw/tech-design/projects/jet/logic/wasm-renderer-webgpu-strokerect.md"
    action: modify
    section: doc
    impl_mode: hand-written
    description: |
      Legacy Jet TD content retained as notes during AW standardization.
      Rewrite this file into semantic TD sections before promoting source to CODEGEN.
```

## Legacy notes
<!-- type: doc lang: markdown -->

# WebGPU backend — StrokeRect lowering

### Overview

Parent issue: #2117. Parent spec: [`wasm-renderer-webgpu-backend.md`](./wasm-renderer-webgpu-backend.md).

The WebGPU paint planner currently reports `PaintOp::StrokeRect` as
unsupported, leaving a visible parity gap between the canvas backend
(which uses the browser's `strokeRect`) and the WebGPU backend (which
emits zero cells for the same op).

This slice closes that gap by lowering one `StrokeRect` into up to four
`CellInstance`s — top, bottom, left, right edge strips of the outline —
reusing the existing cell-rect pipeline. No new GPU work, no atlas, no
text-pass dependency.

Stroke geometry follows canvas's "center-aligned" convention: the
`width`-thick stroke is centered on the rect's path, so the outer
bounds of the drawn pixels are `(x - width/2, y - width/2) ..
(x + w + width/2, y + h + width/2)`. Top + bottom span the full
horizontal extent including corner areas; left + right span only the
*middle* (excluding the rows the top/bottom already cover) so the four
strips tile without overlap. The no-overlap layout keeps translucent
strokes from double-blending at corners, matching the canvas semantic.

```
+----+----------+----+   <- Top: full width, height = stroke_width
|    |          |    |
+----+----------+----+
|Left|          |Rt  |   <- Left/Right: middle only, height = h - stroke_width
|    |          |    |
+----+----------+----+
+----+----------+----+   <- Bottom: full width, height = stroke_width
|    |          |    |
+----+----------+----+
```

Degenerate cases collapse cleanly:
- `width <= 0` → zero cells.
- `w <= 0` or `h <= 0` → zero cells.
- `width >= h` → left/right strips have non-positive height → skip; top + bottom remain.
- `width >= w` → top/bottom still span their (non-positive) width → skip; left + right remain.
- If both axes collapse → zero cells.

### Requirements

```mermaid
---
id: jet-wasm-renderer-webgpu-strokerect-requirements
entry: S1
---
requirementDiagram
    requirement S1 { id: S1 text: StrokeRect lowers to up to four CellInstances risk: high verifymethod: test }
    requirement S2 { id: S2 text: Top and Bottom strips span the full horizontal extent risk: medium verifymethod: test }
    requirement S3 { id: S3 text: Left and Right strips span the middle vertical extent only risk: medium verifymethod: test }
    requirement S4 { id: S4 text: Stroke is center-aligned on the rect path risk: medium verifymethod: test }
    requirement S5 { id: S5 text: Degenerate inputs emit zero cells risk: medium verifymethod: test }
    requirement S6 { id: S6 text: Thick stroke collapses to top plus bottom when stroke greater than or equal height risk: medium verifymethod: test }
    requirement S7 { id: S7 text: Color forwarded via the existing fill rect to cell helper risk: low verifymethod: test }
    requirement S8 { id: S8 text: WebGpuUnsupportedOp StrokeRect variant removed risk: low verifymethod: build }
```

| id | Requirement | Verifies |
|----|-------------|----------|
| S1 | `WebGpuBackend::plan` lowers `PaintOp::StrokeRect { rect, color, width }` to ≤4 `CellInstance`s appended to `cells`; no `unsupported` entry is emitted for stroke. | unit test |
| S2 | Top + Bottom strips both span horizontal `(rect.x - width/2 .. rect.x + rect.w + width/2)` with height = `width`. | unit test |
| S3 | Left + Right strips span vertical `(rect.y + width/2 .. rect.y + rect.h - width/2)` with width = `width`. (Middle only — no corner overlap with top/bottom.) | unit test |
| S4 | Top strip y = `rect.y - width/2`; Bottom strip y = `rect.y + rect.h - width/2`; Left strip x = `rect.x - width/2`; Right strip x = `rect.x + rect.w - width/2`. | unit test |
| S5 | `width <= 0.0`, `rect.w <= 0.0`, or `rect.h <= 0.0` produce zero new `CellInstance`s and zero `unsupported` entries. | unit test |
| S6 | When `width >= rect.h`, only top + bottom are emitted (left + right would have non-positive height and are skipped). Symmetrically for `width >= rect.w`. | unit test |
| S7 | Color is converted via the same `Color → [f32; 4]` mapping used by `fill_rect_to_cell` (straight RGBA in 0..=1). | unit test |
| S8 | `WebGpuUnsupportedOp` no longer contains a `StrokeRect` variant. | `cargo build -p jet-wasm --features webgpu` |

### Interfaces

```rust
// crates/jet/wasm/src/renderer/webgpu.rs

/// Lower a single StrokeRect into up to four CellInstances representing
/// a hollow rectangle outline. Returns the cells; the caller appends
/// them to the frame plan.
///
/// Stroke is center-aligned on the rect path (canvas `strokeRect`
/// convention). Top/Bottom strips span the full horizontal extent
/// (including corner areas); Left/Right strips span only the middle so
/// the four strips tile without overlap.
fn lower_stroke_rect(rect: Rect, color: Color, width: f32) -> Vec<CellInstance>;

pub enum WebGpuUnsupportedOp {
    // StrokeRect variant removed — strokes now lower to cells.
    Text,
    Clip,
}
```

### Changes

```yaml
changes:
  - path: projects/jet/wasm/src/renderer/webgpu.rs
    kind: update
    section: doc
    impl_mode: hand-written
    summary: Replace the StrokeRect unsupported-op short-circuit with a four-CellInstance lowering and remove the WebGpuUnsupportedOp::StrokeRect variant.
  - path: .aw/tech-design/projects/jet/logic/wasm-renderer-webgpu-backend.md
    kind: update
    section: doc
    impl_mode: hand-written
    summary: Drop StrokeRect from W3 (unsupported ops are now Text and Clip only); add W12 cross-reference to this slice doc.
```
