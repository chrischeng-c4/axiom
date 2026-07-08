// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/3786.md#trace-evidence-artifacts
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec trace-evidence-artifacts
// @capability browser-trace-parity
// @claim trace-evidence-artifacts
// @contract trace-evidence-artifacts
// @category behavior
// @required_for_production true
// @command cargo test -p jet --lib trace -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn trace_evidence_artifacts() {
    let command = "cargo test -p jet --lib trace -- --nocapture";
    let id = "trace-evidence-artifacts";
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
