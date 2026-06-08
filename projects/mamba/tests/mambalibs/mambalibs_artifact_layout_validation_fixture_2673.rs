//! Schema gate for the mambalibs artifact layout validation fixture —
//! closes #2673.
//!
//! Acceptance (issue #2673):
//!
//!   1. Missing artifact files fail with exact path names.
//!      `[missing_file_case]` pins `fail` outcome with the exact
//!      missing path; `[diagnostic_contract]` enforces the
//!      `missing_path` field key.
//!   2. Platform-specific suffix handling is documented in the test.
//!      `[platform_suffix_table]` declares macOS/Linux/Windows
//!      suffixes plus a documentation string explaining the
//!      placeholder substitution.
//!   3. Import fixture can consume the same artifact layout.
//!      `[import_compatibility]` pins the cross-fixture invariant
//!      against #2666 (module/__init__.py + native extension are
//!      required for import).
//!
//! Cheap test — single TOML read + field walk. Runs in well under a
//! second; stays in the default `cargo test -p mamba` set.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("mambalibs")
        .join("fixtures")
        .join("artifact_layout_validation")
        .join("manifest.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("manifest {} unreadable: {e}", path.display()));
    raw.parse()
        .unwrap_or_else(|e| panic!("{} parse error: {e}", path.display()))
}

#[test]
fn mambalibs_artifact_layout_manifest_header_is_well_formed() {
    let doc = load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("mambalibs_artifact_layout_validation"),
        "`fixture` must be \"mambalibs_artifact_layout_validation\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2673),
        "`issue` must record #2673"
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
        Some("artifact_layout_validation"),
        "`family` must be \"artifact_layout_validation\""
    );
    assert_eq!(
        doc.get("network").and_then(|v| v.as_str()),
        Some("offline"),
        "`network` must be \"offline\""
    );
}

#[test]
fn mambalibs_artifact_layout_binding_block_pins_artifact_root() {
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

    let root = bind
        .get("artifact_root")
        .and_then(|v| v.as_str())
        .expect("`[binding].artifact_root` must be set");
    assert!(
        root.starts_with("build/"),
        "artifact_root must live under `build/`; got {root:?}"
    );

    let crate_name = bind
        .get("binding_crate")
        .and_then(|v| v.as_str())
        .expect("`[binding].binding_crate` must be set");
    assert!(
        root.contains(crate_name),
        "artifact_root {root:?} must contain the binding_crate name {crate_name:?}"
    );
}

