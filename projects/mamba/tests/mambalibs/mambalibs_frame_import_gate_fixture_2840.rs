//! Schema gate for the mambalibs frame import gate fixture —
//! closes #2840.
//!
//! Acceptance (issue #2840):
//!
//!   1. Import success is asserted or unsupported status is
//!      linked to a blocker.
//!   2. Failure output names frame and mambalibs.
//!   3. Test uses tiny in-memory data only if needed.
//!
//! Cheap test — single TOML read + field walk. Stays in the
//! default `cargo test -p mamba` set.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("mambalibs")
        .join("fixtures")
        .join("frame_import_gate")
        .join("manifest.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("manifest {} unreadable: {e}", path.display()));
    raw.parse()
        .unwrap_or_else(|e| panic!("{} parse error: {e}", path.display()))
}

const LIBRARY: &str = "frame";
const ISSUE: i64 = 2840;
const FIXTURE: &str = "mambalibs_frame_import_gate";
const FAMILY: &str = "frame_import_gate";
const STATUS_FIELD: &str = "frame_import_status";
const SUPPORTED_CASE: &str = "frame_import_supported";
const BLOCKED_CASE: &str = "frame_import_blocked";
const OOS_FLAG: &str = "dataframe_operation_coverage";

#[test]
fn header_is_well_formed() {
    let doc = load_toml(&manifest_path());
    assert_eq!(doc.get("fixture").and_then(|v| v.as_str()), Some(FIXTURE));
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(ISSUE));
    assert_eq!(doc.get("parent_issue").and_then(|v| v.as_integer()), Some(2531));
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("mambalibs"));
    assert_eq!(doc.get("family").and_then(|v| v.as_str()), Some(FAMILY));
    assert_eq!(doc.get("library").and_then(|v| v.as_str()), Some(LIBRARY));
    assert_eq!(doc.get("network").and_then(|v| v.as_str()), Some("offline"));
}

#[test]
fn binding_uses_documented_surface() {
    let doc = load_toml(&manifest_path());
    let bind = doc.get("binding").and_then(|v| v.as_table()).unwrap();
    assert_eq!(bind.get("surface").and_then(|v| v.as_str()), Some("mambalibs"));
    assert_eq!(bind.get("library").and_then(|v| v.as_str()), Some(LIBRARY));
    let stmt = bind.get("import_statement").and_then(|v| v.as_str()).unwrap();
    assert!(stmt.contains("from mambalibs import"));
    assert!(stmt.contains(LIBRARY));
    let sym = bind.get("minimal_exported_symbol").and_then(|v| v.as_str()).unwrap();
    assert!(!sym.is_empty());
}

#[test]
fn support_status_enum_well_formed() {
    let doc = load_toml(&manifest_path());
    let block = doc.get("support_status").and_then(|v| v.as_table()).unwrap();
    let allowed: Vec<&str> = block
        .get("allowed_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for v in &["pass", "xfail", "blocker"] {
        assert!(allowed.contains(v), "allowed_values must include {v}");
    }
    let current = block.get("current_status").and_then(|v| v.as_str()).unwrap();
    let default = block.get("default_status").and_then(|v| v.as_str()).unwrap();
    assert!(allowed.contains(&current) && allowed.contains(&default));
}

#[test]
fn supported_case_asserts_minimal_symbol() {
    let doc = load_toml(&manifest_path());
    let case = doc.get("supported_case").and_then(|v| v.as_table()).unwrap();
    assert_eq!(case.get("case").and_then(|v| v.as_str()), Some(SUPPORTED_CASE));
    assert_eq!(case.get("status_under_which_applicable").and_then(|v| v.as_str()), Some("pass"));
    assert_eq!(case.get("expected_outcome").and_then(|v| v.as_str()), Some("pass"));
    assert_eq!(case.get("expected_exit_code").and_then(|v| v.as_integer()), Some(0));
    for f in &["must_import_module", "must_assert_minimal_symbol"] {
        assert_eq!(case.get(*f).and_then(|v| v.as_bool()), Some(true), "{f}");
    }
    let bind_sym = doc.get("binding").and_then(|v| v.get("minimal_exported_symbol"))
        .and_then(|v| v.as_str()).unwrap();
    assert_eq!(case.get("asserted_symbol").and_then(|v| v.as_str()), Some(bind_sym));
}

