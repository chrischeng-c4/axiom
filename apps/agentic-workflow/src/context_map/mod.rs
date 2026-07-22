// HANDWRITE-BEGIN gap="missing-generator:ddd-context-map-contract" tracker="#2291" reason="DDD identity and relation validation is a new domain contract; the generator does not yet emit its bounded-context model or invariant checker."
//! Stable DDD identity and relationship validation.
//!
//! This bounded context owns logical identity only. File paths and Markdown
//! anchors are optional projections, never keys used to join contracts.
//!
//! @spec apps/agentic-workflow/tech-design/core/logic/ddd-context-map-contract.md#logic

pub mod model;
pub mod narrative_projection;
pub mod validate;

pub use model::{
    DddContextMap, DddIdentity, DddIdentityKind, DddProjection, DddRelation, DddRelationKind,
    DDD_CONTEXT_MAP_SCHEMA,
};
pub use narrative_projection::{
    parse_meta_projection, render_projection_index, validate_meta_projection, DddMetaProjection,
    DddNarrativeFact, DddNarrativeProjection, DddNarrativeSurface, DDD_META_PROJECTION_SCHEMA,
};
pub use validate::{parse_context_map, validate_context_map};
// HANDWRITE-END