#[test]
fn mambalibs_artifact_layout_expected_files_cover_metadata_lock_module_and_native() {
    let doc = load_toml(&manifest_path());
    let block = doc
        .get("expected_files")
        .and_then(|v| v.as_table())
        .expect("missing `[expected_files]` block");

    let paths: Vec<&str> = block
        .get("relative_paths")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    // The four artifact roles named by the issue body.
    for required_substr in &["metadata.json", "mamba.lock", "__init__.py", "_mambalibs_native"] {
        assert!(
            paths.iter().any(|p| p.contains(required_substr)),
            "`[expected_files].relative_paths` must include a path containing \
             `{required_substr}`; got {paths:?}"
        );
    }

    // None of the expected paths may escape the artifact root.
    for p in &paths {
        assert!(
            !p.starts_with('/'),
            "expected file paths must be relative (no leading `/`); got {p:?}"
        );
        assert!(
            !p.contains(".."),
            "expected file paths must not contain `..`; got {p:?}"
        );
    }

    let placeholder = block
        .get("shared_lib_extension_placeholder")
        .and_then(|v| v.as_str())
        .expect("`[expected_files].shared_lib_extension_placeholder` must be set");
    assert!(
        paths.iter().any(|p| p.contains(placeholder)),
        "at least one expected path must use the shared_lib_ext placeholder {placeholder:?}; \
         got {paths:?}"
    );

    for flag in &["must_all_be_non_directory", "must_all_be_under_artifact_root"] {
        assert_eq!(
            block.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[expected_files].{flag}` must be true"
        );
    }
}

#[test]
fn mambalibs_artifact_layout_platform_suffix_table_covers_three_platforms() {
    let doc = load_toml(&manifest_path());
    let block = doc.get("platform_suffix_table").and_then(|v| v.as_table()).expect(
        "missing `[platform_suffix_table]` block \
         (acceptance: \"Platform-specific suffix handling is documented \
         in the test.\")",
    );

    let macos = block
        .get("macos")
        .and_then(|v| v.as_str())
        .expect("`[platform_suffix_table].macos` must be set");
    let linux = block
        .get("linux")
        .and_then(|v| v.as_str())
        .expect("`[platform_suffix_table].linux` must be set");
    let windows = block
        .get("windows")
        .and_then(|v| v.as_str())
        .expect("`[platform_suffix_table].windows` must be set");

    assert!(macos.starts_with('.'), "macos suffix must start with `.`; got {macos:?}");
    assert!(linux.starts_with('.'), "linux suffix must start with `.`; got {linux:?}");
    assert!(windows.starts_with('.'), "windows suffix must start with `.`; got {windows:?}");

    assert_eq!(macos, ".dylib", "macOS shared-library suffix must be `.dylib`");
    assert_eq!(linux, ".so", "Linux shared-library suffix must be `.so`");
    assert_eq!(windows, ".dll", "Windows shared-library suffix must be `.dll`");

    // Suffixes must all be distinct — otherwise the runner can't
    // disambiguate the host platform from the artifact alone.
    assert_ne!(macos, linux, "macos and linux suffixes must differ");
    assert_ne!(macos, windows, "macos and windows suffixes must differ");
    assert_ne!(linux, windows, "linux and windows suffixes must differ");

    assert_eq!(
        block.get("must_be_deterministic_per_platform").and_then(|v| v.as_bool()),
        Some(true),
        "`[platform_suffix_table].must_be_deterministic_per_platform` must be true"
    );

    let docs = block
        .get("documentation")
        .and_then(|v| v.as_str())
        .expect("`[platform_suffix_table].documentation` must be set");
    for required in &["macOS", "Linux", "Windows"] {
        assert!(
            docs.contains(required),
            "documentation must name `{required}`; got {docs:?}"
        );
    }
}

#[test]
fn mambalibs_artifact_layout_present_case_succeeds_with_all_files() {
    let doc = load_toml(&manifest_path());
    let case = doc
        .get("present_layout_case")
        .and_then(|v| v.as_table())
        .expect("missing `[present_layout_case]` block");

    assert_eq!(
        case.get("case").and_then(|v| v.as_str()),
        Some("layout_present"),
        "`[present_layout_case].case` must be \"layout_present\""
    );
    assert_eq!(
        case.get("expected_outcome").and_then(|v| v.as_str()),
        Some("pass"),
        "`[present_layout_case].expected_outcome` must be \"pass\""
    );
    assert_eq!(
        case.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(0),
        "`[present_layout_case].expected_exit_code` must be 0"
    );
    assert_eq!(
        case.get("must_create_all_expected_files").and_then(|v| v.as_bool()),
        Some(true),
        "`[present_layout_case].must_create_all_expected_files` must be true"
    );
}

#[test]
fn mambalibs_artifact_layout_missing_file_case_names_exact_path() {
    let doc = load_toml(&manifest_path());
    let case = doc.get("missing_file_case").and_then(|v| v.as_table()).expect(
        "missing `[missing_file_case]` block \
         (acceptance: \"Missing artifact files fail with exact path names.\")",
    );

    assert_eq!(
        case.get("case").and_then(|v| v.as_str()),
        Some("missing_artifact_file"),
        "`[missing_file_case].case` must be \"missing_artifact_file\""
    );
    assert_eq!(
        case.get("expected_outcome").and_then(|v| v.as_str()),
        Some("fail"),
        "`[missing_file_case].expected_outcome` must be \"fail\""
    );
    assert_eq!(
        case.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(1),
        "`[missing_file_case].expected_exit_code` must be 1"
    );

    let omitted = case
        .get("omitted_relative_path")
        .and_then(|v| v.as_str())
        .expect("`[missing_file_case].omitted_relative_path` must be set");
    let named = case
        .get("diagnostic_must_name_missing_path_value")
        .and_then(|v| v.as_str())
        .expect("`[missing_file_case].diagnostic_must_name_missing_path_value` must be set");
    assert_eq!(
        omitted, named,
        "diagnostic must name the EXACT omitted path \
         (omitted_relative_path == diagnostic_must_name_missing_path_value)"
    );

    // The omitted path MUST be one of the expected paths,
    // otherwise the failure case is structurally incoherent.
    let expected: Vec<&str> = doc
        .get("expected_files")
        .and_then(|v| v.get("relative_paths"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        expected.contains(&omitted),
        "omitted_relative_path {omitted:?} must be listed in \
         `[expected_files].relative_paths`; got {expected:?}"
    );

    for flag in &["diagnostic_must_name_missing_path", "must_not_silently_succeed"] {
        assert_eq!(
            case.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[missing_file_case].{flag}` must be true"
        );
    }
}

