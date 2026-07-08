// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/3780.md#dev-server-replacement-readiness
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec dev-server-replacement-readiness
// @capability dev-server-hmr
// @claim dev-server-replacement-readiness
// @contract dev-server-replacement-readiness
// @category behavior
// @required_for_production true
// @command cargo test -p jet --lib dev_server -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn dev_server_replacement_readiness() {
    let command = "cargo test -p jet --lib dev_server -- --nocapture";
    let id = "dev-server-replacement-readiness";
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
