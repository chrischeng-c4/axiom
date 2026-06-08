//! Schema gate for the mambalibs exported-type roundtrip fixture —
//! closes #2666.
//!
//! Acceptance (issue #2666):
//!
//!   1. Fixture fails on wrong type conversion or return value.
//!      `[wrong_type_case].expected_outcome == "fail"` and
//!      `[wrong_return_value_case].expected_outcome == "fail"`.
//!   2. Failure output names the exported function.
//!      `[wrong_type_case].diagnostic_must_name_function == true`,
//!      same for the wrong-return-value guard; `[exported_function]
//!      .diagnostic_must_name_value` pins the function name; the
//!      `runner_contract` carries `exported_function` as a key.
//!   3. The fixture remains fast and local.
//!      `[fixture_performance]` pins the three locality flags plus a
//!      hard upper bound on wall time; `[isolation]` forbids global
//!      state.
//!
//! Cheap test — single TOML read + field walk. Runs in well under a
//! second; stays in the default `cargo test -p mamba` set.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("mambalibs")
        .join("fixtures")
        .join("type_roundtrip")
        .join("manifest.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("manifest {} unreadable: {e}", path.display()));
    raw.parse()
        .unwrap_or_else(|e| panic!("{} parse error: {e}", path.display()))
}

#[test]
fn mambalibs_type_roundtrip_manifest_header_is_well_formed() {
    let doc = load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("mambalibs_type_roundtrip"),
        "`fixture` must be \"mambalibs_type_roundtrip\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2666),
        "`issue` must record #2666"
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
        Some("type_roundtrip"),
        "`family` must be \"type_roundtrip\" (matches profile manifest)"
    );
    assert_eq!(
        doc.get("network").and_then(|v| v.as_str()),
        Some("offline"),
        "`network` must be \"offline\" (fixture is local-only)"
    );
}

#[test]
fn mambalibs_type_roundtrip_exported_function_block_is_complete() {
    let doc = load_toml(&manifest_path());
    let func = doc
        .get("exported_function")
        .and_then(|v| v.as_table())
        .expect("missing `[exported_function]` block");

    let name = func
        .get("name")
        .and_then(|v| v.as_str())
        .expect("`[exported_function].name` must be set");
    assert!(
        !name.is_empty(),
        "`[exported_function].name` must be non-empty"
    );

    let import_stmt = func
        .get("import_statement")
        .and_then(|v| v.as_str())
        .expect("`[exported_function].import_statement` must be set");
    assert!(
        import_stmt.contains("from mambalibs import"),
        "import statement must use `from mambalibs import ...` shape; got {import_stmt:?}"
    );
    assert!(
        import_stmt.contains(name),
        "import statement {import_stmt:?} must reference the exported function {name:?}"
    );

    // The failure-naming acceptance hinges on the diagnostic naming
    // this exact function — pin it at the schema level so a future
    // edit can't sever the link.
    assert_eq!(
        func.get("diagnostic_must_name_value").and_then(|v| v.as_str()),
        Some(name),
        "`[exported_function].diagnostic_must_name_value` must equal `[exported_function].name`"
    );

    for required in &["accepts_kinds", "returns_kinds"] {
        let kinds: Vec<&str> = func
            .get(*required)
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();
        for kind in &["str", "int", "bool"] {
            assert!(
                kinds.contains(kind),
                "`[exported_function].{required}` must include `{kind}`; got {kinds:?}"
            );
        }
    }
}

