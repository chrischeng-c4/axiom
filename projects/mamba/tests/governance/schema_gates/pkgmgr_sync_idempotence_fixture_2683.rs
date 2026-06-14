//! Schema gate for the package-manager sync idempotence fixture —
//! closes #2683.
//!
//! Acceptance (issue #2683):
//!
//!   1. Second sync does not change lockfile content.
//!      `[lockfile_assertion].must_be_unchanged_between_runs = true`
//!      + `byte_identical_between_runs = true`.
//!   2. Installed package import still works after both syncs.
//!      `[environment_assertion]` pins `import_ok` outcomes after
//!      both the first and second run.
//!   3. Output states idempotent no-op or equivalent behavior.
//!      `[idempotence_assertion]` lists `allowed_no_op_signals` and
//!      asserts the second run reports zero changed files.
//!
//! Out of scope (per issue body): performance tuning of sync —
//! `[out_of_scope].sync_performance_tuning` pins the exclusion.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("pkgmgr")
        .join("sync")
        .join("manifest.toml")
}

fn profile_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("validation")
        .join("profiles")
        .join("package_manager.toml")
}

#[test]
fn pkgmgr_sync_idempotence_manifest_header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("pkgmgr_sync_idempotence"),
        "`fixture` must be \"pkgmgr_sync_idempotence\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2683),
        "`issue` must record #2683"
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("sync"),
        "`family` must be \"sync\""
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
fn pkgmgr_sync_setup_starts_from_locked_unsynced_project() {
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

    let dep = setup
        .get("locked_dependency")
        .and_then(|v| v.as_str())
        .expect("`[setup].locked_dependency` must be set");
    assert!(!dep.is_empty(), "locked dep must be non-empty");

    assert_eq!(
        setup.get("lockfile_present").and_then(|v| v.as_bool()),
        Some(true),
        "`[setup].lockfile_present` must be true — sync resolves against an existing lockfile"
    );
    assert_eq!(
        setup.get("project_env_present").and_then(|v| v.as_bool()),
        Some(false),
        "`[setup].project_env_present` must be false — first sync creates the env"
    );
}

#[test]
fn pkgmgr_sync_first_run_installs_locked_deps() {
    let doc = crate::common::load_toml(&manifest_path());
    let first = doc
        .get("first_run")
        .and_then(|v| v.as_table())
        .expect("missing `[first_run]` block");

    let command: Vec<&str> = first
        .get("command")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert_eq!(
        command.first().copied(),
        Some("sync"),
        "`[first_run].command[0]` must be `sync`; got {command:?}"
    );

    assert_eq!(
        first.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(0),
        "`[first_run].expected_exit_code` must be 0"
    );
    assert_eq!(
        first.get("must_create_env").and_then(|v| v.as_bool()),
        Some(true),
        "`[first_run].must_create_env` must be true"
    );
    assert_eq!(
        first.get("must_install_locked_dependency").and_then(|v| v.as_bool()),
        Some(true),
        "`[first_run].must_install_locked_dependency` must be true"
    );
}

#[test]
fn pkgmgr_sync_second_run_is_a_no_op() {
    let doc = crate::common::load_toml(&manifest_path());
    let second = doc
        .get("second_run")
        .and_then(|v| v.as_table())
        .expect("missing `[second_run]` block");

    let command: Vec<&str> = second
        .get("command")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert_eq!(
        command.first().copied(),
        Some("sync"),
        "`[second_run].command[0]` must be `sync`; got {command:?}"
    );

    assert_eq!(
        second.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(0),
        "`[second_run].expected_exit_code` must be 0 — idempotent no-op exits clean"
    );
    assert_eq!(
        second.get("must_not_create_env").and_then(|v| v.as_bool()),
        Some(true),
        "`[second_run].must_not_create_env` must be true — env already exists"
    );
    assert_eq!(
        second.get("must_not_reinstall_packages").and_then(|v| v.as_bool()),
        Some(true),
        "`[second_run].must_not_reinstall_packages` must be true — \
         packages are already on disk"
    );
}

#[test]
fn pkgmgr_sync_lockfile_assertion_is_unchanged() {
    let doc = crate::common::load_toml(&manifest_path());
    let lock = doc.get("lockfile_assertion").and_then(|v| v.as_table()).expect(
        "missing `[lockfile_assertion]` block \
         (acceptance: \"Second sync does not change lockfile content.\")",
    );

    assert_eq!(
        lock.get("file").and_then(|v| v.as_str()),
        Some("mamba.lock"),
        "`[lockfile_assertion].file` must be `mamba.lock`"
    );
    assert_eq!(
        lock.get("must_be_unchanged_between_runs").and_then(|v| v.as_bool()),
        Some(true),
        "`[lockfile_assertion].must_be_unchanged_between_runs` must be true"
    );
    assert_eq!(
        lock.get("byte_identical_between_runs").and_then(|v| v.as_bool()),
        Some(true),
        "`[lockfile_assertion].byte_identical_between_runs` must be true"
    );
}

