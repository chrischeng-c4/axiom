// SPEC-MANAGED: projects/cap/tech-design/semantic/cap-src.md#schema
// CODEGEN-BEGIN
//! CLI surface — clap-derive command tree.
//!
//! ```text
//!   cap init [claude|codex] [--project] [--print]   # install agent hooks
//!   cap <cmd> [args...]           # default: wrap cmd
//!   cap run -- <cmd> [args...]    # explicit
//!   cap daemon start|stop|status|run
//!   cap status                    # leases + memory pressure
//!   cap ps                        # alias
//!   cap config show|init          # `config init` writes ~/.cap/config.toml
//!   cap ping
//! ```

use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};

use crate::client::Client;
use crate::config::Config;
use crate::daemon;
use crate::hook_install;
use crate::protocol::{LeaseState, Request, Response};
use crate::supervisor::SpawnSpec;

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
    /// Run a command under cap (explicit form).
    Run(RunArgs),
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

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
#[derive(Parser, Debug)]
pub struct RunArgs {
    /// Human-readable label shown in `cap status`.
    #[arg(long)]
    pub label: Option<String>,
    /// The command to run, after `--`.
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
        anyhow::bail!("nothing to run; usage: cap run -- <command> [args...]");
    }
    let program = args.command[0].clone();
    let rest: Vec<String> = args.command[1..].to_vec();

    // The hook wraps EVERY agent Bash call in `cap`, so a broken or
    // unreachable daemon must degrade to "run it unthrottled" — never
    // to "the agent can't run anything". If we can't reach/spawn the
    // daemon, run the command directly and return its exit code.
    let mut client =
        match Client::connect_or_launch(|| daemon::spawn_background().map(|_| ())).await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("cap: daemon unavailable ({e:#}); running unthrottled");
                return run_unmanaged(&program, &rest).await;
            }
        };

    // The lease dance (Acquire → spawn → Spawned → wait → Release) lives in
    // managed_run. If the daemon killed the child, surface the reason on stderr
    // so the agent knows it wasn't a real failure.
    let outcome =
        crate::managed_run::managed_run(&mut client, SpawnSpec::new(program, rest), args.label)
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
// CODEGEN-END
