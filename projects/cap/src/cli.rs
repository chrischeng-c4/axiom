// SPEC-MANAGED: projects/cap/tech-design/semantic/cap-src.md#schema
// CODEGEN-BEGIN
//! CLI surface — clap-derive command tree.
//!
//! ```text
//!   cap init [claude|codex] [--project] [--print]   # install agent hooks
//!   cap <cmd> [args...]           # default: wrap cmd
//!   cap run -- <cmd> [args...]    # explicit
//!   cap run "<bash command>"       # string command entrypoint for hooks
//!   cap explain -- <cmd> [args...] # show command replacement decision
//!   cap daemon start|stop|status|run
//!   cap status                    # leases + memory pressure
//!   cap ps                        # alias
//!   cap config show|init          # `config init` writes ~/.cap/config.toml
//!   cap ping
//!   cap llm [--topic <topic>] [--format md|json]
//!   cap upgrade [--version <tag>] [--check]
//!   cap issue search|view|create
//! ```

use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::client::Client;
use crate::command_planner::{self, CommandPlan, ExternalPlan};
use crate::config::Config;
use crate::daemon;
use crate::hook_install;
use crate::protocol::{LeaseState, Request, Response};
use crate::resident_shell::{ResidentLightShellRun, ResidentLightShellSession};
use crate::session_queue::{self, QueueDecision};
use crate::supervisor::SpawnSpec;

const TOOL: cli_std::ToolInfo = cli_std::ToolInfo {
    project: "cap",
    repo: "chrischeng-c4/axiom",
    target: env!("CAP_TARGET"),
    version: env!("CARGO_PKG_VERSION"),
    git_sha: env!("CAP_GIT_SHA"),
    built_at: env!("CAP_BUILT_AT"),
};

const LLM_TOPICS: &[cli_std::llm::Topic] = &[
    cli_std::llm::Topic {
        id: "workflow",
        summary: "agent hook setup and command wrapping mental model",
        body: "\
# cap workflow

cap protects the local machine from agent-launched command storms. Start with
`cap init` to install fail-open Bash hooks for Claude Code and Codex CLI. The
hooks rewrite agent Bash commands to `cap run '<original command>'`, and cap
then decides whether to run the command under a daemon lease, a resident native
fast path, a session queue barrier, or Bash fallback.

Use `cap status`, `cap ps`, and `cap wait` to inspect or wait for host
capacity. Use `cap explain -- <command>` to see whether a command can use a
native replacement.",
    },
    cli_std::llm::Topic {
        id: "commands",
        summary: "task to command cheat sheet",
        body: "\
# cap commands

| task | command |
|---|---|
| install agent hooks | `cap init` |
| preview hook snippets | `cap init --print` |
| wrap one command string | `cap run 'cargo test -p cap'` |
| wrap argv directly | `cap run -- cargo test -p cap` |
| inspect replacement decision | `cap explain -- find . -type f` |
| show leases and pressure | `cap status` |
| wait for headroom | `cap wait --timeout 30` |
| search cap issues | `cap issue search queue` |
| file a diagnostics issue | `cap issue create --title 'cap: ...' ...` |",
    },
    cli_std::llm::Topic {
        id: "boundaries",
        summary: "what cap does not claim",
        body: "\
# cap boundaries

cap is a resource governor and optimizer boundary. It is not a sandbox, chroot,
container, environment manager, or full replacement shell. Unsupported shell
syntax must fall back to Bash, and unknown or risky command strings stay on the
existing synchronous managed-run path unless a profile explicitly enables an
optimization.",
    },
];

