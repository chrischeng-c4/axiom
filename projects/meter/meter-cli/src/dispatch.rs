// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-meter-cli-src-dispatch-rs.md#source
// CODEGEN-BEGIN
//! Shared verb parse + dispatch for the `meter` agent-first CLI.
//!
//! Both the standalone `[[bin]] meter` and the `CliModule` registration path call
//! through here so behavior is identical. Every verb produces a single
//! [`MeterReport`]; [`print_report`] emits it as exactly one JSON document on
//! stdout (diagnostics to stderr). JSON-on-stdout is the UNFLAGGED default;
//! `--human` and `--compact` are the only opt-ins.

use clap::{Args, Parser, Subcommand};

use meter::report::builder::ReportBuilder;
use meter::report::emit::{diag, emit};
use meter::report::envelope::{EnvBlock, MeterReport, RunnerRecord};
use meter::report::producer::IntoFindings;
use meter::report::{persist, schema};

/// Top-level `meter` command tree.
#[derive(Parser, Debug)]
#[command(
    name = "meter",
    version,
    about = "meter — local runtime resource measurement for agents (JSON on stdout by default)",
    disable_help_subcommand = true
)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-meter-cli-src-dispatch-rs.md#source
pub struct MeterCommand {
    #[command(subcommand)]
    pub verb: Verb,
    /// Global output-format opt-ins (JSON-on-stdout stays the default).
    #[command(flatten)]
    pub output: OutputOpts,
}

/// Output-format opt-ins shared by every verb. JSON-on-stdout is the default;
/// these only switch the rendering, never the channel.
#[derive(Args, Debug, Clone, Default)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-meter-cli-src-dispatch-rs.md#source
pub struct OutputOpts {
    /// Render a human-readable summary to stderr in addition to the JSON report.
    #[arg(long, global = true)]
    pub human: bool,
    /// Emit the JSON report as a single dense line (byte-stable golden form).
    #[arg(long, global = true)]
    pub compact: bool,
}

