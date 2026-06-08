//! Schema gate for the package-manager lock fixture — closes #2682.
//!
//! Acceptance (issue #2682):
//!
//!   1. Lockfile contains direct and transitive dependencies.
//!      `[lockfile_assertion].must_contain_dependencies` lists both,
//!      and `must_distinguish_direct_vs_transitive = true`.
//!   2. No package files are installed during lock-only run.
//!      `[install_assertion]` pins site-packages untouched, env
//!      not created, no wheels extracted.
//!   3. Lock failure diagnostics are deterministic.
//!      `[failure_case]` pins a non-zero exit, a named substring,
//!      a `diagnostic_is_deterministic` flag, and a "must not write
//!      partial lockfile" invariant.
//!
//! Out of scope (per issue body): resolver algorithm improvements —
//! `[out_of_scope].resolver_algorithm_improvements` pins exclusion.
//!
//! Cheap test — TOML read + field walks.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("pkgmgr")
        .join("lock")
        .join("manifest.toml")
}

fn profile_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("validation")
        .join("profiles")
        .join("package_manager.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("manifest {} unreadable: {e}", path.display()));
    raw.parse()
        .unwrap_or_else(|e| panic!("{} parse error: {e}", path.display()))
}

#[test]
fn pkgmgr_lock_manifest_header_is_well_formed() {
    let doc = load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("pkgmgr_lock"),
        "`fixture` must be \"pkgmgr_lock\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2682),
        "`issue` must record #2682"
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("lock"),
        "`family` must be \"lock\""
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
fn pkgmgr_lock_setup_carries_direct_and_transitive_deps() {
    let doc = load_toml(&manifest_path());
    let setup = doc
        .get("setup")
        .and_then(|v| v.as_table())
        .expect("missing `[setup]` block");

    assert_eq!(
        setup.get("project_initialized").and_then(|v| v.as_bool()),
        Some(true),
        "`[setup].project_initialized` must be true"
    );

    let direct = setup
        .get("direct_dependency")
        .and_then(|v| v.as_str())
        .expect("`[setup].direct_dependency` must be set");
    assert!(!direct.is_empty(), "direct dep must be non-empty");

    let transitive = setup
        .get("transitive_dependency")
        .and_then(|v| v.as_str())
        .expect("`[setup].transitive_dependency` must be set");
    assert!(
        !transitive.is_empty(),
        "transitive dep must be non-empty (acceptance requires both kinds)"
    );
    assert_ne!(
        direct, transitive,
        "direct and transitive deps must differ — otherwise the fixture can't \
         prove the lockfile distinguishes the two roles"
    );

    assert_eq!(
        setup.get("lockfile_present").and_then(|v| v.as_bool()),
        Some(false),
        "`[setup].lockfile_present` must be false — lock creates the lockfile"
    );
    assert_eq!(
        setup
            .get("site_packages_populated")
            .and_then(|v| v.as_bool()),
        Some(false),
        "`[setup].site_packages_populated` must be false — lock does NOT install"
    );
}

#[test]
fn pkgmgr_lock_action_invokes_lock() {
    let doc = load_toml(&manifest_path());
    let action = doc
        .get("action")
        .and_then(|v| v.as_table())
        .expect("missing `[action]` block");

    let command: Vec<&str> = action
        .get("command")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert_eq!(
        command.first().copied(),
        Some("lock"),
        "`[action].command[0]` must be `lock`; got {command:?}"
    );

    assert_eq!(
        action
            .get("expected_exit_code")
            .and_then(|v| v.as_integer()),
        Some(0),
        "`[action].expected_exit_code` must be 0 — happy path succeeds"
    );
}

#[test]
fn pkgmgr_lock_lockfile_assertion_lists_both_dep_kinds() {
    let doc = load_toml(&manifest_path());
    let lock = doc
        .get("lockfile_assertion")
        .and_then(|v| v.as_table())
        .expect("missing `[lockfile_assertion]` block");

    assert_eq!(
        lock.get("file").and_then(|v| v.as_str()),
        Some("mamba.lock"),
        "`[lockfile_assertion].file` must be `mamba.lock`"
    );
    assert_eq!(
        lock.get("must_exist").and_then(|v| v.as_bool()),
        Some(true),
        "`[lockfile_assertion].must_exist` must be true"
    );

    let needed: Vec<&str> = lock
        .get("must_contain_dependencies")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    let direct = doc
        .get("setup")
        .and_then(|v| v.get("direct_dependency"))
        .and_then(|v| v.as_str())
        .expect("`[setup].direct_dependency` must be set");
    let transitive = doc
        .get("setup")
        .and_then(|v| v.get("transitive_dependency"))
        .and_then(|v| v.as_str())
        .expect("`[setup].transitive_dependency` must be set");
    assert!(
        needed.contains(&direct),
        "`[lockfile_assertion].must_contain_dependencies` must list the direct dep ({direct:?}); got {needed:?}"
    );
    assert!(
        needed.contains(&transitive),
        "`[lockfile_assertion].must_contain_dependencies` must list the transitive dep ({transitive:?}); got {needed:?}"
    );

    assert_eq!(
        lock.get("must_distinguish_direct_vs_transitive")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[lockfile_assertion].must_distinguish_direct_vs_transitive` must be true \
         (acceptance: \"contains direct and transitive\")"
    );

    assert_eq!(
        lock.get("deterministic").and_then(|v| v.as_bool()),
        Some(true),
        "`[lockfile_assertion].deterministic` must be true"
    );
    assert_eq!(
        lock.get("byte_identical_on_replay")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[lockfile_assertion].byte_identical_on_replay` must be true"
    );
}

#[test]
fn pkgmgr_lock_install_assertion_keeps_env_untouched() {
    let doc = load_toml(&manifest_path());
    let install = doc
        .get("install_assertion")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[install_assertion]` block \
         (acceptance: \"No package files are installed during lock-only run.\")",
        );

    for flag in &[
        "site_packages_must_remain_untouched",
        "project_env_must_not_be_created",
        "no_wheels_extracted",
    ] {
        assert_eq!(
            install.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[install_assertion].{flag}` must be true"
        );
    }
}

