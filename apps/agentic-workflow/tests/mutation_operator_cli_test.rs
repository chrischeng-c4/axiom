use agentic_workflow::services::{
    python_td::compile_python_td_project,
    python_td_mutation::{
        enumerate_python_td_mutants, supported_python_td_mutation_scopes, PythonTdMutationScope,
    },
};
use std::{collections::BTreeSet, path::PathBuf};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn repeated_enumeration_is_stable_for_every_supported_lowering() {
    let ir = compile_python_td_project(&fixture("python_spec_typer")).unwrap();
    let first = enumerate_python_td_mutants(&ir).unwrap();
    let second = enumerate_python_td_mutants(&ir).unwrap();
    let first_ids = first
        .iter()
        .map(|mutant| mutant.descriptor.id.clone())
        .collect::<Vec<_>>();
    let second_ids = second
        .iter()
        .map(|mutant| mutant.descriptor.id.clone())
        .collect::<Vec<_>>();

    assert!(!first_ids.is_empty());
    assert_eq!(first_ids, second_ids);
    assert_eq!(
        first_ids.iter().collect::<BTreeSet<_>>().len(),
        first_ids.len()
    );
    assert_eq!(
        supported_python_td_mutation_scopes(),
        &[
            PythonTdMutationScope::Semantic,
            PythonTdMutationScope::Python,
            PythonTdMutationScope::Rust,
            PythonTdMutationScope::TypeScript,
        ]
    );
    for scope in supported_python_td_mutation_scopes() {
        assert!(
            first.iter().any(|mutant| mutant.descriptor.scope == *scope),
            "{scope:?} emitted no mutants"
        );
    }
    for (left, right) in first.iter().zip(second.iter()) {
        assert_eq!(left.descriptor, right.descriptor);
        assert_eq!(left.mutated_semantic_digest, right.mutated_semantic_digest);
        assert_ne!(left.mutated_semantic_digest, ir.semantic_digest);
    }
}
