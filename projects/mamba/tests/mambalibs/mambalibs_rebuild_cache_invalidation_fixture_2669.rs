//! Schema gate for the mambalibs rebuild cache invalidation fixture —
//! closes #2669.
//!
//! Acceptance (issue #2669):
//!
//!   1. Stale artifact reuse fails the test.
//!      `[stale_artifact_guard]` pins fail flags for cache reuse,
//!      stale import-probe, and unchanged artifact bytes after the
//!      mutation step.
//!   2. Build cache paths are isolated to the temp fixture project.
//!      `[cache_isolation].must_be_under_fixture_tmpdir == true` and
//!      `forbid_global_cache_paths` lists the canonical escape
//!      paths.
//!   3. Summary names cache hit or rebuild decision.
//!      `[summary_assertion].must_name_cache_decision == true` and
//!      `cache_decision_values` covers `cache_miss`, `cache_hit`,
//!      and `rebuild`.
//!
//! Cheap test — single TOML read + field walk. Runs in well under a
//! second; stays in the default `cargo test -p mamba` set.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("mambalibs")
        .join("fixtures")
        .join("rebuild_cache_invalidation")
        .join("manifest.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("manifest {} unreadable: {e}", path.display()));
    raw.parse()
        .unwrap_or_else(|e| panic!("{} parse error: {e}", path.display()))
}

#[test]
fn mambalibs_rebuild_manifest_header_is_well_formed() {
    let doc = load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("mambalibs_rebuild_cache_invalidation"),
        "`fixture` must be \"mambalibs_rebuild_cache_invalidation\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2669),
        "`issue` must record #2669"
    );
    assert_eq!(
        doc.get("parent_issue").and_then(|v| v.as_integer()),
        Some(2531),
        "`parent_issue` must record the Mode 2 mambalibs MVP cohort (#2531)"
    );
    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("mambalibs"),
        "`profile` must be \"mambalibs\""
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("rebuild_cache"),
        "`family` must be \"rebuild_cache\""
    );
    assert_eq!(
        doc.get("network").and_then(|v| v.as_str()),
        Some("offline"),
        "`network` must be \"offline\" (fixture is local-only)"
    );
}

#[test]
fn mambalibs_rebuild_binding_block_pins_distinct_sentinels() {
    let doc = load_toml(&manifest_path());
    let bind = doc
        .get("binding")
        .and_then(|v| v.as_table())
        .expect("missing `[binding]` block");

    assert_eq!(
        bind.get("module_name").and_then(|v| v.as_str()),
        Some("mambalibs"),
        "`[binding].module_name` must be \"mambalibs\""
    );
    let exported = bind
        .get("exported_function")
        .and_then(|v| v.as_str())
        .expect("`[binding].exported_function` must be set");
    assert!(
        !exported.is_empty(),
        "`[binding].exported_function` must be non-empty"
    );

    let initial = bind
        .get("initial_sentinel_value")
        .and_then(|v| v.as_str())
        .expect("`[binding].initial_sentinel_value` must be set");
    let mutated = bind
        .get("mutated_sentinel_value")
        .and_then(|v| v.as_str())
        .expect("`[binding].mutated_sentinel_value` must be set");
    assert_ne!(
        initial, mutated,
        "initial and mutated sentinel values MUST differ — the import \
         probe distinguishes pre vs post rebuild via these values"
    );

    let sentinel_attr = bind
        .get("sentinel_attribute")
        .and_then(|v| v.as_str())
        .expect("`[binding].sentinel_attribute` must be set");
    assert!(
        !sentinel_attr.is_empty(),
        "`[binding].sentinel_attribute` must be non-empty"
    );
}

