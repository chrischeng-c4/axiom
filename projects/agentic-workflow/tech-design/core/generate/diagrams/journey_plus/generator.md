---
id: sdd-journey_plus-generator-output
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# JourneyPlusOutput

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diagrams/journey_plus/generator.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `JourneyPlusGenerator` | projects/agentic-workflow/src/generate/diagrams/journey_plus/generator.rs | struct | pub | 18 |  |
| `JourneyPlusOutput` | projects/agentic-workflow/src/generate/diagrams/journey_plus/generator.rs | struct | pub | 11 |  |
| `generate` | projects/agentic-workflow/src/generate/diagrams/journey_plus/generator.rs | function | pub | 26 | generate(         &self,         journey: &JourneyDef,         validation: JourneyValidationResult,     ) -> Result<JourneyPlusOutput, String> |
| `generate_mermaid` | projects/agentic-workflow/src/generate/diagrams/journey_plus/generator.rs | function | pub | 64 | generate_mermaid(&self, journey: &JourneyDef) -> Result<String, String> |
| `new` | projects/agentic-workflow/src/generate/diagrams/journey_plus/generator.rs | function | pub | 22 | new() -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  JourneyPlusOutput:
    type: object
    required: [frontmatter, diagram, validation, combined]
    description: Output of the Mermaid Plus generator.
    properties:
      frontmatter: { type: string }
      diagram: { type: string }
      validation:
        type: object
        x-rust-type: JourneyValidationResult
      combined: { type: string }
    x-rust-struct:
      derive: [Debug, Clone, "serde::Serialize"]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diagrams/journey_plus/generator.rs -->
````rust
//! Journey+ generator

use super::schema::JourneyDef;
use super::validator::JourneyValidationResult;

/// Output of the Mermaid Plus generator.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/journey_plus/generator.md#schema
#[derive(Debug, Clone, serde::Serialize)]
pub struct JourneyPlusOutput {
    pub frontmatter: String,
    pub diagram: String,
    pub validation: JourneyValidationResult,
    pub combined: String,
}
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/journey_plus/generator.md#source
pub struct JourneyPlusGenerator;

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/journey_plus/generator.md#source
impl JourneyPlusGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(
        &self,
        journey: &JourneyDef,
        validation: JourneyValidationResult,
    ) -> Result<JourneyPlusOutput, String> {
        let frontmatter = self.generate_frontmatter(journey)?;
        let mermaid = self.generate_mermaid(journey)?;

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

        Ok(JourneyPlusOutput {
            frontmatter,
            diagram: mermaid,
            validation,
            combined,
        })
    }

    fn generate_frontmatter(&self, journey: &JourneyDef) -> Result<String, String> {
        let yaml = serde_yaml::to_string(journey).map_err(|e| format!("YAML error: {}", e))?;
        Ok(yaml.strip_prefix("---\n").unwrap_or(&yaml).to_string())
    }

    pub fn generate_mermaid(&self, journey: &JourneyDef) -> Result<String, String> {
        let mut mermaid = String::new();
        mermaid.push_str("journey\n");
        mermaid.push_str(&format!("    title {}\n", journey.title));

        for section in &journey.sections {
            mermaid.push_str(&format!("    section {}\n", section.name));
            for task in &section.tasks {
                let actors = task.actors.join(", ");
                mermaid.push_str(&format!(
                    "        {}: {}: {}\n",
                    task.name, task.score, actors
                ));
            }
        }

        Ok(mermaid)
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/journey_plus/generator.md#source
impl Default for JourneyPlusGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::validator::JourneyValidator;
    use super::*;
    use serde_json::json;

    fn parse_journey(json: serde_json::Value) -> JourneyDef {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn test_generate_journey() {
        let journey = parse_journey(json!({
            "id": "test",
            "title": "Shopping Experience",
            "sections": [
                {
                    "name": "Discovery",
                    "tasks": [
                        { "name": "Browse", "score": 4, "actors": ["User"] },
                        { "name": "Search", "score": 3, "actors": ["User", "Search"] }
                    ]
                },
                {
                    "name": "Purchase",
                    "tasks": [
                        { "name": "Checkout", "score": 5, "actors": ["User", "Payment"] }
                    ]
                }
            ]
        }));

        let validation = JourneyValidator::new().validate(&journey);
        let output = JourneyPlusGenerator::new()
            .generate(&journey, validation)
            .unwrap();

        assert!(output.diagram.contains("journey"));
        assert!(output.diagram.contains("title Shopping Experience"));
        assert!(output.diagram.contains("section Discovery"));
        assert!(output.diagram.contains("Browse: 4: User"));
        assert!(output.diagram.contains("Search: 3: User, Search"));
        assert!(output.diagram.contains("section Purchase"));
        assert!(output.diagram.contains("Checkout: 5: User, Payment"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/journey_plus/generator.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete Journey+ generator module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- ok.
