//! Schema gate for the package-manager add fixture — closes #2681.
//!
//! Acceptance (issue #2681):
//!
//!   1. Add records the requested dependency deterministically.
//!      `[metadata_assertion]` + `[lockfile_assertion]` both set
//!      `deterministic = true` and `byte_identical_on_replay = true`.
//!   2. Missing package fails with a clear diagnostic.
//!      `[missing_package_case]` pins a non-zero exit, a stderr
//!      substring, and a no-mutation invariant.
//!   3. Test runs without PyPI or external network.
//!      `network = "offline"` + `index_source = "frozen_local"`.
//!
//! Out of scope (per issue body): complex version selection —
//! `[out_of_scope].complex_version_selection` pins that exclusion.
//!
//! Cheap test — TOML read + field walks.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("pkgmgr")
        .join("add")
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
fn pkgmgr_add_manifest_header_is_well_formed() {
    let doc = load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("pkgmgr_add"),
        "`fixture` must be \"pkgmgr_add\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2681),
        "`issue` must record #2681"
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("add"),
        "`family` must be \"add\" (matches the package_manager profile)"
    );
    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("package_manager"),
        "`profile` must be \"package_manager\""
    );
    assert_eq!(
        doc.get("network").and_then(|v| v.as_str()),
        Some("offline"),
        "`network` must be \"offline\" (acceptance: no external network)"
    );
    assert_eq!(
        doc.get("index_source").and_then(|v| v.as_str()),
        Some("frozen_local"),
        "`index_source` must be \"frozen_local\""
    );
}

#[test]
fn pkgmgr_add_setup_starts_from_empty_project() {
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

    let deps: Vec<&str> = setup
        .get("dependencies_present")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        deps.is_empty(),
        "`[setup].dependencies_present` must be empty — add starts from no deps; got {deps:?}"
    );

    assert_eq!(
        setup.get("lockfile_present").and_then(|v| v.as_bool()),
        Some(false),
        "`[setup].lockfile_present` must be false — add creates the lockfile"
    );
}

