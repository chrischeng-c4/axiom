use agentic_workflow::services::{
    python_td::compile_python_td_project,
    python_td_mutation::{enumerate_python_td_mutants, PythonTdMutationScope},
    python_td_mutation_runner::{
        run_python_td_mutant, MutationGate, MutationGateKind, MutationRunOptions,
        PythonTdNativeTarget,
    },
};
use std::{collections::BTreeSet, path::PathBuf};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn targets_for_scope(scope: PythonTdMutationScope) -> Vec<PythonTdNativeTarget> {
    match scope {
        PythonTdMutationScope::Semantic => vec![
            PythonTdNativeTarget::Python,
            PythonTdNativeTarget::Rust,
            PythonTdNativeTarget::TypeScript,
        ],
        PythonTdMutationScope::Python => vec![PythonTdNativeTarget::Python],
        PythonTdMutationScope::Rust => vec![PythonTdNativeTarget::Rust],
        PythonTdMutationScope::TypeScript => vec![PythonTdNativeTarget::TypeScript],
    }
}

fn complete_gates(target: PythonTdNativeTarget) -> Vec<MutationGate> {
    let marker = format!("compiled-target:{}", target.as_str());
    vec![
        MutationGate {
            id: "native-unit".to_string(),
            kind: MutationGateKind::Unit,
            command: format!("printf 'running 1 test\\n{marker}\\n'"),
            compiled_target_marker: Some(marker),
        },
        MutationGate {
            id: "external-contract".to_string(),
            kind: MutationGateKind::ExternalContract,
            command: "printf 'contract assertion passed\\n'".to_string(),
            compiled_target_marker: None,
        },
    ]
}

#[test]
fn every_mutant_runs_reemission_unit_and_ec_gates() {
    let ir = compile_python_td_project(&fixture("python_spec_typer")).unwrap();
    let mutants = enumerate_python_td_mutants(&ir).unwrap();
    let stages = tempfile::tempdir().unwrap();
    let mut executed_ids = BTreeSet::new();

    let mut execution_count = 0;
    for (index, mutant) in mutants.iter().enumerate() {
        for target in targets_for_scope(mutant.descriptor.scope) {
            let output_root = stages.path().join(format!("{index}-{}", target.as_str()));
            let result = run_python_td_mutant(
                mutant,
                target,
                &output_root,
                &complete_gates(target),
                &MutationRunOptions::default(),
            )
            .unwrap();
            assert_eq!(result.gates.len(), 2);
            assert!(result
                .gates
                .iter()
                .any(|gate| gate.kind == MutationGateKind::Unit));
            assert!(result
                .gates
                .iter()
                .any(|gate| gate.kind == MutationGateKind::ExternalContract));
            executed_ids.insert(result.mutant_id);
            execution_count += 1;
        }
    }
    assert_eq!(executed_ids.len(), mutants.len());
    assert!(execution_count > mutants.len());
}

#[test]
fn zero_test_and_uncompiled_target_false_greens_are_rejected() {
    let ir = compile_python_td_project(&fixture("python_spec_typer")).unwrap();
    let mutant = enumerate_python_td_mutants(&ir)
        .unwrap()
        .into_iter()
        .find(|mutant| mutant.descriptor.scope == PythonTdMutationScope::Python)
        .unwrap();
    let stages = tempfile::tempdir().unwrap();
    let ec = MutationGate {
        id: "external-contract".to_string(),
        kind: MutationGateKind::ExternalContract,
        command: "printf 'contract assertion passed\\n'".to_string(),
        compiled_target_marker: None,
    };

    let zero_test = MutationGate {
        id: "native-unit".to_string(),
        kind: MutationGateKind::Unit,
        command: "printf 'running 0 tests\\ncompiled-target:python\\n'".to_string(),
        compiled_target_marker: Some("compiled-target:python".to_string()),
    };
    let zero_error = run_python_td_mutant(
        &mutant,
        PythonTdNativeTarget::Python,
        &stages.path().join("zero"),
        &[zero_test, ec.clone()],
        &MutationRunOptions::default(),
    )
    .unwrap_err();
    assert!(zero_error.to_string().contains("executed zero tests"));

    let uncompiled = MutationGate {
        id: "native-unit".to_string(),
        kind: MutationGateKind::Unit,
        command: "printf 'running 1 test\\n'".to_string(),
        compiled_target_marker: Some("compiled-target:python".to_string()),
    };
    let uncompiled_error = run_python_td_mutant(
        &mutant,
        PythonTdNativeTarget::Python,
        &stages.path().join("uncompiled"),
        &[uncompiled, ec],
        &MutationRunOptions::default(),
    )
    .unwrap_err();
    assert!(uncompiled_error
        .to_string()
        .contains("without compiled-target marker"));
}
