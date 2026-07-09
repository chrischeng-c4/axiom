// SPEC-MANAGED: apps/tape/tech-design/semantic/source/apps-tape-src-bin-tape-rs.md#logic
// <HANDWRITE gap="missing-generator:logic:tape-bootstrap" tracker="#768" reason="Initial Tape CLI surface before generated command wiring exists.">
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use serde_json::{json, Value};
use tape::{spec, TapeJournal};

#[derive(Parser)]
#[command(name = "tape", version, about = "tape - topic replay journal service")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Append one event envelope to a topic journal.
    Append(AppendArgs),
    /// Replay topic history by offset or timestamp.
    Replay(ReplayArgs),
    /// Manage durable consumer replay checkpoints.
    Checkpoint(CheckpointArgs),
    /// Serve the topic journal over HTTP (h2c + HTTP/1.1 on one port).
    Serve(ServeArgs),
    /// Print Tape's machine-readable API contract, offline.
    Spec(SpecArgs),
    /// Print agent-facing LLM topics, offline.
    Llm(LlmArgs),
    /// Self-update this binary from a published GitHub release.
    Upgrade(UpgradeArgs),
    /// Search, view, file, and comment on Tape issues.
    Issue(IssueArgs),
}

#[derive(clap::Args)]
struct AppendArgs {
    /// Topic name.
    topic: String,
    /// Optional partitioning/idempotency key carried in the event envelope.
    #[arg(long)]
    key: Option<String>,
    /// JSON payload or a string payload when the value is not valid JSON.
    #[arg(long)]
    payload: String,
    /// Override event timestamp for deterministic tests/backfill.
    #[arg(long)]
    timestamp_ms: Option<u64>,
    /// Journal file. Defaults to `.tape/journal.json`.
    #[arg(long, default_value = ".tape/journal.json")]
    store: PathBuf,
}

#[derive(clap::Args)]
struct ReplayArgs {
    /// Topic name.
    topic: String,
    /// First offset to include.
    #[arg(long)]
    from_offset: Option<u64>,
    /// First event timestamp to include.
    #[arg(long)]
    from_timestamp_ms: Option<u64>,
    /// Maximum number of events to return.
    #[arg(long)]
    limit: Option<usize>,
    /// Journal file. Defaults to `.tape/journal.json`.
    #[arg(long, default_value = ".tape/journal.json")]
    store: PathBuf,
}

#[derive(clap::Args)]
struct CheckpointArgs {
    #[command(subcommand)]
    command: CheckpointCommand,
}

#[derive(Subcommand)]
enum CheckpointCommand {
    /// Read a consumer checkpoint.
    Get(CheckpointGetArgs),
    /// Advance a consumer checkpoint.
    Put(CheckpointPutArgs),
}

#[derive(clap::Args)]
struct CheckpointGetArgs {
    topic: String,
    consumer: String,
    #[arg(long, default_value = ".tape/journal.json")]
    store: PathBuf,
}

#[derive(clap::Args)]
struct CheckpointPutArgs {
    topic: String,
    consumer: String,
    #[arg(long)]
    offset: u64,
    #[arg(long, default_value = ".tape/journal.json")]
    store: PathBuf,
}

#[derive(clap::Args, Debug)]
struct ServeArgs {
    /// h2c + HTTP/1.1 listen address.
    #[arg(long, env = "TAPE_BIND", default_value = "127.0.0.1:7137")]
    bind: String,
    /// Journal file to load at boot and persist to on every mutation.
    /// Defaults to an empty in-memory journal when unset.
    #[arg(long, env = "TAPE_STORE")]
    store: Option<PathBuf>,
    /// Graceful-drain window (seconds) held after SIGTERM before the
    /// listener closes, while `/readyz` reports 503 so k8s stops routing.
    #[arg(long, env = "TAPE_GRACE_SECS", default_value_t = 10)]
    grace_secs: u64,
    /// Request-auth mode for the /topics data plane: `off` (tokenless dev,
    /// the default) or `required` (bearer tokens from the registry file).
    /// Probes stay tokenless either way.
    #[arg(long, env = "TAPE_AUTH", default_value = "off")]
    auth: String,
    /// Bearer-token registry file (JSON `{token: {subject, roles}}`),
    /// mounted from a Secret in production. Required (and validated at
    /// startup) when `--auth required`.
    #[arg(long, env = "TAPE_TOKEN_REGISTRY_FILE")]
    token_registry_file: Option<PathBuf>,
}

