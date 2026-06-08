---
id: sdd-generate-requirement
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Requirement Diagram

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diagrams/requirement.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `DesignElement` | projects/agentic-workflow/src/generate/diagrams/requirement.rs | struct | pub | 77 |  |
| `ReqRelationship` | projects/agentic-workflow/src/generate/diagrams/requirement.rs | struct | pub | 107 |  |
| `ReqRelationshipType` | projects/agentic-workflow/src/generate/diagrams/requirement.rs | enum | pub | 94 |  |
| `RequirementDef` | projects/agentic-workflow/src/generate/diagrams/requirement.rs | struct | pub | 60 |  |
| `RequirementInput` | projects/agentic-workflow/src/generate/diagrams/requirement.rs | struct | pub | 120 |  |
| `RequirementType` | projects/agentic-workflow/src/generate/diagrams/requirement.rs | enum | pub | 15 |  |
| `RiskLevel` | projects/agentic-workflow/src/generate/diagrams/requirement.rs | enum | pub | 34 |  |
| `VerificationMethod` | projects/agentic-workflow/src/generate/diagrams/requirement.rs | enum | pub | 46 |  |
| `generate_requirement_diagram` | projects/agentic-workflow/src/generate/diagrams/requirement.rs | function | pub | 132 | generate_requirement_diagram(input: &RequirementInput) -> Result<String> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  RequirementType:
    type: string
    enum:
      - Requirement
      - FunctionalRequirement
      - InterfaceRequirement
      - PerformanceRequirement
      - PhysicalRequirement
      - DesignConstraint
    description: Requirement node kind.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, Default]
      serde_rename_all: camelCase
      variants:
        - { name: Requirement,            is_default: true, doc: "Generic requirement." }
        - { name: FunctionalRequirement,                    doc: "Functional requirement." }
        - { name: InterfaceRequirement,                     doc: "Interface requirement." }
        - { name: PerformanceRequirement,                   doc: "Performance requirement." }
        - { name: PhysicalRequirement,                      doc: "Physical requirement." }
        - { name: DesignConstraint,                         doc: "Design constraint." }

  RiskLevel:
    type: string
    enum: [Low, Medium, High]
    description: Risk level (bare PascalCase serialization).
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize]

  VerificationMethod:
    type: string
    enum: [Analysis, Inspection, Test, Demonstration]
    description: Verification method (bare PascalCase serialization).
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize]

  RequirementDef:
    type: object
    required: [id, text, req_type, risk, verification]
    description: One requirement node.
    properties:
      id:
        type: string
        description: "Requirement identifier."
      text:
        type: string
        description: "Requirement text."
      req_type:
        $ref: "#/definitions/RequirementType"
        description: "Requirement kind. JSON key 'type'; defaults to Requirement."
        x-serde-rename: "type"
        x-serde-default: true
      risk:
        $ref: "#/definitions/RiskLevel"
        description: "Associated risk level (required)."
      verification:
        $ref: "#/definitions/VerificationMethod"
        description: "Verification method (required)."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  DesignElement:
    type: object
    required: [id, text, elem_type]
    description: One design-element node.
    properties:
      id:
        type: string
        description: "Element identifier."
      text:
        type: string
        description: "Element text."
      elem_type:
        type: string
        x-serde-rename: "type"
        description: "Element kind tag. JSON key 'type' (Rust reserved word)."
      docref:
        type: string
        description: "Optional documentation reference."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ReqRelationshipType:
    type: string
    enum: [Satisfies, Verifies, Refines, Traces, Contains, Copies, Derives]
    description: Relationship kind between two requirements / elements.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize]
      serde_rename_all: lowercase

  ReqRelationship:
    type: object
    required: [from, to, rel_type]
    description: One edge between two requirement / element nodes.
    properties:
      from:
        type: string
        description: "Source node id."
      to:
        type: string
        description: "Target node id."
      rel_type:
        $ref: "#/definitions/ReqRelationshipType"
        description: "Edge kind. JSON key 'type' (Rust reserved word)."
        x-serde-rename: "type"
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  RequirementInput:
    type: object
    required: [requirements, elements, relationships]
    description: Input for requirement-diagram generation.
    properties:
      requirements:
        type: array
        items:
          $ref: "#/definitions/RequirementDef"
        description: "All requirement nodes (need at least one at runtime)."
      elements:
        type: array
        items:
          $ref: "#/definitions/DesignElement"
        description: "Design elements."
        x-serde-default: true
      relationships:
        type: array
        items:
          $ref: "#/definitions/ReqRelationship"
        description: "Edges between requirements / elements."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diagrams/requirement.rs -->
