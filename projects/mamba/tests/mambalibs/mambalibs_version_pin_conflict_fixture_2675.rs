//! Schema gate for the mambalibs version pin conflict fixture —
//! closes #2675.
//!
//! Acceptance (issue #2675):
//!
//!   1. Conflict fails deterministically before import.
//!      `[conflict_case]` pins fail outcome + non-zero exit code +
//!      `must_fail_before_import = true`; `[crash_guard]` pins
//!      `must_not_attempt_import_on_conflict = true`.
//!   2. Error does not mutate lockfile into a partial success state.
//!      `[lockfile_integrity]` pins `must_remain_unchanged`,
//!      `must_not_record_partial_resolution`, and that
//!      `hash_compare_field_keys` are recorded as runner contract
//!      keys.
//!   3. Positive single-version fixture still passes.
//!      `[isolation_from_positive_cases]` pins the cross-fixture
//!      invariant against #2666.
//!
//! Cheap test — single TOML read + field walk. Runs in well under
//! a second; stays in the default `cargo test -p mamba` set.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("mambalibs")
        .join("fixtures")
        .join("version_pin_conflict")
        .join("manifest.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("manifest {} unreadable: {e}", path.display()));
    raw.parse()
        .unwrap_or_else(|e| panic!("{} parse error: {e}", path.display()))
}

#[test]
fn mambalibs_version_pin_conflict_manifest_header_is_well_formed() {
    let doc = load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("mambalibs_version_pin_conflict"),
        "`fixture` must be \"mambalibs_version_pin_conflict\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2675),
        "`issue` must record #2675"
    );
    assert_eq!(
        doc.get("parent_issue").and_then(|v| v.as_integer()),
        Some(2531),
        "`parent_issue` must record #2531"
    );
    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("mambalibs"),
        "`profile` must be \"mambalibs\""
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("version_pin_conflict"),
        "`family` must be \"version_pin_conflict\""
    );
    assert_eq!(
        doc.get("network").and_then(|v| v.as_str()),
        Some("offline"),
        "`network` must be \"offline\""
    );
}

