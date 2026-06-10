//! Verb tree + dispatch. One JSON `RigReport` per invocation on stdout.

use clap::{Args, Parser, Subcommand};
use rig::report::{ReportBuilder, RigReport};

/// Top-level `rig` command tree.
#[derive(Parser, Debug)]
#[command(
    name = "rig",
    version,
    about = "rig — declarative test-scenario harness: e2e scenarios + open-loop load pins (JSON on stdout by default)",
    disable_help_subcommand = true
)]
pub struct RigCommand {
    #[command(subcommand)]
    pub verb: Verb,
    #[command(flatten)]
    pub output: OutputOpts,
}

/// Output-format opt-ins shared by every verb. JSON-on-stdout is the default.
#[derive(Args, Debug, Clone, Default)]
pub struct OutputOpts {
    /// Render a human-readable summary to stderr in addition to the JSON report.
    #[arg(long, global = true)]
    pub human: bool,
    /// Emit the JSON report as a single dense line.
    #[arg(long, global = true)]
    pub compact: bool,
}

#[derive(Subcommand, Debug)]
pub enum Verb {
    /// Discover, lint, execute scenarios; gate pins; print ONE report.
    Run(RunArgs),
    /// Validate scenario record contracts only (path==record, key presence) — no execution.
    Lint(LintArgs),
    /// Re-project the persisted `.rig/last-report.json` (read-only).
    Report,
    /// Offline self-describer (report schema + step-type catalog). v1.
    Spec,
    /// Offline agent playbook. v1.
    Llm,
}

#[derive(Args, Debug, Default)]
pub struct RunArgs {
    /// Run a single scenario file.
    #[arg(long, group = "run_target")]
    pub scenario: Option<String>,
    /// Discover and run every scenario under this directory.
    #[arg(long, group = "run_target")]
    pub dir: Option<String>,
    /// Gate metrics against pins discovered under this directory.
    #[arg(long)]
    pub pins: Option<String>,
    /// Record measured metrics as new baselines instead of gating.
    #[arg(long)]
    pub update_baselines: bool,
    /// Wrap execution in `vat run` (the scenario's [vat] table names the runner).
    #[arg(long)]
    pub vat: bool,
}

#[derive(Args, Debug, Default)]
pub struct LintArgs {
    /// Directory to lint (defaults to the current directory).
    #[arg(long, default_value = ".")]
    pub dir: String,
}

/// Execute a parsed command and return the report to print.
pub fn execute(cmd: RigCommand) -> RigReport {
    match cmd.verb {
        Verb::Run(args) => run_run(args),
        Verb::Lint(args) => run_lint(args),
        Verb::Report => run_report(),
        Verb::Spec => stub_report("spec", "rig spec is not implemented yet (v1)"),
        Verb::Llm => stub_report("llm", "rig llm is not implemented yet (v1)"),
    }
}

