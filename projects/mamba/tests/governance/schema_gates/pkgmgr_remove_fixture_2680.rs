//! Schema gate for the package-manager remove fixture — closes #2680.
//!
//! Acceptance (issue #2680):
//!
//!   1. Removed dependency no longer appears in project metadata.
//!      `[metadata_assertion]` names the project file and the
//!      dependency that must not appear post-remove.
//!   2. Lockfile is updated deterministically.
//!      `[lockfile_assertion]` pins the lockfile path, asserts the
//!      removed dependency is gone, and locks
//!      `byte_identical_on_replay = true`.
//!   3. Import of the removed package fails in the project
//!      environment.
//!      `[environment_assertion]` declares the import probe and the
//!      expected ModuleNotFoundError-class outcome.
//!
//! Out of scope (per issue body): garbage collection of shared
//! global caches — `[out_of_scope].global_cache_garbage_collection`
//! pins that exclusion.
//!
//! Cheap test — TOML read + field walks. Stays in the default
//! `cargo test -p mamba` set.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("pkgmgr")
        .join("remove")
        .join("manifest.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("manifest {} unreadable: {e}", path.display()));
    raw.parse()
        .unwrap_or_else(|e| panic!("{} parse error: {e}", path.display()))
}

#[test]
fn pkgmgr_remove_manifest_header_is_well_formed() {
    let doc = load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("pkgmgr_remove"),
        "manifest.toml `fixture` must be \"pkgmgr_remove\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2680),
        "manifest.toml `issue` must record #2680 as the owning issue"
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("remove"),
        "manifest.toml `family` must be \"remove\""
    );
    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("package_manager"),
        "manifest.toml `profile` must be \"package_manager\""
    );
    assert_eq!(
        doc.get("network").and_then(|v| v.as_str()),
        Some("offline"),
        "manifest.toml `network` must be \"offline\""
    );
    assert_eq!(
        doc.get("index_source").and_then(|v| v.as_str()),
        Some("frozen_local"),
        "manifest.toml `index_source` must be \"frozen_local\""
    );
}

#[test]
fn pkgmgr_remove_setup_pins_starting_state() {
    let doc = load_toml(&manifest_path());

    let setup = doc
        .get("setup")
        .and_then(|v| v.as_table())
        .expect("manifest.toml missing `[setup]` block");

    assert_eq!(
        setup.get("project_initialized").and_then(|v| v.as_bool()),
        Some(true),
        "`[setup].project_initialized` must be true — remove operates on an init'd project"
    );

    let dep = setup
        .get("dependency_added")
        .and_then(|v| v.as_str())
        .expect("`[setup].dependency_added` must name the dependency to remove");
    assert!(
        !dep.is_empty(),
        "`[setup].dependency_added` must be non-empty"
    );

    assert!(
        setup
            .get("lockfile_present")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        "`[setup].lockfile_present` must be true — remove updates an existing lockfile"
    );

    assert!(
        setup
            .get("environment_synced")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        "`[setup].environment_synced` must be true — remove must affect a synced env"
    );
}

#[test]
fn pkgmgr_remove_action_pins_the_command() {
    let doc = load_toml(&manifest_path());

    let action = doc
        .get("action")
        .and_then(|v| v.as_table())
        .expect("manifest.toml missing `[action]` block");

    let command: Vec<&str> = action
        .get("command")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert_eq!(
        command.first().copied(),
        Some("remove"),
        "`[action].command[0]` must be `remove`; got {command:?}"
    );
    assert!(
        command.len() >= 2,
        "`[action].command` must include the target dependency; got {command:?}"
    );

    let setup_dep = doc
        .get("setup")
        .and_then(|v| v.get("dependency_added"))
        .and_then(|v| v.as_str())
        .expect("`[setup].dependency_added` must be set");
    assert_eq!(
        command.get(1).copied(),
        Some(setup_dep),
        "`[action].command[1]` must match `[setup].dependency_added`; got {command:?}"
    );

    let exit_code = action
        .get("expected_exit_code")
        .and_then(|v| v.as_integer())
        .expect("`[action].expected_exit_code` must be set");
    assert_eq!(
        exit_code, 0,
        "`[action].expected_exit_code` must be 0 — remove succeeds on a present dep"
    );
}

