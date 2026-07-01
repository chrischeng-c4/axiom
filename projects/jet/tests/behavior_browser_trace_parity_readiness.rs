// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/3786.md#browser-trace-parity-readiness
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec browser-trace-parity-readiness
// @capability browser-trace-parity
// @claim browser-trace-parity-readiness
// @contract browser-trace-parity-readiness
// @category behavior
// @required_for_production true
// @command projects/jet/scripts/verify-basic-dom-gates.sh --phase browser
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn browser_trace_parity_readiness() {
    let command = "projects/jet/scripts/verify-basic-dom-gates.sh --phase browser";
    let id = "browser-trace-parity-readiness";
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
