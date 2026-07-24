// SPEC-MANAGED: libs/openapi-codegen/external-contracts/behavior/multi-language-openapi-client-generation-contract.md#deterministic-target-requirements-and-artifacts-contract
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec deterministic-target-requirements-and-artifacts-contract
// @capability multi-language-openapi-client-generation
// @claim multi-language-openapi-client-generation-contract
// @contract deterministic-target-requirements-and-artifacts-contract
// @category behavior
// @required_for_production true
// @command cargo test -p openapi-codegen --test target_profile_matrix all_target_requirements_and_artifacts_are_deterministic -- --exact
// AW-EC-END

// Contract: All seven declared profiles reproduce identical ordered files, file bytes, target requirements, runtime-dependency order, and parsed on-disk manifest values across repeated runs.
// Contract: Every targeted output is materialized twice and the two sidecar files are byte-identical after exact field validation.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn deterministic_target_requirements_and_artifacts_contract() {
    let command =
        "cargo test -p openapi-codegen --test target_profile_matrix all_target_requirements_and_artifacts_are_deterministic -- --exact";
    let id = "deterministic-target-requirements-and-artifacts-contract";
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
