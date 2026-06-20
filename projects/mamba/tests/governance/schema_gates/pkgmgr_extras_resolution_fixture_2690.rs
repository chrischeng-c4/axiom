//! Schema gate for the package-manager extras resolution fixture —
//! closes #2690.
//!
//! Acceptance (issue #2690):
//!
//!   1. Extra dependency appears only when requested.
//!      `[without_extra_case]` and `[with_extra_case]` flip the
//!      `must_install_extra_dependency` flag, and the lockfile diff
//!      assertion checks both directions.
//!   2. Runtime import of extra dependency matches lockfile state.
//!      `[runtime_lockfile_consistency]` pins both directions.
//!   3. No network is used.
//!      Header `network = "offline"` + `[isolation]` block.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("pkgmgr")
        .join("extras_resolution")
        .join("manifest.toml")
}

#[test]
fn pkgmgr_extras_resolution_manifest_header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("pkgmgr_extras_resolution"),
        "`fixture` must be \"pkgmgr_extras_resolution\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2690),
        "`issue` must record #2690"
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("extras_resolution"),
        "`family` must be \"extras_resolution\""
    );
    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("package_manager"),
        "`profile` must be \"package_manager\""
    );
    assert_eq!(
        doc.get("network").and_then(|v| v.as_str()),
        Some("offline"),
        "`network` must be \"offline\" — acceptance: \"No network is used.\""
    );
}

#[test]
fn pkgmgr_extras_base_package_and_extra_blocks_pin_one_optional_dep() {
    let doc = crate::common::load_toml(&manifest_path());

    let base = doc
        .get("base_package")
        .and_then(|v| v.as_table())
        .expect("missing `[base_package]` block");
    let base_name = base
        .get("name")
        .and_then(|v| v.as_str())
        .expect("`[base_package].name` must be set");
    assert!(!base_name.is_empty(), "base package name must be non-empty");
    let base_version = base
        .get("version")
        .and_then(|v| v.as_str())
        .expect("`[base_package].version` must be set");
    assert!(
        !base_version.is_empty(),
        "base package version must be non-empty"
    );

    let extra = doc
        .get("extra")
        .and_then(|v| v.as_table())
        .expect("missing `[extra]` block");
    let extra_name = extra
        .get("name")
        .and_then(|v| v.as_str())
        .expect("`[extra].name` must be set");
    assert!(!extra_name.is_empty(), "extra name must be non-empty");

    let extra_dep = extra
        .get("dependency")
        .and_then(|v| v.as_str())
        .expect("`[extra].dependency` must be set");
    assert_ne!(
        extra_dep, base_name,
        "extra dependency must differ from base package — otherwise lockfile diff is meaningless"
    );

    let extra_version = extra
        .get("dependency_version")
        .and_then(|v| v.as_str())
        .expect("`[extra].dependency_version` must be set");
    assert!(
        !extra_version.is_empty(),
        "extra dependency version must be non-empty"
    );
}

#[test]
fn pkgmgr_extras_without_extra_case_excludes_extra_dependency() {
    let doc = crate::common::load_toml(&manifest_path());
    let case = doc
        .get("without_extra_case")
        .and_then(|v| v.as_table())
        .expect("missing `[without_extra_case]` block");

    let command: Vec<&str> = case
        .get("command")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert_eq!(
        command.first().copied(),
        Some("add"),
        "`[without_extra_case].command[0]` must be `add`"
    );
    let spec = command
        .get(1)
        .copied()
        .expect("`[without_extra_case].command` must include a package spec");
    assert!(
        !spec.contains('[') && !spec.contains(']'),
        "without-extra spec must NOT include `[extra]`; got {spec:?}"
    );

    assert_eq!(
        case.get("expected_outcome").and_then(|v| v.as_str()),
        Some("pass"),
        "`[without_extra_case].expected_outcome` must be \"pass\""
    );
    assert_eq!(
        case.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(0),
        "`[without_extra_case].expected_exit_code` must be 0"
    );
    assert_eq!(
        case.get("must_install_base").and_then(|v| v.as_bool()),
        Some(true),
        "`[without_extra_case].must_install_base` must be true"
    );
    assert_eq!(
        case.get("must_not_install_extra_dependency")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[without_extra_case].must_not_install_extra_dependency` must be true"
    );

    let extra_dep = doc
        .get("extra")
        .and_then(|v| v.get("dependency"))
        .and_then(|v| v.as_str())
        .expect("`[extra].dependency` must be set");
    assert_eq!(
        case.get("import_probe").and_then(|v| v.as_str()),
        Some(extra_dep),
        "`[without_extra_case].import_probe` must equal `[extra].dependency`"
    );
    assert_eq!(
        case.get("expected_import_outcome").and_then(|v| v.as_str()),
        Some("module_not_found"),
        "`[without_extra_case].expected_import_outcome` must be \"module_not_found\""
    );
}

#[test]
fn pkgmgr_extras_with_extra_case_includes_extra_dependency() {
    let doc = crate::common::load_toml(&manifest_path());
    let case = doc
        .get("with_extra_case")
        .and_then(|v| v.as_table())
        .expect("missing `[with_extra_case]` block");

    let command: Vec<&str> = case
        .get("command")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert_eq!(
        command.first().copied(),
        Some("add"),
        "`[with_extra_case].command[0]` must be `add`"
    );
    let spec = command
        .get(1)
        .copied()
        .expect("`[with_extra_case].command` must include a package spec");

    let base_name = doc
        .get("base_package")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .expect("`[base_package].name` must be set");
    let extra_name = doc
        .get("extra")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .expect("`[extra].name` must be set");
    let bracket = format!("{base_name}[{extra_name}]");
    assert!(
        spec.contains(&bracket),
        "with-extra spec must use PEP 508 `pkg[extra]` shape ({bracket:?}); got {spec:?}"
    );

    assert_eq!(
        case.get("expected_outcome").and_then(|v| v.as_str()),
        Some("pass"),
        "`[with_extra_case].expected_outcome` must be \"pass\""
    );
    assert_eq!(
        case.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(0),
        "`[with_extra_case].expected_exit_code` must be 0"
    );
    assert_eq!(
        case.get("must_install_base").and_then(|v| v.as_bool()),
        Some(true),
        "`[with_extra_case].must_install_base` must be true"
    );
    assert_eq!(
        case.get("must_install_extra_dependency")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[with_extra_case].must_install_extra_dependency` must be true"
    );

    let extra_dep = doc
        .get("extra")
        .and_then(|v| v.get("dependency"))
        .and_then(|v| v.as_str())
        .expect("`[extra].dependency` must be set");
    assert_eq!(
        case.get("import_probe").and_then(|v| v.as_str()),
        Some(extra_dep),
        "`[with_extra_case].import_probe` must equal `[extra].dependency`"
    );
    assert_eq!(
        case.get("expected_import_outcome").and_then(|v| v.as_str()),
        Some("import_ok"),
        "`[with_extra_case].expected_import_outcome` must be \"import_ok\""
    );
}

