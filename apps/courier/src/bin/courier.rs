// HANDWRITE-BEGIN gap="missing-generator:logic:c0ur1e08" tracker="pending-tracker" reason="Single courier CLI bin (clap): bare courier (no subcommand) runs the h2c server with ServeArgs flags falling back to COURIER_* env; Command::Llm/Upgrade/Issue dispatch to cli_std::{llm,upgrade,issue} with courier's ToolInfo; mirrors apps/relay/src/bin/relay.rs."
//! courier — stateless, GCP-hosted GitHub-issues proxy (HTTP/2 + OpenAPI).
//!
//! Bare `courier` (no subcommand) runs the server (env-driven; flags
//! override). The standard agent-facing commands — `courier llm`, `courier
//! upgrade`, `courier issue` (all offline-safe, network paths behind the
//! `self-update`/`issue` features via the shared `cli-std` lib) — sit
//! alongside it per the CONTRIBUTING.md CLI convention. Agents start at
//! `courier llm`.

use anyhow::Result;
use clap::{Parser, Subcommand};

use courier::http::auth::AuthConfig;
use courier::http::github::GithubClient;
use courier::http::{router, AppState};

#[path = "../llm.rs"]
mod llm;

use llm::{TOOL, TOPICS};

#[derive(Parser, Debug)]
#[command(
    name = "courier",
    version,
    about = "courier — stateless, GCP-hosted GitHub-issues proxy (HTTP/2 + OpenAPI)"
)]
struct Cli {
    /// Standard agent-facing command. Omit it to run the server (the default).
    #[command(subcommand)]
    cmd: Option<Command>,
    /// Server flags — used when no subcommand is given (`courier <flags>`).
    #[command(flatten)]
    serve: ServeArgs,
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

/// Server flags (the bare `courier` path) — env-driven with flag overrides.
#[derive(clap::Args, Debug)]
struct ServeArgs {
    /// h2c/h1 listen address.
    #[arg(long, env = "COURIER_BIND", default_value = "0.0.0.0:7400")]
    bind: String,
    /// Graceful-drain window (seconds) held after SIGTERM before the
    /// listener closes, while `/readyz` reports 503 so k8s stops routing.
    #[arg(long, env = "COURIER_GRACE_SECS", default_value_t = 10)]
    grace_secs: u64,
    /// Request-auth mode for the /v1 data plane: `off` (tokenless dev, the
    /// default) or `required` (bearer tokens from the registry file).
    /// Probes stay tokenless either way.
    #[arg(long, env = "COURIER_AUTH", default_value = "off")]
    auth: String,
    /// Bearer-token registry file (JSON `{token: {subject, roles}}`),
    /// mounted from a Secret in production. Required (and validated at
    /// startup) when `--auth required`.
    #[arg(long, env = "COURIER_TOKEN_REGISTRY_FILE")]
    token_registry_file: Option<String>,
    /// The real GitHub credential courier forwards with. Required — a
    /// courier that can never call GitHub is a startup misconfiguration.
    #[arg(long, env = "COURIER_GITHUB_TOKEN")]
    github_token: Option<String>,
    /// Comma-separated `owner/name` allow-list. Defaults to
    /// `chrischeng-c4/axiom` when unset.
    #[arg(long, env = "COURIER_ALLOWED_REPOS")]
    allowed_repos: Option<String>,
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
    service_tls::install_default_crypto_provider();
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

/// Run the courier server (the default, no-subcommand path): resolve auth +
/// the GitHub client, serve the app through the shared service shell —
/// HTTP/1.1 + h2c on one port with a SIGTERM-aware graceful drain
/// (`--grace-secs`).
async fn serve_main(args: ServeArgs) -> Result<()> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    // Resolve the bearer-auth contract BEFORE anything serves: with --auth
    // required a missing/unparseable/empty registry file is a startup error
    // (nonzero exit), never a per-request 401.
    let auth = AuthConfig::resolve(
        &args.auth,
        args.token_registry_file.as_deref(),
        std::env::var(courier::http::auth::LEGACY_TOKENS_ENV)
            .ok()
            .as_deref(),
    )?;
    tracing::info!(
        required = auth.required,
        "request auth resolved (COURIER_AUTH; probes stay tokenless)"
    );

    // Resolve the GitHub credential + allow-list BEFORE anything serves: a
    // courier that can never call GitHub fails fast at startup, never per
    // request.
    if let Some(token) = &args.github_token {
        // SAFETY: single-threaded startup, before any other thread reads env.
        std::env::set_var(courier::http::github::GITHUB_TOKEN_ENV, token);
    }
    if let Some(repos) = &args.allowed_repos {
        std::env::set_var(courier::http::github::ALLOWED_REPOS_ENV, repos);
    }
    let github = GithubClient::from_env()?;

    let state = AppState::new(github, auth);
    let app = router(state.clone());
    let listener = tokio::net::TcpListener::bind(&args.bind).await?;
    tracing::info!(
        addr = %listener.local_addr()?,
        "courier listening (HTTP/1.1 + HTTP/2 cleartext)"
    );

    // Serve HTTP/1.1 + h2c on one port and drain on SIGTERM through the
    // shared service shell: `start_drain` flips `/readyz` to 503 for the
    // grace window before the listener closes.
    let grace = std::time::Duration::from_secs(args.grace_secs);
    service_http::serve(
        listener,
        app,
        service_http::shutdown_with_drain(move || state.start_drain(), grace),
    )
    .await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    /// The convention verbs + bare-serve default all parse.
    #[test]
    fn cli_parse_surface() {
        Cli::command().debug_assert();
        assert!(Cli::try_parse_from(["courier"]).unwrap().cmd.is_none());
        assert!(matches!(
            Cli::try_parse_from(["courier", "llm"]).unwrap().cmd,
            Some(Command::Llm(_))
        ));
        assert!(matches!(
            Cli::try_parse_from(["courier", "upgrade", "--check"])
                .unwrap()
                .cmd,
            Some(Command::Upgrade(_))
        ));
        assert!(matches!(
            Cli::try_parse_from(["courier", "issue", "search", "proxy"])
                .unwrap()
                .cmd,
            Some(Command::Issue(_))
        ));
        assert!(matches!(
            Cli::try_parse_from(["courier", "issue", "view", "42"])
                .unwrap()
                .cmd,
            Some(Command::Issue(_))
        ));
        assert!(matches!(
            Cli::try_parse_from(["courier", "issue", "create", "--dry-run", "it", "broke"])
                .unwrap()
                .cmd,
            Some(Command::Issue(_))
        ));
        let cli = Cli::try_parse_from([
            "courier",
            "--bind",
            "127.0.0.1:0",
            "--github-token",
            "t",
            "--allowed-repos",
            "a/b,c/d",
        ])
        .unwrap();
        assert!(cli.cmd.is_none());
        assert_eq!(cli.serve.bind, "127.0.0.1:0");
        assert_eq!(cli.serve.grace_secs, 10);
        assert_eq!(cli.serve.auth, "off");
        assert!(cli.serve.token_registry_file.is_none());
        assert_eq!(cli.serve.github_token.as_deref(), Some("t"));
        assert_eq!(cli.serve.allowed_repos.as_deref(), Some("a/b,c/d"));
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
