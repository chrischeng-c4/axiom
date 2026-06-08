---
id: sdd-content-requirement
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# RequirementContent

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diagrams/content/requirement.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Element` | projects/agentic-workflow/src/generate/diagrams/content/requirement.rs | struct | pub | 82 |  |
| `Relationship` | projects/agentic-workflow/src/generate/diagrams/content/requirement.rs | struct | pub | 106 |  |
| `RelationshipKind` | projects/agentic-workflow/src/generate/diagrams/content/requirement.rs | enum | pub | 93 |  |
| `Requirement` | projects/agentic-workflow/src/generate/diagrams/content/requirement.rs | struct | pub | 65 |  |
| `RequirementContent` | projects/agentic-workflow/src/generate/diagrams/content/requirement.rs | struct | pub | 115 |  |
| `RequirementType` | projects/agentic-workflow/src/generate/diagrams/content/requirement.rs | enum | pub | 18 |  |
| `RiskLevel` | projects/agentic-workflow/src/generate/diagrams/content/requirement.rs | enum | pub | 36 |  |
| `VerifyMethod` | projects/agentic-workflow/src/generate/diagrams/content/requirement.rs | enum | pub | 50 |  |
| `uncovered_ids` | projects/agentic-workflow/src/generate/diagrams/content/requirement.rs | function | pub | 158 | uncovered_ids(&self) -> Vec<&str> |
| `verified_by` | projects/agentic-workflow/src/generate/diagrams/content/requirement.rs | function | pub | 149 | verified_by(&self, element_id: &str) -> Vec<&str> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  RequirementType:
    type: string
    enum: [Functional, Performance, Interface, Physical, Design]
    description: Type of requirement.
    x-rust-enum:
      derive: [Debug, Clone, PartialEq, Serialize, Deserialize, Default]
      serde_rename_all: lowercase
      variants:
        - { name: Functional, is_default: true, doc: "Functional (default)." }
        - { name: Performance,                  doc: "Performance." }
        - { name: Interface,                    doc: "Interface." }
        - { name: Physical,                     doc: "Physical." }
        - { name: Design,                       doc: "Design constraint." }

  RiskLevel:
    type: string
    enum: [Low, Medium, High]
    description: Risk level of a requirement.
    x-rust-enum:
      derive: [Debug, Clone, PartialEq, Serialize, Deserialize, Default]
      serde_rename_all: lowercase
      variants:
        - { name: Low,                         doc: "Low." }
        - { name: Medium, is_default: true,    doc: "Medium (default)." }
        - { name: High,                        doc: "High." }

  VerifyMethod:
    type: string
    enum: [Analysis, Demonstration, Inspection, Test]
    description: Verification method for a requirement.
    x-rust-enum:
      derive: [Debug, Clone, PartialEq, Serialize, Deserialize, Default]
      serde_rename_all: lowercase
      variants:
        - { name: Analysis,                  doc: "Analysis." }
        - { name: Demonstration,             doc: "Demonstration." }
        - { name: Inspection,                doc: "Inspection." }
        - { name: Test, is_default: true,    doc: "Test (default)." }

  Requirement:
    type: object
    required: [text, req_type, risk, verification]
    description: A single requirement entry.
    properties:
      text:
        type: string
      req_type:
        $ref: "#/definitions/RequirementType"
        x-serde-rename: "type"
        x-serde-default: true
      risk:
        $ref: "#/definitions/RiskLevel"
        x-serde-default: true
      verification:
        $ref: "#/definitions/VerifyMethod"
        x-serde-default: true
      priority:
        type: string
      notes:
        type: string
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  Element:
    type: object
    description: An element that verifies or satisfies requirements.
    properties:
      element_type:
        type: string
      doc_ref:
        type: string
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  RelationshipKind:
    type: string
    enum: [Verifies, Refines, Copies, Contains, Derives, Satisfies, Traces]
    description: Kind of relationship between elements and requirements.
    x-rust-enum:
      derive: [Debug, Clone, PartialEq, Serialize, Deserialize]
      serde_rename_all: lowercase

  Relationship:
    type: object
    required: [from, to, kind]
    description: A relationship between a requirement and an element or another requirement.
    properties:
      from:
        type: string
      to:
        type: string
      kind:
        $ref: "#/definitions/RelationshipKind"
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  RequirementContent:
    type: object
    required: [id, requirements, elements, relationships]
    description: Content type for `requirements` section (requirementDiagram).
    properties:
      id:
        type: string
      requirements:
        type: object
        x-rust-type: "HashMap<String, Requirement>"
        x-serde-default: true
      elements:
        type: object
        x-rust-type: "HashMap<String, Element>"
        x-serde-default: true
      relationships:
        type: array
        items:
          $ref: "#/definitions/Relationship"
        x-serde-default: true
      title:
        type: string
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diagrams/content/requirement.rs -->
````rust
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/content/requirement.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete requirement content module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] 7-type scope; 3 enums with is_default migration.
- [schema] Combined rename+default on Requirement.req_type; HashMaps via x-rust-type.
- [changes] codegen + hand-written split correct.
