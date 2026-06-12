---
id: projects-meter-src-agent-eval-evaluator-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: json-default-report-envelope-and-findings
    claim: json-default-report-envelope-and-findings
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/src/agent_eval/evaluator.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/agent_eval/evaluator.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AgentEvalSuiteResult` | projects/meter/src/agent_eval/evaluator.rs | struct | pub | 452 |  |
| `AgentEvaluator` | projects/meter/src/agent_eval/evaluator.rs | struct | pub | 19 |  |
| `AgentResponseData` | projects/meter/src/agent_eval/evaluator.rs | struct | pub | 423 |  |
| `TokenUsage` | projects/meter/src/agent_eval/evaluator.rs | struct | pub | 443 |  |
| `cost_calculator` | projects/meter/src/agent_eval/evaluator.rs | function | pub | 415 | cost_calculator(&self) -> &CostCalculator |
| `evaluate_suite` | projects/meter/src/agent_eval/evaluator.rs | function | pub | 263 | evaluate_suite(         &self,         responses: Vec<AgentResponseData>,     ) -> Result<AgentEvalSuiteResult, String> |
| `evaluate_test_case` | projects/meter/src/agent_eval/evaluator.rs | function | pub | 62 | evaluate_test_case(         &self,         test_case: &AgentTestCase,         agent_response: &AgentResponseData,     ) -> AgentEvalResult |
| `new` | projects/meter/src/agent_eval/evaluator.rs | function | pub | 28 | new(test_cases: Vec<AgentTestCase>) -> Self |
| `test_cases` | projects/meter/src/agent_eval/evaluator.rs | function | pub | 410 | test_cases(&self) -> &[AgentTestCase] |
| `with_llm_judge` | projects/meter/src/agent_eval/evaluator.rs | function | pub | 49 | with_llm_judge(mut self, judge: LLMJudge) -> Self |
| `with_pricing_registry` | projects/meter/src/agent_eval/evaluator.rs | function | pub | 37 | with_pricing_registry(         test_cases: Vec<AgentTestCase>,         registry: PricingRegistry,     ) -> Self |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Core agent evaluation logic

use crate::agent_eval::{
    cost::{CostCalculator, PricingRegistry},
    llm_judge::LLMJudge,
    result::*,
    test_case::{AgentTestCase, ExpectedToolCall},
};
use crate::benchmark::BenchmarkStats;
use chrono::Utc;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

/// Agent evaluator for running test cases
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-evaluator-rs.md#source
pub struct AgentEvaluator {
    test_cases: Vec<AgentTestCase>,
    cost_calculator: CostCalculator,
    llm_judge: Option<LLMJudge>,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-evaluator-rs.md#source
impl AgentEvaluator {
    /// Create a new agent evaluator
    pub fn new(test_cases: Vec<AgentTestCase>) -> Self {
        Self {
            test_cases,
            cost_calculator: CostCalculator::new(),
            llm_judge: None,
        }
    }

    /// Create with custom pricing registry
    pub fn with_pricing_registry(
        test_cases: Vec<AgentTestCase>,
        registry: PricingRegistry,
    ) -> Self {
        Self {
            test_cases,
            cost_calculator: CostCalculator::with_registry(registry),
            llm_judge: None,
        }
    }

    /// Enable LLM-as-judge quality evaluation
    pub fn with_llm_judge(mut self, judge: LLMJudge) -> Self {
        self.llm_judge = Some(judge);
        self
    }

    /// Evaluate a single test case
    ///
    /// # Arguments
    /// * `test_case` - The test case to evaluate
    /// * `agent_response` - Response from the agent (content, tool_calls, usage, model)
    ///
    /// # Returns
    /// AgentEvalResult containing all evaluation metrics
    pub async fn evaluate_test_case(
        &self,
        test_case: &AgentTestCase,
        agent_response: &AgentResponseData,
    ) -> AgentEvalResult {
        let start = Instant::now();

        // Evaluate correctness
        let correctness = self.evaluate_correctness(test_case, &agent_response.content);

        // Evaluate tool accuracy
        let tool_accuracy =
            self.evaluate_tool_accuracy(&test_case.expected_tools, &agent_response.tool_calls);

        // Calculate latency
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0 + agent_response.latency_ms;
        let latency = LatencyMetrics {
            total_ms: latency_ms,
            within_budget: test_case
                .max_latency_ms
                .map_or(true, |max| latency_ms <= max),
            budget_ms: test_case.max_latency_ms,
        };

        // Calculate cost
        let cost_result = self.cost_calculator.calculate_cost(
            &agent_response.model,
            agent_response.usage.prompt_tokens,
            agent_response.usage.completion_tokens,
        );

        let total_cost = cost_result.unwrap_or(0.0);
        let cost = CostMetrics {
            total_cost_usd: total_cost,
            prompt_tokens: agent_response.usage.prompt_tokens,
            completion_tokens: agent_response.usage.completion_tokens,
            total_tokens: agent_response.usage.total_tokens,
            within_budget: test_case.max_cost_usd.map_or(true, |max| total_cost <= max),
            budget_usd: test_case.max_cost_usd,
            model: agent_response.model.clone(),
        };

        // Determine overall pass/fail
        let passed = correctness.matches
            && tool_accuracy.f1_score >= 0.8
            && latency.within_budget
            && cost.within_budget;

        // Evaluate quality with LLM-as-judge if enabled
        let quality_scores = if let Some(ref judge) = self.llm_judge {
            judge
                .evaluate(
                    &test_case.input,
                    test_case.expected_output.as_deref(),
                    &agent_response.content,
                )
                .await
                .ok()
        } else {
            None
        };

        let failure_reason = if !passed {
            let mut reasons = Vec::new();
            if !correctness.matches {
                reasons.push("Correctness check failed");
            }
            if tool_accuracy.f1_score < 0.8 {
                reasons.push("Tool accuracy below threshold (F1 < 0.8)");
            }
            if !latency.within_budget {
                reasons.push("Latency exceeded budget");
            }
            if !cost.within_budget {
                reasons.push("Cost exceeded budget");
            }
            Some(reasons.join("; "))
        } else {
            None
        };

        AgentEvalResult {
            test_case_id: test_case.id.clone(),
            test_case_name: test_case.name.clone(),
            passed,
            actual_output: agent_response.content.clone(),
            correctness,
            tool_accuracy,
            quality_scores,
            latency,
            cost,
            timestamp: Utc::now(),
            failure_reason,
        }
    }

    /// Evaluate correctness of agent output
    fn evaluate_correctness(
        &self,
        test_case: &AgentTestCase,
        actual_output: &str,
    ) -> CorrectnessResult {
        // Try exact match first
        if let Some(ref expected) = test_case.expected_output {
            if actual_output == expected {
                return CorrectnessResult::passed(MatchType::Exact);
            }
            // Try contains match
            if actual_output.contains(expected) {
                return CorrectnessResult::passed(MatchType::Contains);
            }
        }

        // Try regex match
        if let Some(ref pattern) = test_case.expected_output_regex {
            match Regex::new(pattern) {
                Ok(re) => {
                    if re.is_match(actual_output) {
                        return CorrectnessResult::passed(MatchType::Regex);
                    }
                }
                Err(e) => {
                    return CorrectnessResult::failed(format!("Invalid regex pattern: {}", e));
                }
            }
        }

        // If no expected output specified, pass by default
        if test_case.expected_output.is_none() && test_case.expected_output_regex.is_none() {
            return CorrectnessResult::passed(MatchType::None);
        }

        // Failed to match
        CorrectnessResult::failed(format!(
            "Output did not match expected. Expected: {:?}, Got: {}",
            test_case
                .expected_output
                .as_ref()
                .or(test_case.expected_output_regex.as_ref()),
            actual_output.chars().take(100).collect::<String>()
        ))
    }

    /// Evaluate tool accuracy (precision, recall, F1)
    fn evaluate_tool_accuracy(
        &self,
        expected_tools: &[ExpectedToolCall],
        actual_tools: &[String],
    ) -> ToolAccuracyResult {
        // If no tools expected and none called, perfect score
        if expected_tools.is_empty() && actual_tools.is_empty() {
            return ToolAccuracyResult::default();
        }

        // Build sets for comparison
        let expected_names: HashSet<String> = expected_tools
            .iter()
            .filter(|t| !t.optional)
            .map(|t| t.name.clone())
            .collect();
        let actual_names: HashSet<String> = actual_tools.iter().cloned().collect();

        // Calculate metrics
        let true_positives = expected_names.intersection(&actual_names).count() as f64;
        let false_positives = actual_names.difference(&expected_names).count() as f64;
        let false_negatives = expected_names.difference(&actual_names).count() as f64;

        let precision = if true_positives + false_positives > 0.0 {
            true_positives / (true_positives + false_positives)
        } else {
            1.0
        };

        let recall = if true_positives + false_negatives > 0.0 {
            true_positives / (true_positives + false_negatives)
        } else {
            1.0
        };

        let f1_score = if precision + recall > 0.0 {
            2.0 * (precision * recall) / (precision + recall)
        } else {
            0.0
        };

        let missing_tools: Vec<String> =
            expected_names.difference(&actual_names).cloned().collect();

        let unexpected_tools: Vec<String> =
            actual_names.difference(&expected_names).cloned().collect();

        ToolAccuracyResult {
            precision,
            recall,
            f1_score,
            missing_tools,
            unexpected_tools,
        }
    }

    /// Evaluate multiple test cases and aggregate metrics
    pub async fn evaluate_suite(
        &self,
        responses: Vec<AgentResponseData>,
    ) -> Result<AgentEvalSuiteResult, String> {
        if responses.len() != self.test_cases.len() {
            return Err(format!(
                "Mismatch: {} test cases but {} responses",
                self.test_cases.len(),
                responses.len()
            ));
        }

        let mut results = Vec::new();
        let mut latency_times = Vec::new();

        for (test_case, response) in self.test_cases.iter().zip(responses.iter()) {
            let result = self.evaluate_test_case(test_case, response).await;
            latency_times.push(result.latency.total_ms);
            results.push(result);
        }

        let metrics = self.aggregate_metrics(&results, &latency_times);

        Ok(AgentEvalSuiteResult { results, metrics })
    }

    /// Aggregate metrics from individual results
    fn aggregate_metrics(
        &self,
        results: &[AgentEvalResult],
        latency_times: &[f64],
    ) -> AgentEvalMetrics {
        let total_cases = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let pass_rate = if total_cases > 0 {
            passed as f64 / total_cases as f64
        } else {
            0.0
        };

        // Correctness metrics
        let correct = results.iter().filter(|r| r.correctness.matches).count();
        let mut by_match_type: HashMap<String, usize> = HashMap::new();
        for result in results {
            if result.correctness.matches {
                let match_type_str = format!("{:?}", result.correctness.match_type);
                *by_match_type.entry(match_type_str).or_insert(0) += 1;
            }
        }

        let correctness = CorrectnessMetrics {
            total: total_cases,
            correct,
            rate: if total_cases > 0 {
                correct as f64 / total_cases as f64
            } else {
                0.0
            },
            by_match_type,
        };

        // Tool usage metrics
        let avg_precision = if total_cases > 0 {
            results
                .iter()
                .map(|r| r.tool_accuracy.precision)
                .sum::<f64>()
                / total_cases as f64
        } else {
            0.0
        };

        let avg_recall = if total_cases > 0 {
            results.iter().map(|r| r.tool_accuracy.recall).sum::<f64>() / total_cases as f64
        } else {
            0.0
        };

        let avg_f1_score = if total_cases > 0 {
            results
                .iter()
                .map(|r| r.tool_accuracy.f1_score)
                .sum::<f64>()
                / total_cases as f64
        } else {
            0.0
        };

        let total_tools_called: usize = results
            .iter()
            .map(|r| r.tool_accuracy.missing_tools.len() + r.tool_accuracy.unexpected_tools.len())
            .sum();

        let tool_usage = ToolUsageMetrics {
            avg_precision,
            avg_recall,
            avg_f1_score,
            total_tools_called,
            total_expected_tools: self.test_cases.iter().map(|t| t.expected_tools.len()).sum(),
        };

        // Latency stats (reuse BenchmarkStats)
        let latency_stats =
            BenchmarkStats::from_times(latency_times.to_vec(), total_cases as u32, 1, 0);

        // Cost stats
        let costs: Vec<f64> = results.iter().map(|r| r.cost.total_cost_usd).collect();
        let total_cost_usd = costs.iter().sum();
        let avg_cost_per_case_usd = if total_cases > 0 {
            total_cost_usd / total_cases as f64
        } else {
            0.0
        };

        let min_cost_usd = costs.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_cost_usd = costs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        let total_tokens: u64 = results.iter().map(|r| r.cost.total_tokens as u64).sum();
        let total_prompt_tokens: u64 = results.iter().map(|r| r.cost.prompt_tokens as u64).sum();
        let total_completion_tokens: u64 = results
            .iter()
            .map(|r| r.cost.completion_tokens as u64)
            .sum();

        let cost_stats = CostStats {
            total_cost_usd,
            avg_cost_per_case_usd,
            total_tokens,
            total_prompt_tokens,
            total_completion_tokens,
            min_cost_usd,
            max_cost_usd,
        };

        AgentEvalMetrics {
            total_cases,
            passed,
            pass_rate,
            correctness,
            tool_usage,
            quality: None, // Populated in Phase 3
            latency_stats,
            cost_stats,
        }
    }

    /// Get test cases
    pub fn test_cases(&self) -> &[AgentTestCase] {
        &self.test_cases
    }

    /// Get cost calculator
    pub fn cost_calculator(&self) -> &CostCalculator {
        &self.cost_calculator
    }
}

/// Agent response data structure
#[derive(Debug, Clone)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-evaluator-rs.md#source
pub struct AgentResponseData {
    /// Response content/text
    pub content: String,

    /// Tools called (just names)
    pub tool_calls: Vec<String>,

    /// Token usage
    pub usage: TokenUsage,

    /// Model used
    pub model: String,

    /// Latency in milliseconds (measured externally)
    pub latency_ms: f64,
}

/// Token usage (simplified version)
#[derive(Debug, Clone)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-evaluator-rs.md#source
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Suite evaluation result containing all test results and aggregated metrics
#[derive(Debug, Clone)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-evaluator-rs.md#source
pub struct AgentEvalSuiteResult {
    pub results: Vec<AgentEvalResult>,
    pub metrics: AgentEvalMetrics,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_response(
        content: &str,
        model: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
    ) -> AgentResponseData {
        AgentResponseData {
            content: content.to_string(),
            tool_calls: Vec::new(),
            usage: TokenUsage {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
            },
            model: model.to_string(),
            latency_ms: 0.0,
        }
    }

    #[test]
    fn test_evaluate_correctness_exact() {
        let test_case =
            AgentTestCase::new("test-001", "Test", "Input").with_expected_output("Paris");

        let evaluator = AgentEvaluator::new(vec![test_case.clone()]);
        let result = evaluator.evaluate_correctness(&test_case, "Paris");

        assert!(result.matches);
        assert_eq!(result.match_type, MatchType::Exact);
    }

    #[test]
    fn test_evaluate_correctness_contains() {
        let test_case =
            AgentTestCase::new("test-001", "Test", "Input").with_expected_output("Paris");

        let evaluator = AgentEvaluator::new(vec![test_case.clone()]);
        let result = evaluator.evaluate_correctness(&test_case, "The capital is Paris, France");

        assert!(result.matches);
        assert_eq!(result.match_type, MatchType::Contains);
    }

    #[test]
    fn test_evaluate_correctness_regex() {
        let test_case = AgentTestCase::new("test-001", "Test", "Input")
            .with_expected_output_regex(r"\bParis\b");

        let evaluator = AgentEvaluator::new(vec![test_case.clone()]);
        let result = evaluator.evaluate_correctness(&test_case, "The answer is Paris");

        assert!(result.matches);
        assert_eq!(result.match_type, MatchType::Regex);
    }

    #[test]
    fn test_evaluate_correctness_failed() {
        let test_case =
            AgentTestCase::new("test-001", "Test", "Input").with_expected_output("Paris");

        let evaluator = AgentEvaluator::new(vec![test_case.clone()]);
        let result = evaluator.evaluate_correctness(&test_case, "London");

        assert!(!result.matches);
    }

    #[test]
    fn test_evaluate_tool_accuracy_perfect() {
        let expected = vec![
            ExpectedToolCall::new("search"),
            ExpectedToolCall::new("calculate"),
        ];
        let actual = vec!["search".to_string(), "calculate".to_string()];

        let evaluator = AgentEvaluator::new(vec![]);
        let result = evaluator.evaluate_tool_accuracy(&expected, &actual);

        assert_eq!(result.precision, 1.0);
        assert_eq!(result.recall, 1.0);
        assert_eq!(result.f1_score, 1.0);
        assert!(result.missing_tools.is_empty());
        assert!(result.unexpected_tools.is_empty());
    }

    #[test]
    fn test_evaluate_tool_accuracy_missing_tool() {
        let expected = vec![
            ExpectedToolCall::new("search"),
            ExpectedToolCall::new("calculate"),
        ];
        let actual = vec!["search".to_string()];

        let evaluator = AgentEvaluator::new(vec![]);
        let result = evaluator.evaluate_tool_accuracy(&expected, &actual);

        assert_eq!(result.precision, 1.0); // All called tools were expected
        assert_eq!(result.recall, 0.5); // Only called 1 of 2 expected
        assert!((result.f1_score - 0.6666666).abs() < 0.0001);
        assert_eq!(result.missing_tools, vec!["calculate"]);
        assert!(result.unexpected_tools.is_empty());
    }

    #[test]
    fn test_evaluate_tool_accuracy_unexpected_tool() {
        let expected = vec![ExpectedToolCall::new("search")];
        let actual = vec!["search".to_string(), "delete".to_string()];

        let evaluator = AgentEvaluator::new(vec![]);
        let result = evaluator.evaluate_tool_accuracy(&expected, &actual);

        assert_eq!(result.precision, 0.5); // 1 correct out of 2 called
        assert_eq!(result.recall, 1.0); // All expected tools were called
        assert!((result.f1_score - 0.6666666).abs() < 0.0001);
        assert!(result.missing_tools.is_empty());
        assert_eq!(result.unexpected_tools, vec!["delete"]);
    }

    #[tokio::test]
    async fn test_evaluate_test_case() {
        let test_case = AgentTestCase::new(
            "test-001",
            "Capital question",
            "What is the capital of France?",
        )
        .with_expected_output_regex(r"Paris")
        .with_max_latency_ms(2000.0)
        .with_max_cost_usd(0.01);

        let evaluator = AgentEvaluator::new(vec![test_case.clone()]);

        let response = create_test_response("The capital is Paris", "gpt-4o-mini", 100, 50);

        let result = evaluator.evaluate_test_case(&test_case, &response).await;

        assert!(result.passed);
        assert!(result.correctness.matches);
        assert!(result.latency.within_budget);
        assert!(result.cost.within_budget);
        assert!(result.failure_reason.is_none());
    }

    #[tokio::test]
    async fn test_evaluate_test_case_failed_correctness() {
        let test_case =
            AgentTestCase::new("test-001", "Test", "Input").with_expected_output("Paris");

        let evaluator = AgentEvaluator::new(vec![test_case.clone()]);
        let response = create_test_response("London", "gpt-4o-mini", 100, 50);

        let result = evaluator.evaluate_test_case(&test_case, &response).await;

        assert!(!result.passed);
        assert!(!result.correctness.matches);
        assert!(result.failure_reason.is_some());
    }

    #[tokio::test]
    async fn test_aggregate_metrics() {
        let test_cases = vec![
            AgentTestCase::new("test-001", "Test 1", "Input 1").with_expected_output("Output 1"),
            AgentTestCase::new("test-002", "Test 2", "Input 2").with_expected_output("Output 2"),
        ];

        let evaluator = AgentEvaluator::new(test_cases);

        let responses = vec![
            create_test_response("Output 1", "gpt-4o-mini", 100, 50),
            create_test_response("Wrong output", "gpt-4o-mini", 100, 50),
        ];

        let suite_result = evaluator.evaluate_suite(responses).await.unwrap();

        assert_eq!(suite_result.metrics.total_cases, 2);
        assert_eq!(suite_result.metrics.passed, 1);
        assert_eq!(suite_result.metrics.pass_rate, 0.5);
        assert_eq!(suite_result.metrics.correctness.correct, 1);
        assert_eq!(suite_result.metrics.correctness.rate, 0.5);
    }

    #[tokio::test]
    async fn test_evaluate_with_llm_judge() {
        use crate::agent_eval::llm_judge::{LLMJudge, LLMJudgeConfig};

        let test_case = AgentTestCase::new("test-001", "Quality test", "What is 2+2?")
            .with_expected_output("4");

        let judge_config = LLMJudgeConfig::default();
        let judge = LLMJudge::new(judge_config);

        let evaluator = AgentEvaluator::new(vec![test_case.clone()]).with_llm_judge(judge);

        let response = create_test_response("The answer is 4", "gpt-4o-mini", 100, 50);

        let result = evaluator.evaluate_test_case(&test_case, &response).await;

        assert!(result.passed);
        assert!(result.correctness.matches);
        assert!(result.quality_scores.is_some());

        let quality = result.quality_scores.unwrap();
        assert!(quality.overall_score > 0.0);
        assert!(!quality.scores.is_empty());
        assert!(quality.feedback.is_some());
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/agent_eval/evaluator.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/agent_eval/evaluator.rs` captured during meter full-codegen standardization.
```
