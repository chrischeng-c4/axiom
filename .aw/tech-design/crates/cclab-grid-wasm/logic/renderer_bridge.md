---
id: crates-cclab-grid-wasm-src-renderer-bridge-rs
fill_sections: [changes]
---

# Standardized crates/cclab-grid-wasm/src/renderer_bridge.rs

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: crates/cclab-grid-wasm/src/renderer_bridge.rs
    action: modify
    impl_mode: hand-written
    description: |
      Existing source claimed by `aw standardize managed run`. This slice
      also adds `renderFrameWithText(cells, textRuns)` as the structured
      browser/Rust wire contract for Jet WebGPU text runs. The method validates
      text-run payloads and preserves the latest text-run count while the lower
      renderer remains cell-pass-only until the glyph/text pass is wired.

      Slice #2191 wires the text pass: after validating runs, the bridge plans
      placeholder `GlyphInstance`s (one per char, mono-advance
      `font_size_px * 0.6`, full-atlas uv, run color) via
      `plan_placeholder_glyphs`, then calls
      `WebGpuRenderer::render_frame_with_text(cells, glyphs)` so the lower
      renderer encodes both the cell pass and the text pass in a single
      submit. The resulting glyph count is observed onto `BridgeState` and
      exposed to JS via the `lastTextGlyphCount()` wasm-bindgen getter so the
      browser e2e (T8) can distinguish encode-fired-empty from
      encode-fired-with-glyphs.
```
