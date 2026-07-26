use agentic_workflow::{
    cli::{project::mutation_health_summary, validate_emitted_aw_command},
    services::{
        python_ec::discover_python_ec_inventory,
        python_td::compile_python_td_project,
        python_td_mutation::{enumerate_python_td_mutants, PythonTdMutationScope},
        python_td_mutation_evidence::{
            build_python_td_mutation_evidence, write_python_td_mutation_evidence,
            MutationEvidenceBindings,
        },
        python_td_mutation_health::{
            digest_source_tree, evaluate, MutationAdequacyPolicy, MutationAdequacyStatus,
        },
        python_td_mutation_runner::{
            MutationGateKind, MutationGateResult, MutationGateStatus, MutationRunResult,
            MutationVerdict, PythonTdNativeTarget, PYTHON_TD_MUTATION_RUN_SCHEMA,
        },
    },
};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn copy_tree(source: &Path, target: &Path) {
    std::fs::create_dir_all(target).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&source_path, &target_path);
        } else {
            std::fs::copy(source_path, target_path).unwrap();
        }
    }
}

fn repository(policy: &str) -> tempfile::TempDir {
    let repository = tempfile::tempdir().unwrap();
    let project = repository.path().join("projects/demo");
    copy_tree(
        &fixture("python_spec_typer").join("src"),
        &project.join("src"),
    );
    copy_tree(
        &fixture("python_spec_typer").join("tests"),
        &project.join("tests"),
    );
    copy_tree(
        &fixture("python-project").join("external-contracts"),
        &project.join("external-contracts"),
    );
    write_config(repository.path(), policy);
    repository
}

fn write_config(root: &Path, policy: &str) {
    std::fs::write(
        root.join("aw.toml"),
        r#"[[projects]]
name = "demo"
path = "projects/demo"
td_path = "projects/demo"
spec_model = "python"
"#,
    )
    .unwrap();
    std::fs::write(
        root.join("projects/demo/aw.toml"),
        format!(
            r#"[project]
name = "demo"
mutation_adequacy = "{policy}"
mutation_evidence_dir = "evidence/mutation-adequacy"
mutation_source_path = "src"
"#
        ),
    )
    .unwrap();
}

fn digest(value: &str) -> String {
    format!("sha256:{:x}", Sha256::digest(value.as_bytes()))
}

fn killed_run(mutant_id: &str, target: PythonTdNativeTarget) -> MutationRunResult {
    MutationRunResult {
        schema_version: PYTHON_TD_MUTATION_RUN_SCHEMA.to_string(),
        mutant_id: mutant_id.to_string(),
        target,
        target_digest: digest(&format!("{mutant_id}:{}", target.as_str())),
        verdict: MutationVerdict::Killed,
        gates: vec![
            MutationGateResult {
                gate_id: "unit".to_string(),
                kind: MutationGateKind::Unit,
                command: "fixture-unit".to_string(),
                status: MutationGateStatus::Failed,
                exit_code: 1,
                executed_tests: Some(1),
                compiled_target_marker: Some(format!("compiled-target:{}", target.as_str())),
                stdout: "running 1 test".to_string(),
                stderr: "mutation killed".to_string(),
            },
            MutationGateResult {
                gate_id: "ec".to_string(),
                kind: MutationGateKind::ExternalContract,
                command: "fixture-ec".to_string(),
                status: MutationGateStatus::Failed,
                exit_code: 1,
                executed_tests: Some(1),
                compiled_target_marker: None,
                stdout: "running 1 test".to_string(),
                stderr: "contract rejected mutant".to_string(),
            },
        ],
    }
}

#[test]
fn advisory_missing_evidence_reports_without_becoming_required() {
    let repository = repository("advisory");
    let report = evaluate(repository.path(), "demo").unwrap();
    assert_eq!(report.policy, MutationAdequacyPolicy::Advisory);
    assert_eq!(report.status, MutationAdequacyStatus::Missing);
    assert!(!report.ready);
    assert!(!report.required_for_production);
    validate_emitted_aw_command(report.next_command.as_deref().unwrap()).unwrap();
    let health = mutation_health_summary("demo", &report);
    assert_eq!(health["status"], "done");
    assert_eq!(health["next"]["kind"], "done");
}

#[test]
fn required_missing_evidence_routes_to_chain_valid_goal() {
    let repository = repository("required");
    let report = evaluate(repository.path(), "demo").unwrap();
    assert_eq!(report.policy, MutationAdequacyPolicy::Required);
    assert_eq!(report.status, MutationAdequacyStatus::Missing);
    assert!(!report.ready);
    assert!(report.required_for_production);
    assert_eq!(
        report.next_command.as_deref(),
        Some(
            r#"aw goal set --gate "aw health --project demo mutation" "Produce complete digest-bound mutation evidence for demo""#
        )
    );
    validate_emitted_aw_command(report.next_command.as_deref().unwrap()).unwrap();
    let health = mutation_health_summary("demo", &report);
    assert_eq!(health["status"], "blocked");
    assert_eq!(health["completion"]["workflow_complete"], false);
    validate_emitted_aw_command(health["next"]["command"].as_str().unwrap()).unwrap();
}

#[test]
fn complete_killed_inventory_is_adequate() {
    let repository = repository("required");
    let project = repository.path().join("projects/demo");
    let ir = compile_python_td_project(&project).unwrap();
    let ec = discover_python_ec_inventory(&project.join("external-contracts")).unwrap();
    let bindings = MutationEvidenceBindings {
        td_digest: ir.semantic_digest.clone(),
        ec_digest: ec.source_digest,
        source_digest: digest_source_tree(&project.join("src")).unwrap(),
    };
    let evidence_dir = project.join("evidence/mutation-adequacy");
    for (index, mutant) in enumerate_python_td_mutants(&ir)
        .unwrap()
        .into_iter()
        .enumerate()
    {
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
            let run = killed_run(&mutant.descriptor.id, *target);
            let evidence =
                build_python_td_mutation_evidence(bindings.clone(), &mutant, &run).unwrap();
            write_python_td_mutation_evidence(
                &evidence_dir.join(format!("{index}-{}.json", target.as_str())),
                &evidence,
            )
            .unwrap();
        }
    }

    let report = evaluate(repository.path(), "demo").unwrap();
    assert_eq!(report.status, MutationAdequacyStatus::Adequate);
    assert!(report.ready);
    assert!(report.required_for_production);
    assert_eq!(report.expected_run_count, report.evidence_run_count);
    assert_eq!(report.expected_run_count, report.killed_count);
    assert_eq!(report.survived_count, 0);
    assert!(report.findings.is_empty());
    assert!(report.next_command.is_none());
    let health = mutation_health_summary("demo", &report);
    assert_eq!(health["status"], "done");
    assert_eq!(health["completion"]["workflow_complete"], true);
}
