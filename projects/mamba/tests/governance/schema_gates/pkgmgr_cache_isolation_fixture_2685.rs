//! Schema gate for the package-manager cache isolation fixture —
//! closes #2685.
//!
//! Acceptance (issue #2685):
//!
//!   1. Test fails if user home cache is used.
//!      `[user_cache_guard]` lists watched user-home cache paths
//!      and sets `abort_on_user_home_read = true` +
//!      `abort_on_user_home_write = true`.
//!   2. Temp cache cleanup is deterministic.
//!      `[cleanup_assertion]` sets `deterministic = true`,
//!      captures a post-run inventory, and pins an empty
//!      `allowed_residue` list.
//!   3. Summary includes cache root used for debugging.
//!      `[summary].must_include_cache_root = true` (plus byte count
//!      and user-cache-access count).

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("pkgmgr")
        .join("cache")
        .join("manifest.toml")
}

fn profile_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("validation")
        .join("profiles")
        .join("package_manager.toml")
}

#[test]
fn pkgmgr_cache_isolation_manifest_header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("pkgmgr_cache_isolation"),
        "`fixture` must be \"pkgmgr_cache_isolation\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2685),
        "`issue` must record #2685"
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("cache"),
        "`family` must be \"cache\""
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
fn pkgmgr_cache_user_cache_guard_aborts_on_user_home_touch() {
    let doc = crate::common::load_toml(&manifest_path());
    let guard = doc
        .get("user_cache_guard")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[user_cache_guard]` block \
         (acceptance: \"Test fails if user home cache is used.\")",
        );

    assert_eq!(
        guard
            .get("abort_on_user_home_read")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[user_cache_guard].abort_on_user_home_read` must be true"
    );
    assert_eq!(
        guard
            .get("abort_on_user_home_write")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[user_cache_guard].abort_on_user_home_write` must be true"
    );

    let watched: Vec<&str> = guard
        .get("watched_paths")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        !watched.is_empty(),
        "`[user_cache_guard].watched_paths` must list at least one default cache path"
    );
    // Every watched path must live under the user home (`~/`); the
    // guard's job is to catch leaks into the user's home directory.
    for path in &watched {
        assert!(
            path.starts_with("~/"),
            "every entry in `[user_cache_guard].watched_paths` must be a user-home \
             path (`~/...`); got {path:?}"
        );
    }
    // Sanity: the mamba-specific cache directory must be watched.
    assert!(
        watched.iter().any(|p| p.contains("mamba")),
        "`[user_cache_guard].watched_paths` must watch at least one mamba-specific \
         cache path; got {watched:?}"
    );
}

#[test]
fn pkgmgr_cache_temp_cache_lives_under_tempdir() {
    let doc = crate::common::load_toml(&manifest_path());
    let temp = doc
        .get("temp_cache")
        .and_then(|v| v.as_table())
        .expect("missing `[temp_cache]` block");

    let env_var = temp
        .get("cache_root_env_var")
        .and_then(|v| v.as_str())
        .expect("`[temp_cache].cache_root_env_var` must name the env var");
    assert!(!env_var.is_empty(), "cache_root_env_var must be non-empty");

    for flag in &[
        "cache_root_must_be_under_tempdir",
        "must_be_created_before_run",
        "must_be_removed_after_run",
    ] {
        assert_eq!(
            temp.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[temp_cache].{flag}` must be true"
        );
    }
}

#[test]
fn pkgmgr_cache_cleanup_assertion_is_deterministic() {
    let doc = crate::common::load_toml(&manifest_path());
    let cleanup = doc
        .get("cleanup_assertion")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[cleanup_assertion]` block \
         (acceptance: \"Temp cache cleanup is deterministic.\")",
        );

    assert_eq!(
        cleanup.get("deterministic").and_then(|v| v.as_bool()),
        Some(true),
        "`[cleanup_assertion].deterministic` must be true"
    );
    assert_eq!(
        cleanup
            .get("captures_post_run_inventory")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[cleanup_assertion].captures_post_run_inventory` must be true"
    );

    let baseline = cleanup
        .get("inventory_baseline_path")
        .and_then(|v| v.as_str())
        .expect("`[cleanup_assertion].inventory_baseline_path` must be set");
    assert!(
        !baseline.is_empty(),
        "inventory_baseline_path must be non-empty"
    );

    let residue: Vec<&str> = cleanup
        .get("allowed_residue")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_else(|| {
            panic!("`[cleanup_assertion].allowed_residue` must be a (possibly empty) array")
        });
    // Empty list is the right baseline — every leftover file must
    // be intentional and added explicitly.
    assert!(
        residue.is_empty(),
        "`[cleanup_assertion].allowed_residue` should be empty by default; got {residue:?}"
    );
}

