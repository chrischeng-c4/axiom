// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-mod-rs.md#source
// CODEGEN-BEGIN
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
// CODEGEN-END
