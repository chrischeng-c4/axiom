// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/journey.md#source
// CODEGEN-BEGIN
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

// CODEGEN-END
