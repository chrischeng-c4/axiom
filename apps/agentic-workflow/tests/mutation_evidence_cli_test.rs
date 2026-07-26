use agentic_workflow::services::{
    python_td::compile_python_td_project,
    python_td_mutation::{enumerate_python_td_mutants, PythonTdMutationScope},
    python_td_mutation_evidence::{
        build_python_td_mutation_evidence, read_python_td_mutation_evidence,
        render_python_td_mutation_evidence, verify_python_td_mutation_evidence,
        write_python_td_mutation_evidence, MutationEvidenceBindings,
    },
    python_td_mutation_runner::{
        run_python_td_mutant, MutationGate, MutationGateKind, MutationRunOptions, MutationVerdict,
        PythonTdNativeTarget,
    },
};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn digest(value: &str) -> String {
    format!("sha256:{:x}", Sha256::digest(value.as_bytes()))
}

fn evidence_fixture() -> (
    agentic_workflow::services::python_td_mutation_evidence::PythonTdMutationEvidence,
    MutationEvidenceBindings,
) {
    let ir = compile_python_td_project(&fixture("python_spec_typer")).unwrap();
    let mutant = enumerate_python_td_mutants(&ir)
        .unwrap()
        .into_iter()
        .find(|mutant| mutant.descriptor.scope == PythonTdMutationScope::Python)
        .unwrap();
    let stage = tempfile::tempdir().unwrap();
    let gates = vec![
        MutationGate {
            id: "native-unit".to_string(),
            kind: MutationGateKind::Unit,
            command: "printf 'running 1 test\\ncompiled-target:python\\n'".to_string(),
            compiled_target_marker: Some("compiled-target:python".to_string()),
        },
        MutationGate {
            id: "external-contract".to_string(),
            kind: MutationGateKind::ExternalContract,
            command: "printf 'contract assertion passed\\n'".to_string(),
            compiled_target_marker: None,
        },
    ];
    let run = run_python_td_mutant(
        &mutant,
        PythonTdNativeTarget::Python,
        &stage.path().join("target"),
        &gates,
        &MutationRunOptions::default(),
    )
    .unwrap();
    let bindings = MutationEvidenceBindings {
        td_digest: ir.semantic_digest,
        ec_digest: digest("ec inventory"),
        source_digest: digest("baseline source"),
    };
    let evidence = build_python_td_mutation_evidence(bindings.clone(), &mutant, &run).unwrap();
    (evidence, bindings)
}

#[test]
fn evidence_round_trips_reproducibly() {
    let (evidence, bindings) = evidence_fixture();
    let first = render_python_td_mutation_evidence(&evidence).unwrap();
    let second = render_python_td_mutation_evidence(&evidence).unwrap();
    assert_eq!(first, second);

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("mutation.json");
    write_python_td_mutation_evidence(&path, &evidence).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), first);
    let loaded = read_python_td_mutation_evidence(&path, &bindings).unwrap();
    assert_eq!(loaded, evidence);
}

#[test]
fn any_bound_input_gate_or_verdict_tamper_is_rejected() {
    let (evidence, bindings) = evidence_fixture();

    for current in [
        MutationEvidenceBindings {
            td_digest: digest("different td"),
            ..bindings.clone()
        },
        MutationEvidenceBindings {
            ec_digest: digest("different ec"),
            ..bindings.clone()
        },
        MutationEvidenceBindings {
            source_digest: digest("different source"),
            ..bindings.clone()
        },
    ] {
        assert!(verify_python_td_mutation_evidence(&evidence, &current).is_err());
    }

    let mut gate = evidence.clone();
    gate.gates[0].command.push_str(" # tampered");
    assert!(verify_python_td_mutation_evidence(&gate, &bindings).is_err());

    let mut output = evidence.clone();
    output.gates[0].stdout.push_str("tampered");
    assert!(verify_python_td_mutation_evidence(&output, &bindings).is_err());

    let mut target = evidence.clone();
    target.target_digest = digest("different target");
    assert!(verify_python_td_mutation_evidence(&target, &bindings).is_err());

    let mut mutant = evidence.clone();
    mutant.mutated_semantic_digest = digest("different mutant");
    assert!(verify_python_td_mutation_evidence(&mutant, &bindings).is_err());

    let mut verdict = evidence.clone();
    verdict.verdict = match verdict.verdict {
        MutationVerdict::Killed => MutationVerdict::Survived,
        MutationVerdict::Survived => MutationVerdict::Killed,
    };
    assert!(verify_python_td_mutation_evidence(&verdict, &bindings).is_err());
}
