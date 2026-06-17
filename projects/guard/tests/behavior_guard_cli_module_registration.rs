// SPEC-MANAGED: projects/guard/tech-design/semantic/guard-ec-static-security-smoke.md#guard-cli-module-registration
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec guard-cli-module-registration
// @capability security-policy-profile
// @claim cli-module-registration
// @contract cli-module-registration
// @category behavior
// @required_for_production true
// @command CC=/usr/bin/cc PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin" cargo test -p guard-cli registered_in_slice
// AW-EC-END

// Contract: guard-cli registers through the CLI distributed slice
// Contract: the guard command remains discoverable by the root binary
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn guard_cli_module_registration() {
    let command = "CC=/usr/bin/cc PATH=\"$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin\" cargo test -p guard-cli registered_in_slice";
    let id = "guard-cli-module-registration";
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
