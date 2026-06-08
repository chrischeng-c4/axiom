---
id: projects-meter-src-agent-eval-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: json-default-report-envelope-and-findings
    claim: json-default-report-envelope-and-findings
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/src/agent_eval/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/agent_eval/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `cost` | projects/meter/src/agent_eval/mod.rs | module | pub | 47 |  |
| `dataset` | projects/meter/src/agent_eval/mod.rs | module | pub | 48 |  |
| `evaluator` | projects/meter/src/agent_eval/mod.rs | module | pub | 49 |  |
| `llm_judge` | projects/meter/src/agent_eval/mod.rs | module | pub | 50 |  |
| `prompt` | projects/meter/src/agent_eval/mod.rs | module | pub | 51 |  |
| `regression` | projects/meter/src/agent_eval/mod.rs | module | pub | 52 |  |
| `result` | projects/meter/src/agent_eval/mod.rs | module | pub | 53 |  |
| `test_case` | projects/meter/src/agent_eval/mod.rs | module | pub | 54 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/meter/src/agent_eval/mod.rs -->
````rust
//! Agent Evaluation Framework - Integration with cclab-probe
//!
//! Extends cclab-probe test framework with agent-specific evaluation capabilities:
//! - Correctness checking (exact match, regex, semantic similarity)
//! - Tool accuracy measurement (precision, recall, F1)
//! - Cost tracking (model pricing + token usage)
//! - Latency profiling (P50, P95, P99 percentiles)
//! - LLM-as-judge quality assessment with prompt templates
//!
//! ## Prompt Template System
//!
//! The framework includes a flexible prompt template system for LLM-as-judge evaluation:
//! - **Basic**: Simple evaluation prompt (default)
//! - **Few-Shot**: Calibration examples for improved consistency
//! - **Chain-of-Thought**: Step-by-step reasoning for explainability
//! - **Self-Consistency**: Multiple sampling for high reliability
//!
//! Templates are loaded from `templates/llm_judge/` and support:
//! - Variable substitution with `{{variable}}` syntax
//! - Conditional sections based on context
//! - Few-shot examples
//! - Version management
//! - Custom template creation via YAML
//!
//! # Example
//!
//! ```rust
//! use meter::agent_eval::{AgentEvaluator, AgentTestCase};
//!
//! let test_cases = vec![
//!     AgentTestCase {
//!         id: "test-001".to_string(),
//!         name: "Capital question".to_string(),
//!         input: "What's the capital of France?".to_string(),
//!         expected_output_regex: Some(r"Paris".to_string()),
//!         max_latency_ms: Some(2000.0),
//!         max_cost_usd: Some(0.01),
//!         ..Default::default()
//!     },
//! ];
//!
//! let evaluator = AgentEvaluator::new(test_cases);
//! ```

pub mod cost;
pub mod dataset;
pub mod evaluator;
pub mod llm_judge;
pub mod prompt;
pub mod regression;
pub mod result;
pub mod test_case;

// Re-export main types
pub use cost::{CostCalculator, ModelPricing, PricingRegistry};
pub use dataset::{DatasetGitIntegration, DatasetMetadata, DatasetSnapshot, GoldenDataset};
pub use evaluator::AgentEvaluator;
pub use llm_judge::{LLMJudge, LLMJudgeConfig, LLMJudgeResponse};
pub use prompt::{
    FewShotExample, PromptContext, PromptEngine, PromptRegistry, PromptSection, PromptTemplate,
    PromptVariable,
};
pub use regression::{
    AgentRegression, AgentRegressionDetector, AgentRegressionReport, AgentRegressionSummary,
    AgentRegressionThresholds,
};
pub use result::{
    AgentEvalMetrics, AgentEvalResult, CorrectnessMetrics, CorrectnessResult, CostMetrics,
    CostStats, LatencyMetrics, MatchType, QualityMetrics, QualityScores, ToolAccuracyResult,
    ToolUsageMetrics,
};
pub use test_case::{AgentTestCase, ExpectedToolCall, QualityCriterion};
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/agent_eval/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/agent_eval/mod.rs` captured during meter full-codegen standardization.
```