fn run_run(args: RunArgs) -> RigReport {
    use rig::report::{finding_id, Finding, Invoke, Kind, Severity};
    use rig::scenario::{ExpectedOutcome, ScenarioKind};
    use rig::verdict::{bucket, Verdict};

    let target = args
        .scenario
        .clone()
        .or(args.dir.clone())
        .unwrap_or_else(|| ".".to_string());
    let mut b = ReportBuilder::new("run", &target);
    b.add_criterion("every required scenario verdicts pass");
    b.add_criterion("every pin gate holds");

    // Discover: a single file or a directory walk.
    let root = std::path::Path::new(&target);
    let discovered = if args.scenario.is_some() {
        match std::fs::read_to_string(root) {
            Ok(text) => vec![rig::discovery::Discovered {
                path: root.to_path_buf(),
                result: rig::scenario::parse_scenario(root, &text),
            }],
            Err(e) => {
                b.tool_error(5, format!("could not read `{target}`: {e}"));
                return b.finalize();
            }
        }
    } else {
        match rig::discovery::discover(root) {
            Ok(d) => d,
            Err(e) => {
                b.tool_error(5, format!("could not walk `{target}`: {e}"));
                return b.finalize();
            }
        }
    };
    if discovered.is_empty() {
        b.tool_error(3, format!("no scenario .toml files under `{target}`"));
        return b.finalize();
    }

    if args.vat {
        return run_via_vat(&discovered, b);
    }

    // Pins + baseline store (cwd-scoped, like .rig/last-report.json).
    let pins = match &args.pins {
        Some(dir) => match rig::pins::load_pins(std::path::Path::new(dir)) {
            Ok(p) => p,
            Err(e) => {
                b.tool_error(5, e);
                return b.finalize();
            }
        },
        None => Vec::new(),
    };
    let mut baselines = rig::pins::BaselineStore::load(std::path::Path::new("."));
    let strict = std::env::var("RIG_STRICT").is_ok_and(|v| v == "1");
    let mut baselines_dirty = false;

    let mut executed = 0usize;
    for d in discovered {
        let rel = d.path.display().to_string();
        let scenario = match d.result {
            Ok(s) => s,
            Err(violations) => {
                for v in violations {
                    b.add_finding(Finding {
                        id: finding_id(Kind::LintError, &rel),
                        severity: Severity::High,
                        kind: Kind::LintError,
                        title: format!("lint: {rel}"),
                        detail: v.message,
                        remediation: "Fix the record so path == record and the schema validates.".into(),
                        invoke: Invoke::command(format!("rig lint --dir {target}")),
                        evidence: serde_json::json!({ "path": rel }),
                    });
                }
                b.scenarios_mut().red += 1;
                continue;
            }
        };

        if scenario.record.expected == ExpectedOutcome::Skip {
            b.scenarios_mut().skip += 1;
            continue;
        }

        let run = if scenario.record.kind == ScenarioKind::Load {
            run_load_scenario(&scenario, &rel)
        } else {
            rig::engine::run_scenario(&scenario)
        };
        executed += 1;

        // Pins gate metrics captured by a scenario whose steps held.
        if run.raw_passed {
            for pin in pins.iter().filter(|p| p.matches(&run.scenario_id)) {
                let Some(value) = run.vars.get_f64(&pin.metric) else {
                    b.add_finding(Finding {
                        id: finding_id(Kind::ScenarioError, &format!("pin/{}/{}", run.scenario_id, pin.metric)),
                        severity: Severity::High,
                        kind: Kind::ScenarioError,
                        title: format!("pin metric `{}` not captured by `{}`", pin.metric, run.scenario_id),
                        detail: "The pin references a metric the scenario never captured/emitted.".into(),
                        remediation: "Capture the metric in the scenario (sample/load emit it) or fix the pin's metric name.".into(),
                        invoke: Invoke::command(format!("rig run --scenario {rel}")),
                        evidence: serde_json::json!({ "pin": pin.issue, "metric": pin.metric }),
                    });
                    continue;
                };
                if args.update_baselines {
                    baselines.record(&run.scenario_id, &pin.metric, value);
                    baselines_dirty = true;
                    b.add_finding(Finding {
                        id: finding_id(Kind::PinMissingBaseline, &format!("recorded/{}/{}", run.scenario_id, pin.metric)),
                        severity: Severity::Info,
                        kind: Kind::PinMissingBaseline,
                        title: format!("baseline recorded: {} {} = {value:.3}", run.scenario_id, pin.metric),
                        detail: "Recorded via --update-baselines; future runs gate against it.".into(),
                        remediation: "None — informational.".into(),
                        invoke: Invoke::command(format!("rig run --scenario {rel} --pins {}", args.pins.as_deref().unwrap_or("."))),
                        evidence: serde_json::json!({ "value": value }),
                    });
                    continue;
                }
                use rig::pins::GateOutcome;
                match rig::pins::gate(pin, &run.scenario_id, value, &baselines) {
                    GateOutcome::Pass => {}
                    GateOutcome::FloorBreach { value, floor } => {
                        b.add_finding(Finding {
                            id: finding_id(Kind::PinRegression, &format!("{}/{}", run.scenario_id, pin.metric)),
                            severity: Severity::High,
                            kind: Kind::PinRegression,
                            title: format!("{} {} = {value:.3} breaches floor {floor:.3}", run.scenario_id, pin.metric),
                            detail: format!("Absolute ceiling breached (pin {}).", pin.issue),
                            remediation: "Investigate the regression; the floor is the promised envelope.".into(),
                            invoke: Invoke::command(format!("rig run --scenario {rel} --pins {}", args.pins.as_deref().unwrap_or("."))),
                            evidence: serde_json::json!({ "value": value, "floor": floor, "pin": pin.issue }),
                        });
                    }
                    GateOutcome::RatchetBreach { value, baseline, limit } => {
                        b.add_finding(Finding {
                            id: finding_id(Kind::PinRegression, &format!("{}/{}", run.scenario_id, pin.metric)),
                            severity: Severity::High,
                            kind: Kind::PinRegression,
                            title: format!("{} {} = {value:.3} regressed past ratchet limit {limit:.3}", run.scenario_id, pin.metric),
                            detail: format!("Baseline {baseline:.3} (pin {}); ratchet allows up to {limit:.3}.", pin.issue),
                            remediation: "Investigate the regression, or re-record baselines deliberately with --update-baselines.".into(),
                            invoke: Invoke::command(format!("rig run --scenario {rel} --pins {}", args.pins.as_deref().unwrap_or("."))),
                            evidence: serde_json::json!({ "value": value, "baseline": baseline, "limit": limit, "pin": pin.issue }),
                        });
                    }
                    GateOutcome::NoBaseline { value } => {
                        b.add_finding(Finding {
                            id: finding_id(Kind::PinMissingBaseline, &format!("{}/{}", run.scenario_id, pin.metric)),
                            severity: if strict { Severity::High } else { Severity::Info },
                            kind: Kind::PinMissingBaseline,
                            title: format!("no baseline for {} {} (measured {value:.3})", run.scenario_id, pin.metric),
                            detail: "The pin has a ratchet but no recorded baseline on this host.".into(),
                            remediation: "Record one: re-run with --update-baselines.".into(),
                            invoke: Invoke::command(format!("rig run --scenario {rel} --pins {} --update-baselines", args.pins.as_deref().unwrap_or("."))),
                            evidence: serde_json::json!({ "value": value, "strict": strict }),
                        });
                    }
                }
            }
        }

        match bucket(scenario.record.expected, run.raw_passed) {
            Verdict::Pass => b.scenarios_mut().pass += 1,
            Verdict::Red => {
                b.scenarios_mut().red += 1;
                if scenario.record.required {
                    b.add_findings(run.findings);
                } else {
                    // Optional scenarios report, never gate: demote to Info.
                    b.add_findings(run.findings.into_iter().map(|mut f| {
                        f.severity = Severity::Info;
                        f.detail = format!("[optional scenario — does not gate] {}", f.detail);
                        f
                    }));
                }
            }
            Verdict::Xfail => {
                b.scenarios_mut().xfail += 1;
                b.add_findings(run.findings.into_iter().map(|mut f| {
                    f.severity = Severity::Info;
                    f.detail = format!("[xfail — known gap, does not gate] {}", f.detail);
                    f
                }));
            }
            Verdict::Xpass => {
                b.scenarios_mut().xpass += 1;
                b.add_finding(Finding {
                    id: finding_id(Kind::ScenarioError, &format!("xpass/{}", run.scenario_id)),
                    severity: Severity::Info,
                    kind: Kind::ScenarioError,
                    title: format!("`{}` passed but is marked xfail", run.scenario_id),
                    detail: "Graduate it: set expected = \"pass\" in the record.".into(),
                    remediation: "Flip the record's expected to pass.".into(),
                    invoke: Invoke::command(format!("rig run --scenario {rel}")),
                    evidence: serde_json::json!({ "scenario": run.scenario_id }),
                });
            }
            Verdict::Skip => b.scenarios_mut().skip += 1,
        }
    }

    let _ = executed;
    if baselines_dirty {
        if let Err(e) = baselines.save() {
            b.add_missing(format!("baseline store not saved: {e}"));
        }
    }
    let report = b.finalize();
    rig::report::persist(&report, std::path::Path::new("."));
    report
}