#[test]
fn pkgmgr_cache_workflow_populates_temp_not_user() {
    let doc = crate::common::load_toml(&manifest_path());
    let workflow = doc
        .get("workflow")
        .and_then(|v| v.as_table())
        .expect("missing `[workflow]` block");

    let prep: Vec<&str> = workflow
        .get("preparatory_command")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        !prep.is_empty(),
        "`[workflow].preparatory_command` must list a command to populate the cache"
    );

    assert_eq!(
        workflow
            .get("expected_exit_code")
            .and_then(|v| v.as_integer()),
        Some(0),
        "`[workflow].expected_exit_code` must be 0"
    );
    assert_eq!(
        workflow
            .get("must_populate_temp_cache")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[workflow].must_populate_temp_cache` must be true"
    );
    assert_eq!(
        workflow
            .get("must_not_populate_user_cache")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[workflow].must_not_populate_user_cache` must be true"
    );
}

#[test]
fn pkgmgr_cache_summary_names_cache_root() {
    let doc = crate::common::load_toml(&manifest_path());
    let summary = doc.get("summary").and_then(|v| v.as_table()).expect(
        "missing `[summary]` block \
         (acceptance: \"Summary includes cache root used for debugging.\")",
    );

    for flag in &[
        "must_include_cache_root",
        "must_include_cache_byte_count",
        "must_include_user_cache_access_count",
    ] {
        assert_eq!(
            summary.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[summary].{flag}` must be true"
        );
    }
}

#[test]
fn pkgmgr_cache_isolation_pins_no_global_state() {
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
fn pkgmgr_cache_runner_contract_declares_outcome_keys() {
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
        "cache_root",
        "cache_byte_count",
        "user_cache_access_count",
        "workflow_exit_code",
        "cleanup_status",
    ] {
        assert!(
            keys.contains(required),
            "`[runner_contract].keys` must include `{required}`; got {keys:?}"
        );
    }

    let cleanup_statuses: Vec<&str> = contract
        .get("cleanup_status_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        cleanup_statuses.contains(&"clean") && cleanup_statuses.contains(&"residue_detected"),
        "`[runner_contract].cleanup_status_values` must include `clean` and \
         `residue_detected`; got {cleanup_statuses:?}"
    );
}

#[test]
fn pkgmgr_cache_pins_out_of_scope_per_issue_2685() {
    let doc = crate::common::load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("global_cache_implementation_policy")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].global_cache_implementation_policy` must be true \
         (issue text: \"Out of scope: global cache implementation policy.\")"
    );
}

#[test]
fn pkgmgr_profile_links_to_cache_fixture_directory() {
    let doc = crate::common::load_toml(&profile_path());
    let cache = doc
        .get("families")
        .and_then(|v| v.get("cache"))
        .and_then(|v| v.as_table())
        .expect("validation/profiles/package_manager.toml missing `[families.cache]`");

    let source = cache
        .get("source")
        .and_then(|v| v.as_str())
        .expect("`[families.cache].source` must be set");
    assert_eq!(
        source, "tests/governance/gates/pkgmgr/cache",
        "`[families.cache].source` must point at `tests/governance/gates/pkgmgr/cache`; got {source:?}"
    );

    let kind = cache
        .get("kind")
        .and_then(|v| v.as_str())
        .expect("`[families.cache].kind` must be set");
    assert_eq!(
        kind, "pkgmgr_cache",
        "`[families.cache].kind` must be `pkgmgr_cache`"
    );
}
