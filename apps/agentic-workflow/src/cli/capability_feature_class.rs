//! Capability FEATURE CLASS -> which logical root a capability belongs to.
//!
//! A capability's *feature class* answers one question: is this capability a
//! **core** feature of the product, or a **non-core** one? The class is the
//! document-level partition behind the canonical `Core Features` and
//! `Non-Core Features` roots in a project's `CAPABILITIES.md`.
//!
//! The class is deliberately **orthogonal** to every other capability axis, and
//! must never be conflated with them:
//!
//! - [`crate::cli::capability_type::CapabilityType`] (AgentFirst / Service /
//!   Devops / DeveloperTool / RuntimeTool / SecurityTool) is a *structural*
//!   classification that decides **which EC dimensions are
//!   production-required**. A core capability and a non-core one may share the
//!   same type, and a single type spans both classes. Reusing `CapabilityType`
//!   to express core-ness would make the required-dimension set depend on
//!   product prioritization, which it must not.
//! - Maturity (`smoke` / `conformance` / …) decides only *whether* a gate is
//!   verified or runnable. A non-core capability may carry the strictest
//!   maturity, and a core one the weakest.
//! - `required_for_production` stays derived from the type plus the EC
//!   dimension state. The feature class never flips it.
//!
//! The class carries no work-item, priority, dependency, or execution state:
//! those live in the tracker, not in `CAPABILITIES.md`. It is a stable
//! statement about the product's own shape.
//!
//! Trait-derived baselines are always [`CapabilityFeatureClass::NonCore`]: a
//! capability that exists because an archetype trait demands it is, by
//! construction, not part of what makes this product itself.
//!
//! The class is read from the explicit `Feature Class:` field (or a
//! `Feature Class` contract-table column) in a capability's canonical contract,
//! and from `feature_class:` in the YAML-fenced form. It is optional at parse
//! time so existing unclassified documents keep parsing; requiring a class is a
//! separate, later gate.

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Which logical feature root a capability belongs to.
///
/// @spec apps/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md#schema
///
/// A closed two-member enumeration: a capability is either core or non-core,
/// and there is no third class. Quality gates, verification tiers, and
/// archetype baselines are expressed as claims and gates *within* a
/// capability — never as an additional feature class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityFeatureClass {
    /// Part of what the product fundamentally is; belongs under `Core Features`.
    Core,
    /// Supporting or trait-derived; belongs under `Non-Core Features`.
    NonCore,
}

impl CapabilityFeatureClass {
    /// The exact string used on disk, in documents, and in HITL choices.
    pub fn as_str(self) -> &'static str {
        match self {
            CapabilityFeatureClass::Core => "core",
            CapabilityFeatureClass::NonCore => "non_core",
        }
    }

    /// The canonical document root heading for this class.
    pub fn root_heading(self) -> &'static str {
        match self {
            CapabilityFeatureClass::Core => "Core Features",
            CapabilityFeatureClass::NonCore => "Non-Core Features",
        }
    }

    /// Parse a CLI / document / HITL-answer string into a
    /// [`CapabilityFeatureClass`].
    ///
    /// Accepts the canonical `core` and `non_core` plus the spellings a human
    /// author is likely to write in Markdown (`non-core`, `noncore`,
    /// `Non-Core Features`), case-insensitively. Anything else is an error
    /// rather than a silent default, because a wrong class silently moves a
    /// capability between the two logical roots.
    pub fn from_cli_str(value: &str) -> Result<CapabilityFeatureClass> {
        let normalized = value
            .trim()
            .trim_matches('`')
            .to_ascii_lowercase()
            .replace(['-', ' '], "_");
        match normalized.trim_end_matches("_features") {
            "core" => Ok(CapabilityFeatureClass::Core),
            "non_core" | "noncore" => Ok(CapabilityFeatureClass::NonCore),
            _ => anyhow::bail!(
                "unknown capability feature class `{}`; expected core or non_core",
                value.trim()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_str_round_trips_through_from_cli_str() {
        for class in [
            CapabilityFeatureClass::Core,
            CapabilityFeatureClass::NonCore,
        ] {
            assert_eq!(
                CapabilityFeatureClass::from_cli_str(class.as_str()).unwrap(),
                class
            );
        }
    }

    #[test]
    fn from_cli_str_accepts_human_spellings() {
        for value in ["core", "Core", " CORE ", "`core`", "Core Features"] {
            assert_eq!(
                CapabilityFeatureClass::from_cli_str(value).unwrap(),
                CapabilityFeatureClass::Core,
                "{value}"
            );
        }
        for value in [
            "non_core",
            "non-core",
            "NonCore",
            "Non-Core",
            "Non-Core Features",
        ] {
            assert_eq!(
                CapabilityFeatureClass::from_cli_str(value).unwrap(),
                CapabilityFeatureClass::NonCore,
                "{value}"
            );
        }
    }

    #[test]
    fn from_cli_str_rejects_anything_outside_the_closed_pair() {
        // Notably: no third class, and no capability *type* is accepted here.
        for value in ["", "-", "optional", "baseline", "Service", "AgentFirst"] {
            let err = CapabilityFeatureClass::from_cli_str(value).unwrap_err();
            assert!(
                err.to_string().contains("expected core or non_core"),
                "{value}: {err}"
            );
        }
    }

    #[test]
    fn serde_uses_the_on_disk_spelling() {
        let core = serde_yaml::to_string(&CapabilityFeatureClass::Core).unwrap();
        let non_core = serde_yaml::to_string(&CapabilityFeatureClass::NonCore).unwrap();
        assert_eq!(core.trim(), "core");
        assert_eq!(non_core.trim(), "non_core");
        assert_eq!(
            serde_yaml::from_str::<CapabilityFeatureClass>("non_core").unwrap(),
            CapabilityFeatureClass::NonCore
        );
    }

    #[test]
    fn root_heading_names_the_two_canonical_roots() {
        assert_eq!(CapabilityFeatureClass::Core.root_heading(), "Core Features");
        assert_eq!(
            CapabilityFeatureClass::NonCore.root_heading(),
            "Non-Core Features"
        );
    }
}
