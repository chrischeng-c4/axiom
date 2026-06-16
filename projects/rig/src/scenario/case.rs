// SPEC-MANAGED: projects/rig/tech-design/semantic/source/projects-rig-src-scenario-case-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Lifecycle case model: one TOML file = `[case]` + `[prepare]` + `[exercise]`
//! + `[load]`? + `[clean]`. The converged shape `prepare(1) -> exercise(N) ->
//! clean(1)`: `n` is the only knob — `n=1` is behavior (full assert -> verdict),
//! `n>>1` is load (status-only -> folded stats -> pin gate). The Rust engine
//! supplies the execution semantics (short-circuit / N-switch / finally); this
//! module only describes the structure.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

use super::record::{ExpectedOutcome, LintViolation};
use super::step::{HttpRequest, Step};
use super::Limits;

fn default_required() -> bool {
    true
}
fn default_n() -> u32 {
    1
}
fn default_metric() -> String {
    "p99_ms".to_string()
}

/// `[case]` — the lifecycle case identity. `id` MUST equal the filename stem
/// and `dimension` the parent directory (path==record, enforced by lint).
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-scenario-case-rs.md#source
pub struct CaseRecord {
    /// snake_case case key; MUST equal the filename stem.
    pub id: String,
    /// Owning suite/project, e.g. `lumen`.
    pub suite: String,
    /// Facet directory: `resilience`, `load`, `api`, ... (== parent dir).
    pub dimension: String,
    /// Human one-liner: the behavior under test.
    #[serde(default)]
    pub subject: String,
    pub expected: ExpectedOutcome,
    /// Required cases gate the run; optional ones report only.
    #[serde(default = "default_required")]
    pub required: bool,
    /// Back-link to the EC markdown contract that generated this case.
    #[serde(default)]
    pub source_contract: Option<String>,
}

/// `[prepare]` — run-once setup. Heavy work (services, seed) is delegated to
/// vat via `needs`/`runner`; case-local `[[prepare.step]]` entries are
/// lightweight http/sql only (enforced by lint in rig-2).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-scenario-case-rs.md#source
pub struct Prepare {
    /// Services vat must provision before the case runs.
    #[serde(default)]
    pub needs: Vec<String>,
    /// vat runner that performs heavy seed work (optional).
    #[serde(default)]
    pub runner: Option<String>,
    /// Case-local lightweight setup steps (http/sql only).
    #[serde(default, rename = "step")]
    pub steps: Vec<Step>,
}

/// `[exercise]` — the measured op. Exactly one request; `n` is the knob.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-scenario-case-rs.md#source
pub struct Exercise {
    /// Iteration count. `1` (default) => behavior; `>>1` => load.
    #[serde(default = "default_n")]
    pub n: u32,
    /// The single measured request (the load subject). Its `expect` carries
    /// the n=1 assertions; in load mode only the status contract is checked.
    pub request: HttpRequest,
}

/// `[load]` — consulted only when `exercise.n > 1`. The qps/workers/duration
/// schedule is injected by the launcher from efficiency-EC config, not stored
/// here; the case only names the folded metric the pin gates on.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-scenario-case-rs.md#source
pub struct LoadSpec {
    /// `p50_ms` | `p99_ms` | `error_rate` | `achieved_qps`.
    #[serde(default = "default_metric")]
    pub metric: String,
}

/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-scenario-case-rs.md#source
impl Default for LoadSpec {
    fn default() -> Self {
        Self {
            metric: default_metric(),
        }
    }
}

/// `[clean]` — run-once teardown. Under vat the COW clone is discarded, so
/// `delegate = "vat-cow"` needs no steps; otherwise case-local http/sql
/// `[[clean.step]]` entries run.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-scenario-case-rs.md#source
pub struct Clean {
    /// Teardown delegate, e.g. `vat-cow` (drop the COW clone).
    #[serde(default)]
    pub delegate: Option<String>,
    /// Case-local teardown steps (http/sql only).
    #[serde(default, rename = "step")]
    pub steps: Vec<Step>,
}

/// A parsed lifecycle case file.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-scenario-case-rs.md#source
pub struct TestCase {
    #[serde(rename = "case")]
    pub record: CaseRecord,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub limits: Limits,
    #[serde(default)]
    pub prepare: Prepare,
    pub exercise: Exercise,
    #[serde(default)]
    pub load: LoadSpec,
    #[serde(default)]
    pub clean: Clean,
}

