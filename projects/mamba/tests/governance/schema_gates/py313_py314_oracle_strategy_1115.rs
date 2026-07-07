//! Governance lock for #1115: document the CPython 3.12 baseline and the
//! opt-in multi-version oracle plan before any 3.13/3.14 promotion.

use std::path::PathBuf;

fn project_root() -> PathBuf {
    crate::common::project_root()
}

fn strategy_doc() -> PathBuf {
    project_root().join("docs/compatibility/py313-py314-oracle-strategy.md")
}

fn load_strategy_doc() -> String {
    std::fs::read_to_string(strategy_doc()).expect("read #1115 oracle strategy doc")
}

#[test]
fn py313_py314_oracle_strategy_doc_exists() {
    assert!(strategy_doc().is_file(), "#1115 strategy doc must exist");
}

#[test]
fn py313_py314_oracle_strategy_keeps_baseline_and_lane_contract() {
    let doc = load_strategy_doc();
    for needle in [
        "# Py3.13 / Py3.14 Oracle Strategy",
        "default CPython replacement target remains Python 3.12",
        "tests/cpython/.cache/oracle-env/bin/python3",
        "Python 3.13 and Python 3.14 are opt-in oracle lanes",
        "tests/cpython/.cache/oracle-env-3.13/bin/python3",
        "tests/cpython/.cache/oracle-env-3.14/bin/python3",
        "MAMBA_ORACLE_PYTHON",
    ] {
        assert!(
            doc.contains(needle),
            "missing #1115 baseline/lane contract marker: {needle}"
        );
    }
}

#[test]
fn py313_py314_oracle_strategy_declares_fixture_metadata_and_removed_battery_policy() {
    let doc = load_strategy_doc();
    for needle in [
        "preferred record field is `python_version`",
        "`python_version = \"3.12\"`",
        "`python_version = \"3.13\"`",
        "`python_version = \"3.14\"`",
        "Governance/schema gates should reject version-specific additions",
        "PEP 594 removals are version-gated/retired for 3.13+ lanes",
        "without deleting 3.12 coverage",
        "`asynchat`",
        "`asyncore`",
        "`smtpd`",
    ] {
        assert!(
            doc.contains(needle),
            "missing #1115 metadata/PEP 594 marker: {needle}"
        );
    }
}

#[test]
fn py313_py314_oracle_strategy_lists_required_child_work_categories() {
    let doc = load_strategy_doc();
    for needle in [
        "PEP 696 TypeVar defaults",
        "PEP 701 follow-up fixture/doc alignment",
        "PEP 649 lazy annotations",
        "PEP 749 `annotationlib`",
        "PEP 750 template strings",
        "free-threading opportunity assessment",
        "PEP 594 removed-battery gating/docs",
        "multi-version oracle lane docs and metadata gates",
        "projects/mamba/tests/governance/schema_gates/strict_type_accounting_gate_704.rs",
    ] {
        assert!(
            doc.contains(needle),
            "missing #1115 child-WI/evidence marker: {needle}"
        );
    }
}
