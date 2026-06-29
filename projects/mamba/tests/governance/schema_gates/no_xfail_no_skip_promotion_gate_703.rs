//! Executable gate for #703: replacement readiness cannot silently pass while
//! CPython fixtures still carry xfail or skip debt.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn script_path() -> PathBuf {
    crate::common::project_root().join("tests/harness/cpython/tools/promotion_gate.py")
}

fn write_fixture(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create fixture parent");
    }
    fs::write(path, body).expect("write fixture");
}

fn recorded_fixture(case: &str, xfail: &str, extra: &str) -> String {
    format!(
        r#"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "promotion_gate"
# dimension = "behavior"
# case = "{case}"
# subject = "promotion gate"
# kind = "semantic"
# xfail = "{xfail}"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
{extra}
print("{case}")
"#
    )
}

fn run_gate(root: &Path) -> std::process::Output {
    Command::new("python3.12")
        .arg(script_path())
        .args([
            "--root",
            root.to_str().unwrap(),
            "--profile",
            "replacement",
            "--json",
        ])
        .current_dir(crate::common::project_root())
        .output()
        .expect("run promotion_gate.py")
}

fn output_json(output: &std::process::Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|err| {
        panic!(
            "promotion_gate.py did not emit JSON: {err}\nstdout={}\nstderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

#[test]
fn clean_replacement_profile_passes() {
    let root = crate::common::unique_temp_dir("promotion-clean");
    write_fixture(
        root.path(),
        "behavior/core/promotion_gate/clean.py",
        &recorded_fixture("clean", "", ""),
    );

    let output = run_gate(root.path());
    let json = output_json(&output);
    assert!(
        output.status.success(),
        "clean promotion corpus should pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(json["failed"].as_bool(), Some(false));
    assert_eq!(json["xfail_count"].as_u64(), Some(0));
    assert_eq!(json["skip_count"].as_u64(), Some(0));
}

#[test]
fn replacement_profile_fails_on_xfail_debt() {
    let root = crate::common::unique_temp_dir("promotion-xfail");
    write_fixture(
        root.path(),
        "behavior/core/promotion_gate/xfail_case.py",
        &recorded_fixture("xfail_case", "tracked runtime gap #703", ""),
    );

    let output = run_gate(root.path());
    let json = output_json(&output);
    assert_eq!(output.status.code(), Some(70));
    assert_eq!(json["failed"].as_bool(), Some(true));
    assert_eq!(json["xfail_count"].as_u64(), Some(1));
    assert_eq!(json["promotion_debt_total"].as_u64(), Some(1));
    assert_eq!(json["owned_count"].as_u64(), Some(1));
    assert_eq!(
        json["debt_by_axis"]["dimension"]["behavior"]["xfail"].as_u64(),
        Some(1)
    );
    assert_eq!(
        json["debt_by_axis"]["bucket"]["core"]["xfail"].as_u64(),
        Some(1)
    );
    assert_eq!(
        json["debt_by_axis"]["lib"]["core/promotion_gate"]["xfail"].as_u64(),
        Some(1)
    );
}

#[test]
fn replacement_profile_fails_on_runtime_skip_debt() {
    let root = crate::common::unique_temp_dir("promotion-skip");
    write_fixture(
        root.path(),
        "behavior/core/promotion_gate/skip_case.py",
        &recorded_fixture(
            "skip_case",
            "",
            "import unittest\n\n@unittest.skip(\"not ready\")\ndef test_skip():\n    pass\n",
        ),
    );

    let output = run_gate(root.path());
    let json = output_json(&output);
    assert_eq!(output.status.code(), Some(70));
    assert_eq!(json["failed"].as_bool(), Some(true));
    assert_eq!(json["skip_count"].as_u64(), Some(1));
    assert_eq!(json["promotion_debt_total"].as_u64(), Some(1));
}

#[test]
fn replacement_profile_reports_unowned_promotion_pending_debt() {
    let root = crate::common::unique_temp_dir("promotion-unowned-pending");
    write_fixture(
        root.path(),
        "behavior/std-libs/promotion_gate/pending_case.py",
        &recorded_fixture(
            "pending_case",
            "auto-ported CPython test; mamba promotion pending",
            "",
        ),
    );

    let output = run_gate(root.path());
    let json = output_json(&output);
    assert_eq!(output.status.code(), Some(70));
    assert_eq!(json["xfail_count"].as_u64(), Some(1));
    assert_eq!(json["unowned_count"].as_u64(), Some(1));
    assert_eq!(json["promotion_pending_count"].as_u64(), Some(1));
    assert_eq!(
        json["unowned_debt"][0]["path"].as_str(),
        Some("behavior/std-libs/promotion_gate/pending_case.py")
    );
}

#[test]
fn replacement_profile_counts_real_world_optional_debt() {
    let root = crate::common::unique_temp_dir("promotion-optional");
    write_fixture(
        root.path(),
        "real_world/std-libs/promotion_gate/optional_case.py",
        &recorded_fixture("optional_case", "", ""),
    );

    let output = run_gate(root.path());
    let json = output_json(&output);
    assert_eq!(output.status.code(), Some(70));
    assert_eq!(json["optional_count"].as_u64(), Some(1));
    assert_eq!(json["promotion_debt_total"].as_u64(), Some(1));
    assert_eq!(
        json["optional_debt"][0]["path"].as_str(),
        Some("real_world/std-libs/promotion_gate/optional_case.py")
    );
}

#[test]
fn replacement_profile_reports_metadata_parse_errors_distinctly() {
    let root = crate::common::unique_temp_dir("promotion-parse");
    write_fixture(
        root.path(),
        "behavior/core/promotion_gate/bad.py",
        "# /// script\n# [tool.mamba]\n# xfail = \"unterminated\n# ///\nprint('bad')\n",
    );

    let output = run_gate(root.path());
    let json = output_json(&output);
    assert_eq!(output.status.code(), Some(71));
    assert_eq!(json["failed"].as_bool(), Some(true));
    assert_eq!(json["parse_error_count"].as_u64(), Some(1));
}
