// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#tests
// CODEGEN-BEGIN
//! Text-shaping S1: Latin shaping round-trip.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/text-shaping.md#scenarios
//!
//! L0 pure-Rust tier. The font-dependent end-to-end check uses the
//! same vendored Tuffy face as the WebGPU glyph atlas bridge.

use jet_wasm::text::{shape_text, FontError, FontFace};

mod common;

#[test]
fn s1_invalid_font_bytes_rejected() {
    let err = FontFace::from_bytes(b"not a font").unwrap_err();
    assert!(matches!(err, FontError::InvalidData(_)));
}

#[test]
fn s1_latin_shaping_round_trip() {
    let font = common::tuffy_regular();
    let run = shape_text(&font, "Hello", 16.0).expect("Hello shapes");
    assert_eq!(run.glyphs.len(), 5);
    assert!(run.advance_width > 0.0);
    assert!(run.line_height() > 0.0);
}
// CODEGEN-END
