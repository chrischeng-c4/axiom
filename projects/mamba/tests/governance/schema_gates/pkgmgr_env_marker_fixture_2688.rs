//! Schema gate for the package-manager environment marker fixture —
//! closes #2688.
//!
//! Acceptance (issue #2688):
//!
//!   1. Marker result is recorded in lockfile or resolver summary.
//!      `[marker_record_assertion]` sets both
//!      `recorded_in_lockfile` and `recorded_in_resolver_summary`
//!      to true and lists the record keys.
//!   2. Wrong marker inclusion fails the test.
//!      `[wrong_inclusion_guard]` pins failure flags and
//!      `diagnostic_must_name_marker = true`.
//!   3. Current runtime metadata used for marker evaluation is
//!      visible.
//!      `[runtime_visibility]` flags python_version, implementation,
//!      and platform.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("pkgmgr")
        .join("env_marker")
        .join("manifest.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("manifest {} unreadable: {e}", path.display()));
    raw.parse()
        .unwrap_or_else(|e| panic!("{} parse error: {e}", path.display()))
}

#[test]
fn pkgmgr_env_marker_manifest_header_is_well_formed() {
    let doc = load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("pkgmgr_env_marker"),
        "`fixture` must be \"pkgmgr_env_marker\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2688),
        "`issue` must record #2688"
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("env_marker"),
        "`family` must be \"env_marker\""
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
}

#[test]
fn pkgmgr_env_marker_block_pins_python_version_expression() {
    let doc = load_toml(&manifest_path());
    let marker = doc
        .get("marker")
        .and_then(|v| v.as_table())
        .expect("missing `[marker]` block");

    let expr = marker
        .get("expression")
        .and_then(|v| v.as_str())
        .expect("`[marker].expression` must be set");
    assert!(
        expr.contains("python_version"),
        "`[marker].expression` must reference python_version; got {expr:?}"
    );

    assert_eq!(
        marker.get("expected_evaluation").and_then(|v| v.as_bool()),
        Some(true),
        "`[marker].expected_evaluation` must be true — mamba targets 3.12+"
    );
    assert_eq!(
        marker.get("marker_kind").and_then(|v| v.as_str()),
        Some("python_version"),
        "`[marker].marker_kind` must be \"python_version\" — keeps scope tight"
    );
}

#[test]
fn pkgmgr_env_marker_candidates_split_included_vs_excluded() {
    let doc = load_toml(&manifest_path());
    let cand = doc
        .get("candidates")
        .and_then(|v| v.as_table())
        .expect("missing `[candidates]` block");

    let included = cand
        .get("included_dependency")
        .and_then(|v| v.as_str())
        .expect("`[candidates].included_dependency` must be set");
    let excluded = cand
        .get("excluded_dependency")
        .and_then(|v| v.as_str())
        .expect("`[candidates].excluded_dependency` must be set");
    assert_ne!(
        included, excluded,
        "included and excluded deps must differ — gate compares both ways"
    );

    let inc_marker = cand
        .get("included_marker")
        .and_then(|v| v.as_str())
        .expect("`[candidates].included_marker` must be set");
    let exc_marker = cand
        .get("excluded_marker")
        .and_then(|v| v.as_str())
        .expect("`[candidates].excluded_marker` must be set");
    assert!(
        inc_marker.contains(">="),
        "included marker should match the runtime (`>=`); got {inc_marker:?}"
    );
    assert!(
        exc_marker.contains("<"),
        "excluded marker should fail the runtime (`<`); got {exc_marker:?}"
    );
}

#[test]
fn pkgmgr_env_marker_record_assertion_pins_lockfile_and_summary() {
    let doc = load_toml(&manifest_path());
    let rec = doc
        .get("marker_record_assertion")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[marker_record_assertion]` block \
         (acceptance: \"Marker result is recorded in lockfile or resolver summary.\")",
        );

    assert_eq!(
        rec.get("recorded_in_lockfile").and_then(|v| v.as_bool()),
        Some(true),
        "`[marker_record_assertion].recorded_in_lockfile` must be true"
    );
    assert_eq!(
        rec.get("recorded_in_resolver_summary")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[marker_record_assertion].recorded_in_resolver_summary` must be true"
    );

    let keys: Vec<&str> = rec
        .get("record_keys")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &[
        "marker_expression",
        "marker_evaluation",
        "runtime_python_version",
        "runtime_platform",
    ] {
        assert!(
            keys.contains(required),
            "`[marker_record_assertion].record_keys` must include `{required}`; got {keys:?}"
        );
    }
}

#[test]
fn pkgmgr_env_marker_lockfile_includes_modern_excludes_legacy() {
    let doc = load_toml(&manifest_path());
    let lock = doc
        .get("lockfile_assertion")
        .and_then(|v| v.as_table())
        .expect("missing `[lockfile_assertion]` block");

    let included = doc
        .get("candidates")
        .and_then(|v| v.get("included_dependency"))
        .and_then(|v| v.as_str())
        .expect("`[candidates].included_dependency` must be set");
    let excluded = doc
        .get("candidates")
        .and_then(|v| v.get("excluded_dependency"))
        .and_then(|v| v.as_str())
        .expect("`[candidates].excluded_dependency` must be set");

    assert_eq!(
        lock.get("must_contain_dependency").and_then(|v| v.as_str()),
        Some(included),
        "`[lockfile_assertion].must_contain_dependency` must equal the included candidate"
    );
    assert_eq!(
        lock.get("must_not_contain_dependency")
            .and_then(|v| v.as_str()),
        Some(excluded),
        "`[lockfile_assertion].must_not_contain_dependency` must equal the excluded candidate"
    );
}

#[test]
fn pkgmgr_env_marker_wrong_inclusion_guard_fails_loud() {
    let doc = load_toml(&manifest_path());
    let guard = doc
        .get("wrong_inclusion_guard")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[wrong_inclusion_guard]` block \
         (acceptance: \"Wrong marker inclusion fails the test.\")",
        );

    for flag in &[
        "fail_if_excluded_present_in_lockfile",
        "fail_if_excluded_imported",
        "diagnostic_must_name_marker",
    ] {
        assert_eq!(
            guard.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[wrong_inclusion_guard].{flag}` must be true"
        );
    }
}

#[test]
fn pkgmgr_env_marker_runtime_visibility_names_pyver_impl_platform() {
    let doc = load_toml(&manifest_path());
    let vis = doc
        .get("runtime_visibility")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[runtime_visibility]` block \
         (acceptance: \"Current runtime metadata used for marker evaluation is visible.\")",
        );

    for flag in &[
        "summary_must_name_python_version",
        "summary_must_name_implementation",
        "summary_must_name_platform",
    ] {
        assert_eq!(
            vis.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[runtime_visibility].{flag}` must be true"
        );
    }
}

#[test]
fn pkgmgr_env_marker_isolation_pins_no_global_state() {
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
fn pkgmgr_env_marker_runner_contract_declares_outcome_keys() {
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
        "marker_expression",
        "marker_evaluation",
        "runtime_python_version",
        "runtime_implementation",
        "runtime_platform",
        "included_count",
        "excluded_count",
        "exit_code",
    ] {
        assert!(
            keys.contains(required),
            "`[runner_contract].keys` must include `{required}`; got {keys:?}"
        );
    }
}

#[test]
fn pkgmgr_env_marker_pins_out_of_scope_per_issue_2688() {
    let doc = load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("every_pep508_marker_expression")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].every_pep508_marker_expression` must be true \
         (issue text: \"Out of scope: every PEP 508 marker expression.\")"
    );
}
