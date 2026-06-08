//! MVP performance benchmark manifest lock (closes #2567).
//!
//! Parent: #2530 (performance gate suite).
//!
//! Locks the schema + checker contract of
//! `validation/perf_benchmark_manifest.toml` plus
//! `scripts/perf_benchmark_manifest_check.py`. The manifest catalogs
//! the benchmark corpus used by the MVP 10× performance gate (id +
//! fixture path + category + tier) so a CI runner can fail fast when a
//! required benchmark fixture file goes missing.
//!
//! Acceptance (issue #2567):
//!
//!     1. Gate fails if a required benchmark is missing.
//!     2. Exploratory benchmarks are reported but not counted in MVP geomean.
//!     3. Manifest location and update command are documented.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use toml::Value as TomlValue;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn shipped_manifest_path() -> PathBuf {
    project_root()
        .join("validation")
        .join("perf_benchmark_manifest.toml")
}

fn checker_script() -> PathBuf {
    project_root()
        .join("scripts")
        .join("perf_benchmark_manifest_check.py")
}

fn read_manifest() -> TomlValue {
    let raw = std::fs::read_to_string(shipped_manifest_path())
        .expect("read shipped perf benchmark manifest");
    raw.parse::<TomlValue>().expect("manifest parses as TOML")
}

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("mamba-perf-bench-manifest-{tag}-{nanos}"));
    std::fs::create_dir_all(&dir).expect("create tempdir");
    dir
}

fn write_manifest(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, body).expect("write manifest fixture");
    path
}

