---
id: projects-rig-src-report-finding-rs
capability_refs:
  - id: scenario-engine
    role: primary
    claim: scenario-step-dsl-execution
    coverage: partial
    rationale: "This source unit implements rig scenario discovery, execution, verdict, or report behavior used by the scenario engine."
fill_sections: [overview, source, changes]
---

# Standardized projects/rig/src/report/finding.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/rig/src/report/finding.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Finding` | projects/rig/src/report/finding.rs | struct | pub | 108 |  |
| `Invoke` | projects/rig/src/report/finding.rs | struct | pub | 91 |  |
| `Kind` | projects/rig/src/report/finding.rs | enum | pub | 53 |  |
| `Severity` | projects/rig/src/report/finding.rs | enum | pub | 17 |  |
| `as_str` | projects/rig/src/report/finding.rs | function | pub | 27 | as_str(&self) -> &'static str |
| `as_str` | projects/rig/src/report/finding.rs | function | pub | 74 | as_str(&self) -> &'static str |
| `command` | projects/rig/src/report/finding.rs | function | pub | 97 | command(cmd: impl Into<String>) -> Self |
| `finding_id` | projects/rig/src/report/finding.rs | function | pub | 122 | finding_id(kind: Kind, subject: &str) -> String |
| `rank` | projects/rig/src/report/finding.rs | function | pub | 38 | rank(&self) -> u8 |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Finding model — one actionable observation inside a [`RigReport`].
//!
//! Mirrors the meter finding contract (severity / kind / evidence /
//! remediation / invoke) so agents read both tools the same way, without a
//! crate dependency between them.
//!
//! [`RigReport`]: crate::report::envelope::RigReport

use serde::{Deserialize, Serialize};

/// Severity bucket. Sorted critical -> info when ordering findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Critical => "critical",
            Severity::High => "high",
            Severity::Medium => "medium",
            Severity::Low => "low",
            Severity::Info => "info",
        }
    }

    /// Rank for worst-wins ordering: lower = more severe.
    pub fn rank(&self) -> u8 {
        match self {
            Severity::Critical => 0,
            Severity::High => 1,
            Severity::Medium => 2,
            Severity::Low => 3,
            Severity::Info => 4,
        }
    }
}

/// The closed kind set rig emits. Every kind maps to one producing stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// A scenario step failed (http non-expect, exec non-zero, wait_until budget).
    StepFailure,
    /// An `assert` step expression evaluated false.
    AssertionFailure,
    /// A pin gate breached its floor or ratchet — maps to exit 2.
    PinRegression,
    /// A pin had no recorded baseline (Info; failure under `RIG_STRICT=1`).
    PinMissingBaseline,
    /// A scenario file violated the record contract (path==record, keys).
    LintError,
    /// The scenario exceeded its `[limits] timeout_secs` budget.
    Timeout,
    /// Scenario-level execution problem (bad interpolation, missing var, vat mismatch).
    ScenarioError,
    /// Load-mode honesty warning: achieved_qps fell below the offered target.
    LoadHonesty,
}

impl Kind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Kind::StepFailure => "step_failure",
            Kind::AssertionFailure => "assertion_failure",
            Kind::PinRegression => "pin_regression",
            Kind::PinMissingBaseline => "pin_missing_baseline",
            Kind::LintError => "lint_error",
            Kind::Timeout => "timeout",
            Kind::ScenarioError => "scenario_error",
            Kind::LoadHonesty => "load_honesty",
        }
    }
}

/// The smallest next command an agent can run to act on a finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoke {
    pub command: String,
}

impl Invoke {
    pub fn command(cmd: impl Into<String>) -> Self {
        Self {
            command: cmd.into(),
        }
    }
}

/// One actionable observation. `evidence` carries the kind-specific payload
/// (captured vars, expected/actual, metric values).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub severity: Severity,
    pub kind: Kind,
    pub title: String,
    pub detail: String,
    pub remediation: String,
    pub invoke: Invoke,
    pub evidence: serde_json::Value,
}

/// Stable finding id: `<kind>:<subject>` with the subject squashed to a
/// filesystem/grep-friendly token.
pub fn finding_id(kind: Kind, subject: &str) -> String {
    let squashed: String = subject
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '/' || c == '_' || c == '-' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("{}:{}", kind.as_str(), squashed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_ranks_are_worst_first() {
        assert!(Severity::Critical.rank() < Severity::Info.rank());
        assert!(Severity::Medium.rank() < Severity::Low.rank());
    }

    #[test]
    fn finding_id_squashes_subject() {
        assert_eq!(
            finding_id(Kind::StepFailure, "resilience/partition recovery"),
            "step_failure:resilience/partition_recovery"
        );
    }

    #[test]
    fn kind_serializes_snake_case() {
        let j = serde_json::to_value(Kind::PinRegression).unwrap();
        assert_eq!(j, "pin_regression");
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/rig/src/report/finding.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/rig/src/report/finding.rs` captured during rig
      standardization onto the codegen ladder.
```