#[derive(Parser, Debug)]
#[command(
    name = "cap",
    version,
    about = "Live-throttling wrapper for heavy local commands",
    trailing_var_arg = true
)]
/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Cmd>,

    /// Default form — anything after `cap` that isn't a known
    /// subcommand is treated as the command to wrap.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub passthrough: Vec<String>,
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// Print offline agent-facing docs for driving cap.
    Llm(LlmArgs),
    /// Self-update cap from cap@* GitHub releases.
    Upgrade(UpgradeArgs),
    /// Search, view, and file cap issues on the axiom tracker.
    Issue {
        #[command(subcommand)]
        action: IssueCmd,
    },
    /// Deprecated alias for `cap issue create`.
    #[command(name = "report-issue")]
    ReportIssue(ReportIssueArgs),
    /// Run a command under cap (explicit form).
    Run(RunArgs),
    /// Show how cap would execute a command.
    Explain(ExplainArgs),
    /// Daemon lifecycle.
    Daemon {
        #[command(subcommand)]
        action: DaemonCmd,
    },
    /// Print leases + memory pressure.
    Status,
    /// Alias of `status`.
    Ps,
    /// Show or initialize ~/.cap/config.toml.
    Config {
        #[command(subcommand)]
        action: ConfigCmd,
    },
    /// Liveness probe.
    Ping,
    /// Block until system memory/CPU headroom recovers above the pause
    /// floor. Exit 0 = capacity OK, exit 124 = timed out (matches GNU
    /// `timeout`'s convention). The daemon caps wait time at 5 min
    /// server-side regardless of `--timeout`.
    Wait {
        /// Maximum seconds to wait. Default: server cap (5 min).
        #[arg(long)]
        timeout: Option<u64>,
    },
    /// Set up cap: install the PreToolUse hook into your coding agents.
    ///
    /// With no arguments, installs into BOTH Claude Code and Codex CLI
    /// at the user level (`~/.claude/settings.json`,
    /// `~/.codex/config.toml`) so every Bash command the agent runs is
    /// transparently throttled by cap. Pass agent names to narrow it
    /// (`cap init claude`), `--project` for cwd scope, or `--print` to
    /// preview the snippets without writing anything.
    Init {
        /// Install into the current project (`./.claude`, `./.codex`)
        /// instead of the user-global location.
        #[arg(long)]
        project: bool,
        /// Print the snippet(s) to stdout and exit; do not modify files.
        #[arg(long)]
        print: bool,
        /// Limit to specific agents. Default: all of them.
        #[arg(value_enum)]
        agents: Vec<AgentArg>,
    },
    /// Hook adapters (Claude Code, etc.).
    Hook {
        #[command(subcommand)]
        action: HookCmd,
    },
}

/// @spec projects/cap/tech-design/interfaces/cli/adopt-cli-convention-llm-upgrade-report-issue-via-cclab-cli-std.md
#[derive(Parser, Debug)]
pub struct LlmArgs {
    /// Topic to print: outline (default), workflow, commands, boundaries.
    #[arg(long, default_value = "outline")]
    pub topic: String,
    /// Output format.
    #[arg(long, value_parser = ["md", "json"], default_value = "md")]
    pub format: String,
}

/// @spec projects/cap/tech-design/interfaces/cli/adopt-cli-convention-llm-upgrade-report-issue-via-cclab-cli-std.md
#[derive(Parser, Debug)]
pub struct UpgradeArgs {
    /// Install a specific release tag, e.g. cap@0.3.62 or 0.3.62.
    #[arg(long = "version")]
    pub tag: Option<String>,
    /// Only report whether a newer release exists; do not install.
    #[arg(long)]
    pub check: bool,
}

/// @spec projects/cap/tech-design/interfaces/cli/adopt-cli-convention-llm-upgrade-report-issue-via-cclab-cli-std.md
#[derive(Subcommand, Debug)]
pub enum IssueCmd {
    /// Search cap issues; omit the query to list recent open project:cap issues.
    Search(IssueSearchArgs),
    /// Print a single issue by number.
    View(IssueViewArgs),
    /// File a structured diagnostics-rich issue.
    Create(IssueCreateArgs),
}

