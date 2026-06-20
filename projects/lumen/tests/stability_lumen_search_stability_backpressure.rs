// SPEC-MANAGED: projects/lumen/external-contracts/search/stability/query-resilience.md#lumen-search-stability-overload-backpressure
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-search-stability-overload-backpressure
// @capability search
// @claim graceful-degradation-under-overload
// @contract search-stability-backpressure
// @category stability
// @required_for_production true
// @command rig run --dir projects/lumen/tests/rig/cases/load --pins projects/lumen/tests/rig/config/pins
// AW-EC-END

// Contract: (d) Under 3x steady-state concurrent load the server stays up and bounded: error_rate <= 0.05 and p99 <= 250ms (rig load/backpressure_overload.toml + pins); no OOM/crash. Env-dependent (vat-provisioned lumen).
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_search_stability_overload_backpressure() {
    let command =
        "rig run --dir projects/lumen/tests/rig/cases/load --pins projects/lumen/tests/rig/config/pins";
    let id = "lumen-search-stability-overload-backpressure";
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
