// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/envelope.md#source
// CODEGEN-BEGIN
//! Common `Diagram<C>` envelope and `DiagramFrontmatter` trait.
//!
//! All Mermaid Plus diagram blocks share the same outer YAML shape (id, title)
//! while each diagram type has a distinct Content struct (design decision D3).
//!
//! Usage:
//! ```ignore
//! let fm: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
//! let sm: Diagram<StateMachineContent> = Diagram::from_yaml(&fm).unwrap();
//! println!("id={}, states={}", sm.id, sm.content.nodes.len());
//! ```

// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/envelope.md#source

/// Common envelope for all Mermaid Plus diagram types.
///
/// `C` is the diagram-specific Content struct
/// (e.g. `StateMachineContent`, `InteractionContent`, `LogicContent`).
// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/envelope.md#source
use serde::{Deserialize, Serialize};

/// Generic diagram envelope wrapping diagram-specific content.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/envelope.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagram<C> {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    pub content: C,
}
impl<C> Diagram<C>
where
    C: for<'de> Deserialize<'de>,
{
    /// Parse a `Diagram<C>` from a `serde_yaml::Value` frontmatter block.
    ///
    /// The YAML value must have an `id` field plus all fields required by `C`.
    pub fn from_yaml(value: &serde_yaml::Value) -> Result<Self, serde_yaml::Error> {
        // Extract id and title from the top-level value
        let id = value
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let title = value
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Deserialize the full value as C (content types embed id/title too)
        let content: C = serde_yaml::from_value(value.clone())?;

        Ok(Diagram { id, title, content })
    }
}

/// Common interface for all diagram Content types.
///
/// Provides diagram identity and node enumeration without knowledge of
/// the specific diagram structure.
// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/envelope.md#source
pub trait DiagramFrontmatter {
    /// Diagram identifier.
    fn id(&self) -> &str;

    /// Return all node/state/actor/element IDs in this diagram.
    fn node_ids(&self) -> Vec<&str>;
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/envelope.md#source
impl DiagramFrontmatter for super::content::StateMachineContent {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_ids(&self) -> Vec<&str> {
        self.nodes.keys().map(|k| k.as_str()).collect()
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/envelope.md#source
impl DiagramFrontmatter for super::content::InteractionContent {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_ids(&self) -> Vec<&str> {
        self.actors.iter().map(|a| a.id.as_str()).collect()
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/envelope.md#source
impl DiagramFrontmatter for super::content::LogicContent {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_ids(&self) -> Vec<&str> {
        self.nodes.keys().map(|k| k.as_str()).collect()
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/envelope.md#source
impl DiagramFrontmatter for super::content::RequirementContent {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_ids(&self) -> Vec<&str> {
        self.requirements.keys().map(|k| k.as_str()).collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::diagrams::content::state_machine::StateMachineContent;

    // @spec projects/agentic-workflow/tech-design/core/generate/diagrams/envelope.md#source
    #[test]
    fn test_diagram_from_yaml() {
        let yaml = r#"
id: my-sm
initial: idle
nodes:
  idle:
    kind: normal
  done:
    kind: terminal
"#;
        let value: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let diagram: Diagram<StateMachineContent> = Diagram::from_yaml(&value).unwrap();

        assert_eq!(diagram.id, "my-sm");
        assert!(diagram.title.is_none());
        assert_eq!(diagram.content.nodes.len(), 2);
    }

    // @spec projects/agentic-workflow/tech-design/core/generate/diagrams/envelope.md#source
    #[test]
    fn test_diagram_frontmatter_trait() {
        let yaml = r#"
id: test-sm
initial: s1
nodes:
  s1:
    kind: normal
  s2:
    kind: terminal
"#;
        let value: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let diagram: Diagram<StateMachineContent> = Diagram::from_yaml(&value).unwrap();

        // Test DiagramFrontmatter trait
        assert_eq!(diagram.content.id(), "test-sm");
        let node_ids = diagram.content.node_ids();
        assert_eq!(node_ids.len(), 2, "Should have 2 nodes");
        assert!(node_ids.contains(&"s1"), "Should contain s1");
        assert!(node_ids.contains(&"s2"), "Should contain s2");
    }

    #[test]
    fn test_diagram_with_title() {
        let yaml = r#"
id: my-diagram
title: My State Machine
initial: start
nodes:
  start:
    kind: normal
"#;
        let value: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        let diagram: Diagram<StateMachineContent> = Diagram::from_yaml(&value).unwrap();

        assert_eq!(diagram.title.as_deref(), Some("My State Machine"));
    }
}

// CODEGEN-END
