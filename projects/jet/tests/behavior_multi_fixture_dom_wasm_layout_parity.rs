// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/3944.md#multi-fixture-dom-wasm-layout-parity
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec multi-fixture-dom-wasm-layout-parity
// @capability browser-trace-parity
// @claim parity-corpus-gates
// @contract multi-fixture-dom-wasm-layout-parity
// @category behavior
// @required_for_production true
// @command cargo test -p jet --test react_dom_oracle_conformance multi_fixture_dom_wasm_layout_parity -- --nocapture
// AW-EC-END

// Contract: static fixture initial DOM/WASM layout boxes match
// Contract: class/state fixture initial and after-click DOM/WASM layout boxes match
// Contract: list/state fixture initial and after-click DOM/WASM layout boxes match
// Contract: layout mismatches include fixture id, phase, expected, and actual JSON
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn multi_fixture_dom_wasm_layout_parity() {
    let command =
        "cargo test -p jet --test react_dom_oracle_conformance multi_fixture_dom_wasm_layout_parity -- --nocapture";
    let id = "multi-fixture-dom-wasm-layout-parity";
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
