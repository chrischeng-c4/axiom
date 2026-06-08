// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-result-rs.md#source
// CODEGEN-BEGIN
//! Result types for agent evaluation

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Match type for correctness evaluation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-result-rs.md#source
pub enum MatchType {
    /// Exact string match
    Exact,
    /// Contains substring
    Contains,
    /// Regex pattern match
    Regex,
    /// Semantic similarity above threshold
    Semantic,
    /// No match
    None,
}

/// Correctness evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-result-rs.md#source
pub struct CorrectnessResult {
    /// Whether the output matches expected
    pub matches: bool,

    /// Type of match
    pub match_type: MatchType,

    /// Similarity score (0.0-1.0, for semantic matching)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub similarity_score: Option<f64>,

    /// Additional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-result-rs.md#source
impl CorrectnessResult {
    /// Create a passing correctness result
    pub fn passed(match_type: MatchType) -> Self {
        Self {
            matches: true,
            match_type,
            similarity_score: None,
            details: None,
        }
    }

    /// Create a failing correctness result
    pub fn failed(details: impl Into<String>) -> Self {
        Self {
            matches: false,
            match_type: MatchType::None,
            similarity_score: None,
            details: Some(details.into()),
        }
    }

    /// Set similarity score
    pub fn with_similarity_score(mut self, score: f64) -> Self {
        self.similarity_score = Some(score);
        self
    }
}

/// Tool accuracy evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-result-rs.md#source
pub struct ToolAccuracyResult {
    /// Precision: TP / (TP + FP)
    pub precision: f64,

    /// Recall: TP / (TP + FN)
    pub recall: f64,

    /// F1 score: 2 * (precision * recall) / (precision + recall)
    pub f1_score: f64,

    /// Tools that were expected but not called
    pub missing_tools: Vec<String>,

    /// Tools that were called but not expected
    pub unexpected_tools: Vec<String>,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-result-rs.md#source
impl Default for ToolAccuracyResult {
    fn default() -> Self {
        Self {
            precision: 1.0,
            recall: 1.0,
            f1_score: 1.0,
            missing_tools: Vec::new(),
            unexpected_tools: Vec::new(),
        }
    }
}

/// Latency metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-result-rs.md#source
pub struct LatencyMetrics {
    /// Total latency in milliseconds
    pub total_ms: f64,

    /// Whether latency is within budget
    pub within_budget: bool,

    /// Budget threshold (if specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_ms: Option<f64>,
}

/// Cost metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-result-rs.md#source
pub struct CostMetrics {
    /// Total cost in USD
    pub total_cost_usd: f64,

    /// Number of prompt tokens
    pub prompt_tokens: u32,

    /// Number of completion tokens
    pub completion_tokens: u32,

    /// Total tokens
    pub total_tokens: u32,

    /// Whether cost is within budget
    pub within_budget: bool,

    /// Budget threshold (if specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_usd: Option<f64>,

    /// Model used for cost calculation
    pub model: String,
}

/// Quality scores from LLM-as-judge
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-result-rs.md#source
pub struct QualityScores {
    /// Individual criterion scores (0.0-1.0)
    pub scores: HashMap<String, f64>,

    /// Weighted average score
    pub overall_score: f64,

    /// Textual feedback from judge
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback: Option<String>,
}

/// Single agent evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-result-rs.md#source
pub struct AgentEvalResult {
    /// Test case ID
    pub test_case_id: String,

    /// Test case name
    pub test_case_name: String,

    /// Whether the test passed overall
    pub passed: bool,

    /// Actual agent output
    pub actual_output: String,

    /// Correctness evaluation
    pub correctness: CorrectnessResult,

    /// Tool accuracy evaluation
    pub tool_accuracy: ToolAccuracyResult,

    /// Quality scores (from LLM-as-judge, if enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality_scores: Option<QualityScores>,

    /// Latency metrics
    pub latency: LatencyMetrics,

    /// Cost metrics
    pub cost: CostMetrics,

    /// When the evaluation was performed
    pub timestamp: DateTime<Utc>,

    /// Failure reason (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-result-rs.md#source
impl AgentEvalResult {
    /// Determine if result passed all checks
    pub fn determine_passed(&self) -> bool {
        self.correctness.matches
            && self.tool_accuracy.f1_score >= 0.8 // Configurable threshold
            && self.latency.within_budget
            && self.cost.within_budget
    }
}

/// Aggregated correctness metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-result-rs.md#source
pub struct CorrectnessMetrics {
    /// Total test cases
    pub total: usize,

    /// Number of correct responses
    pub correct: usize,

    /// Correctness rate (0.0-1.0)
    pub rate: f64,

    /// Breakdown by match type
    pub by_match_type: HashMap<String, usize>,
}

/// Aggregated tool usage metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-result-rs.md#source
pub struct ToolUsageMetrics {
    /// Average precision across all tests
    pub avg_precision: f64,

