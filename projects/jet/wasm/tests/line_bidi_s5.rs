// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#tests
// CODEGEN-BEGIN
//! Line-bidi S5: line-break opportunities for Latin punctuation.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/line-bidi.md#scenarios

use jet_wasm::text::{LineBreakKind, LineBreaker};

#[test]
fn s5_hello_world_has_breaks() {
    let s = "Hello, world!";
    let v: Vec<_> = LineBreaker::new(s).collect();
    assert!(!v.is_empty(), "expected non-empty line_breaks");
    // At least one Allowed (after the space).
    assert!(v.iter().any(|op| op.kind == LineBreakKind::Allowed));
    // Mandatory at end-of-text.
    let last = v.last().unwrap();
    assert_eq!(last.byte_offset, s.len());
    assert_eq!(last.kind, LineBreakKind::Mandatory);
}

#[test]
fn s5_byte_offsets_ascending_and_on_char_boundary() {
    let s = "Hello, world!";
    let v: Vec<_> = LineBreaker::new(s).collect();
    let mut prev = 0_usize;
    for op in &v {
        assert!(op.byte_offset > prev, "ascending offsets");
        assert!(s.is_char_boundary(op.byte_offset), "on char boundary");
        prev = op.byte_offset;
    }
}
// CODEGEN-END
