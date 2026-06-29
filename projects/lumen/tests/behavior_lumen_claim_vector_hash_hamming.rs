// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-vector-hash-hamming
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-vector-hash-hamming
// @capability vector-hash-search
// @claim hash-hamming-search
// @contract hash-hamming-search
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --test hash_hamming -- --nocapture
// AW-EC-END

// Contract: Hash Hamming search returns bounded-distance matches over the hash index.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_vector_hash_hamming() {
    let command = "cargo test -p lumen --test hash_hamming -- --nocapture";
    let id = "lumen-claim-vector-hash-hamming";
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
