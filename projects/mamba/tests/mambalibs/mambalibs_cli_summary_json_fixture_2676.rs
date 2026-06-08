//! Schema gate for the mambalibs CLI summary JSON fixture — closes
//! #2676.
//!
//! Acceptance (issue #2676):
//!
//!   1. JSON summary can be parsed without scraping logs.
//!      `[summary_emission]` pins `must_be_machine_parseable_json`,
//!      `must_be_emitted_on_dedicated_stream`, and a single-JSON-
//!      object emission shape; `summary_json` appears in
//!      `[runner_contract].keys`.
//!   2. Missing required fields fail the test.
//!      Every key in `[required_fields].keys` appears in
//!      `[success_case.summary]`. `[missing_field_case]` pins fail
//!      outcome + exit 1 and names the omitted field; the
//!      `omitted_required_field` MUST be one of the required keys
//!      so the case is meaningful.
//!   3. Failure entry includes issue or blocker reference when
//!      applicable. `[required_fields_on_failure].keys` includes
//!      `linked_issue_or_blocker` AND every such key appears in
//!      `[failure_case.summary]`.
//!
//! Cheap test — single TOML read + field walk. Runs in well under
//! a second; stays in the default `cargo test -p mamba` set.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("mambalibs")
        .join("fixtures")
        .join("cli_summary_json")
        .join("manifest.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("manifest {} unreadable: {e}", path.display()));
    raw.parse()
        .unwrap_or_else(|e| panic!("{} parse error: {e}", path.display()))
}

fn required_keys(doc: &toml::Value, block: &str) -> Vec<String> {
    doc.get(block)
        .and_then(|v| v.get("keys"))
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn mambalibs_cli_summary_json_manifest_header_is_well_formed() {
    let doc = load_toml(&manifest_path());

    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("mambalibs_cli_summary_json"),
        "`fixture` must be \"mambalibs_cli_summary_json\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2676),
        "`issue` must record #2676"
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
        Some("cli_summary_json"),
        "`family` must be \"cli_summary_json\""
    );
    assert_eq!(
        doc.get("network").and_then(|v| v.as_str()),
        Some("offline"),
        "`network` must be \"offline\""
    );
}