/// The verb set. Every public verb does real work: `test`/`report`/`state`/
/// `spec`/`llm`/`measure`/`profile`/`bench` plus the composite `run` sweep.
#[derive(Subcommand, Debug)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-meter-cli-src-dispatch-rs.md#source
pub enum Verb {
    /// Delegate to cargo nextest/test and FORWARD the child exit code.
    Test {
        /// Args passed through to the runner (after `--`).
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Re-project the persisted `.meter/last-report.json` (read-only, no engine work).
    Report,
    /// Alias of `report`.
    State,
    /// Offline self-describer: `--json-schema` (default) or `--catalog`.
    Spec(SpecArgs),
    /// Offline agent playbook: `guide` (markdown) or `recipes` (json).
    Llm(LlmArgs),
    /// External executable measurement: wall/cpu/RSS and optional sampler data.
    Measure(MeasureArgs),
    /// Source/runtime-aware profiling, or embedded PhaseBreakdown folding.
    Profile(ProfileArgs),
    /// Delegate `cargo bench` and, with a baseline, fold regressions => exit 2.
    Bench(BenchArgs),
    /// Composite sweep: fold delegated test (+ opt-in bench/profile) into ONE worst-wins report.
    Run(RunArgs),
}

/// `meter spec` flags.
#[derive(Args, Debug, Default)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-meter-cli-src-dispatch-rs.md#source
pub struct SpecArgs {
    /// Emit the MeterReport JSON-Schema (this is the default).
    #[arg(long)]
    pub json_schema: bool,
    /// Emit the severity/kind/evidence catalog instead.
    #[arg(long)]
    pub catalog: bool,
}

/// `meter llm` flags.
#[derive(Args, Debug)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-meter-cli-src-dispatch-rs.md#source
pub struct LlmArgs {
    /// Topic: `guide` (markdown playbook) or `recipes` (machine recipes).
    #[arg(default_value = "guide")]
    pub topic: String,
    /// Output format for `recipes`: `json` (default) or `text`.
    #[arg(long, default_value = "json")]
    pub format: String,
}

/// `meter bench` flags.
#[derive(Args, Debug, Default)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-meter-cli-src-dispatch-rs.md#source
pub struct BenchArgs {
    /// Crate path (or `Cargo.toml`) to benchmark via `cargo bench` (default `.`).
    #[arg(long, default_value = ".")]
    pub target: String,
    /// A serialized `RegressionReport` file. When present, its regressions are
    /// folded into findings so the exit-2 path is reachable. Without it, `meter
    /// bench` cannot detect regressions and emits Clean after delegating.
    #[arg(long)]
    pub baseline: Option<String>,
}

/// `meter measure` flags.
///
/// Exactly one target selector picks the executable workload to observe. The
/// default path emits `Vital` findings (cpu_time_ms / wall_time_ms /
/// peak_rss_bytes). `--level sample` also folds platform sampler stacks into
/// ranked `Hotspot` findings.
#[derive(Args, Debug, Default)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-meter-cli-src-dispatch-rs.md#source
pub struct MeasureArgs {
    /// Executable path or command name to run directly.
    pub target: Option<String>,
    /// Measure `cargo run --bin <name>`.
    #[arg(long, group = "measure_target")]
    pub bin: Option<String>,
    /// Measure `cargo run --example <name>`.
    #[arg(long, group = "measure_target")]
    pub example: Option<String>,
    /// Measure `cargo bench --bench <name>`.
    #[arg(long, group = "measure_target")]
    pub bench: Option<String>,
    /// Measure a pre-built executable at this path.
    #[arg(long, group = "measure_target")]
    pub exec: Option<String>,
    /// Measurement level: `off | vitals | sample`.
    #[arg(long)]
    pub level: Option<String>,
    /// Optional cap (seconds) on the measurement window. Default: the window
    /// lasts until the target child exits (a self-terminating target is never
    /// killed mid-run).
    #[arg(long, alias = "duration")]
    pub duration_cap: Option<u64>,
    /// Opaque driver command (spawned via `sh -c` after the target); its exit
    /// ends the measurement window. meter records the command but never
    /// interprets or implements its traffic.
    #[arg(long)]
    pub drive: Option<String>,
    /// Sampling rate in Hz (overrides the default interval).
    #[arg(long)]
    pub hz: Option<u64>,
    /// Fail (exit 1) if any hot spot's self `pct` exceeds this percentage.
    #[arg(long)]
    pub fail_hot: Option<f64>,
    /// Args passed to the sampled target after `--`.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub target_args: Vec<String>,
}

/// `meter profile` flags.
///
/// Profile is source/runtime-aware. Today the shipped embed path folds a
/// serialized `PhaseBreakdown`; direct RS/TS/PY auto-instrumentation returns a
/// clear unsupported message until the probe-injection pipeline is wired.
#[derive(Args, Debug, Default)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-meter-cli-src-dispatch-rs.md#source
pub struct ProfileArgs {
    /// Source/runtime target to profile (RS/TS/PY); auto-instrumentation is not
    /// wired yet, so use `--phases` for shipped embedded data folding.
    pub target: Option<String>,
    /// EMBED path: read a serialized `PhaseBreakdown` -> `BoundaryCost` findings
    /// (no child spawn, no sampler).
    #[arg(long)]
    pub phases: Option<String>,
    /// Profile policy file. Reserved for source instrumentation policy.
    #[arg(long, default_value = "meter.toml")]
    pub config: String,
    /// Args passed to the source/runtime target after `--`.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub target_args: Vec<String>,
}

/// `meter run` flags — the composite sweep.
///
/// The DEFAULT sweep over `--target <crate>` delegates tests as a carried signal.
/// Resource checks are OPT-IN: `bench` with `--baseline`, `profile` with
/// `--profile-bin`/`--profile-example`. Any public sub-verb can be pruned with
/// its `--skip-*` flag; a pruned or un-driven sub-verb is listed in
/// `completion.missing` with a human reason. Status is WORST-WINS over every
/// sub-finding (`ToolError > Regression > Findings > Clean`); a delegated test
/// failure NEVER overrides a meter-native regression.
#[derive(Args, Debug, Default)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-meter-cli-src-dispatch-rs.md#source
pub struct RunArgs {
    /// The crate path the delegated/resource sub-verbs operate on.
    #[arg(long, default_value = ".")]
    pub target: String,
    /// Skip the `test` sub-verb.
    #[arg(long)]
    pub skip_test: bool,
    /// Skip the `bench` sub-verb.
    #[arg(long)]
    pub skip_bench: bool,
    /// Skip the `profile` sub-verb.
    #[arg(long)]
    pub skip_profile: bool,
    /// Run `bench` against this serialized regression baseline.
    #[arg(long)]
    pub baseline: Option<String>,
    /// Run `profile` by sampling `cargo run --bin <name>`.
    #[arg(long)]
    pub profile_bin: Option<String>,
    /// Run `profile` by sampling `cargo run --example <name>`.
    #[arg(long)]
    pub profile_example: Option<String>,
    /// Instrumentation level for the `profile` sub-verb: `off | vitals |
    /// sample | hooks | deep` (CLI > meter.toml in --target > default vitals).
    #[arg(long)]
    pub level: Option<String>,
    /// Opaque driver command bounding the `profile` window (`sh -c`; its exit
    /// ends the window; never interpreted).
    #[arg(long)]
    pub drive: Option<String>,
    /// Optional cap (seconds) on the `profile` window. Default: until the
    /// profiled child exits.
    #[arg(long)]
    pub profile_duration_cap: Option<u64>,
}

/// Outcome of dispatching a verb.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-meter-cli-src-dispatch-rs.md#source
pub struct Dispatched {
    /// The report carrying the process exit code (and, for non-offline verbs,
    /// the document printed to stdout).
    pub report: MeterReport,
    /// `true` when the verb ALREADY wrote its single stdout document (the
    /// offline self-describers `spec`/`llm` print raw schema/markdown directly),
    /// so the caller must NOT also emit the wrapped report.
    pub stdout_written: bool,
}

/// Dispatch a parsed [`MeterCommand`].
///
/// Populator verbs (`test`, stubs, `report`) return a report for the caller to
/// emit; offline verbs (`spec`, `llm`) print their own raw payload and set
/// `stdout_written = true` so stdout stays exactly one JSON/markdown document.
/// The returned report's `exit_code` is the process exit code the caller yields.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-meter-cli-src-dispatch-rs.md#source
pub fn dispatch(cmd: MeterCommand, out: &OutputOpts) -> Dispatched {
    let (report, stdout_written) = match cmd.verb {
        Verb::Test { args } => (run_test(args), false),
        Verb::Report | Verb::State => (run_report(), false),
        Verb::Spec(args) => (run_spec(args, out), true),
        Verb::Llm(args) => run_llm(args, out),
        Verb::Measure(a) => (run_measure(a, out), false),
        Verb::Profile(a) => (run_profile(a, out), false),
        Verb::Bench(a) => (run_bench(a, out), false),
        Verb::Run(a) => (run_run(a), false),
    };
    Dispatched {
        report,
        stdout_written,
    }
}

/// `meter test` — delegate to the runner and forward the child exit code.
fn run_test(passthrough: Vec<String>) -> MeterReport {
    let env = EnvBlock::detect();
    let nextest_present = env.nextest_present;
    let target = if passthrough.is_empty() {
        "<workspace>".to_string()
    } else {
        passthrough.join(" ")
    };

    let mut builder = ReportBuilder::new("test", target);
    builder.with_environment(env);
    builder.add_criterion("delegated test runner exits 0");

    match meter::capture::delegate::delegate_test(&passthrough, nextest_present) {
        Ok(outcome) => {
            builder.with_last_run(outcome.record);
            builder.add_findings(outcome.findings);
            // The ONE verb that yields its exit to the runner: forward the child
            // code verbatim rather than imposing meter's own 0/1/2 verdict.
            builder.forward_exit(outcome.child_exit_code);
        }
        Err(e) => {
            // Runner could not be spawned at all => meter-native ToolError(5, io).
            builder.tool_error(5, format!("could not spawn test runner: {e}"));
        }
    }

    let report = builder.finalize();
    persist_quietly(&report);
    report
}

/// `meter bench` — delegate `cargo bench` for `--target` and, when `--baseline` is
/// supplied, fold a serialized [`RegressionReport`](meter::baseline::RegressionReport)
/// into findings so the exit-2 regression path is reachable from the CLI.
///
/// Behavior:
/// - The benchmark run itself is DELEGATED to `cargo bench` (captured into a
///   `RunnerRecord{kind:"cargo-bench", delegated:true}` with timing). A spawn
///   failure (cargo/target un-invocable) => `ToolError(5)`, never fake-clean.
/// - With `--baseline <file>`: load the serialized regression report and fold
///   `report.into_findings()`. Medium-or-worse regressions elevate the report to
///   `OverallStatus::Regression` (exit 2); minor-only regressions stay exit 1.
///   A load/parse failure of the baseline file => `ToolError(5)`.
/// - Without `--baseline`: no regression detection is possible (a live baseline
///   store is out of this wave's scope), so the report is `Clean` after the
///   delegated run, with an agent_prompt telling the agent to pass a baseline.
///
/// `--human`/`--compact` are honored by the shared [`print_report`] path
/// (stderr human summary / dense JSON); the delegated `cargo bench` output
/// already streams live to the terminal.
fn run_bench(args: BenchArgs, _out: &OutputOpts) -> MeterReport {
    let target = args.target;
    let mut builder = ReportBuilder::new("bench", target.clone());
    builder.with_environment(EnvBlock::detect());
    builder.add_criterion("no medium-or-worse benchmark regressions vs baseline");

    // 1) Delegate the benchmark run to `cargo bench`. A spawn failure is a
    //    harness error (ToolError 5), not a fake-clean result.
    match meter::capture::bench::delegate_bench(&target) {
        Ok(outcome) => {
            builder.with_last_run(outcome.record);
        }
        Err(e) => {
            builder.tool_error(
                5,
                format!("could not run `cargo bench` for `{target}`: {e}"),
            );
            let report = builder.finalize();
            persist_quietly(&report);
            return report;
        }
    }

    // 2) Fold a baseline regression report, if one was supplied.
    match args.baseline {
        Some(baseline_path) => {
            match meter::capture::bench::load_regression_report(&baseline_path) {
                Ok(report) => {
                    // Medium-or-worse regressions elevate to exit 2 in finalize().
                    builder.add_findings(report.into_findings());
                }
                Err(e) => {
                    builder.tool_error(5, format!("baseline error: {e}"));
                }
            }
        }
        None => {
            // No baseline => no regression detection possible this wave.
        }
    }

    let mut report = builder.finalize();
    if report.clean && report.last_run.is_some() {
        report.agent_prompt = format!(
            "meter bench ran `cargo bench` for `{target}` but no baseline was supplied, so no \
             regression could be detected. Re-run with `meter bench --target {target} --baseline \
             <serialized-RegressionReport.json>` to enable the exit-2 regression gate."
        );
    }
    persist_quietly(&report);
    report
}

/// `meter measure` — external executable measurement.
///
/// Effective level = CLI `--level` > built-in `vitals`:
/// - `off` => no measurement (Clean report saying so).
/// - `vitals` (default) => spawn + wait the target and emit `Vital` findings
///   (cpu_time_ms / wall_time_ms / peak_rss_bytes via `wait4`+`rusage`) with no
///   sampler attach (~zero overhead).
/// - `sample` => vitals PLUS the platform stack sampler: ranked `Hotspot`
///   findings (sorted self_ns desc) and a `.meter/<target>.collapsed` artifact
///   whose path rides the agent_prompt. `--human` ALSO writes an SVG side file.
///
/// The measurement window defaults to UNTIL CHILD EXIT; `--duration-cap`
/// bounds it; `--drive <cmd>` runs an opaque driver whose exit ends the window
/// (server-shaped targets compose with an external driver — meter never
/// generates load).
///
/// Exit codes: 0 ran clean; 1 `--fail-hot` or measurement quality surfaced a
/// finding; 3 usage; 4 sampler backend unavailable; 5 spawn/build io.
fn run_measure(args: MeasureArgs, out: &OutputOpts) -> MeterReport {
    use meter::capture::fold;
    use meter::capture::sampler::SampleError;
    use meter::capture::vitals::{self, Level};

    let target = match measure_target(&args) {
        Ok(target) => target,
        Err(message) => {
            let mut builder = ReportBuilder::new("measure", "<unset>");
            builder.with_environment(EnvBlock::detect());
            builder.tool_error(3, message);
            let report = builder.finalize();
            persist_quietly(&report);
            return report;
        }
    };

    let label = target.label();
    let mut builder = ReportBuilder::new("measure", label.clone());
    builder.with_environment(EnvBlock::detect());

    let cli_level = match args.level.as_deref().map(Level::parse).transpose() {
        Ok(l) => l,
        Err(e) => {
            builder.tool_error(3, e);
            let report = builder.finalize();
            persist_quietly(&report);
            return report;
        }
    };
    let level = cli_level.unwrap_or(Level::Vitals);

    match level {
        Level::Off => {
            builder.add_criterion("level off: no measurement requested");
            let mut report = builder.finalize();
            report.agent_prompt = format!(
                "meter measure did not measure `{label}`: the effective level is `off`. \
                 Raise the level to `vitals` or `sample` to measure."
            );
            persist_quietly(&report);
            return report;
        }
        Level::Hooks | Level::Deep => {
            builder.tool_error(
                3,
                format!(
                    "meter measure supports `off`, `vitals`, and `sample`; level `{}` belongs \
                     to source-aware `meter profile` instrumentation.",
                    level.as_str()
                ),
            );
            let report = builder.finalize();
            persist_quietly(&report);
            return report;
        }
        Level::Vitals | Level::Sample => {}
    }

    builder.add_criterion("process vitals captured (cpu/wall/peak RSS)");
    if level >= Level::Sample {
        builder.add_criterion("a dominant hot spot was located and ranked");
    }

    let wopts = vitals::WindowOpts {
        attach_sampler: level >= Level::Sample,
        duration_cap_secs: args.duration_cap,
        drive: args.drive.clone(),
        hz: args.hz,
    };

    let started_at = chrono::Utc::now();
    diag(format!(
        "meter measure: measuring `{label}` at level `{}` ({}; this builds/runs the target)...",
        level.as_str(),
        match (args.duration_cap, &args.drive) {
            (_, Some(_)) => "window = driver lifetime".to_string(),
            (Some(s), None) => format!("window capped at {s}s"),
            (None, None) => "window = until child exit".to_string(),
        }
    ));

    match vitals::capture_window(&target, &args.target_args, &wopts) {
        Ok(outcome) => {
            let finished_at = chrono::Utc::now();
            let command = outcome
                .sample
                .as_ref()
                .map(|s| s.command.clone())
                .unwrap_or_else(|| vec![label.clone()]);
            builder.with_last_run(RunnerRecord {
                command,
                kind: "capture".into(),
                started_at,
                finished_at: Some(finished_at),
                exit_code: outcome.child_exit,
                duration_ms: Some(outcome.vitals.wall_time_ms),
                delegated: true,
            });

            let mut findings = vitals::vitals_findings(&outcome.vitals, &label);

            let mut sample_note = String::new();
            if let Some(run) = &outcome.sample {
                let hot = fold::fold_hotspots(&run.stacks, run.effective_hz, args.fail_hot);
                sample_note = format!(
                    " Ranked {} hot spot(s) from {} samples via {} (kind `hotspot`, sorted self_ns desc).",
                    hot.len(),
                    run.stacks.iter().map(|s| s.count).sum::<u64>(),
                    run.backend,
                );
                match vitals::write_collapsed(&run.stacks, &label) {
                    Ok(p) => sample_note.push_str(&format!(" Collapsed stacks: {}.", p.display())),
                    Err(e) => diag(format!(
                        "meter measure: could not write collapsed artifact: {e}"
                    )),
                }
                if out.human {
                    if let Some(p) = write_human_svg(&run.stacks, &label) {
                        sample_note.push_str(&format!(" Flamegraph SVG: {p}."));
                    }
                }
                findings.extend(hot);
            }

            let breached = findings.iter().any(|f| {
                matches!(
                    f.severity,
                    meter::report::Severity::High | meter::report::Severity::Medium
                )
            });
            builder.informational_findings_are_clean();
            builder.add_findings(findings);
            builder.forward_exit(if breached { 1 } else { 0 });

            let mut report = builder.finalize();
            let v = &outcome.vitals;
            report.agent_prompt = format!(
                "meter measure measured `{label}` at level `{}`: cpu_time {} ms, wall {} ms, \
                 peak RSS {} bytes (kind `vital`).{}",
                level.as_str(),
                v.cpu_time_ms,
                v.wall_time_ms,
                v.peak_rss_bytes,
                sample_note,
            );
            persist_quietly(&report);
            report
        }
        Err(e) => {
            // Map the sampler error to the right exit code.
            let (code, hint) = match &e {
                SampleError::NoBackend(_) => (
                    4,
                    "no stack sampler backend is available; on macOS ensure `/usr/bin/sample` \
                     exists, on Linux install `perf` (linux-tools)",
                ),
                SampleError::Spawn(_) => (
                    5,
                    "the profiling target could not be built or spawned; check the target name and \
                     that it compiles",
                ),
                SampleError::Sampler(_) => (
                    4,
                    "the sampler ran but produced no usable stacks; try `--duration-cap` with a \
                     longer window or a longer-running target",
                ),
            };
            builder.tool_error(code, format!("{e} ({hint})"));
            let report = builder.finalize();
            persist_quietly(&report);
            report
        }
    }
}

fn measure_target(args: &MeasureArgs) -> Result<meter::capture::sampler::Target, String> {
    use meter::capture::sampler::Target;
    let selector_count = args.target.iter().count()
        + args.bin.iter().count()
        + args.example.iter().count()
        + args.bench.iter().count()
        + args.exec.iter().count();
    if selector_count != 1 {
        return Err(
            "specify exactly one measure target: <target>, --bin, --example, --bench, or --exec"
                .to_string(),
        );
    }
    if let Some(target) = &args.target {
        Ok(Target::Exec(std::path::PathBuf::from(target)))
    } else if let Some(bin) = &args.bin {
        Ok(Target::Bin(bin.clone()))
    } else if let Some(example) = &args.example {
        Ok(Target::Example(example.clone()))
    } else if let Some(bench) = &args.bench {
        Ok(Target::Bench(bench.clone()))
    } else if let Some(exec) = &args.exec {
        Ok(Target::Exec(std::path::PathBuf::from(exec)))
    } else {
        unreachable!("selector_count checked")
    }
}

/// `meter profile` — source/runtime-aware profiling.
fn run_profile(args: ProfileArgs, _out: &OutputOpts) -> MeterReport {
    if let Some(phases_path) = args.phases.clone() {
        return run_profile_phases(&phases_path);
    }

    let target = args.target.unwrap_or_else(|| "<unset>".to_string());
    let mut builder = ReportBuilder::new("profile", target.clone());
    builder.with_environment(EnvBlock::detect());
    builder.tool_error(
        3,
        format!(
            "source/runtime auto-instrumentation for `{target}` is not wired yet; use \
             `meter profile --phases <PhaseBreakdown.json>` for embedded meter data, or \
             `meter measure <target>` for external executable measurement"
        ),
    );
    let report = builder.finalize();
    persist_quietly(&report);
    report
}

/// `meter profile --phases <file>` — the EMBED path. Reads a serialized
/// `PhaseBreakdown` and folds it into `BoundaryCost` findings (no spawn).
fn run_profile_phases(path: &str) -> MeterReport {
    use meter::performance::profiler::PhaseBreakdown;

    let mut builder = ReportBuilder::new("profile", format!("phases:{path}"));
    builder.with_environment(EnvBlock::detect());
    builder.add_criterion("phase breakdown folded into boundary-cost findings");

    let raw = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            builder.tool_error(5, format!("could not read phases file `{path}`: {e}"));
            let report = builder.finalize();
            persist_quietly(&report);
            return report;
        }
    };
    let breakdown: PhaseBreakdown = match serde_json::from_str(&raw) {
        Ok(b) => b,
        Err(e) => {
            builder.tool_error(
                3,
                format!("could not parse `{path}` as a PhaseBreakdown: {e}"),
            );
            let report = builder.finalize();
            persist_quietly(&report);
            return report;
        }
    };

    builder.add_findings(breakdown.into_findings());
    // Boundary-cost findings are purely informational (they describe where time
    // goes, they are not failures), so the embed path is exit 0 / clean.
    builder.informational_findings_are_clean();
    builder.forward_exit(0);
    let mut report = builder.finalize();
    report.agent_prompt = format!(
        "meter profile --phases folded `{path}` into {} boundary-cost finding(s) (kind \
         `boundary_cost`). Inspect `findings[]` to see where the operation spends time.",
        report.findings.len()
    );
    persist_quietly(&report);
    report
}

