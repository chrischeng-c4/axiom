// SPEC-MANAGED: projects/rig/tech-design/semantic/source/projects-rig-src-scenario-mod-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Scenario model: one TOML file = `[record]` + `[env]` + `[vat]`? +
//! `[limits]` + (`[[steps]]` | `[load]`).

pub mod case;
pub mod interp;
pub mod load;
pub mod record;
pub mod step;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

pub use case::{
    lint_case, parse_case, CaseRecord, Clean, Exercise, LoadSpec, Prepare, QuerySpec, TestCase,
};
pub use interp::VarStore;
pub use load::LoadProfile;
pub use record::{lint_record, scenario_id, ExpectedOutcome, LintViolation, Record, ScenarioKind};
pub use step::Step;

/// Optional `[vat]` table: what the scenario expects vat to provide when
/// run with `--vat`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-scenario-mod-rs.md#source
pub struct VatNeeds {
    /// The vat.toml runner that re-invokes rig inside the workspace.
    pub runner: String,
    /// Service ids the scenario relies on (validated informationally).
    #[serde(default)]
    pub services: Vec<String>,
}

/// `[limits]` — whole-scenario budget.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-scenario-mod-rs.md#source
pub struct Limits {
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

fn default_timeout_secs() -> u64 {
    300
}

/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-scenario-mod-rs.md#source
impl Default for Limits {
    fn default() -> Self {
        Self {
            timeout_secs: default_timeout_secs(),
        }
    }
}

/// A parsed scenario file.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-scenario-mod-rs.md#source
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
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-scenario-mod-rs.md#source
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
// CODEGEN-END
