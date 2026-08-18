// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-compose-import-pure-fixture-shape
// @capability agent-native-gpu-native-dev-containers
// @claim vat-compose-bounded-compose-subset-up-down-ps-logs
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_compose_import -- --nocapture
// AW-EC-END

// Contract: AC2: image-only expansion-shape assertions over a fixture compose file -- one ServiceConfig per compose service, the synthesized project.up runner with requires listing every service id in expand()'s order, environment injected onto ServiceConfig.image_env, and ports mapped per R2's H:C / bare C rules. These image-only cases run with no container/docker binary on PATH.
// Contract: R3: one assertion per hard-reject key (deploy, secrets, configs, extends, networks, profiles, healthcheck, command/entrypoint override, bind-mount-form volumes) asserting compose::parse returns the exact error text naming file/service/key.
// Contract: AC7: feeding a fixture containing deploy: or healthcheck: into vat compose import fails with an error naming the exact file, service, and key.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_compose_import_pure_fixture_shape() {
    let command = "cargo test -p vat --test vat_compose_import -- --nocapture";
    let id = "vat-compose-import-pure-fixture-shape";
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
