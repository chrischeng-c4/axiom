---
id: projects-meter-src-agent-eval-regression-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: json-default-report-envelope-and-findings
    claim: json-default-report-envelope-and-findings
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/src/agent_eval/regression.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/agent_eval/regression.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AgentRegression` | projects/meter/src/agent_eval/regression.rs | struct | pub | 46 |  |
| `AgentRegressionDetector` | projects/meter/src/agent_eval/regression.rs | struct | pub | 115 |  |
| `AgentRegressionReport` | projects/meter/src/agent_eval/regression.rs | struct | pub | 69 |  |
| `AgentRegressionSummary` | projects/meter/src/agent_eval/regression.rs | struct | pub | 102 |  |
| `AgentRegressionThresholds` | projects/meter/src/agent_eval/regression.rs | struct | pub | 13 |  |
| `detect_regressions` | projects/meter/src/agent_eval/regression.rs | function | pub | 120 | detect_regressions(         baseline: &BaselineSnapshot,         current: &[AgentEvalResult],         thresholds: &AgentRegressionThresholds,     ) -> AgentRegressionReport |
| `has_regressions` | projects/meter/src/agent_eval/regression.rs | function | pub | 86 | has_regressions(&self) -> bool |
| `severe_regressions` | projects/meter/src/agent_eval/regression.rs | function | pub | 91 | severe_regressions(&self) -> Vec<&AgentRegression> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/meter/src/agent_eval/regression.rs -->
````rust
//! Agent evaluation regression detection

use crate::agent_eval::result::AgentEvalResult;
use crate::baseline::{BaselineSnapshot, RegressionSeverity};
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Thresholds for agent evaluation regression detection
#[derive(Debug, Clone)]
pub struct AgentRegressionThresholds {
    /// Pass rate decrease threshold as percentage (e.g., 5.0 = 5%)
    pub pass_rate_threshold_percent: f64,

    /// Latency P95 increase threshold as percentage (e.g., 15.0 = 15%)
    pub latency_p95_threshold_percent: f64,

    /// Cost increase threshold as percentage (e.g., 10.0 = 10%)
    pub cost_threshold_percent: f64,

    /// Tool accuracy (F1) decrease threshold (e.g., 0.05 = 5% decrease)
    pub tool_accuracy_threshold: f64,

    /// Quality score decrease threshold (e.g., 0.1 = 10% decrease)
    pub quality_score_threshold: Option<f64>,
}

impl Default for AgentRegressionThresholds {
    fn default() -> Self {
        Self {
            pass_rate_threshold_percent: 5.0,
            latency_p95_threshold_percent: 15.0,
            cost_threshold_percent: 10.0,
            tool_accuracy_threshold: 0.05,
            quality_score_threshold: Some(0.1),
        }
    }
}

/// A detected agent evaluation regression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegression {
    /// Metric name
    pub metric: String,

    /// Baseline value
    pub baseline_value: f64,

    /// Current value
    pub current_value: f64,

    /// Change percentage (positive = worse)
    pub change_percent: f64,

    /// Severity
    pub severity: RegressionSeverity,

    /// Description
    pub description: String,
}

/// Agent regression detection report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegressionReport {
    /// Baseline timestamp
    pub baseline_timestamp: String,

    /// Current run timestamp
    pub current_timestamp: String,

    /// Detected regressions
    pub regressions: Vec<AgentRegression>,

    /// Summary
    pub summary: AgentRegressionSummary,
}

impl AgentRegressionReport {
    /// Check if any regressions were detected
    pub fn has_regressions(&self) -> bool {
        !self.regressions.is_empty()
    }

    /// Get severe regressions only
    pub fn severe_regressions(&self) -> Vec<&AgentRegression> {
        self.regressions
            .iter()
            .filter(|r| matches!(r.severity, RegressionSeverity::Severe))
            .collect()
    }
}

/// Summary of regression analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegressionSummary {
    /// Total test cases compared
    pub total_cases: usize,

    /// Number of regressions detected
    pub regressions_found: usize,

    /// Whether any regression is severe
    pub has_severe_regressions: bool,
}

/// Agent regression detector
pub struct AgentRegressionDetector;

impl AgentRegressionDetector {
    /// Detect regressions by comparing current results against baseline
    pub fn detect_regressions(
        baseline: &BaselineSnapshot,
        current: &[AgentEvalResult],
        thresholds: &AgentRegressionThresholds,
    ) -> AgentRegressionReport {
        let mut regressions = Vec::new();

        // Extract baseline agent eval results
        let baseline_results = match baseline.agent_eval() {
            Some(results) => results,
            None => {
                return AgentRegressionReport {
                    baseline_timestamp: baseline.metadata.timestamp.clone(),
                    current_timestamp: Utc::now().to_rfc3339(),
                    regressions: Vec::new(),
                    summary: AgentRegressionSummary {
                        total_cases: current.len(),
                        regressions_found: 0,
                        has_severe_regressions: false,
                    },
                };
            }
        };

        // Calculate baseline metrics
        let baseline_pass_rate = baseline_results.iter().filter(|r| r.passed).count() as f64
            / baseline_results.len() as f64;

        let baseline_avg_latency = baseline_results
            .iter()
            .map(|r| r.latency.total_ms)
            .sum::<f64>()
            / baseline_results.len() as f64;

        let baseline_avg_cost = baseline_results
            .iter()
            .map(|r| r.cost.total_cost_usd)
            .sum::<f64>()
            / baseline_results.len() as f64;

        let baseline_avg_f1 = baseline_results
            .iter()
            .map(|r| r.tool_accuracy.f1_score)
            .sum::<f64>()
            / baseline_results.len() as f64;

        // Calculate current metrics
        let current_pass_rate =
            current.iter().filter(|r| r.passed).count() as f64 / current.len() as f64;

        let current_avg_latency =
            current.iter().map(|r| r.latency.total_ms).sum::<f64>() / current.len() as f64;

        let current_avg_cost =
            current.iter().map(|r| r.cost.total_cost_usd).sum::<f64>() / current.len() as f64;

        let current_avg_f1 = current
            .iter()
            .map(|r| r.tool_accuracy.f1_score)
            .sum::<f64>()
            / current.len() as f64;

        // Check pass rate regression
        let pass_rate_change =
            ((baseline_pass_rate - current_pass_rate) / baseline_pass_rate) * 100.0;
        if pass_rate_change > thresholds.pass_rate_threshold_percent {
            let severity = if pass_rate_change > 15.0 {
                RegressionSeverity::Severe
            } else if pass_rate_change > 10.0 {
                RegressionSeverity::Moderate
            } else {
                RegressionSeverity::Minor
            };

            regressions.push(AgentRegression {
                metric: "Pass Rate".to_string(),
                baseline_value: baseline_pass_rate * 100.0,
                current_value: current_pass_rate * 100.0,
                change_percent: -pass_rate_change,
                severity,
                description: format!(
                    "Pass rate decreased from {:.1}% to {:.1}%",
                    baseline_pass_rate * 100.0,
                    current_pass_rate * 100.0
                ),
            });
        }

        // Check latency regression (P95)
        // For simplicity, we use average latency here
        // In production, calculate P95 from latency_stats
        let latency_change =
            ((current_avg_latency - baseline_avg_latency) / baseline_avg_latency) * 100.0;
        if latency_change > thresholds.latency_p95_threshold_percent {
            let severity = if latency_change > 30.0 {
                RegressionSeverity::Severe
            } else if latency_change > 20.0 {
                RegressionSeverity::Moderate
            } else {
                RegressionSeverity::Minor
            };

            regressions.push(AgentRegression {
                metric: "Latency (avg)".to_string(),
                baseline_value: baseline_avg_latency,
                current_value: current_avg_latency,
                change_percent: latency_change,
                severity,
                description: format!(
                    "Average latency increased from {:.0}ms to {:.0}ms",
                    baseline_avg_latency, current_avg_latency
                ),
            });
        }

        // Check cost regression
        let cost_change = ((current_avg_cost - baseline_avg_cost) / baseline_avg_cost) * 100.0;
        if cost_change > thresholds.cost_threshold_percent {
            let severity = if cost_change > 25.0 {
                RegressionSeverity::Severe
            } else if cost_change > 15.0 {
                RegressionSeverity::Moderate
            } else {
                RegressionSeverity::Minor
            };

            regressions.push(AgentRegression {
                metric: "Cost".to_string(),
                baseline_value: baseline_avg_cost,
                current_value: current_avg_cost,
                change_percent: cost_change,
                severity,
                description: format!(
                    "Average cost increased from ${:.4} to ${:.4}",
                    baseline_avg_cost, current_avg_cost
                ),
            });
        }

        // Check tool accuracy regression
        let f1_change = baseline_avg_f1 - current_avg_f1;
        if f1_change > thresholds.tool_accuracy_threshold {
            let severity = if f1_change > 0.15 {
                RegressionSeverity::Severe
            } else if f1_change > 0.10 {
                RegressionSeverity::Moderate
            } else {
                RegressionSeverity::Minor
            };

            regressions.push(AgentRegression {
                metric: "Tool Accuracy (F1)".to_string(),
                baseline_value: baseline_avg_f1,
                current_value: current_avg_f1,
                change_percent: (f1_change / baseline_avg_f1) * 100.0,
                severity,
                description: format!(
                    "Tool F1 score decreased from {:.2} to {:.2}",
                    baseline_avg_f1, current_avg_f1
                ),
            });
        }

        let has_severe = regressions
            .iter()
            .any(|r| matches!(r.severity, RegressionSeverity::Severe));

        AgentRegressionReport {
            baseline_timestamp: baseline.metadata.timestamp.clone(),
            current_timestamp: Utc::now().to_rfc3339(),
            summary: AgentRegressionSummary {
                total_cases: current.len(),
                regressions_found: regressions.len(),
                has_severe_regressions: has_severe,
            },
            regressions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_eval::result::*;
    use crate::baseline::{BaselineContent, BaselineMetadata};
    use crate::benchmark::BenchmarkEnvironment;

    fn create_test_result(
        test_id: &str,
        passed: bool,
        latency_ms: f64,
        cost_usd: f64,
        f1: f64,
    ) -> AgentEvalResult {
        AgentEvalResult {
            test_case_id: test_id.to_string(),
            test_case_name: format!("Test {}", test_id),
            passed,
            actual_output: "output".to_string(),
            correctness: CorrectnessResult::passed(MatchType::Exact),
            tool_accuracy: ToolAccuracyResult {
                precision: f1,
                recall: f1,
                f1_score: f1,
                missing_tools: Vec::new(),
                unexpected_tools: Vec::new(),
            },
            quality_scores: None,
            latency: LatencyMetrics {
                total_ms: latency_ms,
                within_budget: true,
                budget_ms: None,
            },
            cost: CostMetrics {
                total_cost_usd: cost_usd,
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
                within_budget: true,
                budget_usd: None,
                model: "gpt-4o-mini".to_string(),
            },
            timestamp: Utc::now(),
            failure_reason: None,
        }
    }

    fn create_baseline(results: Vec<AgentEvalResult>) -> BaselineSnapshot {
        BaselineSnapshot {
            metadata: BaselineMetadata {
                version: "1.0".to_string(),
                timestamp: "2026-01-01T00:00:00Z".to_string(),
                git_metadata: None,
                environment: BenchmarkEnvironment::default(),
            },
            content: BaselineContent::AgentEval(results),
        }
    }

    #[test]
    fn test_no_regression() {
        let baseline = create_baseline(vec![
            create_test_result("001", true, 1000.0, 0.01, 0.9),
            create_test_result("002", true, 1100.0, 0.01, 0.9),
        ]);

        let current = vec![
            create_test_result("001", true, 1000.0, 0.01, 0.9),
            create_test_result("002", true, 1100.0, 0.01, 0.9),
        ];

        let thresholds = AgentRegressionThresholds::default();
        let report = AgentRegressionDetector::detect_regressions(&baseline, &current, &thresholds);

        assert!(!report.has_regressions());
        assert_eq!(report.summary.regressions_found, 0);
    }

    #[test]
    fn test_pass_rate_regression() {
        let baseline = create_baseline(vec![
            create_test_result("001", true, 1000.0, 0.01, 0.9),
            create_test_result("002", true, 1100.0, 0.01, 0.9),
        ]);

        // 50% pass rate (was 100%)
        let current = vec![
            create_test_result("001", true, 1000.0, 0.01, 0.9),
            create_test_result("002", false, 1100.0, 0.01, 0.9),
        ];

        let thresholds = AgentRegressionThresholds::default();
        let report = AgentRegressionDetector::detect_regressions(&baseline, &current, &thresholds);

        assert!(report.has_regressions());
        assert_eq!(report.summary.regressions_found, 1);
        assert_eq!(report.regressions[0].metric, "Pass Rate");
        assert!(report.summary.has_severe_regressions); // 50% decrease is severe
    }

    #[test]
    fn test_latency_regression() {
        let baseline = create_baseline(vec![
            create_test_result("001", true, 1000.0, 0.01, 0.9),
            create_test_result("002", true, 1000.0, 0.01, 0.9),
        ]);

        // 50% latency increase
        let current = vec![
            create_test_result("001", true, 1500.0, 0.01, 0.9),
            create_test_result("002", true, 1500.0, 0.01, 0.9),
        ];

        let thresholds = AgentRegressionThresholds::default();
        let report = AgentRegressionDetector::detect_regressions(&baseline, &current, &thresholds);

        assert!(report.has_regressions());
        let latency_reg = report
            .regressions
            .iter()
            .find(|r| r.metric == "Latency (avg)");
        assert!(latency_reg.is_some());
        assert!(latency_reg.unwrap().change_percent > 40.0);
    }

    #[test]
    fn test_cost_regression() {
        let baseline = create_baseline(vec![
            create_test_result("001", true, 1000.0, 0.01, 0.9),
            create_test_result("002", true, 1000.0, 0.01, 0.9),
        ]);

        // 50% cost increase
        let current = vec![
            create_test_result("001", true, 1000.0, 0.015, 0.9),
            create_test_result("002", true, 1000.0, 0.015, 0.9),
        ];

        let thresholds = AgentRegressionThresholds::default();
        let report = AgentRegressionDetector::detect_regressions(&baseline, &current, &thresholds);

        assert!(report.has_regressions());
        let cost_reg = report.regressions.iter().find(|r| r.metric == "Cost");
        assert!(cost_reg.is_some());
    }

    #[test]
    fn test_tool_accuracy_regression() {
        let baseline = create_baseline(vec![
            create_test_result("001", true, 1000.0, 0.01, 0.9),
            create_test_result("002", true, 1000.0, 0.01, 0.9),
        ]);

        // F1 dropped from 0.9 to 0.7
        let current = vec![
            create_test_result("001", true, 1000.0, 0.01, 0.7),
            create_test_result("002", true, 1000.0, 0.01, 0.7),
        ];

        let thresholds = AgentRegressionThresholds::default();
        let report = AgentRegressionDetector::detect_regressions(&baseline, &current, &thresholds);

        assert!(report.has_regressions());
        let f1_reg = report
            .regressions
            .iter()
            .find(|r| r.metric == "Tool Accuracy (F1)");
        assert!(f1_reg.is_some());
        assert!(f1_reg.unwrap().severity == RegressionSeverity::Severe);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/agent_eval/regression.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/agent_eval/regression.rs` captured during meter full-codegen standardization.
```
