---
id: projects-meter-src-agent-eval-llm-judge-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: json-default-report-envelope-and-findings
    claim: json-default-report-envelope-and-findings
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/src/agent_eval/llm_judge.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/agent_eval/llm_judge.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `LLMJudge` | projects/meter/src/agent_eval/llm_judge.rs | struct | pub | 118 |  |
| `LLMJudgeConfig` | projects/meter/src/agent_eval/llm_judge.rs | struct | pub | 15 |  |
| `LLMJudgeResponse` | projects/meter/src/agent_eval/llm_judge.rs | struct | pub | 107 |  |
| `config` | projects/meter/src/agent_eval/llm_judge.rs | function | pub | 402 | config(&self) -> &LLMJudgeConfig |
| `default_config` | projects/meter/src/agent_eval/llm_judge.rs | function | pub | 149 | default_config() -> Self |
| `evaluate` | projects/meter/src/agent_eval/llm_judge.rs | function | pub | 183 | evaluate(         &self,         input: &str,         expected: Option<&str>,         actual: &str,     ) -> Result<QualityScores, String> |
| `new` | projects/meter/src/agent_eval/llm_judge.rs | function | pub | 60 | new(model: impl Into<String>, provider: impl Into<String>) -> Self |
| `new` | projects/meter/src/agent_eval/llm_judge.rs | function | pub | 130 | new(config: LLMJudgeConfig) -> Self |
| `with_criteria` | projects/meter/src/agent_eval/llm_judge.rs | function | pub | 75 | with_criteria(mut self, criteria: Vec<QualityCriterion>) -> Self |
| `with_structured_output` | projects/meter/src/agent_eval/llm_judge.rs | function | pub | 81 | with_structured_output(mut self, enabled: bool) -> Self |
| `with_temperature` | projects/meter/src/agent_eval/llm_judge.rs | function | pub | 69 | with_temperature(mut self, temperature: f32) -> Self |
| `with_template` | projects/meter/src/agent_eval/llm_judge.rs | function | pub | 87 | with_template(mut self, template_name: impl Into<String>) -> Self |
| `with_template_dir` | projects/meter/src/agent_eval/llm_judge.rs | function | pub | 154 | with_template_dir(         config: LLMJudgeConfig,         template_dir: impl AsRef<Path>,     ) -> Result<Self, String> |
| `with_template_version` | projects/meter/src/agent_eval/llm_judge.rs | function | pub | 93 | with_template_version(         mut self,         template_name: impl Into<String>,         version: impl Into<String>,     ) -> Self |
| `without_templates` | projects/meter/src/agent_eval/llm_judge.rs | function | pub | 167 | without_templates(config: LLMJudgeConfig) -> Self |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! LLM-as-judge quality evaluation system

use crate::agent_eval::prompt::{PromptContext, PromptEngine, PromptRegistry};
use crate::agent_eval::result::QualityScores;
use crate::agent_eval::test_case::QualityCriterion;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// LLM-as-judge configuration
#[derive(Debug, Clone)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-llm-judge-rs.md#source
pub struct LLMJudgeConfig {
    /// Model to use for judging (e.g., "gpt-4o-mini", "claude-3-haiku")
    pub model: String,

    /// Provider (e.g., "openai", "anthropic")
    pub provider: String,

    /// Temperature for generation (lower = more consistent)
    pub temperature: f32,

    /// Quality criteria to evaluate
    pub criteria: Vec<QualityCriterion>,

    /// Enable structured output (JSON mode)
    pub structured_output: bool,

    /// Prompt template to use ("basic", "few_shot", "chain_of_thought", "self_consistency")
    pub template_name: String,

    /// Template version (defaults to latest if None)
    pub template_version: Option<String>,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-llm-judge-rs.md#source
impl Default for LLMJudgeConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4o-mini".to_string(),
            provider: "openai".to_string(),
            temperature: 0.0,
            criteria: vec![
                QualityCriterion::new("relevance", "Is the response relevant to the input?"),
                QualityCriterion::new("accuracy", "Is the response factually accurate?"),
                QualityCriterion::new("clarity", "Is the response clear and well-structured?"),
            ],
            structured_output: true,
            template_name: "llm_judge_basic".to_string(),
            template_version: None, // Use latest
        }
    }
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-llm-judge-rs.md#source
impl LLMJudgeConfig {
    /// Create a new LLM judge configuration
    pub fn new(model: impl Into<String>, provider: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            provider: provider.into(),
            ..Default::default()
        }
    }

    /// Set temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    /// Set criteria
    pub fn with_criteria(mut self, criteria: Vec<QualityCriterion>) -> Self {
        self.criteria = criteria;
        self
    }

    /// Enable/disable structured output
    pub fn with_structured_output(mut self, enabled: bool) -> Self {
        self.structured_output = enabled;
        self
    }

    /// Set prompt template
    pub fn with_template(mut self, template_name: impl Into<String>) -> Self {
        self.template_name = template_name.into();
        self
    }

    /// Set prompt template with specific version
    pub fn with_template_version(
        mut self,
        template_name: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        self.template_name = template_name.into();
        self.template_version = Some(version.into());
        self
    }
}

