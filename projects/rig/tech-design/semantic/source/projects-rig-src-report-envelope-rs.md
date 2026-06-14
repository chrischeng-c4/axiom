---
id: projects-rig-src-report-envelope-rs
fill_sections: [overview, source, changes]
---

# Standardized projects/rig/src/report/envelope.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/rig/src/report/envelope.rs`, captured as a rust-source-unit (td_ast) item-tree
during rig standardization onto the codegen ladder.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! The one JSON document every rig verb prints to stdout.
//!
//! Field names and the exit-code ladder mirror meter's report contract
//! (`meter.report/1`) so agents parse both tools identically; rig owns its
//! own copy to keep the tools decoupled.
//!
//! Exit contract: 0 clean · 1 findings · 2 regression · 3 usage ·
//! 4 missing-tool · 5 io.

use serde::{Deserialize, Serialize};

use super::finding::Finding;

pub const SCHEMA_VERSION: &str = "rig.report/1";

/// Worst-wins overall status. `ToolError` carries its specific exit code
/// (3 usage / 4 missing tool / 5 io).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum OverallStatus {
    Clean,
    Findings,
    Regression,
    ToolError { code: u8 },
}

impl OverallStatus {
    pub fn exit_code(&self) -> i32 {
        match self {
            OverallStatus::Clean => 0,
            OverallStatus::Findings => 1,
            OverallStatus::Regression => 2,
            OverallStatus::ToolError { code } => *code as i32,
        }
    }

    pub fn is_clean(&self) -> bool {
        matches!(self, OverallStatus::Clean)
    }

    /// Worst-wins rank: higher = worse. ToolError outranks everything.
    pub fn rank(&self) -> u8 {
        match self {
            OverallStatus::Clean => 0,
            OverallStatus::Findings => 1,
            OverallStatus::Regression => 2,
            OverallStatus::ToolError { .. } => 3,
        }
    }
}

/// Severity histogram plus a short id sample for cheap triage.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Summary {
    pub critical: u32,
    pub high: u32,
    pub medium: u32,
    pub low: u32,
    pub info: u32,
    pub total: u32,
    pub truncated: bool,
    pub sample: Vec<String>,
}

/// Per-scenario verdict roll-up (pass/xfail/skip bucketing).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScenarioCounts {
    pub pass: u32,
    pub red: u32,
    /// Expected-fail scenarios that failed (never gate).
    pub xfail: u32,
    /// Expected-fail scenarios that PASSED — graduate-to-pass signal.
    pub xpass: u32,
    pub skip: u32,
}

/// What the run promised vs what it did not cover.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Completion {
    pub clean: bool,
    pub criteria: Vec<String>,
    pub missing: Vec<String>,
}

/// Host/environment stamp for reproducibility.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnvBlock {
    pub os: String,
    pub arch: String,
    pub tool_version: String,
}

impl EnvBlock {
    pub fn current() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// The single report document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigReport {
    pub schema_version: String,
    pub tool_version: String,
    pub verb: String,
    pub target: String,
    pub status: OverallStatus,
    pub clean: bool,
    pub exit_code: i32,
    pub summary: Summary,
    pub scenarios: ScenarioCounts,
    pub findings: Vec<Finding>,
    pub environment: EnvBlock,
    pub completion: Completion,
    pub agent_prompt: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_codes_follow_the_ladder() {
        assert_eq!(OverallStatus::Clean.exit_code(), 0);
        assert_eq!(OverallStatus::Findings.exit_code(), 1);
        assert_eq!(OverallStatus::Regression.exit_code(), 2);
        assert_eq!(OverallStatus::ToolError { code: 3 }.exit_code(), 3);
        assert_eq!(OverallStatus::ToolError { code: 5 }.exit_code(), 5);
    }

    #[test]
    fn status_serializes_tagged_state() {
        let j = serde_json::to_value(OverallStatus::Clean).unwrap();
        assert_eq!(j["state"], "clean");
        let j = serde_json::to_value(OverallStatus::ToolError { code: 4 }).unwrap();
        assert_eq!(j["state"], "tool_error");
        assert_eq!(j["code"], 4);
    }

    #[test]
    fn worst_wins_rank_orders() {
        assert!(OverallStatus::ToolError { code: 3 }.rank() > OverallStatus::Regression.rank());
        assert!(OverallStatus::Regression.rank() > OverallStatus::Findings.rank());
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/rig/src/report/envelope.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/rig/src/report/envelope.rs` captured during rig
      standardization onto the codegen ladder.
```
