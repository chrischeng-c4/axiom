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
    let target = args
        .scenario
        .clone()
        .or(args.dir.clone())
        .unwrap_or_else(|| ".".to_string());
    let mut b = ReportBuilder::new("run", &target);
    b.add_criterion("every required scenario verdicts pass");
    b.add_criterion("every pin gate holds");
    // Phase 1+ wires discovery/engine here. Until then: usage tool-error.
    b.tool_error(
        3,
        "no scenarios to run yet — scenario discovery lands in Phase 1; pass --scenario <file> once it does",
    );
    b.finalize()
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
    fn run_without_scenarios_is_usage_tool_error() {
        let r = run_run(RunArgs::default());
        assert_eq!(r.exit_code, 3);
        assert_eq!(r.schema_version, rig::report::SCHEMA_VERSION);
    }

    #[test]
    fn scenario_and_dir_conflict() {
        let res = RigCommand::try_parse_from([
            "rig", "run", "--scenario", "a.toml", "--dir", "scenarios",
        ]);
        assert!(res.is_err());
    }
}
