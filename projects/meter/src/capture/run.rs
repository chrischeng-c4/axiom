// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-src-capture-run-rs.md#source
// CODEGEN-BEGIN
//! `meter run` composite — the dual-mode sweep that folds every sub-verb into ONE
//! [`MeterReport`].
//!
//! `meter run` is the agent's "measure the local workload" verb: it delegates
//! tests as a carried signal and folds optional benchmark/profile resource
//! findings into a single report. Status is WORST-WINS over every sub-finding via
//! the shared [`ReportBuilder::finalize`] ladder (`ToolError > Regression >
//! Findings > Clean`); the per-sub-verb logic lives in sibling capture modules
//! ([`super::delegate`], [`super::bench`], [`super::sampler`]/[`super::fold`]) so
//! `run` is pure composition — it adds no new engine work.
//!
//! ## The §4 rule (test failure does NOT override a regression)
//!
//! A delegated `test` child's non-zero exit is recorded in `last_run`, and its
//! failures become `TestFailure` findings — a plain `Findings` rung (exit 1). A
//! meter-native `Regression` finding is exit 2. Because the composite folds every
//! finding into ONE builder and lets `finalize()` derive the status from the
//! finding SET (never `forward_exit`), a `Regression` automatically outranks a
//! test failure. `run` deliberately does NOT forward the test child's exit code
//! (unlike the standalone `meter test`), so the worst-wins ladder is honored.
//!
//! ## Soft tool-unavailability is `completion.missing`, not a dominating ToolError
//!
//! A single sub-verb tool being unavailable (e.g. no sampler backend or an
//! unreadable baseline) is recorded as a `completion.missing` entry with a human
//! reason and the sweep CONTINUES with the other sub-verbs, rather than
//! collapsing the whole report into a dominating `ToolError`. This keeps the
//! composite useful on partial environments. Only a sub-verb that PRODUCES
//! findings or a regression moves the status off Clean.

use crate::report::builder::ReportBuilder;
use crate::report::envelope::{EnvBlock, MeterReport};
use crate::report::producer::IntoFindings;

/// Options controlling which sub-verbs the composite sweep runs.
///
/// Defaults run delegated `test` over [`Self::target`]. `bench`/`profile` are
/// opt-in: each runs only when its driving input is supplied. Every public
/// sub-verb can be pruned with its `skip_*` flag; a pruned or un-driven sub-verb
/// is recorded in `completion.missing` with a human reason.
#[derive(Debug, Clone, Default)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-run-rs.md#source
pub struct RunOptions {
    /// The crate path the delegated/resource sub-verbs operate on.
    pub target: String,
    /// Skip the `test` sub-verb.
    pub skip_test: bool,
    /// Skip the `bench` sub-verb.
    pub skip_bench: bool,
    /// Skip the `profile` sub-verb.
    pub skip_profile: bool,
    /// `bench` runs only when a serialized regression baseline is supplied here.
    pub baseline: Option<String>,
    /// `profile` runs only when a `--profile-bin` is supplied here.
    pub profile_bin: Option<String>,
    /// `profile` runs only when a `--profile-example` is supplied here.
    pub profile_example: Option<String>,
    /// Instrumentation level for the `profile` sub-verb (CLI > meter.toml in
    /// the target dir > built-in `vitals`).
    pub level: Option<String>,
    /// Opaque driver command bounding the `profile` window (its exit ends the
    /// window; never interpreted).
    pub drive: Option<String>,
    /// Optional cap (seconds) on the `profile` window; `None` = until exit.
    pub profile_duration_cap: Option<u64>,
    /// Whether the delegated `test` runner should prefer nextest.
    pub nextest_present: bool,
}

