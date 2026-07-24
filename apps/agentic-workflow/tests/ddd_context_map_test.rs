// HANDWRITE-BEGIN gap="missing-generator:ddd-context-map-contract" tracker="#2291" reason="Static fixtures exercise stable DDD identity and relation invariants beyond the current generator's schema coverage."
//! DDD context-map contract tests.
//!
//! @spec apps/agentic-workflow/tech-design/core/logic/ddd-context-map-contract.md#unit-test

use agentic_workflow::context_map::{
    parse_context_map, DddContextMap, DddIdentityKind, DddRelationKind,
};

fn logical_signature(
    map: &DddContextMap,
) -> (
    Vec<(String, DddIdentityKind)>,
    Vec<(String, DddRelationKind, String)>,
) {
    let mut identities = map
        .nodes
        .iter()
        .map(|node| (node.id.clone(), node.kind))
        .collect::<Vec<_>>();
    identities.sort();
    let mut relations = map
        .relations
        .iter()
        .map(|relation| (relation.from.clone(), relation.kind, relation.to.clone()))
        .collect::<Vec<_>>();
    relations.sort();
    (identities, relations)
}

#[test]
fn ddd_context_map_keeps_logical_identity_when_projections_move() {
    let original = parse_context_map(include_str!("fixtures/ddd_context_map/valid.yaml")).unwrap();
    let moved = parse_context_map(include_str!(
        "fixtures/ddd_context_map/moved_projections.yaml"
    ))
    .unwrap();

    assert_eq!(logical_signature(&original), logical_signature(&moved));
    assert_ne!(original.nodes[0].projections, moved.nodes[0].projections);
}

#[test]
fn ddd_context_map_rejects_cross_context_dependency_in_the_wrong_direction() {
    let error = parse_context_map(include_str!(
        "fixtures/ddd_context_map/invalid_reverse_dependency.yaml"
    ))
    .unwrap_err();

    assert!(
        error.to_string().contains(
            "requires declared context dependency context:verification -> context:workflow"
        ),
        "unexpected error: {error:#}"
    );
}

#[test]
fn ddd_context_map_rejects_duplicate_identity_ownership() {
    let error = parse_context_map(include_str!(
        "fixtures/ddd_context_map/duplicate_identity.yaml"
    ))
    .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("duplicate DDD identity ownership"),
        "unexpected error: {error:#}"
    );
}

#[test]
fn ddd_context_map_rejects_paths_and_markdown_anchors_as_identity() {
    let error =
        parse_context_map(include_str!("fixtures/ddd_context_map/path_identity.yaml")).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("never a path or Markdown anchor"),
        "unexpected error: {error:#}"
    );
}
// HANDWRITE-END
