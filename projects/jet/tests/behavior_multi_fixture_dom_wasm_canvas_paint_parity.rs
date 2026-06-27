// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/3958.md#multi-fixture-dom-wasm-canvas-paint-parity
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec multi-fixture-dom-wasm-canvas-paint-parity
// @capability browser-trace-parity
// @claim parity-corpus-gates
// @contract multi-fixture-dom-wasm-canvas-paint-parity
// @category behavior
// @required_for_production true
// @command cargo test -p jet --test react_dom_oracle_conformance multi_fixture_dom_wasm_canvas_paint_parity -- --nocapture
// AW-EC-END

// Contract: initial paintOps-derived method sequence is observed in browser canvas calls
// Contract: after-click paintOps-derived method sequence is observed in browser canvas calls for interactive fixtures
// Contract: mismatch output is machine-readable JSON
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn multi_fixture_dom_wasm_canvas_paint_parity() {
    let command =
        "cargo test -p jet --test react_dom_oracle_conformance multi_fixture_dom_wasm_canvas_paint_parity -- --nocapture";
    let id = "multi-fixture-dom-wasm-canvas-paint-parity";
    let mut root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    while !root.join(".aw").is_dir() {
        assert!(
            root.pop(),
            "AW EC {id}: no .aw/ project root above {}",
            env!("CARGO_MANIFEST_DIR")
        );
    }
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(&root)
        .status()
        .unwrap_or_else(|e| panic!("AW EC {id}: failed to spawn `{command}`: {e}"));
    assert!(
        status.success(),
        "AW EC {id} FAILED (exit {:?}): {command}",
        status.code()
    );
}
// CODEGEN-END