/// Run the composite sweep and fold every sub-verb result into ONE report.
///
/// The status is derived worst-wins by `finalize()` over the union of all
/// sub-findings; the test child's exit is recorded in `last_run` but never
/// forwarded (so a regression outranks a test failure per §4). Un-run sub-verbs
/// are listed in `completion.missing`.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-run-rs.md#source
pub fn run_sweep(opts: &RunOptions) -> MeterReport {
    let mut builder = ReportBuilder::new("run", opts.target.clone());
    builder.with_environment(EnvBlock::detect());

    // --- test (default; bare-crate, delegated) ---
    if opts.skip_test {
        builder.add_missing("test: skipped via --skip-test");
    } else {
        builder.add_criterion("test: delegated runner exits 0");
        // The composite runs the runner against the target crate's manifest.
        let passthrough = test_passthrough(&opts.target);
        match super::delegate::delegate_test(&passthrough, opts.nextest_present) {
            Ok(outcome) => {
                // Record the delegated child (with its FORWARDED exit) in
                // last_run, and fold its TestFailure findings — but do NOT
                // forward_exit: the worst-wins ladder must keep a regression
                // (exit 2) above a test failure (exit 1). §4.
                builder.with_last_run(outcome.record);
                builder.add_findings(outcome.findings);
            }
            Err(e) => {
                // Soft: the runner could not be spawned at all; record the gap.
                builder.add_missing(format!("test: could not spawn the test runner ({e})"));
            }
        }
    }

    // --- bench (opt-in: needs --baseline) ---
    if opts.skip_bench {
        builder.add_missing("bench: skipped via --skip-bench");
    } else if let Some(baseline) = &opts.baseline {
        builder.add_criterion("bench: no medium-or-worse regressions vs baseline");
        match super::bench::load_regression_report(baseline) {
            Ok(report) => {
                builder.add_findings(report.into_findings());
            }
            Err(e) => {
                builder.add_missing(format!("bench: baseline error ({e})"));
            }
        }
    } else {
        builder.add_missing("bench: no --baseline given");
    }

    // --- profile (opt-in: needs --profile-bin/--profile-example) ---
    if opts.skip_profile {
        builder.add_missing("profile: skipped via --skip-profile");
    } else if let Some(reason) = profile_target_missing(opts) {
        builder.add_missing(reason);
    } else {
        builder.add_criterion("profile: a dominant hot spot was located and ranked");
        run_profile_sub(opts, &mut builder);
    }

    let mut report = builder.finalize();
    report.agent_prompt = compose_prompt(&report);
    report
}

/// Build the `cargo test`/`nextest` passthrough for the target crate. If the
/// target points at a real directory, scope the run to that crate's manifest;
/// otherwise pass nothing (the runner uses the ambient workspace).
fn test_passthrough(target: &str) -> Vec<String> {
    let path = std::path::Path::new(target);
    let manifest = if path.file_name().and_then(|n| n.to_str()) == Some("Cargo.toml") {
        Some(path.to_path_buf())
    } else if path.is_dir() && path.join("Cargo.toml").is_file() {
        Some(path.join("Cargo.toml"))
    } else {
        None
    };
    match manifest {
        Some(m) => vec!["--manifest-path".into(), m.display().to_string()],
        None => Vec::new(),
    }
}

/// Reason string when the `profile` sub-verb has no driving target, or `None`
/// when a profile target IS available.
fn profile_target_missing(opts: &RunOptions) -> Option<String> {
    if opts.profile_bin.is_some() || opts.profile_example.is_some() {
        None
    } else {
        Some("profile: no --profile-bin/--profile-example given".to_string())
    }
}

/// Run the `profile` sub-verb under the single-knob measurement contract:
/// resolve the level (CLI > meter.toml in the target dir > default `vitals`),
/// run one capture window (until-exit / cap / driver lifetime), and fold the
/// `vital` findings (plus ranked hot spots at level `sample`). meter.toml
/// `[gate]` ceilings adjudicate via High findings on the worst-wins ladder.
fn run_profile_sub(opts: &RunOptions, builder: &mut ReportBuilder) {
    use super::fold;
    use super::sampler::Target;
    use super::vitals::{self, Level};

    let target = if let Some(bin) = &opts.profile_bin {
        Target::Bin(bin.clone())
    } else if let Some(example) = &opts.profile_example {
        Target::Example(example.clone())
    } else {
        // Guarded by profile_target_missing() above; unreachable in practice.
        builder.add_missing("profile: no --profile-bin/--profile-example given");
        return;
    };

    // The measurement contract lives with the measured project (the --target dir).
    let config = match vitals::MeterConfig::load(std::path::Path::new(&opts.target)) {
        Ok(c) => c,
        Err(e) => {
            builder.add_missing(format!("profile: {e}"));
            return;
        }
    };
    let cli_level = match opts.level.as_deref().map(Level::parse).transpose() {
        Ok(l) => l,
        Err(e) => {
            builder.add_missing(format!("profile: {e}"));
            return;
        }
    };
    let level = vitals::resolve_level(cli_level, config.as_ref());
    match level {
        Level::Off => {
            builder.add_missing("profile: level off (no measurement requested)");
            return;
        }
        Level::Hooks | Level::Deep => {
            builder.add_missing(format!(
                "profile: level {} not implemented yet (meter L3/L4 instrumentation epic, WI #4)",
                level.as_str()
            ));
            return;
        }
        Level::Vitals | Level::Sample => {}
    }
    let gate = config.map(|c| c.gate).unwrap_or_default();

    let wopts = vitals::WindowOpts {
        attach_sampler: level >= Level::Sample,
        duration_cap_secs: opts.profile_duration_cap,
        drive: opts.drive.clone(),
        hz: None,
    };
    match vitals::capture_window(&target, &[], &wopts) {
        Ok(outcome) => {
            let escalate = format!(
                "meter run --target {} {} --level sample",
                opts.target,
                if let Some(b) = &opts.profile_bin {
                    format!("--profile-bin {b}")
                } else {
                    format!(
                        "--profile-example {}",
                        opts.profile_example.as_deref().unwrap_or("<target>")
                    )
                }
            );
            builder.add_findings(vitals::vitals_findings(
                &outcome.vitals,
                &target.label(),
                &gate,
                &escalate,
            ));
            if let Some(run) = &outcome.sample {
                // Hot spots are informational by default (no --fail-hot in the
                // composite), so they are Info findings and never move the
                // status off Clean on their own — but they ARE folded in.
                let findings = fold::fold_hotspots(&run.stacks, run.effective_hz, None);
                builder.add_findings(findings);
                if let Err(e) = vitals::write_collapsed(&run.stacks, &target.label()) {
                    builder.add_missing(format!("profile: collapsed artifact not written ({e})"));
                }
            }
        }
        Err(e) => {
            // Soft: no sampler / un-spawnable target => record the gap, keep going.
            builder.add_missing(format!("profile: capture unavailable ({e})"));
        }
    }
}

