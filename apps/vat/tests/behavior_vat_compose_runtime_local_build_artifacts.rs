// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-compose-runtime-local-build-artifacts
// @capability agent-native-gpu-native-dev-containers
// @claim compose-runtime-local-build-artifacts
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_compose_build -- --nocapture
// AW-EC-END

// Contract: #1529: a build-only service resolves short/full build paths relative to the canonical compose source, not the invoking cwd; explicit Dockerfile and deterministically ordered build.args reach the selected builder; generated tags use an OCI-safe readable project/service prefix plus a BLAKE3 raw-pair identity suffix, so normalization or delimiter ambiguity cannot collide.
// Contract: #1529: auto/native/docker select Docker and MicroVm selects Apple Container; a preflight/build failure occurs before generated vat.toml replacement, preserving a prior materialized import. Image-only compose files remain builder-independent.
// Contract: #1529: a fresh inactive imported compose up refuses a parseable registry/config service-ID-set mismatch before Docker or Apple Container starts, ignoring service-table order; it accepts a user-edited valid vat.toml when its identity set still matches project.json. Bound or active records bypass this gate for VAT-evidence cleanup, and malformed configs retain vat run's existing parse failure; no full config digest blocks compatible local edits.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_compose_runtime_local_build_artifacts() {
    let command = "cargo test -p vat --test vat_compose_build -- --nocapture";
    let id = "vat-compose-runtime-local-build-artifacts";
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