#[test]
fn pkgmgr_extras_lockfile_diff_assertion_pins_both_directions() {
    let doc = crate::common::load_toml(&manifest_path());
    let diff = doc
        .get("lockfile_diff_assertion")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[lockfile_diff_assertion]` block \
         (acceptance: \"Assert lockfile contents differ deterministically.\")",
        );

    for flag in &[
        "deterministic",
        "byte_identical_on_replay",
        "extras_field_required_when_requested",
    ] {
        assert_eq!(
            diff.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[lockfile_diff_assertion].{flag}` must be true"
        );
    }

    let extra_dep = doc
        .get("extra")
        .and_then(|v| v.get("dependency"))
        .and_then(|v| v.as_str())
        .expect("`[extra].dependency` must be set");
    let extra_name = doc
        .get("extra")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .expect("`[extra].name` must be set");

    assert_eq!(
        diff.get("without_extra_must_not_contain").and_then(|v| v.as_str()),
        Some(extra_dep),
        "`[lockfile_diff_assertion].without_extra_must_not_contain` must equal `[extra].dependency`"
    );
    assert_eq!(
        diff.get("with_extra_must_contain").and_then(|v| v.as_str()),
        Some(extra_dep),
        "`[lockfile_diff_assertion].with_extra_must_contain` must equal `[extra].dependency`"
    );

    let extras_value: Vec<&str> = diff
        .get("extras_field_value_when_requested")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        extras_value.contains(&extra_name),
        "`[lockfile_diff_assertion].extras_field_value_when_requested` must include `[extra].name`; got {extras_value:?}"
    );
}

#[test]
fn pkgmgr_extras_runtime_lockfile_consistency_pins_both_directions() {
    let doc = crate::common::load_toml(&manifest_path());
    let cons = doc
        .get("runtime_lockfile_consistency")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[runtime_lockfile_consistency]` block \
         (acceptance: \"Runtime import of extra dependency matches lockfile state.\")",
        );

    for flag in &[
        "without_extra_import_matches_lockfile",
        "with_extra_import_matches_lockfile",
        "diagnostic_must_name_extra_when_mismatched",
    ] {
        assert_eq!(
            cons.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[runtime_lockfile_consistency].{flag}` must be true"
        );
    }
}

#[test]
fn pkgmgr_extras_isolation_pins_no_global_state() {
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
fn pkgmgr_extras_runner_contract_declares_outcome_keys() {
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
        "case",
        "base_package",
        "extra_name",
        "extra_dependency",
        "lockfile_contains_extra_dep",
        "import_outcome",
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
        cases.contains(&"without_extra") && cases.contains(&"with_extra"),
        "`[runner_contract].case_values` must carry `without_extra` and `with_extra`; got {cases:?}"
    );
}

#[test]
fn pkgmgr_extras_pins_out_of_scope_per_issue_2690() {
    let doc = crate::common::load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("all_marker_and_extras_combinations")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].all_marker_and_extras_combinations` must be true \
         (issue text: \"Out of scope: all marker and extras combinations.\")"
    );
}