/// LLM judge response structure (for JSON parsing)
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-llm-judge-rs.md#source
pub struct LLMJudgeResponse {
    /// Individual criterion scores (0.0-1.0)
    pub scores: HashMap<String, f64>,

    /// Overall feedback/reasoning
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback: Option<String>,
}

/// LLM-as-judge evaluator
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-llm-judge-rs.md#source
pub struct LLMJudge {
    config: LLMJudgeConfig,
    registry: PromptRegistry,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-llm-judge-rs.md#source
impl LLMJudge {
    /// Create a new LLM judge with the given configuration
    ///
    /// Attempts to load templates from default locations:
    /// 1. `./templates/llm_judge/` (relative to current directory)
    /// 2. Falls back to empty registry if templates not found
    pub fn new(config: LLMJudgeConfig) -> Self {
        let mut registry = PromptRegistry::new();

        // Try to load templates from default location
        let default_template_dir = Path::new("templates/llm_judge");
        if default_template_dir.exists() {
            let _ = registry.load_from_directory(default_template_dir);
        }

        // Also try from crates/cclab-probe/templates/llm_judge (for tests)
        let alt_template_dir = Path::new("crates/cclab-probe/templates/llm_judge");
        if alt_template_dir.exists() {
            let _ = registry.load_from_directory(alt_template_dir);
        }

        Self { config, registry }
    }

    /// Create with default configuration
    pub fn default_config() -> Self {
        Self::new(LLMJudgeConfig::default())
    }

    /// Create with custom template directory
    pub fn with_template_dir(
        config: LLMJudgeConfig,
        template_dir: impl AsRef<Path>,
    ) -> Result<Self, String> {
        let mut registry = PromptRegistry::new();
        registry
            .load_from_directory(template_dir)
            .map_err(|e| format!("Failed to load templates: {}", e))?;

        Ok(Self { config, registry })
    }

    /// Create without any templates (uses legacy hardcoded prompts)
    pub fn without_templates(config: LLMJudgeConfig) -> Self {
        Self {
            config,
            registry: PromptRegistry::new(),
        }
    }

    /// Evaluate agent response quality
    ///
    /// # Arguments
    /// * `input` - Original user input
    /// * `expected` - Expected output (optional)
    /// * `actual` - Actual agent output
    ///
    /// # Returns
    /// Quality scores for each criterion
    pub async fn evaluate(
        &self,
        input: &str,
        expected: Option<&str>,
        actual: &str,
    ) -> Result<QualityScores, String> {
        // Build evaluation prompt using templates
        let prompt = self.build_prompt_from_template(input, expected, actual)?;

        // Call LLM (placeholder - would integrate with cclab-nova-llm)
        let response = self.call_llm(&prompt).await?;

        // Parse structured output
        let judge_response = self.parse_response(&response)?;

        // Calculate weighted average
        let overall_score = self.calculate_weighted_score(&judge_response.scores);

        Ok(QualityScores {
            scores: judge_response.scores,
            overall_score,
            feedback: judge_response.feedback,
        })
    }

