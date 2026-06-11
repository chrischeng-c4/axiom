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
