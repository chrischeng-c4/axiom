// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
// CODEGEN-BEGIN
//! Coverage summary fields and threshold evaluation for `jet test` (#2714).
//!
//! Atomic slice scope: define the data shape that downstream coverage
//! producers will serialize into the result envelope, and the threshold
//! evaluator that decides whether a coverage run should fail CI. The
//! actual instrumentation engine is **out of scope** here — this module
//! is fed by fixture data today and by a real producer later.
//!
//! @spec #2714

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// One metric family in a coverage summary — line, function, or branch.
///
/// `covered` and `total` are absolute counts. `pct()` derives the
/// percentage so the on-disk representation can stay
/// integer-truth — callers that want the percentage compute it on demand.
/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageMetric {
    pub covered: u32,
    pub total: u32,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
impl CoverageMetric {
    pub fn new(covered: u32, total: u32) -> Self {
        debug_assert!(covered <= total, "covered ({covered}) > total ({total})");
        Self { covered, total }
    }

    /// Percentage covered as a `0.0..=100.0` float. An empty metric
    /// (`total == 0`) reports 100% — there is nothing to miss.
    pub fn pct(&self) -> f64 {
        if self.total == 0 {
            100.0
        } else {
            (self.covered as f64) * 100.0 / (self.total as f64)
        }
    }
}

/// Aggregated coverage summary attached to a `jet test` run.
///
/// `artifact` points at the on-disk coverage report (e.g. `lcov.info`,
/// `coverage-final.json`) so the unified result envelope can deep-link
/// human and agent consumers without inlining the full report.
/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageSummary {
    pub lines: CoverageMetric,
    pub functions: CoverageMetric,
    pub branches: CoverageMetric,
    /// Absolute path to the coverage artifact this summary was derived
    /// from. `None` when coverage was disabled or no artifact was
    /// produced.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact: Option<PathBuf>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
impl CoverageSummary {
    /// Evaluate the summary against the given thresholds. Returns `Ok(())`
    /// when every configured threshold is met, or a non-empty list of
    /// failures identifying *which* metric fell short and by how much.
    pub fn evaluate(
        &self,
        thresholds: &CoverageThresholds,
    ) -> Result<(), Vec<CoverageThresholdFailure>> {
        let mut failures = Vec::new();
        Self::check(
            CoverageMetricKind::Lines,
            self.lines,
            thresholds.lines,
            &mut failures,
        );
        Self::check(
            CoverageMetricKind::Functions,
            self.functions,
            thresholds.functions,
            &mut failures,
        );
        Self::check(
            CoverageMetricKind::Branches,
            self.branches,
            thresholds.branches,
            &mut failures,
        );
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }

    fn check(
        kind: CoverageMetricKind,
        metric: CoverageMetric,
        threshold: Option<f64>,
        failures: &mut Vec<CoverageThresholdFailure>,
    ) {
        let Some(required) = threshold else {
            return;
        };
        let observed = metric.pct();
        if observed + f64::EPSILON < required {
            failures.push(CoverageThresholdFailure {
                metric: kind,
                required,
                observed,
            });
        }
    }
}

/// CI-facing threshold configuration. A `None` value means "do not gate
/// on this metric" — disabling coverage entirely is expressed by leaving
/// every field `None` and not attaching a `CoverageSummary` to the run.
/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct CoverageThresholds {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lines: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub functions: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branches: Option<f64>,
}

/// Which family of metric a threshold failure refers to.
/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageMetricKind {
    Lines,
    Functions,
    Branches,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
impl CoverageMetricKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lines => "lines",
            Self::Functions => "functions",
            Self::Branches => "branches",
        }
    }
}

/// One missed coverage threshold. Carries the metric kind, the required
/// percentage, and what we actually observed — enough for the runner to
/// print a deterministic, agent-readable failure line such as
/// `coverage: lines 71.42% < required 80.00%`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CoverageThresholdFailure {
    pub metric: CoverageMetricKind,
    pub required: f64,
    pub observed: f64,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