/// Write a flamegraph SVG side artifact for `--human` and return its path.
/// Best-effort: a failure is a stderr diag and yields `None` (the JSON report is
/// unaffected — the SVG is NEVER part of the document).
fn write_human_svg(stacks: &[meter::capture::sampler::FoldedStack], label: &str) -> Option<String> {
    use meter::capture::fold;
    use meter::performance::profiler::generate_flamegraph_svg;

    let data = fold::to_flamegraph(stacks);
    if !data.has_data() {
        return None;
    }
    let safe: String = label
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    let path = std::env::temp_dir().join(format!("meter-measure-{safe}.svg"));
    let path_str = path.display().to_string();
    match generate_flamegraph_svg(
        &data.folded_stacks,
        &format!("meter measure: {label}"),
        &path_str,
    ) {
        Ok(()) => {
            diag(format!("meter measure: wrote flamegraph SVG to {path_str}"));
            Some(path_str)
        }
        Err(e) => {
            diag(format!(
                "meter measure: could not write flamegraph SVG: {e}"
            ));
            None
        }
    }
}

/// `meter report`/`state` — re-project the persisted report with zero engine work.
fn run_report() -> MeterReport {
    if let Some(report) = persist::read_last_report() {
        return report;
    }
    // No cache: emit a Clean report whose prompt tells the agent to run a populator.
    let mut builder = ReportBuilder::new("report", "<no-cache>");
    builder.with_environment(EnvBlock::detect());
    builder.add_criterion("a populator has written .meter/last-report.json");
    let mut report = builder.finalize();
    report.agent_prompt =
        "No persisted report at .meter/last-report.json. Run a populator verb first, e.g. \
         `meter test`, `meter bench`, `meter measure`, or `meter profile`, then re-run `meter report`."
            .to_string();
    report
}

