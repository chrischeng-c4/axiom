// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/3784.md#product-flow-e2e-review
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec product-flow-e2e-review
// @capability native-test-product-flow-e2e
// @claim product-flow-e2e-review
// @contract product-flow-e2e-review
// @category behavior
// @required_for_production true
// @command cargo test -p jet --lib e2e -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn product_flow_e2e_review() {
    let command = "cargo test -p jet --lib e2e -- --nocapture";
    let id = "product-flow-e2e-review";
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
