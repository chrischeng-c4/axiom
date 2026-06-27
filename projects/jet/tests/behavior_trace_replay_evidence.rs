// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/3784.md#trace-replay-evidence
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec trace-replay-evidence
// @capability native-test-product-flow-e2e
// @claim trace-replay-evidence
// @contract trace-replay-evidence
// @category behavior
// @required_for_production true
// @command cargo test -p jet --lib trace -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn trace_replay_evidence() {
    let command = "cargo test -p jet --lib trace -- --nocapture";
    let id = "trace-replay-evidence";
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
