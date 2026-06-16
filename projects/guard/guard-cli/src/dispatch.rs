// SPEC-MANAGED: projects/guard/tech-design/semantic/source/projects-guard-guard-cli-src-dispatch-rs.md#rust-source-unit
// CODEGEN-BEGIN
use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand, ValueEnum};
use guard::{
    scan_paths_with_options, Baseline, GuardConfig, GuardReport, PolicyProfile, ScanOptions,
};

#[derive(Parser, Debug)]
#[command(
    name = "guard",
    version,
    about = "guard — security posture gate (JSON on stdout by default)",
    disable_help_subcommand = true
)]
/// @spec projects/guard/tech-design/semantic/source/projects-guard-guard-cli-src-dispatch-rs.md#source
pub struct GuardCommand {
    #[command(subcommand)]
    pub verb: Verb,
    #[command(flatten)]
    pub output: OutputOpts,
}

#[derive(Args, Debug, Clone, Default)]
/// @spec projects/guard/tech-design/semantic/source/projects-guard-guard-cli-src-dispatch-rs.md#source
pub struct OutputOpts {
    /// Emit the report as a single dense line.
    #[arg(long, global = true)]
    pub compact: bool,
    /// Render a short human-readable summary to stderr in addition to JSON.
    #[arg(long, global = true)]
    pub human: bool,
}

#[derive(Subcommand, Debug)]
/// @spec projects/guard/tech-design/semantic/source/projects-guard-guard-cli-src-dispatch-rs.md#source
pub enum Verb {
    /// Scan for static security findings, gating only on findings absent from
    /// the accepted baseline. Zero-args reads `guard.toml`.
    Scan(ScanArgs),
    /// Snapshot the current findings into `.guard/baseline.json` so they stop
    /// gating; only later, newly introduced findings will gate.
    Accept(ScanArgs),
    /// Re-project `.guard/last-report.json` without scanning.
    Report,
}

