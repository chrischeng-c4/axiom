//! Journey+ definition schema

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/journey_plus/schema.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// Journey diagram definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/journey_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyDef {
    /// Diagram identifier.
    pub id: String,
    /// Journey title.
    pub title: String,
    /// Sections in this journey.
    pub sections: Vec<SectionDef>,
    /// Diagram description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Section definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/journey_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionDef {
    /// Section name.
    pub name: String,
    /// Tasks in this section.
    pub tasks: Vec<TaskDef>,
    /// Section description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Task definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/journey_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDef {
    /// Task name.
    pub name: String,
    /// Satisfaction score (1-5).
    pub score: i32,
    /// Actors involved.
    pub actors: Vec<String>,
    /// Task description.
    #[serde(default)]
    pub description: Option<String>,
}
// CODEGEN-END
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_journey() {
        let json = json!({
            "id": "shopping-journey",
            "title": "User Shopping Experience",
            "sections": [
                {
                    "name": "Discovery",
                    "tasks": [
                        { "name": "Browse products", "score": 4, "actors": ["User"] },
                        { "name": "Search", "score": 3, "actors": ["User", "Search Engine"] }
                    ]
                },
                {
                    "name": "Purchase",
                    "tasks": [
                        { "name": "Add to cart", "score": 5, "actors": ["User"] },
                        { "name": "Checkout", "score": 4, "actors": ["User", "Payment"] }
                    ]
                }
            ]
        });

        let journey: JourneyDef = serde_json::from_value(json).unwrap();
        assert_eq!(journey.title, "User Shopping Experience");
        assert_eq!(journey.sections.len(), 2);
        assert_eq!(journey.sections[0].tasks.len(), 2);
    }
}
