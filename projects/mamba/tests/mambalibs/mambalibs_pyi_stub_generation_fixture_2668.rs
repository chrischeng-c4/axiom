//! Schema gate for the mambalibs .pyi stub generation fixture —
//! closes #2668.
//!
//! Acceptance (issue #2668):
//!
//!   1. Stub output includes the exported module and function
//!      signature.
//!      `[stub_generation_case]` pins must_include_module_name,
//!      must_include_exported_function, must_include_param_kinds,
//!      must_include_return_kind.
//!   2. Missing stub generation is reported with a linked blocker,
//!      not silent pass.
//!      `[missing_stub_generator_case].expected_outcome == "blocked"`,
//!      `forbid_silent_pass == true`, `linked_blocker_issue == 2668`;
//!      `[unsupported_behavior_contract]` enforces the same.
//!   3. Import fixture still works independently.
//!      `[import_independence]` pins the cross-fixture invariant
//!      against #2666 type-roundtrip.
//!
//! Cheap test — single TOML read + field walk. Runs in well under a
//! second; stays in the default `cargo test -p mamba` set.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("mambalibs")
        .join("fixtures")
        .join("pyi_stub_generation")
        .join("manifest.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("manifest {} unreadable: {e}", path.display()));
    raw.parse()
        .unwrap_or_else(|e| panic!("{} parse error: {e}", path.display()))
}

#[test]
fn mambalibs_pyi_stub_manifest_header_is_well_formed() {
    let doc = load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("mambalibs_pyi_stub_generation"),
        "`fixture` must be \"mambalibs_pyi_stub_generation\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2668),
        "`issue` must record #2668"
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
        Some("pyi_stub_generation"),
        "`family` must be \"pyi_stub_generation\""
    );
    assert_eq!(
        doc.get("network").and_then(|v| v.as_str()),
        Some("offline"),
        "`network` must be \"offline\" (fixture is local-only)"
    );
}

#[test]
fn mambalibs_pyi_stub_binding_block_pins_signature_components() {
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

    let sig = bind
        .get("function_signature")
        .and_then(|v| v.as_str())
        .expect("`[binding].function_signature` must be set");
    assert!(
        sig.contains(exported),
        "function signature {sig:?} must reference the exported function {exported:?}"
    );

    let params: Vec<&str> = bind
        .get("param_kinds")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for kind in &["str", "int", "bool"] {
        assert!(
            params.contains(kind),
            "`[binding].param_kinds` must include `{kind}`; got {params:?}"
        );
    }

    let ret = bind
        .get("return_kind")
        .and_then(|v| v.as_str())
        .expect("`[binding].return_kind` must be set");
    assert!(
        !ret.is_empty(),
        "`[binding].return_kind` must be non-empty"
    );
}

#[test]
fn mambalibs_pyi_stub_artifact_block_pins_deterministic_pyi() {
    let doc = load_toml(&manifest_path());
    let artifact = doc
        .get("stub_artifact")
        .and_then(|v| v.as_table())
        .expect("missing `[stub_artifact]` block");

    let filename = artifact
        .get("filename")
        .and_then(|v| v.as_str())
        .expect("`[stub_artifact].filename` must be set");
    assert!(
        filename.ends_with(".pyi"),
        "stub artifact filename must end with `.pyi`; got {filename:?}"
    );

    assert_eq!(
        artifact.get("stub_kind").and_then(|v| v.as_str()),
        Some("pyi"),
        "`[stub_artifact].stub_kind` must be \"pyi\""
    );
    for flag in &[
        "must_be_deterministic",
        "must_be_byte_identical_on_replay",
        "ships_alongside_wheel",
    ] {
        assert_eq!(
            artifact.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[stub_artifact].{flag}` must be true"
        );
    }
}

#[test]
fn mambalibs_pyi_stub_generation_case_pins_module_and_signature() {
    let doc = load_toml(&manifest_path());
    let block = doc.get("stub_generation_case").and_then(|v| v.as_table()).expect(
        "missing `[stub_generation_case]` block \
         (acceptance: \"Stub output includes the exported module and \
         function signature.\")",
    );

    assert_eq!(
        block.get("case").and_then(|v| v.as_str()),
        Some("stub_emitted"),
        "`[stub_generation_case].case` must be \"stub_emitted\""
    );
    assert_eq!(
        block.get("expected_outcome").and_then(|v| v.as_str()),
        Some("pass"),
        "`[stub_generation_case].expected_outcome` must be \"pass\""
    );
    assert_eq!(
        block.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(0),
        "`[stub_generation_case].expected_exit_code` must be 0"
    );
    assert_eq!(
        block.get("must_create_stub_artifact").and_then(|v| v.as_bool()),
        Some(true),
        "`[stub_generation_case].must_create_stub_artifact` must be true"
    );

    let module_in_stub = block
        .get("must_include_module_name")
        .and_then(|v| v.as_str())
        .expect("`must_include_module_name` must be set");
    let binding_module = doc
        .get("binding")
        .and_then(|v| v.get("module_name"))
        .and_then(|v| v.as_str())
        .expect("`[binding].module_name` must be set");
    assert_eq!(
        module_in_stub, binding_module,
        "stub case must include `[binding].module_name` ({binding_module:?})"
    );

    let func_in_stub = block
        .get("must_include_exported_function")
        .and_then(|v| v.as_str())
        .expect("`must_include_exported_function` must be set");
    let binding_func = doc
        .get("binding")
        .and_then(|v| v.get("exported_function"))
        .and_then(|v| v.as_str())
        .expect("`[binding].exported_function` must be set");
    assert_eq!(
        func_in_stub, binding_func,
        "stub case must include `[binding].exported_function` ({binding_func:?})"
    );

    assert_eq!(
        block.get("must_include_function_signature").and_then(|v| v.as_bool()),
        Some(true),
        "`[stub_generation_case].must_include_function_signature` must be true"
    );

    let stub_param_kinds: Vec<&str> = block
        .get("must_include_param_kinds")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    let binding_param_kinds: Vec<&str> = doc
        .get("binding")
        .and_then(|v| v.get("param_kinds"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for kind in &binding_param_kinds {
        assert!(
            stub_param_kinds.contains(kind),
            "stub case `must_include_param_kinds` ({stub_param_kinds:?}) must cover \
             `[binding].param_kinds` ({binding_param_kinds:?}), missing {kind:?}"
        );
    }

    let stub_return = block
        .get("must_include_return_kind")
        .and_then(|v| v.as_str())
        .expect("`must_include_return_kind` must be set");
    let binding_return = doc
        .get("binding")
        .and_then(|v| v.get("return_kind"))
        .and_then(|v| v.as_str())
        .expect("`[binding].return_kind` must be set");
    assert_eq!(
        stub_return, binding_return,
        "stub case `must_include_return_kind` must equal `[binding].return_kind` \
         ({binding_return:?})"
    );
}

#[test]
fn mambalibs_pyi_missing_stub_generator_case_blocks_with_linked_issue() {
    let doc = load_toml(&manifest_path());
    let block = doc.get("missing_stub_generator_case").and_then(|v| v.as_table()).expect(
        "missing `[missing_stub_generator_case]` block \
         (acceptance: \"Missing stub generation is reported with a \
         linked blocker, not silent pass.\")",
    );

    assert_eq!(
        block.get("case").and_then(|v| v.as_str()),
        Some("stub_generator_missing"),
        "`[missing_stub_generator_case].case` must be \"stub_generator_missing\""
    );
    assert_eq!(
        block.get("expected_outcome").and_then(|v| v.as_str()),
        Some("blocked"),
        "`[missing_stub_generator_case].expected_outcome` must be \"blocked\" \
         (not \"pass\" — silent pass is forbidden)"
    );

    for flag in &[
        "forbid_silent_pass",
        "must_not_create_stub_artifact",
        "must_report_missing_generator",
        "diagnostic_must_name_blocker_issue",
    ] {
        assert_eq!(
            block.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[missing_stub_generator_case].{flag}` must be true"
        );
    }

    assert_eq!(
        block.get("linked_blocker_issue").and_then(|v| v.as_integer()),
        Some(2668),
        "`[missing_stub_generator_case].linked_blocker_issue` must record #2668"
    );

    assert_eq!(
        block.get("diagnostic_must_name_offending_feature").and_then(|v| v.as_str()),
        Some("stub_generation"),
        "`[missing_stub_generator_case].diagnostic_must_name_offending_feature` \
         must be \"stub_generation\""
    );
}