#[test]
fn mambalibs_type_roundtrip_pins_one_pass_case_per_primitive() {
    let doc = load_toml(&manifest_path());

    let cases = [
        ("string_roundtrip_case", "string_roundtrip", "str"),
        ("int_roundtrip_case", "int_roundtrip", "int"),
        ("bool_roundtrip_case", "bool_roundtrip", "bool"),
    ];

    for (block_name, case_name, kind) in cases {
        let block = doc
            .get(block_name)
            .and_then(|v| v.as_table())
            .unwrap_or_else(|| panic!("missing `[{block_name}]` block"));

        assert_eq!(
            block.get("case").and_then(|v| v.as_str()),
            Some(case_name),
            "`[{block_name}].case` must be {case_name:?}"
        );
        assert_eq!(
            block.get("input_kind").and_then(|v| v.as_str()),
            Some(kind),
            "`[{block_name}].input_kind` must be {kind:?}"
        );
        assert_eq!(
            block.get("expected_outcome").and_then(|v| v.as_str()),
            Some("pass"),
            "`[{block_name}].expected_outcome` must be \"pass\""
        );
        assert_eq!(
            block.get("expected_return_type").and_then(|v| v.as_str()),
            Some(kind),
            "`[{block_name}].expected_return_type` must be {kind:?} \
             (roundtrip preserves the Python type)"
        );
        assert_eq!(
            block.get("roundtrip_is_symmetric").and_then(|v| v.as_bool()),
            Some(true),
            "`[{block_name}].roundtrip_is_symmetric` must be true"
        );

        let input = block
            .get("input_value")
            .unwrap_or_else(|| panic!("`[{block_name}].input_value` must be set"));
        let output = block
            .get("expected_return_value")
            .unwrap_or_else(|| panic!("`[{block_name}].expected_return_value` must be set"));
        assert_eq!(
            input, output,
            "`[{block_name}]` must roundtrip the value unchanged \
             (input == expected_return_value)"
        );
    }
}

