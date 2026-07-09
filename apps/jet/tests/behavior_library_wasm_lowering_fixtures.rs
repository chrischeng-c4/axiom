// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/3786.md#library-wasm-lowering-fixtures
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec library-wasm-lowering-fixtures
// @capability browser-trace-parity
// @claim library-wasm-lowering-fixtures
// @contract library-wasm-lowering-fixtures
// @category behavior
// @required_for_production true
// @command cargo test -p jet --test tsx_to_rust_imports -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn library_wasm_lowering_fixtures() {
    let command = "cargo test -p jet --test tsx_to_rust_imports -- --nocapture";
    let id = "library-wasm-lowering-fixtures";
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