    /// Build evaluation prompt using templates
    fn build_prompt_from_template(
        &self,
        input: &str,
        expected: Option<&str>,
        actual: &str,
    ) -> Result<String, String> {
        // Get template from registry
        let template = if let Some(ref version) = self.config.template_version {
            self.registry
                .get(&self.config.template_name, version)
                .ok_or_else(|| {
                    format!(
                        "Template '{}' version '{}' not found",
                        self.config.template_name, version
                    )
                })?
        } else {
            self.registry
                .get_latest(&self.config.template_name)
                .ok_or_else(|| format!("Template '{}' not found", self.config.template_name))?
        };

        // Build context
        let mut context = PromptContext::new();
        context.set("input", input);
        context.set("actual", actual);

        if let Some(exp) = expected {
            context.set("expected", exp);
            context.set("has_expected", "true");
        }

        // Format criteria
        let criteria_text = self
            .config
            .criteria
            .iter()
            .map(|c| format!("- **{}** (weight: {}): {}", c.name, c.weight, c.description))
            .collect::<Vec<_>>()
            .join("\n");
        context.set("criteria", &criteria_text);

        // Format criteria keys for JSON schema
        let criteria_keys = self
            .config
            .criteria
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let comma = if i < self.config.criteria.len() - 1 {
                    ","
                } else {
                    ""
                };
                format!("\"{}\"= 0.0{}", c.name, comma)
            })
            .collect::<Vec<_>>()
            .join("\n      ");
        context.set("criteria_keys", &criteria_keys);

