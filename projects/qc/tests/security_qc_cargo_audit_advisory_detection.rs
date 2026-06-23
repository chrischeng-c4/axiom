// SPEC-MANAGED: projects/qc/tech-design/logic/external-contracts.md#qc-cargo-audit-advisory-detection
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec qc-cargo-audit-advisory-detection
// @capability security-check
// @contract cargo-audit-advisory-detection
// @category security
// @command cargo test -p qc --test audit_trust_bug
// AW-EC-END

#[test]
#[ignore = "AW EC placeholder: implement this external contract test or keep the aw.toml inventory command authoritative"]
fn qc_cargo_audit_advisory_detection() {
    panic!(
        "AW EC placeholder for {}",
        "qc-cargo-audit-advisory-detection"
    );
}
// CODEGEN-END
