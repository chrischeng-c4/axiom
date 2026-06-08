// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#tests
// CODEGEN-BEGIN
//! Line-bidi S7: empty string fast path.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/line-bidi.md#scenarios

use jet_wasm::text::{script_runs, BidiResolver, Direction, LineBreaker, Paragraph};

#[test]
fn s7_paragraph_empty_constructor_all_empty() {
    let p = Paragraph::empty();
    assert!(p.runs.is_empty());
    assert!(p.line_breaks.is_empty());
    assert!(p.bidi_runs.is_empty());
    assert!(p.script_runs.is_empty());
}

#[test]
fn s7_empty_string_each_layer_returns_empty() {
    assert!(BidiResolver::resolve("", Direction::Ltr).is_empty());
    assert!(script_runs("").is_empty());
    let breaks: Vec<_> = LineBreaker::new("").collect();
    assert!(breaks.is_empty());
}
// CODEGEN-END