#[test]
fn pkgmgr_remove_metadata_assertion_drops_dependency() {
    let doc = load_toml(&manifest_path());

    let meta = doc
        .get("metadata_assertion")
        .and_then(|v| v.as_table())
        .expect(
            "manifest.toml missing `[metadata_assertion]` block \
             (acceptance: \"Removed dependency no longer appears in project metadata.\")",
        );

    assert_eq!(
        meta.get("file").and_then(|v| v.as_str()),
        Some("mamba.toml"),
        "`[metadata_assertion].file` must be `mamba.toml`"
    );

    let dropped = meta
        .get("must_not_contain_dependency")
        .and_then(|v| v.as_str())
        .expect("`[metadata_assertion].must_not_contain_dependency` must name the removed dep");
    let setup_dep = doc
        .get("setup")
        .and_then(|v| v.get("dependency_added"))
        .and_then(|v| v.as_str())
        .expect("`[setup].dependency_added` must be set");
    assert_eq!(
        dropped, setup_dep,
        "`[metadata_assertion].must_not_contain_dependency` must equal the setup dep"
    );

    assert_eq!(
        meta.get("must_preserve_other_deps")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[metadata_assertion].must_preserve_other_deps` must be true — remove targets one dep"
    );

    let preserve_fields: Vec<&str> = meta
        .get("must_preserve_fields")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &["project_name", "project_version", "python_requires"] {
        assert!(
            preserve_fields.contains(required),
            "`[metadata_assertion].must_preserve_fields` must include `{required}`; got {preserve_fields:?}"
        );
    }
}

#[test]
fn pkgmgr_remove_lockfile_assertion_is_deterministic() {
    let doc = load_toml(&manifest_path());

    let lock = doc
        .get("lockfile_assertion")
        .and_then(|v| v.as_table())
        .expect(
            "manifest.toml missing `[lockfile_assertion]` block \
             (acceptance: \"Lockfile is updated deterministically.\")",
        );

    assert_eq!(
        lock.get("file").and_then(|v| v.as_str()),
        Some("mamba.lock"),
        "`[lockfile_assertion].file` must be `mamba.lock`"
    );
    assert_eq!(
        lock.get("must_exist").and_then(|v| v.as_bool()),
        Some(true),
        "`[lockfile_assertion].must_exist` must be true — remove writes a new lockfile"
    );
    assert_eq!(
        lock.get("deterministic").and_then(|v| v.as_bool()),
        Some(true),
        "`[lockfile_assertion].deterministic` must be true (acceptance text)"
    );
    assert_eq!(
        lock.get("byte_identical_on_replay")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[lockfile_assertion].byte_identical_on_replay` must be true — \
         determinism implies the replay produces the same bytes"
    );

    let dropped = lock
        .get("must_not_contain_dependency")
        .and_then(|v| v.as_str())
        .expect("`[lockfile_assertion].must_not_contain_dependency` must be set");
    let setup_dep = doc
        .get("setup")
        .and_then(|v| v.get("dependency_added"))
        .and_then(|v| v.as_str())
        .expect("`[setup].dependency_added` must be set");
    assert_eq!(
        dropped, setup_dep,
        "`[lockfile_assertion].must_not_contain_dependency` must equal the setup dep"
    );
}

#[test]
fn pkgmgr_remove_environment_assertion_blocks_import() {
    let doc = load_toml(&manifest_path());

    let env = doc
        .get("environment_assertion")
        .and_then(|v| v.as_table())
        .expect(
            "manifest.toml missing `[environment_assertion]` block \
             (acceptance: \"Import of the removed package fails in the project environment.\")",
        );

    let probe = env
        .get("import_probe")
        .and_then(|v| v.as_str())
        .expect("`[environment_assertion].import_probe` must name the module to import");
    let setup_dep = doc
        .get("setup")
        .and_then(|v| v.get("dependency_added"))
        .and_then(|v| v.as_str())
        .expect("`[setup].dependency_added` must be set");
    assert_eq!(
        probe, setup_dep,
        "`[environment_assertion].import_probe` must equal the setup dep"
    );

    assert_eq!(
        env.get("expected_import_outcome").and_then(|v| v.as_str()),
        Some("module_not_found"),
        "`[environment_assertion].expected_import_outcome` must be \"module_not_found\""
    );

    assert_eq!(
        env.get("project_env_must_exist").and_then(|v| v.as_bool()),
        Some(true),
        "`[environment_assertion].project_env_must_exist` must be true — remove does not delete the env"
    );
}

#[test]
fn pkgmgr_remove_reentry_is_idempotent_no_op() {
    let doc = load_toml(&manifest_path());

    let reentry = doc
        .get("reentry")
        .and_then(|v| v.as_table())
        .expect("manifest.toml missing `[reentry]` block");

    let policy = reentry
        .get("policy")
        .and_then(|v| v.as_str())
        .expect("`[reentry].policy` must be set");
    assert_eq!(
        policy, "idempotent_no_op",
        "`[reentry].policy` must be \"idempotent_no_op\" — \
         remove of an absent dep is a clean no-op"
    );

    let exit_code = reentry
        .get("expected_exit_code")
        .and_then(|v| v.as_integer())
        .expect("`[reentry].expected_exit_code` must be set");
    assert_eq!(
        exit_code, 0,
        "`[reentry].expected_exit_code` must be 0 — idempotent no-op succeeds"
    );
}

#[test]
fn pkgmgr_remove_isolation_pins_no_global_state() {
    let doc = load_toml(&manifest_path());

    let isolation = doc
        .get("isolation")
        .and_then(|v| v.as_table())
        .expect("manifest.toml missing `[isolation]` block");

    for flag in &[
        "forbid_writes_outside_project",
        "forbid_user_home_reads",
        "forbid_global_cache_reads",
        "forbid_global_cache_writes",
    ] {
        let value = isolation.get(*flag).and_then(|v| v.as_bool());
        assert_eq!(value, Some(true), "`[isolation].{flag}` must be `true`");
    }
}

#[test]
fn pkgmgr_remove_runner_contract_declares_outcome_keys() {
    let doc = load_toml(&manifest_path());
    let contract = doc
        .get("runner_contract")
        .and_then(|v| v.as_table())
        .expect("manifest.toml missing `[runner_contract]` block");

    let keys: Vec<&str> = contract
        .get("keys")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &[
        "outcome",
        "project_path",
        "removed_dependency",
        "lockfile_path",
        "environment_path",
        "exit_code",
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
fn pkgmgr_remove_pins_out_of_scope_per_issue_2680() {
    let doc = load_toml(&manifest_path());
    let oos = doc.get("out_of_scope").and_then(|v| v.as_table()).expect(
        "manifest.toml missing `[out_of_scope]` block — issue #2680 \
             explicitly excludes global cache GC",
    );
    assert_eq!(
        oos.get("global_cache_garbage_collection")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].global_cache_garbage_collection` must be true \
         (issue text: \"Out of scope: garbage collection of shared global caches.\")"
    );
}
