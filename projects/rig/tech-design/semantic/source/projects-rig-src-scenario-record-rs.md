---
id: projects-rig-src-scenario-record-rs
capability_refs:
  - id: scenario-engine
    role: primary
    claim: scenario-step-dsl-execution
    coverage: partial
    rationale: "This source unit implements rig scenario discovery, execution, verdict, or report behavior used by the scenario engine."
fill_sections: [overview, source, changes]
---

# Standardized projects/rig/src/scenario/record.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/rig/src/scenario/record.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ExpectedOutcome` | projects/rig/src/scenario/record.rs | enum | pub | 29 |  |
| `LintViolation` | projects/rig/src/scenario/record.rs | struct | pub | 67 |  |
| `Record` | projects/rig/src/scenario/record.rs | struct | pub | 44 |  |
| `ScenarioKind` | projects/rig/src/scenario/record.rs | enum | pub | 17 |  |
| `lint_record` | projects/rig/src/scenario/record.rs | function | pub | 74 | lint_record(path: &Path, record: &Record) -> Vec<LintViolation> |
| `scenario_id` | projects/rig/src/scenario/record.rs | function | pub | 131 | scenario_id(record: &Record) -> String |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! The embedded `[record]` table — the machine-readable identity of a
//! scenario file. The record is the source of truth, not the path; the
//! path==record invariant (lint) keeps the tree a queryable database.
//!
//! Inherited from mamba's fixture-record contract
//! (`projects/mamba/tests/harness/cpython/conventions/FIXTURE-LAYOUT.md`).

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Scenario kind: which execution engine drives the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioKind {
    /// Step-DSL behavior scenario.
    E2e,
    /// Open-loop load profile (`[load]` block).
    Load,
}

/// Expected outcome bucket — drives verdict accounting, never silently
/// flips a failure into a pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpectedOutcome {
    /// Must pass; a failure is RED and gates the run.
    Pass,
    /// Known gap: expected to fail. Failure never gates; an unexpected
    /// PASS surfaces as a graduate-to-pass signal (xpass).
    Xfail,
    /// Structurally unrunnable here (missing service, platform). Skipped
    /// before execution, reported separately.
    Skip,
}

/// The `[record]` table. `dimension` must equal the scenario file's parent
/// directory name and `case` its file stem (enforced by [`lint_record`]).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    /// Owning suite/project, e.g. `lumen`.
    pub suite: String,
    /// Facet directory: `resilience`, `endurance`, `load`, `api`, ...
    pub dimension: String,
    /// snake_case case name; MUST equal the filename stem.
    pub case: String,
    /// Human one-liner: the behavior under test.
    pub subject: String,
    pub kind: ScenarioKind,
    pub expected: ExpectedOutcome,
    /// Required scenarios gate the run; optional ones report only.
    #[serde(default = "default_required")]
    pub required: bool,
}

fn default_required() -> bool {
    true
}

/// One lint violation, ready to become a `lint_error` finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintViolation {
    pub message: String,
}

/// Enforce the path==record invariant and field sanity for a scenario file
/// at `path` whose parsed record is `record`.
pub fn lint_record(path: &Path, record: &Record) -> Vec<LintViolation> {
    let mut violations = Vec::new();
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    let parent = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or_default();

    if record.case != stem {
        violations.push(LintViolation {
            message: format!(
                "record.case `{}` != filename stem `{stem}` — the filename is the case key",
                record.case
            ),
        });
    }
    if record.dimension != parent {
        violations.push(LintViolation {
            message: format!(
                "record.dimension `{}` != parent directory `{parent}` — the directory is the taxonomy",
                record.dimension
            ),
        });
    }
    if record.case.is_empty() || !record.case.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
        violations.push(LintViolation {
            message: format!(
                "record.case `{}` must be non-empty snake_case ([a-z0-9_])",
                record.case
            ),
        });
    }
    if record.subject.trim().is_empty() {
        violations.push(LintViolation {
            message: "record.subject must name the behavior under test".to_string(),
        });
    }
    if record.suite.trim().is_empty() {
        violations.push(LintViolation {
            message: "record.suite must name the owning suite/project".to_string(),
        });
    }
    violations
}

/// `suite/dimension/case` — the stable scenario id used in reports, pins,
/// and baselines.
pub fn scenario_id(record: &Record) -> String {
    format!("{}/{}/{}", record.suite, record.dimension, record.case)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn record() -> Record {
        Record {
            suite: "lumen".into(),
            dimension: "resilience".into(),
            case: "partition_recovery".into(),
            subject: "lumen under toxiproxy partition".into(),
            kind: ScenarioKind::E2e,
            expected: ExpectedOutcome::Pass,
            required: true,
        }
    }

    #[test]
    fn matching_path_lints_clean() {
        let p = PathBuf::from("scenarios/resilience/partition_recovery.toml");
        assert!(lint_record(&p, &record()).is_empty());
    }

    #[test]
    fn stem_mismatch_is_violation() {
        let p = PathBuf::from("scenarios/resilience/other_name.toml");
        let v = lint_record(&p, &record());
        assert!(v.iter().any(|v| v.message.contains("filename stem")));
    }

    #[test]
    fn dimension_mismatch_is_violation() {
        let p = PathBuf::from("scenarios/load/partition_recovery.toml");
        let v = lint_record(&p, &record());
        assert!(v.iter().any(|v| v.message.contains("parent directory")));
    }

    #[test]
    fn non_snake_case_rejected() {
        let mut r = record();
        r.case = "Partition-Recovery".into();
        let p = PathBuf::from("scenarios/resilience/Partition-Recovery.toml");
        let v = lint_record(&p, &r);
        assert!(v.iter().any(|v| v.message.contains("snake_case")));
    }

    #[test]
    fn scenario_id_is_suite_dimension_case() {
        assert_eq!(scenario_id(&record()), "lumen/resilience/partition_recovery");
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/rig/src/scenario/record.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/rig/src/scenario/record.rs` captured during rig
      standardization onto the codegen ladder.
```