#[derive(Args, Debug, Default)]
/// @spec projects/guard/tech-design/semantic/source/projects-guard-guard-cli-src-dispatch-rs.md#source
pub struct ScanArgs {
    /// File or directory to scan. Overrides `guard.toml` `paths`; defaults to
    /// the configured paths (or `.`) when omitted.
    pub path: Option<PathBuf>,
    /// Policy profile override (else `guard.toml` `profile`, else baseline static).
    #[arg(long, value_enum)]
    pub profile: Option<ProfileArg>,
    /// Do not persist `.guard/last-report.json`.
    #[arg(long)]
    pub no_persist: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
/// @spec projects/guard/tech-design/semantic/source/projects-guard-guard-cli-src-dispatch-rs.md#source
pub enum ProfileArg {
    BaselineStatic,
    SecurityLint,
    Strict,
}

/// @spec projects/guard/tech-design/semantic/source/projects-guard-guard-cli-src-dispatch-rs.md#source
impl From<ProfileArg> for PolicyProfile {
    fn from(value: ProfileArg) -> Self {
        match value {
            ProfileArg::BaselineStatic => PolicyProfile::BaselineStatic,
            ProfileArg::SecurityLint => PolicyProfile::SecurityLint,
            ProfileArg::Strict => PolicyProfile::Strict,
        }
    }
}

/// @spec projects/guard/tech-design/semantic/source/projects-guard-guard-cli-src-dispatch-rs.md#source
pub fn dispatch(cmd: GuardCommand) -> GuardReport {
    let cwd = PathBuf::from(".");
    match cmd.verb {
        Verb::Scan(args) => run_scan(&cwd, args, false),
        Verb::Accept(args) => run_scan(&cwd, args, true),
        Verb::Report => GuardReport::read_last(&cwd).unwrap_or_else(|e| {
            GuardReport::tool_error(
                "report",
                ".",
                5,
                format!("no readable .guard/last-report.json: {e}"),
            )
        }),
    }
}

/// Resolve config + CLI overrides, scan, then gate against the baseline. When
/// `accept` is set, the current findings are first snapshotted into the
/// baseline so the resulting report is clean.
fn run_scan(cwd: &Path, args: ScanArgs, accept: bool) -> GuardReport {
    let config = GuardConfig::load_from(cwd);
    let profile = args
        .profile
        .map(PolicyProfile::from)
        .or_else(|| config.profile())
        .unwrap_or(PolicyProfile::BaselineStatic);
    let no_persist = args.no_persist || config.no_persist.unwrap_or(false);
    let targets = match args.path {
        Some(path) => vec![path],
        None => config.scan_paths("."),
    };

    let mut options = ScanOptions::default();
    options.profile = profile;
    let mut report = scan_paths_with_options(&targets, options);

    if accept {
        let baseline = Baseline::from_report(&report);
        let _ = baseline.save(cwd);
        report.verb = "accept".to_string();
        report.apply_baseline(&baseline);
    } else {
        let baseline = Baseline::load(cwd);
        report.apply_baseline(&baseline);
    }

    if !no_persist {
        report.persist(cwd);
    }
    report
}

/// @spec projects/guard/tech-design/semantic/source/projects-guard-guard-cli-src-dispatch-rs.md#source
pub fn print_report(report: &GuardReport, out: &OutputOpts) {
    let json = if out.compact {
        serde_json::to_string(report)
    } else {
        serde_json::to_string_pretty(report)
    }
    .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"));
    println!("{json}");
    if out.human {
        eprintln!(
            "guard {} -> exit {} (security_findings={}, new_findings={})",
            report.verb,
            report.exit_code,
            report.summary.security_findings,
            report.summary.new_findings
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scan_args(path: &Path) -> ScanArgs {
        ScanArgs {
            path: Some(path.to_path_buf()),
            profile: None,
            no_persist: true,
        }
    }

    #[test]
    fn accept_snapshots_findings_then_scan_is_clean() {
        let tmp = tempfile::tempdir().unwrap();
        let proj = tmp.path().join("proj");
        std::fs::create_dir_all(&proj).unwrap();
        std::fs::write(proj.join("unsafe.js"), "eval('x');\n").unwrap();

        // First scan gates on the finding.
        let scanned = run_scan(tmp.path(), scan_args(&proj), false);
        assert_eq!(scanned.exit_code, 1);
        assert_eq!(scanned.summary.new_findings, 1);

        // Accept snapshots the finding into the baseline (clean result).
        let accepted = run_scan(tmp.path(), scan_args(&proj), true);
        assert_eq!(accepted.verb, "accept");
        assert_eq!(accepted.exit_code, 0);

        // Re-scan: the known finding is suppressed from the gate.
        let rescanned = run_scan(tmp.path(), scan_args(&proj), false);
        assert_eq!(rescanned.exit_code, 0);
        assert_eq!(rescanned.summary.security_findings, 1);
        assert_eq!(rescanned.summary.new_findings, 0);
    }

    #[test]
    fn newly_introduced_finding_gates_against_baseline() {
        let tmp = tempfile::tempdir().unwrap();
        let proj = tmp.path().join("proj");
        std::fs::create_dir_all(&proj).unwrap();
        std::fs::write(proj.join("unsafe.js"), "eval('x');\n").unwrap();

        run_scan(tmp.path(), scan_args(&proj), true); // accept the first finding

        // A second, unaccepted finding must gate.
        std::fs::write(proj.join("unsafe2.js"), "eval('y');\n").unwrap();
        let regated = run_scan(tmp.path(), scan_args(&proj), false);
        assert_eq!(regated.exit_code, 1);
        assert_eq!(regated.summary.security_findings, 2);
        assert_eq!(regated.summary.new_findings, 1);
    }

    #[test]
    fn config_drives_zero_arg_scan() {
        let tmp = tempfile::tempdir().unwrap();
        let proj = tmp.path().join("src");
        std::fs::create_dir_all(&proj).unwrap();
        std::fs::write(proj.join("safe.js"), "const x = 1;\n").unwrap();
        std::fs::write(
            tmp.path().join("guard.toml"),
            format!("paths = [\"{}\"]\n", proj.display()),
        )
        .unwrap();

        // No positional path => paths come from guard.toml.
        let report = run_scan(
            tmp.path(),
            ScanArgs {
                path: None,
                profile: None,
                no_persist: true,
            },
            false,
        );
        assert_eq!(report.exit_code, 0);
        assert_eq!(report.target, proj.display().to_string());
    }
}
// CODEGEN-END