/// `meter spec` — offline self-describer. Emits raw schema/catalog JSON to stdout
/// (NOT wrapped in a MeterReport) so agents can `jq .title`/`.properties` directly.
/// Returns the MeterReport that carries only the exit code.
fn run_spec(args: SpecArgs, out: &OutputOpts) -> MeterReport {
    let value = if args.catalog {
        schema::catalog()
    } else {
        // `--json-schema` is the default, including when no flag is passed.
        schema::json_schema()
    };
    let rendered = if out.compact {
        serde_json::to_string(&value)
    } else {
        serde_json::to_string_pretty(&value)
    }
    .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"));
    println!("{rendered}");

    // spec always exits 0; build a minimal Clean report only to carry the code.
    let mut builder = ReportBuilder::new("spec", "-");
    builder.with_environment(EnvBlock::detect());
    builder.finalize()
}

/// `meter llm` — offline agent playbook. `guide` => markdown to stdout; `recipes`
/// => machine recipes (json default). Like `spec`, the payload is the raw
/// document, not a wrapped report. Returns `(report, stdout_written)`; a bad
/// topic writes NOTHING to stdout and yields a wrapped tool-error report.
fn run_llm(args: LlmArgs, out: &OutputOpts) -> (MeterReport, bool) {
    match args.topic.as_str() {
        "guide" => {
            print!("{}", guide_markdown());
        }
        "recipes" => {
            if args.format == "text" {
                print!("{}", recipes_text());
            } else {
                let recipes = recipes_json();
                let rendered = if out.compact {
                    serde_json::to_string(&recipes)
                } else {
                    serde_json::to_string_pretty(&recipes)
                }
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"));
                println!("{rendered}");
            }
        }
        other => {
            // Bad topic => usage error (exit 3); caller emits the wrapped report.
            let mut builder = ReportBuilder::new("llm", other.to_string());
            builder.with_environment(EnvBlock::detect());
            builder.tool_error(
                3,
                format!("unknown llm topic `{other}`; use `guide` or `recipes`"),
            );
            return (builder.finalize(), false);
        }
    }
    let mut builder = ReportBuilder::new("llm", args.topic);
    builder.with_environment(EnvBlock::detect());
    (builder.finalize(), true)
}

