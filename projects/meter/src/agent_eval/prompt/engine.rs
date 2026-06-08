// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-prompt-engine-rs.md#source
// CODEGEN-BEGIN
//! Prompt template rendering engine

use super::template::{PromptContext, PromptTemplate};
use regex::Regex;

/// Prompt rendering engine
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-prompt-engine-rs.md#source
pub struct PromptEngine;

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-prompt-engine-rs.md#source
impl PromptEngine {
    /// Render a template with context
    pub fn render(template: &PromptTemplate, context: &PromptContext) -> Result<String, String> {
        let mut output = String::new();

        // Add system role if present
        if let Some(role) = &template.system_role {
            output.push_str(role);
            output.push_str("\n\n");
        }

        // Add few-shot examples if present
        if !template.examples.is_empty() {
            output.push_str("# Examples\n\n");
            for (i, example) in template.examples.iter().enumerate() {
                output.push_str(&format!("## Example {}\n", i + 1));
                output.push_str(&format!("Input: {}\n", example.input));
                output.push_str(&format!("Output: {}\n", example.output));
                if let Some(explanation) = &example.explanation {
                    output.push_str(&format!("Explanation: {}\n", explanation));
                }
                output.push('\n');
            }
        }

        // Render sections
        for section in &template.sections {
            // Check condition if present
            if let Some(condition) = &section.condition {
                if !context.check_condition(condition) {
                    continue; // Skip this section
                }
            }

            // Render section title
            output.push_str(&format!("# {}\n", section.title));

            // Render section content with variable substitution
            let rendered_content = Self::substitute_variables(&section.content, context)?;

            // Skip if optional and content is empty
            if section.optional && rendered_content.trim().is_empty() {
                output.pop(); // Remove the title line
                output.pop(); // Remove newline
                continue;
            }

            output.push_str(&rendered_content);
            output.push_str("\n\n");
        }

        Ok(output.trim().to_string())
    }

    /// Substitute {{variables}} in content
    fn substitute_variables(content: &str, context: &PromptContext) -> Result<String, String> {
        let re = Regex::new(r"\{\{(\w+)\}\}").unwrap();
        let mut result = content.to_string();

        for cap in re.captures_iter(content) {
            let var_name = &cap[1];
            let value = context
                .get(var_name)
                .ok_or_else(|| format!("Variable '{}' not found in context", var_name))?;

            result = result.replace(&format!("{{{{{}}}}}", var_name), value);
        }

        Ok(result)
    }

    /// Render with inline context builder
    pub fn render_with<F>(template: &PromptTemplate, builder: F) -> Result<String, String>
    where
        F: FnOnce(&mut PromptContext),
    {
        let mut context = PromptContext::new();
        builder(&mut context);
        Self::render(template, &context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_eval::prompt::template::{FewShotExample, PromptSection};

    #[test]
    fn test_basic_rendering() {
        let template = PromptTemplate::basic("test")
            .with_section(PromptSection::new("Input", "{{input}}"))
            .with_section(PromptSection::new("Output", "{{output}}"));

        let mut context = PromptContext::new();
        context.set("input", "Hello");
        context.set("output", "World");

        let result = PromptEngine::render(&template, &context).unwrap();

        assert!(result.contains("# Input"));
        assert!(result.contains("Hello"));
        assert!(result.contains("# Output"));
        assert!(result.contains("World"));
    }

    #[test]
    fn test_system_role() {
        let template = PromptTemplate::basic("test")
            .with_system_role("You are a helpful assistant")
            .with_section(PromptSection::new("Task", "{{task}}"));

        let mut context = PromptContext::new();
        context.set("task", "Do something");

        let result = PromptEngine::render(&template, &context).unwrap();

        assert!(result.starts_with("You are a helpful assistant"));
        assert!(result.contains("Do something"));
    }

    #[test]
    fn test_few_shot_examples() {
        let template = PromptTemplate::basic("test")
            .with_example(FewShotExample::new("2+2", "4"))
            .with_example(FewShotExample::new("3+3", "6").with_explanation("Basic addition"))
            .with_section(PromptSection::new("Question", "{{question}}"));

        let mut context = PromptContext::new();
        context.set("question", "What is 5+5?");

        let result = PromptEngine::render(&template, &context).unwrap();

        assert!(result.contains("# Examples"));
        assert!(result.contains("Example 1"));
        assert!(result.contains("2+2"));
        assert!(result.contains("Basic addition"));
    }

    #[test]
    fn test_conditional_sections() {
        let template = PromptTemplate::basic("test")
            .with_section(PromptSection::new("Input", "{{input}}"))
            .with_section(
                PromptSection::new("Expected", "{{expected}}").with_condition("has_expected"),
            );

        // Without expected
        let mut context1 = PromptContext::new();
        context1.set("input", "Test input");

        let result1 = PromptEngine::render(&template, &context1).unwrap();
        assert!(result1.contains("# Input"));
        assert!(!result1.contains("# Expected"));

        // With expected
        let mut context2 = PromptContext::new();
        context2.set("input", "Test input");
        context2.set("has_expected", "true");
        context2.set("expected", "Expected output");

        let result2 = PromptEngine::render(&template, &context2).unwrap();
        assert!(result2.contains("# Input"));
        assert!(result2.contains("# Expected"));
        assert!(result2.contains("Expected output"));
    }

    #[test]
    fn test_missing_variable() {
        let template = PromptTemplate::basic("test")
            .with_section(PromptSection::new("Input", "{{missing_var}}"));

        let context = PromptContext::new();

        let result = PromptEngine::render(&template, &context);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("missing_var"));
    }

    #[test]
    fn test_render_with() {
        let template = PromptTemplate::basic("test")
            .with_section(PromptSection::new("Greeting", "Hello, {{name}}!"));

        let result = PromptEngine::render_with(&template, |ctx| {
            ctx.set("name", "Alice");
        })
        .unwrap();

        assert!(result.contains("Hello, Alice!"));
    }
}
// CODEGEN-END
