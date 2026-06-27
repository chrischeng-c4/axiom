// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/4004.md#dom-renderer-controlled-input-parity
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec dom-renderer-controlled-input-parity
// @capability browser-trace-parity
// @claim dom-renderer-controlled-input-parity
// @contract dom-renderer-controlled-input-parity
// @category behavior
// @required_for_production true
// @command cargo test -p jet --test react_dom_oracle_conformance dom_renderer_controlled_input_parity -- --nocapture
// AW-EC-END

// Contract: React DOM and Jet WASM DOM renderer initial host trees match.
// Contract: React DOM and Jet WASM DOM renderer initial input states match.
// Contract: After replacing text, input values match.
// Contract: After replacing text, derived label text matches.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn dom_renderer_controlled_input_parity() {
    let command =
        "cargo test -p jet --test react_dom_oracle_conformance dom_renderer_controlled_input_parity -- --nocapture";
    let id = "dom-renderer-controlled-input-parity";
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