#[test]
fn mambalibs_type_roundtrip_wrong_type_case_fails_and_names_function() {
    let doc = load_toml(&manifest_path());
    let block = doc.get("wrong_type_case").and_then(|v| v.as_table()).expect(
        "missing `[wrong_type_case]` block \
         (acceptance: \"Fixture fails on wrong type conversion ...\")",
    );

    assert_eq!(
        block.get("case").and_then(|v| v.as_str()),
        Some("wrong_type"),
        "`[wrong_type_case].case` must be \"wrong_type\""
    );
    assert_eq!(
        block.get("expected_outcome").and_then(|v| v.as_str()),
        Some("fail"),
        "`[wrong_type_case].expected_outcome` must be \"fail\""
    );

    let input_kind = block
        .get("input_kind")
        .and_then(|v| v.as_str())
        .expect("`[wrong_type_case].input_kind` must be set");
    let expected_kind = block
        .get("expected_kind")
        .and_then(|v| v.as_str())
        .expect("`[wrong_type_case].expected_kind` must be set");
    assert_ne!(
        input_kind, expected_kind,
        "wrong-type case must declare DIFFERENT input vs expected kinds"
    );

    for flag in &[
        "diagnostic_must_name_function",
        "diagnostic_must_name_expected_kind",
        "diagnostic_must_name_actual_kind",
    ] {
        assert_eq!(
            block.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[wrong_type_case].{flag}` must be true \
             (acceptance: \"Failure output names the exported function.\")"
        );
    }
}

#[test]
fn mambalibs_type_roundtrip_wrong_return_value_case_fails_and_names_function() {
    let doc = load_toml(&manifest_path());
    let block = doc.get("wrong_return_value_case").and_then(|v| v.as_table()).expect(
        "missing `[wrong_return_value_case]` block \
         (acceptance: \"Fixture fails on ... wrong return value.\")",
    );

    assert_eq!(
        block.get("case").and_then(|v| v.as_str()),
        Some("wrong_return_value"),
        "`[wrong_return_value_case].case` must be \"wrong_return_value\""
    );
    assert_eq!(
        block.get("expected_outcome").and_then(|v| v.as_str()),
        Some("fail"),
        "`[wrong_return_value_case].expected_outcome` must be \"fail\""
    );

    let input = block
        .get("input_value")
        .and_then(|v| v.as_str())
        .expect("`[wrong_return_value_case].input_value` must be a string");
    let pinned = block
        .get("runner_pins_return_value")
        .and_then(|v| v.as_str())
        .expect("`[wrong_return_value_case].runner_pins_return_value` must be a string");
    assert_ne!(
        input, pinned,
        "wrong-return-value case must pin a return value DIFFERENT from the input"
    );

    for flag in &[
        "diagnostic_must_name_function",
        "diagnostic_must_name_expected_value",
        "diagnostic_must_name_actual_value",
    ] {
        assert_eq!(
            block.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[wrong_return_value_case].{flag}` must be true \
             (acceptance: \"Failure output names the exported function.\")"
        );
    }
}

#[test]
fn mambalibs_type_roundtrip_fixture_performance_pins_fast_and_local() {
    let doc = load_toml(&manifest_path());
    let perf = doc.get("fixture_performance").and_then(|v| v.as_table()).expect(
        "missing `[fixture_performance]` block \
         (acceptance: \"The fixture remains fast and local.\")",
    );

    for flag in &[
        "must_be_fast_and_local",
        "must_not_touch_network",
        "must_run_in_process",
        "must_not_spawn_subprocesses",
    ] {
        assert_eq!(
            perf.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[fixture_performance].{flag}` must be true"
        );
    }

    let bound = perf
        .get("must_complete_under_seconds")
        .and_then(|v| v.as_integer())
        .expect("`[fixture_performance].must_complete_under_seconds` must be an integer");
    assert!(
        bound > 0 && bound <= 30,
        "`[fixture_performance].must_complete_under_seconds` must be in (0, 30]; \
         got {bound} — fixture must remain fast"
    );
}

#[test]
fn mambalibs_type_roundtrip_isolation_pins_no_global_state() {
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
fn mambalibs_type_roundtrip_runner_contract_declares_all_keys_and_cases() {
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
        "exported_function",
        "input_kind",
        "input_value",
        "return_value",
        "return_type",
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
    for required in &["pass", "fail", "missing", "skip"] {
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
    for required in &[
        "string_roundtrip",
        "int_roundtrip",
        "bool_roundtrip",
        "wrong_type",
        "wrong_return_value",
    ] {
        assert!(
            cases.contains(required),
            "`[runner_contract].case_values` must include `{required}`; got {cases:?}"
        );
    }
}

#[test]
fn mambalibs_type_roundtrip_pins_out_of_scope_per_issue_2666() {
    let doc = load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("complex_object_ownership_or_async_bindings").and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].complex_object_ownership_or_async_bindings` must be true \
         (issue text: \"Out of scope: complex object ownership or async bindings.\")"
    );
}

#[test]
fn mambalibs_type_roundtrip_family_matches_profile_manifest() {
    // Cross-block: the family declared by this fixture must match the
    // family the mambalibs profile manifest points its source at.
    let fixture = load_toml(&manifest_path());
    let fixture_family = fixture
        .get("family")
        .and_then(|v| v.as_str())
        .expect("`family` must be set");

    let profile_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("validation")
        .join("profiles")
        .join("mambalibs.toml");
    let profile = load_toml(&profile_path);
    let family_block = profile
        .get("families")
        .and_then(|v| v.get(fixture_family))
        .and_then(|v| v.as_table())
        .unwrap_or_else(|| {
            panic!(
                "profile manifest is missing `[families.{fixture_family}]` \
                 — fixture family must be declared by the profile"
            )
        });

    let source = family_block
        .get("source")
        .and_then(|v| v.as_str())
        .expect("`[families.<family>].source` must be set in the profile");
    assert!(
        source.ends_with("mambalibs/fixtures/type_roundtrip"),
        "profile family `source` must point at the fixture directory; \
         got {source:?}"
    );

    let outcome_rule = family_block
        .get("outcome_rule")
        .and_then(|v| v.as_str())
        .expect("`[families.<family>].outcome_rule` must be set");
    assert_eq!(
        outcome_rule, "must_roundtrip",
        "profile family `outcome_rule` must be `must_roundtrip`; got {outcome_rule:?}"
    );
}
