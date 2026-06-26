// SPEC-MANAGED: projects/arena/tech-design/logic/external-contracts.md#arena-host-scoped-baseline-ratchet
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec arena-host-scoped-baseline-ratchet
// @capability ratio-ratchet-gates
// @claim host-scoped-baseline-ratchet
// @contract host-scoped-baseline-ratchet
// @category benchmark
// @required_for_production false
// @command cargo test -p arena
// AW-EC-END

// Contract: baseline ratchet tests pass
// Contract: host-scoped baselines remain part of arena's gate semantics
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn arena_host_scoped_baseline_ratchet() {
    let command = "cargo test -p arena";
    let id = "arena-host-scoped-baseline-ratchet";
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