fn run_checker(args: &[&str]) -> (i32, String, String) {
    let output = Command::new("python3")
        .arg(checker_script())
        .args(args)
        .current_dir(project_root())
        .output()
        .expect("invoke perf_benchmark_manifest_check.py");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

fn run_checker_json(args: &[&str]) -> (i32, Value) {
    let mut full = vec!["--format", "json"];
    full.extend_from_slice(args);
    let (code, stdout, stderr) = run_checker(&full);
    let payload: Value =
        serde_json::from_str(&stdout).unwrap_or_else(|e| {
            panic!(
                "checker JSON parse failed (code={code}): {e}\n--stdout--\n{stdout}\n--stderr--\n{stderr}"
            )
        });
    (code, payload)
}

// ─── Schema header ────────────────────────────────────────────────

#[test]
fn manifest_header_declares_version_and_issue_links() {
    let manifest = read_manifest();
    assert_eq!(
        manifest.get("version").and_then(|v| v.as_integer()),
        Some(1),
        "version must be locked to 1"
    );
    assert_eq!(
        manifest.get("manifest").and_then(|v| v.as_str()),
        Some("perf_benchmark"),
        "manifest field identifies the catalog"
    );
    assert_eq!(
        manifest.get("parent_issue").and_then(|v| v.as_integer()),
        Some(2530),
        "parent must point at the perf gate epic"
    );
    assert_eq!(
        manifest.get("issue").and_then(|v| v.as_integer()),
        Some(2567),
        "issue must be 2567"
    );
}

#[test]
fn manifest_declares_fixture_root_pointing_at_bench_fixtures() {
    let manifest = read_manifest();
    let root = manifest
        .get("fixture_root")
        .and_then(|v| v.as_str())
        .expect("fixture_root is a string");
    assert!(
        root.contains("tests/cpython/fixtures/core/bench"),
        "fixture_root must resolve to the bench fixture directory; got {root}"
    );
}

#[test]
fn manifest_declares_two_tier_role_model() {
    let manifest = read_manifest();
    let tiers: Vec<String> = manifest
        .get("tiers")
        .and_then(|v| v.as_array())
        .expect("tiers is an array")
        .iter()
        .filter_map(|t| t.as_str().map(String::from))
        .collect();
    assert!(tiers.iter().any(|t| t == "required"));
    assert!(tiers.iter().any(|t| t == "exploratory"));
}

#[test]
fn manifest_declares_category_enum() {
    let manifest = read_manifest();
    let cats: Vec<String> = manifest
        .get("categories")
        .and_then(|v| v.as_array())
        .expect("categories is an array")
        .iter()
        .filter_map(|t| t.as_str().map(String::from))
        .collect();
    for required in &["numeric", "recursion", "workload"] {
        assert!(
            cats.iter().any(|c| c == required),
            "categories must include {required}; got {cats:?}"
        );
    }
}

#[test]
fn shipped_manifest_lists_at_least_one_required_benchmark() {
    let manifest = read_manifest();
    let benches = manifest
        .get("benchmarks")
        .and_then(|v| v.as_array())
        .expect("benchmarks is an array");
    let required = benches
        .iter()
        .filter(|e| e.get("tier").and_then(|t| t.as_str()) == Some("required"))
        .count();
    assert!(
        required >= 1,
        "manifest must list at least one required benchmark"
    );
}

#[test]
fn every_shipped_required_benchmark_has_an_existing_fixture() {
    let manifest = read_manifest();
    let root = project_root().join("validation").join(
        manifest
            .get("fixture_root")
            .and_then(|v| v.as_str())
            .unwrap(),
    );
    let benches = manifest
        .get("benchmarks")
        .and_then(|v| v.as_array())
        .unwrap();
    for entry in benches {
        let tier = entry.get("tier").and_then(|t| t.as_str()).unwrap();
        if tier != "required" {
            continue;
        }
        let fixture = entry
            .get("fixture")
            .and_then(|f| f.as_str())
            .expect("required entry has fixture path");
        let resolved = root.join(fixture);
        assert!(
            resolved.exists(),
            "required benchmark fixture {fixture} not at {}",
            resolved.display()
        );
    }
}

// ─── Acceptance 1: gate fails if required fixture missing ──────────

#[test]
fn checker_fails_when_required_fixture_is_missing() {
    let dir = unique_dir("required-missing");
    std::fs::create_dir_all(dir.join("fixtures")).unwrap();
    let body = r#"
version = 1
manifest = "perf_benchmark"
parent_issue = 2530
issue = 2567
fixture_root = "fixtures"
categories = ["numeric"]
tiers = ["required", "exploratory"]

[[benchmarks]]
id = "ghost"
fixture = "ghost.py"
category = "numeric"
tier = "required"

[update]
location = "x"
command = "y"
"#;
    let path = write_manifest(&dir, "manifest.toml", body);
    let (code, _stdout, stderr) =
        run_checker(&["--manifest", path.to_str().unwrap(), "--format", "text"]);
    assert_eq!(
        code, 1,
        "missing required fixture must exit 1; stderr={stderr}"
    );
    assert!(
        stderr.contains("ghost.py"),
        "stderr must name the missing fixture; got {stderr}"
    );
    assert!(
        stderr.contains("gate fails"),
        "stderr must label the failure as gating; got {stderr}"
    );
}

#[test]
fn checker_passes_when_required_fixture_exists() {
    let dir = unique_dir("required-ok");
    let fx = dir.join("fixtures");
    std::fs::create_dir_all(&fx).unwrap();
    std::fs::write(fx.join("ok.py"), "x = 1\n").unwrap();
    let body = r#"
version = 1
manifest = "perf_benchmark"
parent_issue = 2530
issue = 2567
fixture_root = "fixtures"
categories = ["numeric"]
tiers = ["required", "exploratory"]

[[benchmarks]]
id = "ok"
fixture = "ok.py"
category = "numeric"
tier = "required"

[update]
location = "x"
command = "y"
"#;
    let path = write_manifest(&dir, "manifest.toml", body);
    let (code, payload) = run_checker_json(&["--manifest", path.to_str().unwrap()]);
    assert_eq!(code, 0, "valid manifest must exit 0; payload={payload}");
    assert_eq!(payload["required_missing"].as_array().unwrap().len(), 0);
    assert_eq!(payload["checked_count"], 1);
}

// ─── Acceptance 2: exploratory missing is reported, not gating ─────

#[test]
fn checker_warns_but_passes_when_only_exploratory_fixture_missing() {
    let dir = unique_dir("explore-missing");
    let fx = dir.join("fixtures");
    std::fs::create_dir_all(&fx).unwrap();
    std::fs::write(fx.join("ok.py"), "x = 1\n").unwrap();
    let body = r#"
version = 1
manifest = "perf_benchmark"
parent_issue = 2530
issue = 2567
fixture_root = "fixtures"
categories = ["numeric"]
tiers = ["required", "exploratory"]

[[benchmarks]]
id = "ok"
fixture = "ok.py"
category = "numeric"
tier = "required"

[[benchmarks]]
id = "ghost_explore"
fixture = "ghost_explore.py"
category = "numeric"
tier = "exploratory"

[update]
location = "x"
command = "y"
"#;
    let path = write_manifest(&dir, "manifest.toml", body);
    let (code, payload) = run_checker_json(&["--manifest", path.to_str().unwrap()]);
    assert_eq!(
        code, 0,
        "missing exploratory fixture must NOT gate; payload={payload}"
    );
    assert_eq!(payload["required_missing"].as_array().unwrap().len(), 0);
    let explor = payload["exploratory_missing"].as_array().unwrap();
    assert_eq!(explor.len(), 1);
    assert_eq!(explor[0]["id"], "ghost_explore");
    assert_eq!(explor[0]["tier"], "exploratory");
}

// ─── Acceptance 3: manifest location + update command documented ───

#[test]
fn shipped_manifest_documents_update_location_and_command() {
    let manifest = read_manifest();
    let update = manifest
        .get("update")
        .and_then(|v| v.as_table())
        .expect("manifest declares [update]");
    let location = update
        .get("location")
        .and_then(|v| v.as_str())
        .expect("update.location is a string");
    let command = update
        .get("command")
        .and_then(|v| v.as_str())
        .expect("update.command is a string");
    let docs = update
        .get("docs")
        .and_then(|v| v.as_str())
        .expect("update.docs is a string");
    assert!(
        location.ends_with("perf_benchmark_manifest.toml"),
        "update.location must name the manifest file; got {location}"
    );
    assert!(
        command.contains("perf_benchmark_manifest_check.py"),
        "update.command must invoke the checker; got {command}"
    );
    assert!(
        docs.contains("[[benchmarks]]"),
        "update.docs must reference the [[benchmarks]] section name; got {docs}"
    );
}

#[test]
fn checker_prints_update_location_and_command_in_text_mode() {
    let (_code, _stdout, stderr) = run_checker(&["--format", "text"]);
    assert!(
        stderr.contains("update.location"),
        "text output must surface update.location; got {stderr}"
    );
    assert!(
        stderr.contains("update.command"),
        "text output must surface update.command; got {stderr}"
    );
}

// ─── Schema-error / robustness coverage ────────────────────────────

#[test]
fn checker_reports_duplicate_ids() {
    let dir = unique_dir("dup-ids");
    let fx = dir.join("fixtures");
    std::fs::create_dir_all(&fx).unwrap();
    std::fs::write(fx.join("ok.py"), "x = 1\n").unwrap();
    let body = r#"
version = 1
manifest = "perf_benchmark"
parent_issue = 2530
issue = 2567
fixture_root = "fixtures"
categories = ["numeric"]
tiers = ["required", "exploratory"]

[[benchmarks]]
id = "twin"
fixture = "ok.py"
category = "numeric"
tier = "required"

[[benchmarks]]
id = "twin"
fixture = "ok.py"
category = "numeric"
tier = "required"

[update]
location = "x"
command = "y"
"#;
    let path = write_manifest(&dir, "manifest.toml", body);
    let (code, payload) = run_checker_json(&["--manifest", path.to_str().unwrap()]);
    assert_eq!(code, 1, "duplicate id must surface as schema error gating");
    let errs = payload["schema_errors"].as_array().unwrap();
    assert!(
        errs.iter()
            .any(|e| e.as_str().unwrap_or("").contains("duplicate")),
        "schema_errors must call out duplicate id; got {payload}"
    );
}

#[test]
fn checker_rejects_unknown_category() {
    let dir = unique_dir("bad-category");
    let fx = dir.join("fixtures");
    std::fs::create_dir_all(&fx).unwrap();
    std::fs::write(fx.join("ok.py"), "x = 1\n").unwrap();
    let body = r#"
version = 1
manifest = "perf_benchmark"
parent_issue = 2530
issue = 2567
fixture_root = "fixtures"
categories = ["numeric"]
tiers = ["required", "exploratory"]

[[benchmarks]]
id = "ok"
fixture = "ok.py"
category = "made-up"
tier = "required"

[update]
location = "x"
command = "y"
"#;
    let path = write_manifest(&dir, "manifest.toml", body);
    let (code, payload) = run_checker_json(&["--manifest", path.to_str().unwrap()]);
    assert_eq!(code, 1, "bad category must be a schema error");
    let errs = payload["schema_errors"].as_array().unwrap();
    assert!(
        errs.iter()
            .any(|e| e.as_str().unwrap_or("").contains("category")),
        "schema_errors must name the offending category; got {payload}"
    );
}

#[test]
fn checker_exits_101_when_manifest_missing() {
    let (code, _stdout, stderr) =
        run_checker(&["--manifest", "/tmp/perf-bench-manifest-does-not-exist.toml"]);
    assert_eq!(code, 101, "missing manifest must exit 101");
    assert!(
        stderr.contains("manifest missing"),
        "stderr must name the missing manifest; got {stderr}"
    );
}

#[test]
fn checker_help_documents_manifest_and_format_flags() {
    let (code, stdout, _stderr) = run_checker(&["--help"]);
    assert_eq!(code, 0, "--help must exit 0");
    assert!(
        stdout.contains("--manifest"),
        "help must document --manifest; got {stdout}"
    );
    assert!(
        stdout.contains("--format"),
        "help must document --format; got {stdout}"
    );
}

// ─── Cross-link to perf profile + baseline tier metadata ───────────

#[test]
fn manifest_cross_links_performance_profile_and_baseline_tier() {
    let manifest = read_manifest();
    let refs = manifest
        .get("references")
        .and_then(|v| v.as_table())
        .expect("[references] table present");
    let perf = refs
        .get("performance_profile")
        .and_then(|v| v.as_table())
        .expect("[references.performance_profile]");
    assert_eq!(
        perf.get("issue").and_then(|v| v.as_integer()),
        Some(2815),
        "performance profile cross-ref must point at #2815"
    );
    let baseline = refs
        .get("baseline_tier_metadata")
        .and_then(|v| v.as_table())
        .expect("[references.baseline_tier_metadata]");
    assert_eq!(
        baseline.get("issue").and_then(|v| v.as_integer()),
        Some(2566),
        "baseline tier metadata cross-ref must point at #2566"
    );
}
