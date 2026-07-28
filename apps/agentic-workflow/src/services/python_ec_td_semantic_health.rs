//! Two-cell semantic health for Python EC and TD artifacts.
//!
//! Operational diagnostics remain available elsewhere in `aw health`; this
//! module answers only whether EC accepts TD and whether their declared public
//! artifact boundary is aligned in both directions.

use crate::services::{project_registry, python_ec, python_td};
use anyhow::{Context, Result};
use serde::Serialize;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SemanticCellEvaluation {
    Passed,
    Failed,
    Unavailable,
    NotEvaluated,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EcAcceptsTdCell {
    pub evaluation: SemanticCellEvaluation,
    pub case_count: usize,
    pub passed_count: usize,
    pub failed_cases: Vec<String>,
    pub missing_evidence_cases: Vec<String>,
    pub findings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EcTdAlignmentCell {
    pub evaluation: SemanticCellEvaluation,
    pub missing_in_td: Vec<String>,
    pub missing_in_ec: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PythonEcTdSemanticHealth {
    pub ec_accepts_td: EcAcceptsTdCell,
    pub ec_td_alignment: EcTdAlignmentCell,
}

impl PythonEcTdSemanticHealth {
    pub fn not_evaluated() -> Self {
        Self {
            ec_accepts_td: EcAcceptsTdCell {
                evaluation: SemanticCellEvaluation::NotEvaluated,
                case_count: 0,
                passed_count: 0,
                failed_cases: Vec::new(),
                missing_evidence_cases: Vec::new(),
                findings: Vec::new(),
            },
            ec_td_alignment: EcTdAlignmentCell {
                evaluation: SemanticCellEvaluation::NotEvaluated,
                missing_in_td: Vec::new(),
                missing_in_ec: Vec::new(),
            },
        }
    }

    pub fn unavailable(finding: impl Into<String>) -> Self {
        let finding = finding.into();
        Self {
            ec_accepts_td: EcAcceptsTdCell {
                evaluation: SemanticCellEvaluation::Unavailable,
                case_count: 0,
                passed_count: 0,
                failed_cases: Vec::new(),
                missing_evidence_cases: Vec::new(),
                findings: vec![finding],
            },
            ec_td_alignment: EcTdAlignmentCell {
                evaluation: SemanticCellEvaluation::Unavailable,
                missing_in_td: Vec::new(),
                missing_in_ec: Vec::new(),
            },
        }
    }
}

pub fn evaluate(project_root: &Path, project: &str) -> Result<PythonEcTdSemanticHealth> {
    let row = project_registry::resolve_project_config_row(project_root, project)?;
    let artifact_root = project_root.join(&row.path);
    let td_root = PathBuf::from(
        project_registry::resolve_td_root_from_config(project_root, &row.name)
            .map_err(|error| anyhow::anyhow!("{}: {}", error.kind, error.message))?
            .root,
    );
    let ec_root = artifact_root.join("external-contracts");
    let ir = python_td::compile_python_td_project(&td_root)?;
    let inventory = python_ec::discover_python_ec_inventory(&ec_root)?;

    let public_td_behaviors = ir
        .modules
        .iter()
        .filter(|module| module.public_contract)
        .flat_map(|module| {
            let artifact_id = module
                .artifact_id
                .as_deref()
                .expect("public TD contract has an explicit artifact identity");
            module
                .public_behaviors
                .iter()
                .map(move |behavior| behavior_edge(artifact_id, behavior))
        })
        .collect::<BTreeSet<_>>();
    let ec_behaviors = inventory
        .cases
        .iter()
        .map(|case| behavior_edge(&case.artifact_id, &case.use_case_id))
        .collect::<BTreeSet<_>>();
    let ec_td_alignment = align_behaviors(&public_td_behaviors, &ec_behaviors);

    let td_cases = inventory
        .cases
        .iter()
        .filter(|case| case.applicability == "td")
        .collect::<Vec<_>>();
    let mut passed_count = 0;
    let mut failed_cases = Vec::new();
    let mut missing_evidence_cases = Vec::new();
    let mut findings = inventory.findings.clone();
    for case in &td_cases {
        match case_evidence(&ec_root, case) {
            Ok(CaseEvidence::Passed) => passed_count += 1,
            Ok(CaseEvidence::Failed(finding)) => {
                failed_cases.push(case.id.clone());
                findings.push(finding);
            }
            Ok(CaseEvidence::Missing(finding)) => {
                missing_evidence_cases.push(case.id.clone());
                findings.push(finding);
            }
            Err(error) => {
                failed_cases.push(case.id.clone());
                findings.push(format!(
                    "Python EC case `{}` evidence is invalid: {error}",
                    case.id
                ));
            }
        }
    }
    failed_cases.sort();
    missing_evidence_cases.sort();
    findings.sort();
    findings.dedup();
    let evaluation = if !inventory.findings.is_empty() || !failed_cases.is_empty() {
        SemanticCellEvaluation::Failed
    } else if td_cases.is_empty() || !missing_evidence_cases.is_empty() {
        SemanticCellEvaluation::NotEvaluated
    } else {
        SemanticCellEvaluation::Passed
    };

    Ok(PythonEcTdSemanticHealth {
        ec_accepts_td: EcAcceptsTdCell {
            evaluation,
            case_count: td_cases.len(),
            passed_count,
            failed_cases,
            missing_evidence_cases,
            findings,
        },
        ec_td_alignment,
    })
}

fn behavior_edge(artifact_id: &str, behavior_id: &str) -> String {
    format!("{artifact_id}#{behavior_id}")
}

fn align_behaviors(
    public_td_behaviors: &BTreeSet<String>,
    ec_behaviors: &BTreeSet<String>,
) -> EcTdAlignmentCell {
    let missing_in_td = ec_behaviors
        .difference(public_td_behaviors)
        .cloned()
        .collect::<Vec<_>>();
    let missing_in_ec = public_td_behaviors
        .difference(ec_behaviors)
        .cloned()
        .collect::<Vec<_>>();
    EcTdAlignmentCell {
        evaluation: if missing_in_td.is_empty() && missing_in_ec.is_empty() {
            SemanticCellEvaluation::Passed
        } else {
            SemanticCellEvaluation::Failed
        },
        missing_in_td,
        missing_in_ec,
    }
}

enum CaseEvidence {
    Passed,
    Failed(String),
    Missing(String),
}

fn case_evidence(ec_root: &Path, case: &python_ec::PythonEcCase) -> Result<CaseEvidence> {
    for relative in &case.evidence_paths {
        let path = ec_root.join(relative);
        if !path.is_file() {
            return Ok(CaseEvidence::Missing(format!(
                "Python EC case `{}` has no evidence at `{relative}`",
                case.id
            )));
        }
        let body = fs::read_to_string(&path)
            .with_context(|| format!("read Python EC evidence {}", path.display()))?;
        let value: serde_json::Value = serde_json::from_str(&body)
            .with_context(|| format!("parse Python EC evidence {}", path.display()))?;
        if value.get("protocol").and_then(serde_json::Value::as_str)
            != Some("aw.python-ec.evidence.v1")
        {
            return Ok(CaseEvidence::Failed(format!(
                "Python EC case `{}` evidence `{relative}` has an unsupported protocol",
                case.id
            )));
        }
        if value.get("exit_code").and_then(serde_json::Value::as_i64) != Some(0) {
            return Ok(CaseEvidence::Failed(format!(
                "Python EC case `{}` evidence `{relative}` does not record exit_code=0",
                case.id
            )));
        }
    }
    Ok(CaseEvidence::Passed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(values: &[&str]) -> BTreeSet<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn alignment_reports_both_set_differences() {
        let alignment = align_behaviors(
            &set(&[
                "artifact:demo/shared#shared",
                "artifact:demo/shared#td-only",
            ]),
            &set(&[
                "artifact:demo/shared#shared",
                "artifact:demo/shared#ec-only",
            ]),
        );

        assert_eq!(alignment.evaluation, SemanticCellEvaluation::Failed);
        assert_eq!(
            alignment.missing_in_td,
            vec!["artifact:demo/shared#ec-only"]
        );
        assert_eq!(
            alignment.missing_in_ec,
            vec!["artifact:demo/shared#td-only"]
        );
    }

    #[test]
    fn internal_td_artifacts_do_not_require_ec() {
        let alignment = align_behaviors(
            &set(&["artifact:demo/shared#behavior"]),
            &set(&["artifact:demo/shared#behavior"]),
        );

        assert_eq!(alignment.evaluation, SemanticCellEvaluation::Passed);
        assert!(alignment.missing_in_td.is_empty());
        assert!(alignment.missing_in_ec.is_empty());
    }
}