/// Compose the agent next-action prompt for the folded report, surfacing the
/// worst-wins outcome plus the coverage gaps in `completion.missing`.
fn compose_prompt(report: &MeterReport) -> String {
    let verdict = if report.clean {
        "found no issues".to_string()
    } else {
        format!(
            "surfaced {} finding(s) (worst-wins status `{}`, exit {})",
            report.summary.total,
            status_word(report),
            report.exit_code
        )
    };
    let missing = if report.completion.missing.is_empty() {
        String::new()
    } else {
        format!(
            " Sub-verbs not run (coverage gaps): {}.",
            report.completion.missing.join("; ")
        )
    };
    format!(
        "meter run swept `{}` (delegated test by default; bench/profile opt-in) and {verdict}. \
         Inspect `findings[]` and run each finding's `invoke.command`.{missing}",
        report.target
    )
}

/// The status word for the prompt.
fn status_word(report: &MeterReport) -> &'static str {
    use crate::report::OverallStatus;
    match report.status {
        OverallStatus::Clean => "clean",
        OverallStatus::Findings { .. } => "findings",
        OverallStatus::Regression { .. } => "regression",
        OverallStatus::ToolError { .. } => "tool_error",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passthrough_scopes_to_manifest_for_a_crate_dir() {
        // The meter crate itself is a real dir with a Cargo.toml; the passthrough
        // must scope the runner to its manifest.
        let meter_dir = env!("CARGO_MANIFEST_DIR");
        let pt = test_passthrough(meter_dir);
        assert_eq!(pt.len(), 2);
        assert_eq!(pt[0], "--manifest-path");
        assert!(pt[1].ends_with("Cargo.toml"));
    }

    #[test]
    fn test_passthrough_is_empty_for_a_non_crate_path() {
        let pt = test_passthrough("/nonexistent/not/a/crate");
        assert!(pt.is_empty());
    }

    #[test]
    fn profile_target_missing_when_no_selector() {
        let opts = RunOptions::default();
        assert!(profile_target_missing(&opts).is_some());
    }

    #[test]
    fn profile_target_present_when_bin_given() {
        let opts = RunOptions {
            profile_bin: Some("my-bin".into()),
            ..Default::default()
        };
        assert!(profile_target_missing(&opts).is_none());
    }

    #[test]
    fn skipping_every_verb_yields_clean_with_all_missing() {
        // Pruning every sub-verb runs no engine work: a Clean report whose
        // completion.missing lists every pruned verb. This exercises the
        // composite folding/missing aggregation without spawning anything.
        let opts = RunOptions {
            target: "/tmp/meter-run-test-target".into(),
            skip_test: true,
            skip_bench: true,
            skip_profile: true,
            ..Default::default()
        };
        let report = run_sweep(&opts);
        assert_eq!(report.verb, "run");
        assert_eq!(report.exit_code, 0);
        assert!(report.clean);
        assert!(report.completion.clean);
        assert_eq!(report.summary.total, 0);
        // All public sub-verbs are recorded as missing (skipped).
        assert_eq!(report.completion.missing.len(), 3);
        for v in ["test:", "bench:", "profile:"] {
            assert!(
                report.completion.missing.iter().any(|m| m.starts_with(v)),
                "missing should list `{v}`: {:?}",
                report.completion.missing
            );
        }
        // The agent prompt surfaces the coverage gaps.
        assert!(report.agent_prompt.contains("coverage gaps"));
    }
}
// CODEGEN-END