/// `meter run` — the composite sweep. Folds delegated `test` by default plus the
/// opt-in `bench`/`profile` resource sub-verbs into ONE worst-wins
/// [`MeterReport`]. A delegated test failure is recorded in `last_run` but never
/// overrides a meter-native regression; un-run sub-verbs are listed in
/// `completion.missing` with a human reason. The folding/status derivation is
/// owned by [`meter::capture::run::run_sweep`]; this arm only maps the CLI args.
fn run_run(args: RunArgs) -> MeterReport {
    let nextest_present = EnvBlock::detect().nextest_present;
    let opts = meter::capture::run::RunOptions {
        target: args.target,
        skip_test: args.skip_test,
        skip_bench: args.skip_bench,
        skip_profile: args.skip_profile,
        baseline: args.baseline,
        profile_bin: args.profile_bin,
        profile_example: args.profile_example,
        level: args.level,
        drive: args.drive,
        profile_duration_cap: args.profile_duration_cap,
        nextest_present,
    };
    let report = meter::capture::run::run_sweep(&opts);
    persist_quietly(&report);
    report
}

/// Persist a populator report best-effort; a write failure is a stderr diag only.
fn persist_quietly(report: &MeterReport) {
    if let Err(e) = persist::write_last_report(report) {
        diag(format!(
            "warning: could not write .meter/last-report.json: {e}"
        ));
    }
}

/// Emit the report as the single stdout JSON document, plus an optional
/// human-readable stderr summary.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-meter-cli-src-dispatch-rs.md#source
pub fn print_report(report: &MeterReport, out: &OutputOpts) {
    emit(report, out.compact);
    if out.human {
        diag(human_summary(report));
    }
}

/// A short human-readable summary (stderr only, never stdout).
fn human_summary(report: &MeterReport) -> String {
    let s = &report.summary;
    format!(
        "meter {} -> {} (exit {}); findings: {} (crit {} / high {} / med {} / low {} / info {})\n{}",
        report.verb,
        if report.clean { "clean" } else { "issues" },
        report.exit_code,
        s.total,
        s.critical,
        s.high,
        s.medium,
        s.low,
        s.info,
        report.agent_prompt,
    )
}

