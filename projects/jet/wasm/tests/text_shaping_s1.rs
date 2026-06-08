// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#tests
// CODEGEN-BEGIN
//! Text-shaping S1: Latin shaping round-trip.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/text-shaping.md#scenarios
//!
//! L0 pure-Rust tier. The font-dependent end-to-end check is
//! `#[ignore]`'d pending an embedded TEST_FONT_BYTES asset; the
//! structural contracts that don't need a font are exercised inline.

use jet_wasm::text::{shape_text, FontError, FontFace};

#[test]
fn s1_invalid_font_bytes_rejected() {
    let err = FontFace::from_bytes(b"not a font").unwrap_err();
    assert!(matches!(err, FontError::InvalidData(_)));
}

#[test]
#[ignore = "S1 latin round-trip — needs an embedded TEST_FONT_BYTES asset"]
fn s1_latin_shaping_round_trip() {
    // GIVEN a known TTF font with a Latin codepage.
    // WHEN  shape_text(font, "Hello", 16.0) is called.
    // THEN  Result is Ok(ShapedRun) with glyphs.len() == 5
    //       AND advance_width matches the font's metric for "Hello".
    //
    // Activation: replace `b""` with an `include_bytes!(...)` of a
    // permissively-licensed TTF (e.g. Inter, Noto Sans) once a font
    // asset is checked into `projects/jet/wasm/tests/fixtures/`.
    let bytes: &[u8] = b"";
    let font = FontFace::from_bytes(bytes).expect("test font loads");
    let run = shape_text(&font, "Hello", 16.0).expect("Hello shapes");
    assert_eq!(run.glyphs.len(), 5);
    assert!(run.advance_width > 0.0);
    assert!(run.line_height() > 0.0);
}
// CODEGEN-END
