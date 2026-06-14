---
id: projects-rig-src-scenario-mod-rs
capability_refs:
  - id: scenario-engine
    role: primary
    claim: scenario-step-dsl-execution
    coverage: partial
    rationale: "This source unit implements rig scenario discovery, execution, verdict, or report behavior used by the scenario engine."
fill_sections: [overview, source, changes]
---

# Standardized projects/rig/src/scenario/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/rig/src/scenario/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Limits` | projects/rig/src/scenario/mod.rs | struct | pub | 35 |  |
| `Scenario` | projects/rig/src/scenario/mod.rs | struct | pub | 56 |  |
| `VatNeeds` | projects/rig/src/scenario/mod.rs | struct | pub | 24 |  |
| `interp` | projects/rig/src/scenario/mod.rs | module | pub | 6 |  |
| `load` | projects/rig/src/scenario/mod.rs | module | pub | 7 |  |
| `parse_scenario` | projects/rig/src/scenario/mod.rs | function | pub | 73 | parse_scenario(path: &Path, text: &str) -> Result<Scenario, Vec<LintViolation>> |
| `record` | projects/rig/src/scenario/mod.rs | module | pub | 8 |  |
| `step` | projects/rig/src/scenario/mod.rs | module | pub | 9 |  |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Scenario model: one TOML file = `[record]` + `[env]` + `[vat]`? +
//! `[limits]` + (`[[steps]]` | `[load]`).

pub mod interp;
pub mod load;
pub mod record;
pub mod step;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

pub use interp::VarStore;
pub use load::LoadProfile;
pub use record::{lint_record, scenario_id, ExpectedOutcome, LintViolation, Record, ScenarioKind};
pub use step::Step;

/// Optional `[vat]` table: what the scenario expects vat to provide when
/// run with `--vat`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VatNeeds {
    /// The vat.toml runner that re-invokes rig inside the workspace.
    pub runner: String,
    /// Service ids the scenario relies on (validated informationally).
    #[serde(default)]
    pub services: Vec<String>,
}

/// `[limits]` — whole-scenario budget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limits {
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

fn default_timeout_secs() -> u64 {
    300
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            timeout_secs: default_timeout_secs(),
        }
    }
}

/// A parsed scenario file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub record: Record,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub vat: Option<VatNeeds>,
    #[serde(default)]
    pub limits: Limits,
    #[serde(default)]
    pub steps: Vec<Step>,
    #[serde(default)]
    pub load: Option<LoadProfile>,
}

/// Parse + structurally validate one scenario file. Returns the scenario
/// or the list of lint violations (parse errors are a single violation).
pub fn parse_scenario(path: &Path, text: &str) -> Result<Scenario, Vec<LintViolation>> {
    let scenario: Scenario = toml::from_str(text).map_err(|e| {
        vec![LintViolation {
            message: format!("TOML parse error: {e}"),
        }]
    })?;

    let mut violations = lint_record(path, &scenario.record);

    match scenario.record.kind {
        ScenarioKind::E2e => {
            if scenario.steps.is_empty() {
                violations.push(LintViolation {
                    message: "kind = \"e2e\" requires at least one [[steps]] entry".into(),
                });
            }
            if scenario.load.is_some() {
                violations.push(LintViolation {
                    message: "kind = \"e2e\" must not carry a [load] block".into(),
                });
            }
        }
        ScenarioKind::Load => {
            if scenario.load.is_none() {
                violations.push(LintViolation {
                    message: "kind = \"load\" requires a [load] block".into(),
                });
            }
        }
    }

    if violations.is_empty() {
        Ok(scenario)
    } else {
        Err(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const MINIMAL_E2E: &str = r#"
[record]
suite = "lumen"
dimension = "resilience"
case = "partition_recovery"
subject = "lumen under partition"
kind = "e2e"
expected = "pass"

[env]
upstream = "127.0.0.1:7373"

[[steps]]
type = "sleep"
name = "settle"
secs = 1
"#;

    #[test]
    fn minimal_e2e_parses() {
        let p = PathBuf::from("scenarios/resilience/partition_recovery.toml");
        let s = parse_scenario(&p, MINIMAL_E2E).unwrap();
        assert_eq!(s.record.suite, "lumen");
        assert_eq!(s.limits.timeout_secs, 300);
        assert_eq!(s.steps.len(), 1);
    }

    #[test]
    fn e2e_without_steps_is_violation() {
        let p = PathBuf::from("scenarios/resilience/partition_recovery.toml");
        let text = MINIMAL_E2E.replace(
            "[[steps]]\ntype = \"sleep\"\nname = \"settle\"\nsecs = 1\n",
            "",
        );
        let v = parse_scenario(&p, &text).unwrap_err();
        assert!(v.iter().any(|v| v.message.contains("at least one")));
    }

    #[test]
    fn load_kind_requires_load_block() {
        let p = PathBuf::from("scenarios/load/search_qps.toml");
        let text = r#"
[record]
suite = "lumen"
dimension = "load"
case = "search_qps"
subject = "search p99 under offered load"
kind = "load"
expected = "pass"
"#;
        let v = parse_scenario(&p, text).unwrap_err();
        assert!(v.iter().any(|v| v.message.contains("[load] block")));
    }

    #[test]
    fn parse_error_is_single_violation() {
        let p = PathBuf::from("scenarios/resilience/partition_recovery.toml");
        let v = parse_scenario(&p, "not toml [").unwrap_err();
        assert_eq!(v.len(), 1);
        assert!(v[0].message.contains("TOML parse error"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/rig/src/scenario/mod.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/rig/src/scenario/mod.rs` captured during rig
      standardization onto the codegen ladder.
```
