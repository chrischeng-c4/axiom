// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/3785.md#native-test-runner-core
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec native-test-runner-core
// @capability native-test-product-flow-e2e
// @claim native-test-runner-core
// @contract native-test-runner-core
// @category behavior
// @required_for_production true
// @command cargo test -p jet --lib test_runner -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn native_test_runner_core() {
    let command = "cargo test -p jet --lib test_runner -- --nocapture";
    let id = "native-test-runner-core";
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
