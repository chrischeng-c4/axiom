// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#tests
// CODEGEN-BEGIN
//! Line-bidi S6: Auto direction detection (UAX #9 P2).
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/line-bidi.md#scenarios

use jet_wasm::text::{BidiResolver, Direction};

#[test]
fn s6_auto_first_strong_arabic_yields_rtl_paragraph() {
    let s = "\u{0645}\u{0631}\u{062D}\u{0628}\u{0627} hello";
    let runs = BidiResolver::resolve(s, Direction::Auto);
    let first = runs
        .iter()
        .find(|r| r.byte_range.start == 0)
        .expect("first run starts at 0");
    assert_eq!(first.level % 2, 1, "first strong is Arabic → RTL");
}

#[test]
fn s6_auto_first_strong_latin_yields_ltr_paragraph() {
    let s = "Hello \u{0645}\u{0631}\u{062D}\u{0628}\u{0627}";
    let runs = BidiResolver::resolve(s, Direction::Auto);
    let first = runs
        .iter()
        .find(|r| r.byte_range.start == 0)
        .expect("first run starts at 0");
    assert_eq!(first.level % 2, 0, "first strong is Latin → LTR");
}

#[test]
fn s6_auto_no_strong_defaults_to_ltr() {
    // Pure punctuation/whitespace has no strong character; per UAX #9 P3
    // the paragraph defaults to Ltr (level 0).
    let s = "   ";
    let runs = BidiResolver::resolve(s, Direction::Auto);
    if let Some(first) = runs.first() {
        assert_eq!(first.level % 2, 0, "no strong → LTR default");
    }
}
// CODEGEN-END
