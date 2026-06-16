// SPEC-MANAGED: projects/rig/tech-design/semantic/source/projects-rig-src-engine-case-rs.md#rust-source-unit
// CODEGEN-BEGIN
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
            let step = exercise_http_step(case);
            let ex = run_phase(std::slice::from_ref(&step), timeout, &id, &mut vars);
            run_clean(case, &id, &mut vars);
            let raw_passed = ex.raw_passed;
            CaseResult::Verdict {
                verdict: bucket(case.record.expected, raw_passed),
                raw_passed,
                findings: ex.findings,
                case_id: id,
                vars,
            }
        }
        // ===== n >> 1: stats (pin gating happens at the launcher) =====
        Mode::Load(schedule) => {
            // Same request, status-only success: strip jsonpath for the load path.
            let mut req = case.exercise.request.clone();
            req.expect.jsonpath.clear();
            let transport: Arc<dyn Transport> = Arc::new(HttpTransport {
                request: req,
                vars: vars.clone(),
            });
            let stats = run_transport(&schedule, &transport);
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

/// Lower the exercise request into a single `http` step so it reuses the same
/// `run_step` dispatcher (status + jsonpath assertions + capture) as `n=1`.
fn exercise_http_step(case: &TestCase) -> Step {
    Step::Http(HttpStep {
        name: "exercise".to_string(),
        request: case.exercise.request.clone(),
        capture: BTreeMap::new(),
    })
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
// CODEGEN-END