/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-scenario-case-rs.md#source
impl TestCase {
    /// `suite/dimension/id` — the stable id used in reports, pins, baselines.
    /// Identical shape to [`super::scenario_id`] so pins/baselines carry over.
    pub fn case_id(&self) -> String {
        format!(
            "{}/{}/{}",
            self.record.suite, self.record.dimension, self.record.id
        )
    }

    /// True when this case is load-driven (`exercise.n > 1`).
    pub fn is_load(&self) -> bool {
        self.exercise.n > 1
    }
}

/// Parse + structurally validate one lifecycle case file. The full 6-rule lint
/// lands in rig-2; this enforces the parse + path==record floor.
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-scenario-case-rs.md#source
pub fn parse_case(path: &Path, text: &str) -> Result<TestCase, Vec<LintViolation>> {
    let case: TestCase = toml::from_str(text).map_err(|e| {
        vec![LintViolation {
            message: format!("TOML parse error: {e}"),
        }]
    })?;
    let violations = lint_case(path, &case);
    if violations.is_empty() {
        Ok(case)
    } else {
        Err(violations)
    }
}

/// Path==record floor: `[case].id` == filename stem, `dimension` == parent dir,
/// subject present.
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-scenario-case-rs.md#source
pub fn lint_case(path: &Path, case: &TestCase) -> Vec<LintViolation> {
    let mut v = Vec::new();
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    let parent = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    if case.record.id != stem {
        v.push(LintViolation {
            message: format!(
                "[case].id `{}` != filename stem `{stem}` — the filename is the case key",
                case.record.id
            ),
        });
    }
    if case.record.dimension != parent {
        v.push(LintViolation {
            message: format!(
                "[case].dimension `{}` != parent directory `{parent}` — the directory is the taxonomy",
                case.record.dimension
            ),
        });
    }
    if case.record.subject.trim().is_empty() {
        v.push(LintViolation {
            message: "[case].subject must name the behavior under test".to_string(),
        });
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const BEHAVIOR: &str = r#"
[case]
id = "search_basic"
suite = "lumen"
dimension = "api"
subject = "search returns a hit"
expected = "pass"

[exercise]
[exercise.request]
method = "POST"
url = "http://{{upstream}}/search"
body = '{"q":"hi"}'
[exercise.request.expect]
status = 200
"#;

    const LOAD: &str = r#"
[case]
id = "search_qps"
suite = "lumen"
dimension = "load"
subject = "search p99 under offered load"
expected = "pass"

[prepare]
needs = ["lumen"]

[exercise]
n = 200
[exercise.request]
method = "POST"
url = "http://{{upstream}}/search"
body = '{"q":"hi"}'
[exercise.request.expect]
status = 200

[load]
metric = "p99_ms"

[clean]
delegate = "vat-cow"
"#;

    #[test]
    fn behavior_case_parses_n_defaults_to_one() {
        let p = PathBuf::from("cases/api/search_basic.toml");
        let c = parse_case(&p, BEHAVIOR).unwrap();
        assert_eq!(c.exercise.n, 1);
        assert!(!c.is_load());
        assert_eq!(c.case_id(), "lumen/api/search_basic");
        assert_eq!(c.load.metric, "p99_ms");
    }

    #[test]
    fn load_case_parses_with_metric_and_prepare() {
        let p = PathBuf::from("cases/load/search_qps.toml");
        let c = parse_case(&p, LOAD).unwrap();
        assert_eq!(c.exercise.n, 200);
        assert!(c.is_load());
        assert_eq!(c.load.metric, "p99_ms");
        assert_eq!(c.prepare.needs, vec!["lumen".to_string()]);
        assert_eq!(c.clean.delegate.as_deref(), Some("vat-cow"));
    }

    #[test]
    fn id_mismatch_is_violation() {
        let p = PathBuf::from("cases/api/other.toml");
        let v = parse_case(&p, BEHAVIOR).unwrap_err();
        assert!(v.iter().any(|x| x.message.contains("filename stem")));
    }

    #[test]
    fn parse_error_is_single_violation() {
        let p = PathBuf::from("cases/api/search_basic.toml");
        let v = parse_case(&p, "not toml [").unwrap_err();
        assert_eq!(v.len(), 1);
        assert!(v[0].message.contains("TOML parse error"));
    }
}
// CODEGEN-END
