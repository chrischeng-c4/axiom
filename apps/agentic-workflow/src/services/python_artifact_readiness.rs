// HANDWRITE-BEGIN gap="missing-generator:python-artifact-readiness" tracker="#2304" reason="The Python protocol consumer joins hand-authored TD and EC inventories until their report adapter is generator-owned."
//! Read-only readiness projection for `python-v1` artifact projects.
//!
//! Capability and health consume this exact projection so neither command
//! recreates its own interpretation of Python TD/EC inventory or evidence.

use crate::{
    models::project::ProjectArtifactModel,
    services::{project_registry, python_ec, python_td},
};
use anyhow::Result;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct PythonArtifactReadiness {
    pub enabled: bool,
    pub ready: bool,
    pub td_semantic_digest: Option<String>,
    pub td_module_ids: Vec<String>,
    pub ec_inventory_path: Option<String>,
    pub ec_source_digest: Option<String>,
    pub dependency_lock_digest: Option<String>,
    pub cases: Vec<PythonArtifactCaseReadiness>,
    pub required_case_count: usize,
    pub ready_case_count: usize,
    pub blockers: Vec<String>,
    pub next_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PythonArtifactCaseReadiness {
    pub id: String,
    pub capability_id: String,
    pub use_case_id: String,
    pub dimension: String,
    pub applicability: String,
    pub required_for_production: bool,
    pub evidence_paths: Vec<String>,
    pub evidence_ready: bool,
}

/// Return `None` for legacy projects. Python projects always receive a
/// projection, including malformed inventories, so consumers can report one
/// deterministic remediation without mutating the project.
pub fn evaluate(project_root: &Path, project: &str) -> Result<Option<PythonArtifactReadiness>> {
    let row = project_registry::resolve_project_config_row(project_root, project)?;
    if row.effective_artifact_model() != ProjectArtifactModel::PythonV1 {
        return Ok(None);
    }

    let artifact_root = project_root.join(&row.path);
    let td_root = artifact_root.join("tech-design");
    let ec_root = artifact_root.join("external-contracts");
    let mut readiness = PythonArtifactReadiness {
        enabled: true,
        ready: false,
        ..Default::default()
    };

    match python_td::compile_python_td_project(&td_root) {
        Ok(ir) => {
            readiness.td_semantic_digest = Some(ir.semantic_digest);
            readiness.td_module_ids = ir.modules.into_iter().map(|module| module.id).collect();
        }
        Err(error) => readiness
            .blockers
            .push(format!("Python TD inventory unavailable: {error}")),
    }

    match python_ec::discover_python_ec_inventory(&ec_root) {
        Ok(inventory) => {
            readiness.ec_inventory_path = inventory
                .inventory_path
                .strip_prefix(project_root)
                .ok()
                .map(|path| path.to_string_lossy().replace('\\', "/"))
                .or_else(|| Some(inventory.inventory_path.to_string_lossy().into_owned()));
            readiness.ec_source_digest = Some(inventory.source_digest);
            readiness.dependency_lock_digest = Some(inventory.dependency_lock_digest);
            readiness.blockers.extend(inventory.findings);
            for case in inventory.cases {
                let required_for_production =
                    case.dimension != "efficiency" || inventory.efficiency_policy == "required";
                let evidence_ready = case.evidence_paths.iter().all(|path| {
                    let evidence = ec_root.join(path);
                    evidence.is_file()
                        && std::fs::metadata(evidence)
                            .map(|metadata| metadata.len() > 0)
                            .unwrap_or(false)
                });
                if required_for_production {
                    readiness.required_case_count += 1;
                    if evidence_ready {
                        readiness.ready_case_count += 1;
                    } else {
                        readiness.blockers.push(format!(
                            "Python EC case `{}` has missing or empty digest-bound evidence",
                            case.id
                        ));
                    }
                }
                readiness.cases.push(PythonArtifactCaseReadiness {
                    id: case.id,
                    capability_id: case.capability_id,
                    use_case_id: case.use_case_id,
                    dimension: case.dimension,
                    applicability: case.applicability,
                    required_for_production,
                    evidence_paths: case.evidence_paths,
                    evidence_ready,
                });
            }
        }
        Err(error) => readiness
            .blockers
            .push(format!("Python EC inventory unavailable: {error}")),
    }

    readiness.blockers.sort();
    readiness.blockers.dedup();
    readiness.cases.sort_by(|left, right| left.id.cmp(&right.id));
    readiness.td_module_ids.sort();
    readiness.ready = readiness.blockers.is_empty();
    readiness.next_command = (!readiness.ready).then(|| {
        if readiness.ec_inventory_path.is_none() {
            format!("aw ec check --project {}", row.name)
        } else if readiness.td_semantic_digest.is_none() {
            format!(
                "aw td check {} --project {}",
                td_root.display(),
                row.name
            )
        } else if readiness
            .cases
            .iter()
            .any(|case| case.required_for_production && !case.evidence_ready && case.applicability == "td")
        {
            format!("aw ec verify --project {} --stage core", row.name)
        } else {
            format!("aw ec verify --project {} --stage operational", row.name)
        }
    });
    Ok(Some(readiness))
}
// HANDWRITE-END
