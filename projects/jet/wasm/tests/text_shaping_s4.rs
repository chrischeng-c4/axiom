// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#tests
// CODEGEN-BEGIN
//! Text-shaping S4: Empty string fast path.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/text-shaping.md#scenarios
//!
//! L0 pure-Rust tier. The empty-string path is exercised on the
//! `ShapedRun::empty` constructor (which `shape_text` returns directly
//! for `text == ""` without touching rustybuzz). The end-to-end check
//! through a real font is `#[ignore]`'d pending an embedded asset.

use jet_wasm::text::ShapedRun;

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
#[ignore = "S4 end-to-end — needs an embedded TEST_FONT_BYTES asset"]
fn s4_shape_empty_string_returns_metrics_only() {
    // GIVEN a real font face.
    // WHEN  shape_text(font, "", 16.0) is called.
    // THEN  Result is Ok(ShapedRun) with glyphs.is_empty()
    //       AND advance_width == 0.0
    //       AND ascent > 0 AND descent > 0
    //       AND no rustybuzz call was made (fast path).
}
// CODEGEN-END
