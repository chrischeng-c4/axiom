// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/3972.md#multi-fixture-dom-wasm-screenshot-pixel-parity
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec multi-fixture-dom-wasm-screenshot-pixel-parity
// @capability browser-trace-parity
// @claim parity-corpus-gates
// @contract multi-fixture-dom-wasm-screenshot-pixel-parity
// @category behavior
// @required_for_production true
// @command cargo test -p jet --test react_dom_oracle_conformance multi_fixture_dom_wasm_screenshot_pixel_parity -- --nocapture
// AW-EC-END

// Contract: React DOM and Jet WASM screenshots decode as PNG.
// Contract: Viewport dimensions match.
// Contract: Foreground bounds match within bounds_css_px.
// Contract: Foreground pixel counts match within foreground_count_ratio.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn multi_fixture_dom_wasm_screenshot_pixel_parity() {
    let command =
        "cargo test -p jet --test react_dom_oracle_conformance multi_fixture_dom_wasm_screenshot_pixel_parity -- --nocapture";
    let id = "multi-fixture-dom-wasm-screenshot-pixel-parity";
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
