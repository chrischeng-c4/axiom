---
id: sdd-generate-journey
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Journey Diagram

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diagrams/journey.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `JourneyInput` | projects/agentic-workflow/src/generate/diagrams/journey.rs | struct | pub | 36 |  |
| `JourneySection` | projects/agentic-workflow/src/generate/diagrams/journey.rs | struct | pub | 26 |  |
| `JourneyTask` | projects/agentic-workflow/src/generate/diagrams/journey.rs | struct | pub | 14 |  |
| `generate_journey` | projects/agentic-workflow/src/generate/diagrams/journey.rs | function | pub | 44 | generate_journey(input: &JourneyInput) -> Result<String> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  JourneyTask:
    type: object
    required: [name, score, actors]
    description: Journey task.
    properties:
      name:
        type: string
        description: "Task name."
      score:
        type: integer
        x-rust-type: i32
        description: "Satisfaction score (1-5)."
      actors:
        type: array
        items: { type: string }
        description: "Actors involved in the task."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  JourneySection:
    type: object
    required: [name, tasks]
    description: Journey section.
    properties:
      name:
        type: string
        description: "Section name."
      tasks:
        type: array
        items:
          $ref: "#/definitions/JourneyTask"
        description: "Tasks in this section."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  JourneyInput:
    type: object
    required: [title, sections]
    description: Input for journey diagram generation.
    properties:
      title:
        type: string
        description: "Diagram title."
      sections:
        type: array
        items:
          $ref: "#/definitions/JourneySection"
        description: "Diagram sections."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diagrams/journey.rs -->
```rust
//! User Journey Diagram Generation
//!
//! Generates Mermaid user journey diagrams for UX flows and service blueprints.

use crate::generate::{GenerateError, Result};

use serde::{Deserialize, Serialize};

/// Journey task.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/journey.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyTask {
    /// Task name.
    pub name: String,
    /// Satisfaction score (1-5).
    pub score: i32,
    /// Actors involved in the task.
    pub actors: Vec<String>,
}

/// Journey section.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/journey.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneySection {
    /// Section name.
    pub name: String,
    /// Tasks in this section.
    pub tasks: Vec<JourneyTask>,
}

/// Input for journey diagram generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/journey.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyInput {
    /// Diagram title.
    pub title: String,
    /// Diagram sections.
    pub sections: Vec<JourneySection>,
}
/// Generate a Mermaid user journey diagram
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/journey.md#source
pub fn generate_journey(input: &JourneyInput) -> Result<String> {
    if input.sections.is_empty() {
        return Err(GenerateError::InvalidValue(
            "At least one section required".to_string(),
        ));
    }

    let mut mermaid = String::new();
    mermaid.push_str("journey\n");
    mermaid.push_str(&format!("    title {}\n", input.title));

    for section in &input.sections {
        mermaid.push_str(&format!("    section {}\n", section.name));
        for task in &section.tasks {
            // Validate score
            if task.score < 1 || task.score > 5 {
                return Err(GenerateError::InvalidValue(format!(
                    "Score must be 1-5, got {} for task '{}'",
                    task.score, task.name
                )));
            }
            if task.actors.is_empty() {
                return Err(GenerateError::MissingField(format!(
                    "Task '{}' requires at least one actor",
                    task.name
                )));
            }
            let actors = task.actors.join(", ");
            mermaid.push_str(&format!(
                "        {}: {}: {}\n",
                task.name, task.score, actors
            ));
        }
    }

    Ok(mermaid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_journey() {
        let input = JourneyInput {
            title: "User Shopping Experience".to_string(),
            sections: vec![
                JourneySection {
                    name: "Discovery".to_string(),
                    tasks: vec![
                        JourneyTask {
                            name: "Browse products".to_string(),
                            score: 4,
                            actors: vec!["User".to_string()],
                        },
                        JourneyTask {
                            name: "Search".to_string(),
                            score: 3,
                            actors: vec!["User".to_string(), "Search Engine".to_string()],
                        },
                    ],
                },
                JourneySection {
                    name: "Purchase".to_string(),
                    tasks: vec![
                        JourneyTask {
                            name: "Add to cart".to_string(),
                            score: 5,
                            actors: vec!["User".to_string()],
                        },
                        JourneyTask {
                            name: "Checkout".to_string(),
                            score: 4,
                            actors: vec!["User".to_string(), "Payment".to_string()],
                        },
                    ],
                },
            ],
        };

        let result = generate_journey(&input).unwrap();
        assert!(result.contains("journey"));
        assert!(result.contains("title User Shopping Experience"));
        assert!(result.contains("section Discovery"));
        assert!(result.contains("Browse products: 4: User"));
    }

    #[test]
    fn test_invalid_score() {
        let input = JourneyInput {
            title: "Test".to_string(),
            sections: vec![JourneySection {
                name: "Section".to_string(),
                tasks: vec![JourneyTask {
                    name: "Task".to_string(),
                    score: 6,
                    actors: vec!["Actor".to_string()],
                }],
            }],
        };

        let result = generate_journey(&input);
        assert!(result.is_err());
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/journey.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete journey diagram module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Clear identification of the three structs and the hand-written boundary; no ambiguity.
- [schema] All three definitions are complete with required fields, correct types, $ref cross-references, and x-rust-struct derives matching R2.
- [changes] Two-entry split (codegen + hand-written) correctly covers R4 and R5; `replaces:` lists all three struct names explicitly.
