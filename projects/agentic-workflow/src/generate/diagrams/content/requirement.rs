// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/content/requirement.md#source
// CODEGEN-BEGIN
//! RequirementContent — per-diagram Content type for requirements (requirementDiagram).
//!
//! Replaces the existing `requirement_plus/schema.rs` with a new Graph-based design.
//! Content is parsed from Mermaid Plus YAML frontmatter in spec files.

// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/requirement.md#source

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Type of requirement.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/requirement.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RequirementType {
    /// Functional (default).
    #[default]
    Functional,
    /// Performance.
    Performance,
    /// Interface.
    Interface,
    /// Physical.
    Physical,
    /// Design constraint.
    Design,
}

/// Risk level of a requirement.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/requirement.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    /// Low.
    Low,
    /// Medium (default).
    #[default]
    Medium,
    /// High.
    High,
}

/// Verification method for a requirement.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/requirement.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum VerifyMethod {
    /// Analysis.
    Analysis,
    /// Demonstration.
    Demonstration,
    /// Inspection.
    Inspection,
    /// Test (default).
    #[default]
    Test,
}

/// A single requirement entry.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/requirement.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub text: String,
    #[serde(rename = "type", default)]
    pub req_type: RequirementType,
    #[serde(default)]
    pub risk: RiskLevel,
    #[serde(default)]
    pub verification: VerifyMethod,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

/// An element that verifies or satisfies requirements.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/requirement.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Element {
    #[serde(default)]
    pub element_type: Option<String>,
    #[serde(default)]
    pub doc_ref: Option<String>,
}

/// Kind of relationship between elements and requirements.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/requirement.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RelationshipKind {
    Verifies,
    Refines,
    Copies,
    Contains,
    Derives,
    Satisfies,
    Traces,
}

/// A relationship between a requirement and an element or another requirement.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/requirement.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub from: String,
    pub to: String,
    pub kind: RelationshipKind,
}

/// Content type for `requirements` section (requirementDiagram).
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/requirement.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementContent {
    pub id: String,
    #[serde(default)]
    pub requirements: HashMap<String, Requirement>,
    #[serde(default)]
    pub elements: HashMap<String, Element>,
    #[serde(default)]
    pub relationships: Vec<Relationship>,
    #[serde(default)]
    pub title: Option<String>,
}

/// Content type for `requirements` section (requirementDiagram).
///
/// Parsed from Mermaid Plus YAML frontmatter:
/// ```yaml
/// id: my-requirements
/// requirements:
///   R1:
///     text: "The system shall ..."
///     type: functional
///     risk: low
///     verification: test
/// elements:
///   T1:
///     element_type: Test
/// relationships:
///   - from: T1
///     to: R1
///     kind: verifies
/// ```
// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/requirement.md#source
impl RequirementContent {
    /// Return IDs of all requirements verified by a given element.
    pub fn verified_by(&self, element_id: &str) -> Vec<&str> {
        self.relationships
            .iter()
            .filter(|r| r.from == element_id && r.kind == RelationshipKind::Verifies)
            .map(|r| r.to.as_str())
            .collect()
    }

    /// Return all requirement IDs with no verifying elements.
    pub fn uncovered_ids(&self) -> Vec<&str> {
        let verified: std::collections::HashSet<&str> = self
            .relationships
            .iter()
            .filter(|r| r.kind == RelationshipKind::Verifies)
            .map(|r| r.to.as_str())
            .collect();
        self.requirements
            .keys()
            .filter(|id| !verified.contains(id.as_str()))
            .map(|id| id.as_str())
            .collect()
    }
}

// CODEGEN-END
