// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/requirement_plus/generator.md#source
// CODEGEN-BEGIN
//! Requirement+ generator

use super::schema::{
    ReqDirection, ReqRelationshipTypePlus, RequirementDiagramDef, RequirementTypePlus,
    RiskLevelPlus, VerificationMethodPlus,
};
use super::validator::RequirementValidationResult;

/// Output of the Mermaid Plus generator.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement_plus/generator.md#schema
#[derive(Debug, Clone, serde::Serialize)]
pub struct RequirementPlusOutput {
    pub frontmatter: String,
    pub diagram: String,
    pub validation: RequirementValidationResult,
    pub combined: String,
}
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement_plus/generator.md#source
pub struct RequirementPlusGenerator;

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement_plus/generator.md#source
impl RequirementPlusGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(
        &self,
        diagram: &RequirementDiagramDef,
        validation: RequirementValidationResult,
    ) -> Result<RequirementPlusOutput, String> {
        let frontmatter = self.generate_frontmatter(diagram)?;
        let mermaid = self.generate_mermaid(diagram)?;

        // Combine into Mermaid+ format (frontmatter inside code block per Mermaid spec)
        let mut combined = String::new();
        combined.push_str("```mermaid\n");
        combined.push_str("---\n");
        combined.push_str(&frontmatter);
        combined.push_str("---\n");
        combined.push_str(&mermaid);
        combined.push_str("```\n");

        if !validation.warnings.is_empty() {
            combined.push_str("\n<!-- Validation Warnings:\n");
            for w in &validation.warnings {
                combined.push_str(&format!("  - {}: {} (at {})\n", w.code, w.message, w.path));
            }
            combined.push_str("-->\n");
        }

        Ok(RequirementPlusOutput {
            frontmatter,
            diagram: mermaid,
            validation,
            combined,
        })
    }

    fn generate_frontmatter(&self, diagram: &RequirementDiagramDef) -> Result<String, String> {
        let yaml = serde_yaml::to_string(diagram).map_err(|e| format!("YAML error: {}", e))?;
        Ok(yaml.strip_prefix("---\n").unwrap_or(&yaml).to_string())
    }

    pub fn generate_mermaid(&self, diagram: &RequirementDiagramDef) -> Result<String, String> {
        let mut mermaid = String::new();
        mermaid.push_str("requirementDiagram\n");

        // Direction
        if let Some(dir) = &diagram.direction {
            let dir_str = match dir {
                ReqDirection::TB => "TB",
                ReqDirection::BT => "BT",
                ReqDirection::LR => "LR",
                ReqDirection::RL => "RL",
            };
            mermaid.push_str(&format!("    accDirection {}\n", dir_str));
        }

        // Generate requirements (sorted)
        let mut requirements: Vec<_> = diagram.requirements.iter().collect();
        requirements.sort_by(|a, b| a.0.cmp(b.0));

        for (req_id, req) in requirements {
            let type_str = match req.req_type {
                RequirementTypePlus::Requirement => "requirement",
                RequirementTypePlus::FunctionalRequirement => "functionalRequirement",
                RequirementTypePlus::InterfaceRequirement => "interfaceRequirement",
                RequirementTypePlus::PerformanceRequirement => "performanceRequirement",
                RequirementTypePlus::PhysicalRequirement => "physicalRequirement",
                RequirementTypePlus::DesignConstraint => "designConstraint",
            };
            let risk_str = match req.risk {
                RiskLevelPlus::Low => "Low",
                RiskLevelPlus::Medium => "Medium",
                RiskLevelPlus::High => "High",
            };
            let verif_str = match req.verification {
                VerificationMethodPlus::Analysis => "Analysis",
                VerificationMethodPlus::Inspection => "Inspection",
                VerificationMethodPlus::Test => "Test",
                VerificationMethodPlus::Demonstration => "Demonstration",
            };

            mermaid.push_str(&format!("    {} {} {{\n", type_str, req_id));
            mermaid.push_str(&format!("        id: \"{}\"\n", req_id));
            mermaid.push_str(&format!("        text: \"{}\"\n", req.text));
            mermaid.push_str(&format!("        risk: {}\n", risk_str));
            mermaid.push_str(&format!("        verifymethod: {}\n", verif_str));
            mermaid.push_str("    }\n");
        }

        // Generate elements (sorted)
        let mut elements: Vec<_> = diagram.elements.iter().collect();
        elements.sort_by(|a, b| a.0.cmp(b.0));

        for (elem_id, elem) in elements {
            mermaid.push_str(&format!("    element {} {{\n", elem_id));
            mermaid.push_str(&format!("        type: \"{}\"\n", elem.elem_type));
            if let Some(ref docref) = elem.docref {
                mermaid.push_str(&format!("        docref: \"{}\"\n", docref));
            }
            mermaid.push_str("    }\n");
        }

        // Generate relationships
        for rel in &diagram.relationships {
            let rel_str = match rel.rel_type {
                ReqRelationshipTypePlus::Satisfies => "satisfies",
                ReqRelationshipTypePlus::Verifies => "verifies",
                ReqRelationshipTypePlus::Refines => "refines",
                ReqRelationshipTypePlus::Traces => "traces",
                ReqRelationshipTypePlus::Contains => "contains",
                ReqRelationshipTypePlus::Copies => "copies",
                ReqRelationshipTypePlus::Derives => "derives",
            };
            mermaid.push_str(&format!("    {} - {} -> {}\n", rel.from, rel_str, rel.to));
        }

        Ok(mermaid)
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement_plus/generator.md#source
impl Default for RequirementPlusGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::validator::RequirementValidator;
    use super::*;
    use serde_json::json;

    fn parse_diagram(json: serde_json::Value) -> RequirementDiagramDef {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn test_generate_requirement_diagram() {
        let diagram = parse_diagram(json!({
            "id": "test",
            "requirements": {
                "R1": {
                    "text": "System shall be fast",
                    "type": "performanceRequirement",
                    "risk": "Medium",
                    "verification": "Test"
                }
            },
            "elements": {
                "E1": { "text": "Performance Module", "type": "module", "docref": "doc.md", "test_type": "unit", "given": "system is idle", "when": "load test runs", "then": "response time < 100ms" }
            },
            "relationships": [
                { "from": "E1", "to": "R1", "type": "satisfies" }
            ]
        }));

        let validation = RequirementValidator::new().validate(&diagram);
        let output = RequirementPlusGenerator::new()
            .generate(&diagram, validation)
            .unwrap();

        assert!(output.diagram.contains("requirementDiagram"));
        assert!(output.diagram.contains("performanceRequirement R1"));
        assert!(output.diagram.contains("E1 - satisfies -> R1"));
    }
}

// CODEGEN-END
