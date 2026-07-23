// HANDWRITE-BEGIN gap="missing-generator:python-artifact-code-check" tracker="#2305" reason="The terminal graph verifier composes compiler and target manifests until the Python artifact protocol generator owns the closure."
//! Cold target verification for the Python-v1 terminal artifact graph.

use super::{
    project_registry,
    python_ec::{self, PythonEcInventory},
    python_td::{compile_python_td_project, PythonTdIr},
    python_td_target::emit_python_td_target,
};
use anyhow::{Context, Result};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    process::Command,
};

use crate::models::project::ProjectArtifactModel;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PythonTargetBuildCheck {
    pub td_semantic_digest: String,
    pub target_build_digest: String,
    pub clean: bool,
    pub drifted_paths: Vec<String>,
}

/// Terminal graph result for an opt-in Python artifact project.  Every value
/// is derived from the current sources; a clean result therefore cannot be
/// reused after TD, generated-target, lock, or EC-inventory drift.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PythonArtifactCodeCheck {
    pub project: String,
    pub td_semantic_digest: String,
    pub target_build_digest: String,
    pub td_lock_clean: bool,
    pub ec_lock_clean: bool,
    pub native_unit_clean: bool,
    pub clean: bool,
    pub artifact_ids: Vec<String>,
    pub findings: Vec<String>,
    pub next_command: String,
}

/// Compile the TD into a fresh target directory, then compare only the files
/// owned by the emitter manifest. Unrelated product files are intentionally
/// outside this comparison and cannot create either a false red or a false
/// green for generated output.
pub fn verify_python_target_build(
    td_root: &Path,
    output_root: &Path,
) -> Result<PythonTargetBuildCheck> {
    let ir = compile_python_td_project(td_root)?;
    let cold = tempfile::tempdir().context("create Python TD cold output directory")?;
    let target = emit_python_td_target(&ir, cold.path())?;
    let mut drifted_paths = Vec::new();
    for file in &target.files {
        let expected = cold.path().join(&file.path);
        let actual = output_root.join(&file.path);
        let matches = match (fs::read(&expected), fs::read(&actual)) {
            (Ok(expected), Ok(actual)) => expected == actual,
            _ => false,
        };
        if !matches {
            drifted_paths.push(file.path.clone());
        }
    }
    drifted_paths.sort();
    Ok(PythonTargetBuildCheck {
        td_semantic_digest: ir.semantic_digest,
        target_build_digest: target.digest,
        clean: drifted_paths.is_empty(),
        drifted_paths,
    })
}