#[test]
fn blocked_case_links_to_tracker() {
    let doc = load_toml(&manifest_path());
    let case = doc.get("blocked_case").and_then(|v| v.as_table()).unwrap();
    assert_eq!(case.get("case").and_then(|v| v.as_str()), Some(BLOCKED_CASE));
    assert_eq!(case.get("status_under_which_applicable").and_then(|v| v.as_str()), Some("blocker"));
    assert_eq!(case.get("expected_outcome").and_then(|v| v.as_str()), Some("blocked"));
    assert_eq!(case.get("linked_blocker_issue").and_then(|v| v.as_integer()), Some(ISSUE));
    for f in &["must_emit_structured_blocker", "must_not_attempt_import"] {
        assert_eq!(case.get(*f).and_then(|v| v.as_bool()), Some(true), "{f}");
    }
    assert_eq!(case.get("must_name_offending_library").and_then(|v| v.as_str()), Some(LIBRARY));
    assert_eq!(case.get("must_name_surface").and_then(|v| v.as_str()), Some("mambalibs"));

    let outcomes: Vec<&str> = doc.get("runner_contract").and_then(|v| v.get("outcome_values"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    assert!(outcomes.contains(&"blocked"));
}

#[test]
fn diagnostic_contract_names_library_and_surface() {
    let doc = load_toml(&manifest_path());
    let block = doc.get("diagnostic_contract").and_then(|v| v.as_table()).unwrap();
    assert_eq!(block.get("diagnostic_must_name_library").and_then(|v| v.as_str()), Some(LIBRARY));
    assert_eq!(block.get("diagnostic_must_name_surface").and_then(|v| v.as_str()), Some("mambalibs"));
    let contract_keys: Vec<&str> = doc.get("runner_contract").and_then(|v| v.get("keys"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for key_field in &[
        "diagnostic_must_name_library_field_key",
        "diagnostic_must_name_surface_field_key",
        "diagnostic_must_name_linked_blocker_field_key",
    ] {
        let k = block.get(*key_field).and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("{key_field} must be set"));
        assert!(contract_keys.contains(&k), "runner_contract.keys must include {k}");
    }
}

#[test]
fn sample_policy_pins_tiny_in_memory() {
    let doc = load_toml(&manifest_path());
    let block = doc.get("sample_policy").and_then(|v| v.as_table()).unwrap();
    for f in &[
        "must_be_tiny",
        "must_be_in_memory",
        "must_be_deterministic",
        "forbid_network_io",
        "forbid_disk_io_outside_artifact_root",
    ] {
        assert_eq!(block.get(*f).and_then(|v| v.as_bool()), Some(true), "{f}");
    }
    let max_bytes = block.get("max_sample_bytes").and_then(|v| v.as_integer()).unwrap();
    assert!(max_bytes > 0 && max_bytes <= 1024, "max_sample_bytes must be ≤1024; got {max_bytes}");
}

#[test]
fn gate_summary_field_lives_in_runner_contract() {
    let doc = load_toml(&manifest_path());
    let block = doc.get("gate_summary_contract").and_then(|v| v.as_table()).unwrap();
    assert_eq!(block.get("field_name").and_then(|v| v.as_str()), Some(STATUS_FIELD));
    let contract_keys: Vec<&str> = doc.get("runner_contract").and_then(|v| v.get("keys"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    assert!(contract_keys.contains(&STATUS_FIELD));

    let field_allowed: Vec<&str> = block.get("allowed_field_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    let support_allowed: Vec<&str> = doc.get("support_status").and_then(|v| v.get("allowed_values"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for v in &support_allowed {
        assert!(field_allowed.contains(v), "field allowed must include {v}");
    }
}

#[test]
fn isolation_pins_no_global_state() {
    let doc = load_toml(&manifest_path());
    let i = doc.get("isolation").and_then(|v| v.as_table()).unwrap();
    for f in &[
        "forbid_writes_outside_project",
        "forbid_user_home_reads",
        "forbid_global_cache_reads",
        "forbid_global_cache_writes",
    ] {
        assert_eq!(i.get(*f).and_then(|v| v.as_bool()), Some(true), "{f}");
    }
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let doc = load_toml(&manifest_path());
    let c = doc.get("runner_contract").and_then(|v| v.as_table()).unwrap();
    let keys: Vec<&str> = c.get("keys").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for r in &[
        "outcome", "case", "surface", "library", "import_statement",
        "asserted_symbol", STATUS_FIELD, "linked_blocker_issue",
        "diagnostic_message", "exit_code",
    ] {
        assert!(keys.contains(r), "runner_contract.keys must include {r}");
    }
    let cases: Vec<&str> = c.get("case_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for r in &[SUPPORTED_CASE, BLOCKED_CASE] {
        assert!(cases.contains(r), "case_values must include {r}");
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get(OOS_FLAG).and_then(|v| v.as_bool()), Some(true), "{OOS_FLAG} must be true");
}
