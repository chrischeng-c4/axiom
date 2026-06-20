// SPEC-MANAGED: projects/lumen/external-contracts/security-hardening/security/access-control.md#lumen-security-hardening-query-injection
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-security-hardening-query-injection
// @capability security-hardening
// @claim adversarial-query-safety
// @contract search-security-injection
// @category security
// @required_for_production false
// @command cargo test -p lumen --test coverage_gaps_e2e search_security_query_injection_rejects_bad_queries -- --nocapture
// AW-EC-END

// Contract: C2: malformed JSON, deeply-nested JSON query DSL, special-char search text, inverted ranges, and range numeric overflow are rejected or evaluated safely (no panic, no 5xx, bounded work).
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_security_hardening_query_injection() {
    let command =
        "cargo test -p lumen --test coverage_gaps_e2e search_security_query_injection_rejects_bad_queries -- --nocapture";
    let id = "lumen-security-hardening-query-injection";
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