/// Verify the Python-v1 terminal graph without mutating the project.  Legacy
/// projects return `None` and retain the established Markdown code-check
/// lifecycle.  Python projects must prove: clean TD and EC locks, explicit
/// DDD artifact identities, dimension/target applicability, a fresh emitted
/// Python target, and its target-native unit inventory.
pub fn verify_python_artifact_code_check(
    project_root: &Path,
    project: &str,
) -> Result<Option<PythonArtifactCodeCheck>> {
    let row = project_registry::resolve_project_config_row(project_root, project)?;
    if row.effective_artifact_model() != ProjectArtifactModel::PythonV1 {
        return Ok(None);
    }

    let artifact_root = project_root.join(&row.path);
    let td_root = artifact_root.join("tech-design");
    let ir = compile_python_td_project(&td_root)?;
    let target = verify_python_target_build(&td_root, &artifact_root)?;
    let td_lock = crate::cli::td_lock::check_project_td_lock_at_root(project_root, &row.name)?;
    let ec_lock = crate::cli::ec::project_ec_lock_status_at_root(project_root, &row.name)?;
    let inventory =
        python_ec::discover_python_ec_inventory(&artifact_root.join("external-contracts"))?;
    let workspace_targets = project_registry::load_projects(project_root)?
        .into_iter()
        .find(|configured| configured.name == row.name)
        .map(|configured| {
            configured
                .workspaces
                .into_iter()
                .map(|workspace| match workspace.target {
                    crate::models::tech_stack::Language::Python => "python",
                    crate::models::tech_stack::Language::Rust => "rust",
                    crate::models::tech_stack::Language::TypeScript => "typescript",
                    crate::models::tech_stack::Language::JavaScript => "javascript",
                    crate::models::tech_stack::Language::Schemas => "schemas",
                })
                .collect::<BTreeSet<_>>()
        })
        .ok_or_else(|| {
            anyhow::anyhow!(
                "project `{}` disappeared from the project registry",
                row.name
            )
        })?;

    let inventory_clean = inventory.findings.is_empty();
    let mut findings = inventory.findings.clone();
    if !td_lock.clean {
        findings.push(format!("TD lock is not clean: {}", td_lock.message));
    }
    if !ec_lock.clean {
        findings.push(format!("EC lock is not clean: {}", ec_lock.message));
    }
    for path in &target.drifted_paths {
        findings.push(format!("generated Python target drifted: {path}"));
    }

    let artifact_ids = validate_identity_edges(&ir, &inventory, &workspace_targets, &mut findings);
    let native_unit_clean = if target.clean {
        match run_native_unit_inventory(&artifact_root) {
            Ok(()) => true,
            Err(error) => {
                findings.push(format!(
                    "generated Python native unit test failed: {error:#}"
                ));
                false
            }
        }
    } else {
        findings.push(
            "generated Python native unit test was not run because the target is stale".to_string(),
        );
        false
    };

    findings.sort();
    findings.dedup();
    let clean = findings.is_empty();
    let next_command = if !inventory_clean {
        format!("aw ec check --project {}", row.name)
    } else if !ec_lock.clean {
        format!("aw ec review --project {}", row.name)
    } else {
        // Identity, TD-lock, target, and native-unit failures are repaired by
        // the TD/generation side. `cb` substitutes its root WI slug here.
        format!("aw cb gen --project {}", row.name)
    };
    Ok(Some(PythonArtifactCodeCheck {
        project: row.name,
        td_semantic_digest: ir.semantic_digest,
        target_build_digest: target.target_build_digest,
        td_lock_clean: td_lock.clean,
        ec_lock_clean: ec_lock.clean,
        native_unit_clean,
        clean,
        artifact_ids,
        findings,
        next_command,
    }))
}

fn validate_identity_edges(
    ir: &PythonTdIr,
    inventory: &PythonEcInventory,
    workspace_targets: &BTreeSet<&str>,
    findings: &mut Vec<String>,
) -> Vec<String> {
    let mut artifacts = BTreeSet::new();
    for module in ir
        .modules
        .iter()
        .filter(|module| module.path.starts_with("src/"))
    {
        match module.artifact_id.as_deref() {
            Some(id) => {
                artifacts.insert(id.to_string());
            }
            None => findings.push(format!(
                "Python TD module `{}` has no explicit artifact:<context>/<name> identity",
                module.path
            )),
        }
    }
    if artifacts.is_empty() {
        findings.push("Python TD declares no explicit artifact identities under src/*".to_string());
    }

    let mut dimensions = BTreeMap::<String, BTreeSet<String>>::new();
    for case in &inventory.cases {
        if !artifacts.contains(&case.artifact_id) {
            findings.push(format!(
                "Python EC case `{}` references undeclared TD artifact `{}`",
                case.id, case.artifact_id
            ));
        }
        if !workspace_targets.contains(case.target.as_str()) {
            findings.push(format!(
                "Python EC case `{}` targets `{}`, which has no configured project workspace",
                case.id, case.target
            ));
        }
        dimensions
            .entry(case.artifact_id.clone())
            .or_default()
            .insert(case.dimension.clone());
    }

    let rust_targeted = workspace_targets.contains("rust");
    for artifact in &artifacts {
        let declared = dimensions.get(artifact).cloned().unwrap_or_default();
        for dimension in ["behavior", "security", "stability"] {
            if !declared.contains(dimension) {
                findings.push(format!(
                    "artifact `{artifact}` is missing required `{dimension}` EC coverage"
                ));
            }
        }
        if (inventory.efficiency_policy == "required" || rust_targeted)
            && !declared.contains("efficiency")
        {
            findings.push(format!(
                "artifact `{artifact}` is missing required `efficiency` EC coverage"
            ));
        }
    }
    artifacts.into_iter().collect()
}