/// `--vat`: resolve the [vat].runner set of the discovered scenarios, run
/// each runner once through `vat run` (vat owns services/workspace; the
/// runner's cmd re-invokes rig inside the COW clone), and fold every inner
/// report into this one.
fn run_via_vat(discovered: &[rig::discovery::Discovered], mut b: ReportBuilder) -> RigReport {
    use rig::report::{finding_id, Finding, Invoke, Kind, Severity};

    let mut runners: Vec<String> = Vec::new();
    for d in discovered {
        match &d.result {
            Ok(s) => match &s.vat {
                Some(needs) if !needs.runner.is_empty() => {
                    if !runners.contains(&needs.runner) {
                        runners.push(needs.runner.clone());
                    }
                }
                _ => {
                    b.add_missing(format!(
                        "{}: no [vat] runner declared — not runnable under --vat",
                        d.path.display()
                    ));
                }
            },
            Err(_) => {
                b.add_missing(format!(
                    "{}: lint errors — skipped under --vat",
                    d.path.display()
                ));
            }
        }
    }
    if runners.is_empty() {
        b.tool_error(3, "no discovered scenario declares a [vat] runner");
        return b.finalize();
    }

    for runner in &runners {
        let run = match rig::vat::run_runner(runner) {
            Ok(r) => r,
            Err(e) => {
                b.tool_error(4, e);
                return b.finalize();
            }
        };
        b.add_criterion(format!(
            "vat runner `{runner}` (vat {}; services ready: {})",
            run.vat_id,
            run.ready_services.join(", ")
        ));
        let log = match rig::vat::runner_log(&run.vat_id) {
            Ok(l) => l,
            Err(e) => {
                b.tool_error(5, e);
                return b.finalize();
            }
        };
        rig::vat::remove(&run.vat_id);
        match rig::vat::extract_report(&log) {
            Some(inner) => {
                let counts = b.scenarios_mut();
                counts.pass += inner.scenarios.pass;
                counts.red += inner.scenarios.red;
                counts.xfail += inner.scenarios.xfail;
                counts.xpass += inner.scenarios.xpass;
                counts.skip += inner.scenarios.skip;
                b.add_findings(inner.findings);
                for m in inner.completion.missing {
                    b.add_missing(format!("[{runner}] {m}"));
                }
            }
            None => {
                b.add_finding(Finding {
                    id: finding_id(Kind::ScenarioError, &format!("vat/{runner}")),
                    severity: Severity::High,
                    kind: Kind::ScenarioError,
                    title: format!("runner `{runner}` produced no rig report (exit {})", run.exit_code),
                    detail: "The vat runner's cmd must invoke `rig run ...` so its stdout carries one rig.report/1 document.".into(),
                    remediation: format!("Inspect `vat logs {} runner` and fix the runner cmd in vat.toml.", run.vat_id),
                    invoke: Invoke::command(format!("vat logs {} runner", run.vat_id)),
                    evidence: serde_json::json!({ "vat_id": run.vat_id, "exit_code": run.exit_code }),
                });
            }
        }
    }

    let report = b.finalize();
    rig::report::persist(&report, std::path::Path::new("."));
    report
}

