// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#tests
// CODEGEN-BEGIN
//! Text-shaping S3: Missing-glyph error.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/text-shaping.md#scenarios
//!
//! L0 pure-Rust tier. The structural error variant, display contract,
//! and real-font missing-glyph path are exercised inline.

use jet_wasm::text::{shape_text, FontError};

mod common;

#[test]
fn s3_glyph_missing_carries_codepoint() {
    let err = FontError::GlyphMissing(0x1F600);
    match err {
        FontError::GlyphMissing(cp) => assert_eq!(cp, 0x1F600),
        _ => panic!("expected GlyphMissing variant"),
    }
}

#[test]
fn s3_glyph_missing_display_format() {
    let err = FontError::GlyphMissing(0x1F600);
    let s = err.to_string();
    assert!(s.contains("U+1F600"), "got: {s}");
}

#[test]
fn s3_shape_with_missing_glyph_returns_error() {
    let font = common::tuffy_regular();
    let err = shape_text(&font, "\u{1F600}", 16.0).unwrap_err();
    assert!(matches!(err, FontError::GlyphMissing(0x1F600)));
}
// CODEGEN-END
