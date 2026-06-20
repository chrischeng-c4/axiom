//! Schema gate for the package-manager hash verification fixture —
//! closes #2686.
//!
//! Acceptance (issue #2686):
//!
//!   1. Hash mismatch fails before installation is accepted.
//!      `[tampered_hash_case]` pins `expected_exit_code != 0`,
//!      `must_fail_before_install = true`,
//!      `must_not_install_package = true`.
//!   2. Diagnostic names package, version, expected hash, and
//!      observed hash shape.
//!      `[diagnostic_assertion]` sets all four `must_name_*` flags
//!      and lists the `observed_hash_shape_keys`.
//!   3. Correct hash path still installs and imports.
//!      `[correct_hash_case]` pins exit 0, install required,
//!      import probe = `import_ok`.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("pkgmgr")
        .join("hash")
        .join("manifest.toml")
}

fn profile_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("validation")
        .join("profiles")
        .join("package_manager.toml")
}

#[test]
fn pkgmgr_hash_verification_manifest_header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("pkgmgr_hash_verification"),
        "`fixture` must be \"pkgmgr_hash_verification\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2686),
        "`issue` must record #2686"
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("hash"),
        "`family` must be \"hash\""
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
fn pkgmgr_hash_package_block_pins_algorithm_and_length() {
    let doc = crate::common::load_toml(&manifest_path());
    let pkg = doc
        .get("package")
        .and_then(|v| v.as_table())
        .expect("missing `[package]` block");

    let name = pkg
        .get("name")
        .and_then(|v| v.as_str())
        .expect("`[package].name` must be set");
    assert!(!name.is_empty(), "package name must be non-empty");

    let version = pkg
        .get("version")
        .and_then(|v| v.as_str())
        .expect("`[package].version` must be set");
    assert!(!version.is_empty(), "package version must be non-empty");

    let algo = pkg
        .get("hash_algorithm")
        .and_then(|v| v.as_str())
        .expect("`[package].hash_algorithm` must be set");
    assert_eq!(
        algo, "sha256",
        "`[package].hash_algorithm` must be `sha256` — MVP uses one canonical algo"
    );

    let length = pkg
        .get("expected_hash_hex_length")
        .and_then(|v| v.as_integer())
        .expect("`[package].expected_hash_hex_length` must be set");
    assert_eq!(length, 64, "sha256 hex length must be 64; got {length}");
}

#[test]
fn pkgmgr_hash_correct_hash_case_installs_and_imports() {
    let doc = crate::common::load_toml(&manifest_path());
    let case = doc
        .get("correct_hash_case")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[correct_hash_case]` block \
         (acceptance: \"Correct hash path still installs and imports.\")",
        );

    assert_eq!(
        case.get("hash_matches").and_then(|v| v.as_bool()),
        Some(true),
        "`[correct_hash_case].hash_matches` must be true"
    );
    assert_eq!(
        case.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(0),
        "`[correct_hash_case].expected_exit_code` must be 0"
    );
    assert_eq!(
        case.get("must_install_package").and_then(|v| v.as_bool()),
        Some(true),
        "`[correct_hash_case].must_install_package` must be true"
    );

    let probe = case
        .get("import_probe")
        .and_then(|v| v.as_str())
        .expect("`[correct_hash_case].import_probe` must be set");
    let pkg_name = doc
        .get("package")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .expect("`[package].name` must be set");
    assert_eq!(
        probe, pkg_name,
        "`[correct_hash_case].import_probe` must equal `[package].name`"
    );

    assert_eq!(
        case.get("expected_import_outcome").and_then(|v| v.as_str()),
        Some("import_ok"),
        "`[correct_hash_case].expected_import_outcome` must be \"import_ok\""
    );
}

#[test]
fn pkgmgr_hash_tampered_hash_case_fails_before_install() {
    let doc = crate::common::load_toml(&manifest_path());
    let case = doc
        .get("tampered_hash_case")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[tampered_hash_case]` block \
         (acceptance: \"Hash mismatch fails before installation is accepted.\")",
        );

    assert_eq!(
        case.get("hash_matches").and_then(|v| v.as_bool()),
        Some(false),
        "`[tampered_hash_case].hash_matches` must be false"
    );

    let exit = case
        .get("expected_exit_code")
        .and_then(|v| v.as_integer())
        .expect("`[tampered_hash_case].expected_exit_code` must be set");
    assert_ne!(exit, 0, "tampered hash case must NOT exit 0; got {exit}");

    assert_eq!(
        case.get("must_fail_before_install")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[tampered_hash_case].must_fail_before_install` must be true — \
         the verifier rejects before any wheel is extracted"
    );
    assert_eq!(
        case.get("must_not_install_package")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[tampered_hash_case].must_not_install_package` must be true"
    );
    assert_eq!(
        case.get("expected_import_outcome").and_then(|v| v.as_str()),
        Some("module_not_found"),
        "`[tampered_hash_case].expected_import_outcome` must be \"module_not_found\" \
         — install rejected, env stays clean"
    );
}

#[test]
fn pkgmgr_hash_diagnostic_assertion_names_pkg_version_and_hashes() {
    let doc = crate::common::load_toml(&manifest_path());
    let diag = doc
        .get("diagnostic_assertion")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[diagnostic_assertion]` block \
         (acceptance: \"Diagnostic names package, version, expected hash, \
         and observed hash shape.\")",
        );

    for flag in &[
        "must_name_package",
        "must_name_version",
        "must_name_expected_hash",
        "must_name_observed_hash_shape",
    ] {
        assert_eq!(
            diag.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[diagnostic_assertion].{flag}` must be true"
        );
    }

    let keys: Vec<&str> = diag
        .get("observed_hash_shape_keys")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &["algorithm", "length", "prefix"] {
        assert!(
            keys.contains(required),
            "`[diagnostic_assertion].observed_hash_shape_keys` must include `{required}`; got {keys:?}"
        );
    }
}

#[test]
fn pkgmgr_hash_isolation_pins_no_global_state() {
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
fn pkgmgr_hash_runner_contract_declares_outcome_keys() {
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
        "package_name",
        "package_version",
        "expected_hash",
        "observed_hash_prefix",
        "case",
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
        cases.contains(&"correct_hash") && cases.contains(&"tampered_hash"),
        "`[runner_contract].case_values` must carry `correct_hash` and `tampered_hash`; got {cases:?}"
    );
}

#[test]
fn pkgmgr_hash_pins_out_of_scope_per_issue_2686() {
    let doc = crate::common::load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("remote_download_transport")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].remote_download_transport` must be true \
         (issue text: \"Out of scope: remote download transport.\")"
    );
}

#[test]
fn pkgmgr_profile_links_to_hash_fixture_directory() {
    let doc = crate::common::load_toml(&profile_path());
    let hash = doc
        .get("families")
        .and_then(|v| v.get("hash"))
        .and_then(|v| v.as_table())
        .expect("validation/profiles/package_manager.toml missing `[families.hash]`");

    let source = hash
        .get("source")
        .and_then(|v| v.as_str())
        .expect("`[families.hash].source` must be set");
    assert_eq!(
        source, "tests/governance/gates/pkgmgr/hash",
        "`[families.hash].source` must point at `tests/governance/gates/pkgmgr/hash`; got {source:?}"
    );

    let kind = hash
        .get("kind")
        .and_then(|v| v.as_str())
        .expect("`[families.hash].kind` must be set");
    assert_eq!(
        kind, "pkgmgr_hash",
        "`[families.hash].kind` must be `pkgmgr_hash`"
    );
}
