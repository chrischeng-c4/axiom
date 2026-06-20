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

/// `[exercise]` — the measured op. Exactly one engine: `request` (http) XOR
/// `query` (sql); `n` is the knob.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-scenario-case-rs.md#source
pub struct Exercise {
    /// Iteration count. `1` (default) => behavior; `>>1` => load.
    #[serde(default = "default_n")]
    pub n: u32,
    /// The http request engine (XOR `query`). Its `expect` carries the n=1
    /// assertions; in load mode only the status contract is checked.
    #[serde(default)]
    pub request: Option<HttpRequest>,
    /// The sql query engine (XOR `request`) — the lumen-vs-pg comparability
    /// half, driven on the SAME scheduler via PostgresTransport (feature `postgres`).
    #[serde(default)]
    pub query: Option<QuerySpec>,
}

/// `[exercise.query]` — a sql query op (executed via the `postgres` feature).
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-scenario-case-rs.md#source
pub struct QuerySpec {
    /// libpq-style DSN, e.g. `postgresql://user@127.0.0.1/db`.
    pub dsn: String,
    /// The SQL executed each tick (literals inline; no bind params in v1).
    pub sql: String,
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

/// Known EC dimensions a lifecycle case may live under (rule 6).
const VALID_DIMENSIONS: &[&str] = &[
    "behavior",
    "efficiency",
    "security",
    "stability",
    "resilience",
    "load",
    "endurance",
    "api",
];

/// True when `pred` is a compare-only predicate (`<op> <literal>`) or `exists`
/// — the bound (rule 4) that keeps the expect grammar from growing control flow.
fn is_compare_predicate(pred: &str) -> bool {
    let p = pred.trim();
    if p == "exists" {
        return true;
    }
    for op in ["==", "!=", "<=", ">=", "<", ">"] {
        if let Some(rest) = p.strip_prefix(op) {
            return !rest.trim().is_empty();
        }
    }
    false
}

/// The full lifecycle lint (the 6 rules): path==record floor plus the structural
/// bounds that keep the DSL a pure-data EC description — request present, http/sql
/// only steps (no exec/script), compare-only expects (no control flow), and a
/// known dimension.
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

    // Rule 6: dimension must be a known EC dimension.
    if !VALID_DIMENSIONS.contains(&case.record.dimension.as_str()) {
        v.push(LintViolation {
            message: format!(
                "[case].dimension `{}` is not a known EC dimension ({})",
                case.record.dimension,
                VALID_DIMENSIONS.join(", ")
            ),
        });
    }

    // Rule 5: prepare/clean steps are http only — no exec/script in the case DSL
    // (heavy/imperative work belongs to the vat runner).
    for step in case.prepare.steps.iter().chain(case.clean.steps.iter()) {
        if !matches!(step, Step::Http(_)) {
            v.push(LintViolation {
                message: format!(
                    "step `{}` in [prepare]/[clean] must be an http step — no exec/script in the case DSL (push logic to the vat runner)",
                    step.name()
                ),
            });
        }
    }

    // Rule 2: exactly one exercise engine — request (http) XOR query (sql).
    match (&case.exercise.request, &case.exercise.query) {
        (Some(_), Some(_)) => v.push(LintViolation {
            message: "[exercise] has both request and query — exactly one engine (http XOR sql)"
                .to_string(),
        }),
        (None, None) => v.push(LintViolation {
            message: "[exercise] has neither request nor query — exactly one engine (http XOR sql)"
                .to_string(),
        }),
        _ => {}
    }

