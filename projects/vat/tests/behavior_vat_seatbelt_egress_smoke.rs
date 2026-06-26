// SPEC-MANAGED: projects/vat/tech-design/logic/vat-network-sandbox-v3-seatbelt-egress-policy-deny-outbound-exce.md#vat-seatbelt-egress-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-seatbelt-egress-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim vat-seatbelt-egress-smoke
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_sandbox_egress -- --nocapture
// AW-EC-END

// Contract: under sandbox-exec with the localhost-only profile, a process connecting to a localhost listener succeeds while a connect to a non-loopback address is denied. Skips cleanly if sandbox-exec is absent or not macOS.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_seatbelt_egress_smoke() {
    let command = "cargo test -p vat --test vat_sandbox_egress -- --nocapture";
    let id = "vat-seatbelt-egress-smoke";
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
