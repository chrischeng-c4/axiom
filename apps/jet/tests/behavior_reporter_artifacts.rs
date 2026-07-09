// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/3785.md#reporter-artifacts
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec reporter-artifacts
// @capability native-test-product-flow-e2e
// @claim reporter-artifacts
// @contract reporter-artifacts
// @category behavior
// @required_for_production true
// @command cargo test -p jet --lib reporter -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn reporter_artifacts() {
    let command = "cargo test -p jet --lib reporter -- --nocapture";
    let id = "reporter-artifacts";
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
