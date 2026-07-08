// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/4015.md#dom-renderer-controlled-textarea-parity
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec dom-renderer-controlled-textarea-parity
// @capability browser-trace-parity
// @claim dom-renderer-controlled-textarea-parity
// @contract dom-renderer-controlled-textarea-parity
// @category behavior
// @required_for_production true
// @command cargo test -p jet --test react_dom_oracle_conformance dom_renderer_controlled_textarea_parity -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn dom_renderer_controlled_textarea_parity() {
    let command =
        "cargo test -p jet --test react_dom_oracle_conformance dom_renderer_controlled_textarea_parity -- --nocapture";
    let id = "dom-renderer-controlled-textarea-parity";
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