/// Drive a `kind = "load"` scenario through the open-loop generator and
/// shape the result like an engine run (metrics land in vars).
fn run_load_scenario(
    scenario: &rig::scenario::Scenario,
    rel: &str,
) -> rig::engine::ScenarioRun {
    use rig::report::{finding_id, Finding, Invoke, Kind, Severity};
    use rig::scenario::load::ACHIEVED_QPS_HONESTY_RATIO;
    use rig::scenario::{scenario_id, VarStore};

    let id = scenario_id(&scenario.record);
    let mut findings = Vec::new();
    let profile = scenario.load.as_ref().expect("lint guarantees [load]");

    // Setup steps run BEFORE the load block (seed corpus, create
    // collections); their captured vars feed the load templates.
    let mut vars = if scenario.steps.is_empty() {
        VarStore::seed(&scenario.env)
    } else {
        let setup = rig::engine::run_scenario(scenario);
        if !setup.raw_passed {
            return setup;
        }
        setup.vars
    };

    let stats = rig::engine::loadgen::run(profile, &vars);
    if let Some(abort) = &stats.abort {
        findings.push(Finding {
            id: finding_id(Kind::ScenarioError, &id),
            severity: Severity::High,
            kind: Kind::ScenarioError,
            title: format!("load scenario `{id}` aborted"),
            detail: abort.clone(),
            remediation: "Fix the load request template/vars and re-run.".into(),
            invoke: Invoke::command(format!("rig run --scenario {rel}")),
            evidence: serde_json::json!({}),
        });
    } else {
        for (key, value) in [
            ("p50_ms", stats.p50_ms),
            ("p99_ms", stats.p99_ms),
            ("error_rate", stats.error_rate),
            ("achieved_qps", stats.achieved_qps),
        ] {
            vars.set(key, serde_json::json!(value));
        }
        let honesty_floor = profile.target_qps as f64 * ACHIEVED_QPS_HONESTY_RATIO;
        if stats.achieved_qps < honesty_floor {
            findings.push(Finding {
                id: finding_id(Kind::LoadHonesty, &id),
                severity: Severity::Medium,
                kind: Kind::LoadHonesty,
                title: format!(
                    "achieved {:.1} qps < {:.0}% of offered {} qps — percentiles are not trustworthy",
                    stats.achieved_qps,
                    ACHIEVED_QPS_HONESTY_RATIO * 100.0,
                    profile.target_qps
                ),
                detail: format!(
                    "{} of {} measured requests failed; an under-achieved schedule means the system (or the generator) saturated below the offered load.",
                    stats.failed, stats.total
                ),
                remediation: "Lower target_qps, raise workers, or treat this as the saturation finding it is.".into(),
                invoke: Invoke::command(format!("rig run --scenario {rel}")),
                evidence: serde_json::json!({
                    "achieved_qps": stats.achieved_qps,
                    "target_qps": profile.target_qps,
                    "failed": stats.failed,
                    "total": stats.total,
                }),
            });
        }
    }

    rig::engine::ScenarioRun {
        raw_passed: findings.is_empty(),
        scenario_id: id,
        findings,
        vars,
        steps_run: 1,
    }
}