#[test]
fn mambalibs_version_pin_conflict_dependency_names_both_incompatible_versions() {
    let doc = load_toml(&manifest_path());
    let dep = doc
        .get("dependency")
        .and_then(|v| v.as_table())
        .expect("missing `[dependency]` block");

    let name = dep
        .get("name")
        .and_then(|v| v.as_str())
        .expect("`[dependency].name` must be set");
    assert!(!name.is_empty(), "`[dependency].name` must be non-empty");

    let va = dep
        .get("version_a")
        .and_then(|v| v.as_str())
        .expect("`[dependency].version_a` must be set");
    let vb = dep
        .get("version_b")
        .and_then(|v| v.as_str())
        .expect("`[dependency].version_b` must be set");
    assert_ne!(
        va, vb,
        "`[dependency].version_a` MUST differ from `[dependency].version_b` — \
         the conflict requires two distinct pinned versions"
    );

    for flag in &["versions_are_incompatible"] {
        assert_eq!(
            dep.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[dependency].{flag}` must be true"
        );
    }

    let ra = dep
        .get("requested_by_a")
        .and_then(|v| v.as_str())
        .expect("`[dependency].requested_by_a` must be set");
    let rb = dep
        .get("requested_by_b")
        .and_then(|v| v.as_str())
        .expect("`[dependency].requested_by_b` must be set");
    assert_ne!(
        ra, rb,
        "`[dependency].requested_by_a` MUST differ from `[dependency].requested_by_b` — \
         the conflict requires two distinct requesters"
    );
}

#[test]
fn mambalibs_version_pin_conflict_case_cross_checks_dependency_block() {
    let doc = load_toml(&manifest_path());
    let case = doc.get("conflict_case").and_then(|v| v.as_table()).expect(
        "missing `[conflict_case]` block \
         (acceptance: \"Conflict fails deterministically before import.\")",
    );

    assert_eq!(
        case.get("case").and_then(|v| v.as_str()),
        Some("version_pin_conflict_detected"),
        "`[conflict_case].case` must be \"version_pin_conflict_detected\""
    );
    assert_eq!(
        case.get("expected_outcome").and_then(|v| v.as_str()),
        Some("fail"),
        "`[conflict_case].expected_outcome` must be \"fail\""
    );
    assert_eq!(
        case.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(1),
        "`[conflict_case].expected_exit_code` must be 1"
    );

    for flag in &[
        "must_fail_before_import",
        "must_be_deterministic",
        "diagnostic_must_name_dependency",
        "diagnostic_must_name_requested_versions",
        "diagnostic_must_name_requesters",
    ] {
        assert_eq!(
            case.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[conflict_case].{flag}` must be true"
        );
    }

    // Cross-check: case must name the SAME dependency the
    // `[dependency]` block names.
    let dep_name = doc
        .get("dependency")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .expect("`[dependency].name` must be set");
    assert_eq!(
        case.get("diagnostic_must_name_dependency_value")
            .and_then(|v| v.as_str()),
        Some(dep_name),
        "`[conflict_case].diagnostic_must_name_dependency_value` must equal \
         `[dependency].name`"
    );

    // Cross-check: case must list BOTH requested versions.
    let versions: Vec<&str> = case
        .get("diagnostic_must_name_requested_versions_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    let va = doc
        .get("dependency")
        .and_then(|v| v.get("version_a"))
        .and_then(|v| v.as_str())
        .expect("`[dependency].version_a` must be set");
    let vb = doc
        .get("dependency")
        .and_then(|v| v.get("version_b"))
        .and_then(|v| v.as_str())
        .expect("`[dependency].version_b` must be set");
    assert!(
        versions.contains(&va),
        "`[conflict_case].diagnostic_must_name_requested_versions_values` must include \
         `{va}` (from `[dependency].version_a`); got {versions:?}"
    );
    assert!(
        versions.contains(&vb),
        "`[conflict_case].diagnostic_must_name_requested_versions_values` must include \
         `{vb}` (from `[dependency].version_b`); got {versions:?}"
    );
}

#[test]
fn mambalibs_version_pin_conflict_crash_guard_blocks_import_attempt() {
    let doc = load_toml(&manifest_path());
    let guard = doc
        .get("crash_guard")
        .and_then(|v| v.as_table())
        .expect("missing `[crash_guard]` block");

    for flag in &[
        "must_not_attempt_import_on_conflict",
        "fail_if_interpreter_loaded_before_diagnostic",
        "fail_if_exit_code_indicates_crash",
    ] {
        assert_eq!(
            guard.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[crash_guard].{flag}` must be true"
        );
    }

    assert_eq!(
        guard
            .get("pinned_fail_exit_code")
            .and_then(|v| v.as_integer()),
        Some(1),
        "`[crash_guard].pinned_fail_exit_code` must be 1"
    );
}

#[test]
fn mambalibs_version_pin_conflict_lockfile_integrity_blocks_partial_writes() {
    let doc = load_toml(&manifest_path());
    let block = doc
        .get("lockfile_integrity")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[lockfile_integrity]` block \
         (acceptance: \"Error does not mutate lockfile into a partial success state.\")",
        );

    let lockfile = block
        .get("lockfile_relative_path")
        .and_then(|v| v.as_str())
        .expect("`[lockfile_integrity].lockfile_relative_path` must be set");
    assert_eq!(
        lockfile, "mamba.lock",
        "`[lockfile_integrity].lockfile_relative_path` must be \"mamba.lock\""
    );

    for flag in &[
        "must_remain_unchanged",
        "must_not_record_partial_resolution",
        "must_not_record_either_candidate_version",
        "hash_compare_must_be_equal",
    ] {
        assert_eq!(
            block.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[lockfile_integrity].{flag}` must be true"
        );
    }

    // hash_compare_field_keys must be a 2-tuple and BOTH keys must
    // appear in runner_contract.keys.
    let keys: Vec<&str> = block
        .get("hash_compare_field_keys")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert_eq!(
        keys.len(),
        2,
        "`[lockfile_integrity].hash_compare_field_keys` must have exactly 2 entries; got {keys:?}"
    );

    let contract_keys: Vec<&str> = doc
        .get("runner_contract")
        .and_then(|v| v.get("keys"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for k in &keys {
        assert!(
            contract_keys.contains(k),
            "`[lockfile_integrity].hash_compare_field_keys` entry `{k}` must appear in \
             `[runner_contract].keys`; got {contract_keys:?}"
        );
    }
}

#[test]
fn mambalibs_version_pin_conflict_isolates_positive_baseline() {
    let doc = load_toml(&manifest_path());
    let block = doc
        .get("isolation_from_positive_cases")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[isolation_from_positive_cases]` block \
         (acceptance: \"Positive single-version fixture still passes.\")",
        );

    assert_eq!(
        block
            .get("positive_baseline_fixture_issue")
            .and_then(|v| v.as_integer()),
        Some(2666),
        "`positive_baseline_fixture_issue` must record #2666"
    );
    for flag in &[
        "positive_baseline_fixture_must_remain_green",
        "this_fixture_must_not_affect_positive_baseline",
    ] {
        assert_eq!(
            block.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[isolation_from_positive_cases].{flag}` must be true"
        );
    }
}

#[test]
fn mambalibs_version_pin_conflict_diagnostic_contract_pins_field_keys() {
    let doc = load_toml(&manifest_path());
    let block = doc
        .get("diagnostic_contract")
        .and_then(|v| v.as_table())
        .expect("missing `[diagnostic_contract]` block");

    for flag in &[
        "diagnostic_must_be_deterministic",
        "diagnostic_must_be_actionable",
    ] {
        assert_eq!(
            block.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[diagnostic_contract].{flag}` must be true"
        );
    }

    let contract_keys: Vec<&str> = doc
        .get("runner_contract")
        .and_then(|v| v.get("keys"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for key_field in &[
        "diagnostic_must_name_dependency_field_key",
        "diagnostic_must_name_requested_versions_field_key",
        "diagnostic_must_name_requesters_field_key",
    ] {
        let key = block
            .get(*key_field)
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("`[diagnostic_contract].{key_field}` must be set"));
        assert!(
            contract_keys.contains(&key),
            "`[runner_contract].keys` must include `{key}` (from `{key_field}`); got \
             {contract_keys:?}"
        );
    }
}

#[test]
fn mambalibs_version_pin_conflict_isolation_pins_no_global_state() {
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
fn mambalibs_version_pin_conflict_runner_contract_declares_keys_and_cases() {
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
        "case",
        "dependency",
        "requested_versions",
        "requesters",
        "lockfile_before_hash",
        "lockfile_after_hash",
        "import_attempted",
        "diagnostic_message",
        "exit_code",
    ] {
        assert!(
            keys.contains(required),
            "`[runner_contract].keys` must include `{required}`; got {keys:?}"
        );
    }

    let cases: Vec<&str> = contract
        .get("case_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        cases.contains(&"version_pin_conflict_detected"),
        "`[runner_contract].case_values` must include `version_pin_conflict_detected`; \
         got {cases:?}"
    );
}

#[test]
fn mambalibs_version_pin_conflict_pins_out_of_scope_per_issue_2675() {
    let doc = load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("full_semver_resolver_implementation")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].full_semver_resolver_implementation` must be true \
         (issue text: \"Out of scope: full semver resolver implementation.\")"
    );
}
