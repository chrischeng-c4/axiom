//! Schema gate for the package-manager run fixture — closes #2684.
//!
//! Acceptance (issue #2684):
//!
//!   1. Running without sync or install fails with a clear diagnostic.
//!      `[unsynced_failure_case]` pins non-zero exit, named stderr
//!      substring, and a "no sentinel / no env" invariant.
//!   2. Running after sync prints the expected sentinel.
//!      `[synced_success_case]` runs `sync` then `run` and checks
//!      stdout contains `[setup].expected_stdout_sentinel`.
//!   3. No global PATH or user environment mutation is required.
//!      `[isolation].forbid_global_path_mutation` +
//!      `[isolation].forbid_user_shell_env_mutation` +
//!      `[no_global_state_assertion]` pin the no-touch invariant.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("pkgmgr")
        .join("run")
        .join("manifest.toml")
}

fn profile_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("validation")
        .join("profiles")
        .join("package_manager.toml")
}

#[test]
fn pkgmgr_run_manifest_header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("pkgmgr_run"),
        "`fixture` must be \"pkgmgr_run\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2684),
        "`issue` must record #2684"
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("run"),
        "`family` must be \"run\""
    );
    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("package_manager"),
        "`profile` must be \"package_manager\""
    );
    assert_eq!(
        doc.get("network").and_then(|v| v.as_str()),
        Some("offline"),
        "`network` must be \"offline\""
    );
    assert_eq!(
        doc.get("index_source").and_then(|v| v.as_str()),
        Some("frozen_local"),
        "`index_source` must be \"frozen_local\""
    );
}

#[test]
fn pkgmgr_run_setup_names_script_and_sentinel() {
    let doc = crate::common::load_toml(&manifest_path());
    let setup = doc
        .get("setup")
        .and_then(|v| v.as_table())
        .expect("missing `[setup]` block");

    assert_eq!(
        setup.get("project_initialized").and_then(|v| v.as_bool()),
        Some(true),
        "`[setup].project_initialized` must be true"
    );
    assert_eq!(
        setup.get("lockfile_present").and_then(|v| v.as_bool()),
        Some(true),
        "`[setup].lockfile_present` must be true — lockfile pins what `sync` installs"
    );
    assert_eq!(
        setup
            .get("project_env_present_initial")
            .and_then(|v| v.as_bool()),
        Some(false),
        "`[setup].project_env_present_initial` must be false — phase 1 has no env yet"
    );

    let script = setup
        .get("script_path")
        .and_then(|v| v.as_str())
        .expect("`[setup].script_path` must name the script");
    assert!(!script.is_empty(), "script path must be non-empty");

    let sentinel = setup
        .get("expected_stdout_sentinel")
        .and_then(|v| v.as_str())
        .expect("`[setup].expected_stdout_sentinel` must name the sentinel string");
    assert!(!sentinel.is_empty(), "sentinel string must be non-empty");
}

#[test]
fn pkgmgr_run_unsynced_failure_case_fails_cleanly() {
    let doc = crate::common::load_toml(&manifest_path());
    let phase = doc
        .get("unsynced_failure_case")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[unsynced_failure_case]` block \
         (acceptance: \"Running without sync or install fails with a clear diagnostic.\")",
        );

    let command: Vec<&str> = phase
        .get("command")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert_eq!(
        command.first().copied(),
        Some("run"),
        "`[unsynced_failure_case].command[0]` must be `run`; got {command:?}"
    );

    let exit = phase
        .get("expected_exit_code")
        .and_then(|v| v.as_integer())
        .expect("`[unsynced_failure_case].expected_exit_code` must be set");
    assert_ne!(exit, 0, "unsynced run must NOT exit 0; got {exit}");

    let diag = phase
        .get("expected_stderr_contains")
        .and_then(|v| v.as_str())
        .expect("`[unsynced_failure_case].expected_stderr_contains` must name a substring");
    assert!(!diag.is_empty(), "diagnostic substring must be non-empty");

    assert_eq!(
        phase
            .get("must_not_print_sentinel")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[unsynced_failure_case].must_not_print_sentinel` must be true — \
         the script never gets to run"
    );
    assert_eq!(
        phase.get("must_not_create_env").and_then(|v| v.as_bool()),
        Some(true),
        "`[unsynced_failure_case].must_not_create_env` must be true — \
         `run` does not implicitly sync"
    );
}

