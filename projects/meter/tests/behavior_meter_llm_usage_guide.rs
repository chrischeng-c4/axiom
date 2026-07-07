// SPEC-MANAGED: projects/meter/tech-design/logic/external-contracts.md#meter-llm-usage-guide
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec meter-llm-usage-guide
// @capability agent-use-first-cli
// @claim llm-usage-guide
// @contract llm-usage-guide
// @category behavior
// @required_for_production true
// @command cargo run -p meter-cli --bin meter -- llm guide
// AW-EC-END

// Contract: meter llm guide emits the agent-facing usage contract without target setup
// Contract: the guide identifies meter as a resource measurement tool rather than a security scanner or test framework
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn meter_llm_usage_guide() {
    let command = "cargo run -p meter-cli --bin meter -- llm guide";
    let id = "meter-llm-usage-guide";
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