#[test]
fn pkgmgr_sync_environment_assertion_keeps_import_ok() {
    let doc = crate::common::load_toml(&manifest_path());
    let env = doc
        .get("environment_assertion")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[environment_assertion]` block \
             (acceptance: \"Installed package import still works after both syncs.\")",
        );

    let probe = env
        .get("import_probe")
        .and_then(|v| v.as_str())
        .expect("`[environment_assertion].import_probe` must name the module to import");
    let setup_dep = doc
        .get("setup")
        .and_then(|v| v.get("locked_dependency"))
        .and_then(|v| v.as_str())
        .expect("`[setup].locked_dependency` must be set");
    assert_eq!(
        probe, setup_dep,
        "`[environment_assertion].import_probe` must equal the locked dep"
    );

    assert_eq!(
        env.get("expected_import_outcome_after_first_run")
            .and_then(|v| v.as_str()),
        Some("import_ok"),
        "`[environment_assertion].expected_import_outcome_after_first_run` must be \"import_ok\""
    );
    assert_eq!(
        env.get("expected_import_outcome_after_second_run")
            .and_then(|v| v.as_str()),
        Some("import_ok"),
        "`[environment_assertion].expected_import_outcome_after_second_run` must be \"import_ok\""
    );
}

#[test]
fn pkgmgr_sync_idempotence_assertion_reports_no_op() {
    let doc = crate::common::load_toml(&manifest_path());
    let idem = doc.get("idempotence_assertion").and_then(|v| v.as_table()).expect(
        "missing `[idempotence_assertion]` block \
         (acceptance: \"Output states idempotent no-op or equivalent behavior.\")",
    );

    assert_eq!(
        idem.get("second_run_must_report_no_op").and_then(|v| v.as_bool()),
        Some(true),
        "`[idempotence_assertion].second_run_must_report_no_op` must be true"
    );

    let signals: Vec<&str> = idem
        .get("allowed_no_op_signals")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        !signals.is_empty(),
        "`[idempotence_assertion].allowed_no_op_signals` must list at least one signal"
    );
    // Carry at least the two common conventions for "did nothing".
    for required in &["no_op", "unchanged"] {
        assert!(
            signals.contains(required),
            "`[idempotence_assertion].allowed_no_op_signals` must include `{required}`; got {signals:?}"
        );
    }

    assert_eq!(
        idem.get("second_run_changed_file_count_is_zero")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[idempotence_assertion].second_run_changed_file_count_is_zero` must be true"
    );
}

#[test]
fn pkgmgr_sync_isolation_pins_no_global_state() {
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
    ] {
        assert_eq!(
            isolation.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[isolation].{flag}` must be true"
        );
    }
}

#[test]
fn pkgmgr_sync_runner_contract_declares_outcome_keys() {
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
        "lockfile_path",
        "environment_path",
        "first_run_changed_files",
        "second_run_changed_files",
        "exit_code",
        "no_op_signal",
    ] {
        assert!(
            keys.contains(required),
            "`[runner_contract].keys` must include `{required}`; got {keys:?}"
        );
    }
}

#[test]
fn pkgmgr_sync_pins_out_of_scope_per_issue_2683() {
    let doc = crate::common::load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("sync_performance_tuning").and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].sync_performance_tuning` must be true \
         (issue text: \"Out of scope: performance tuning of sync.\")"
    );
}

#[test]
fn pkgmgr_profile_links_to_sync_fixture_directory() {
    let doc = crate::common::load_toml(&profile_path());
    let sync = doc
        .get("families")
        .and_then(|v| v.get("sync"))
        .and_then(|v| v.as_table())
        .expect("validation/profiles/package_manager.toml missing `[families.sync]`");

    let source = sync
        .get("source")
        .and_then(|v| v.as_str())
        .expect("`[families.sync].source` must be set");
    assert_eq!(
        source, "tests/governance/gates/pkgmgr/sync",
        "`[families.sync].source` must point at `tests/governance/gates/pkgmgr/sync`; got {source:?}"
    );

    let kind = sync
        .get("kind")
        .and_then(|v| v.as_str())
        .expect("`[families.sync].kind` must be set");
    assert_eq!(
        kind, "pkgmgr_sync",
        "`[families.sync].kind` must be `pkgmgr_sync` (matches profile family kind)"
    );
}
