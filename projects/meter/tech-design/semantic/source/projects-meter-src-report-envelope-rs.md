---
id: projects-meter-src-report-envelope-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: json-default-report-envelope-and-findings
    claim: json-default-report-envelope-and-findings
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/src/report/envelope.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/report/envelope.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Completion` | projects/meter/src/report/envelope.rs | struct | pub | 112 |  |
| `EnvBlock` | projects/meter/src/report/envelope.rs | struct | pub | 141 |  |
| `FindingsSummary` | projects/meter/src/report/envelope.rs | struct | pub | 93 |  |
| `MeterReport` | projects/meter/src/report/envelope.rs | struct | pub | 21 |  |
| `OverallStatus` | projects/meter/src/report/envelope.rs | enum | pub | 60 |  |
| `RunnerRecord` | projects/meter/src/report/envelope.rs | struct | pub | 121 |  |
| `SCHEMA_VERSION` | projects/meter/src/report/envelope.rs | constant | pub | 16 |  |
| `exit_code` | projects/meter/src/report/envelope.rs | function | pub | 74 | exit_code(&self) -> i32 |
| `is_clean` | projects/meter/src/report/envelope.rs | function | pub | 85 | is_clean(&self) -> bool |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Report envelope schema — [`MeterReport`], the ONE self-describing document every
//! `meter` verb prints to stdout.
//!
//! All structs `derive(Serialize, Deserialize)`; field order = declaration order
//! = byte-stable JSON. The ONLY nondeterministic fields are timestamps /
//! durations / ns; golden gates never assert on those.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::finding::Finding;

/// Namespaced schema version string; agents pin on this.
pub const SCHEMA_VERSION: &str = "meter.report/1";

/// The one self-describing document every verb prints to stdout.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-envelope-rs.md#source
pub struct MeterReport {
    /// `= SCHEMA_VERSION`.
    pub schema_version: String,
    /// `env!("CARGO_PKG_VERSION")` of the `meter` crate.
    pub tool_version: String,
    /// `report|profile|bench|test|run|spec|llm`.
    pub verb: String,
    /// Absolute crate path / symbol / dir the verb operated on.
    pub target: String,
    /// SOLE source of `exit_code`/`clean`/`terminal`.
    pub status: OverallStatus,
    /// Mirror: `status == Clean`.
    pub clean: bool,
    /// The process's actual exit code, mirrored for `jq`.
    pub exit_code: i32,
    /// `true` => the agent may stop.
    pub terminal: bool,
    /// Delegated child runner record + forwarded exit, when a child ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_run: Option<RunnerRecord>,
    /// Tally + sample of findings.
    pub summary: FindingsSummary,
    /// FULL, pre-sorted findings (severity desc, id asc; Hotspots self_ns desc).
    pub findings: Vec<Finding>,
    /// Surfaced in EVERY report.
    pub environment: EnvBlock,
    /// `{clean, criteria, missing}`.
    pub completion: Completion,
    /// Machine next-action prose for the agent.
    pub agent_prompt: String,
    /// `true` => a human must intervene.
    #[serde(default)]
    pub requires_hitl: bool,
}

/// SOLE source of the process exit code, clean flag, and terminal flag.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-envelope-rs.md#source
pub enum OverallStatus {
    /// exit 0, terminal.
    Clean,
    /// exit 1, terminal.
    Findings { count: usize },
    /// exit 2, terminal.
    Regression { count: usize },
    /// exit >2 (3=usage, 4=missing-tool, 5=io).
    ToolError { code: i32, message: String },
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-envelope-rs.md#source
impl OverallStatus {
    /// The process exit code this status maps to.
    pub fn exit_code(&self) -> i32 {
        use OverallStatus::*;
        match self {
            Clean => 0,
            Findings { .. } => 1,
            Regression { .. } => 2,
            ToolError { code, .. } => (*code).max(3),
        }
    }

    /// `true` iff this is the clean status.
    pub fn is_clean(&self) -> bool {
        matches!(self, OverallStatus::Clean)
    }
}

/// Tally of findings by severity plus a bounded sample.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-envelope-rs.md#source
pub struct FindingsSummary {
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub info: usize,
    pub total: usize,
    /// `true` when `findings` was truncated and a `payload_path` was written.
    pub truncated: bool,
    /// First N findings (default 20).
    pub sample: Vec<Finding>,
    /// Path to the full findings payload, written only when truncated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_path: Option<String>,
}

/// Completion contract: whether the verb's success criteria are met.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-envelope-rs.md#source
pub struct Completion {
    pub clean: bool,
    pub criteria: Vec<String>,
    pub missing: Vec<String>,
}

/// Record of a delegated child runner (cargo test / nextest / sampler).
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-envelope-rs.md#source
pub struct RunnerRecord {
    /// argv exactly as invoked.
    pub command: Vec<String>,
    /// `cargo-test|nextest|sampler`.
    pub kind: String,
    pub started_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<DateTime<Utc>>,
    /// FORWARDED child exit code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(default)]
    pub delegated: bool,
}

/// Side-effect-free environment block, surfaced in EVERY report.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-envelope-rs.md#source
pub struct EnvBlock {
    pub os: String,
    pub arch: String,
    /// `cargo nextest --version` probe.
    pub nextest_present: bool,
    /// `macos-sample|linux-perf|none`.
    pub sampler_backend: String,
    pub rustc_version: Option<String>,
    /// Literal environment note or remediation.
    pub note: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overall_status_exit_codes() {
        assert_eq!(OverallStatus::Clean.exit_code(), 0);
        assert_eq!(OverallStatus::Findings { count: 3 }.exit_code(), 1);
        assert_eq!(OverallStatus::Regression { count: 1 }.exit_code(), 2);
        assert_eq!(
            OverallStatus::ToolError {
                code: 4,
                message: "x".into()
            }
            .exit_code(),
            4
        );
        // ToolError code is clamped up to a minimum of 3.
        assert_eq!(
            OverallStatus::ToolError {
                code: 1,
                message: "x".into()
            }
            .exit_code(),
            3
        );
    }

    #[test]
    fn overall_status_is_clean() {
        assert!(OverallStatus::Clean.is_clean());
        assert!(!OverallStatus::Findings { count: 1 }.is_clean());
    }

    #[test]
    fn overall_status_tags_state() {
        let j = serde_json::to_value(OverallStatus::Findings { count: 2 }).unwrap();
        assert_eq!(j["state"], "findings");
        assert_eq!(j["count"], 2);
        let j2 = serde_json::to_value(OverallStatus::Clean).unwrap();
        assert_eq!(j2["state"], "clean");
    }

    #[test]
    fn schema_version_is_namespaced() {
        assert_eq!(SCHEMA_VERSION, "meter.report/1");
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/report/envelope.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/report/envelope.rs` captured during meter full-codegen standardization.
```
