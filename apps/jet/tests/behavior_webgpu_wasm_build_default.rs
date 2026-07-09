// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/3786.md#webgpu-wasm-build-default
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec webgpu-wasm-build-default
// @capability browser-trace-parity
// @claim webgpu-wasm-build-default
// @contract webgpu-wasm-build-default
// @category behavior
// @required_for_production true
// @command cargo test -p jet --test wasm_build_end_to_end wasm_build_selects_webgpu_scaffold_by_default -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn webgpu_wasm_build_default() {
    let command =
        "cargo test -p jet --test wasm_build_end_to_end wasm_build_selects_webgpu_scaffold_by_default -- --nocapture";
    let id = "webgpu-wasm-build-default";
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