```rust
//! Requirement Diagram Generation
//!
//! Generates Mermaid requirement diagrams for requirement traceability.

use crate::generate::{GenerateError, Result};

use serde::{Deserialize, Serialize};

/// Requirement node kind.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum RequirementType {
    /// Generic requirement.
    #[default]
    Requirement,
    /// Functional requirement.
    FunctionalRequirement,
    /// Interface requirement.
    InterfaceRequirement,
    /// Performance requirement.
    PerformanceRequirement,
    /// Physical requirement.
    PhysicalRequirement,
    /// Design constraint.
    DesignConstraint,
}

/// Risk level (bare PascalCase serialization).
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    #[serde(rename = "Low")]
    Low,
    #[serde(rename = "Medium")]
    Medium,
    #[serde(rename = "High")]
    High,
}

/// Verification method (bare PascalCase serialization).
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationMethod {
    #[serde(rename = "Analysis")]
    Analysis,
    #[serde(rename = "Inspection")]
    Inspection,
    #[serde(rename = "Test")]
    Test,
    #[serde(rename = "Demonstration")]
    Demonstration,
}

/// One requirement node.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementDef {
    /// Requirement identifier.
    pub id: String,
    /// Requirement text.
    pub text: String,
    /// Requirement kind. JSON key 'type'; defaults to Requirement.
    #[serde(rename = "type", default)]
    pub req_type: RequirementType,
    /// Associated risk level (required).
    pub risk: RiskLevel,
    /// Verification method (required).
    pub verification: VerificationMethod,
}

/// One design-element node.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignElement {
    /// Element identifier.
    pub id: String,
    /// Element text.
    pub text: String,
    /// Element kind tag. JSON key 'type' (Rust reserved word).
    #[serde(rename = "type")]
    pub elem_type: String,
    /// Optional documentation reference.
    #[serde(default)]
    pub docref: Option<String>,
}

/// Relationship kind between two requirements / elements.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReqRelationshipType {
    Satisfies,
    Verifies,
    Refines,
    Traces,
    Contains,
    Copies,
    Derives,
}

/// One edge between two requirement / element nodes.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReqRelationship {
    /// Source node id.
    pub from: String,
    /// Target node id.
    pub to: String,
    /// Edge kind. JSON key 'type' (Rust reserved word).
    #[serde(rename = "type")]
    pub rel_type: ReqRelationshipType,
}

/// Input for requirement-diagram generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementInput {
    /// All requirement nodes (need at least one at runtime).
    pub requirements: Vec<RequirementDef>,
    /// Design elements.
    #[serde(default)]
    pub elements: Vec<DesignElement>,
    /// Edges between requirements / elements.
    #[serde(default)]
    pub relationships: Vec<ReqRelationship>,
}
/// Generate a Mermaid requirement diagram
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement.md#source
pub fn generate_requirement_diagram(input: &RequirementInput) -> Result<String> {
    if input.requirements.is_empty() {
        return Err(GenerateError::InvalidValue(
            "At least one requirement required".to_string(),
        ));
    }

    let mut mermaid = String::new();
    mermaid.push_str("requirementDiagram\n");

    // Generate requirements
    for req in &input.requirements {
        let type_str = match req.req_type {
            RequirementType::Requirement => "requirement",
            RequirementType::FunctionalRequirement => "functionalRequirement",
            RequirementType::InterfaceRequirement => "interfaceRequirement",
            RequirementType::PerformanceRequirement => "performanceRequirement",
            RequirementType::PhysicalRequirement => "physicalRequirement",
            RequirementType::DesignConstraint => "designConstraint",
        };
        let risk_str = match req.risk {
            RiskLevel::Low => "Low",
            RiskLevel::Medium => "Medium",
            RiskLevel::High => "High",
        };
        let verif_str = match req.verification {
            VerificationMethod::Analysis => "Analysis",
            VerificationMethod::Inspection => "Inspection",
            VerificationMethod::Test => "Test",
            VerificationMethod::Demonstration => "Demonstration",
        };

        mermaid.push_str(&format!("    {} {} {{\n", type_str, req.id));
        mermaid.push_str(&format!("        id: \"{}\"\n", req.id));
        mermaid.push_str(&format!("        text: \"{}\"\n", req.text));
        mermaid.push_str(&format!("        risk: {}\n", risk_str));
        mermaid.push_str(&format!("        verifymethod: {}\n", verif_str));
        mermaid.push_str("    }\n");
    }

    // Generate elements
    for elem in &input.elements {
        mermaid.push_str(&format!("    element {} {{\n", elem.id));
        mermaid.push_str(&format!("        type: \"{}\"\n", elem.elem_type));
        if let Some(ref docref) = elem.docref {
            mermaid.push_str(&format!("        docref: \"{}\"\n", docref));
        }
        mermaid.push_str("    }\n");
    }

    // Generate relationships
    for rel in &input.relationships {
        let rel_str = match rel.rel_type {
            ReqRelationshipType::Satisfies => "satisfies",
            ReqRelationshipType::Verifies => "verifies",
            ReqRelationshipType::Refines => "refines",
            ReqRelationshipType::Traces => "traces",
            ReqRelationshipType::Contains => "contains",
            ReqRelationshipType::Copies => "copies",
            ReqRelationshipType::Derives => "derives",
        };
        mermaid.push_str(&format!("    {} - {} -> {}\n", rel.from, rel_str, rel.to));
    }

    Ok(mermaid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_requirement() {
        let input = RequirementInput {
            requirements: vec![RequirementDef {
                id: "R1".to_string(),
                text: "System shall respond within 100ms".to_string(),
                req_type: RequirementType::PerformanceRequirement,
                risk: RiskLevel::Medium,
                verification: VerificationMethod::Test,
            }],
            elements: vec![],
            relationships: vec![],
        };

        let result = generate_requirement_diagram(&input).unwrap();
        assert!(result.contains("requirementDiagram"));
        assert!(result.contains("performanceRequirement R1"));
        assert!(result.contains("risk: Medium"));
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/requirement.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete requirement diagram generator module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** needs-revision

- [schema] `RequirementInput.required` lists `[requirements, elements, relationships]` but `elements` and `relationships` carry `x-serde-default: true` and are optional vecs in the source (`#[serde(default)]`). Including them in `required:` contradicts both their `x-serde-default` annotation and the overview's description of "optional elements/relationships vec with default". Fix: change `required: [requirements, elements, relationships]` to `required: [requirements]` for `RequirementInput`.
- [schema] `DesignElement.docref` is `type: string` with `x-serde-default: true`, but the source field is `pub docref: Option<String>`. If the codegen requires an explicit signal to emit `Option<T>` (e.g., `x-rust-option: true`), add it; if `x-serde-default` on a non-required string field already implies `Option<String>` by convention, document that assumption in a comment or confirm consistency with sibling specs.

## Review 2
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Round 1 findings were wrong. The reviser correctly pushed back and added a clarifying paragraph documenting the codegen convention: fields listed in `required:` with `x-serde-default: true` emit bare `Vec<T>` + `#[serde(default)]` (not `Option<Vec<T>>`), while fields omitted from `required:` are auto-wrapped as `Option<T>`. This is consistent with the already-merged sibling specs and the rust-schema generator in `projects/agentic-workflow/src/generate/gen/rust/schema.rs`. The schema and changes sections are correct and unchanged.