#[test]
fn mambalibs_pyi_unsupported_behavior_contract_forbids_silent_pass() {
    let doc = load_toml(&manifest_path());
    let contract = doc.get("unsupported_behavior_contract").and_then(|v| v.as_table()).expect(
        "missing `[unsupported_behavior_contract]` block",
    );

    assert_eq!(
        contract.get("when_stub_generation_unsupported_outcome").and_then(|v| v.as_str()),
        Some("blocked"),
        "`when_stub_generation_unsupported_outcome` must be \"blocked\""
    );
    for flag in &["forbid_silent_pass", "must_distinguish_blocked_from_fail"] {
        assert_eq!(
            contract.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[unsupported_behavior_contract].{flag}` must be true"
        );
    }
    assert_eq!(
        contract.get("must_link_to_blocker_issue").and_then(|v| v.as_integer()),
        Some(2668),
        "`[unsupported_behavior_contract].must_link_to_blocker_issue` must record #2668"
    );
}

#[test]
fn mambalibs_pyi_import_independence_preserves_2666() {
    let doc = load_toml(&manifest_path());
    let block = doc.get("import_independence").and_then(|v| v.as_table()).expect(
        "missing `[import_independence]` block \
         (acceptance: \"Import fixture still works independently.\")",
    );

    assert_eq!(
        block.get("type_roundtrip_fixture_issue").and_then(|v| v.as_integer()),
        Some(2666),
        "`[import_independence].type_roundtrip_fixture_issue` must record #2666"
    );
    for flag in &[
        "type_roundtrip_fixture_must_remain_green",
        "disabling_stub_generation_must_not_break_import",
        "stub_artifact_is_optional_for_import",
    ] {
        assert_eq!(
            block.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[import_independence].{flag}` must be true"
        );
    }
}

#[test]
fn mambalibs_pyi_isolation_pins_no_global_state() {
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
fn mambalibs_pyi_runner_contract_declares_blocked_outcome_and_cases() {
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
        "stub_artifact_path",
        "stub_artifact_present",
        "blocker_issue",
        "diagnostic_message",
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
    for required in &["pass", "fail", "blocked"] {
        assert!(
            outcomes.contains(required),
            "`[runner_contract].outcome_values` must include `{required}`; got {outcomes:?}"
        );
    }

    let cases: Vec<&str> = contract
        .get("case_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &["stub_emitted", "stub_generator_missing"] {
        assert!(
            cases.contains(required),
            "`[runner_contract].case_values` must include `{required}`; got {cases:?}"
        );
    }
}

#[test]
fn mambalibs_pyi_pins_out_of_scope_per_issue_2668() {
    let doc = load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("full_typing_coverage_for_every_library").and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].full_typing_coverage_for_every_library` must be true \
         (issue text: \"Out of scope: full typing coverage for every library.\")"
    );
}