#[test]
fn mambalibs_rebuild_cache_isolation_pins_temp_root_and_forbids_globals() {
    let doc = load_toml(&manifest_path());
    let block = doc
        .get("cache_isolation")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[cache_isolation]` block \
         (acceptance: \"Build cache paths are isolated to the temp \
         fixture project.\")",
        );

    let cache_root = block
        .get("cache_root")
        .and_then(|v| v.as_str())
        .expect("`[cache_isolation].cache_root` must be set");
    assert!(
        cache_root.contains("${FIXTURE_TMPDIR}"),
        "cache_root must live under ${{FIXTURE_TMPDIR}}; got {cache_root:?}"
    );

    assert_eq!(
        block
            .get("must_be_under_fixture_tmpdir")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[cache_isolation].must_be_under_fixture_tmpdir` must be true"
    );
    assert_eq!(
        block
            .get("must_not_persist_across_fixture_runs")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[cache_isolation].must_not_persist_across_fixture_runs` must be true"
    );

    let forbidden: Vec<&str> = block
        .get("forbid_global_cache_paths")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &["mamba"] {
        assert!(
            forbidden.iter().any(|p| p.contains(required)),
            "`[cache_isolation].forbid_global_cache_paths` must mention \
             a `{required}` path; got {forbidden:?}"
        );
    }
    assert!(
        forbidden
            .iter()
            .any(|p| p.contains("${HOME}") || p.contains("${XDG_CACHE_HOME}")),
        "`[cache_isolation].forbid_global_cache_paths` must include at \
         least one ${{HOME}} or ${{XDG_CACHE_HOME}} path; got {forbidden:?}"
    );
}

#[test]
fn mambalibs_rebuild_initial_build_case_populates_cache() {
    let doc = load_toml(&manifest_path());
    let block = doc
        .get("initial_build_case")
        .and_then(|v| v.as_table())
        .expect("missing `[initial_build_case]` block");

    assert_eq!(
        block.get("case").and_then(|v| v.as_str()),
        Some("initial_build"),
        "`[initial_build_case].case` must be \"initial_build\""
    );
    assert_eq!(
        block.get("expected_outcome").and_then(|v| v.as_str()),
        Some("pass"),
        "`[initial_build_case].expected_outcome` must be \"pass\""
    );
    assert_eq!(
        block.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(0),
        "`[initial_build_case].expected_exit_code` must be 0"
    );
    assert_eq!(
        block.get("must_populate_cache").and_then(|v| v.as_bool()),
        Some(true),
        "`[initial_build_case].must_populate_cache` must be true"
    );
    assert_eq!(
        block
            .get("expected_cache_decision")
            .and_then(|v| v.as_str()),
        Some("cache_miss"),
        "`[initial_build_case].expected_cache_decision` must be \"cache_miss\""
    );

    let probe = block
        .get("import_probe_value_must_be")
        .and_then(|v| v.as_str())
        .expect("`[initial_build_case].import_probe_value_must_be` must be set");
    let initial = doc
        .get("binding")
        .and_then(|v| v.get("initial_sentinel_value"))
        .and_then(|v| v.as_str())
        .expect("`[binding].initial_sentinel_value` must be set");
    assert_eq!(
        probe, initial,
        "initial build's import probe must equal `[binding].initial_sentinel_value`"
    );
}

#[test]
fn mambalibs_rebuild_mutation_step_flips_sentinel() {
    let doc = load_toml(&manifest_path());
    let block = doc
        .get("mutation_step")
        .and_then(|v| v.as_table())
        .expect("missing `[mutation_step]` block");

    let old = block
        .get("old_value")
        .and_then(|v| v.as_str())
        .expect("`[mutation_step].old_value` must be set");
    let new_val = block
        .get("new_value")
        .and_then(|v| v.as_str())
        .expect("`[mutation_step].new_value` must be set");
    assert_ne!(old, new_val, "mutation step old/new values must differ");

    let initial = doc
        .get("binding")
        .and_then(|v| v.get("initial_sentinel_value"))
        .and_then(|v| v.as_str())
        .expect("`[binding].initial_sentinel_value` must be set");
    let mutated = doc
        .get("binding")
        .and_then(|v| v.get("mutated_sentinel_value"))
        .and_then(|v| v.as_str())
        .expect("`[binding].mutated_sentinel_value` must be set");
    assert_eq!(
        old, initial,
        "mutation step old_value must equal `[binding].initial_sentinel_value`"
    );
    assert_eq!(
        new_val, mutated,
        "mutation step new_value must equal `[binding].mutated_sentinel_value`"
    );

    assert_eq!(
        block.get("must_be_deterministic").and_then(|v| v.as_bool()),
        Some(true),
        "`[mutation_step].must_be_deterministic` must be true"
    );
    let mutates_attr = block
        .get("mutates_attribute")
        .and_then(|v| v.as_str())
        .expect("`[mutation_step].mutates_attribute` must be set");
    let binding_attr = doc
        .get("binding")
        .and_then(|v| v.get("sentinel_attribute"))
        .and_then(|v| v.as_str())
        .expect("`[binding].sentinel_attribute` must be set");
    assert_eq!(
        mutates_attr, binding_attr,
        "mutation step must target the binding's sentinel attribute"
    );
}

#[test]
fn mambalibs_rebuild_case_reflects_mutated_source() {
    let doc = load_toml(&manifest_path());
    let block = doc
        .get("rebuild_case")
        .and_then(|v| v.as_table())
        .expect("missing `[rebuild_case]` block");

    assert_eq!(
        block.get("case").and_then(|v| v.as_str()),
        Some("rebuild_after_mutation"),
        "`[rebuild_case].case` must be \"rebuild_after_mutation\""
    );
    assert_eq!(
        block.get("expected_outcome").and_then(|v| v.as_str()),
        Some("pass"),
        "`[rebuild_case].expected_outcome` must be \"pass\""
    );
    assert_eq!(
        block
            .get("expected_cache_decision")
            .and_then(|v| v.as_str()),
        Some("rebuild"),
        "`[rebuild_case].expected_cache_decision` must be \"rebuild\" \
         (cache MUST be invalidated by the mutation)"
    );
    for flag in &["rebuild_must_happen", "import_probe_must_be_post_mutation"] {
        assert_eq!(
            block.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[rebuild_case].{flag}` must be true"
        );
    }

    let mutated = doc
        .get("binding")
        .and_then(|v| v.get("mutated_sentinel_value"))
        .and_then(|v| v.as_str())
        .expect("`[binding].mutated_sentinel_value` must be set");
    let initial = doc
        .get("binding")
        .and_then(|v| v.get("initial_sentinel_value"))
        .and_then(|v| v.as_str())
        .expect("`[binding].initial_sentinel_value` must be set");

    assert_eq!(
        block
            .get("import_probe_value_must_be")
            .and_then(|v| v.as_str()),
        Some(mutated),
        "`[rebuild_case].import_probe_value_must_be` must equal \
         `[binding].mutated_sentinel_value`"
    );
    assert_eq!(
        block
            .get("import_probe_must_not_be")
            .and_then(|v| v.as_str()),
        Some(initial),
        "`[rebuild_case].import_probe_must_not_be` must equal \
         `[binding].initial_sentinel_value`"
    );
}

