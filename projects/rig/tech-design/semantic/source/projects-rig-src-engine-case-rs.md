---
id: projects-rig-src-engine-case-rs
capability_refs:
  - id: scenario-engine
    role: primary
    claim: scenario-step-dsl-execution
    coverage: partial
    rationale: "This source unit implements rig scenario configuration, case modeling, or execution behavior used by the scenario engine."
fill_sections: [overview, source, changes]
---

# Standardized projects/rig/src/engine/case.rs

## Overview
<!-- type: overview lang: markdown -->

Rust source-unit TD for `projects/rig/src/engine/case.rs`, captured during #39 rig traceability closure.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Lifecycle case execution: `prepare(1) -> exercise(N) -> clean(1)`, with the
//! collector switching on N. `n=1` runs the exercise with full assertions and
//! buckets a verdict; `n>>1` drives the SAME request under an injected open-loop
//! schedule and folds stats. The send layer (transport) and the phase runner
//! ([`run_phase`]) are shared across both; only the collector differs.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::engine::loadgen::{run_transport, LoadStats, Schedule};
use crate::engine::run_phase;
use crate::engine::transport::{HttpTransport, Transport};
use crate::report::Finding;
use crate::scenario::interp::VarStore;
use crate::scenario::step::{HttpStep, Step};
use crate::scenario::TestCase;
use crate::verdict::{bucket, Verdict};

/// How the launcher wants the exercise driven. Built from efficiency-EC config,
/// never from the case (which carries only the request + metric).
pub enum Mode {
    /// `n=1`: run the exercise with full assertions -> verdict.
    Behavior,
    /// `n>>1`: drive the same request under this open-loop schedule -> stats.
    Load(Schedule),
}

/// The per-case result the launcher folds into one `rig.report/1`.
pub enum CaseResult {
    /// Behavior outcome: a bucketed verdict + any step findings.
    Verdict {
        case_id: String,
        verdict: Verdict,
        raw_passed: bool,
        findings: Vec<Finding>,
        vars: VarStore,
    },
    /// Load outcome: folded stats keyed by the gated metric + prepare findings.
    Stats {
        case_id: String,
        metric: String,
        stats: LoadStats,
        findings: Vec<Finding>,
        vars: VarStore,
    },
}

/// Run one lifecycle case end to end. `prepare` runs once (short-circuiting the
/// case on failure); the exercise is driven per `mode`; `clean` runs once and
/// never gates.
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-engine-case-rs.md#source
pub fn run_case(case: &TestCase, mode: Mode) -> CaseResult {
    let id = case.case_id();
    let mut vars = VarStore::seed(&case.env);
    let timeout = case.limits.timeout_secs;

    // --- prepare(1): always run-once, OUTSIDE any measured window. ---
    let prep = run_phase(&case.prepare.steps, timeout, &id, &mut vars);
    if !prep.raw_passed {
        return finish_prepare_failure(case, id, prep.findings, mode, vars);
    }

    match mode {
        // ===== n = 1: assertions -> verdict =====
        Mode::Behavior => {
            let (raw_passed, findings) = match exercise_http_step(case) {
                // http engine: reuse the full step dispatcher (status + jsonpath).
                Some(step) => {
                    let ex = run_phase(std::slice::from_ref(&step), timeout, &id, &mut vars);
                    (ex.raw_passed, ex.findings)
                }
                // sql engine: a single query exec (feature `postgres`).
                None => run_query_once(case, &id),
            };
            run_clean(case, &id, &mut vars);
            CaseResult::Verdict {
                verdict: bucket(case.record.expected, raw_passed),
                raw_passed,
                findings,
                case_id: id,
                vars,
            }
        }
        // ===== n >> 1: stats (pin gating happens at the launcher) =====
        Mode::Load(schedule) => {
            let stats = match build_load_transport(case, &vars) {
                Ok(transport) => run_transport(&schedule, &transport),
                Err(e) => LoadStats {
                    abort: Some(e),
                    ..Default::default()
                },
            };
            run_clean(case, &id, &mut vars);
            CaseResult::Stats {
                metric: case.load.metric.clone(),
                stats,
                findings: Vec::new(),
                case_id: id,
                vars,
            }
        }
    }
}

/// Lower the exercise's http request into a single `http` step so it reuses the
/// same `run_step` dispatcher (status + jsonpath + capture) as `n=1`. `None` for
/// a sql (`query`) exercise.
fn exercise_http_step(case: &TestCase) -> Option<Step> {
    case.exercise.request.as_ref().map(|req| {
        Step::Http(HttpStep {
            name: "exercise".to_string(),
            request: req.clone(),
            capture: BTreeMap::new(),
        })
    })
}

/// Build the load transport from the exercise engine: http -> `HttpTransport`
/// (status-only, jsonpath stripped); sql -> `PostgresTransport` (feature
/// `postgres`). Both ride the SAME scheduler — the lumen-vs-pg comparability.
fn build_load_transport(case: &TestCase, vars: &VarStore) -> Result<Arc<dyn Transport>, String> {
    if let Some(req) = &case.exercise.request {
        let mut req = req.clone();
        req.expect.jsonpath.clear();
        return Ok(Arc::new(HttpTransport {
            request: req,
            vars: vars.clone(),
        }));
    }
    if let Some(q) = &case.exercise.query {
        #[cfg(feature = "postgres")]
        {
            return Ok(Arc::new(crate::engine::transport::PostgresTransport {
                dsn: q.dsn.clone(),
                sql: q.sql.clone(),
            }));
        }
        #[cfg(not(feature = "postgres"))]
        {
            let _ = q;
            return Err("query engine needs the `postgres` feature".to_string());
        }
    }
    Err("exercise has neither request nor query".to_string())
}

