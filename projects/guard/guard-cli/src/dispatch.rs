// SPEC-MANAGED: projects/guard/tech-design/semantic/source/projects-guard-guard-cli-src-dispatch-rs.md#rust-source-unit
// CODEGEN-BEGIN
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use guard::{EvidenceCommand, GuardReport, PolicyProfile, ScanOptions};

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
    /// Run the baseline static security profile over a file or directory.
    Scan(ScanArgs),
    /// Re-project `.guard/last-report.json` without scanning.
    Report,
    /// Offline self-description of the report/policy surface.
    Spec,
    /// Offline agent playbook.
    Llm,
}

#[derive(Args, Debug)]
/// @spec projects/guard/tech-design/semantic/source/projects-guard-guard-cli-src-dispatch-rs.md#source
pub struct ScanArgs {
    /// File or directory to scan.
    #[arg(default_value = ".")]
    pub path: PathBuf,
    /// Policy profile: baseline static security, security-impacting lint, or strict.
    #[arg(long, value_enum, default_value_t = ProfileArg::BaselineStatic)]
    pub profile: ProfileArg,
    /// Run a named vat runner as isolated security evidence.
    #[arg(long = "vat-runner")]
    pub vat_runners: Vec<String>,
    /// Run an exact vat evidence command through `sh -c`.
    #[arg(long = "vat-command")]
    pub vat_commands: Vec<String>,
    /// Run rig scenarios under a directory as dynamic exploit/e2e evidence.
    #[arg(long = "rig-dir")]
    pub rig_dirs: Vec<PathBuf>,
    /// Run one rig scenario file as dynamic exploit/e2e evidence.
    #[arg(long = "rig-scenario")]
    pub rig_scenarios: Vec<PathBuf>,
    /// Run an exact rig evidence command through `sh -c`.
    #[arg(long = "rig-command")]
    pub rig_commands: Vec<String>,
    /// Run meter against a crate/project target as resource evidence.
    #[arg(long = "meter-target")]
    pub meter_targets: Vec<PathBuf>,
    /// Run an exact meter evidence command through `sh -c`.
    #[arg(long = "meter-command")]
    pub meter_commands: Vec<String>,
    /// Run an arena comparison spec as security budget evidence.
    #[arg(long = "arena-spec")]
    pub arena_specs: Vec<PathBuf>,
    /// Run an exact arena evidence command through `sh -c`.
    #[arg(long = "arena-command")]
    pub arena_commands: Vec<String>,
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
    match cmd.verb {
        Verb::Scan(args) => {
            let no_persist = args.no_persist;
            let mut options = ScanOptions::default();
            options.profile = args.profile.into();
            options.evidence_commands = evidence_commands_from_scan_args(&args);
            let report = guard::scan::scan_path_with_options(&args.path, options);
            if !no_persist {
                report.persist(std::path::Path::new("."));
            }
            report
        }
        Verb::Report => GuardReport::read_last(std::path::Path::new(".")).unwrap_or_else(|e| {
            GuardReport::tool_error(
                "report",
                ".",
                5,
                format!("no readable .guard/last-report.json: {e}"),
            )
        }),
        Verb::Spec => GuardReport::stub(
            "spec",
            "guard.report/1: compass-backed static/security-lint findings plus optional vat/rig/meter/arena evidence adapters.",
        ),
        Verb::Llm => GuardReport::stub(
            "llm",
            "Use `guard scan <path> --profile security-lint` for security posture. Add vat/rig/meter/arena evidence flags when dynamic evidence is required. Treat non-zero findings as actionable unless a documented guard policy exception exists.",
        ),
    }
}

fn evidence_commands_from_scan_args(args: &ScanArgs) -> Vec<EvidenceCommand> {
    let mut commands = Vec::new();
    let vat_cwd = vat_cwd_for_scan_path(&args.path);
    for runner in &args.vat_runners {
        let vat = sibling_tool("vat");
        let command = EvidenceCommand::argv(
            "vat",
            runner.clone(),
            vec![vat, "run".to_string(), "--json".to_string(), runner.clone()],
        );
        commands.push(match &vat_cwd {
            Some(cwd) => command.with_cwd(cwd),
            None => command,
        });
    }
    for command in &args.vat_commands {
        commands.push(EvidenceCommand::shell("vat", command, command));
    }
    for dir in &args.rig_dirs {
        let rig = sibling_tool("rig");
        let dir = dir.display().to_string();
        let label = dir.clone();
        commands.push(EvidenceCommand::argv(
            "rig",
            label,
            vec![
                rig,
                "run".to_string(),
                "--dir".to_string(),
                dir,
                "--compact".to_string(),
            ],
        ));
    }
    for scenario in &args.rig_scenarios {
        let rig = sibling_tool("rig");
        let scenario = scenario.display().to_string();
        let label = scenario.clone();
        commands.push(EvidenceCommand::argv(
            "rig",
            label,
            vec![
                rig,
                "run".to_string(),
                "--scenario".to_string(),
                scenario,
                "--compact".to_string(),
            ],
        ));
    }
    for command in &args.rig_commands {
        commands.push(EvidenceCommand::shell("rig", command, command));
    }
    for target in &args.meter_targets {
        let meter = sibling_tool("meter");
        let target = target.display().to_string();
        let label = target.clone();
        commands.push(
            EvidenceCommand::argv(
                "meter",
                label,
                vec![
                    meter,
                    "run".to_string(),
                    "--target".to_string(),
                    target,
                    "--skip-bench".to_string(),
                    "--skip-profile".to_string(),
                    "--compact".to_string(),
                ],
            )
            .with_env("CC", "/usr/bin/cc")
            .with_env("PATH", stable_rust_path()),
        );
    }
    for command in &args.meter_commands {
        commands.push(EvidenceCommand::shell("meter", command, command));
    }
    for spec in &args.arena_specs {
        let arena = sibling_tool("arena");
        let spec = spec.display().to_string();
        let label = spec.clone();
        commands.push(EvidenceCommand::argv(
            "arena",
            label,
            vec![
                arena,
                "run".to_string(),
                "--spec".to_string(),
                spec,
                "--compact".to_string(),
            ],
        ));
    }
    for command in &args.arena_commands {
        commands.push(EvidenceCommand::shell("arena", command, command));
    }
    commands
}

fn vat_cwd_for_scan_path(path: &std::path::Path) -> Option<PathBuf> {
    let dir = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()?.to_path_buf()
    };
    if dir.join("vat.toml").exists() {
        Some(dir)
    } else {
        None
    }
}

fn sibling_tool(name: &str) -> String {
    let Ok(current_exe) = std::env::current_exe() else {
        return name.to_string();
    };
    let Some(parent) = current_exe.parent() else {
        return name.to_string();
    };
    let candidate = parent.join(name);
    if candidate.exists() {
        candidate.display().to_string()
    } else {
        name.to_string()
    }
}

fn stable_rust_path() -> String {
    let Ok(home) = std::env::var("HOME") else {
        return std::env::var("PATH").unwrap_or_default();
    };
    format!(
        "{home}/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:{home}/.cargo/bin"
    )
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
            "guard {} -> exit {} (security_findings={})",
            report.verb, report.exit_code, report.summary.security_findings
        );
    }
}
// CODEGEN-END