#[test]
fn mambalibs_artifact_layout_diagnostic_contract_pins_missing_path_key() {
    let doc = load_toml(&manifest_path());
    let contract = doc
        .get("diagnostic_contract")
        .and_then(|v| v.as_table())
        .expect("missing `[diagnostic_contract]` block");

    for flag in &[
        "diagnostic_must_name_offending_artifact_root",
        "diagnostic_must_be_deterministic",
        "diagnostic_must_be_actionable",
    ] {
        assert_eq!(
            contract.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[diagnostic_contract].{flag}` must be true"
        );
    }

    let field_key = contract
        .get("diagnostic_must_name_missing_path_field_key")
        .and_then(|v| v.as_str())
        .expect("`diagnostic_must_name_missing_path_field_key` must be set");
    assert_eq!(field_key, "missing_path", "missing-path field key must be `missing_path`");

    let keys: Vec<&str> = doc
        .get("runner_contract")
        .and_then(|v| v.get("keys"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        keys.contains(&field_key),
        "`[runner_contract].keys` must include `{field_key}`; got {keys:?}"
    );
}

#[test]
fn mambalibs_artifact_layout_import_compatibility_preserves_2666() {
    let doc = load_toml(&manifest_path());
    let block = doc.get("import_compatibility").and_then(|v| v.as_table()).expect(
        "missing `[import_compatibility]` block \
         (acceptance: \"Import fixture can consume the same artifact layout.\")",
    );

    assert_eq!(
        block.get("type_roundtrip_fixture_issue").and_then(|v| v.as_integer()),
        Some(2666),
        "`[import_compatibility].type_roundtrip_fixture_issue` must record #2666"
    );
    for flag in &[
        "must_provide_module_init_for_import",
        "must_provide_native_extension_for_import",
        "must_not_require_optional_stub_for_import",
    ] {
        assert_eq!(
            block.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[import_compatibility].{flag}` must be true"
        );
    }

    let stub_path = block
        .get("optional_stub_relative_path")
        .and_then(|v| v.as_str())
        .expect("`optional_stub_relative_path` must be set");
    assert!(
        stub_path.ends_with(".pyi"),
        "optional stub path must end with `.pyi`; got {stub_path:?}"
    );

    // The optional stub MUST NOT be in the required expected_files
    // list (otherwise "optional" is a lie).
    let expected: Vec<&str> = doc
        .get("expected_files")
        .and_then(|v| v.get("relative_paths"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        !expected.contains(&stub_path),
        "optional stub {stub_path:?} must NOT be listed in [expected_files].relative_paths \
         (it is optional, not required); got {expected:?}"
    );
}

#[test]
fn mambalibs_artifact_layout_isolation_pins_no_global_state() {
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
fn mambalibs_artifact_layout_runner_contract_declares_keys_and_cases() {
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
        "artifact_root",
        "missing_path",
        "platform_suffix",
        "files_present",
        "files_missing",
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
    for required in &["layout_present", "missing_artifact_file"] {
        assert!(
            cases.contains(required),
            "`[runner_contract].case_values` must include `{required}`; got {cases:?}"
        );
    }
}

#[test]
fn mambalibs_artifact_layout_pins_out_of_scope_per_issue_2673() {
    let doc = load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("installer_ux_changes").and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].installer_ux_changes` must be true \
         (issue text: \"Out of scope: installer UX changes.\")"
    );
}
