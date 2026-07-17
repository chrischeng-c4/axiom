// SPEC-MANAGED: apps/vat/tech-design/logic/vat-microvm-phase-3-vat-compose-limited-compose-subset-up-down-p.md#vat-compose-mainstream-manual-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-compose-mainstream-manual-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim vat-compose-bounded-compose-subset-up-down-ps-logs
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command vat compose import ./docker-compose.yml && vat compose up -d --project demo && vat compose ps demo && vat compose logs demo web && vat compose down demo
// AW-EC-END

// Contract: AC6: a real, unmodified mainstream docker-compose.yml (one image: service, one build: service, a depends_on entry) succeeds through the full import -> up -d -> ps -> logs -> down cycle, retaining an imported registry ready for another up, and the source compose file itself required no edits.
// Contract: Not part of the cargo test / generated aw.toml EC gated surface (no CI-portable fixture repo is bundled for this manual smoke); recorded here as the human verification step named by AC6, run once against a developer-supplied compose file during this WI's own close-out.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_compose_mainstream_manual_smoke() {
    let command =
        "vat compose import ./docker-compose.yml && vat compose up -d --project demo && vat compose ps demo && vat compose logs demo web && vat compose down demo";
    let id = "vat-compose-mainstream-manual-smoke";
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
