//! MVP performance internal-time marker gate (closes #2570).
//!
//! Parent: #2530 (performance gate suite).
//!
//! Locks the contract of `scripts/perf_internal_time_check.py`. The
//! checker enforces that every required perf benchmark declares
//! `timing_mode = "internal"` in the manifest AND that the fixture
//! file emits the `INTERNAL_TIME_NS=<u64>` marker the bench harness
//! reads. Process-wall timing is still acceptable for exploratory
//! benchmarks.
//!
//! Acceptance (issue #2570):
//!
//!     1. Required fixture without timing metadata fails validation.
//!     2. Summary distinguishes internal timing from process-wall timing.
//!     3. Existing exploratory fixtures may remain process-wall timed.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use toml::Value as TomlValue;

fn shipped_manifest_path() -> PathBuf {
    crate::common::project_root()
        .join("validation")
        .join("perf_benchmark_manifest.toml")
}

fn checker_script() -> PathBuf {
    crate::common::project_root()
        .join("scripts")
        .join("perf_internal_time_check.py")
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
    let dir = std::env::temp_dir().join(format!("mamba-perf-internal-time-{tag}-{nanos}"));
    std::fs::create_dir_all(&dir).expect("create tempdir");
    dir
}

fn run_checker(args: &[&str]) -> (i32, String, String) {
    let output = Command::new("python3")
        .arg(checker_script())
        .args(args)
        .current_dir(crate::common::project_root())
        .output()
        .expect("invoke perf_internal_time_check.py");
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

fn write_manifest_and_fixtures(dir: &Path, body: &str, fixtures: &[(&str, &str)]) -> PathBuf {
    let fx = dir.join("fixtures");
    std::fs::create_dir_all(&fx).unwrap();
    for (name, content) in fixtures {
        std::fs::write(fx.join(name), content).unwrap();
    }
    let path = dir.join("manifest.toml");
    std::fs::write(&path, body).unwrap();
    path
}

// ─── Schema: manifest declares timing_modes enum ──────────────────

#[test]
fn manifest_declares_timing_modes_enum_with_internal_and_process_wall() {
    let manifest = read_manifest();
    let modes: Vec<String> = manifest
        .get("timing_modes")
        .and_then(|v| v.as_array())
        .expect("timing_modes is an array")
        .iter()
        .filter_map(|t| t.as_str().map(String::from))
        .collect();
    assert!(modes.iter().any(|m| m == "internal"));
    assert!(modes.iter().any(|m| m == "process_wall"));
}

// ─── Shipped manifest: required entries declare internal timing ───

#[test]
fn shipped_required_entries_declare_internal_timing_mode() {
    let manifest = read_manifest();
    let benches = manifest
        .get("benchmarks")
        .and_then(|v| v.as_array())
        .expect("benchmarks array");
    let mut required_seen = 0usize;
    for e in benches {
        if e.get("tier").and_then(|v| v.as_str()) != Some("required") {
            continue;
        }
        required_seen += 1;
        let tm = e
            .get("timing_mode")
            .and_then(|v| v.as_str())
            .expect("required entry declares timing_mode");
        assert_eq!(
            tm,
            "internal",
            "required entry id={:?} must declare timing_mode=internal",
            e.get("id")
        );
    }
    assert!(
        required_seen >= 1,
        "shipped manifest must list at least one required entry"
    );
}

#[test]
fn shipped_required_fixtures_emit_internal_time_marker() {
    let manifest = read_manifest();
    let root = crate::common::project_root().join("validation").join(
        manifest
            .get("fixture_root")
            .and_then(|v| v.as_str())
            .unwrap(),
    );
    let benches = manifest
        .get("benchmarks")
        .and_then(|v| v.as_array())
        .unwrap();
    for e in benches {
        if e.get("tier").and_then(|v| v.as_str()) != Some("required") {
            continue;
        }
        let fixture = e.get("fixture").and_then(|v| v.as_str()).unwrap();
        let body = std::fs::read_to_string(root.join(fixture)).expect("read required fixture");
        assert!(
            body.contains("INTERNAL_TIME_NS="),
            "required fixture {fixture} must emit INTERNAL_TIME_NS marker"
        );
    }
}

#[test]
fn shipped_manifest_passes_internal_time_check() {
    let (code, _stdout, stderr) = run_checker(&["--format", "text"]);
    assert_eq!(code, 0, "shipped manifest must pass; stderr={stderr}");
    assert!(
        stderr.contains("perf_internal_time_check: clean"),
        "text output must report clean; got {stderr}"
    );
}

// ─── Acceptance 1: required without timing metadata fails ─────────

#[test]
fn required_entry_missing_timing_mode_fails_validation() {
    let dir = unique_dir("required-no-tm");
    let body = r#"
version = 1
manifest = "perf_benchmark"
parent_issue = 2530
issue = 2567
fixture_root = "fixtures"
categories = ["numeric"]
tiers = ["required", "exploratory"]
timing_modes = ["internal", "process_wall"]

[[benchmarks]]
id = "rogue"
fixture = "rogue.py"
category = "numeric"
tier = "required"

[update]
location = "x"
command = "y"
"#;
    let path =
        write_manifest_and_fixtures(&dir, body, &[("rogue.py", "print('INTERNAL_TIME_NS=1')\n")]);
    let (code, payload) = run_checker_json(&["--manifest", path.to_str().unwrap()]);
    assert_eq!(code, 1, "missing timing_mode must gate; payload={payload}");
    let v = payload["violations"].as_array().unwrap();
    assert_eq!(v.len(), 1);
    assert!(v[0]["reason"]
        .as_str()
        .unwrap()
        .contains("missing timing_mode"));
}

#[test]
fn required_entry_with_process_wall_timing_mode_fails_validation() {
    let dir = unique_dir("required-process-wall");
    let body = r#"
version = 1
manifest = "perf_benchmark"
parent_issue = 2530
issue = 2567
fixture_root = "fixtures"
categories = ["numeric"]
tiers = ["required", "exploratory"]
timing_modes = ["internal", "process_wall"]

[[benchmarks]]
id = "slow"
fixture = "slow.py"
category = "numeric"
tier = "required"
timing_mode = "process_wall"

[update]
location = "x"
command = "y"
"#;
    let path = write_manifest_and_fixtures(&dir, body, &[("slow.py", "x = 1\n")]);
    let (code, payload) = run_checker_json(&["--manifest", path.to_str().unwrap()]);
    assert_eq!(code, 1, "process_wall on required must gate");
    let v = payload["violations"].as_array().unwrap();
    assert!(v
        .iter()
        .any(|item| item["reason"].as_str().unwrap().contains("process_wall")));
}

#[test]
fn required_entry_internal_mode_without_marker_in_fixture_fails() {
    let dir = unique_dir("internal-no-marker");
    let body = r#"
version = 1
manifest = "perf_benchmark"
parent_issue = 2530
issue = 2567
fixture_root = "fixtures"
categories = ["numeric"]
tiers = ["required", "exploratory"]
timing_modes = ["internal", "process_wall"]

[[benchmarks]]
id = "liar"
fixture = "liar.py"
category = "numeric"
tier = "required"
timing_mode = "internal"

[update]
location = "x"
command = "y"
"#;
    let path = write_manifest_and_fixtures(
        &dir,
        body,
        &[("liar.py", "# no marker here\nprint('hi')\n")],
    );
    let (code, payload) = run_checker_json(&["--manifest", path.to_str().unwrap()]);
    assert_eq!(code, 1, "missing marker must gate");
    let v = payload["violations"].as_array().unwrap();
    assert!(v.iter().any(|item| item["reason"]
        .as_str()
        .unwrap()
        .contains("INTERNAL_TIME_NS")));
}

#[test]
fn required_entry_with_internal_mode_and_marker_passes() {
    let dir = unique_dir("internal-ok");
    let body = r#"
version = 1
manifest = "perf_benchmark"
parent_issue = 2530
issue = 2567
fixture_root = "fixtures"
categories = ["numeric"]
tiers = ["required", "exploratory"]
timing_modes = ["internal", "process_wall"]

[[benchmarks]]
id = "ok"
fixture = "ok.py"
category = "numeric"
tier = "required"
timing_mode = "internal"

[update]
location = "x"
command = "y"
"#;
    let path = write_manifest_and_fixtures(
        &dir,
        body,
        &[("ok.py", "print('INTERNAL_TIME_NS=12345')\n")],
    );
    let (code, payload) = run_checker_json(&["--manifest", path.to_str().unwrap()]);
    assert_eq!(code, 0, "valid required must pass; payload={payload}");
    assert_eq!(payload["violations"].as_array().unwrap().len(), 0);
}

// ─── Acceptance 2: summary distinguishes the two cohorts ──────────

#[test]
fn summary_separates_internal_and_process_wall_cohorts() {
    let dir = unique_dir("cohorts");
    let body = r#"
version = 1
manifest = "perf_benchmark"
parent_issue = 2530
issue = 2567
fixture_root = "fixtures"
categories = ["numeric"]
tiers = ["required", "exploratory"]
timing_modes = ["internal", "process_wall"]

[[benchmarks]]
id = "fast"
fixture = "fast.py"
category = "numeric"
tier = "required"
timing_mode = "internal"

[[benchmarks]]
id = "legacy_a"
fixture = "legacy_a.py"
category = "numeric"
tier = "exploratory"
timing_mode = "process_wall"

[[benchmarks]]
id = "legacy_b"
fixture = "legacy_b.py"
category = "numeric"
tier = "exploratory"
timing_mode = "process_wall"

[update]
location = "x"
command = "y"
"#;
    let path = write_manifest_and_fixtures(
        &dir,
        body,
        &[
            ("fast.py", "print('INTERNAL_TIME_NS=1')\n"),
            ("legacy_a.py", "x = 1\n"),
            ("legacy_b.py", "x = 1\n"),
        ],
    );
    let (code, payload) = run_checker_json(&["--manifest", path.to_str().unwrap()]);
    assert_eq!(code, 0, "mixed cohort must pass");
    let internal = payload["internal_timed"].as_array().unwrap();
    let process_wall = payload["process_wall_timed"].as_array().unwrap();
    assert_eq!(internal.len(), 1);
    assert_eq!(process_wall.len(), 2);
    assert_eq!(internal[0]["id"], "fast");
    let pw_ids: Vec<&str> = process_wall
        .iter()
        .map(|e| e["id"].as_str().unwrap())
        .collect();
    assert!(pw_ids.contains(&"legacy_a"));
    assert!(pw_ids.contains(&"legacy_b"));
}

#[test]
fn text_summary_prints_two_cohort_counts() {
    let (_code, _stdout, stderr) = run_checker(&["--format", "text"]);
    assert!(
        stderr.contains("internal_timed="),
        "text summary must print internal_timed count; got {stderr}"
    );
    assert!(
        stderr.contains("process_wall_timed="),
        "text summary must print process_wall_timed count; got {stderr}"
    );
}

// ─── Acceptance 3: exploratory may remain process-wall ────────────

#[test]
fn exploratory_entry_with_process_wall_timing_passes() {
    let dir = unique_dir("explore-process-wall");
    let body = r#"
version = 1
manifest = "perf_benchmark"
parent_issue = 2530
issue = 2567
fixture_root = "fixtures"
categories = ["numeric"]
tiers = ["required", "exploratory"]
timing_modes = ["internal", "process_wall"]

[[benchmarks]]
id = "ok"
fixture = "ok.py"
category = "numeric"
tier = "required"
timing_mode = "internal"

[[benchmarks]]
id = "legacy"
fixture = "legacy.py"
category = "numeric"
tier = "exploratory"
timing_mode = "process_wall"

[update]
location = "x"
command = "y"
"#;
    let path = write_manifest_and_fixtures(
        &dir,
        body,
        &[
            ("ok.py", "print('INTERNAL_TIME_NS=1')\n"),
            ("legacy.py", "x = 1\n"),
        ],
    );
    let (code, _payload) = run_checker_json(&["--manifest", path.to_str().unwrap()]);
    assert_eq!(code, 0, "exploratory process_wall must NOT gate");
}

#[test]
fn exploratory_entry_missing_timing_mode_does_not_gate() {
    let dir = unique_dir("explore-missing-tm");
    let body = r#"
version = 1
manifest = "perf_benchmark"
parent_issue = 2530
issue = 2567
fixture_root = "fixtures"
categories = ["numeric"]
tiers = ["required", "exploratory"]
timing_modes = ["internal", "process_wall"]

[[benchmarks]]
id = "ok"
fixture = "ok.py"
category = "numeric"
tier = "required"
timing_mode = "internal"

[[benchmarks]]
id = "legacy"
fixture = "legacy.py"
category = "numeric"
tier = "exploratory"

[update]
location = "x"
command = "y"
"#;
    let path = write_manifest_and_fixtures(
        &dir,
        body,
        &[
            ("ok.py", "print('INTERNAL_TIME_NS=1')\n"),
            ("legacy.py", "x = 1\n"),
        ],
    );
    let (code, _payload) = run_checker_json(&["--manifest", path.to_str().unwrap()]);
    assert_eq!(
        code, 0,
        "exploratory missing timing_mode must NOT gate (acceptance #3)"
    );
}

// ─── Robustness ───────────────────────────────────────────────────

#[test]
fn checker_exits_101_when_manifest_missing() {
    let (code, _stdout, stderr) =
        run_checker(&["--manifest", "/tmp/perf-internal-time-does-not-exist.toml"]);
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
    assert!(stdout.contains("--manifest"));
    assert!(stdout.contains("--format"));
}

// ─── Shipped fixtures still print their golden answer ─────────────

#[test]
fn shipped_int_sum_fixture_still_prints_final_answer_to_stdout() {
    let path =
        crate::common::project_root().join("tests/cpython/_regression/core/bench/int_sum.py");
    let body = std::fs::read_to_string(&path).expect("read int_sum.py");
    // Internal time marker MUST go to stderr; the golden answer
    // (`print(total)`) MUST stay on stdout. Anything else would
    // break golden .expected comparison downstream.
    assert!(
        body.contains("file=sys.stderr"),
        "int_sum.py must emit marker to stderr, not stdout"
    );
    assert!(
        body.contains("print(total)"),
        "int_sum.py must still print final total to stdout"
    );
}
