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
use sha2::{Digest, Sha256};
use std::path::Path;

const EVIDENCE_PROTOCOL: &str = "aw.python-ec.evidence.v1";

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

enum LoadErrorKind {
    MissingOrEmpty,
    Unreadable,
    NotJson,
}

fn load_evidence_bytes(path: &Path) -> Result<Vec<u8>, LoadErrorKind> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(_) => return Err(LoadErrorKind::MissingOrEmpty),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(LoadErrorKind::MissingOrEmpty);
    }
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => return Err(LoadErrorKind::Unreadable),
    };
    if bytes.iter().all(u8::is_ascii_whitespace) {
        return Err(LoadErrorKind::MissingOrEmpty);
    }
    Ok(bytes)
}

fn compute_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{:x}", hasher.finalize())
}

fn compute_assertions_digest(assertions: &[String]) -> String {
    let json_bytes = serde_json::to_string(assertions).unwrap_or_default();
    compute_sha256(json_bytes.as_bytes())
}

fn bind_evidence_record(
    raw: &serde_json::Value,
    case_id: &str,
    evidence_rel_path: &str,
    current_source_digest: &str,
    declared_command: &str,
    ec_root: &Path,
    expected_implementation: &str,
) -> Result<(), String> {
    let prefix = format!("Python EC case `{case_id}` evidence `{evidence_rel_path}`");

    let raw_obj = match raw.as_object() {
        Some(obj) => obj,
        None => return Err(format!("{prefix} is not valid JSON")),
    };

    let protocol = raw_obj.get("protocol").and_then(|v| v.as_str());
    if protocol != Some(EVIDENCE_PROTOCOL) {
        return Err(format!("{prefix} has unsupported protocol"));
    }

    let raw_case_id = raw_obj.get("case_id");
    if raw_case_id.and_then(|v| v.as_str()) != Some(case_id) {
        let name_str = match raw_case_id {
            Some(serde_json::Value::String(s)) => s.clone(),
            Some(v) => v.to_string(),
            None => "None".to_string(),
        };
        return Err(format!("{prefix} names case `{name_str}`"));
    }

    let ec = raw_obj.get("exit_code");
    if ec.and_then(|v| v.as_i64()) != Some(0) {
        return Err(format!("{prefix} does not record successful execution"));
    }

    let attempts = match raw_obj.get("attempts").and_then(|v| v.as_array()) {
        Some(arr) if !arr.is_empty() => arr,
        _ => return Err(format!("{prefix} has no attempt records")),
    };

    let source_digest = raw_obj.get("source_digest").and_then(|v| v.as_str());
    if source_digest != Some(current_source_digest) {
        return Err(format!("{prefix} is stale for the current source digest"));
    }

    let decl_cmd = raw_obj.get("declared_command").and_then(|v| v.as_str());
    if decl_cmd != Some(declared_command) {
        return Err(format!("{prefix} does not match the declared command"));
    }

    let impl_val = raw_obj.get("implementation").and_then(|v| v.as_str());
    if impl_val != Some(expected_implementation) {
        return Err(format!("{prefix} is stale for `{expected_implementation}`"));
    }

    let impl_path = ec_root.join(expected_implementation);
    let impl_metadata = std::fs::symlink_metadata(&impl_path);
    let impl_valid = match impl_metadata {
        Ok(m) => !m.file_type().is_symlink() && m.is_file(),
        Err(_) => false,
    };
    if !impl_valid {
        return Err(format!("{prefix} is stale for `{expected_implementation}`"));
    }

    let impl_bytes = match std::fs::read(&impl_path) {
        Ok(bytes) => bytes,
        Err(_) => return Err(format!("{prefix} is stale for `{expected_implementation}`")),
    };
    let current_impl_digest = compute_sha256(&impl_bytes);
    let impl_digest_val = raw_obj.get("implementation_digest").and_then(|v| v.as_str());
    if impl_digest_val != Some(&current_impl_digest) {
        return Err(format!("{prefix} is stale for `{expected_implementation}`"));
    }

    let assertions_opt = raw_obj.get("assertions").filter(|v| !v.is_null());

    if let Some(assertions_val) = assertions_opt {
        let assertions_arr = match assertions_val.as_array() {
            Some(arr) if !arr.is_empty() => arr,
            _ => return Err(format!("{prefix} records zero executed assertions or tests")),
        };
        let mut assertions = Vec::with_capacity(assertions_arr.len());
        for item in assertions_arr {
            match item.as_str() {
                Some(s) if !s.is_empty() => assertions.push(s.to_string()),
                _ => return Err(format!("{prefix} records zero executed assertions or tests")),
            }
        }

        for attempt in attempts {
            let attempt_obj = match attempt.as_object() {
                Some(obj) => obj,
                None => return Err(format!("{prefix} does not record successful execution")),
            };
            let ac = attempt_obj.get("exit_code");
            if ac.and_then(|v| v.as_i64()) != Some(0) {
                return Err(format!("{prefix} does not record successful execution"));
            }
            let cnt = attempt_obj.get("assertion_count");
            let cnt_i64 = match cnt.and_then(|v| v.as_i64()) {
                Some(n) => n,
                None => return Err(format!("{prefix} has non-integer assertion_count")),
            };
            if cnt_i64 != assertions.len() as i64 {
                return Err(format!("{prefix} has wrong assertion_count"));
            }
            if let Some(ad) = attempt_obj.get("assertions_digest").filter(|v| !v.is_null()) {
                let ad_str = match ad.as_str() {
                    Some(s) => s,
                    None => return Err(format!("{prefix} has invalid assertions_digest type")),
                };
                if ad_str != compute_assertions_digest(&assertions) {
                    return Err(format!("{prefix} has wrong assertions_digest"));
                }
            }
        }
    } else {
        for attempt in attempts {
            let attempt_obj = match attempt.as_object() {
                Some(obj) => obj,
                None => return Err(format!("{prefix} does not record successful execution")),
            };
            let ac = attempt_obj.get("exit_code");
            if ac.and_then(|v| v.as_i64()) != Some(0) {
                return Err(format!("{prefix} does not record successful execution"));
            }
            let pt = attempt_obj.get("passed_tests").and_then(|v| v.as_i64());
            match pt {
                Some(n) if n > 0 => {}
                _ => return Err(format!("{prefix} records zero executed assertions or tests")),
            }
            let ft = attempt_obj.get("failed_tests").and_then(|v| v.as_i64());
            match ft {
                Some(0) => {}
                _ => return Err(format!("{prefix} records zero executed assertions or tests")),
            }
        }
    }

    Ok(())
}