fn run_lint(args: LintArgs) -> RigReport {
    use rig::report::{finding_id, Finding, Invoke, Kind, Severity};

    let mut b = ReportBuilder::new("lint", &args.dir);
    b.add_criterion("every scenario record matches its path and schema");

    let root = std::path::Path::new(&args.dir);
    let discovered = match rig::discovery::discover(root) {
        Ok(d) => d,
        Err(e) => {
            b.tool_error(5, format!("could not walk `{}`: {e}", args.dir));
            return b.finalize();
        }
    };
    if discovered.is_empty() {
        b.tool_error(3, format!("no scenario .toml files under `{}`", args.dir));
        return b.finalize();
    }
    let total = discovered.len();
    let mut clean = 0usize;
    for d in discovered {
        match d.result {
            Ok(_) => clean += 1,
            Err(violations) => {
                let rel = d.path.display().to_string();
                for v in violations {
                    b.add_finding(Finding {
                        id: finding_id(Kind::LintError, &rel),
                        severity: Severity::High,
                        kind: Kind::LintError,
                        title: format!("lint: {rel}"),
                        detail: v.message.clone(),
                        remediation: "Fix the scenario record so path == record (dimension = parent dir, case = file stem) and the schema validates, then re-lint.".into(),
                        invoke: Invoke::command(format!("rig lint --dir {}", args.dir)),
                        evidence: serde_json::json!({ "path": rel, "violation": v.message }),
                    });
                }
            }
        }
    }
    b.agent_prompt(format!(
        "rig lint checked {total} scenario file(s) under `{}`: {clean} clean.",
        args.dir
    ));
    b.finalize()
}

fn run_report() -> RigReport {
    let path = std::path::Path::new(".rig/last-report.json");
    match std::fs::read_to_string(path) {
        Ok(json) => match serde_json::from_str::<RigReport>(&json) {
            Ok(report) => report,
            Err(e) => {
                let mut b = ReportBuilder::new("report", path.display().to_string());
                b.tool_error(5, format!("could not parse persisted report: {e}"));
                b.finalize()
            }
        },
        Err(e) => {
            let mut b = ReportBuilder::new("report", path.display().to_string());
            b.tool_error(
                5,
                format!("could not read `.rig/last-report.json`: {e}; run `rig run` first"),
            );
            b.finalize()
        }
    }
}

fn stub_report(verb: &str, msg: &str) -> RigReport {
    let mut b = ReportBuilder::new(verb, ".");
    b.tool_error(3, msg);
    b.finalize()
}

/// Print the report as the single stdout document and return its exit code.
pub fn print_report(report: &RigReport, opts: &OutputOpts) -> i32 {
    let json = if opts.compact {
        serde_json::to_string(report)
    } else {
        serde_json::to_string_pretty(report)
    }
    .expect("report serializes");
    println!("{json}");
    if opts.human {
        eprintln!(
            "rig {} — {} | findings: {} | exit {}",
            report.verb,
            if report.clean { "clean" } else { "NOT clean" },
            report.summary.total,
            report.exit_code
        );
    }
    report.exit_code
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parses_run_with_scenario() {
        let cmd = RigCommand::parse_from(["rig", "run", "--scenario", "a.toml"]);
        match cmd.verb {
            Verb::Run(a) => assert_eq!(a.scenario.as_deref(), Some("a.toml")),
            _ => panic!("expected run"),
        }
    }

    #[test]
    fn run_on_empty_dir_is_usage_tool_error() {
        let tmp = std::env::temp_dir().join(format!("rig-empty-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        let r = run_run(RunArgs {
            dir: Some(tmp.display().to_string()),
            ..Default::default()
        });
        assert_eq!(r.exit_code, 3);
        assert_eq!(r.schema_version, rig::report::SCHEMA_VERSION);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn run_executes_the_demo_fixture_clean() {
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../tests/fixtures/scenarios");
        let r = run_run(RunArgs {
            dir: Some(fixture.display().to_string()),
            ..Default::default()
        });
        assert_eq!(r.exit_code, 0, "agent_prompt: {}", r.agent_prompt);
        assert_eq!(r.scenarios.pass, 1);
    }

    #[test]
    fn scenario_and_dir_conflict() {
        let res = RigCommand::try_parse_from([
            "rig", "run", "--scenario", "a.toml", "--dir", "scenarios",
        ]);
        assert!(res.is_err());
    }
}
