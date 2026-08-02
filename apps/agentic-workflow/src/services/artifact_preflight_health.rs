//! Artifact pre-flight gate evidence evaluation for `aw health`.
//!
//! Opt-in per project: a project that has never created
//! `<project-path>/evidence/artifact-preflight/*.json` sees zero behavior
//! change from this axis (`evaluate` returns an empty `Vec`). A project that
//! opts in gets each manifest's declared evidence evaluated for real against
//! `crate::models::preflight::default_preflight_gates` for its declared
//! `artifact_kind`, using the existing, already-unit-tested
//! `PreFlightGateReport::evaluate` -- this module supplies the missing
//! reachable-production-path evidence source, not new gate/severity logic.
//!
//! @spec apps/agentic-workflow/tech-design/src/agentic_workflow/migrated/specs/td_3904.py#logic

use super::project_registry;
use crate::models::artifact_quality::ArtifactKind;
use crate::models::preflight::{default_preflight_gates, PreFlightEvidence, PreFlightGateReport};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// One artifact's declared pre-flight evidence bundle. Deserialized directly
/// from `evidence/artifact-preflight/*.json`; `PreFlightEvidence` already
/// derives `Deserialize`, so this wrapper only adds the artifact identity.
#[derive(Debug, Clone, Deserialize)]
struct ArtifactPreflightManifest {
    artifact_ref: String,
    artifact_kind: ArtifactKind,
    #[serde(default)]
    evidence: Vec<PreFlightEvidence>,
}

/// Evaluate every artifact pre-flight manifest registered for `project`.
/// Absent-by-default: returns `Ok(vec![])` whenever
/// `<project-path>/evidence/artifact-preflight/` does not exist, which holds
/// for every project that has not opted in -- zero blast radius elsewhere.
pub fn evaluate(project_root: &Path, project: &str) -> Result<Vec<PreFlightGateReport>> {
    let row = project_registry::resolve_project_config_row(project_root, project)?;
    let artifact_root = project_root.join(&row.path);
    evaluate_dir(project_root, &evidence_dir(&artifact_root))
}

/// `<artifact_root>/evidence/artifact-preflight`, mirroring the
/// `evidence/mutation-adequacy` convention already established by
/// `python_td_mutation_health`.
pub fn evidence_dir(artifact_root: &Path) -> PathBuf {
    artifact_root.join("evidence/artifact-preflight")
}

fn evaluate_dir(project_root: &Path, dir: &Path) -> Result<Vec<PreFlightGateReport>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut entries = fs::read_dir(dir)
        .with_context(|| {
            format!(
                "read artifact preflight evidence directory {}",
                dir.display()
            )
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());

    let mut reports = Vec::with_capacity(entries.len());
    for entry in entries {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let relative = display_path(project_root, &path);
        let parsed = fs::read(&path)
            .with_context(|| format!("read artifact preflight manifest {}", path.display()))
            .and_then(|bytes| {
                serde_json::from_slice::<ArtifactPreflightManifest>(&bytes).with_context(|| {
                    format!("parse artifact preflight manifest {}", path.display())
                })
            });
        match parsed {
            Ok(manifest) => {
                let gates = default_preflight_gates(manifest.artifact_kind);
                reports.push(PreFlightGateReport::evaluate(
                    manifest.artifact_ref,
                    &gates,
                    &manifest.evidence,
                ));
            }
            // Fail closed: an unreadable/malformed manifest must not be
            // indistinguishable from "no evidence was ever declared" -- that
            // would let a corrupt file silently defeat a hard gate.
            Err(error) => {
                reports.push(PreFlightGateReport {
                    artifact_ref: relative.clone(),
                    results: Vec::new(),
                    production_blockers: vec![format!(
                        "artifact preflight evidence manifest {relative} is invalid: {error}"
                    )],
                    quality_warnings: Vec::new(),
                });
            }
        }
    }
    Ok(reports)
}

fn display_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_manifest(dir: &Path, name: &str, contents: &str) {
        fs::create_dir_all(dir).unwrap();
        fs::write(dir.join(name), contents).unwrap();
    }

    #[test]
    fn absent_directory_yields_no_reports() {
        let tmp = tempfile::tempdir().unwrap();

        let reports =
            evaluate_dir(tmp.path(), &tmp.path().join("evidence/artifact-preflight")).unwrap();

        assert!(reports.is_empty());
    }

    #[test]
    fn missing_hard_evidence_blocks_production() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("evidence/artifact-preflight");
        write_manifest(
            &dir,
            "example.json",
            r#"{"artifact_ref": "src/cli/example.rs", "artifact_kind": "code_artifact", "evidence": []}"#,
        );

        let reports = evaluate_dir(tmp.path(), &dir).unwrap();

        assert_eq!(reports.len(), 1);
        assert!(reports[0].blocks_production());
        assert!(reports[0]
            .production_blockers()
            .iter()
            .any(|blocker| blocker.contains("code-artifact-test")));
    }

    #[test]
    fn accepted_evidence_passes_and_is_recorded() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("evidence/artifact-preflight");
        write_manifest(
            &dir,
            "example.json",
            r#"{
                "artifact_ref": "src/cli/example.rs",
                "artifact_kind": "code_artifact",
                "evidence": [
                    {"gate_id": "code-artifact-test", "evidence_kind": "test", "source_ref": "cargo test example", "status": "accepted"},
                    {"gate_id": "code-artifact-spec-annotation", "evidence_kind": "source_annotation", "source_ref": "src/cli/example.rs#L1", "status": "accepted"}
                ]
            }"#,
        );

        let reports = evaluate_dir(tmp.path(), &dir).unwrap();

        assert_eq!(reports.len(), 1);
        assert!(!reports[0].blocks_production());
        assert_eq!(reports[0].artifact_ref, "src/cli/example.rs");
        assert!(reports[0].quality_warnings().is_empty());
    }

    #[test]
    fn malformed_manifest_fails_closed_as_a_blocker() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("evidence/artifact-preflight");
        write_manifest(&dir, "broken.json", "not json");

        let reports = evaluate_dir(tmp.path(), &dir).unwrap();

        assert_eq!(reports.len(), 1);
        assert!(reports[0].blocks_production());
    }

    fn write_demo_aw_toml(root: &Path) {
        fs::write(
            root.join("aw.toml"),
            concat!(
                "[[projects]]\n",
                "name = \"demo\"\n",
                "path = \"apps/demo\"\n",
                "\n",
                "[[projects.workspaces]]\n",
                "name = \"demo\"\n",
                "paths = [\"apps/demo/**\"]\n",
                "target = \"rust\"\n",
            ),
        )
        .unwrap();
    }

    #[test]
    fn evaluate_resolves_project_row_and_reads_its_evidence_dir() {
        let tmp = tempfile::tempdir().unwrap();
        write_demo_aw_toml(tmp.path());
        write_manifest(
            &tmp.path().join("apps/demo/evidence/artifact-preflight"),
            "example.json",
            r#"{"artifact_ref": "apps/demo/src/lib.rs", "artifact_kind": "code_artifact", "evidence": []}"#,
        );

        let reports = evaluate(tmp.path(), "demo").unwrap();

        assert_eq!(reports.len(), 1);
        assert!(reports[0].blocks_production());
    }

    #[test]
    fn evaluate_is_a_no_op_for_projects_that_never_opted_in() {
        let tmp = tempfile::tempdir().unwrap();
        write_demo_aw_toml(tmp.path());

        let reports = evaluate(tmp.path(), "demo").unwrap();

        assert!(reports.is_empty());
    }
}
