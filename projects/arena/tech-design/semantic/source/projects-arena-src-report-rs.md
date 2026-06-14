---
id: projects-arena-src-report-rs
capability_refs:
  - id: n-target-comparison-runner
    role: primary
    claim: arena-report-envelope
    coverage: partial
    rationale: "This source unit implements arena.report/1 output for the N-target comparison runner."
fill_sections: [overview, source, changes]
---

# Standardized projects/arena/src/report.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/arena/src/report.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ArenaReport` | projects/arena/src/report.rs | struct | pub | 52 |  |
| `ComparisonRow` | projects/arena/src/report.rs | struct | pub | 40 |  |
| `PeerCell` | projects/arena/src/report.rs | struct | pub | 19 |  |
| `SCHEMA_VERSION` | projects/arena/src/report.rs | constant | pub | 14 |  |
| `exit_code` | projects/arena/src/report.rs | function | pub | 81 | exit_code(&self) -> i32 |
| `persist` | projects/arena/src/report.rs | function | pub | 86 | persist(&self, dir: &std::path::Path) |
| `stub` | projects/arena/src/report.rs | function | pub | 74 | stub(verb: &str, prompt: &str) -> Self |
| `tool_error` | projects/arena/src/report.rs | function | pub | 67 | tool_error(code: u8, message: impl Into<String>) -> Self |
| `wrap` | projects/arena/src/report.rs | function | pub | 61 | wrap(mut base: RigReport, comparison: Vec<ComparisonRow>) -> Self |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! `arena.report/1` — rig's report envelope (worst-wins fold + exit ladder)
//! plus a typed comparison table.
//!
//! arena reuses rig's [`ReportBuilder`] verbatim for findings/status/exit, then
//! wraps the finalized [`RigReport`] (re-namespaced to `arena.report/1`) with a
//! `comparison` grid the rig envelope does not carry.

use serde::{Deserialize, Serialize};

use rig::report::{ReportBuilder, RigReport};

pub const SCHEMA_VERSION: &str = "arena.report/1";

/// One peer's outcome within a cell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerCell {
    pub target: String,
    /// The peer's measured scalar (same metric/unit as the base).
    pub value: f64,
    /// `peer / base` — `> 1` means the base target wins.
    pub ratio: f64,
    /// `win` | `exempt` | `target`.
    pub gate: String,
    /// Human verdict label, e.g. `WIN ok>=14.6` / `WIN<14.6` / `exempt`.
    pub verdict: String,
    /// The ratcheted baseline ratio used to gate a WIN cell, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline: Option<f64>,
    /// `false` when the measurement was untrustworthy (load saturated / target
    /// unreachable) — the ratio should not be acted on.
    pub trustworthy: bool,
}

/// One cell's base value plus every peer's comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonRow {
    pub cell: String,
    pub metric: String,
    pub base_target: String,
    pub base_value: f64,
    pub peers: Vec<PeerCell>,
}

/// The single document `arena run` prints to stdout: rig's envelope flattened
/// at the top level, plus the comparison grid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArenaReport {
    #[serde(flatten)]
    pub base: RigReport,
    pub comparison: Vec<ComparisonRow>,
}

impl ArenaReport {
    /// Wrap a finalized rig report, re-namespacing the schema to arena's.
    pub fn wrap(mut base: RigReport, comparison: Vec<ComparisonRow>) -> Self {
        base.schema_version = SCHEMA_VERSION.to_string();
        Self { base, comparison }
    }

    /// A tool-error report (no comparison) — usage/missing-tool/io failures.
    pub fn tool_error(code: u8, message: impl Into<String>) -> Self {
        let mut b = ReportBuilder::new("run", "-");
        b.tool_error(code, message);
        Self::wrap(b.finalize(), Vec::new())
    }

    /// An offline self-describer report (spec/llm stubs) carrying only a prompt.
    pub fn stub(verb: &str, prompt: &str) -> Self {
        let mut b = ReportBuilder::new(verb, "-");
        b.agent_prompt(prompt);
        Self::wrap(b.finalize(), Vec::new())
    }

    /// The process exit code (rig's worst-wins ladder: 0/1/2/3/4/5).
    pub fn exit_code(&self) -> i32 {
        self.base.exit_code
    }

    /// Persist to `<dir>/.arena/last-report.json` (best-effort).
    pub fn persist(&self, dir: &std::path::Path) {
        let adir = dir.join(".arena");
        if std::fs::create_dir_all(&adir).is_err() {
            return;
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(adir.join("last-report.json"), json);
        }
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/arena/src/report.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/arena/src/report.rs` captured during arena
      standardization onto the codegen ladder.
```
