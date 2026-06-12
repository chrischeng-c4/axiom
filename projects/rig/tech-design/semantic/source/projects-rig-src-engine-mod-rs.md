---
id: projects-rig-src-engine-mod-rs
fill_sections: [overview, source, changes]
---

# Standardized projects/rig/src/engine/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/rig/src/engine/mod.rs`, captured as a rust-source-unit (td_ast) item-tree
during rig standardization onto the codegen ladder.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Scenario execution engine: drive steps sequentially against a
//! [`VarStore`], collecting findings and captured metrics.
//!
//! Budget model: every individual step is bounded by its own timeout
//! (http `timeout_ms`, exec `timeout_secs`, wait_until `budget_secs`), and
//! the WHOLE scenario is bounded by `[limits] timeout_secs`, checked
//! between steps — so the worst overshoot is one step's own budget.

pub mod assert;
pub mod exec;
pub mod http;
pub mod loadgen;
pub mod rss;
pub mod sample;
pub mod timeout;
pub mod transport;

use std::time::Instant;

use serde_json::json;

use crate::report::{finding_id, Finding, Invoke, Kind, Severity};
use crate::scenario::{scenario_id, Scenario, Step, VarStore};

/// The raw result of executing one scenario's steps.
pub struct ScenarioRun {
    pub scenario_id: String,
    /// True when every step held its expectations.
    pub raw_passed: bool,
    pub findings: Vec<Finding>,
    /// Vars captured during the run (metrics for pins live here too).
    pub vars: VarStore,
    pub steps_run: usize,
}

/// Execute an e2e scenario's steps. (Load scenarios are driven by the
/// loadgen engine — Phase 5.)
pub fn run_scenario(scenario: &Scenario) -> ScenarioRun {
    let id = scenario_id(&scenario.record);
    let mut vars = VarStore::seed(&scenario.env);
    let mut findings: Vec<Finding> = Vec::new();
    let started = Instant::now();
    let rerun = format!("rig run --scenario <path-of {id}>");
    let mut steps_run = 0usize;

    for step in &scenario.steps {
        if started.elapsed().as_secs() >= scenario.limits.timeout_secs {
            findings.push(Finding {
                id: finding_id(Kind::Timeout, &id),
                severity: Severity::High,
                kind: Kind::Timeout,
                title: format!("scenario `{id}` exceeded its {}s budget", scenario.limits.timeout_secs),
                detail: format!(
                    "stopped before step `{}` after {} step(s); raise [limits] timeout_secs or trim the scenario",
                    step.name(),
                    steps_run
                ),
                remediation: "Raise [limits].timeout_secs or split the scenario.".into(),
                invoke: Invoke::command(rerun.clone()),
                evidence: json!({ "steps_run": steps_run, "budget_secs": scenario.limits.timeout_secs }),
            });
            break;
        }
        match run_step(step, &id, &mut vars, &rerun) {
            StepResult::Ok => {}
            StepResult::Failed(finding) => {
                findings.push(finding);
                // A failed step invalidates everything after it; stop.
                break;
            }
        }
        steps_run += 1;
    }

    ScenarioRun {
        raw_passed: findings.is_empty(),
        scenario_id: id,
        findings,
        vars,
        steps_run,
    }
}

enum StepResult {
    Ok,
    Failed(Finding),
}

