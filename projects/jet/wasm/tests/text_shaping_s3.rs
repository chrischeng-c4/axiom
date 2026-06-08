// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#tests
// CODEGEN-BEGIN
//! Text-shaping S3: Missing-glyph error.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/text-shaping.md#scenarios
//!
//! L0 pure-Rust tier. The full font-dependent assertion is
//! `#[ignore]`'d; the structural error variant + display contract
//! are exercised inline.

use jet_wasm::text::FontError;

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
#[ignore = "S3 end-to-end — needs an embedded TEST_FONT_BYTES that lacks U+1F600"]
fn s3_shape_with_missing_glyph_returns_error() {
    // GIVEN a Latin-only font lacking emoji glyphs.
    // WHEN  shape_text(font, "\u{1F600}", 16.0) is called.
    // THEN  Result is Err(FontError::GlyphMissing(0x1F600)).
}
// CODEGEN-END
