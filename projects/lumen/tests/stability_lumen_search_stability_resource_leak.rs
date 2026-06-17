// SPEC-MANAGED: projects/lumen/external-contracts/search/stability/query-resilience.md#lumen-search-stability-resource-leak
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-search-stability-resource-leak
// @capability search
// @claim no-fd-socket-thread-leak
// @contract search-stability-resource-leak
// @category stability
// @required_for_production true
// @command target/debug/rig test --dir projects/lumen/tests/rig/cases/endurance
// AW-EC-END

// Contract: (e) Open FD count after sustained index+search load <= 1.20x before + 16 (rig endurance/fd_leak.toml). Env-dependent (vat-provisioned lumen).
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_search_stability_resource_leak() {
    let command = "target/debug/rig test --dir projects/lumen/tests/rig/cases/endurance";
    let id = "lumen-search-stability-resource-leak";
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
