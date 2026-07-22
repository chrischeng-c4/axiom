// HANDWRITE-BEGIN gap="missing-generator:ddd-meta-projection-contract" tracker="#2292" reason="The fixture suite proves stable-ID narrative ownership and reproducible rendering across all six DDD artifact surfaces."
//! DDD meta-projection contract tests.
//!
//! @spec apps/agentic-workflow/tech-design/core/logic/ddd-meta-projection-contract.md#unit-test

use agentic_workflow::context_map::{
    parse_context_map, parse_meta_projection, render_projection_index, DddNarrativeSurface,
};

fn context_map() -> agentic_workflow::context_map::DddContextMap {
    parse_context_map(include_str!("fixtures/ddd_context_map/valid.yaml")).unwrap()
}

#[test]
fn ddd_meta_projection_assigns_each_surface_its_declared_narrative_role() {
    let projection = parse_meta_projection(
        &context_map(),
        include_str!("fixtures/ddd_meta_projection/valid.yaml"),
    )
    .unwrap();

    for surface in [
        DddNarrativeSurface::Capabilities,
        DddNarrativeSurface::Readme,
        DddNarrativeSurface::Contributing,
        DddNarrativeSurface::Ec,
        DddNarrativeSurface::Td,
        DddNarrativeSurface::Source,
    ] {
        assert!(!surface.narrative_role().is_empty());
        assert!(projection
            .projections
            .iter()
            .any(|declaration| declaration.surface == surface));
    }
}

#[test]
fn ddd_meta_projection_rejects_duplicate_ownership() {
    let error = parse_meta_projection(
        &context_map(),
        include_str!("fixtures/ddd_meta_projection/duplicate_ownership.yaml"),
    )
    .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("duplicate DDD narrative ownership"),
        "unexpected error: {error:#}"
    );
}

#[test]
fn ddd_meta_projection_rejects_unresolved_stable_identity_references() {
    let error = parse_meta_projection(
        &context_map(),
        include_str!("fixtures/ddd_meta_projection/unresolved_identity.yaml"),
    )
    .unwrap_err();

    assert!(
        error.to_string().contains("unresolved stable identity"),
        "unexpected error: {error:#}"
    );
}

#[test]
fn ddd_meta_projection_rejects_forbidden_surface_ownership() {
    let error = parse_meta_projection(
        &context_map(),
        include_str!("fixtures/ddd_meta_projection/forbidden_ownership.yaml"),
    )
    .unwrap_err();

    assert!(
        error.to_string().contains("may not own fact `promise`"),
        "unexpected error: {error:#}"
    );
}

#[test]
fn ddd_meta_projection_regenerates_the_same_index_after_markdown_changes() {
    let context_map = context_map();
    let original = parse_meta_projection(
        &context_map,
        include_str!("fixtures/ddd_meta_projection/valid.yaml"),
    )
    .unwrap();
    let rerendered = parse_meta_projection(
        &context_map,
        include_str!("fixtures/ddd_meta_projection/rerendered.yaml"),
    )
    .unwrap();

    assert_eq!(
        render_projection_index(&original),
        render_projection_index(&rerendered)
    );
}
// HANDWRITE-END
