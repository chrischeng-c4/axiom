// HANDWRITE-BEGIN gap="missing-generator:logic:1eefd229" tracker="pending-tracker" reason="Single relay CLI bin (clap): bare relay (no subcommand) runs the h2c server with ServeArgs flags falling back to RELAY_BIND/RELAY_DATA_DIR env (the relay_server.rs behavior verbatim); Command::Llm/Upgrade/Issue dispatch to cli_std::{llm,upgrade,issue} with relay's ToolInfo; mirrors projects/keep/src/bin/keep.rs."
//! relay — cloud-native work-queue broker (HTTP/2 + OpenAPI).
//!
//! Bare `relay` (no subcommand) runs the server — the former `relay-server`
//! entrypoint verbatim (env-driven; flags override). The standard agent-facing
//! commands — `relay llm`, `relay upgrade`, `relay issue` (all offline-safe,
//! network paths behind the `self-update`/`issue` features via the shared
//! `cli-std` lib) — sit alongside it per the CONTRIBUTING.md CLI convention.
//! Agents start at `relay llm`.

use std::time::Duration;

use anyhow::Result;
use clap::{Parser, Subcommand};

use relay::server::{router, AppState};
use relay::server_config::RelayServerConfig;
use relay::spawn_reconciler;

#[path = "../llm.rs"]
mod llm;

use llm::{TOOL, TOPICS};

#[derive(Parser, Debug)]
#[command(
    name = "relay",
    version,
    about = "relay — durable single-cast work-queue broker (HTTP/2 + OpenAPI)"
)]
struct Cli {
    /// Standard agent-facing command. Omit it to run the server (the default).
    #[command(subcommand)]
    cmd: Option<Command>,
    /// Server flags — used when no subcommand is given (`relay <flags>`).
    #[command(flatten)]
    serve: ServeArgs,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Print agent-facing LLM topics — offline, no server. `outline` (default)
    /// maps the topics; pass a topic id for detail (`--format json` for a
    /// machine-readable form).
    Llm(LlmArgs),
    /// Self-update this binary from a published GitHub release. Resolves the
    /// running target + version, downloads the matching `relay-<target>.tar.gz`,
    /// verifies its sha256, and atomically replaces the executable. `--check`
    /// reports the available version without changing anything.
    Upgrade(UpgradeArgs),
    /// Search, view, and file relay issues on the axiom tracker
    /// (`search`/`view`/`create`). `create` auto-tags `project:relay`;
    /// `search` is filtered to relay's own issues.
    Issue(IssueArgs),
}

/// Server flags (the bare `relay` path) — the `relay-server` env knobs
/// surfaced as flags with env fallback.
#[derive(clap::Args, Debug)]
struct ServeArgs {
    /// h2c listen address for this shard.
    #[arg(long, env = "RELAY_BIND", default_value = "0.0.0.0:7000")]
    bind: String,
    /// Durable log directory (defaults to the core config's data dir).
    #[arg(long, env = "RELAY_DATA_DIR")]
    data_dir: Option<String>,
}

/// `relay llm` flags.
#[derive(clap::Args, Debug)]
struct LlmArgs {
    /// Topic id (`outline` lists them all).
    #[arg(default_value = "outline")]
    topic: String,
    /// Output format: `md` (default) or `json`.
    #[arg(long, default_value = "md")]
    format: String,
}

/// `relay upgrade` flags.
#[derive(clap::Args, Debug)]
struct UpgradeArgs {
    /// Report the current and latest version without modifying the binary.
    #[arg(long)]
    check: bool,
    /// Install this exact version (`0.4.3` or `relay@0.4.3`) instead of the latest.
    #[arg(long)]
    tag: Option<String>,
    /// Reinstall even when already on the selected version.
    #[arg(long)]
    force: bool,
    /// Skip the confirmation prompt.
    #[arg(short = 'y', long)]
    yes: bool,
}

/// `relay issue <search|view|create>` — search, read, and file relay issues.
/// Positional slots are reserved for the verb + its primary object, so the rest
/// are flags (the CLI convention).
#[derive(clap::Args, Debug)]
struct IssueArgs {
    #[command(subcommand)]
    cmd: IssueCommand,
}

#[derive(Subcommand, Debug)]
enum IssueCommand {
    /// Search relay's issues (`project:relay`); omit the query to list recent.
    Search(IssueSearchArgs),
    /// Print a single issue by number.
    View(IssueViewArgs),
    /// File a structured issue (auto-tagged `project:relay`).
    Create(IssueCreateArgs),
}

/// `relay issue search [query] [--state] [--limit]` flags.
#[derive(clap::Args, Debug)]
struct IssueSearchArgs {
    /// Search text (omit to list recent issues).
    #[arg(num_args = 0..)]
    query: Vec<String>,
    /// Issue state filter.
    #[arg(long, value_parser = ["open", "closed", "all"], default_value = "open")]
    state: String,
    /// Max results.
    #[arg(long, default_value_t = 20)]
    limit: u32,
}

/// `relay issue view <number>` flags.
#[derive(clap::Args, Debug)]
struct IssueViewArgs {
    /// Issue number.
    number: u64,
}

