//! Requirement+ definition schema

use std::collections::HashMap;

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/requirement_plus/schema.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// Design element definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementDef {
    pub text: String,
    #[serde(rename = "type")]
    pub elem_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docref: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub given: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub then: Option<String>,
}

/// Layout direction for requirement diagram.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReqDirection {
    #[serde(rename = "TB")]
    TB,
    #[serde(rename = "BT")]
    BT,
    #[serde(rename = "LR")]
    LR,
    #[serde(rename = "RL")]
    RL,
}

/// Relationship definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReqRelationshipDef {
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub rel_type: ReqRelationshipTypePlus,
}

/// Relationship type.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ReqRelationshipTypePlus {
    Satisfies,
    Verifies,
    Refines,
    Traces,
    Contains,
    Copies,
    Derives,
}

/// Requirement definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementDefPlus {
    pub text: String,
    #[serde(rename = "type", default)]
    pub req_type: RequirementTypePlus,
    pub risk: RiskLevelPlus,
    pub verification: VerificationMethodPlus,
    #[serde(default)]
    pub description: Option<String>,
}

/// Requirement diagram definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementDiagramDef {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub direction: Option<ReqDirection>,
    pub requirements: HashMap<String, RequirementDefPlus>,
    #[serde(default)]
    pub elements: HashMap<String, ElementDef>,
    #[serde(default)]
    pub relationships: Vec<ReqRelationshipDef>,
    #[serde(default)]
    pub description: Option<String>,
}

/// Requirement type.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum RequirementTypePlus {
    /// Generic.
    #[default]
    Requirement,
    /// Functional.
    FunctionalRequirement,
    /// Interface.
    InterfaceRequirement,
    /// Performance.
    PerformanceRequirement,
    /// Physical.
    PhysicalRequirement,
    /// Design constraint.
    DesignConstraint,
}

/// Risk level.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevelPlus {
    #[serde(rename = "Low")]
    Low,
    #[serde(rename = "Medium")]
    Medium,
    #[serde(rename = "High")]
    High,
}

/// Verification method.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerificationMethodPlus {
    #[serde(rename = "Analysis")]
    Analysis,
    #[serde(rename = "Inspection")]
    Inspection,
    #[serde(rename = "Test")]
    Test,
    #[serde(rename = "Demonstration")]
    Demonstration,
}
// CODEGEN-END
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_requirement_diagram() {
        let json = json!({
            "id": "auth-requirements",
            "requirements": {
                "REQ-001": {
                    "text": "System shall authenticate users",
                    "type": "functionalRequirement",
                    "risk": "High",
                    "verification": "Test"
                },
                "REQ-002": {
                    "text": "Response time < 100ms",
                    "type": "performanceRequirement",
                    "risk": "Medium",
                    "verification": "Test"
                }
            },
            "elements": {
                "AUTH-MOD": {
                    "text": "Authentication Module",
                    "type": "module",
                    "docref": "docs/auth.md"
                }
            },
            "relationships": [
                { "from": "AUTH-MOD", "to": "REQ-001", "type": "satisfies" }
            ]
        });

        let diagram: RequirementDiagramDef = serde_json::from_value(json).unwrap();
        assert_eq!(diagram.requirements.len(), 2);
        assert_eq!(diagram.elements.len(), 1);
        assert_eq!(diagram.relationships.len(), 1);
    }

    #[test]
    fn test_parse_element_without_docref() {
        let json = json!({
            "id": "test",
            "requirements": {
                "R1": { "text": "Must pass", "risk": "Low", "verification": "Test" }
            },
            "elements": {
                "TC-1": {
                    "text": "Login test",
                    "type": "test",
                    "test_type": "integration",
                    "given": "user has credentials",
                    "when": "user submits login form",
                    "then": "session is created"
                }
            },
            "relationships": [
                { "from": "TC-1", "to": "R1", "type": "verifies" }
            ]
        });

        let diagram: RequirementDiagramDef = serde_json::from_value(json).unwrap();
        let elem = &diagram.elements["TC-1"];
        assert!(elem.docref.is_none());
        assert_eq!(elem.test_type.as_deref(), Some("integration"));
        assert_eq!(elem.given.as_deref(), Some("user has credentials"));
        assert_eq!(elem.when.as_deref(), Some("user submits login form"));
        assert_eq!(elem.then.as_deref(), Some("session is created"));
    }
}
