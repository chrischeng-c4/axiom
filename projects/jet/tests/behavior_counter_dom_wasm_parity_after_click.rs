// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/3941.md#counter-dom-wasm-parity-after-click
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec counter-dom-wasm-parity-after-click
// @capability browser-trace-parity
// @claim parity-corpus-gates
// @contract counter-dom-wasm-parity-after-click
// @category behavior
// @required_for_production true
// @command cargo test -p jet --test react_dom_oracle_conformance counter_demo_matches_react_dom_oracle_initial_and_after_click -- --nocapture
// AW-EC-END

// Contract: initial DOM tree equals normalized Jet WASM element tree
// Contract: post-click DOM tree equals normalized Jet WASM element tree
// Contract: WASM observation includes hook value 1 after click
// Contract: mismatch output includes concrete DOM and WASM JSON
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn counter_dom_wasm_parity_after_click() {
    let command =
        "cargo test -p jet --test react_dom_oracle_conformance counter_demo_matches_react_dom_oracle_initial_and_after_click -- --nocapture";
    let id = "counter-dom-wasm-parity-after-click";
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
