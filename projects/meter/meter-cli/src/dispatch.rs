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
/// `spec`/`llm`/`profile`/`bench` plus the composite `run` sweep.
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
    /// Capture-mode ranked hot-spot profiling (C1). Default = ranked Hotspot
    /// JSON; `--phases <file>` = embed BoundaryCost; `--human` also writes an SVG.
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

/// `meter profile` flags.
///
/// Exactly one target selector picks the workload to sample (`--bin` /
/// `--example` / `--bench` / `--exec`). The DEFAULT path samples the workload
/// and emits ranked `Hotspot` findings. `--phases <file>` switches to the EMBED
/// path: it reads a serialized `PhaseBreakdown` and emits `BoundaryCost`
/// findings without spawning anything.
#[derive(Args, Debug, Default)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-meter-cli-src-dispatch-rs.md#source
pub struct ProfileArgs {
    /// Sample `cargo run --bin <name>`.
    #[arg(long, group = "profile_target")]
    pub bin: Option<String>,
    /// Sample `cargo run --example <name>`.
    #[arg(long, group = "profile_target")]
    pub example: Option<String>,
    /// Sample `cargo bench --bench <name>`.
    #[arg(long, group = "profile_target")]
    pub bench: Option<String>,
    /// Sample a pre-built executable at this path.
    #[arg(long, group = "profile_target")]
    pub exec: Option<String>,
    /// EMBED path: read a serialized `PhaseBreakdown` -> `BoundaryCost` findings
    /// (no child spawn, no sampler). Mutually exclusive with a sampler target.
    #[arg(long, group = "profile_target")]
    pub phases: Option<String>,
    /// Sampling duration in seconds (default 3).
    #[arg(long, default_value_t = 3)]
    pub duration: u64,
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
    /// Sampling duration (seconds) for the `profile` sub-verb (default 3).
    #[arg(long, default_value_t = 3)]
    pub profile_duration: u64,
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
        Some(baseline_path) => match meter::capture::bench::load_regression_report(&baseline_path) {
            Ok(report) => {
                // Medium-or-worse regressions elevate to exit 2 in finalize().
                builder.add_findings(report.into_findings());
            }
            Err(e) => {
                builder.tool_error(5, format!("baseline error: {e}"));
            }
        },
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

/// `meter profile` — the C1 capture-mode hot-spot profiler.
///
/// Two paths, mutually exclusive (clap group `profile_target`):
/// - DEFAULT (capture): a `--bin`/`--example`/`--bench`/`--exec` target is
///   spawned under the platform stack sampler for `--duration` (at `--hz` if
///   given); the folded stacks are ranked into `Hotspot` findings, PRE-SORTED by
///   `self_ns` desc. This is JSON-ONLY: the report carries NO `flamegraph_svg`
///   key. `--human` ALSO writes a `<target>.svg` (path surfaced in `agent_prompt`
///   /stderr) but never embeds SVG bytes in the JSON.
/// - EMBED (`--phases <file>`): reads a serialized
///   [`PhaseBreakdown`](meter::performance::profiler::PhaseBreakdown) and folds it
///   into `BoundaryCost` findings — no child spawn, no sampler. Deterministic.
///
/// Exit codes: 0 ran clean (hot spots are Info by default); 1 if any hot spot's
/// `pct` exceeds `--fail-hot`; 4 sampler backend unavailable (NEVER fake-clean);
/// 5 child spawn/build io error.
fn run_profile(args: ProfileArgs, out: &OutputOpts) -> MeterReport {
    use meter::capture::fold;
    use meter::capture::sampler::{self, SampleError, Target};

    // --- EMBED path: --phases reads a recorded breakdown, no spawn. ---
    if let Some(phases_path) = args.phases.clone() {
        return run_profile_phases(&phases_path);
    }

    // --- CAPTURE path: pick exactly one sampler target. ---
    let target = match (&args.bin, &args.example, &args.bench, &args.exec) {
        (Some(b), None, None, None) => Target::Bin(b.clone()),
        (None, Some(e), None, None) => Target::Example(e.clone()),
        (None, None, Some(b), None) => Target::Bench(b.clone()),
        (None, None, None, Some(p)) => Target::Exec(std::path::PathBuf::from(p)),
        _ => {
            // No target / ambiguous selector => usage error (exit 3).
            let mut builder = ReportBuilder::new("profile", "<unset>");
            builder.with_environment(EnvBlock::detect());
            builder.tool_error(
                3,
                "specify exactly one of --bin, --example, --bench, --exec, or --phases",
            );
            let report = builder.finalize();
            persist_quietly(&report);
            return report;
        }
    };

    let label = target.label();
    let mut builder = ReportBuilder::new("profile", label.clone());
    builder.with_environment(EnvBlock::detect());
    builder.add_criterion("a dominant hot spot was located and ranked");

    let started_at = chrono::Utc::now();
    diag(format!(
        "meter profile: sampling `{label}` for {}s (this builds + runs the target)...",
        args.duration
    ));
    let run = sampler::sample_target(&target, &args.target_args, args.duration, args.hz);

    match run {
        Ok(run) => {
            let finished_at = chrono::Utc::now();
            builder.with_last_run(RunnerRecord {
                command: run.command.clone(),
                kind: "sampler".into(),
                started_at,
                finished_at: Some(finished_at),
                exit_code: None,
                duration_ms: Some(args.duration * 1000),
                delegated: true,
            });

            // Fold -> ranked Hotspot findings (the default stdout, JSON-only).
            let findings = fold::fold_hotspots(&run.stacks, run.effective_hz, args.fail_hot);
            let n = findings.len();
            // A hot spot that breached `--fail-hot` is marked High by the fold
            // step. Hot spots are INFORMATIONAL by default: locating where time
            // goes is success, not failure. So the profile run is exit 0 (clean)
            // UNLESS a `--fail-hot` threshold was breached OR the fold step
            // escalated a measurement-quality warning (Medium — e.g.
            // unsymbolicated frames dominate a stripped target), in which case
            // it is exit 1. Forwarding the exit keeps the full ranked findings
            // in the report while honoring the contract (0 ran clean / 1
            // fail-hot or unusable measurement).
            let breached = findings.iter().any(|f| {
                matches!(
                    f.severity,
                    meter::report::Severity::High | meter::report::Severity::Medium
                )
            });
            builder.informational_findings_are_clean();
            builder.add_findings(findings);
            builder.forward_exit(if breached { 1 } else { 0 });

            // --human: ALSO write the SVG side artifact (never in the JSON).
            let svg_note = if out.human {
                write_human_svg(&run.stacks, &label)
            } else {
                None
            };

            let mut report = builder.finalize();
            report.agent_prompt =
                format!(
                "meter profile sampled `{label}` ({} samples via {}) and ranked {n} hot spot(s) by \
                 self time. Inspect `findings[]` (kind `hotspot`, sorted self_ns desc); the top \
                 entry is the dominant leaf.{}",
                run.stacks.iter().map(|s| s.count).sum::<u64>(),
                run.backend,
                svg_note.map(|p| format!(" Flamegraph SVG: {p}")).unwrap_or_default(),
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
                    "the sampler ran but produced no usable stacks; try a longer --duration or a \
                     longer-running target",
                ),
            };
            builder.tool_error(code, format!("{e} ({hint})"));
            let report = builder.finalize();
            persist_quietly(&report);
            report
        }
    }
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
    let path = std::env::temp_dir().join(format!("meter-profile-{safe}.svg"));
    let path_str = path.display().to_string();
    match generate_flamegraph_svg(
        &data.folded_stacks,
        &format!("meter profile: {label}"),
        &path_str,
    ) {
        Ok(()) => {
            diag(format!("meter profile: wrote flamegraph SVG to {path_str}"));
            Some(path_str)
        }
        Err(e) => {
            diag(format!("meter profile: could not write flamegraph SVG: {e}"));
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
         `meter test`, `meter bench`, or `meter profile`, then re-run `meter report`."
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
        profile_duration: args.profile_duration,
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
- Use targeted verbs to deepen the signal: `profile` for CPU hot spots or
  boundary cost, `bench` for regressions, and `test` for delegated test failures.
- Re-run the smallest command from the finding's `invoke.command` after a fix.

## Defaults
- Every populator verb prints ONE JSON `MeterReport` to stdout. Diagnostics go to
  stderr.
- `--human` adds a stderr summary; `--compact` emits dense single-line JSON.
- Pin on `schema_version == "meter.report/1"`.

## Verbs
- `meter test [-- <runner args>]` — delegate + forward exit. Failures =>
  `findings[].kind == test_failure`.
- `meter bench [--target <path>] [--baseline <file>]` — delegate `cargo bench`;
  with a baseline, medium-or-worse regressions => `findings[].kind ==
  regression` and exit 2. No baseline => Clean after delegation.
- `meter profile [--bin|--example|--bench|--exec <t>] [--duration <s>] [--hz <r>] [--fail-hot <pct>]` —
  capture-mode CPU stack sampler; ranked `findings[].kind == hotspot` (sorted
  self_ns desc), JSON-only. `--phases <file>` reads a recorded PhaseBreakdown =>
  `boundary_cost`. `--human` also writes an SVG side artifact.
- `meter run [--target <path>] [--skip-test|--skip-bench|--skip-profile]
  [--baseline <f>] [--profile-bin|--profile-example <n>]` — composite sweep:
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
                "turn local Rust profile, benchmark, and delegated test signals into actionable resource findings",
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
                "goal": "find where a binary spends its time (ranked hot spots)",
                "command": "meter profile --example profile_target --duration 3",
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
    fn profile_without_target_is_usage_error() {
        // No --bin/--example/--bench/--exec/--phases => usage error (exit 3),
        // never a fake-clean result.
        let cmd = parse(&["meter", "profile"]);
        let report = dispatch(cmd, &OutputOpts::default()).report;
        assert_eq!(report.exit_code, 3);
        assert_eq!(report.verb, "profile");
    }

    #[test]
    fn profile_phases_parses_and_is_embed_path() {
        // --phases groups with the target selector and routes to the embed path.
        let cmd = parse(&["meter", "profile", "--phases", "/tmp/b.json"]);
        match cmd.verb {
            Verb::Profile(a) => {
                assert_eq!(a.phases.as_deref(), Some("/tmp/b.json"));
                assert!(a.bin.is_none());
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
    fn profile_parses_capture_flags() {
        let cmd = parse(&[
            "meter",
            "profile",
            "--example",
            "profile_target",
            "--duration",
            "2",
            "--hz",
            "250",
            "--fail-hot",
            "40",
        ]);
        match cmd.verb {
            Verb::Profile(a) => {
                assert_eq!(a.example.as_deref(), Some("profile_target"));
                assert_eq!(a.duration, 2);
                assert_eq!(a.hz, Some(250));
                assert_eq!(a.fail_hot, Some(40.0));
            }
            _ => panic!("expected profile verb"),
        }
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

    struct ChdirGuard {
        prev: std::path::PathBuf,
    }
    impl ChdirGuard {
        fn enter(dir: &std::path::Path) -> Self {
            let prev = std::env::current_dir().unwrap();
            std::env::set_current_dir(dir).unwrap();
            Self { prev }
        }
    }
    impl Drop for ChdirGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.prev);
        }
    }
}
// CODEGEN-END
