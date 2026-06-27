// SPEC-MANAGED: .aw/tech-design/projects/jet/interfaces/cli/openapi-client-codegen-types-fetch-client-react-query-hooks.md#stack-aware-openapi-codegen
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec stack-aware-openapi-codegen
// @capability rust-native-frontend-toolchain
// @claim stack-aware-openapi-codegen
// @contract stack-aware-openapi-codegen
// @category behavior
// @required_for_production true
// @command cargo test -p jet --test openapi_golden
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn stack_aware_openapi_codegen() {
    let command = "cargo test -p jet --test openapi_golden";
    let id = "stack-aware-openapi-codegen";
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
