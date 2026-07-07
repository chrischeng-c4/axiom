// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/4072.md#library-dom-wasm-parity
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec library-dom-wasm-parity
// @capability browser-trace-parity
// @claim library-dom-wasm-parity-fixtures
// @contract library-dom-wasm-parity
// @category behavior
// @required_for_production true
// @command cargo test -p jet --test react_dom_oracle_conformance library_dom_wasm_parity -- --nocapture
// AW-EC-END

// Contract: Each fixture renders in React DOM and Jet WASM DOM.
// Contract: Initial visible form state matches for executable form-control fixtures.
// Contract: Post-input or post-click visible state matches when a fixture declares an interaction.
// Contract: Failures identify library id, fixture id, phase, expected state, and actual state.
// Contract: The test uses Jet browser observation helpers and no Python test server.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn library_dom_wasm_parity() {
    let command =
        "cargo test -p jet --test react_dom_oracle_conformance library_dom_wasm_parity -- --nocapture";
    let id = "library-dom-wasm-parity";
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