/// Single sql exec for an `n=1` query behavior case (feature `postgres`).
#[cfg(feature = "postgres")]
fn run_query_once(case: &TestCase, id: &str) -> (bool, Vec<Finding>) {
    use crate::engine::transport::{PostgresTransport, Transport};
    let Some(q) = &case.exercise.query else {
        return (false, vec![query_finding(id, "exercise has no query")]);
    };
    let t = PostgresTransport {
        dsn: q.dsn.clone(),
        sql: q.sql.clone(),
    };
    match t.connect().and_then(|mut w| w.execute()) {
        Ok(()) => (true, Vec::new()),
        Err(e) => (false, vec![query_finding(id, &e)]),
    }
}

#[cfg(not(feature = "postgres"))]
fn run_query_once(case: &TestCase, id: &str) -> (bool, Vec<Finding>) {
    let _ = case;
    (
        false,
        vec![query_finding(
            id,
            "query engine needs the `postgres` feature",
        )],
    )
}

/// A `ScenarioError` finding for a failed/unavailable sql exercise.
fn query_finding(id: &str, detail: &str) -> Finding {
    use crate::report::{finding_id, Invoke, Kind, Severity};
    Finding {
        id: finding_id(Kind::ScenarioError, &format!("query/{id}")),
        severity: Severity::High,
        kind: Kind::ScenarioError,
        title: format!("sql exercise failed in `{id}`"),
        detail: detail.to_string(),
        remediation: "Check the dsn/sql and that the `postgres` feature is enabled.".to_string(),
        invoke: Invoke::command("rig test".to_string()),
        evidence: serde_json::json!({ "engine": "sql" }),
    }
}

/// Run-once teardown. Findings are discarded — clean is hygiene, never a gate
/// (under vat the COW clone is dropped, so clean is usually empty).
fn run_clean(case: &TestCase, id: &str, vars: &mut VarStore) {
    let _ = run_phase(&case.clean.steps, case.limits.timeout_secs, id, vars);
}

/// prepare failed: behavior buckets a failing verdict; load yields aborted stats.
/// Both carry the prepare findings so the launcher can surface them.
fn finish_prepare_failure(
    case: &TestCase,
    id: String,
    findings: Vec<Finding>,
    mode: Mode,
    vars: VarStore,
) -> CaseResult {
    match mode {
        Mode::Behavior => CaseResult::Verdict {
            verdict: bucket(case.record.expected, false),
            raw_passed: false,
            findings,
            case_id: id,
            vars,
        },
        Mode::Load(_) => CaseResult::Stats {
            metric: case.load.metric.clone(),
            stats: LoadStats {
                abort: Some("prepare failed before the load window".to_string()),
                ..Default::default()
            },
            findings,
            case_id: id,
            vars,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::parse_case;
    use std::path::PathBuf;

    // Exercise points at an unreachable port: behavior must bucket Red (expected
    // pass, but the request fails) — no stub server needed.
    const UNREACHABLE: &str = r#"
[case]
id = "search_basic"
suite = "lumen"
dimension = "api"
subject = "search returns a hit"
expected = "pass"

[exercise]
[exercise.request]
method = "GET"
url = "http://127.0.0.1:1/search"
[exercise.request.expect]
status = 200
timeout_ms = 500
"#;

    #[test]
    fn behavior_unreachable_exercise_is_red() {
        let p = PathBuf::from("cases/api/search_basic.toml");
        let case = parse_case(&p, UNREACHABLE).unwrap();
        match run_case(&case, Mode::Behavior) {
            CaseResult::Verdict {
                verdict,
                raw_passed,
                case_id,
                ..
            } => {
                assert!(!raw_passed);
                assert_eq!(verdict, Verdict::Red);
                assert_eq!(case_id, "lumen/api/search_basic");
            }
            _ => panic!("behavior mode must yield a verdict"),
        }
    }

    #[test]
    fn load_unreachable_exercise_folds_full_error_rate() {
        let p = PathBuf::from("cases/load/search_basic.toml");
        let text = UNREACHABLE.replace("dimension = \"api\"", "dimension = \"load\"");
        // path stem still search_basic; dimension load is fine for this unit test
        let case = parse_case(&PathBuf::from("cases/load/search_basic.toml"), &text).unwrap();
        let schedule = Schedule {
            target_qps: 20,
            workers: 2,
            duration_secs: 1,
            warmup_secs: 0,
        };
        match run_case(&case, Mode::Load(schedule)) {
            CaseResult::Stats { metric, stats, .. } => {
                assert_eq!(metric, "p99_ms");
                assert!(stats.error_rate > 0.0 || stats.abort.is_some());
            }
            _ => panic!("load mode must yield stats"),
        }
        let _ = p;
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/rig/src/engine/case.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/rig/src/engine/case.rs` captured during #39 rig standardization.
```
