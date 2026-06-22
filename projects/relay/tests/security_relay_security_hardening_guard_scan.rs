// SPEC-MANAGED: projects/relay/external-contracts/security-hardening/security/security-evidence.md#relay-security-hardening-guard-scan
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec relay-security-hardening-guard-scan
// @capability security-hardening
// @claim guard-static-runtime-evidence
// @contract relay-guard-security-report
// @category security
// @required_for_production false
// @command cd projects/relay && ../../target/debug/vat run guard-security
// AW-EC-END

// Contract: guard scan over relay reports no untriaged Docker, Kubernetes, or static security findings.
// Contract: guard attaches meter evidence for relay_core opaque-payload request-boundary smoke.
// Contract: The security evidence runs inside vat so generated reports and transient files do not mutate the host checkout.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn relay_security_hardening_guard_scan() {
    let command = "cd projects/relay && ../../target/debug/vat run guard-security";
    let id = "relay-security-hardening-guard-scan";
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