struct CaseEvidenceEval {
    evidence_ready: bool,
    blocker: Option<String>,
}

fn evaluate_case_evidence(
    ec_root: &Path,
    case: &python_ec::PythonEcCase,
    current_source_digest: &str,
) -> CaseEvidenceEval {
    if case.evidence_paths.is_empty() {
        return CaseEvidenceEval {
            evidence_ready: false,
            blocker: Some(format!(
                "Python EC case `{}` has missing or empty digest-bound evidence",
                case.id
            )),
        };
    }

    for relative_path in &case.evidence_paths {
        let evidence_path = ec_root.join(relative_path);
        let raw_bytes = match load_evidence_bytes(&evidence_path) {
            Ok(bytes) => bytes,
            Err(LoadErrorKind::MissingOrEmpty) => {
                return CaseEvidenceEval {
                    evidence_ready: false,
                    blocker: Some(format!(
                        "Python EC case `{}` has missing or empty digest-bound evidence",
                        case.id
                    )),
                };
            }
            Err(LoadErrorKind::Unreadable) => {
                return CaseEvidenceEval {
                    evidence_ready: false,
                    blocker: Some(format!(
                        "Python EC case `{}` evidence `{relative_path}` is unreadable",
                        case.id
                    )),
                };
            }
            Err(LoadErrorKind::NotJson) => {
                return CaseEvidenceEval {
                    evidence_ready: false,
                    blocker: Some(format!(
                        "Python EC case `{}` evidence `{relative_path}` is not valid JSON",
                        case.id
                    )),
                };
            }
        };

        let text = match std::str::from_utf8(&raw_bytes) {
            Ok(t) => t,
            Err(_) => {
                return CaseEvidenceEval {
                    evidence_ready: false,
                    blocker: Some(format!(
                        "Python EC case `{}` evidence `{relative_path}` is not valid JSON",
                        case.id
                    )),
                };
            }
        };

        let json_val: serde_json::Value = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(_) => {
                return CaseEvidenceEval {
                    evidence_ready: false,
                    blocker: Some(format!(
                        "Python EC case `{}` evidence `{relative_path}` is not valid JSON",
                        case.id
                    )),
                };
            }
        };

        if let Err(blocker) = bind_evidence_record(
            &json_val,
            &case.id,
            relative_path,
            current_source_digest,
            &case.command,
            ec_root,
            &case.test_path,
        ) {
            return CaseEvidenceEval {
                evidence_ready: false,
                blocker: Some(blocker),
            };
        }
    }

    CaseEvidenceEval {
        evidence_ready: true,
        blocker: None,
    }
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
    let td_root = project_registry::resolve_td_root_from_config(project_root, &row.name)
        .map(|resolved| resolved.root)
        .unwrap_or_else(|_| {
            artifact_root
                .join("tech-design")
                .to_string_lossy()
                .into_owned()
        });
    let td_root = std::path::PathBuf::from(td_root);
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
            let current_source_digest = inventory.source_digest.clone();
            readiness.ec_source_digest = Some(inventory.source_digest);
            readiness.dependency_lock_digest = Some(inventory.dependency_lock_digest);
            readiness.blockers.extend(inventory.findings);
            for case in inventory.cases {
                let required_for_production =
                    case.dimension != "efficiency" || inventory.efficiency_policy == "required";
                let eval = evaluate_case_evidence(&ec_root, &case, &current_source_digest);
                let evidence_ready = eval.evidence_ready;
                if required_for_production {
                    readiness.required_case_count += 1;
                    if evidence_ready {
                        readiness.ready_case_count += 1;
                    } else if let Some(blocker) = eval.blocker {
                        readiness.blockers.push(blocker);
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
    readiness
        .cases
        .sort_by(|left, right| left.id.cmp(&right.id));
    readiness.td_module_ids.sort();
    readiness.ready = readiness.blockers.is_empty();
    readiness.next_command = (!readiness.ready).then(|| {
        if readiness.ec_inventory_path.is_none() {
            format!("aw ec check --project {}", row.name)
        } else if readiness.td_semantic_digest.is_none() {
            format!("aw td check {} --project {}", td_root.display(), row.name)
        } else if readiness.cases.iter().any(|case| {
            case.required_for_production && !case.evidence_ready && case.applicability == "td"
        }) {
            format!("aw ec verify --project {} --stage td", row.name)
        } else {
            format!("aw ec verify --project {} --stage cb", row.name)
        }
    });
    Ok(Some(readiness))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn malformed_evidence_rejected_with_deterministic_blocker() {
        let dir = tempdir().unwrap();
        let ec_root = dir.path();
        let evidence_rel = "evidence/bad.json";
        let evidence_path = ec_root.join(evidence_rel);
        fs::create_dir_all(evidence_path.parent().unwrap()).unwrap();
        fs::write(&evidence_path, b"{invalid json").unwrap();

        let case = python_ec::PythonEcCase {
            id: "test-case".to_string(),
            artifact_id: "artifact:demo/test".to_string(),
            capability_id: "cap".to_string(),
            use_case_id: "uc".to_string(),
            dimension: "behavior".to_string(),
            applicability: "td".to_string(),
            test_path: "src/cases/test.py".to_string(),
            promise: "promise".to_string(),
            oracle: "oracle".to_string(),
            threshold: None,
            target: "python".to_string(),
            command: "cmd".to_string(),
            evidence_paths: vec![evidence_rel.to_string()],
            known_failure: None,
        };

        let eval = evaluate_case_evidence(ec_root, &case, "sha256:src");
        assert!(!eval.evidence_ready);
        assert_eq!(
            eval.blocker,
            Some("Python EC case `test-case` evidence `evidence/bad.json` is not valid JSON".to_string())
        );
    }

    #[test]
    fn stale_source_and_binding_rejection() {
        let dir = tempdir().unwrap();
        let ec_root = dir.path();
        let impl_rel = "src/cases/test.py";
        let impl_path = ec_root.join(impl_rel);
        fs::create_dir_all(impl_path.parent().unwrap()).unwrap();
        fs::write(&impl_path, b"def verify(): return ['ok']\n").unwrap();
        let impl_digest = compute_sha256(&fs::read(&impl_path).unwrap());

        let evidence_rel = "evidence/test.json";
        let evidence_path = ec_root.join(evidence_rel);
        fs::create_dir_all(evidence_path.parent().unwrap()).unwrap();

        let valid_json = serde_json::json!({
            "protocol": EVIDENCE_PROTOCOL,
            "case_id": "test-case",
            "mode": "behavior",
            "source_digest": "sha256:src",
            "declared_command": "cmd",
            "implementation": impl_rel,
            "implementation_digest": impl_digest,
            "exit_code": 0,
            "assertions": ["ok"],
            "attempts": [
                {
                    "exit_code": 0,
                    "assertion_count": 1
                }
            ]
        });

        fs::write(&evidence_path, serde_json::to_vec_pretty(&valid_json).unwrap()).unwrap();

        let case = python_ec::PythonEcCase {
            id: "test-case".to_string(),
            artifact_id: "artifact:demo/test".to_string(),
            capability_id: "cap".to_string(),
            use_case_id: "uc".to_string(),
            dimension: "behavior".to_string(),
            applicability: "td".to_string(),
            test_path: impl_rel.to_string(),
            promise: "promise".to_string(),
            oracle: "oracle".to_string(),
            threshold: None,
            target: "python".to_string(),
            command: "cmd".to_string(),
            evidence_paths: vec![evidence_rel.to_string()],
            known_failure: None,
        };

        let eval = evaluate_case_evidence(ec_root, &case, "sha256:src");
        assert!(eval.evidence_ready);
        assert_eq!(eval.blocker, None);

        let eval_stale = evaluate_case_evidence(ec_root, &case, "sha256:stale");
        assert!(!eval_stale.evidence_ready);
        assert_eq!(
            eval_stale.blocker,
            Some("Python EC case `test-case` evidence `evidence/test.json` is stale for the current source digest".to_string())
        );

        fs::write(&impl_path, b"def verify(): return ['drifted']\n").unwrap();
        let eval_impl_stale = evaluate_case_evidence(ec_root, &case, "sha256:src");
        assert!(!eval_impl_stale.evidence_ready);
        assert_eq!(
            eval_impl_stale.blocker,
            Some("Python EC case `test-case` evidence `evidence/test.json` is stale for `src/cases/test.py`".to_string())
        );
    }
}
// HANDWRITE-END