fn run_step(step: &Step, scenario: &str, vars: &mut VarStore, rerun: &str) -> StepResult {
    let subject = format!("{scenario}#{}", step.name());
    let fail = |kind: Kind, detail: String, remediation: &str, evidence: serde_json::Value| {
        StepResult::Failed(Finding {
            id: finding_id(kind, &subject),
            severity: Severity::High,
            kind,
            title: format!("step `{}` failed in `{scenario}`", subject_name(&subject)),
            detail,
            remediation: remediation.to_string(),
            invoke: Invoke::command(rerun.to_string()),
            evidence,
        })
    };

    match step {
        Step::Sleep { secs, .. } => {
            std::thread::sleep(std::time::Duration::from_secs(*secs));
            StepResult::Ok
        }

        Step::Http(s) => match http::execute(&s.request, vars) {
            Err(e) => fail(
                Kind::ScenarioError,
                e,
                "Fix the scenario's vars/templates and re-run.",
                json!({}),
            ),
            Ok(outcome) => {
                if let Some(v) = &outcome.violation {
                    return fail(
                        Kind::StepFailure,
                        v.clone(),
                        "Inspect the serving process / expectation; the evidence carries status and latency.",
                        json!({ "status": outcome.status, "latency_ms": outcome.latency_ms }),
                    );
                }
                for (var, key) in &s.capture {
                    match http::capture_value(&outcome, key) {
                        Some(v) => vars.set(var, v),
                        None => {
                            return fail(
                                Kind::ScenarioError,
                                format!("capture `{var}` = `{key}` resolved nothing"),
                                "Capture keys are `status`, `latency_ms`, or a `$.dot.path` present in the response body.",
                                json!({ "capture": key }),
                            )
                        }
                    }
                }
                StepResult::Ok
            }
        },

        Step::Sample(s) => {
            let mut observations = Vec::with_capacity(s.samples as usize);
            for _ in 0..s.samples {
                match http::execute(&s.request, vars) {
                    Err(e) => {
                        return fail(
                            Kind::ScenarioError,
                            e,
                            "Fix the scenario's vars/templates and re-run.",
                            json!({}),
                        )
                    }
                    Ok(o) => observations.push((o.latency_ms, o.violation.is_none())),
                }
            }
            let stats = sample::SampleStats::fold(&observations);
            if !s.allow_failures && stats.fail_count > 0 {
                return fail(
                    Kind::StepFailure,
                    format!(
                        "{}/{} sampled requests violated expectations",
                        stats.fail_count, s.samples
                    ),
                    "Set `allow_failures = true` if failures are the signal being measured; otherwise inspect the serving process.",
                    json!({ "fail_count": stats.fail_count, "ok_count": stats.ok_count }),
                );
            }
            for (var, key) in &s.capture {
                match stats.get(key) {
                    Some(v) => vars.set(var, json!(v)),
                    None => {
                        return fail(
                            Kind::ScenarioError,
                            format!("unknown sample stat `{key}` for capture `{var}`"),
                            "Valid stats: p50_ms p90_ms p99_ms mean_ms ok_count fail_count.",
                            json!({ "capture": key }),
                        )
                    }
                }
            }
            StepResult::Ok
        }

        Step::Assert(s) => {
            for expr in &s.exprs {
                match assert::evaluate(expr, vars) {
                    Err(e) => {
                        return fail(
                            Kind::ScenarioError,
                            e,
                            "Fix the expression or the vars it references.",
                            json!({ "expr": expr }),
                        )
                    }
                    Ok(false) => {
                        let snapshot = expr_snapshot(expr, vars);
                        return fail(
                            Kind::AssertionFailure,
                            format!("`{expr}` is false ({snapshot})"),
                            "The asserted behavior did not hold; the evidence carries the operand values.",
                            json!({ "expr": expr, "operands": snapshot }),
                        );
                    }
                    Ok(true) => {}
                }
            }
            StepResult::Ok
        }

        Step::WaitUntil(s) => {
            let started = Instant::now();
            loop {
                if let Ok(outcome) = http::execute(&s.probe, vars) {
                    if outcome.violation.is_none() {
                        vars.set(
                            format!("{}_recovered_secs", s.name),
                            json!(started.elapsed().as_secs_f64()),
                        );
                        return StepResult::Ok;
                    }
                }
                if started.elapsed().as_secs() >= s.budget_secs {
                    return fail(
                        Kind::StepFailure,
                        format!("probe did not pass within {}s", s.budget_secs),
                        "The system did not recover/become ready inside the budget.",
                        json!({ "budget_secs": s.budget_secs }),
                    );
                }
                std::thread::sleep(std::time::Duration::from_millis(s.interval_ms));
            }
        }

        Step::Exec(s) => match exec::execute(s, vars) {
            Err(e) => fail(
                Kind::ScenarioError,
                e,
                "Fix the exec step's argv/vars and re-run.",
                json!({}),
            ),
            Ok(outcome) => {
                if let Some(v) = &outcome.violation {
                    let kind = if outcome.timed_out {
                        Kind::Timeout
                    } else {
                        Kind::StepFailure
                    };
                    return fail(
                        kind,
                        v.clone(),
                        "Inspect the command; its stderr tail is in the detail.",
                        json!({ "exit_code": outcome.exit_code, "timed_out": outcome.timed_out }),
                    );
                }
                for (var, key) in &s.capture {
                    let value = match key.as_str() {
                        "stdout" => json!(outcome.stdout),
                        "exit_code" => json!(outcome.exit_code),
                        _ => {
                            return fail(
                                Kind::ScenarioError,
                                format!("unknown exec capture `{key}`"),
                                "Valid exec captures: stdout, exit_code.",
                                json!({ "capture": key }),
                            )
                        }
                    };
                    vars.set(var, value);
                }
                StepResult::Ok
            }
        },

        Step::MeasureRss(s) => match rss::execute(s, vars) {
            Err(e) => fail(
                Kind::ScenarioError,
                e,
                "Give the step a resolvable `pid_var` (preferred) or a unique `process` name.",
                json!({}),
            ),
            Ok(()) => StepResult::Ok,
        },
    }
}

fn subject_name(subject: &str) -> &str {
    subject.rsplit('#').next().unwrap_or(subject)
}

/// Render the operand values of a failed assertion for the report.
fn expr_snapshot(expr: &str, vars: &VarStore) -> String {
    expr.split_whitespace()
        .filter(|t| vars.get_f64(t).is_some() && t.parse::<f64>().is_err())
        .map(|t| format!("{t}={}", vars.get_f64(t).unwrap()))
        .collect::<Vec<_>>()
        .join(", ")
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/rig/src/engine/mod.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/rig/src/engine/mod.rs` captured during rig
      standardization onto the codegen ladder.
```
