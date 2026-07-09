// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#tests
// CODEGEN-BEGIN
//! Line-bidi S3: bidirectional Latin + Arabic.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/line-bidi.md#scenarios

use jet_wasm::text::{BidiResolver, Direction};

#[test]
fn s3_latin_arabic_two_levels() {
    let s = "Hello \u{0645}\u{0631}\u{062D}\u{0628}\u{0627}";
    let runs = BidiResolver::resolve(s, Direction::Ltr);
    let has_ltr = runs.iter().any(|r| r.level % 2 == 0);
    let has_rtl = runs.iter().any(|r| r.level % 2 == 1);
    assert!(has_ltr, "expected at least one LTR (Latin) run");
    assert!(has_rtl, "expected at least one RTL (Arabic) run");
}

#[test]
fn s3_runs_in_logical_byte_order() {
    let s = "Hello \u{0645}\u{0631}\u{062D}\u{0628}\u{0627}";
    let runs = BidiResolver::resolve(s, Direction::Ltr);
    let mut prev = 0_usize;
    for r in &runs {
        assert!(r.byte_range.start >= prev);
        prev = r.byte_range.end;
    }
}
// CODEGEN-END
