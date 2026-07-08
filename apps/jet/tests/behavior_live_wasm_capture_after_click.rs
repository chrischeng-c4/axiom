// SPEC-MANAGED: .aw/tech-design/projects/jet/logic/jet-browser-observation-bundle.md#live-wasm-capture-after-click
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec live-wasm-capture-after-click
// @capability browser-trace-parity
// @claim browser-automation-diagnostics
// @contract live-wasm-capture-after-click
// @category behavior
// @required_for_production true
// @command cargo test -p jet --test browser_cli_smoke -- --nocapture
// AW-EC-END

// Contract: bundle.schema_version == jet.browser.observation.v1
// Contract: bundle.layout_tree is non-empty
// Contract: bundle.paint_ops is non-empty
// Contract: bundle.hook_values includes the post-click counter value
// Contract: no python static server is used
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn live_wasm_capture_after_click() {
    let command = "cargo test -p jet --test browser_cli_smoke -- --nocapture";
    let id = "live-wasm-capture-after-click";
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