/// @spec projects/cap/tech-design/interfaces/cli/adopt-cli-convention-llm-upgrade-report-issue-via-cclab-cli-std.md
#[derive(Args, Debug)]
pub struct IssueSearchArgs {
    /// Search text.
    #[arg(num_args = 0..)]
    pub query: Vec<String>,
    /// Issue state filter.
    #[arg(long, default_value = "open", value_parser = ["open", "closed", "all"])]
    pub state: String,
    /// Max results.
    #[arg(long, default_value_t = 20)]
    pub limit: u32,
}

/// @spec projects/cap/tech-design/interfaces/cli/adopt-cli-convention-llm-upgrade-report-issue-via-cclab-cli-std.md
#[derive(Args, Debug)]
pub struct IssueViewArgs {
    /// Issue number.
    pub number: u64,
}

/// @spec projects/cap/tech-design/interfaces/cli/adopt-cli-convention-llm-upgrade-report-issue-via-cclab-cli-std.md
#[derive(Args, Debug)]
pub struct IssueCreateArgs {
    /// Issue title. Defaults to a title derived from the message.
    #[arg(long)]
    pub title: Option<String>,
    /// Print the issue without creating it.
    #[arg(long)]
    pub dry_run: bool,
    /// Free-text problem description.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub message: Vec<String>,
}