        // Render prompt
        PromptEngine::render(template, &context)
    }

    /// Build evaluation prompt (legacy method, kept for backward compatibility)
    #[deprecated(since = "0.1.0", note = "Use build_prompt_from_template instead")]
    #[allow(dead_code)]
    fn build_prompt(&self, input: &str, expected: Option<&str>, actual: &str) -> String {
        self.build_prompt_from_template(input, expected, actual)
            .unwrap_or_else(|_| {
                // Fallback to hardcoded prompt if template fails
                let mut prompt = String::new();
                prompt.push_str("You are an expert evaluator assessing the quality of an AI agent's response.\n\n");
                prompt.push_str("# Input\n");
                prompt.push_str(input);
                prompt.push_str("\n\n");

                if let Some(exp) = expected {
                    prompt.push_str("# Expected Output\n");
                    prompt.push_str(exp);
                    prompt.push_str("\n\n");
                }

                prompt.push_str("# Actual Output\n");
                prompt.push_str(actual);
                prompt.push_str("\n\n");

                prompt.push_str("# Evaluation Criteria\n");
                for criterion in &self.config.criteria {
                    prompt.push_str(&format!(
                        "- **{}** (weight: {}): {}\n",
                        criterion.name, criterion.weight, criterion.description
                    ));
                }

                prompt.push_str("\n# Instructions\n");
                prompt.push_str("Evaluate the actual output against each criterion. ");
                prompt.push_str("Assign a score from 0.0 (terrible) to 1.0 (perfect) for each criterion.\n\n");

                if self.config.structured_output {
                    prompt.push_str("Respond in JSON format:\n");
                    prompt.push_str("```json\n");
                    prompt.push_str("{\n");
                    prompt.push_str("  \"scores\": {\n");
                    for (i, criterion) in self.config.criteria.iter().enumerate() {
                        prompt.push_str(&format!("    \"{}\": 0.0", criterion.name));
                        if i < self.config.criteria.len() - 1 {
                            prompt.push_str(",");
                        }
                        prompt.push('\n');
                    }
                    prompt.push_str("  },\n");
                    prompt.push_str("  \"feedback\": \"Brief explanation of the evaluation\"\n");
                    prompt.push_str("}\n");
                    prompt.push_str("```\n");
                }

                prompt
            })
    }

    /// Call LLM provider (placeholder - integrate with cclab-nova-llm)
    async fn call_llm(&self, _prompt: &str) -> Result<String, String> {
        // Placeholder implementation
        // In production, this would call cclab-nova-llm providers:
        // - OpenAI: use ChatCompletionRequest with JSON mode
        // - Anthropic: use Messages API with structured output
        // - etc.

        // For now, return a mock response for testing
        let mock_response = LLMJudgeResponse {
            scores: self
                .config
                .criteria
                .iter()
                .map(|c| (c.name.clone(), 0.85))
                .collect(),
            feedback: Some("Mock evaluation - integration pending".to_string()),
        };

        serde_json::to_string(&mock_response).map_err(|e| e.to_string())
    }

    /// Parse LLM response into structured format
    fn parse_response(&self, response: &str) -> Result<LLMJudgeResponse, String> {
        // Try to extract JSON from markdown code blocks
        let json_str = if response.contains("```json") {
            response
                .split("```json")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(response)
                .trim()
        } else if response.contains("```") {
            response
                .split("```")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(response)
                .trim()
        } else {
            response.trim()
        };

        // Parse JSON
        serde_json::from_str(json_str).map_err(|e| {
            format!(
                "Failed to parse LLM judge response as JSON: {}. Response: {}",
                e, json_str
            )
        })
    }

    /// Calculate weighted average score
    fn calculate_weighted_score(&self, scores: &HashMap<String, f64>) -> f64 {
        let mut total_weight = 0.0;
        let mut weighted_sum = 0.0;

        for criterion in &self.config.criteria {
            if let Some(&score) = scores.get(&criterion.name) {
                weighted_sum += score * criterion.weight;
                total_weight += criterion.weight;
            }
        }

        if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        }
    }

    /// Get configuration
    pub fn config(&self) -> &LLMJudgeConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_judge_config_default() {
        let config = LLMJudgeConfig::default();
        assert_eq!(config.model, "gpt-4o-mini");
        assert_eq!(config.provider, "openai");
        assert_eq!(config.temperature, 0.0);
        assert_eq!(config.criteria.len(), 3);
        assert!(config.structured_output);
    }

    #[test]
    fn test_llm_judge_config_builder() {
        let config = LLMJudgeConfig::new("claude-3-haiku", "anthropic")
            .with_temperature(0.5)
            .with_structured_output(false);

        assert_eq!(config.model, "claude-3-haiku");
        assert_eq!(config.provider, "anthropic");
        assert_eq!(config.temperature, 0.5);
        assert!(!config.structured_output);
    }

    #[test]
    #[allow(deprecated)]
    fn test_build_prompt_legacy() {
        let config = LLMJudgeConfig::default();
        let judge = LLMJudge::without_templates(config);

        let prompt = judge.build_prompt("What is 2+2?", Some("4"), "The answer is 4");

        assert!(prompt.contains("What is 2+2?"));
        assert!(prompt.contains("Expected Output"));
        assert!(prompt.contains("The answer is 4"));
        assert!(prompt.contains("relevance"));
        assert!(prompt.contains("accuracy"));
        assert!(prompt.contains("clarity"));
        assert!(prompt.contains("JSON format"));
    }

    #[test]
    fn test_build_prompt_from_template() {
        // Load templates from the templates directory
        let config = LLMJudgeConfig::default();
        let template_dir = Path::new("templates/llm_judge");

        // Skip test if templates not found (e.g., during cargo test)
        if !template_dir.exists() {
            eprintln!("Skipping test: templates directory not found");
            return;
        }

        let judge =
            LLMJudge::with_template_dir(config, template_dir).expect("Failed to load templates");

        let prompt = judge
            .build_prompt_from_template("What is 2+2?", Some("4"), "The answer is 4")
            .expect("Failed to build prompt");

        assert!(prompt.contains("What is 2+2?"));
        assert!(prompt.contains("Expected Output"));
        assert!(prompt.contains("The answer is 4"));
        assert!(prompt.contains("relevance"));
        assert!(prompt.contains("accuracy"));
        assert!(prompt.contains("clarity"));
    }

    #[test]
    fn test_parse_response_plain_json() {
        let config = LLMJudgeConfig::default();
        let judge = LLMJudge::new(config);

        let response =
            r#"{"scores": {"relevance": 0.9, "accuracy": 0.85}, "feedback": "Good response"}"#;
        let parsed = judge.parse_response(response).unwrap();

        assert_eq!(parsed.scores.get("relevance"), Some(&0.9));
        assert_eq!(parsed.scores.get("accuracy"), Some(&0.85));
        assert_eq!(parsed.feedback, Some("Good response".to_string()));
    }

    #[test]
    fn test_parse_response_markdown_json() {
        let config = LLMJudgeConfig::default();
        let judge = LLMJudge::new(config);

        let response = r#"Here is my evaluation:
```json
{"scores": {"relevance": 0.9, "accuracy": 0.85}, "feedback": "Good response"}
```"#;
        let parsed = judge.parse_response(response).unwrap();

        assert_eq!(parsed.scores.get("relevance"), Some(&0.9));
        assert_eq!(parsed.scores.get("accuracy"), Some(&0.85));
    }

    #[test]
    fn test_calculate_weighted_score() {
        let config = LLMJudgeConfig::default().with_criteria(vec![
            QualityCriterion::new("relevance", "Test").with_weight(2.0),
            QualityCriterion::new("accuracy", "Test").with_weight(1.0),
        ]);
        let judge = LLMJudge::new(config);

        let mut scores = HashMap::new();
        scores.insert("relevance".to_string(), 0.8);
        scores.insert("accuracy".to_string(), 0.6);

        // Weighted: (0.8 * 2.0 + 0.6 * 1.0) / (2.0 + 1.0) = 2.2 / 3.0 = 0.7333...
        let weighted = judge.calculate_weighted_score(&scores);
        assert!((weighted - 0.7333333).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_evaluate_mock() {
        let judge = LLMJudge::default_config();

        let result = judge
            .evaluate("What is 2+2?", Some("4"), "The answer is 4")
            .await;

        assert!(result.is_ok());
        let quality_scores = result.unwrap();
        assert!(quality_scores.overall_score > 0.0);
        assert!(quality_scores.scores.len() >= 3);
        assert!(quality_scores.feedback.is_some());
    }

    #[test]
    fn test_parse_response_invalid_json() {
        let config = LLMJudgeConfig::default();
        let judge = LLMJudge::new(config);

        let response = "This is not JSON";
        let result = judge.parse_response(response);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse"));
    }

    #[test]
    fn test_template_configuration() {
        let config = LLMJudgeConfig::default().with_template("llm_judge_few_shot");

        assert_eq!(config.template_name, "llm_judge_few_shot");
        assert!(config.template_version.is_none());

        let config_with_version =
            LLMJudgeConfig::default().with_template_version("llm_judge_cot", "1.0.0");

        assert_eq!(config_with_version.template_name, "llm_judge_cot");
        assert_eq!(
            config_with_version.template_version,
            Some("1.0.0".to_string())
        );
    }

    #[test]
    fn test_different_template_types() {
        let template_dir = Path::new("templates/llm_judge");

        // Skip test if templates not found
        if !template_dir.exists() {
            eprintln!("Skipping test: templates directory not found");
            return;
        }

        // Test basic template
        let config_basic = LLMJudgeConfig::default().with_template("llm_judge_basic");
        let judge_basic = LLMJudge::with_template_dir(config_basic, template_dir)
            .expect("Failed to load templates");
        let prompt_basic = judge_basic
            .build_prompt_from_template("Test input", None, "Test output")
            .expect("Failed to build basic prompt");
        assert!(prompt_basic.contains("Test input"));

        // Test few-shot template
        let config_few_shot = LLMJudgeConfig::default().with_template("llm_judge_few_shot");
        let judge_few_shot = LLMJudge::with_template_dir(config_few_shot, template_dir)
            .expect("Failed to load templates");
        let prompt_few_shot = judge_few_shot
            .build_prompt_from_template("Test input", None, "Test output")
            .expect("Failed to build few-shot prompt");
        assert!(prompt_few_shot.contains("Test input"));

        // Test chain-of-thought template
        let config_cot = LLMJudgeConfig::default().with_template("llm_judge_cot");
        let judge_cot = LLMJudge::with_template_dir(config_cot, template_dir)
            .expect("Failed to load templates");
        let prompt_cot = judge_cot
            .build_prompt_from_template("Test input", None, "Test output")
            .expect("Failed to build CoT prompt");
        assert!(prompt_cot.contains("Test input"));
        assert!(prompt_cot.contains("step by step") || prompt_cot.contains("Step"));

        // Test self-consistency template
        let config_sc = LLMJudgeConfig::default().with_template("llm_judge_self_consistency");
        let judge_sc =
            LLMJudge::with_template_dir(config_sc, template_dir).expect("Failed to load templates");
        let prompt_sc = judge_sc
            .build_prompt_from_template("Test input", None, "Test output")
            .expect("Failed to build self-consistency prompt");
        assert!(prompt_sc.contains("Test input"));
    }

    #[test]
    fn test_template_not_found() {
        let config = LLMJudgeConfig::default().with_template("nonexistent_template");
        let judge = LLMJudge::without_templates(config);

        let result = judge.build_prompt_from_template("Test input", None, "Test output");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_fallback_to_legacy_prompt() {
        let config = LLMJudgeConfig::default().with_template("nonexistent_template");
        let judge = LLMJudge::without_templates(config);

        // The deprecated build_prompt method should fall back to hardcoded prompt
        #[allow(deprecated)]
        let prompt = judge.build_prompt("What is 2+2?", Some("4"), "The answer is 4");

        // Should still work with fallback
        assert!(prompt.contains("What is 2+2?"));
        assert!(prompt.contains("The answer is 4"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/agent_eval/llm_judge.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/agent_eval/llm_judge.rs` captured during meter full-codegen standardization.
```