#[test]
fn mambalibs_rebuild_stale_artifact_guard_pins_all_fail_flags() {
    let doc = load_toml(&manifest_path());
    let guard = doc
        .get("stale_artifact_guard")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[stale_artifact_guard]` block \
         (acceptance: \"Stale artifact reuse fails the test.\")",
        );

    for flag in &[
        "fail_if_cache_decision_is_cache_hit_after_mutation",
        "fail_if_import_probe_returns_initial_after_mutation",
        "fail_if_artifact_bytes_unchanged_after_mutation",
        "diagnostic_must_name_offending_artifact",
        "diagnostic_must_name_old_and_new_sentinel_values",
    ] {
        assert_eq!(
            guard.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[stale_artifact_guard].{flag}` must be true"
        );
    }
}

#[test]
fn mambalibs_rebuild_summary_assertion_names_cache_decision() {
    let doc = load_toml(&manifest_path());
    let block = doc
        .get("summary_assertion")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[summary_assertion]` block \
         (acceptance: \"Summary names cache hit or rebuild decision.\")",
        );

    assert_eq!(
        block
            .get("must_name_cache_decision")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[summary_assertion].must_name_cache_decision` must be true"
    );
    let field_key = block
        .get("cache_decision_field_key")
        .and_then(|v| v.as_str())
        .expect("`[summary_assertion].cache_decision_field_key` must be set");
    assert_eq!(
        field_key, "cache_decision",
        "field key should be `cache_decision`; got {field_key:?}"
    );

    let values: Vec<&str> = block
        .get("cache_decision_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &["cache_miss", "cache_hit", "rebuild"] {
        assert!(
            values.contains(required),
            "`[summary_assertion].cache_decision_values` must include `{required}`; \
             got {values:?}"
        );
    }
    for flag in &["must_name_artifact_identity", "must_name_sentinel_value"] {
        assert_eq!(
            block.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[summary_assertion].{flag}` must be true"
        );
    }
}

#[test]
fn mambalibs_rebuild_isolation_pins_no_global_state() {
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
fn mambalibs_rebuild_runner_contract_declares_keys_and_cases() {
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
        "module",
        "exported_function",
        "cache_decision",
        "artifact_identity",
        "sentinel_value",
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
    for required in &["initial_build", "rebuild_after_mutation"] {
        assert!(
            cases.contains(required),
            "`[runner_contract].case_values` must include `{required}`; got {cases:?}"
        );
    }
}

#[test]
fn mambalibs_rebuild_pins_out_of_scope_per_issue_2669() {
    let doc = load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("optimizing_rebuild_performance")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].optimizing_rebuild_performance` must be true \
         (issue text: \"Out of scope: optimizing rebuild performance.\")"
    );
}
