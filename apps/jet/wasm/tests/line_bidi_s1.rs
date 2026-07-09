// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#tests
// CODEGEN-BEGIN
//! Line-bidi S1: ASCII no-bidi single run.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/line-bidi.md#scenarios

use jet_wasm::text::{script_runs, BidiResolver, Direction, LineBreaker};

#[test]
fn s1_hello_world_single_ltr_run() {
    let s = "Hello world";
    let bidi = BidiResolver::resolve(s, Direction::Ltr);
    assert!(!bidi.is_empty());
    assert!(
        bidi.iter().all(|r| r.level % 2 == 0),
        "all LTR (even levels)"
    );
}

#[test]
fn s1_hello_world_single_latin_script_run() {
    let s = "Hello world";
    let runs = script_runs(s);
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].script, "Latin");
    assert_eq!(runs[0].byte_range, 0..s.len());
}

#[test]
fn s1_hello_world_has_line_breaks_at_spaces() {
    let s = "Hello world";
    let v: Vec<_> = LineBreaker::new(s).collect();
    assert!(!v.is_empty());
    // At least one Allowed break and a Mandatory at end-of-text.
    assert!(v
        .iter()
        .any(|op| matches!(op.kind, jet_wasm::text::LineBreakKind::Allowed)));
    assert_eq!(
        v.last().unwrap().kind,
        jet_wasm::text::LineBreakKind::Mandatory
    );
    assert_eq!(v.last().unwrap().byte_offset, s.len());
}
// CODEGEN-END
