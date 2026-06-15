// SPEC-MANAGED: projects/lumen/external-contracts/search/security/access-control.md#lumen-search-security-query-injection
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-search-security-query-injection
// @capability search
// @claim adversarial-query-safety
// @contract search-security-injection
// @category security
// @required_for_production false
// @command cargo test -p lumen --test security_lumen_search_security_query_injection -- --ignored
// AW-EC-END

// Contract: GAP (C2): malformed / oversized / deeply-nested JSON query DSL, special-char search text, and range numeric overflow are rejected safely (no panic, no UB, bounded work). Test not yet written.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_search_security_query_injection() {
    let command = "cargo test -p lumen --test security_lumen_search_security_query_injection -- --ignored";
    let id = "lumen-search-security-query-injection";
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
