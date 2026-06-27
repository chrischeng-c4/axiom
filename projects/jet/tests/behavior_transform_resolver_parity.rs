// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/3782.md#transform-resolver-parity
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec transform-resolver-parity
// @capability bundler-production-build
// @claim transform-resolver-parity
// @contract transform-resolver-parity
// @category behavior
// @required_for_production true
// @command cargo test -p jet --lib transform -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn transform_resolver_parity() {
    let command = "cargo test -p jet --lib transform -- --nocapture";
    let id = "transform-resolver-parity";
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
