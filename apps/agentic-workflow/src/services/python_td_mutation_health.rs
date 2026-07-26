//! Mutation adequacy projection for project health and goal remediation.
//!
//! @spec apps/agentic-workflow/tech-design/logic/aw-python-td-mutation-health.md#logic

use super::{
    project_registry,
    python_ec::discover_python_ec_inventory,
    python_td::compile_python_td_project,
    python_td_mutation::{enumerate_python_td_mutants, PythonTdMutationScope},
    python_td_mutation_evidence::{
        verify_python_td_mutation_evidence, MutationEvidenceBindings, PythonTdMutationEvidence,
    },
    python_td_mutation_runner::{MutationVerdict, PythonTdNativeTarget},
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MutationAdequacyPolicy {
    #[default]
    Advisory,
    Required,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MutationAdequacyStatus {
    Missing,
    Invalid,
    Incomplete,
    Survived,
    Adequate,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PythonTdMutationAdequacy {
    pub enabled: bool,
    pub policy: MutationAdequacyPolicy,
    pub required_for_production: bool,
    pub status: MutationAdequacyStatus,
    pub ready: bool,
    pub evidence_dir: String,
    pub source_path: String,
    pub expected_run_count: usize,
    pub evidence_run_count: usize,
    pub killed_count: usize,
    pub survived_count: usize,
    pub findings: Vec<String>,
    pub next_command: Option<String>,
}

pub fn evaluate(project_root: &Path, project: &str) -> Result<PythonTdMutationAdequacy> {
    let row = project_registry::resolve_project_config_row(project_root, project)?;
    let artifact_root = project_root.join(&row.path);
    let evidence_dir = row
        .mutation_evidence_dir
        .as_deref()
        .map(|path| project_root.join(path))
        .unwrap_or_else(|| artifact_root.join("evidence/mutation-adequacy"));
    let source_path = row
        .mutation_source_path
        .as_deref()
        .map(|path| project_root.join(path))
        .unwrap_or_else(|| artifact_root.join("src"));
    evaluate_paths(
        project_root,
        &row.name,
        row.mutation_adequacy.unwrap_or_default(),
        &evidence_dir,
        &source_path,
    )
}

pub fn mutation_goal_command(project: &str) -> String {
    format!(
        "aw goal set --gate \"aw health --project {project} mutation\" \"Produce complete digest-bound mutation evidence for {project}\""
    )
}

fn evaluate_paths(
    project_root: &Path,
    project: &str,
    policy: MutationAdequacyPolicy,
    evidence_dir: &Path,
    source_path: &Path,
) -> Result<PythonTdMutationAdequacy> {
    let row = project_registry::resolve_project_config_row(project_root, project)?;
    let artifact_root = project_root.join(&row.path);
    let td_root = project_registry::resolve_td_root_from_config(project_root, project)
        .map(|resolved| PathBuf::from(resolved.root))
        .unwrap_or_else(|_| artifact_root.join("tech-design"));
    let ec_root = artifact_root.join("external-contracts");
    let mut findings = Vec::new();

    let ir = match compile_python_td_project(&td_root) {
        Ok(ir) => ir,
        Err(error) => {
            return Ok(result(
                project_root,
                project,
                policy,
                evidence_dir,
                source_path,
                MutationAdequacyStatus::Invalid,
                0,
                0,
                0,
                0,
                vec![format!("current Python TD inventory unavailable: {error}")],
            ))
        }
    };
    let ec = match discover_python_ec_inventory(&ec_root) {
        Ok(ec) => ec,
        Err(error) => {
            return Ok(result(
                project_root,
                project,
                policy,
                evidence_dir,
                source_path,
                MutationAdequacyStatus::Invalid,
                0,
                0,
                0,
                0,
                vec![format!("current Python EC inventory unavailable: {error}")],
            ))
        }
    };
    let source_digest = match digest_source_tree(source_path) {
        Ok(digest) => digest,
        Err(error) => {
            return Ok(result(
                project_root,
                project,
                policy,
                evidence_dir,
                source_path,
                MutationAdequacyStatus::Invalid,
                0,
                0,
                0,
                0,
                vec![format!(
                    "current mutation baseline source unavailable: {error}"
                )],
            ))
        }
    };

    let mutants = enumerate_python_td_mutants(&ir)?;
    let mut expected = BTreeMap::new();
    for mutant in mutants {
        let targets: &[PythonTdNativeTarget] = match mutant.descriptor.scope {
            PythonTdMutationScope::Semantic => &[
                PythonTdNativeTarget::Python,
                PythonTdNativeTarget::Rust,
                PythonTdNativeTarget::TypeScript,
            ],
            PythonTdMutationScope::Python => &[PythonTdNativeTarget::Python],
            PythonTdMutationScope::Rust => &[PythonTdNativeTarget::Rust],
            PythonTdMutationScope::TypeScript => &[PythonTdNativeTarget::TypeScript],
        };
        for target in targets {
            expected.insert(
                (mutant.descriptor.id.clone(), *target),
                mutant.descriptor.clone(),
            );
        }
    }
    let expected_run_count = expected.len();

    if !evidence_dir.is_dir() {
        return Ok(result(
            project_root,
            project,
            policy,
            evidence_dir,
            source_path,
            MutationAdequacyStatus::Missing,
            expected_run_count,
            0,
            0,
            0,
            vec![format!(
                "mutation evidence directory is missing: {}",
                display_path(project_root, evidence_dir)
            )],
        ));
    }

    let current = MutationEvidenceBindings {
        td_digest: ir.semantic_digest,
        ec_digest: ec.source_digest,
        source_digest,
    };
    let mut seen = BTreeSet::new();
    let mut evidence_run_count = 0;
    let mut killed_count = 0;
    let mut survived_count = 0;
    let mut entries = fs::read_dir(evidence_dir)
        .with_context(|| {
            format!(
                "read mutation evidence directory {}",
                evidence_dir.display()
            )
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let parsed = fs::read(&path)
            .with_context(|| format!("read mutation evidence {}", path.display()))
            .and_then(|bytes| {
                serde_json::from_slice::<PythonTdMutationEvidence>(&bytes)
                    .with_context(|| format!("parse mutation evidence {}", path.display()))
            });
        let evidence = match parsed {
            Ok(evidence) => evidence,
            Err(error) => {
                findings.push(error.to_string());
                continue;
            }
        };
        if let Err(error) = verify_python_td_mutation_evidence(&evidence, &current) {
            findings.push(format!("{}: {error}", display_path(project_root, &path)));
            continue;
        }
        let key = (evidence.mutant.id.clone(), evidence.target);
        let Some(descriptor) = expected.get(&key) else {
            findings.push(format!(
                "unexpected mutation evidence pair `{}` / `{}`",
                evidence.mutant.id,
                evidence.target.as_str()
            ));
            continue;
        };
        if descriptor != &evidence.mutant {
            findings.push(format!(
                "mutation evidence descriptor drift for `{}`",
                evidence.mutant.id
            ));
            continue;
        }
        if !seen.insert(key.clone()) {
            findings.push(format!(
                "duplicate mutation evidence pair `{}` / `{}`",
                key.0,
                key.1.as_str()
            ));
            continue;
        }
        evidence_run_count += 1;
        match evidence.verdict {
            MutationVerdict::Killed => killed_count += 1,
            MutationVerdict::Survived => {
                survived_count += 1;
                findings.push(format!(
                    "mutant `{}` survived `{}` gates",
                    key.0,
                    key.1.as_str()
                ));
            }
        }
    }
    for ((id, target), _) in expected.iter().filter(|(key, _)| !seen.contains(*key)) {
        findings.push(format!(
            "missing mutation evidence pair `{id}` / `{}`",
            target.as_str()
        ));
    }
    findings.sort();
    findings.dedup();
    let status = if findings.is_empty() && evidence_run_count == expected_run_count {
        MutationAdequacyStatus::Adequate
    } else if survived_count > 0 {
        MutationAdequacyStatus::Survived
    } else if evidence_run_count < expected_run_count {
        MutationAdequacyStatus::Incomplete
    } else {
        MutationAdequacyStatus::Invalid
    };
    Ok(result(
        project_root,
        project,
        policy,
        evidence_dir,
        source_path,
        status,
        expected_run_count,
        evidence_run_count,
        killed_count,
        survived_count,
        findings,
    ))
}

#[allow(clippy::too_many_arguments)]
fn result(
    project_root: &Path,
    project: &str,
    policy: MutationAdequacyPolicy,
    evidence_dir: &Path,
    source_path: &Path,
    status: MutationAdequacyStatus,
    expected_run_count: usize,
    evidence_run_count: usize,
    killed_count: usize,
    survived_count: usize,
    findings: Vec<String>,
) -> PythonTdMutationAdequacy {
    let ready = status == MutationAdequacyStatus::Adequate;
    PythonTdMutationAdequacy {
        enabled: true,
        policy,
        required_for_production: policy == MutationAdequacyPolicy::Required,
        status,
        ready,
        evidence_dir: display_path(project_root, evidence_dir),
        source_path: display_path(project_root, source_path),
        expected_run_count,
        evidence_run_count,
        killed_count,
        survived_count,
        findings,
        next_command: (!ready).then(|| mutation_goal_command(project)),
    }
}

pub fn digest_source_tree(root: &Path) -> Result<String> {
    if !root.is_dir() {
        anyhow::bail!("{} is not a directory", root.display());
    }
    let mut files = Vec::new();
    collect_files(root, root, &mut files)?;
    files.sort_by(|left, right| left.0.cmp(&right.0));
    let canonical = serde_json::to_vec(&files).context("serialize mutation source inventory")?;
    Ok(format!("sha256:{:x}", Sha256::digest(canonical)))
}

fn collect_files(root: &Path, current: &Path, out: &mut Vec<(String, String)>) -> Result<()> {
    let mut entries = fs::read_dir(current)
        .with_context(|| format!("read mutation source directory {}", current.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_files(root, &path, out)?;
        } else if file_type.is_file() {
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let bytes = fs::read(&path)?;
            out.push((relative, format!("sha256:{:x}", Sha256::digest(bytes))));
        }
    }
    Ok(())
}

fn display_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