    /// Average recall across all tests
    pub avg_recall: f64,

    /// Average F1 score across all tests
    pub avg_f1_score: f64,

    /// Total tools called
    pub total_tools_called: usize,

    /// Total expected tools
    pub total_expected_tools: usize,
}

/// Aggregated quality metrics (from LLM-as-judge)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-result-rs.md#source
pub struct QualityMetrics {
    /// Average scores by criterion
    pub avg_scores_by_criterion: HashMap<String, f64>,

    /// Overall average quality score
    pub overall_avg_score: f64,

    /// Number of evaluations
    pub num_evaluations: usize,
}

/// Aggregated cost statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-result-rs.md#source
pub struct CostStats {
    /// Total cost in USD across all tests
    pub total_cost_usd: f64,

    /// Average cost per test case in USD
    pub avg_cost_per_case_usd: f64,

    /// Total tokens used
    pub total_tokens: u64,

    /// Total prompt tokens
    pub total_prompt_tokens: u64,

    /// Total completion tokens
    pub total_completion_tokens: u64,

    /// Minimum cost
    pub min_cost_usd: f64,

    /// Maximum cost
    pub max_cost_usd: f64,
}

/// Aggregated agent evaluation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-result-rs.md#source
pub struct AgentEvalMetrics {
    /// Total number of test cases
    pub total_cases: usize,

    /// Number of passed tests
    pub passed: usize,

    /// Pass rate (0.0-1.0)
    pub pass_rate: f64,

    /// Correctness metrics
    pub correctness: CorrectnessMetrics,

    /// Tool usage metrics
    pub tool_usage: ToolUsageMetrics,

    /// Quality metrics (if LLM-as-judge was used)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<QualityMetrics>,

    /// Latency statistics (reuses BenchmarkStats)
    pub latency_stats: crate::benchmark::BenchmarkStats,

    /// Cost statistics
    pub cost_stats: CostStats,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-result-rs.md#source
impl Default for AgentEvalMetrics {
    fn default() -> Self {
        Self {
            total_cases: 0,
            passed: 0,
            pass_rate: 0.0,
            correctness: CorrectnessMetrics::default(),
            tool_usage: ToolUsageMetrics::default(),
            quality: None,
            latency_stats: crate::benchmark::BenchmarkStats::default(),
            cost_stats: CostStats::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correctness_result_passed() {
        let result = CorrectnessResult::passed(MatchType::Exact);
        assert!(result.matches);
        assert_eq!(result.match_type, MatchType::Exact);
    }

    #[test]
    fn test_correctness_result_failed() {
        let result = CorrectnessResult::failed("Expected 'Paris' but got 'London'");
        assert!(!result.matches);
        assert_eq!(result.match_type, MatchType::None);
        assert!(result.details.is_some());
    }

    #[test]
    fn test_tool_accuracy_defaults() {
        let result = ToolAccuracyResult::default();
        assert_eq!(result.precision, 1.0);
        assert_eq!(result.recall, 1.0);
        assert_eq!(result.f1_score, 1.0);
        assert!(result.missing_tools.is_empty());
        assert!(result.unexpected_tools.is_empty());
    }

    #[test]
    fn test_latency_metrics() {
        let metrics = LatencyMetrics {
            total_ms: 1500.0,
            within_budget: true,
            budget_ms: Some(2000.0),
        };
        assert_eq!(metrics.total_ms, 1500.0);
        assert!(metrics.within_budget);
    }

    #[test]
    fn test_cost_metrics() {
        let metrics = CostMetrics {
            total_cost_usd: 0.005,
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
            within_budget: true,
            budget_usd: Some(0.01),
            model: "gpt-4o-mini".to_string(),
        };
        assert_eq!(metrics.total_cost_usd, 0.005);
        assert!(metrics.within_budget);
    }

    #[test]
    fn test_serialization() {
        let result = AgentEvalResult {
            test_case_id: "test-001".to_string(),
            test_case_name: "Test 1".to_string(),
            passed: true,
            actual_output: "Paris".to_string(),
            correctness: CorrectnessResult::passed(MatchType::Regex),
            tool_accuracy: ToolAccuracyResult::default(),
            quality_scores: None,
            latency: LatencyMetrics {
                total_ms: 1500.0,
                within_budget: true,
                budget_ms: Some(2000.0),
            },
            cost: CostMetrics {
                total_cost_usd: 0.005,
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
                within_budget: true,
                budget_usd: Some(0.01),
                model: "gpt-4o-mini".to_string(),
            },
            timestamp: Utc::now(),
            failure_reason: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: AgentEvalResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.test_case_id, "test-001");
        assert!(deserialized.passed);
    }
}
// CODEGEN-END
