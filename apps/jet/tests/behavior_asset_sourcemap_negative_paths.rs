// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/3782.md#asset-sourcemap-negative-paths
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec asset-sourcemap-negative-paths
// @capability bundler-production-build
// @claim asset-sourcemap-negative-paths
// @contract asset-sourcemap-negative-paths
// @category behavior
// @required_for_production true
// @command cargo test -p jet --lib asset -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn asset_sourcemap_negative_paths() {
    let command = "cargo test -p jet --lib asset -- --nocapture";
    let id = "asset-sourcemap-negative-paths";
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
