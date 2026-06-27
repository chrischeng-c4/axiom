// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/3785.md#built-in-ts-test-runtime
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec built-in-ts-test-runtime
// @capability native-test-product-flow-e2e
// @claim built-in-ts-test-runtime
// @contract built-in-ts-test-runtime
// @category behavior
// @required_for_production true
// @command cargo test -p jet --lib test_runner -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn built_in_ts_test_runtime() {
    let command = "cargo test -p jet --lib test_runner -- --nocapture";
    let id = "built-in-ts-test-runtime";
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
