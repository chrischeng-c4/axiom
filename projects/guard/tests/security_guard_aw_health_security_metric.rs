// SPEC-MANAGED: projects/guard/tech-design/semantic/guard-ec-static-security-smoke.md#guard-aw-health-security-metric
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec guard-aw-health-security-metric
// @capability security-ec-profile
// @claim aw-health-security-metric
// @contract aw-health-security-metric
// @category security
// @required_for_production true
// @command ./target/debug/aw ec check --project guard
// AW-EC-END

// Contract: AW EC check consumes guard report evidence as a first-class security metric
// Contract: guard EC inventory remains generated and drift-free
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn guard_aw_health_security_metric() {
    let command = "./target/debug/aw ec check --project guard";
    let id = "guard-aw-health-security-metric";
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
