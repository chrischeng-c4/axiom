// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#tests
// CODEGEN-BEGIN
//! Text-shaping S4: Empty string fast path.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/text-shaping.md#scenarios
//!
//! L0 pure-Rust tier. The empty-string path is exercised on both the
//! `ShapedRun::empty` constructor and the same vendored Tuffy face
//! used by the WebGPU glyph atlas bridge.

use jet_wasm::text::{shape_text, ShapedRun};

mod common;

#[test]
fn s4_empty_run_constructor_has_metrics() {
    let run = ShapedRun::empty(12.0, 4.0);
    assert_eq!(run.glyphs.len(), 0);
    assert_eq!(run.advance_width, 0.0);
    assert_eq!(run.ascent, 12.0);
    assert_eq!(run.descent, 4.0);
    assert_eq!(run.line_height(), 16.0);
}

#[test]
fn s4_shape_empty_string_returns_metrics_only() {
    let font = common::tuffy_regular();
    let run = shape_text(&font, "", 16.0).expect("empty text shapes through Tuffy metrics");
    assert!(run.glyphs.is_empty());
    assert_eq!(run.advance_width, 0.0);
    assert!(run.ascent > 0.0);
    assert!(run.descent > 0.0);
}
// CODEGEN-END