impl CoverageThresholdFailure {
    /// Human/agent-readable one-liner. Stable format so CI scrapers and
    /// the JSON reporter can both rely on it.
    pub fn display_line(&self) -> String {
        format!(
            "coverage: {} {:.2}% < required {:.2}%",
            self.metric.as_str(),
            self.observed,
            self.required,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_summary() -> CoverageSummary {
        // Hand-rolled fixture: 5/7 lines (71.43%), 2/3 functions (66.67%),
        // 3/4 branches (75.00%).
        CoverageSummary {
            lines: CoverageMetric::new(5, 7),
            functions: CoverageMetric::new(2, 3),
            branches: CoverageMetric::new(3, 4),
            artifact: Some(PathBuf::from("/tmp/coverage/lcov.info")),
        }
    }

    #[test]
    fn metric_pct_is_one_hundred_when_total_zero() {
        let m = CoverageMetric::new(0, 0);
        assert_eq!(m.pct(), 100.0);
    }

    #[test]
    fn metric_pct_rounds_through_float() {
        let m = CoverageMetric::new(1, 3);
        // 33.333... — assert close enough rather than exact equality.
        assert!((m.pct() - 33.333_333_333).abs() < 1e-6);
    }

    #[test]
    fn evaluate_passes_when_no_thresholds_configured() {
        let s = fixture_summary();
        assert!(s.evaluate(&CoverageThresholds::default()).is_ok());
    }

    #[test]
    fn evaluate_passes_when_all_metrics_meet_thresholds() {
        let s = fixture_summary();
        let t = CoverageThresholds {
            lines: Some(50.0),
            functions: Some(60.0),
            branches: Some(70.0),
        };
        assert!(s.evaluate(&t).is_ok());
    }

    #[test]
    fn evaluate_collects_every_failing_metric() {
        let s = fixture_summary();
        let t = CoverageThresholds {
            lines: Some(80.0),
            functions: Some(80.0),
            branches: Some(80.0),
        };
        let failures = s.evaluate(&t).expect_err("all three should fail");
        assert_eq!(failures.len(), 3);
        let kinds: Vec<_> = failures.iter().map(|f| f.metric).collect();
        assert!(kinds.contains(&CoverageMetricKind::Lines));
        assert!(kinds.contains(&CoverageMetricKind::Functions));
        assert!(kinds.contains(&CoverageMetricKind::Branches));
    }

    #[test]
    fn evaluate_reports_only_the_metric_that_fails() {
        let s = fixture_summary();
        // Lines pct ~71.43%, branches pct 75%, functions pct ~66.67%.
        // Setting only `branches` to 90 must surface branches and only branches.
        let t = CoverageThresholds {
            lines: None,
            functions: None,
            branches: Some(90.0),
        };
        let failures = s.evaluate(&t).expect_err("branches should fail");
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].metric, CoverageMetricKind::Branches);
        assert!((failures[0].observed - 75.0).abs() < 1e-9);
        assert_eq!(failures[0].required, 90.0);
    }

    #[test]
    fn display_line_is_stable_and_identifies_metric() {
        let f = CoverageThresholdFailure {
            metric: CoverageMetricKind::Lines,
            required: 80.0,
            observed: 71.428_571,
        };
        assert_eq!(f.display_line(), "coverage: lines 71.43% < required 80.00%");
    }

    #[test]
    fn summary_round_trips_through_json() {
        let s = fixture_summary();
        let json = serde_json::to_string(&s).unwrap();
        let back: CoverageSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn empty_summary_with_zero_totals_meets_any_threshold() {
        // Empty file → no lines, no branches, no functions. Empty
        // coverage means nothing to miss, so any threshold passes.
        let s = CoverageSummary {
            lines: CoverageMetric::new(0, 0),
            functions: CoverageMetric::new(0, 0),
            branches: CoverageMetric::new(0, 0),
            artifact: None,
        };
        let t = CoverageThresholds {
            lines: Some(100.0),
            functions: Some(100.0),
            branches: Some(100.0),
        };
        assert!(s.evaluate(&t).is_ok());
    }
}
// CODEGEN-END
