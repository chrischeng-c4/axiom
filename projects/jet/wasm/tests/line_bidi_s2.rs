// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#tests
// CODEGEN-BEGIN
//! Line-bidi S2: pure RTL Arabic.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/line-bidi.md#scenarios

use jet_wasm::text::{script_runs, BidiResolver, Direction};

#[test]
fn s2_pure_arabic_rtl_level() {
    let s = "\u{0645}\u{0631}\u{062D}\u{0628}\u{0627}";
    let runs = BidiResolver::resolve(s, Direction::Rtl);
    assert!(!runs.is_empty());
    // All Arabic strong → odd levels.
    assert!(
        runs.iter().any(|r| r.level % 2 == 1),
        "expected at least one RTL run for Arabic"
    );
}

#[test]
fn s2_pure_arabic_single_arabic_script_run() {
    let s = "\u{0645}\u{0631}\u{062D}\u{0628}\u{0627}";
    let runs = script_runs(s);
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].script, "Arabic");
}
// CODEGEN-END