#[derive(clap::Args)]
struct SpecArgs {
    /// Contract format to print.
    #[arg(long, value_enum, default_value_t = SpecFormat::Openapi)]
    format: SpecFormat,
}

#[derive(Clone, Copy, ValueEnum)]
enum SpecFormat {
    Openapi,
    OpenapiYaml,
    JsonSchema,
    Routes,
}

#[derive(clap::Args)]
struct LlmArgs {
    /// Topic: outline, workflow, api, or boundaries.
    #[arg(long, default_value = "outline")]
    topic: String,
    /// Output format: md or json.
    #[arg(long, default_value = "md")]
    format: String,
}

#[derive(clap::Args)]
struct UpgradeArgs {
    /// Report the current and latest version without modifying the binary.
    #[arg(long)]
    check: bool,
    /// Install this exact version (`0.4.3` or `tape@0.4.3`) instead of latest.
    #[arg(long = "version")]
    tag: Option<String>,
    /// Reinstall even when already on the selected version.
    #[arg(long)]
    force: bool,
    /// Skip the confirmation prompt.
    #[arg(short = 'y', long)]
    yes: bool,
}

#[derive(clap::Args)]
struct IssueArgs {
    #[command(subcommand)]
    command: IssueCommand,
}

#[derive(Subcommand)]
enum IssueCommand {
    /// Search Tape issues (`app:tape`); omit query to list recent.
    Search(IssueSearchArgs),
    /// Print one issue by number.
    View(IssueViewArgs),
    /// File a diagnostics-rich Tape issue.
    Create(IssueCreateArgs),
    /// Comment on an issue and ensure it is open.
    Comment(IssueCommentArgs),
}

#[derive(clap::Args)]
struct IssueSearchArgs {
    #[arg(value_name = "QUERY", num_args = 0..)]
    query: Vec<String>,
    #[arg(long, default_value = "open", value_parser = ["open", "closed", "all"])]
    state: String,
    #[arg(long, default_value_t = 20)]
    limit: u32,
}

#[derive(clap::Args)]
struct IssueViewArgs {
    number: u64,
}

#[derive(clap::Args)]
struct IssueCreateArgs {
    #[arg(short = 't', long)]
    title: Option<String>,
    #[arg(value_name = "MSG", num_args = 0..)]
    message: Vec<String>,
    #[arg(long)]
    url: Option<String>,
    #[arg(long)]
    repo: Option<String>,
    #[arg(long)]
    label: Vec<String>,
    #[arg(long)]
    dry_run: bool,
    #[arg(short = 'y', long)]
    yes: bool,
}

#[derive(clap::Args)]
struct IssueCommentArgs {
    number: u64,
    #[arg(value_name = "MSG", num_args = 0..)]
    message: Vec<String>,
    #[arg(long)]
    repo: Option<String>,
    #[arg(long)]
    dry_run: bool,
    #[arg(short = 'y', long)]
    yes: bool,
}

const TOOL: cli_std::ToolInfo = cli_std::ToolInfo {
    project: "tape",
    repo: "chrischeng-c4/axiom",
    target: env!("TAPE_TARGET"),
    version: env!("CARGO_PKG_VERSION"),
    git_sha: env!("TAPE_GIT_SHA"),
    built_at: env!("TAPE_BUILT_AT"),
};

