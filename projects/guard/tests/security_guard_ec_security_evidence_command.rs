// SPEC-MANAGED: projects/guard/tech-design/semantic/guard-ec-static-security-smoke.md#guard-ec-security-evidence-command
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec guard-ec-security-evidence-command
// @capability security-ec-profile
// @claim ec-security-evidence-command
// @contract ec-security-evidence-command
// @category security
// @required_for_production true
// @command target/debug/guard scan projects/guard --profile security-lint --compact --no-persist --vat-runner guard-security-smoke --rig-scenario projects/guard/tests/rig/scenarios/security/guard_self_scan.toml --meter-target projects/guard --arena-command "target/debug/arena spec --compact"
// AW-EC-END

// Contract: guard scan runs the full configured EC evidence command
// Contract: vat, rig, meter, and arena evidence adapters fold into guard.report/1
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn guard_ec_security_evidence_command() {
    let command = "target/debug/guard scan projects/guard --profile security-lint --compact --no-persist --vat-runner guard-security-smoke --rig-scenario projects/guard/tests/rig/scenarios/security/guard_self_scan.toml --meter-target projects/guard --arena-command \"target/debug/arena spec --compact\"";
    let id = "guard-ec-security-evidence-command";
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
