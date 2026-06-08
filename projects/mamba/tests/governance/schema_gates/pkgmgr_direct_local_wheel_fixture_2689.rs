//! Schema gate for the package-manager direct local wheel fixture —
//! closes #2689.
//!
//! Acceptance (issue #2689):
//!
//!   1. Direct local wheel install works offline.
//!      `[action]` pins `expected_outcome = "pass"`, exit 0,
//!      `must_install_package = true`.
//!   2. Lockfile records deterministic local source metadata.
//!      `[lockfile_assertion]` pins `source_kind = "direct_file"`,
//!      records the relative path + sha256 hash, forbids index URL,
//!      and asserts byte-identical replay.
//!   3. Missing wheel path fails with a clear diagnostic.
//!      `[missing_wheel_case]` pins non-zero exit,
//!      `must_fail_before_install`, and a diagnostic that names the
//!      missing path.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("pkgmgr")
        .join("direct_local_wheel")
        .join("manifest.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("manifest {} unreadable: {e}", path.display()));
    raw.parse()
        .unwrap_or_else(|e| panic!("{} parse error: {e}", path.display()))
}

#[test]
fn pkgmgr_direct_local_wheel_manifest_header_is_well_formed() {
    let doc = load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("pkgmgr_direct_local_wheel"),
        "`fixture` must be \"pkgmgr_direct_local_wheel\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2689),
        "`issue` must record #2689"
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("direct_local_wheel"),
        "`family` must be \"direct_local_wheel\""
    );
    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("package_manager"),
        "`profile` must be \"package_manager\""
    );
    assert_eq!(
        doc.get("network").and_then(|v| v.as_str()),
        Some("offline"),
        "`network` must be \"offline\" — local wheel install is offline-only"
    );
    assert_eq!(
        doc.get("index_source").and_then(|v| v.as_str()),
        Some("direct_file_path"),
        "`index_source` must be \"direct_file_path\" — not the simple index"
    );
}

#[test]
fn pkgmgr_direct_local_wheel_block_pins_pep427_filename_shape() {
    let doc = load_toml(&manifest_path());
    let wheel = doc
        .get("wheel")
        .and_then(|v| v.as_table())
        .expect("missing `[wheel]` block");

    let name = wheel
        .get("name")
        .and_then(|v| v.as_str())
        .expect("`[wheel].name` must be set");
    assert!(!name.is_empty(), "wheel name must be non-empty");

    let version = wheel
        .get("version")
        .and_then(|v| v.as_str())
        .expect("`[wheel].version` must be set");
    assert!(!version.is_empty(), "wheel version must be non-empty");

    let filename = wheel
        .get("filename")
        .and_then(|v| v.as_str())
        .expect("`[wheel].filename` must be set");
    assert!(
        filename.ends_with(".whl"),
        "`[wheel].filename` must end with `.whl`; got {filename:?}"
    );
    assert!(
        filename.contains(name) && filename.contains(version),
        "PEP 427 wheel filename must embed name + version; got {filename:?}"
    );

    let relative_path = wheel
        .get("relative_path")
        .and_then(|v| v.as_str())
        .expect("`[wheel].relative_path` must be set");
    assert!(
        relative_path.ends_with(filename),
        "`[wheel].relative_path` must end with `[wheel].filename`; got {relative_path:?}"
    );
    assert!(
        !Path::new(relative_path).is_absolute(),
        "`[wheel].relative_path` must be project-relative; got {relative_path:?}"
    );

    for tag_key in &["python_tag", "abi_tag", "platform_tag"] {
        let tag = wheel
            .get(*tag_key)
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("`[wheel].{tag_key}` must be set"));
        assert!(!tag.is_empty(), "`[wheel].{tag_key}` must be non-empty");
    }
}

#[test]
fn pkgmgr_direct_local_wheel_action_uses_direct_path_not_name() {
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

    let path_arg = command
        .get(1)
        .copied()
        .expect("`[action].command` must include a path argument");
    let wheel_path = doc
        .get("wheel")
        .and_then(|v| v.get("relative_path"))
        .and_then(|v| v.as_str())
        .expect("`[wheel].relative_path` must be set");
    assert!(
        path_arg.contains(wheel_path),
        "`[action].command[1]` must reference the wheel's relative_path; got {path_arg:?}"
    );
    assert!(
        path_arg.starts_with("./")
            || path_arg.starts_with("./wheels")
            || path_arg.starts_with("wheels/"),
        "direct-path add must use a project-relative wheel path; got {path_arg:?}"
    );

    assert_eq!(
        action.get("expected_outcome").and_then(|v| v.as_str()),
        Some("pass"),
        "`[action].expected_outcome` must be \"pass\""
    );
    assert_eq!(
        action
            .get("expected_exit_code")
            .and_then(|v| v.as_integer()),
        Some(0),
        "`[action].expected_exit_code` must be 0"
    );
    assert_eq!(
        action.get("must_install_package").and_then(|v| v.as_bool()),
        Some(true),
        "`[action].must_install_package` must be true"
    );
}