    // Rule 4: every jsonpath expectation is a compare-only predicate — the DSL
    // never grows control flow (multi-condition = multiple entries, implicit AND).
    let mut requests = Vec::new();
    if let Some(req) = &case.exercise.request {
        requests.push(req);
    }
    for step in case.prepare.steps.iter().chain(case.clean.steps.iter()) {
        if let Step::Http(h) = step {
            requests.push(&h.request);
        }
    }
    for req in requests {
        for (jp, pred) in &req.expect.jsonpath {
            if !is_compare_predicate(pred) {
                v.push(LintViolation {
                    message: format!(
                        "expect `{jp}` = `{pred}` must be a compare predicate (== != < <= > >= or `exists`); no and/or/if/fns"
                    ),
                });
            }
        }
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

    // Rule 5: an exec/script step in prepare is rejected (no imperative DSL).
    #[test]
    fn exec_step_in_prepare_is_rejected() {
        let text = BEHAVIOR.replace(
            "[exercise]",
            "[[prepare.step]]\ntype = \"exec\"\nname = \"seed\"\ncmd = [\"python3\", \"-c\", \"pass\"]\n\n[exercise]",
        );
        let p = PathBuf::from("cases/api/search_basic.toml");
        let v = parse_case(&p, &text).unwrap_err();
        assert!(v.iter().any(|x| x.message.contains("must be an http step")));
    }

    // Rule 4: a control-flow / non-compare expect predicate is rejected.
    #[test]
    fn control_flow_expect_is_rejected() {
        let text = BEHAVIOR.replace(
            "status = 200",
            "status = 200\njsonpath = { \"$.total\" = \"len(x) and y\" }",
        );
        let p = PathBuf::from("cases/api/search_basic.toml");
        let v = parse_case(&p, &text).unwrap_err();
        assert!(v.iter().any(|x| x.message.contains("compare predicate")));
    }

    // Rule 6: an unknown dimension is rejected.
    #[test]
    fn unknown_dimension_is_rejected() {
        let text = BEHAVIOR.replace("dimension = \"api\"", "dimension = \"frobnicate\"");
        let p = PathBuf::from("cases/frobnicate/search_basic.toml");
        let v = parse_case(&p, &text).unwrap_err();
        assert!(v.iter().any(|x| x.message.contains("known EC dimension")));
    }

    // A valid compare predicate (>= 1) passes rule 4.
    #[test]
    fn compare_predicate_expect_passes() {
        let text = BEHAVIOR.replace(
            "status = 200",
            "status = 200\njsonpath = { \"$.total\" = \">= 1\" }",
        );
        let p = PathBuf::from("cases/api/search_basic.toml");
        assert!(parse_case(&p, &text).is_ok());
    }

    const SQL: &str = r#"
[case]
id = "search_qps_pg"
suite = "lumen"
dimension = "load"
subject = "pg baseline under offered load"
expected = "pass"

[exercise]
n = 200
[exercise.query]
dsn = "postgresql://localhost/lumenbench"
sql = "SELECT id FROM docs WHERE bio @@ to_tsquery('engineer') LIMIT 1"

[load]
metric = "p99_ms"
"#;

    // Rule 2: a sql `query` exercise parses and selects the sql engine.
    #[test]
    fn query_case_parses_and_is_load() {
        let p = PathBuf::from("cases/load/search_qps_pg.toml");
        let c = parse_case(&p, SQL).unwrap();
        assert!(c.is_load());
        assert!(c.exercise.request.is_none());
        let q = c.exercise.query.as_ref().expect("query engine");
        assert!(q.sql.contains("to_tsquery"));
    }

    // Rule 2: having BOTH request and query is rejected (exactly one engine).
    #[test]
    fn both_engines_is_rejected() {
        let text = SQL.replace(
            "[load]",
            "[exercise.request]\nmethod = \"GET\"\nurl = \"http://x/y\"\n\n[load]",
        );
        let p = PathBuf::from("cases/load/search_qps_pg.toml");
        let v = parse_case(&p, &text).unwrap_err();
        assert!(v
            .iter()
            .any(|x| x.message.contains("both request and query")));
    }

    // Rule 2: an exercise with neither engine is rejected.
    #[test]
    fn no_engine_is_rejected() {
        let text = "[case]\nid = \"x\"\nsuite = \"lumen\"\ndimension = \"api\"\nsubject = \"s\"\nexpected = \"pass\"\n[exercise]\n";
        let p = PathBuf::from("cases/api/x.toml");
        let v = parse_case(&p, text).unwrap_err();
        assert!(v
            .iter()
            .any(|x| x.message.contains("neither request nor query")));
    }
}
// CODEGEN-END
