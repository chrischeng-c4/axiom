// SPEC-MANAGED: projects/keep/external-contracts/security-hardening/security/security-evidence.md#keep-security-hardening-guard-scan
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec keep-security-hardening-guard-scan
// @capability security-hardening
// @claim guard-static-runtime-evidence
// @contract keep-guard-security-report
// @category security
// @required_for_production false
// @command cd projects/keep && ../../target/debug/vat run guard-security
// AW-EC-END

// Contract: guard scan over keep reports no untriaged Docker, Kubernetes, or static security findings.
// Contract: guard attaches meter evidence for Keep's public HTTP route smoke.
// Contract: The security evidence runs inside vat so generated reports and transient files do not mutate the host checkout.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn keep_security_hardening_guard_scan() {
    let command = "cd projects/keep && ../../target/debug/vat run guard-security";
    let id = "keep-security-hardening-guard-scan";
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