#[test]
fn pkgmgr_add_action_records_dep_and_version() {
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
        Some("add"),
        "`[action].command[0]` must be `add`; got {command:?}"
    );
    assert!(
        command.len() >= 2,
        "`[action].command` must include the target spec; got {command:?}"
    );

    let dep = action
        .get("dependency")
        .and_then(|v| v.as_str())
        .expect("`[action].dependency` must name the dep");
    let version = action
        .get("version")
        .and_then(|v| v.as_str())
        .expect("`[action].version` must name the version");
    let spec = command[1];
    assert!(
        spec.starts_with(dep),
        "`[action].command[1]` ({spec:?}) must start with `[action].dependency` ({dep:?})"
    );
    assert!(
        spec.contains(version),
        "`[action].command[1]` ({spec:?}) must contain `[action].version` ({version:?})"
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
fn pkgmgr_add_metadata_assertion_is_deterministic() {
    let doc = load_toml(&manifest_path());
    let meta = doc
        .get("metadata_assertion")
        .and_then(|v| v.as_table())
        .expect("missing `[metadata_assertion]` block");

    assert_eq!(
        meta.get("file").and_then(|v| v.as_str()),
        Some("mamba.toml"),
        "`[metadata_assertion].file` must be `mamba.toml`"
    );

    let added = meta
        .get("must_contain_dependency")
        .and_then(|v| v.as_str())
        .expect("`[metadata_assertion].must_contain_dependency` must be set");
    let action_dep = doc
        .get("action")
        .and_then(|v| v.get("dependency"))
        .and_then(|v| v.as_str())
        .expect("`[action].dependency` must be set");
    assert_eq!(
        added, action_dep,
        "`[metadata_assertion].must_contain_dependency` must match `[action].dependency`"
    );

    let recorded = meta
        .get("must_record_version")
        .and_then(|v| v.as_str())
        .expect("`[metadata_assertion].must_record_version` must be set");
    let action_version = doc
        .get("action")
        .and_then(|v| v.get("version"))
        .and_then(|v| v.as_str())
        .expect("`[action].version` must be set");
    assert_eq!(
        recorded, action_version,
        "`[metadata_assertion].must_record_version` must match `[action].version`"
    );

    assert_eq!(
        meta.get("deterministic").and_then(|v| v.as_bool()),
        Some(true),
        "`[metadata_assertion].deterministic` must be true (acceptance)"
    );
    assert_eq!(
        meta.get("byte_identical_on_replay")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[metadata_assertion].byte_identical_on_replay` must be true"
    );
}

#[test]
fn pkgmgr_add_lockfile_assertion_pins_version() {
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
        lock.get("must_exist_after_add").and_then(|v| v.as_bool()),
        Some(true),
        "`[lockfile_assertion].must_exist_after_add` must be true"
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

    let pinned = lock
        .get("must_pin_version")
        .and_then(|v| v.as_str())
        .expect("`[lockfile_assertion].must_pin_version` must be set");
    let action_version = doc
        .get("action")
        .and_then(|v| v.get("version"))
        .and_then(|v| v.as_str())
        .expect("`[action].version` must be set");
    assert_eq!(
        pinned, action_version,
        "`[lockfile_assertion].must_pin_version` must match `[action].version`"
    );
}

#[test]
fn pkgmgr_add_missing_package_case_fails_cleanly() {
    let doc = load_toml(&manifest_path());
    let miss = doc
        .get("missing_package_case")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[missing_package_case]` block \
         (acceptance: \"Missing package fails with a clear diagnostic.\")",
        );

    let exit = miss
        .get("expected_exit_code")
        .and_then(|v| v.as_integer())
        .expect("`[missing_package_case].expected_exit_code` must be set");
    assert_ne!(exit, 0, "missing package must NOT exit 0; got {exit}");

    let diag = miss
        .get("expected_stderr_contains")
        .and_then(|v| v.as_str())
        .expect("`[missing_package_case].expected_stderr_contains` must name a substring");
    assert!(
        !diag.is_empty(),
        "diagnostic substring must be non-empty (acceptance: \"clear diagnostic\")"
    );

    assert_eq!(
        miss.get("must_not_mutate_metadata")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[missing_package_case].must_not_mutate_metadata` must be true — \
         a failed add must not partially touch mamba.toml"
    );
    assert_eq!(
        miss.get("must_not_mutate_lockfile")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[missing_package_case].must_not_mutate_lockfile` must be true — \
         a failed add must not write a half-baked lockfile"
    );
}

#[test]
fn pkgmgr_add_isolation_pins_no_global_state() {
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
fn pkgmgr_add_runner_contract_declares_outcome_keys() {
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
        "added_dependency",
        "added_version",
        "lockfile_path",
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
fn pkgmgr_add_pins_out_of_scope_per_issue_2681() {
    let doc = load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block — issue #2681 excludes complex version selection");
    assert_eq!(
        oos.get("complex_version_selection")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].complex_version_selection` must be true (issue text)"
    );
}

#[test]
fn pkgmgr_profile_links_to_add_fixture_directory() {
    let doc = load_toml(&profile_path());
    let add = doc
        .get("families")
        .and_then(|v| v.get("add"))
        .and_then(|v| v.as_table())
        .expect("validation/profiles/package_manager.toml missing `[families.add]`");

    let source = add
        .get("source")
        .and_then(|v| v.as_str())
        .expect("`[families.add].source` must be set");
    assert_eq!(
        source, "tests/governance/gates/pkgmgr/add",
        "`[families.add].source` must point at `tests/governance/gates/pkgmgr/add`; got {source:?}"
    );

    let kind = add
        .get("kind")
        .and_then(|v| v.as_str())
        .expect("`[families.add].kind` must be set");
    assert_eq!(
        kind, "pkgmgr_add",
        "`[families.add].kind` must equal the manifest `fixture` field (`pkgmgr_add`)"
    );
}