/// The markdown agent playbook for `meter llm guide`.
fn guide_markdown() -> String {
    r#"# meter — local resource-measurement playbook

`meter` helps an agent measure where local Rust work spends time and resources.
It is not a test framework, not a runtime/environment manager, and not a security
scanner. Use vat to prepare the local env/runner; use meter inside or after that
run to identify CPU hotspots, phase/boundary cost, benchmark regressions, and
delegated test failures.

## Agent loop
- Start broad with `meter run --target .`.
- Read `schema_version`, `status.state`, `exit_code`, and `completion.missing`.
- Triage `findings[]` in order. Every finding carries `id`, `severity`, `kind`,
  `remediation`, `invoke`, and `evidence`.
- Use targeted verbs to deepen the signal: `measure` for external executable
  cpu/wall/RSS and sampled hot spots, `profile` for embedded/source-aware phase
  cost, `bench` for regressions, and `test` for delegated test failures.
- Re-run the smallest command from the finding's `invoke.command` after a fix.

## Defaults
- Every populator verb prints ONE JSON `MeterReport` to stdout. Diagnostics go to
  stderr.
- `--human` adds a stderr summary; `--compact` emits dense single-line JSON.
- Pin on `schema_version == "meter.report/1"`.
- `meter.toml` is profile policy only: it may declare `level =
  "off|vitals|sample|hooks|deep"`. Project thresholds and workload policy belong
  to EC/arena/rig/vat configs, not meter.

## Verbs
- `meter test [-- <runner args>]` — delegate + forward exit. Failures =>
  `findings[].kind == test_failure`.
- `meter bench [--target <path>] [--baseline <file>]` — delegate `cargo bench`;
  with a baseline, medium-or-worse regressions => `findings[].kind ==
  regression` and exit 2. No baseline => Clean after delegation.
- `meter measure [<target>|--bin|--example|--bench|--exec <t>] [--level off|vitals|sample]
  [--duration-cap <s>] [--drive <cmd>] [--hz <r>] [--fail-hot <pct>]` — capture-mode
  measurement. Level `vitals` (the default) emits `findings[].kind == vital`
  (cpu_time_ms / wall_time_ms / peak_rss_bytes, zero overhead); `sample` adds
  ranked `hotspot` findings (sorted self_ns desc) plus a `.meter/*.collapsed`
  artifact. The window lasts until the child exits (`--duration-cap` bounds it;
  `--drive` runs an opaque driver whose exit ends the window — meter never
  generates load). `--human` also writes an SVG side artifact.
- `meter profile [<source-target>] [--phases <file>]` — source/runtime-aware
  profiling. The shipped path is `--phases`, which reads a recorded
  PhaseBreakdown => `boundary_cost`. Direct RS/TS/PY auto-instrumentation returns
  a clear unsupported message until probe injection is wired.
- `meter run [--target <path>] [--skip-test|--skip-bench|--skip-profile]
  [--baseline <f>] [--profile-bin|--profile-example <n>] [--level <l>] [--drive <cmd>]` — composite sweep:
  delegated test by default; bench/profile are opt-in. Folds every sub-verb into
  ONE worst-wins report (`ToolError > Regression > Findings > Clean`); a
  delegated test failure never overrides a regression. Un-run sub-verbs are
  listed in `completion.missing`.
- `meter report` (alias `state`) — re-project `.meter/last-report.json`, no engine
  work.
- `meter spec [--json-schema|--catalog]` — offline self-describer; raw JSON on
  stdout.
- `meter llm guide | recipes [--format json]` — this playbook / machine recipes.

## Boundaries
- meter does not replace test runners; it delegates to them and packages findings.
- meter does not prepare runtime environments; vat owns local env/runner setup.
- meter does not claim security coverage; old audit/fuzz internals are carried
  legacy code, not public meter capability.
- meter does not auto-fix code; it reports evidence and next commands.

## Exit codes
0 clean · 1 findings · 2 regression · 3 usage · 4 missing-tool · 5 io.
For `meter test` the exit code is the FORWARDED child code.
"#
    .to_string()
}

/// Machine recipes for `meter llm recipes` (single source of truth with `spec`).
fn recipes_json() -> serde_json::Value {
    serde_json::json!({
        "schema_version": meter::report::SCHEMA_VERSION,
        "positioning": {
            "role": "local_agent_resource_meter",
            "does": [
                "turn local measurement, profile, benchmark, and delegated test signals into actionable resource findings",
                "tell an agent where CPU time, boundary cost, or benchmark regression appears and which command to run next"
            ],
            "does_not": [
                "prepare runtime environments",
                "replace cargo nextest/cargo test",
                "claim security coverage",
                "fix code automatically"
            ]
        },
        "triage_loop": [
            "start with `meter run --target .`",
            "read `.status.state`, `.exit_code`, `.completion.missing`, and `.findings[]`",
            "follow each finding's `.invoke.command` for the smallest next check",
            "re-run the smallest command after a fix"
        ],
        "recipes": [
            {
                "goal": "start a broad local problem-finding sweep",
                "command": "meter run --target .",
                "read": ".status.state, .exit_code, .completion.missing, .findings[] | {id, severity, kind, remediation, invoke, evidence}"
            },
            {
                "goal": "run the test suite and forward the runner exit",
                "command": "meter test -- -p meter --lib",
                "read": ".last_run.delegated, .last_run.exit_code, .findings[] | select(.kind==\"test_failure\")"
            },
            {
                "goal": "run benchmarks and detect regressions against a saved baseline",
                "command": "meter bench --target . --baseline baseline.json",
                "read": ".status.state, .exit_code, .findings[] | select(.kind==\"regression\")"
            },
            {
                "goal": "measure one run's cpu/wall/peak-RSS vitals (zero overhead)",
                "command": "meter measure --bin my-cli --level vitals",
                "read": ".findings[] | select(.kind==\"vital\") | .evidence"
            },
            {
                "goal": "find where a binary spends its time (ranked hot spots)",
                "command": "meter measure --example profile_target --level sample",
                "read": ".environment.sampler_backend, .findings[] | select(.kind==\"hotspot\") | .evidence"
            },
            {
                "goal": "inspect the last populator report without re-running",
                "command": "meter report",
                "read": ".status.state, .exit_code, .findings"
            },
            {
                "goal": "discover the report schema offline",
                "command": "meter spec --json-schema",
                "read": ".title, .properties, .$defs"
            },
            {
                "goal": "discover the closed severity/kind/evidence sets",
                "command": "meter spec --catalog",
                "read": ".severities, .kinds"
            }
        ]
    })
}

