// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-prompt-template-rs.md#source
// CODEGEN-BEGIN
//! Prompt template definitions and context

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Prompt template with variables and sections
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-prompt-template-rs.md#source
pub struct PromptTemplate {
    /// Template name/ID
    pub name: String,

    /// Template version
    pub version: String,

    /// Description
    pub description: String,

    /// System role definition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_role: Option<String>,

    /// Template sections (ordered)
    pub sections: Vec<PromptSection>,

    /// Few-shot examples (optional)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<FewShotExample>,

    /// Metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// A section in the prompt template
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-prompt-template-rs.md#source
pub struct PromptSection {
    /// Section title (e.g., "Input", "Expected Output")
    pub title: String,

    /// Section content template with {{variables}}
    pub content: String,

    /// Whether this section is optional
    #[serde(default)]
    pub optional: bool,

    /// Condition for including this section
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
}

/// Few-shot example for demonstration
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-prompt-template-rs.md#source
pub struct FewShotExample {
    /// Example input
    pub input: String,

    /// Example output
    pub output: String,

    /// Optional explanation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
}

/// Context for rendering a template
#[derive(Debug, Clone, Default)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-prompt-template-rs.md#source
pub struct PromptContext {
    variables: HashMap<String, String>,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-prompt-template-rs.md#source
impl PromptContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// Set a variable
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// Get a variable
    pub fn get(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    /// Check if a condition is met
    pub fn check_condition(&self, condition: &str) -> bool {
        // Simple condition checking: variable existence
        // Format: "has_expected" or "!has_expected"
        if let Some(var_name) = condition.strip_prefix('!') {
            !self.variables.contains_key(var_name)
        } else {
            self.variables.contains_key(condition)
        }
    }
}

/// Variable type hint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-prompt-template-rs.md#source
pub enum PromptVariable {
    /// String variable
    String,
    /// Number variable
    Number,
    /// Boolean variable
    Boolean,
    /// Optional variable
    Optional,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-prompt-template-rs.md#source
impl PromptTemplate {
    /// Create a basic template
    pub fn basic(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "1.0.0".to_string(),
            description: "Basic prompt template".to_string(),
            system_role: None,
            sections: Vec::new(),
            examples: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set system role
    pub fn with_system_role(mut self, role: impl Into<String>) -> Self {
        self.system_role = Some(role.into());
        self
    }

    /// Add a section
    pub fn with_section(mut self, section: PromptSection) -> Self {
        self.sections.push(section);
        self
    }

    /// Add a few-shot example
    pub fn with_example(mut self, example: FewShotExample) -> Self {
        self.examples.push(example);
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-prompt-template-rs.md#source
impl PromptSection {
    /// Create a new section
    pub fn new(title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            optional: false,
            condition: None,
        }
    }

    /// Mark as optional
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Set condition
    pub fn with_condition(mut self, condition: impl Into<String>) -> Self {
        self.condition = Some(condition.into());
        self
    }
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-prompt-template-rs.md#source
impl FewShotExample {
    /// Create a new example
    pub fn new(input: impl Into<String>, output: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            output: output.into(),
            explanation: None,
        }
    }

    /// Add explanation
    pub fn with_explanation(mut self, explanation: impl Into<String>) -> Self {
        self.explanation = Some(explanation.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_creation() {
        let template = PromptTemplate::basic("test")
            .with_system_role("You are a helpful assistant")
            .with_section(PromptSection::new("Input", "{{input}}"))
            .with_section(PromptSection::new("Output", "{{output}}"));

        assert_eq!(template.name, "test");
        assert_eq!(template.sections.len(), 2);
        assert!(template.system_role.is_some());
    }

    #[test]
    fn test_context() {
        let mut context = PromptContext::new();
        context.set("name", "Alice");
        context.set("age", "30");

        assert_eq!(context.get("name"), Some(&"Alice".to_string()));
        assert_eq!(context.get("age"), Some(&"30".to_string()));
    }

    #[test]
    fn test_condition_checking() {
        let mut context = PromptContext::new();
        context.set("has_expected", "true");

        assert!(context.check_condition("has_expected"));
        assert!(!context.check_condition("!has_expected"));
        assert!(!context.check_condition("missing"));
    }
}
// CODEGEN-END
