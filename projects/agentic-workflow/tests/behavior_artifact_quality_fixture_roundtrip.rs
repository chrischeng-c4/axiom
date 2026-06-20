// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/specs/aw-artifact-quality-profile.md#artifact-quality-fixture-roundtrip
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec artifact-quality-fixture-roundtrip
// @capability unmapped
// @claim artifact-quality-fixture-roundtrip
// @contract artifact-quality-fixture-roundtrip
// @category behavior
// @required_for_production true
// @command cargo test -p agentic-workflow artifact_quality -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn artifact_quality_fixture_roundtrip() {
    let command = "cargo test -p agentic-workflow artifact_quality -- --nocapture";
    let id = "artifact-quality-fixture-roundtrip";
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
