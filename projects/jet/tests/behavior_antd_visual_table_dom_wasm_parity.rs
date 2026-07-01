// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/3786.md#antd-visual-table-dom-wasm-parity
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec antd-visual-table-dom-wasm-parity
// @capability browser-trace-parity
// @claim antd-visual-table-dom-wasm-parity
// @contract antd-visual-table-dom-wasm-parity
// @category behavior
// @required_for_production true
// @command cargo test -p jet --test mui_visual_regression antd_visual_fixture_renders_on_react_dom_and_jet_wasm -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn antd_visual_table_dom_wasm_parity() {
    let command =
        "cargo test -p jet --test mui_visual_regression antd_visual_fixture_renders_on_react_dom_and_jet_wasm -- --nocapture";
    let id = "antd-visual-table-dom-wasm-parity";
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