const LLM_TOPICS: &[cli_std::llm::Topic] = &[
    cli_std::llm::Topic {
        id: "workflow",
        summary: "topic replay model, first CLI slice, and deferred production gates",
        body: spec::llm_workflow_md(),
    },
    cli_std::llm::Topic {
        id: "api",
        summary: "append, replay, checkpoint, retention, and standard endpoint contract",
        body: spec::llm_api_md(),
    },
    cli_std::llm::Topic {
        id: "boundaries",
        summary: "Tape boundary against Relay, Loom, and Keep",
        body: spec::llm_boundaries_md(),
    },
];

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Append(args) => append(args),
        Command::Replay(args) => replay(args),
        Command::Checkpoint(args) => checkpoint(args),
        Command::Serve(args) => serve_main(args).await,
        Command::Spec(args) => spec(args),
        Command::Llm(args) => llm(args),
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
        Command::Issue(args) => issue(args).await,
    }
}

fn append(args: AppendArgs) -> Result<()> {
    let mut journal = load_journal(&args.store)?;
    let payload = parse_payload(&args.payload);
    let event = journal.append(args.topic, args.key, payload, args.timestamp_ms);
    save_journal(&args.store, &journal)?;
    println!("{}", serde_json::to_string_pretty(&event)?);
    println!(
        "next: tape replay {} --from-offset {}",
        event.topic, event.offset
    );
    Ok(())
}

fn replay(args: ReplayArgs) -> Result<()> {
    let journal = load_journal(&args.store)?;
    let events = journal.replay(
        &args.topic,
        args.from_offset,
        args.from_timestamp_ms,
        args.limit,
    );
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({ "events": events }))?
    );
    println!("next: done");
    Ok(())
}

fn checkpoint(args: CheckpointArgs) -> Result<()> {
    match args.command {
        CheckpointCommand::Get(args) => {
            let journal = load_journal(&args.store)?;
            let checkpoint = journal.checkpoint(&args.topic, &args.consumer);
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({ "checkpoint": checkpoint }))?
            );
            println!("next: done");
            Ok(())
        }
        CheckpointCommand::Put(args) => {
            let mut journal = load_journal(&args.store)?;
            let checkpoint = journal.put_checkpoint(args.topic, args.consumer, args.offset)?;
            save_journal(&args.store, &journal)?;
            println!("{}", serde_json::to_string_pretty(&checkpoint)?);
            println!("next: done");
            Ok(())
        }
    }
}

/// Run the tape HTTP server: load the journal from `--store` (or start
/// empty), serve the shared service shell (standard probes merged with the
/// `/topics` data plane) over HTTP/1.1 + h2c on one port, with a
/// SIGTERM-aware graceful drain (`--grace-secs`).
async fn serve_main(args: ServeArgs) -> Result<()> {
    // RUST_LOG wins; otherwise default to info.
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    // Resolve the bearer-auth contract (#1326) BEFORE anything serves: with
    // --auth required a missing/unparseable/empty registry file is a startup
    // error (nonzero exit), never a per-request 401.
    let auth = tape::auth::AuthConfig::resolve(
        &args.auth,
        args.token_registry_file
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned())
            .as_deref(),
        std::env::var(tape::auth::LEGACY_TOKENS_ENV).ok().as_deref(),
    )?;
    tracing::info!(
        required = auth.required,
        "request auth resolved (TAPE_AUTH; probes stay tokenless)"
    );

    let journal = match &args.store {
        Some(path) => load_journal(path)?,
        None => TapeJournal::default(),
    };
    let state = tape::server::AppState::with_auth(journal, args.store.clone(), auth);
    let app = tape::server::router(state.clone());

    let listener = tokio::net::TcpListener::bind(&args.bind).await?;
    tracing::info!(
        addr = %listener.local_addr()?,
        "tape listening (HTTP/1.1 + HTTP/2 cleartext)"
    );

    let grace = Duration::from_secs(args.grace_secs);
    service_http::serve(
        listener,
        app,
        service_http::shutdown_with_drain(move || state.start_drain(), grace),
    )
    .await;
    Ok(())
}

fn spec(args: SpecArgs) -> Result<()> {
    let out = match args.format {
        SpecFormat::Openapi => spec::openapi_json(),
        SpecFormat::OpenapiYaml => spec::openapi_yaml(),
        SpecFormat::JsonSchema => spec::json_schema_json(),
        SpecFormat::Routes => spec::routes_json(),
    };
    println!("{out}");
    Ok(())
}