/// @spec projects/cap/tech-design/interfaces/cli/adopt-cli-convention-llm-upgrade-report-issue-via-cclab-cli-std.md
#[derive(Parser, Debug)]
pub struct ReportIssueArgs {
    /// Issue title.
    #[arg(long)]
    pub title: Option<String>,
    /// Print the issue without creating it.
    #[arg(long)]
    pub dry_run: bool,
    /// Free-text problem description.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub message: Vec<String>,
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
#[derive(Parser, Debug)]
pub struct RunArgs {
    /// Human-readable label shown in `cap status`.
    #[arg(long)]
    pub label: Option<String>,
    /// The command to run. Pass argv after `--`, or one Bash command string.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub command: Vec<String>,
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
#[derive(Parser, Debug)]
pub struct ExplainArgs {
    /// The command to explain, after `--`.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub command: Vec<String>,
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
#[derive(Subcommand, Debug)]
pub enum DaemonCmd {
    /// Spawn a background daemon if none is running.
    Start,
    /// Run the daemon in the foreground (used internally).
    Run,
    /// Ask the daemon to exit cleanly.
    Stop,
    /// Is a daemon currently up?
    Status,
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
#[derive(Subcommand, Debug)]
pub enum ConfigCmd {
    /// Print the effective config.
    Show,
    /// Write a default config to ~/.cap/config.toml if missing.
    Init,
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
#[derive(Subcommand, Debug)]
pub enum HookCmd {
    /// PreToolUse adapter for Bash (the runtime hook entrypoint that
    /// `cap init` registers — you normally don't run this by hand).
    /// Reads the hook event from stdin and prints a permission-decision
    /// JSON that rewrites the command to run under cap.
    Bash {
        /// Emit Codex CLI hook JSON (`updatedInput`).
        #[arg(long, conflicts_with = "claude_code")]
        codex: bool,
        /// Emit Claude Code hook JSON (`modifiedInput`).
        #[arg(long, conflicts_with = "codex")]
        claude_code: bool,
    },
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
#[derive(ValueEnum, Clone, Debug)]
pub enum AgentArg {
    /// Claude Code (~/.claude/settings.json)
    Claude,
    /// Codex CLI (~/.codex/config.toml)
    Codex,
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
pub async fn run() -> Result<ExitCode> {
    init_tracing();
    let cli = Cli::parse();

    match cli.command {
        Some(Cmd::Llm(args)) => handle_llm(args),
        Some(Cmd::Upgrade(args)) => handle_upgrade(args).await,
        Some(Cmd::Issue { action }) => handle_issue(action).await,
        Some(Cmd::ReportIssue(args)) => handle_report_issue(args).await,
        Some(Cmd::Daemon { action }) => handle_daemon(action).await,
        Some(Cmd::Status) | Some(Cmd::Ps) => handle_status().await,
        Some(Cmd::Config { action }) => handle_config(action),
        Some(Cmd::Ping) => handle_ping().await,
        Some(Cmd::Wait { timeout }) => handle_wait(timeout).await,
        Some(Cmd::Init {
            project,
            print,
            agents,
        }) => handle_init(agents, project, print),
        Some(Cmd::Hook { action }) => handle_hook(action),
        Some(Cmd::Run(args)) => handle_run(args).await,
        Some(Cmd::Explain(args)) => handle_explain(args),
        None => {
            if cli.passthrough.is_empty() {
                use clap::CommandFactory;
                Cli::command().print_help().ok();
                println!();
                Ok(ExitCode::SUCCESS)
            } else {
                handle_run(RunArgs {
                    label: None,
                    command: cli.passthrough,
                })
                .await
            }
        }
    }
}

fn handle_llm(args: LlmArgs) -> Result<ExitCode> {
    let out = cli_std::llm::render(
        TOOL.project,
        TOOL.version,
        LLM_TOPICS,
        &args.topic,
        cli_std::llm::Format::parse(&args.format),
    )?;
    println!("{out}");
    Ok(ExitCode::SUCCESS)
}

async fn handle_upgrade(args: UpgradeArgs) -> Result<ExitCode> {
    cli_std::upgrade::run(
        &TOOL,
        cli_std::upgrade::Options {
            check: args.check,
            tag: args.tag,
            force: false,
            yes: true,
        },
    )
    .await?;
    Ok(ExitCode::SUCCESS)
}

async fn handle_issue(action: IssueCmd) -> Result<ExitCode> {
    match action {
        IssueCmd::Search(args) => {
            cli_std::issue::search(
                &TOOL,
                cli_std::issue::SearchOptions {
                    query: join_words(args.query),
                    state: args.state,
                    limit: args.limit,
                },
            )
            .await?;
        }
        IssueCmd::View(args) => {
            cli_std::issue::view(&TOOL, args.number).await?;
        }
        IssueCmd::Create(args) => {
            cli_std::issue::create(&TOOL, issue_create_options(args)).await?;
        }
    }
    Ok(ExitCode::SUCCESS)
}

async fn handle_report_issue(args: ReportIssueArgs) -> Result<ExitCode> {
    cli_std::report_issue::run(&TOOL, report_issue_options(args)).await?;
    Ok(ExitCode::SUCCESS)
}

fn issue_create_options(args: IssueCreateArgs) -> cli_std::issue::CreateOptions {
    let message = join_words(args.message);
    cli_std::issue::CreateOptions {
        title: issue_title(args.title, message.as_deref()),
        message,
        url: None,
        repo: None,
        label: vec!["project:cap".to_string()],
        dry_run: args.dry_run,
        yes: true,
    }
}

fn report_issue_options(args: ReportIssueArgs) -> cli_std::report_issue::Options {
    let message = join_words(args.message);
    cli_std::report_issue::Options {
        title: issue_title(args.title, message.as_deref()),
        message,
        url: None,
        repo: None,
        label: vec!["project:cap".to_string()],
        dry_run: args.dry_run,
        yes: true,
    }
}

fn join_words(words: Vec<String>) -> Option<String> {
    let joined = words.join(" ");
    (!joined.trim().is_empty()).then_some(joined)
}

fn issue_title(explicit: Option<String>, message: Option<&str>) -> String {
    if let Some(title) = explicit.filter(|title| !title.trim().is_empty()) {
        return title;
    }
    let Some(message) = message.map(str::trim).filter(|message| !message.is_empty()) else {
        return "cap: issue report".to_string();
    };
    let head: String = message
        .lines()
        .next()
        .unwrap_or(message)
        .chars()
        .take(72)
        .collect();
    format!("cap: {head}")
}

fn handle_explain(args: ExplainArgs) -> Result<ExitCode> {
    if args.command.is_empty() {
        anyhow::bail!("nothing to explain; usage: cap explain -- <command> [args...]");
    }
    let plan = if args.command.len() == 1 {
        command_planner::plan_shell(&args.command[0], None)
    } else {
        command_planner::plan(&args.command, None)
    };
    println!("{}", plan.explain());
    Ok(ExitCode::SUCCESS)
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(match std::env::var("CAP_LOG").as_deref() {
            Ok("trace") => tracing::Level::TRACE,
            Ok("debug") => tracing::Level::DEBUG,
            Ok("warn") => tracing::Level::WARN,
            Ok("error") => tracing::Level::ERROR,
            _ => tracing::Level::INFO,
        })
        .try_init();
}

async fn handle_daemon(action: DaemonCmd) -> Result<ExitCode> {
    match action {
        DaemonCmd::Run => {
            daemon::run_foreground().await?;
            Ok(ExitCode::SUCCESS)
        }
        DaemonCmd::Start => {
            if daemon::is_running()? {
                println!("cap daemon already running");
                return Ok(ExitCode::SUCCESS);
            }
            let pid = daemon::spawn_background()?;
            println!("cap daemon started (pid {pid})");
            Ok(ExitCode::SUCCESS)
        }
        DaemonCmd::Stop => {
            let mut client = Client::connect().await?;
            let resp = client.request(&Request::Shutdown).await?;
            match resp {
                Response::ShuttingDown => {
                    println!("cap daemon: shutting down");
                    Ok(ExitCode::SUCCESS)
                }
                other => {
                    eprintln!("unexpected response: {other:?}");
                    Ok(ExitCode::FAILURE)
                }
            }
        }
        DaemonCmd::Status => {
            let running = daemon::is_running()?;
            println!(
                "cap daemon: {}",
                if running { "running" } else { "stopped" }
            );
            Ok(if running {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            })
        }
    }
}

async fn handle_status() -> Result<ExitCode> {
    let mut client = match Client::connect().await {
        Ok(c) => c,
        Err(_) => {
            println!("cap daemon: stopped");
            return Ok(ExitCode::SUCCESS);
        }
    };
    let resp = client.request(&Request::Status).await?;
    let Response::Status(snap) = resp else {
        eprintln!("unexpected response: {resp:?}");
        return Ok(ExitCode::FAILURE);
    };
    println!(
        "free {free:.2} GB  (pause<{pause:.2} GB, kill<{kill:.2} GB)   \
         load/core {load:.2}  (pause>{lp:.2})   running {r}  paused {p}",
        free = snap.free_mem_gb,
        pause = snap.pause_floor_gb,
        kill = snap.kill_floor_gb,
        load = snap.load_per_core,
        lp = snap.load_pause_floor,
        r = snap.running,
        p = snap.paused,
    );
    for l in &snap.leases {
        let state = match l.state {
            LeaseState::Pending => "PENDING",
            LeaseState::Running => "RUNNING",
            LeaseState::Paused => "PAUSED ",
            LeaseState::Killing => "KILLING",
            LeaseState::Killed => "KILLED ",
        };
        let pid = l
            .child_pid
            .map(|p| p.to_string())
            .unwrap_or_else(|| "-".into());
        println!(
            "  lease {} {} pid={} age={}s  {}",
            l.lease, state, pid, l.age_secs, l.label
        );
    }
    Ok(ExitCode::SUCCESS)
}

fn handle_config(action: ConfigCmd) -> Result<ExitCode> {
    match action {
        ConfigCmd::Show => {
            let cfg = Config::load()?;
            println!("{}", toml::to_string_pretty(&cfg)?);
            Ok(ExitCode::SUCCESS)
        }
        ConfigCmd::Init => {
            let path = crate::paths::config_path()?;
            if path.exists() {
                println!("config already exists at {}", path.display());
                return Ok(ExitCode::SUCCESS);
            }
            Config::default().save()?;
            println!("wrote default config to {}", path.display());
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn handle_hook(action: HookCmd) -> Result<ExitCode> {
    match action {
        HookCmd::Bash { codex, claude_code } => {
            let agent = match (codex, claude_code) {
                (true, false) => crate::hook::HookAgent::Codex,
                (false, true) => crate::hook::HookAgent::Claude,
                (false, false) => crate::hook::HookAgent::Auto,
                (true, true) => unreachable!("clap conflicts_with prevents both flags"),
            };
            crate::hook::run_bash_hook(agent)?;
            Ok(ExitCode::SUCCESS)
        }
    }
}

/// `cap init` — register the PreToolUse hook with one or more agents.
/// No agents given = all of them; no `--project` = user-global scope.
fn handle_init(agents: Vec<AgentArg>, project: bool, print: bool) -> Result<ExitCode> {
    let scope = if project {
        hook_install::Scope::Project
    } else {
        hook_install::Scope::User
    };
    // Default to every supported agent — the zero-argument `cap init`
    // is meant to "just set everything up".
    let targets: Vec<hook_install::Agent> = if agents.is_empty() {
        vec![hook_install::Agent::Claude, hook_install::Agent::Codex]
    } else {
        agents
            .iter()
            .map(|a| match a {
                AgentArg::Claude => hook_install::Agent::Claude,
                AgentArg::Codex => hook_install::Agent::Codex,
            })
            .collect()
    };
    for agent in targets {
        hook_install::run(agent, scope, print)?;
    }
    Ok(ExitCode::SUCCESS)
}

async fn handle_ping() -> Result<ExitCode> {
    let mut client = Client::connect_or_launch(|| daemon::spawn_background().map(|_| ())).await?;
    let resp = client.request(&Request::Ping).await?;
    println!("{resp:?}");
    Ok(ExitCode::SUCCESS)
}

/// Exit code 124 matches GNU `timeout`'s convention so existing shell
/// idioms like `cap wait --timeout 30 && cargo test` work without
/// special-casing.
const WAIT_TIMEOUT_EXIT: u8 = 124;

async fn handle_wait(timeout: Option<u64>) -> Result<ExitCode> {
    let mut client = Client::connect_or_launch(|| daemon::spawn_background().map(|_| ())).await?;
    let resp = client
        .request(&Request::WaitForCapacity {
            timeout_secs: timeout,
        })
        .await?;
    match resp {
        Response::CapacityOk => Ok(ExitCode::SUCCESS),
        Response::CapacityTimeout => Ok(ExitCode::from(WAIT_TIMEOUT_EXIT)),
        Response::Error { message } => {
            eprintln!("cap wait: {message}");
            Ok(ExitCode::FAILURE)
        }
        other => {
            eprintln!("cap wait: unexpected response: {other:?}");
            Ok(ExitCode::FAILURE)
        }
    }
}

/// Forward SIGINT / SIGTERM / SIGHUP from this process to the child's
/// process group, so Ctrl-C in the user's terminal still aborts the
/// wrapped command even though we put it in its own pgid.
async fn handle_run(args: RunArgs) -> Result<ExitCode> {
    if args.command.is_empty() {
        anyhow::bail!(
            "nothing to run; usage: cap run \"<command>\" or cap run -- <command> [args...]"
        );
    }
    if args.command.len() == 1 {
        match session_queue::handle_command_string(&args.command[0])? {
            QueueDecision::ContinueSynchronously => {}
            decision => {
                if let Some(code) = decision.exit_code() {
                    return Ok(code);
                }
            }
        }
        let session = ResidentLightShellSession::capture();
        return match session.run_command_string(&args.command[0], args.label)? {
            ResidentLightShellRun::Native(code) => Ok(code),
            ResidentLightShellRun::BashFallback(external) => handle_external_run(external).await,
        };
    }
    let plan = command_planner::plan(&args.command, args.label);
    match plan {
        CommandPlan::Native(native) => command_planner::run_native(&native),
        CommandPlan::External(external) => handle_external_run(external).await,
    }
}

async fn handle_external_run(plan: ExternalPlan) -> Result<ExitCode> {
    // The hook wraps EVERY agent Bash call in `cap`, so a broken or
    // unreachable daemon must degrade to "run it unthrottled" — never
    // to "the agent can't run anything". If we can't reach/spawn the
    // daemon, run the command directly and return its exit code.
    let mut client =
        match Client::connect_or_launch(|| daemon::spawn_background().map(|_| ())).await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("cap: daemon unavailable ({e:#}); running unthrottled");
                return run_unmanaged(&plan.program, &plan.args).await;
            }
        };

    // The lease dance (Acquire → spawn → Spawned → wait → Release) lives in
    // managed_run. If the daemon killed the child, surface the reason on stderr
    // so the agent knows it wasn't a real failure.
    let outcome = crate::managed_run::managed_run(
        &mut client,
        SpawnSpec::new(plan.program, plan.args),
        plan.label,
    )
    .await?;
    if let Some(envelope) = &outcome.kill_envelope {
        // `human_message` is pre-formatted multi-line — `eprint!` (not
        // `eprintln!`) so we don't add a trailing blank line.
        eprint!("{}", envelope.human_message);
    }
    Ok(exit_code_from(outcome.status))
}

/// Fallback path when the cap daemon is unreachable: run the command
/// directly with inherited stdio and no throttling, so a broken cap
/// never wedges the caller. No process-group remap and no `nice` bump —
/// the command behaves exactly as if `cap` weren't in front of it.
async fn run_unmanaged(program: &str, args: &[String]) -> Result<ExitCode> {
    use std::process::Stdio;
    let status = tokio::process::Command::new(program)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await
        .with_context(|| format!("spawning {program}"))?;
    Ok(exit_code_from(status))
}

/// Map a child's `ExitStatus` to a process `ExitCode`. Signal-terminated
/// children (`status.code() == None`, e.g. a cap SIGKILL) map to FAILURE.
fn exit_code_from(status: std::process::ExitStatus) -> ExitCode {
    match status.code() {
        Some(0) => ExitCode::SUCCESS,
        Some(c) if (0..=255).contains(&c) => ExitCode::from(c as u8),
        _ => ExitCode::FAILURE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_std_convention_help_lists_current_commands_and_compat_alias() {
        let help = Cli::command().render_help().to_string();
        for verb in ["llm", "upgrade", "issue", "report-issue"] {
            assert!(help.contains(verb), "missing {verb} in help:\n{help}");
        }
    }

    #[test]
    fn cli_std_convention_llm_uses_shared_renderer() {
        let out = cli_std::llm::render(
            TOOL.project,
            TOOL.version,
            LLM_TOPICS,
            "outline",
            cli_std::llm::Format::Md,
        )
        .unwrap();
        assert!(out.contains("cap issue search"));
        assert!(out.contains("cap upgrade"));
        assert!(out.contains("workflow"));
    }

    #[test]
    fn cli_std_convention_issue_create_is_project_scoped() {
        let opts = issue_create_options(IssueCreateArgs {
            title: None,
            dry_run: true,
            message: vec!["queue".into(), "barrier".into()],
        });
        assert_eq!(opts.title, "cap: queue barrier");
        assert_eq!(opts.message.as_deref(), Some("queue barrier"));
        assert_eq!(opts.label, vec!["project:cap"]);
        assert!(opts.dry_run);
    }

    #[test]
    fn cli_std_convention_report_issue_alias_uses_same_payload_shape() {
        let opts = report_issue_options(ReportIssueArgs {
            title: Some("custom".into()),
            dry_run: true,
            message: vec!["details".into()],
        });
        assert_eq!(opts.title, "custom");
        assert_eq!(opts.message.as_deref(), Some("details"));
        assert_eq!(opts.label, vec!["project:cap"]);
        assert!(opts.dry_run);
    }
}
// CODEGEN-END
