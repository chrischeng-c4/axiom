// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-security-tls-rustls
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-security-tls-rustls
// @capability security-hardening
// @claim tls-rustls
// @contract security-tls-rustls
// @category security
// @required_for_production true
// @command cargo test -p lumen tls
// AW-EC-END

// Contract: The rustls-backed TLS surface passes the runtime TLS gate.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_security_tls_rustls() {
    let command = "cargo test -p lumen tls";
    let id = "lumen-claim-security-tls-rustls";
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
