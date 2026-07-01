// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/3786.md#webgpu-large-table-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec webgpu-large-table-smoke
// @capability browser-trace-parity
// @claim webgpu-large-table-smoke
// @contract webgpu-large-table-smoke
// @category behavior
// @required_for_production true
// @command cargo test -p jet --test wasm_build_end_to_end webgpu_renderer_reports_runtime_status_and_visual_probe_when_available -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn webgpu_large_table_smoke() {
    let command =
        "cargo test -p jet --test wasm_build_end_to_end webgpu_renderer_reports_runtime_status_and_visual_probe_when_available -- --nocapture";
    let id = "webgpu-large-table-smoke";
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
