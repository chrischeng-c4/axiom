// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/requirement.md#source
// CODEGEN-BEGIN
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

// CODEGEN-END
