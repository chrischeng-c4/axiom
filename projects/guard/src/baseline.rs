// SPEC-MANAGED: projects/guard/tech-design/semantic/source/projects-guard-src-baseline-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Security baseline store: `.guard/baseline.json`.
//!
//! `guard accept` snapshots the current scan's finding ids here; a later
//! `guard scan` gates only on findings ABSENT from the baseline, so known and
//! accepted findings stop turning the gate red. Keyed by the stable,
//! host-agnostic `finding_id` (`compass:<rule>:<path>:<line>`), mirroring rig's
//! `.rig/baselines.json` precedent.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;

use crate::report::GuardReport;

pub const BASELINE_VERSION: &str = "guard.baseline/1";

#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/guard/tech-design/semantic/source/projects-guard-src-baseline-rs.md#source
pub struct Baseline {
    pub schema_version: String,
    /// Accepted finding ids; their presence suppresses the gate for that id.
    #[serde(default)]
    pub findings: BTreeSet<String>,
}

/// @spec projects/guard/tech-design/semantic/source/projects-guard-src-baseline-rs.md#source
impl Default for Baseline {
    fn default() -> Self {
        Self {
            schema_version: BASELINE_VERSION.to_string(),
            findings: BTreeSet::new(),
        }
    }
}

/// @spec projects/guard/tech-design/semantic/source/projects-guard-src-baseline-rs.md#source
impl Baseline {
    /// Load from `<dir>/.guard/baseline.json` (absent file = empty baseline).
    pub fn load(dir: &Path) -> Self {
        match std::fs::read_to_string(dir.join(".guard").join("baseline.json")) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Snapshot every finding id in `report` as the accepted baseline.
    pub fn from_report(report: &GuardReport) -> Self {
        Self {
            schema_version: BASELINE_VERSION.to_string(),
            findings: report.findings.iter().map(|f| f.id.clone()).collect(),
        }
    }

    pub fn contains(&self, id: &str) -> bool {
        self.findings.contains(id)
    }

    pub fn len(&self) -> usize {
        self.findings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.findings.is_empty()
    }

    /// Persist to `<dir>/.guard/baseline.json`.
    pub fn save(&self, dir: &Path) -> std::io::Result<()> {
        let report_dir = dir.join(".guard");
        std::fs::create_dir_all(&report_dir)?;
        std::fs::write(
            report_dir.join("baseline.json"),
            serde_json::to_string_pretty(self)?,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_baseline_is_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let baseline = Baseline::load(tmp.path());
        assert!(baseline.is_empty());
        assert!(!baseline.contains("compass:JS004:x-js-1"));
    }

    #[test]
    fn roundtrips_through_disk() {
        let tmp = tempfile::tempdir().unwrap();
        let mut baseline = Baseline::default();
        baseline.findings.insert("compass:JS004:a-js-1".to_string());
        baseline.save(tmp.path()).unwrap();

        let reloaded = Baseline::load(tmp.path());
        assert_eq!(reloaded.len(), 1);
        assert!(reloaded.contains("compass:JS004:a-js-1"));
        assert_eq!(reloaded.schema_version, BASELINE_VERSION);
    }
}
// CODEGEN-END
