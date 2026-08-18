// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-compose-container-gated-full-cycle
// @capability agent-native-gpu-native-dev-containers
// @claim vat-compose-bounded-compose-subset-up-down-ps-logs
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_compose -- --nocapture
// AW-EC-END

// Contract: AC5: gated on a container_available() skip helper (mirroring vat_cluster.rs's Docker-gated pattern and vat_sandbox_microvm.rs's container-gated tests): compose up -d against a fixture with one image: service and one build: service, then compose ps reports starting or ready truthfully, compose logs <project> <service> returns non-empty captured output for each, and compose down terminates the backing runner/service processes while retaining project.json as imported metadata ready for retry.
// Contract: R9: foreground and detached up share one project/token ComposeHandoff; only the token owner publishes the durable VAT id and the parent never performs global VAT-store name/time discovery.
// Contract: R9: down writes .compose-stop-request and waits for the VAT parent to persist terminal runner/service cleanup before resetting project.json. Runner exit while VAT remains Running projects stopping and retains the binding. Current handoff_protocol: 1 VAT load/read/malformed/missing failure is EvidenceUnavailable, which retains the binding and requests retry rather than terminal reset; only protocol-absent historic JSON plus metadata NotFound may recover. A concurrent up is rejected during that window; runner PID evidence is never used as a direct signal target.
// Contract: R9: Docker or MicroVM cleanup_error retains the VAT, project binding, and published-port ownership and forces nonzero lifecycle retention. A later down retries only the persisted runtime resource; a failed rm -f releases only after successful bounded exact-name list proof of absence (Docker anchored name filter/exact line, MicroVM parsed JSON/no id).
// Contract: Registered in the generated apps/vat/aw.toml EC inventory alongside vat_compose_import.rs's pure test, so aw ec gen --verify / aw health --verify-tests pick both up as configured EC-gated test commands for the agent-native-gpu-native-dev-containers capability.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_compose_container_gated_full_cycle() {
    let command = "cargo test -p vat --test vat_compose -- --nocapture";
    let id = "vat-compose-container-gated-full-cycle";
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