#[test]
fn mambalibs_cli_summary_emission_is_machine_parseable() {
    let doc = load_toml(&manifest_path());
    let block = doc.get("summary_emission").and_then(|v| v.as_table()).expect(
        "missing `[summary_emission]` block \
         (acceptance: \"JSON summary can be parsed without scraping logs.\")",
    );

    for flag in &[
        "must_be_machine_parseable_json",
        "must_be_emitted_on_dedicated_stream",
        "human_output_must_be_concise",
    ] {
        assert_eq!(
            block.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[summary_emission].{flag}` must be true"
        );
    }

    let emission = block
        .get("emission_stream")
        .and_then(|v| v.as_str())
        .expect("`[summary_emission].emission_stream` must be set");
    let human = block
        .get("human_output_stream")
        .and_then(|v| v.as_str())
        .expect("`[summary_emission].human_output_stream` must be set");
    assert_ne!(
        emission, human,
        "`[summary_emission].emission_stream` MUST differ from `human_output_stream` — \
         workers must be able to parse the summary without reading the human log"
    );

    let shape = block
        .get("emission_shape")
        .and_then(|v| v.as_str())
        .expect("`[summary_emission].emission_shape` must be set");
    assert_eq!(
        shape, "single_json_object",
        "`[summary_emission].emission_shape` must be \"single_json_object\""
    );

    // summary_json must be a first-class runner contract key.
    let contract_keys: Vec<&str> = doc
        .get("runner_contract")
        .and_then(|v| v.get("keys"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        contract_keys.contains(&"summary_json"),
        "`[runner_contract].keys` must include `summary_json`; got {contract_keys:?}"
    );
}

#[test]
fn mambalibs_cli_summary_required_fields_appear_in_success_summary() {
    let doc = load_toml(&manifest_path());
    let required = required_keys(&doc, "required_fields");
    assert!(
        !required.is_empty(),
        "`[required_fields].keys` must be non-empty"
    );
    for required_key in &["dependency_name", "build_status", "artifact_path", "import_status", "diagnostics", "outcome"] {
        assert!(
            required.iter().any(|k| k == required_key),
            "`[required_fields].keys` must include `{required_key}`; got {required:?}"
        );
    }

    let summary = doc
        .get("success_case")
        .and_then(|v| v.get("summary"))
        .and_then(|v| v.as_table())
        .expect("missing `[success_case.summary]` block");
    for key in &required {
        assert!(
            summary.contains_key(key),
            "`[success_case.summary]` MUST contain required field `{key}`; got keys \
             {:?}",
            summary.keys().collect::<Vec<_>>()
        );
    }

    assert_eq!(
        doc.get("success_case").and_then(|v| v.get("expected_outcome")).and_then(|v| v.as_str()),
        Some("pass"),
        "`[success_case].expected_outcome` must be \"pass\""
    );
    assert_eq!(
        doc.get("success_case").and_then(|v| v.get("expected_exit_code")).and_then(|v| v.as_integer()),
        Some(0),
        "`[success_case].expected_exit_code` must be 0"
    );
}

#[test]
fn mambalibs_cli_summary_required_fields_on_failure_appear_in_failure_summary() {
    let doc = load_toml(&manifest_path());
    let required_on_fail = required_keys(&doc, "required_fields_on_failure");
    assert!(
        required_on_fail.iter().any(|k| k == "linked_issue_or_blocker"),
        "`[required_fields_on_failure].keys` must include `linked_issue_or_blocker`; \
         got {required_on_fail:?}"
    );
    assert!(
        required_on_fail.iter().any(|k| k == "diagnostic_message"),
        "`[required_fields_on_failure].keys` must include `diagnostic_message`; got \
         {required_on_fail:?}"
    );

    let summary = doc
        .get("failure_case")
        .and_then(|v| v.get("summary"))
        .and_then(|v| v.as_table())
        .expect("missing `[failure_case.summary]` block");

    // ALL required-everywhere keys must appear in failure_case.summary too.
    let required_always = required_keys(&doc, "required_fields");
    for key in &required_always {
        assert!(
            summary.contains_key(key),
            "`[failure_case.summary]` MUST contain required field `{key}`; got keys \
             {:?}",
            summary.keys().collect::<Vec<_>>()
        );
    }
    // AND failure-only required keys.
    for key in &required_on_fail {
        assert!(
            summary.contains_key(key),
            "`[failure_case.summary]` MUST contain failure-required field `{key}`; got \
             keys {:?}",
            summary.keys().collect::<Vec<_>>()
        );
    }

    assert_eq!(
        doc.get("failure_case").and_then(|v| v.get("expected_outcome")).and_then(|v| v.as_str()),
        Some("fail"),
        "`[failure_case].expected_outcome` must be \"fail\""
    );
    assert_eq!(
        doc.get("failure_case").and_then(|v| v.get("expected_exit_code")).and_then(|v| v.as_integer()),
        Some(1),
        "`[failure_case].expected_exit_code` must be 1"
    );

    // The linked_issue_or_blocker value must be non-empty.
    let linked = summary
        .get("linked_issue_or_blocker")
        .and_then(|v| v.as_str())
        .expect("`[failure_case.summary].linked_issue_or_blocker` must be set as a string");
    assert!(
        !linked.is_empty(),
        "`[failure_case.summary].linked_issue_or_blocker` must be non-empty"
    );
}

#[test]
fn mambalibs_cli_summary_missing_field_case_targets_a_required_field() {
    let doc = load_toml(&manifest_path());
    let case = doc.get("missing_field_case").and_then(|v| v.as_table()).expect(
        "missing `[missing_field_case]` block \
         (acceptance: \"Missing required fields fail the test.\")",
    );

    assert_eq!(
        case.get("case").and_then(|v| v.as_str()),
        Some("summary_missing_required_field"),
        "`[missing_field_case].case` must be \"summary_missing_required_field\""
    );
    assert_eq!(
        case.get("expected_outcome").and_then(|v| v.as_str()),
        Some("fail"),
        "`[missing_field_case].expected_outcome` must be \"fail\""
    );
    assert_eq!(
        case.get("expected_exit_code").and_then(|v| v.as_integer()),
        Some(1),
        "`[missing_field_case].expected_exit_code` must be 1"
    );
    for flag in &[
        "diagnostic_must_name_missing_field",
        "must_not_silently_succeed",
        "must_be_deterministic",
    ] {
        assert_eq!(
            case.get(*flag).and_then(|v| v.as_bool()),
            Some(true),
            "`[missing_field_case].{flag}` must be true"
        );
    }

    // Cross-check: the omitted field MUST be one of the required
    // fields — otherwise the case has no bite.
    let omitted = case
        .get("omitted_required_field")
        .and_then(|v| v.as_str())
        .expect("`[missing_field_case].omitted_required_field` must be set");
    let required = required_keys(&doc, "required_fields");
    assert!(
        required.iter().any(|k| k == omitted),
        "`[missing_field_case].omitted_required_field` `{omitted}` MUST appear in \
         `[required_fields].keys`; got {required:?}"
    );

    // AND the diagnostic value must match the omitted field exactly.
    assert_eq!(
        case.get("diagnostic_must_name_missing_field_value").and_then(|v| v.as_str()),
        Some(omitted),
        "`[missing_field_case].diagnostic_must_name_missing_field_value` must equal \
         `omitted_required_field`"
    );
}

#[test]
fn mambalibs_cli_summary_diagnostic_contract_pins_field_keys() {
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

    let missing_key = block
        .get("diagnostic_must_name_missing_field_field_key")
        .and_then(|v| v.as_str())
        .expect("`[diagnostic_contract].diagnostic_must_name_missing_field_field_key` must be set");
    let linked_key = block
        .get("linked_issue_or_blocker_field_key")
        .and_then(|v| v.as_str())
        .expect("`[diagnostic_contract].linked_issue_or_blocker_field_key` must be set");

    let contract_keys: Vec<&str> = doc
        .get("runner_contract")
        .and_then(|v| v.get("keys"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        contract_keys.contains(&missing_key),
        "`[runner_contract].keys` must include `{missing_key}` (diagnostic_must_name_missing_field_field_key); \
         got {contract_keys:?}"
    );
    assert!(
        contract_keys.contains(&linked_key),
        "`[runner_contract].keys` must include `{linked_key}` (linked_issue_or_blocker_field_key); \
         got {contract_keys:?}"
    );
}

#[test]
fn mambalibs_cli_summary_isolation_pins_no_global_state() {
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
fn mambalibs_cli_summary_runner_contract_declares_keys_and_cases() {
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
        "summary_json",
        "summary_parse_status",
        "missing_field",
        "linked_issue_or_blocker",
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
    for required in &[
        "summary_success",
        "summary_failure",
        "summary_missing_required_field",
    ] {
        assert!(
            cases.contains(required),
            "`[runner_contract].case_values` must include `{required}`; got {cases:?}"
        );
    }
}

#[test]
fn mambalibs_cli_summary_pins_out_of_scope_per_issue_2676() {
    let doc = load_toml(&manifest_path());
    let oos = doc
        .get("out_of_scope")
        .and_then(|v| v.as_table())
        .expect("missing `[out_of_scope]` block");
    assert_eq!(
        oos.get("dashboard_or_reporting_ui").and_then(|v| v.as_bool()),
        Some(true),
        "`[out_of_scope].dashboard_or_reporting_ui` must be true \
         (issue text: \"Out of scope: dashboard or reporting UI.\")"
    );
}
