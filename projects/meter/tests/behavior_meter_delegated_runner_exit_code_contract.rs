// SPEC-MANAGED: projects/meter/tech-design/logic/external-contracts.md#meter-delegated-runner-exit-code-contract
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec meter-delegated-runner-exit-code-contract
// @capability agent-use-first-cli
// @claim delegated-runner-exit-code-contract
// @contract delegated-runner-exit-code-contract
// @category behavior
// @required_for_production true
// @command cargo test -p meter report::builder::tests::forward_exit_overrides_natural_code
// AW-EC-END

// Contract: delegated exit-code forwarding remains encoded in report builder behavior
// Contract: meter-native status does not hide delegated runner exit semantics
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn meter_delegated_runner_exit_code_contract() {
    let command = "cargo test -p meter report::builder::tests::forward_exit_overrides_natural_code";
    let id = "meter-delegated-runner-exit-code-contract";
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