fn llm(args: LlmArgs) -> Result<()> {
    let out = cli_std::llm::render(
        TOOL.project,
        TOOL.version,
        LLM_TOPICS,
        &args.topic,
        cli_std::llm::Format::parse(&args.format),
    )?;
    println!("{out}");
    Ok(())
}

async fn issue(args: IssueArgs) -> Result<()> {
    match args.command {
        IssueCommand::Search(args) => {
            let query = (!args.query.is_empty()).then(|| args.query.join(" "));
            cli_std::issue::search(
                &TOOL,
                cli_std::issue::SearchOptions {
                    query,
                    state: args.state,
                    limit: args.limit,
                },
            )
            .await
        }
        IssueCommand::View(args) => cli_std::issue::view(&TOOL, args.number).await,
        IssueCommand::Create(args) => {
            let message = (!args.message.is_empty()).then(|| args.message.join(" "));
            let title = args.title.unwrap_or_else(|| {
                message
                    .as_deref()
                    .and_then(|msg| msg.lines().next())
                    .map(|head| format!("tape: {}", head.chars().take(72).collect::<String>()))
                    .unwrap_or_else(|| "tape: issue report".to_string())
            });
            cli_std::issue::create(
                &TOOL,
                cli_std::issue::CreateOptions {
                    title,
                    message,
                    url: args.url,
                    repo: args.repo,
                    label: std::iter::once("app:tape".to_string())
                        .chain(args.label)
                        .collect(),
                    dry_run: args.dry_run,
                    yes: args.yes,
                },
            )
            .await
        }
        IssueCommand::Comment(args) => {
            let message = (!args.message.is_empty()).then(|| args.message.join(" "));
            cli_std::issue::comment(
                &TOOL,
                cli_std::issue::CommentOptions {
                    number: args.number,
                    message,
                    repo: args.repo,
                    dry_run: args.dry_run,
                    yes: args.yes,
                },
            )
            .await
        }
    }
}

fn load_journal(path: &Path) -> Result<TapeJournal> {
    if !path.exists() {
        return Ok(TapeJournal::default());
    }
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))
}

fn save_journal(path: &Path, journal: &TapeJournal) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let bytes = serde_json::to_vec_pretty(journal)?;
    fs::write(path, bytes).with_context(|| format!("write {}", path.display()))
}

fn parse_payload(input: &str) -> Value {
    serde_json::from_str(input).unwrap_or_else(|_| Value::String(input.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_parse_surface() {
        Cli::command().debug_assert();

        // #1325: `serve` gains --bind/--store/--grace-secs, with env fallback
        // and a 10s default grace window; existing commands keep parsing.
        // #1326: `serve` also gains --auth/--token-registry-file, defaulting
        // to tokenless (`off`).
        let cli = Cli::try_parse_from(["tape", "serve"]).unwrap();
        let Command::Serve(args) = cli.command else {
            panic!("expected Serve");
        };
        assert_eq!(args.bind, "127.0.0.1:7137");
        assert!(args.store.is_none());
        assert_eq!(args.grace_secs, 10);
        assert_eq!(args.auth, "off");
        assert!(args.token_registry_file.is_none());

        let cli = Cli::try_parse_from([
            "tape",
            "serve",
            "--bind",
            "0.0.0.0:9000",
            "--store",
            "/tmp/journal.json",
            "--grace-secs",
            "3",
            "--auth",
            "required",
            "--token-registry-file",
            "/tmp/tape-token-registry.json",
        ])
        .unwrap();
        let Command::Serve(args) = cli.command else {
            panic!("expected Serve");
        };
        assert_eq!(args.bind, "0.0.0.0:9000");
        assert_eq!(args.store, Some(PathBuf::from("/tmp/journal.json")));
        assert_eq!(args.grace_secs, 3);
        assert_eq!(args.auth, "required");
        assert_eq!(
            args.token_registry_file,
            Some(PathBuf::from("/tmp/tape-token-registry.json"))
        );

        let cli =
            Cli::try_parse_from(["tape", "append", "orders", "--payload", "{\"n\":1}"]).unwrap();
        assert!(matches!(cli.command, Command::Append(_)));
    }
}
// </HANDWRITE>