/// Plain-text recipes for `meter llm recipes --format text`.
fn recipes_text() -> String {
    "meter run --target .          # broad local problem-finding sweep\n\
     meter measure --bin my-cli       # external cpu/wall/RSS measurement\n\
     meter profile --phases p.json    # fold embedded phase profile data\n\
     meter test -- -p meter --lib     # delegate + forward exit\n\
     meter report                  # re-project .meter/last-report.json\n\
     meter spec --json-schema      # offline schema\n\
     meter spec --catalog          # severity/kind/evidence catalog\n\
     meter llm guide               # offline agent playbook\n"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn parse(argv: &[&str]) -> MeterCommand {
        MeterCommand::parse_from(argv)
    }

    #[test]
    fn spec_defaults_to_json_schema() {
        let cmd = parse(&["meter", "spec"]);
        let d = dispatch(cmd, &OutputOpts::default());
        // spec writes its own stdout document and always exits 0.
        assert!(d.stdout_written);
        assert_eq!(d.report.exit_code, 0);
        assert!(d.report.clean);
    }

    #[test]
    fn report_without_cache_is_clean_with_prompt() {
        // Run in a temp cwd so there's no .meter cache.
        let dir = tempfile_dir();
        let _g = ChdirGuard::enter(&dir);
        let cmd = parse(&["meter", "report"]);
        let report = dispatch(cmd, &OutputOpts::default()).report;
        assert!(report.clean);
        assert!(report.agent_prompt.contains("populator"));
    }

    #[test]
    fn run_args_default_target() {
        // `meter run` parses with sensible defaults: target `.`, no skip flags,
        // and no opt-in resource drivers.
        let cmd = parse(&["meter", "run"]);
        match cmd.verb {
            Verb::Run(a) => {
                assert_eq!(a.target, ".");
                assert!(!a.skip_test && !a.skip_bench && !a.skip_profile);
                assert!(a.baseline.is_none());
                assert!(a.profile_bin.is_none());
                assert!(a.profile_example.is_none());
            }
            _ => panic!("expected run verb"),
        }
    }

    #[test]
    fn run_parses_skip_and_opt_in_flags() {
        let cmd = parse(&[
            "meter",
            "run",
            "--target",
            "/x/crate",
            "--skip-test",
            "--baseline",
            "/b.json",
            "--profile-bin",
            "worker",
        ]);
        match cmd.verb {
            Verb::Run(a) => {
                assert_eq!(a.target, "/x/crate");
                assert!(a.skip_test);
                assert_eq!(a.baseline.as_deref(), Some("/b.json"));
                assert_eq!(a.profile_bin.as_deref(), Some("worker"));
            }
            _ => panic!("expected run verb"),
        }
    }

    #[test]
    fn run_pruning_everything_is_clean_composite() {
        // Skipping every sub-verb runs no engine work => a Clean `run` report
        // whose completion.missing lists the pruned verbs. Exercises the wired
        // arm end-to-end without spawning cargo.
        let cmd = parse(&[
            "meter",
            "run",
            "--target",
            "/tmp/meter-cli-run-target",
            "--skip-test",
            "--skip-bench",
            "--skip-profile",
        ]);
        let report = dispatch(cmd, &OutputOpts::default()).report;
        assert_eq!(report.verb, "run");
        assert_eq!(report.exit_code, 0);
        assert!(report.clean);
        assert_eq!(report.completion.missing.len(), 3);
    }

    #[test]
    fn measure_without_target_is_usage_error() {
        // No <target>/--bin/--example/--bench/--exec => usage error (exit 3),
        // never a fake-clean result.
        let cmd = parse(&["meter", "measure"]);
        let report = dispatch(cmd, &OutputOpts::default()).report;
        assert_eq!(report.exit_code, 3);
        assert_eq!(report.verb, "measure");
    }

    #[test]
    fn profile_phases_parses_and_is_embed_path() {
        let cmd = parse(&["meter", "profile", "--phases", "/tmp/b.json"]);
        match cmd.verb {
            Verb::Profile(a) => {
                assert_eq!(a.phases.as_deref(), Some("/tmp/b.json"));
                assert!(a.target.is_none());
            }
            _ => panic!("expected profile verb"),
        }
    }

    #[test]
    fn profile_phases_folds_breakdown_into_boundary_cost() {
        // A serialized PhaseBreakdown loads on the embed path and yields
        // boundary_cost findings deterministically (no spawn / no sampler).
        use meter::performance::profiler::PhaseBreakdown;
        use std::collections::HashMap;
        let mut times: HashMap<String, Vec<u64>> = HashMap::new();
        times.insert("RustConvert".to_string(), vec![3_000_000, 3_000_000]);
        times.insert("PythonExtract".to_string(), vec![1_500_000]);
        let pb = PhaseBreakdown::from_times(times, 2, 9_000_000);
        let json = serde_json::to_string(&pb).unwrap();
        let mut path = std::env::temp_dir();
        path.push(format!("meter-phases-{}.json", std::process::id()));
        std::fs::write(&path, &json).unwrap();

        let cmd = parse(&["meter", "profile", "--phases", path.to_str().unwrap()]);
        let report = dispatch(cmd, &OutputOpts::default()).report;
        let _ = std::fs::remove_file(&path);

        assert!(!report.findings.is_empty());
        assert_eq!(report.findings[0].kind, meter::report::Kind::BoundaryCost);
        // The report carries NO flamegraph_svg key (JSON-only contract).
        let v = serde_json::to_value(&report).unwrap();
        assert!(v.get("flamegraph_svg").is_none());
    }

    #[test]
    fn measure_parses_capture_flags() {
        let cmd = parse(&[
            "meter",
            "measure",
            "--example",
            "profile_target",
            "--level",
            "sample",
            "--duration-cap",
            "2",
            "--drive",
            "scripts/drive.sh --n 100",
            "--hz",
            "250",
            "--fail-hot",
            "40",
        ]);
        match cmd.verb {
            Verb::Measure(a) => {
                assert_eq!(a.example.as_deref(), Some("profile_target"));
                assert_eq!(a.level.as_deref(), Some("sample"));
                assert_eq!(a.duration_cap, Some(2));
                assert_eq!(a.drive.as_deref(), Some("scripts/drive.sh --n 100"));
                assert_eq!(a.hz, Some(250));
                assert_eq!(a.fail_hot, Some(40.0));
            }
            _ => panic!("expected measure verb"),
        }
    }

    #[test]
    fn measure_window_defaults_to_until_exit() {
        // No --duration-cap => None => the window lasts until child exit.
        let cmd = parse(&["meter", "measure", "--exec", "/bin/ls"]);
        match cmd.verb {
            Verb::Measure(a) => {
                assert_eq!(a.duration_cap, None);
                assert_eq!(a.level, None);
                assert_eq!(a.drive, None);
            }
            _ => panic!("expected measure verb"),
        }
    }

    #[test]
    fn profile_help_exposes_no_load_generation_flags() {
        // Charter: meter never grows load-generation knobs.
        use clap::CommandFactory;
        let mut help = Vec::new();
        MeterCommand::command().write_long_help(&mut help).unwrap();
        let help = String::from_utf8(help).unwrap();
        for forbidden in ["--rps", "--concurrency", "--connections", "--qps"] {
            assert!(
                !help.contains(forbidden),
                "load-generation flag `{forbidden}` must not exist"
            );
        }
    }

    #[test]
    fn measure_bad_level_is_usage_error() {
        let cmd = parse(&["meter", "measure", "--exec", "/bin/ls", "--level", "turbo"]);
        let report = dispatch(cmd, &OutputOpts::default()).report;
        assert_eq!(report.exit_code, 3);
    }

    #[test]
    fn measure_level_hooks_is_source_profile_usage_error() {
        // hooks/deep parse but error BEFORE any spawn, pointing at the L3/L4 epic.
        for lvl in ["hooks", "deep"] {
            let cmd = parse(&["meter", "measure", "--exec", "/bin/ls", "--level", lvl]);
            let report = dispatch(cmd, &OutputOpts::default()).report;
            assert_eq!(report.exit_code, 3, "level {lvl} must be a usage error");
            let msg = serde_json::to_string(&report).unwrap();
            assert!(
                msg.contains("source-aware"),
                "error must route to profile: {msg}"
            );
        }
    }

    #[test]
    fn measure_level_off_measures_nothing_and_is_clean() {
        let dir = test_subdir("level-off");
        let _g = ChdirGuard::enter(&dir);
        let cmd = parse(&["meter", "measure", "--exec", "/bin/ls", "--level", "off"]);
        let report = dispatch(cmd, &OutputOpts::default()).report;
        assert_eq!(report.exit_code, 0);
        assert!(report.clean);
        assert!(report.findings.is_empty());
        assert!(report.agent_prompt.contains("off"));
    }

    #[test]
    fn measure_vitals_level_emits_vital_findings_without_sampler() {
        // Default level (no flag, no meter.toml) = vitals: a real child runs to
        // completion and yields kind=vital evidence; no hotspot findings.
        let dir = test_subdir("vitals-default");
        let _g = ChdirGuard::enter(&dir);
        let cmd = parse(&["meter", "measure", "--exec", "/bin/ls"]);
        let report = dispatch(cmd, &OutputOpts::default()).report;
        assert_eq!(report.exit_code, 0, "prompt: {}", report.agent_prompt);
        assert!(report.clean, "vital findings are informational");
        let vitals: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.kind == meter::report::Kind::Vital)
            .collect();
        assert_eq!(vitals.len(), 1);
        assert!(vitals[0].evidence.get("peak_rss_bytes").is_some());
        assert!(report
            .findings
            .iter()
            .all(|f| f.kind != meter::report::Kind::Hotspot));
        // The child's exit rode into last_run.
        assert_eq!(report.last_run.as_ref().unwrap().exit_code, Some(0));
    }

    #[test]
    fn measure_ignores_meter_toml_gate_policy() {
        // meter.toml no longer carries resource gates; measure is a direct
        // external observation command and does not load project gate policy.
        let dir = test_subdir("gate-ignored");
        std::fs::write(
            dir.join("meter.toml"),
            "level = \"vitals\"\n[gate]\nmax_peak_rss_mb = 1\n",
        )
        .unwrap();
        let _g = ChdirGuard::enter(&dir);
        let cmd = parse(&["meter", "measure", "--exec", "/bin/ls"]);
        let report = dispatch(cmd, &OutputOpts::default()).report;
        std::fs::remove_file(dir.join("meter.toml")).ok();
        assert_eq!(report.exit_code, 0);
        assert!(report.clean);
        assert!(report
            .findings
            .iter()
            .all(|f| f.severity == meter::report::Severity::Info));
    }

    #[test]
    fn profile_source_target_is_usage_until_auto_instrumentation_exists() {
        let cmd = parse(&["meter", "profile", "src/main.rs"]);
        let report = dispatch(cmd, &OutputOpts::default()).report;
        assert_eq!(report.exit_code, 3);
        assert!(report.agent_prompt.contains("auto-instrumentation"));
    }

    #[test]
    fn bench_target_defaults_to_dot_and_baseline_optional() {
        let cmd = parse(&["meter", "bench"]);
        match cmd.verb {
            Verb::Bench(a) => {
                assert_eq!(a.target, ".");
                assert!(a.baseline.is_none());
            }
            _ => panic!("expected bench verb"),
        }
    }

    #[test]
    fn bench_parses_target_and_baseline() {
        let cmd = parse(&[
            "meter",
            "bench",
            "--target",
            "/x/crate",
            "--baseline",
            "/b.json",
        ]);
        match cmd.verb {
            Verb::Bench(a) => {
                assert_eq!(a.target, "/x/crate");
                assert_eq!(a.baseline.as_deref(), Some("/b.json"));
            }
            _ => panic!("expected bench verb"),
        }
    }

    #[test]
    fn bench_with_missing_baseline_is_tool_error() {
        // A baseline path that doesn't exist => ToolError(5) (never fake-clean),
        // reachable without actually running cargo bench end-to-end? No — the
        // delegate runs first. This test only asserts the parse + that a
        // nonexistent baseline path is rejected by the loader.
        let r = meter::capture::bench::load_regression_report("/nonexistent/meter/b.json");
        assert!(r.is_err());
    }

    #[test]
    fn llm_guide_is_clean() {
        let cmd = parse(&["meter", "llm", "guide"]);
        let d = dispatch(cmd, &OutputOpts::default());
        assert!(d.stdout_written);
        assert!(d.report.clean);
        assert_eq!(d.report.exit_code, 0);
    }

    #[test]
    fn llm_guide_mentions_resource_meter_contract() {
        let guide = guide_markdown();
        assert!(guide.contains("local resource-measurement"));
        assert!(guide.contains("meter run --target ."));
        assert!(guide.contains("findings[]"));
        assert!(guide.contains("remediation"));
        assert!(guide.contains("invoke"));
        assert!(guide.contains("evidence"));
        assert!(guide.contains("does not prepare runtime environments"));
        assert!(guide.contains("does not replace test runners"));
        assert!(guide.contains("does not claim security coverage"));
    }

    #[test]
    fn llm_recipes_include_resource_meter_positioning() {
        let recipes = recipes_json();
        assert_eq!(recipes["positioning"]["role"], "local_agent_resource_meter");
        let rendered = serde_json::to_string(&recipes).unwrap();
        assert!(rendered.contains("meter run --target ."));
        assert!(rendered.contains("remediation"));
        assert!(rendered.contains("invoke"));
        assert!(rendered.contains("evidence"));
        assert!(!rendered.contains("meter audit"));
        assert!(!rendered.contains("meter fuzz"));
    }

    #[test]
    fn llm_bad_topic_is_usage_error() {
        let cmd = parse(&["meter", "llm", "bogus"]);
        let d = dispatch(cmd, &OutputOpts::default());
        assert!(!d.stdout_written);
        assert_eq!(d.report.exit_code, 3);
    }

    // --- minimal cwd guard so report-cache tests are hermetic ---

    fn tempfile_dir() -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("meter-cli-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&p);
        p
    }

    /// A per-test scratch dir so chdir-based tests cannot contaminate each
    /// other's meter.toml / .meter cache.
    fn test_subdir(tag: &str) -> std::path::PathBuf {
        let p = tempfile_dir().join(tag);
        let _ = std::fs::create_dir_all(&p);
        p
    }

    struct ChdirGuard {
        prev: std::path::PathBuf,
        _lock: std::sync::MutexGuard<'static, ()>,
    }
    impl ChdirGuard {
        fn enter(dir: &std::path::Path) -> Self {
            // cwd is process-global: serialize every chdir-based test.
            static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
            let lock = LOCK.lock().unwrap_or_else(|e| e.into_inner());
            let prev = std::env::current_dir().unwrap();
            std::env::set_current_dir(dir).unwrap();
            Self { prev, _lock: lock }
        }
    }
    impl Drop for ChdirGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.prev);
        }
    }
}
// CODEGEN-END
