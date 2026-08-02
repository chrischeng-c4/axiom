// HANDWRITE-BEGIN gap="missing-generator:ddd-meta-projection-contract" tracker="#2292" reason="The narrative ownership matrix and stable-ID projection validator define a new DDD contract that the generator does not yet emit."
//! Narrative ownership projections for stable DDD identities.
//!
//! @spec apps/agentic-workflow/tech-design/core/logic/ddd-meta-projection-contract.md#logic

use super::{validate_context_map, DddContextMap};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const DDD_META_PROJECTION_SCHEMA: &str = "aw.ddd-meta-projection.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DddMetaProjection {
    pub schema_version: String,
    pub projections: Vec<DddNarrativeProjection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DddNarrativeProjection {
    pub surface: DddNarrativeSurface,
    pub identity: String,
    pub facts: Vec<DddNarrativeFact>,
    /// A disposable rendering captured by a producer; never used as identity.
    #[serde(default)]
    pub rendered_markdown: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DddNarrativeSurface {
    Capabilities,
    Readme,
    Contributing,
    Ec,
    Td,
    Source,
}

impl DddNarrativeSurface {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Capabilities => "capabilities",
            Self::Readme => "readme",
            Self::Contributing => "contributing",
            Self::Ec => "ec",
            Self::Td => "td",
            Self::Source => "source",
        }
    }

    pub const fn narrative_role(self) -> &'static str {
        match self {
            Self::Capabilities => "product promises",
            Self::Readme => "overview and journeys",
            Self::Contributing => "boundaries and authoring rules",
            Self::Ec => "externally observable truth",
            Self::Td => "executable construction",
            Self::Source => "DDD implementation and unit tests",
        }
    }

    const fn owns(self, fact: DddNarrativeFact) -> bool {
        matches!(
            (self, fact),
            (Self::Capabilities, DddNarrativeFact::Promise)
                | (Self::Readme, DddNarrativeFact::Overview)
                | (Self::Readme, DddNarrativeFact::Journey)
                | (Self::Contributing, DddNarrativeFact::Boundary)
                | (Self::Contributing, DddNarrativeFact::AuthoringRule)
                | (Self::Ec, DddNarrativeFact::ExternalTruth)
                | (Self::Td, DddNarrativeFact::ExecutableConstruction)
                | (Self::Source, DddNarrativeFact::Implementation)
                | (Self::Source, DddNarrativeFact::UnitTest)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DddNarrativeFact {
    Promise,
    Overview,
    Journey,
    Boundary,
    AuthoringRule,
    ExternalTruth,
    ExecutableConstruction,
    Implementation,
    UnitTest,
}

impl DddNarrativeFact {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Promise => "promise",
            Self::Overview => "overview",
            Self::Journey => "journey",
            Self::Boundary => "boundary",
            Self::AuthoringRule => "authoring-rule",
            Self::ExternalTruth => "external-truth",
            Self::ExecutableConstruction => "executable-construction",
            Self::Implementation => "implementation",
            Self::UnitTest => "unit-test",
        }
    }
}

/// Parse and validate a narrative-projection document against stable DDD IDs.
pub fn parse_meta_projection(context_map: &DddContextMap, yaml: &str) -> Result<DddMetaProjection> {
    let projection = serde_yaml::from_str(yaml).context("parse DDD meta-projection YAML")?;
    validate_meta_projection(context_map, &projection)?;
    Ok(projection)
}

/// Validate a one-owner narrative projection.
pub fn validate_meta_projection(
    context_map: &DddContextMap,
    projection: &DddMetaProjection,
) -> Result<()> {
    validate_context_map(context_map)
        .context("validate DDD context-map before narrative projection")?;
    if projection.schema_version != DDD_META_PROJECTION_SCHEMA {
        bail!(
            "unsupported DDD meta-projection schema `{}`; expected `{DDD_META_PROJECTION_SCHEMA}`",
            projection.schema_version
        );
    }

    let stable_ids = context_map
        .nodes
        .iter()
        .map(|identity| identity.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut ownership = BTreeSet::new();
    for declaration in &projection.projections {
        if !stable_ids.contains(declaration.identity.as_str()) {
            bail!(
                "DDD narrative projection references unresolved stable identity `{}`",
                declaration.identity
            );
        }
        if declaration.facts.is_empty() {
            bail!(
                "DDD narrative projection for `{}` on {} must declare at least one owned fact",
                declaration.identity,
                declaration.surface.as_str()
            );
        }
        for fact in &declaration.facts {
            if !declaration.surface.owns(*fact) {
                bail!(
                    "DDD narrative surface `{}` owns {}; it may not own fact `{}`",
                    declaration.surface.as_str(),
                    declaration.surface.narrative_role(),
                    fact.as_str()
                );
            }
            if !ownership.insert((declaration.identity.as_str(), *fact)) {
                bail!(
                    "duplicate DDD narrative ownership for stable identity `{}` fact `{}`",
                    declaration.identity,
                    fact.as_str()
                );
            }
        }
    }
    Ok(())
}

/// Render a deterministic, disposable index from owned facts alone.
///
/// The stored `rendered_markdown` field is intentionally not read, so producers
/// can regenerate the same index after a Markdown move or heading rewrite.
pub fn render_projection_index(projection: &DddMetaProjection) -> String {
    let mut entries = projection
        .projections
        .iter()
        .flat_map(|declaration| {
            declaration.facts.iter().map(move |fact| {
                (
                    declaration.surface.as_str(),
                    declaration.identity.as_str(),
                    fact.as_str(),
                )
            })
        })
        .collect::<Vec<_>>();
    entries.sort_unstable();

    let mut output = String::from("# DDD Narrative Projection Index\n\n");
    for (surface, identity, fact) in entries {
        output.push_str(&format!("- {surface}: `{identity}` owns `{fact}`\n"));
    }
    output
}

#[cfg(test)]
mod ddd_meta_projection_test {
    //! DDD meta-projection contract tests.
    //!
    //! @spec apps/agentic-workflow/tech-design/core/logic/ddd-meta-projection-contract.md#unit-test
    use super::*;
    use crate::context_map::parse_context_map;

    fn context_map() -> DddContextMap {
        parse_context_map(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/ddd_context_map/valid.yaml"
        )))
        .unwrap()
    }

    #[test]
    fn ddd_meta_projection_assigns_each_surface_its_declared_narrative_role() {
        let projection = parse_meta_projection(
            &context_map(),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/ddd_meta_projection/valid.yaml"
            )),
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
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/ddd_meta_projection/duplicate_ownership.yaml"
            )),
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
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/ddd_meta_projection/unresolved_identity.yaml"
            )),
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
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/ddd_meta_projection/forbidden_ownership.yaml"
            )),
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
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/ddd_meta_projection/valid.yaml"
            )),
        )
        .unwrap();
        let rerendered = parse_meta_projection(
            &context_map,
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/ddd_meta_projection/rerendered.yaml"
            )),
        )
        .unwrap();

        assert_eq!(
            render_projection_index(&original),
            render_projection_index(&rerendered)
        );
    }
}
// HANDWRITE-END
