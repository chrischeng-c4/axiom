// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/3945.md#full-jet-health-after-unit-repair
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec full-jet-health-after-unit-repair
// @capability browser-trace-parity
// @claim parity-corpus-gates
// @contract full-jet-health-after-unit-repair
// @category behavior
// @required_for_production true
// @command cargo test -p jet-wasm --test renderer_layout -- --nocapture
// AW-EC-END

// Contract: test gate no longer fails on projects/jet/wasm/tests/renderer_layout.rs stale v0 expectations
// Contract: renderer_layout remains a required test gate with zero ignored cases
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn full_jet_health_after_unit_repair() {
    let command = "cargo test -p jet-wasm --test renderer_layout -- --nocapture";
    let id = "full-jet-health-after-unit-repair";
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