#[test]
fn pkgmgr_direct_local_wheel_lockfile_records_direct_file_source() {
    let doc = load_toml(&manifest_path());
    let lock = doc
        .get("lockfile_assertion")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[lockfile_assertion]` block \
         (acceptance: \"Lockfile records deterministic local source metadata.\")",
        );

    let wheel_name = doc
        .get("wheel")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .expect("`[wheel].name` must be set");
    let wheel_version = doc
        .get("wheel")
        .and_then(|v| v.get("version"))
        .and_then(|v| v.as_str())
        .expect("`[wheel].version` must be set");
    let wheel_path = doc
        .get("wheel")
        .and_then(|v| v.get("relative_path"))
        .and_then(|v| v.as_str())
        .expect("`[wheel].relative_path` must be set");

    assert_eq!(
        lock.get("must_contain_dependency").and_then(|v| v.as_str()),
        Some(wheel_name),
        "`[lockfile_assertion].must_contain_dependency` must equal `[wheel].name`"
    );
    assert_eq!(
        lock.get("must_pin_version").and_then(|v| v.as_str()),
        Some(wheel_version),
        "`[lockfile_assertion].must_pin_version` must equal `[wheel].version`"
    );
    assert_eq!(
        lock.get("must_record_source_kind").and_then(|v| v.as_str()),
        Some("direct_file"),
        "`[lockfile_assertion].must_record_source_kind` must be \"direct_file\""
    );
    assert_eq!(
        lock.get("must_record_relative_path")
            .and_then(|v| v.as_str()),
        Some(wheel_path),
        "`[lockfile_assertion].must_record_relative_path` must equal `[wheel].relative_path`"
    );
    assert_eq!(
        lock.get("must_record_hash").and_then(|v| v.as_bool()),
        Some(true),
        "`[lockfile_assertion].must_record_hash` must be true"
    );
    assert_eq!(
        lock.get("hash_algorithm").and_then(|v| v.as_str()),
        Some("sha256"),
        "`[lockfile_assertion].hash_algorithm` must be `sha256`"
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
    assert_eq!(
        lock.get("must_not_record_index_url")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[lockfile_assertion].must_not_record_index_url` must be true — direct file source"
    );
}

#[test]
fn pkgmgr_direct_local_wheel_install_assertion_probes_import() {
    let doc = load_toml(&manifest_path());
    let install = doc
        .get("install_assertion")
        .and_then(|v| v.as_table())
        .expect("missing `[install_assertion]` block");

    let wheel_name = doc
        .get("wheel")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .expect("`[wheel].name` must be set");

    assert_eq!(
        install.get("must_install_package").and_then(|v| v.as_str()),
        Some(wheel_name),
        "`[install_assertion].must_install_package` must equal `[wheel].name`"
    );
    assert_eq!(
        install.get("import_probe").and_then(|v| v.as_str()),
        Some(wheel_name),
        "`[install_assertion].import_probe` must equal `[wheel].name`"
    );
    assert_eq!(
        install
            .get("expected_import_outcome")
            .and_then(|v| v.as_str()),
        Some("import_ok"),
        "`[install_assertion].expected_import_outcome` must be \"import_ok\""
    );
    assert_eq!(
        install
            .get("metadata_records_local_source")
            .and_then(|v| v.as_bool()),
        Some(true),
        "`[install_assertion].metadata_records_local_source` must be true"
    );
}

#[test]
fn pkgmgr_direct_local_wheel_missing_wheel_case_fails_loud() {
    let doc = load_toml(&manifest_path());
    let case = doc
        .get("missing_wheel_case")
        .and_then(|v| v.as_table())
        .expect(
            "missing `[missing_wheel_case]` block \
         (acceptance: \"Missing wheel path fails with a clear diagnostic.\")",
        );

    let relative_path = case
        .get("relative_path")
        .and_then(|v| v.as_str())
        .expect("`[missing_wheel_case].relative_path` must be set");
    let real_path = doc
        .get("wheel")
        .and_then(|v| v.get("relative_path"))
        .and_then(|v| v.as_str())
        .expect("`[wheel].relative_path` must be set");
    assert_ne!(
        relative_path, real_path,
        "missing wheel path must differ from the real wheel — gate compares both"
    );

    assert_eq!(
        case.get("expected_outcome").and_then(|v| v.as_str()),
        Some("fail"),
        "`[missing_wheel_case].expected_outcome` must be \"fail\""
    );
    let exit = case
        .get("expected_exit_code")
        .and_then(|v| v.as_integer())
        .expect("`[missing_wheel_case].expected_exit_code` must be set");
    assert_ne!(exit, 0, "missing wheel exit must be non-zero; got {exit}");

    for flag in &[
        "must_fail_before_install",
        "must_not_install_package",
        "must_not_mutate_lockfile",
        "diagnostic_must_name_path",
    ] {
        assert_eq!(
            case.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[missing_wheel_case].{flag}` must be true"
        );
    }

    let substring = case
        .get("diagnostic_message_substring")
        .and_then(|v| v.as_str())
        .expect("`[missing_wheel_case].diagnostic_message_substring` must be set");
    assert!(
        !substring.is_empty(),
        "`[missing_wheel_case].diagnostic_message_substring` must be non-empty"
    );
    assert!(
        relative_path.contains(substring),
        "diagnostic substring {substring:?} must appear in the missing path {relative_path:?}"
    );
}

#[test]
fn pkgmgr_direct_local_wheel_isolation_pins_no_global_state() {
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
fn pkgmgr_direct_local_wheel_runner_contract_declares_outcome_keys() {
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
        "wheel_name",
        "wheel_version",
        "wheel_path",
        "source_kind",
        "exit_code",
        "diagnostic_message",
        "case",
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
        cases.contains(&"direct_local_wheel") && cases.contains(&"missing_wheel"),
        "`[runner_contract].case_values` must carry `direct_local_wheel` and `missing_wheel`; got {cases:?}"
    );
}

#[test]
fn pkgmgr_direct_local_wheel_pins_out_of_scope_per_issue_2689() {
    let doc = load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("remote_url_dependencies").and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].remote_url_dependencies` must be true \
         (issue text: \"Out of scope: remote URL dependencies.\")"
    );
}