/// `relay issue create [--title <t>] [message...]` flags.
#[derive(clap::Args, Debug)]
struct IssueCreateArgs {
    /// Issue title (default: derived from the message).
    #[arg(long)]
    title: Option<String>,
    /// Print the issue that would be filed (and its URL) without creating it.
    #[arg(long)]
    dry_run: bool,
    /// Free-text description of the problem.
    #[arg(num_args = 0..)]
    message: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        // Default (no subcommand): run the server.
        None => serve_main(cli.serve).await,
        Some(cmd) => dispatch(cmd).await,
    }
}

async fn dispatch(cmd: Command) -> Result<()> {
    match cmd {
        Command::Llm(args) => {
            // Offline: no engine, no server, no I/O beyond stdout.
            let out = cli_std::llm::render(
                TOOL.project,
                TOOL.version,
                TOPICS,
                &args.topic,
                cli_std::llm::Format::parse(&args.format),
            )?;
            println!("{out}");
            Ok(())
        }
        Command::Upgrade(args) => {
            cli_std::upgrade::run(
                &TOOL,
                cli_std::upgrade::Options {
                    check: args.check,
                    tag: args.tag,
                    force: args.force,
                    yes: args.yes,
                },
            )
            .await
        }
        Command::Issue(args) => dispatch_issue(args).await,
    }
}

/// `relay issue <verb>` — dispatch search/view/create to cli-std. `create`
/// always tags `project:relay`; `search` is filtered to relay's own issues.
async fn dispatch_issue(args: IssueArgs) -> Result<()> {
    match args.cmd {
        IssueCommand::Search(m) => {
            let joined = m.query.join(" ");
            let query = (!joined.trim().is_empty()).then_some(joined);
            cli_std::issue::search(
                &TOOL,
                cli_std::issue::SearchOptions {
                    query,
                    state: m.state,
                    limit: m.limit,
                },
            )
            .await
        }
        IssueCommand::View(m) => cli_std::issue::view(&TOOL, m.number).await,
        IssueCommand::Create(m) => {
            let msg = m.message.join(" ");
            let title = m.title.unwrap_or_else(|| {
                if msg.trim().is_empty() {
                    "relay: issue report".to_string()
                } else {
                    let head: String = msg.lines().next().unwrap_or("").chars().take(72).collect();
                    format!("relay: {head}")
                }
            });
            let message = (!msg.trim().is_empty()).then_some(msg);
            cli_std::issue::create(
                &TOOL,
                cli_std::issue::CreateOptions {
                    title,
                    message,
                    url: None,
                    repo: None,
                    label: vec!["project:relay".to_string()],
                    dry_run: m.dry_run,
                    yes: true,
                },
            )
            .await
        }
    }
}

/// Run the relay server (the default, no-subcommand path) — the former
/// `relay-server` entrypoint: load config, spawn the lease reconciler, serve
/// the h2c app.
async fn serve_main(args: ServeArgs) -> Result<()> {
    let mut config = RelayServerConfig::default();
    config.bind = args.bind;
    if let Some(data_dir) = args.data_dir {
        config.core.data_dir = data_dir;
    }
    let bind = config.bind.clone();
    let reconcile_interval = Duration::from_millis(config.reconcile_interval_ms);

    let state = AppState::new(config);
    // Held for the process lifetime; aborts on drop (i.e. never, since serve runs forever).
    let _reconciler = spawn_reconciler(state.relay_handle(), reconcile_interval);

    let app = router(state);
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    eprintln!("relay listening on {} (h2c)", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    /// R1: the convention verbs + bare-serve default all parse.
    #[test]
    fn cli_parse_surface() {
        Cli::command().debug_assert();
        assert!(Cli::try_parse_from(["relay"]).unwrap().cmd.is_none());
        assert!(matches!(
            Cli::try_parse_from(["relay", "llm"]).unwrap().cmd,
            Some(Command::Llm(_))
        ));
        assert!(matches!(
            Cli::try_parse_from(["relay", "llm", "http-api", "--format", "json"])
                .unwrap()
                .cmd,
            Some(Command::Llm(_))
        ));
        assert!(matches!(
            Cli::try_parse_from(["relay", "upgrade", "--check"]).unwrap().cmd,
            Some(Command::Upgrade(_))
        ));
        assert!(matches!(
            Cli::try_parse_from(["relay", "issue", "search", "lease"]).unwrap().cmd,
            Some(Command::Issue(_))
        ));
        assert!(matches!(
            Cli::try_parse_from(["relay", "issue", "view", "42"]).unwrap().cmd,
            Some(Command::Issue(_))
        ));
        assert!(matches!(
            Cli::try_parse_from(["relay", "issue", "create", "--dry-run", "it", "broke"])
                .unwrap()
                .cmd,
            Some(Command::Issue(_))
        ));
        // R2: relay-server's env knobs surface as flags on the bare path.
        let cli = Cli::try_parse_from(["relay", "--bind", "127.0.0.1:0", "--data-dir", "/tmp/x"])
            .unwrap();
        assert!(cli.cmd.is_none());
        assert_eq!(cli.serve.bind, "127.0.0.1:0");
        assert_eq!(cli.serve.data_dir.as_deref(), Some("/tmp/x"));
    }

    /// R3: build-stamp envs populate ToolInfo (never empty; "unknown" is the
    /// stamped fallback outside a git checkout).
    #[test]
    fn toolinfo_is_stamped() {
        assert_eq!(TOOL.project, "relay");
        assert!(!TOOL.version.is_empty());
        assert!(!TOOL.target.is_empty());
        assert!(!TOOL.git_sha.is_empty());
        assert!(!TOOL.built_at.is_empty());
    }
}
// HANDWRITE-END