#[test]
fn pkgmgr_lock_failure_case_diagnostic_is_deterministic() {
    let doc = load_toml(&manifest_path());
    let fail = doc.get("failure_case").and_then(|v| v.as_table()).expect(
        "missing `[failure_case]` block \
         (acceptance: \"Lock failure diagnostics are deterministic.\")",
    );

    let unresolvable = fail
        .get("unresolvable_dependency")
        .and_then(|v| v.as_str())
        .expect("`[failure_case].unresolvable_dependency` must name a missing dep");
    assert!(
        !unresolvable.is_empty(),
        "unresolvable dep must be a non-empty name"
    );

    let exit = fail
        .get("expected_exit_code")
        .and_then(|v| v.as_integer())
        .expect("`[failure_case].expected_exit_code` must be set");
    assert_ne!(exit, 0, "failure case must NOT exit 0; got {exit}");

    let diag = fail
        .get("expected_stderr_contains")
        .and_then(|v| v.as_str())
        .expect("`[failure_case].expected_stderr_contains` must name a substring");
    assert!(!diag.is_empty(), "diagnostic substring must be non-empty");

    assert_eq!(
        fail.get("diagnostic_is_deterministic")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[failure_case].diagnostic_is_deterministic` must be true (acceptance text)"
    );
    assert_eq!(
        fail.get("diagnostic_must_name_failing_dependency")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[failure_case].diagnostic_must_name_failing_dependency` must be true"
    );
    assert_eq!(
        fail.get("must_not_write_partial_lockfile")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[failure_case].must_not_write_partial_lockfile` must be true — \
         a failed lock must not leave half-baked state"
    );
}

#[test]
fn pkgmgr_lock_isolation_pins_no_global_state() {
    let doc = load_toml(&manifest_path());
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
fn pkgmgr_lock_runner_contract_declares_outcome_keys() {
    let doc = load_toml(&manifest_path());
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
        "resolved_count",
        "direct_count",
        "transitive_count",
        "exit_code",
        "diagnostic_stream",
    ] {
        assert!(
            keys.contains(required),
            "`[runner_contract].keys` must include `{required}`; got {keys:?}"
        );
    }

    let outcomes: Vec<&str> = contract
        .get("outcome_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        outcomes.contains(&"pass") && outcomes.contains(&"fail"),
        "`[runner_contract].outcome_values` must carry `pass` and `fail`; got {outcomes:?}"
    );
}

#[test]
fn pkgmgr_lock_pins_out_of_scope_per_issue_2682() {
    let doc = load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("resolver_algorithm_improvements")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].resolver_algorithm_improvements` must be true \
         (issue text: \"Out of scope: resolver algorithm improvements.\")"
    );
}

#[test]
fn pkgmgr_profile_links_to_lock_fixture_directory() {
    let doc = load_toml(&profile_path());
    let lock = doc
        .get("families")
        .and_then(|v| v.get("lock"))
        .and_then(|v| v.as_table())
        .expect("validation/profiles/package_manager.toml missing `[families.lock]`");

    let source = lock
        .get("source")
        .and_then(|v| v.as_str())
        .expect("`[families.lock].source` must be set");
    assert_eq!(
        source, "tests/governance/gates/pkgmgr/lock",
        "`[families.lock].source` must point at `tests/governance/gates/pkgmgr/lock`; got {source:?}"
    );

    let kind = lock
        .get("kind")
        .and_then(|v| v.as_str())
        .expect("`[families.lock].kind` must be set");
    assert_eq!(
        kind, "pkgmgr_lock",
        "`[families.lock].kind` must equal the manifest `fixture` field"
    );
}
