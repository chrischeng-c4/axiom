// SPEC-MANAGED: apps/tape/tech-design/semantic/source/apps-tape-src-bin-tape-rs.md#logic
// <HANDWRITE gap="missing-generator:logic:tape-bootstrap" tracker="#768" reason="Initial Tape CLI surface before generated command wiring exists.">
use std::fs;
use std::path::{Path, PathBuf};

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

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Append(args) => append(args),
        Command::Replay(args) => replay(args),
        Command::Checkpoint(args) => checkpoint(args),
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
// </HANDWRITE>