#[test]
fn pkgmgr_run_synced_success_case_prints_sentinel() {
    let doc = crate::common::load_toml(&manifest_path());
    let phase = doc
        .get("synced_success_case")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[synced_success_case]` block \
         (acceptance: \"Running after sync prints the expected sentinel.\")",
        );

    let prep: Vec<&str> = phase
        .get("preparatory_command")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert_eq!(
        prep,
        vec!["sync"],
        "`[synced_success_case].preparatory_command` must be [\"sync\"]; got {prep:?}"
    );

    let command: Vec<&str> = phase
        .get("command")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert_eq!(
        command.first().copied(),
        Some("run"),
        "`[synced_success_case].command[0]` must be `run`; got {command:?}"
    );

    assert_eq!(
        phase.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(0),
        "`[synced_success_case].expected_exit_code` must be 0"
    );

    let stdout_needle = phase
        .get("expected_stdout_contains")
        .and_then(|v| v.as_str())
        .expect("`[synced_success_case].expected_stdout_contains` must name the sentinel needle");
    let setup_sentinel = doc
        .get("setup")
        .and_then(|v| v.get("expected_stdout_sentinel"))
        .and_then(|v| v.as_str())
        .expect("`[setup].expected_stdout_sentinel` must be set");
    assert_eq!(
        stdout_needle, setup_sentinel,
        "`[synced_success_case].expected_stdout_contains` must equal the setup sentinel"
    );

    assert_eq!(
        phase.get("must_use_project_env").and_then(|v| v.as_bool()),
        Some(true),
        "`[synced_success_case].must_use_project_env` must be true"
    );
    assert_eq!(
        phase
            .get("must_resolve_dependency_from_project_env")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[synced_success_case].must_resolve_dependency_from_project_env` must be true"
    );
}

#[test]
fn pkgmgr_run_no_global_state_assertion_pins_user_env() {
    let doc = crate::common::load_toml(&manifest_path());
    let no_global = doc
        .get("no_global_state_assertion")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[no_global_state_assertion]` block \
         (acceptance: \"No global PATH or user environment mutation is required.\")",
        );

    for flag in &[
        "user_path_must_be_unchanged",
        "user_shell_env_must_be_unchanged",
        "no_system_site_packages_used",
    ] {
        assert_eq!(
            no_global.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[no_global_state_assertion].{flag}` must be true"
        );
    }
}

#[test]
fn pkgmgr_run_isolation_pins_global_path_and_shell_env() {
    let doc = crate::common::load_toml(&manifest_path());
    let isolation = doc
        .get("isolation")
        .and_then(|v| v.as_table())
        .expect("missing `[isolation]` block");

    for flag in &[
        "forbid_writes_outside_project",
        "forbid_user_home_reads",
        "forbid_global_cache_reads",
        "forbid_global_cache_writes",
        "forbid_global_path_mutation",
        "forbid_user_shell_env_mutation",
    ] {
        assert_eq!(
            isolation.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[isolation].{flag}` must be true"
        );
    }
}

#[test]
fn pkgmgr_run_runner_contract_declares_phase_keys() {
    let doc = crate::common::load_toml(&manifest_path());
    let contract = doc
        .get("runner_contract")
        .and_then(|v| v.as_table())
        .expect("missing `[runner_contract]` block");

    let keys: Vec<&str> = contract
        .get("keys")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &[
        "outcome",
        "project_path",
        "script_path",
        "environment_path",
        "stdout_sentinel",
        "exit_code",
        "phase",
    ] {
        assert!(
            keys.contains(required),
            "`[runner_contract].keys` must include `{required}`; got {keys:?}"
        );
    }

    let phases: Vec<&str> = contract
        .get("phase_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        phases.contains(&"unsynced") && phases.contains(&"synced"),
        "`[runner_contract].phase_values` must carry `unsynced` and `synced`; got {phases:?}"
    );
}

#[test]
fn pkgmgr_run_pins_out_of_scope_per_issue_2684() {
    let doc = crate::common::load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("shell_aliases").and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].shell_aliases` must be true (issue text)"
    );
    assert_eq!(
        oos.get("interactive_commands").and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].interactive_commands` must be true (issue text)"
    );
}

#[test]
fn pkgmgr_profile_links_to_run_fixture_directory() {
    let doc = crate::common::load_toml(&profile_path());
    let run = doc
        .get("families")
        .and_then(|v| v.get("run"))
        .and_then(|v| v.as_table())
        .expect("validation/profiles/package_manager.toml missing `[families.run]`");

    let source = run
        .get("source")
        .and_then(|v| v.as_str())
        .expect("`[families.run].source` must be set");
    assert_eq!(
        source, "tests/governance/gates/pkgmgr/run",
        "`[families.run].source` must point at `tests/governance/gates/pkgmgr/run`; got {source:?}"
    );

    let kind = run
        .get("kind")
        .and_then(|v| v.as_str())
        .expect("`[families.run].kind` must be set");
    assert_eq!(
        kind, "pkgmgr_run",
        "`[families.run].kind` must be `pkgmgr_run`"
    );
}
