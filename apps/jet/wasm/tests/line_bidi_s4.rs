// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#tests
// CODEGEN-BEGIN
//! Line-bidi S4: mixed-script Latin + Han (R10 acceptance criterion).
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/line-bidi.md#scenarios

use jet_wasm::text::script_runs;

#[test]
fn s4_mixed_latin_han_yields_both_scripts() {
    let s = "abc 你好";
    let runs = script_runs(s);
    let scripts: Vec<&str> = runs.iter().map(|r| r.script.as_str()).collect();
    assert!(
        scripts.contains(&"Latin"),
        "expected at least one Latin run, got: {scripts:?}"
    );
    assert!(
        scripts.contains(&"Han"),
        "expected at least one Han run, got: {scripts:?}"
    );
}

#[test]
fn s4_mixed_run_byte_ranges_non_overlapping_full_coverage() {
    let s = "abc 你好";
    let runs = script_runs(s);
    assert_eq!(runs.first().unwrap().byte_range.start, 0);
    assert_eq!(runs.last().unwrap().byte_range.end, s.len());
    for w in runs.windows(2) {
        assert_eq!(
            w[0].byte_range.end, w[1].byte_range.start,
            "runs must be contiguous (non-overlapping, full coverage)"
        );
    }
}
// CODEGEN-END
