//! Schema gate for the mambalibs text import gate fixture — closes
//! #2849.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("mambalibs")
        .join("fixtures")
        .join("text_import_gate")
        .join("manifest.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path).unwrap();
    raw.parse().unwrap()
}

const LIBRARY: &str = "text";
const ISSUE: i64 = 2849;
const FIXTURE: &str = "mambalibs_text_import_gate";
const STATUS_FIELD: &str = "text_import_status";
const SUPPORTED_CASE: &str = "text_import_supported";
const BLOCKED_CASE: &str = "text_import_blocked";
const OOS_FLAG: &str = "nlp_pipeline_coverage";

#[test]
fn header_is_well_formed() {
    let doc = load_toml(&manifest_path());
    assert_eq!(doc.get("fixture").and_then(|v| v.as_str()), Some(FIXTURE));
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(ISSUE));
    assert_eq!(doc.get("library").and_then(|v| v.as_str()), Some(LIBRARY));
    assert_eq!(doc.get("network").and_then(|v| v.as_str()), Some("offline"));
}

#[test]
fn binding_uses_documented_surface() {
    let doc = load_toml(&manifest_path());
    let bind = doc.get("binding").and_then(|v| v.as_table()).unwrap();
    let stmt = bind.get("import_statement").and_then(|v| v.as_str()).unwrap();
    assert!(stmt.contains("from mambalibs import") && stmt.contains(LIBRARY));
}

#[test]
fn support_status_enum_well_formed() {
    let doc = load_toml(&manifest_path());
    let block = doc.get("support_status").and_then(|v| v.as_table()).unwrap();
    let allowed: Vec<&str> = block.get("allowed_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for v in &["pass", "xfail", "blocker"] { assert!(allowed.contains(v)); }
}

#[test]
fn supported_case_asserts_minimal_symbol() {
    let doc = load_toml(&manifest_path());
    let case = doc.get("supported_case").and_then(|v| v.as_table()).unwrap();
    assert_eq!(case.get("case").and_then(|v| v.as_str()), Some(SUPPORTED_CASE));
    assert_eq!(case.get("expected_outcome").and_then(|v| v.as_str()), Some("pass"));
    let bind_sym = doc.get("binding").and_then(|v| v.get("minimal_exported_symbol"))
        .and_then(|v| v.as_str()).unwrap();
    assert_eq!(case.get("asserted_symbol").and_then(|v| v.as_str()), Some(bind_sym));
}

#[test]
fn blocked_case_links_to_tracker() {
    let doc = load_toml(&manifest_path());
    let case = doc.get("blocked_case").and_then(|v| v.as_table()).unwrap();
    assert_eq!(case.get("expected_outcome").and_then(|v| v.as_str()), Some("blocked"));
    assert_eq!(case.get("linked_blocker_issue").and_then(|v| v.as_integer()), Some(ISSUE));
    assert_eq!(case.get("must_name_offending_library").and_then(|v| v.as_str()), Some(LIBRARY));
}

#[test]
fn diagnostic_contract_names_library_and_surface() {
    let doc = load_toml(&manifest_path());
    let block = doc.get("diagnostic_contract").and_then(|v| v.as_table()).unwrap();
    assert_eq!(block.get("diagnostic_must_name_library").and_then(|v| v.as_str()), Some(LIBRARY));
    assert_eq!(block.get("diagnostic_must_name_surface").and_then(|v| v.as_str()), Some("mambalibs"));
}

#[test]
fn sample_policy_pins_tiny_deterministic_string() {
    let doc = load_toml(&manifest_path());
    let block = doc.get("sample_policy").and_then(|v| v.as_table()).expect(
        "[sample_policy] missing — acceptance: \
         \"Test uses a tiny deterministic string sample only if needed.\"",
    );
    for f in &[
        "must_be_tiny",
        "must_be_deterministic",
        "must_be_string",
        "must_be_in_memory",
        "forbid_network_io",
        "forbid_disk_io_outside_artifact_root",
        "forbid_random_without_fixed_seed",
    ] {
        assert_eq!(block.get(*f).and_then(|v| v.as_bool()), Some(true), "{f}");
    }
    let max_chars = block.get("max_sample_chars").and_then(|v| v.as_integer()).unwrap();
    assert!(max_chars > 0 && max_chars <= 64, "max_sample_chars must be tiny (1..=64), got {max_chars}");
}

#[test]
fn gate_summary_field_lives_in_runner_contract() {
    let doc = load_toml(&manifest_path());
    let block = doc.get("gate_summary_contract").and_then(|v| v.as_table()).unwrap();
    assert_eq!(block.get("field_name").and_then(|v| v.as_str()), Some(STATUS_FIELD));
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
        assert_eq!(i.get(*f).and_then(|v| v.as_bool()), Some(true));
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
    ] { assert!(keys.contains(r)); }
    let cases: Vec<&str> = c.get("case_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for r in &[SUPPORTED_CASE, BLOCKED_CASE] { assert!(cases.contains(r)); }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get(OOS_FLAG).and_then(|v| v.as_bool()), Some(true));
}