fn run_native_unit_inventory(artifact_root: &Path) -> Result<()> {
    let output = Command::new("python3")
        .args(["-m", "unittest", "discover", "-s", "tests/unit"])
        .current_dir(artifact_root)
        .output()
        .context("run generated Python target-native unit inventory")?;
    if output.status.success() {
        return Ok(());
    }
    anyhow::bail!(
        "exit={}; stdout={}; stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout).trim(),
        String::from_utf8_lossy(&output.stderr).trim(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_graph_fixture(root: &Path) {
        fs::create_dir_all(root.join("projects/demo/tech-design/src/demo/domain")).unwrap();
        fs::create_dir_all(root.join("projects/demo/external-contracts/src")).unwrap();
        fs::create_dir_all(root.join("projects/demo/external-contracts/evidence")).unwrap();
        fs::write(
            root.join("aw.toml"),
            r#"
[[projects]]
name = "demo"
path = "projects/demo"
artifact_model = "python-v1"

[[projects.workspaces]]
name = "python"
paths = ["projects/demo/**"]
target = "python"
test_cmd = "python3 -m unittest discover -s tests/unit"
"#,
        )
        .unwrap();
        fs::write(
            root.join("projects/demo/tech-design/src/demo/domain/order.py"),
            "__aw_artifact_id__ = \"artifact:orders/create-order\"\n\nclass Order:\n    pass\n",
        )
        .unwrap();
        fs::write(
            root.join("projects/demo/external-contracts/pyproject.toml"),
            r#"
[tool.aw.python-artifact]
protocol = "aw.python-artifact.v1"
entrypoint = "src/runner.py"
source_roots = ["src"]
dependency_files = ["pyproject.toml"]
evidence_dir = "evidence"

[tool.aw.python-ec]
protocol = "aw.python-ec.v1"
author = "fixture-author"
efficiency_policy = "required"

[[tool.aw.python-ec.cases]]
id = "order-behavior"
artifact_id = "artifact:orders/create-order"
capability_id = "orders"
use_case_id = "create-order"
dimension = "behavior"
applicability = "td"
test_path = "src/behavior.py"
promise = "orders are created"
oracle = "fixture-target"
target = "python"
command = "true"
evidence_paths = ["evidence/behavior.json"]

[[tool.aw.python-ec.cases]]
id = "order-security"
artifact_id = "artifact:orders/create-order"
capability_id = "orders"
use_case_id = "create-order"
dimension = "security"
applicability = "td"
test_path = "src/security.py"
promise = "orders reject unauthorized changes"
oracle = "fixture-target"
target = "python"
command = "true"
evidence_paths = ["evidence/security.json"]

[[tool.aw.python-ec.cases]]
id = "order-stability"
artifact_id = "artifact:orders/create-order"
capability_id = "orders"
use_case_id = "restart"
dimension = "stability"
applicability = "post-gen"
test_path = "src/stability.py"
promise = "orders survive restart"
oracle = "fixture-target"
threshold = "5 seconds"
target = "python"
command = "true"
evidence_paths = ["evidence/stability.json"]

[[tool.aw.python-ec.cases]]
id = "order-efficiency"
artifact_id = "artifact:orders/create-order"
capability_id = "orders"
use_case_id = "latency"
dimension = "efficiency"
applicability = "post-gen"
test_path = "src/efficiency.py"
promise = "orders meet latency budget"
oracle = "fixture-target"
threshold = "p95 under 100ms"
target = "python"
command = "true"
evidence_paths = ["evidence/efficiency.json"]
"#,
        )
        .unwrap();
        for name in ["runner", "behavior", "security", "stability", "efficiency"] {
            fs::write(
                root.join("projects/demo/external-contracts/src")
                    .join(format!("{name}.py")),
                "def contract() -> None:\n    pass\n",
            )
            .unwrap();
        }
        for name in ["behavior", "security", "stability", "efficiency"] {
            fs::write(
                root.join("projects/demo/external-contracts/evidence")
                    .join(format!("{name}.json")),
                "{\"ok\":true}\n",
            )
            .unwrap();
        }
        let td_root = root.join("projects/demo/tech-design");
        let artifact_root = root.join("projects/demo");
        let ir = compile_python_td_project(&td_root).unwrap();
        emit_python_td_target(&ir, &artifact_root).unwrap();
        assert!(
            crate::cli::td_lock::write_project_td_lock_snapshot_at_root(root, "demo")
                .unwrap()
                .clean
        );
        assert!(
            crate::cli::ec::write_project_ec_lock_snapshot_at_root(root, "demo")
                .unwrap()
                .clean
        );
    }

    #[test]
    fn cold_python_target_build_detects_only_manifest_owned_drift() {
        let td = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        let source = td.path().join("src/demo/domain/invoice.py");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(
            &source,
            "__aw_artifact_id__ = \"artifact:billing/issue-invoice\"\n\ndef issue_invoice() -> None:\n    pass\n",
        )
        .unwrap();
        let ir = compile_python_td_project(td.path()).unwrap();
        emit_python_td_target(&ir, output.path()).unwrap();
        fs::write(output.path().join("notes.txt"), "user-owned\n").unwrap();

        let clean = verify_python_target_build(td.path(), output.path()).unwrap();
        assert!(clean.clean, "{clean:#?}");
        assert!(clean.drifted_paths.is_empty());

        fs::write(
            output.path().join("src/demo/domain/invoice.py"),
            "changed\n",
        )
        .unwrap();
        let drifted = verify_python_target_build(td.path(), output.path()).unwrap();
        assert!(!drifted.clean);
        assert_eq!(drifted.drifted_paths, vec!["src/demo/domain/invoice.py"]);
    }

    #[test]
    fn python_artifact_code_check_closes_graph_and_rejects_stale_target() {
        let root = tempfile::tempdir().unwrap();
        write_graph_fixture(root.path());

        let clean = verify_python_artifact_code_check(root.path(), "demo")
            .unwrap()
            .unwrap();
        assert!(clean.clean, "{clean:#?}");
        assert_eq!(clean.artifact_ids, vec!["artifact:orders/create-order"]);
        assert!(clean.td_lock_clean && clean.ec_lock_clean && clean.native_unit_clean);

        fs::write(
            root.path().join("projects/demo/src/demo/domain/order.py"),
            "stale\n",
        )
        .unwrap();
        let stale = verify_python_artifact_code_check(root.path(), "demo")
            .unwrap()
            .unwrap();
        assert!(!stale.clean);
        assert!(stale
            .findings
            .iter()
            .any(|finding| finding
                .contains("generated Python target drifted: src/demo/domain/order.py")));
    }

    #[test]
    fn python_artifact_code_check_keeps_identity_valid_across_projection_move() {
        let root = tempfile::tempdir().unwrap();
        write_graph_fixture(root.path());
        let before = verify_python_artifact_code_check(root.path(), "demo")
            .unwrap()
            .unwrap();

        let td_root = root.path().join("projects/demo/tech-design");
        let old = td_root.join("src/demo/domain/order.py");
        let moved = td_root.join("src/demo/domain/create_order.py");
        fs::create_dir_all(moved.parent().unwrap()).unwrap();
        fs::rename(old, &moved).unwrap();
        let ir = compile_python_td_project(&td_root).unwrap();
        emit_python_td_target(&ir, &root.path().join("projects/demo")).unwrap();
        crate::cli::td_lock::write_project_td_lock_snapshot_at_root(root.path(), "demo").unwrap();

        let after = verify_python_artifact_code_check(root.path(), "demo")
            .unwrap()
            .unwrap();
        assert!(after.clean, "{after:#?}");
        assert_eq!(before.td_semantic_digest, after.td_semantic_digest);
        assert_eq!(before.artifact_ids, after.artifact_ids);
    }

    #[test]
    fn python_artifact_code_check_rejects_missing_required_dimension_and_target() {
        let root = tempfile::tempdir().unwrap();
        write_graph_fixture(root.path());
        let inventory = root
            .path()
            .join("projects/demo/external-contracts/pyproject.toml");
        let source = fs::read_to_string(&inventory).unwrap();
        let missing_security_and_bad_target = source
            .replace("dimension = \"security\"", "dimension = \"behavior\"")
            .replace(
            "target = \"python\"\ncommand = \"true\"\nevidence_paths = [\"evidence/security.json\"]\n\n[[tool.aw.python-ec.cases]]\nid = \"order-stability\"",
            "target = \"rust\"\ncommand = \"true\"\nevidence_paths = [\"evidence/security.json\"]\n\n[[tool.aw.python-ec.cases]]\nid = \"order-stability\"",
        );
        fs::write(&inventory, missing_security_and_bad_target).unwrap();

        let report = verify_python_artifact_code_check(root.path(), "demo")
            .unwrap()
            .unwrap();
        assert!(!report.clean);
        assert!(report
            .findings
            .iter()
            .any(|finding| finding
                .contains("targets `rust`, which has no configured project workspace")));
        assert!(report
            .findings
            .iter()
            .any(|finding| finding.contains("missing required `security` EC coverage")));
    }
}
// HANDWRITE-END
