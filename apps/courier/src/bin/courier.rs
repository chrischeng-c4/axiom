// SPEC-MANAGED: apps/courier/tech-design/semantic/src/bin/courier.md#schema
// HANDWRITE-BEGIN gap="missing-generator:logic:c0ur1e08" tracker="pending-tracker" reason="Single courier CLI bin (clap): Command::Llm/Upgrade/Issue dispatch to cli_std::{llm,upgrade,issue} with courier's ToolInfo; mirrors apps/relay/src/bin/relay.rs."
//! courier — stateless, GCP-hosted GitHub-issues proxy (HTTP/2 + OpenAPI).
//!
//! The standard agent-facing commands — `courier llm`, `courier
//! upgrade`, `courier issue` (all offline-safe, network paths behind the
//! `self-update`/`issue` features via the shared `cli-std` lib) — sit
//! alongside it per the CONTRIBUTING.md CLI convention. Agents start at
//! `courier llm`.

use anyhow::Result;
use clap::{Parser, Subcommand};

#[path = "../llm.rs"]
mod llm;

use llm::{TOOL, TOPICS};

#[derive(Parser, Debug)]
#[command(
    name = "courier",
    version,
    about = "courier — stateless, GCP-hosted GitHub-issues proxy"
)]
struct Cli {
    /// Standard agent-facing command.
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Print agent-facing LLM topics — offline, no server. `outline`
    /// (default) maps the topics; pass a topic id for detail (`--format
    /// json` for a machine-readable form).
    Llm(LlmArgs),
    /// Self-update this binary from a published GitHub release. Resolves the
    /// running target + version, downloads the matching
    /// `courier-<target>.tar.gz`, verifies its sha256, and atomically
    /// replaces the executable. `--check` reports the available version
    /// without changing anything.
    Upgrade(UpgradeArgs),
    /// Search, view, and file courier issues on the axiom tracker
    /// (`search`/`view`/`create`). `create` auto-tags `app:courier`;
    /// `search` is filtered to courier's own issues.
    Issue(IssueArgs),
}

/// `courier llm` flags.
#[derive(clap::Args, Debug)]
struct LlmArgs {
    /// Topic id (`outline` lists them all).
    #[arg(default_value = "outline")]
    topic: String,
    /// Output format: `md` (default) or `json`.
    #[arg(long, default_value = "md")]
    format: String,
}

/// `courier upgrade` flags.
#[derive(clap::Args, Debug)]
struct UpgradeArgs {
    /// Report the current and latest version without modifying the binary.
    #[arg(long)]
    check: bool,
    /// Install this exact version (`0.4.3` or `courier@0.4.3`) instead of the latest.
    #[arg(long)]
    tag: Option<String>,
    /// Reinstall even when already on the selected version.
    #[arg(long)]
    force: bool,
    /// Skip the confirmation prompt.
    #[arg(short = 'y', long)]
    yes: bool,
}

/// `courier issue <search|view|create>` — search, read, and file courier
/// issues. Positional slots are reserved for the verb + its primary object,
/// so the rest are flags (the CLI convention).
#[derive(clap::Args, Debug)]
struct IssueArgs {
    #[command(subcommand)]
    cmd: IssueCommand,
}

#[derive(Subcommand, Debug)]
enum IssueCommand {
    /// Search courier's issues (`app:courier`); omit the query to list recent.
    Search(IssueSearchArgs),
    /// Print a single issue by number.
    View(IssueViewArgs),
    /// File a structured issue (auto-tagged `app:courier`).
    Create(IssueCreateArgs),
}

/// `courier issue search [query] [--state] [--limit]` flags.
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

/// `courier issue view <number>` flags.
#[derive(clap::Args, Debug)]
struct IssueViewArgs {
    /// Issue number.
    number: u64,
}

/// `courier issue create [--title <t>] [message...]` flags.
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
    // Install the process-level rustls crypto provider before anything
    // dials TLS (reqwest's rustls-tls-native-roots backend + the online CLI
    // paths both link rustls, which panics without a default provider).
    peer_tls::install_default_crypto_provider();
    let cli = Cli::parse();
    dispatch(cli.cmd).await
}

async fn dispatch(cmd: Command) -> Result<()> {
    match cmd {
        Command::Llm(args) => {
            // Offline: no server, no I/O beyond stdout.
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

/// `courier issue <verb>` — dispatch search/view/create to cli-std. `create`
/// always tags `app:courier`; `search` is filtered to courier's own issues.
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
                    "courier: issue report".to_string()
                } else {
                    let head: String = msg.lines().next().unwrap_or("").chars().take(72).collect();
                    format!("courier: {head}")
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
                    label: vec!["app:courier".to_string()],
                    dry_run: m.dry_run,
                    yes: true,
                },
            )
            .await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    /// The convention verbs all parse.
    #[test]
    fn cli_parse_surface() {
        Cli::command().debug_assert();
        assert!(Cli::try_parse_from(["courier"]).is_err());
        assert!(matches!(
            Cli::try_parse_from(["courier", "llm"]).unwrap().cmd,
            Command::Llm(_)
        ));
        assert!(matches!(
            Cli::try_parse_from(["courier", "upgrade", "--check"])
                .unwrap()
                .cmd,
            Command::Upgrade(_)
        ));
        assert!(matches!(
            Cli::try_parse_from(["courier", "issue", "search", "proxy"])
                .unwrap()
                .cmd,
            Command::Issue(_)
        ));
        assert!(matches!(
            Cli::try_parse_from(["courier", "issue", "view", "42"])
                .unwrap()
                .cmd,
            Command::Issue(_)
        ));
        assert!(matches!(
            Cli::try_parse_from(["courier", "issue", "create", "--dry-run", "it", "broke"])
                .unwrap()
                .cmd,
            Command::Issue(_)
        ));
    }

    /// build-stamp envs populate ToolInfo (never empty; "unknown" is the
    /// stamped fallback outside a git checkout).
    #[test]
    fn toolinfo_is_stamped() {
        assert_eq!(TOOL.project, "courier");
        assert!(!TOOL.version.is_empty());
        assert!(!TOOL.target.is_empty());
        assert!(!TOOL.git_sha.is_empty());
        assert!(!TOOL.built_at.is_empty());
    }
}
// HANDWRITE-END
