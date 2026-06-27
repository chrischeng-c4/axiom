// SPEC-MANAGED: .aw/tech-design/projects/jet/logic/scss-sass-compilation-in-the-build-lib-css-pipeline.md#scss-sass-compilation
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec scss-sass-compilation
// @capability bundler-production-build
// @claim scss-sass-compilation
// @contract scss-sass-compilation
// @category behavior
// @required_for_production true
// @command cargo test -p jet --lib css::scss
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn scss_sass_compilation() {
    let command = "cargo test -p jet --lib css::scss";
    let id = "scss-sass-compilation";
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
