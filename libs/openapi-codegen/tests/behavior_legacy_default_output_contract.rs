// SPEC-MANAGED: libs/openapi-codegen/external-contracts/behavior/multi-language-openapi-client-generation-contract.md#legacy-default-output-contract
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec legacy-default-output-contract
// @capability multi-language-openapi-client-generation
// @claim multi-language-openapi-client-generation-contract
// @contract legacy-default-output-contract
// @category behavior
// @required_for_production true
// @command cargo test -p openapi-codegen --test target_profile_matrix legacy_default_output_contract -- --exact
// AW-EC-END

// Contract: GenOptions target None reproduces fixed file lists and byte fingerprints for legacy TypeScript, Python, and Rust generation.
// Contract: TypeScript, Python, and Rust legacy outputs are each materialized independently and each proves the target sidecar is absent.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn legacy_default_output_contract() {
    let command =
        "cargo test -p openapi-codegen --test target_profile_matrix legacy_default_output_contract -- --exact";
    let id = "legacy-default-output-contract";
    let mut root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    while !root.join(".aw").is_dir() {
        assert!(
            root.pop(),
            "AW EC {id}: no .aw/ project root above {}",
            env!("CARGO_MANIFEST_DIR")
        );
    }
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(&root)
        .output()
        .unwrap_or_else(|e| panic!("AW EC {id}: failed to spawn `{command}`: {e}"));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if output.status.success()
        && aw_ec_cargo_test_executed_count(command, &stdout, &stderr) == Some(0)
    {
        panic!("AW EC {id} FAILED: cargo test command passed but executed 0 tests: {command}\nstdout:\n{stdout}\nstderr:\n{stderr}");
    }
    assert!(
        output.status.success(),
        "AW EC {id} FAILED (exit {:?}): {command}\nstdout:\n{stdout}\nstderr:\n{stderr}",
        output.status.code()
    );
}

fn aw_ec_cargo_test_executed_count(command: &str, stdout: &str, stderr: &str) -> Option<usize> {
    if !command.contains("cargo test") {
        return None;
    }
    let mut total = 0usize;
    let mut saw_count = false;
    for line in stdout.lines().chain(stderr.lines()) {
        let Some(count) = aw_ec_parse_cargo_running_test_count(line) else {
            continue;
        };
        total = total.saturating_add(count);
        saw_count = true;
    }
    saw_count.then_some(total)
}

fn aw_ec_parse_cargo_running_test_count(line: &str) -> Option<usize> {
    let rest = line.trim().strip_prefix("running ")?;
    let number = rest
        .strip_suffix(" tests")
        .or_else(|| rest.strip_suffix(" test"))?;
    number.trim().parse().ok()
}
// CODEGEN-END
