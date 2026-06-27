// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/3780.md#prebundle-importmap-parity
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec prebundle-importmap-parity
// @capability dev-server-hmr
// @claim prebundle-importmap-parity
// @contract prebundle-importmap-parity
// @category behavior
// @required_for_production true
// @command cargo test -p jet --lib dev_server::prebundle -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn prebundle_importmap_parity() {
    let command = "cargo test -p jet --lib dev_server::prebundle -- --nocapture";
    let id = "prebundle-importmap-parity";
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
