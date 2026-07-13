// HANDWRITE-BEGIN gap="sift-service-cli" tracker="1576" reason="Implement serve, event, query, replay, spec, llm, upgrade, and issue CLI surfaces."
use std::{path::PathBuf, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use axum::extract::DefaultBodyLimit;
use clap::{Args, Parser, Subcommand, ValueEnum};
use sift::{DurableJournal, EventEnvelope, EventQuery, ServiceState, SignalKind};

#[derive(Parser)]
#[command(
    name = "sift",
    version,
    about = "Sift — operational event service and CLI"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the unified h2c/HTTP1 operational-event service.
    Serve(ServeArgs),
    /// Append one versioned event from a JSON file to the local raw journal.
    Event(EventArgs),
    /// Query durable raw events without starting a server.
    Query(QueryArgs),
    /// Replay durable raw events after a cursor without starting a server.
    Replay(ReplayArgs),
    /// Print the API contract generated from the same OpenAPI document as the service.
    Spec(SpecArgs),
    /// Print offline agent-facing operational documentation.
    Llm(LlmArgs),
    /// Check or install a released Sift binary.
    Upgrade(UpgradeArgs),
    /// Search, inspect, or file Sift issues.
    Issue(IssueArgs),
}

#[derive(Args)]
struct ServeArgs {
    #[arg(long, env = "SIFT_HOST", default_value = "0.0.0.0")]
    host: String,
    #[arg(long, env = "SIFT_PORT", default_value_t = 7380)]
    port: u16,
    #[arg(long, env = "SIFT_DATA_DIR", default_value = "sift-data")]
    data_dir: PathBuf,
    #[arg(long, env = "SIFT_LOG_LEVEL", default_value = "info")]
    log_level: String,
    #[arg(long, env = "SIFT_LOG_FORMAT", value_enum, default_value_t = LogFormat::Json)]
    log_format: LogFormat,
    #[arg(long, env = "SIFT_GRACE_SECS", default_value_t = 20)]
    grace_secs: u64,
    #[arg(long, env = "SIFT_MAX_BODY_BYTES", default_value_t = 1_048_576)]
    max_body_bytes: usize,
    #[arg(long, env = "SIFT_OTLP_ENDPOINT")]
    otlp_endpoint: Option<String>,
}

#[derive(Clone, Copy, ValueEnum)]
enum LogFormat {
    Pretty,
    Json,
}

#[derive(Args)]
struct EventArgs {
    /// JSON file containing one EventEnvelope.
    file: PathBuf,
    #[arg(long, env = "SIFT_DATA_DIR", default_value = "sift-data")]
    data_dir: PathBuf,
}

#[derive(Args)]
struct QueryArgs {
    #[arg(long, env = "SIFT_DATA_DIR", default_value = "sift-data")]
    data_dir: PathBuf,
    #[arg(long, value_enum)]
    signal: Option<SignalKind>,
    #[arg(long, default_value_t = 100)]
    limit: usize,
    #[arg(long, default_value_t = 0)]
    after: u64,
}

#[derive(Args)]
struct ReplayArgs {
    #[arg(long, env = "SIFT_DATA_DIR", default_value = "sift-data")]
    data_dir: PathBuf,
    #[arg(long, default_value_t = 0)]
    after: u64,
    #[arg(long, default_value_t = 100)]
    limit: usize,
}

#[derive(Args)]
struct SpecArgs {
    #[arg(long, default_value = "openapi-json", value_parser = ["openapi-json"])]
    format: String,
}

#[derive(Args)]
struct LlmArgs {
    #[arg(long, default_value = "outline")]
    topic: String,
    #[arg(long, default_value = "md", value_parser = ["md", "json"])]
    format: String,
}

#[derive(Args)]
struct UpgradeArgs {
    #[arg(long)]
    check: bool,
    #[arg(long = "version")]
    tag: Option<String>,
    #[arg(long)]
    force: bool,
    #[arg(short = 'y', long)]
    yes: bool,
}

#[derive(Args)]
struct IssueArgs {
    #[command(subcommand)]
    command: IssueCommand,
}

#[derive(Subcommand)]
enum IssueCommand {
    Search(IssueSearchArgs),
    View(IssueViewArgs),
    Create(IssueCreateArgs),
    Comment(IssueCommentArgs),
}

#[derive(Args)]
struct IssueSearchArgs {
    #[arg(value_name = "QUERY", num_args = 0..)]
    query: Vec<String>,
    #[arg(long, default_value = "open", value_parser = ["open", "closed", "all"])]
    state: String,
    #[arg(long, default_value_t = 20)]
    limit: u32,
}

#[derive(Args)]
struct IssueViewArgs {
    number: u64,
}

#[derive(Args)]
struct IssueCreateArgs {
    #[arg(short = 't', long)]
    title: Option<String>,
    #[arg(value_name = "MESSAGE", num_args = 0..)]
    message: Vec<String>,
    #[arg(long)]
    dry_run: bool,
    #[arg(short = 'y', long)]
    yes: bool,
}

#[derive(Args)]
struct IssueCommentArgs {
    number: u64,
    #[arg(value_name = "MESSAGE", num_args = 0..)]
    message: Vec<String>,
    #[arg(long)]
    dry_run: bool,
    #[arg(short = 'y', long)]
    yes: bool,
}

const TOOL: cli_std::ToolInfo = cli_std::ToolInfo {
    project: "sift",
    repo: "chrischeng-c4/axiom",
    target: "source",
    version: env!("CARGO_PKG_VERSION"),
    git_sha: "unknown",
    built_at: "unknown",
};

const LLM_TOPICS: &[cli_std::llm::Topic] = &[
    cli_std::llm::Topic {
        id: "ingest",
        summary: "versioned six-signal ingest and the fsync acknowledgement boundary",
        body: "# Sift ingest\n\nPOST a versioned `EventEnvelope` to `/v1/events`. Success means the canonical raw journal append has completed `sync_data`; retry the same `event_id` safely. `metric` events must carry direct points and exemplars.",
    },
    cli_std::llm::Topic {
        id: "operations",
        summary: "h2c serving, probe routes, query, replay, and local CLI use",
        body: "# Sift operations\n\nRun `sift serve --data-dir ./sift-data`. The process serves HTTP/1.1 and h2c on one port plus `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, and `/docs`. Use `sift event`, `sift query`, and `sift replay` for local durable-journal inspection.",
    },
];

#[tokio::main]
async fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Serve(args) => serve(args).await,
        Command::Event(args) => append_event(args),
        Command::Query(args) => query(args),
        Command::Replay(args) => replay(args),
        Command::Spec(args) => {
            let _ = args.format;
            println!("{}", sift::openapi_json()?);
            println!("next: done");
            Ok(())
        }
        Command::Llm(args) => {
            let output = cli_std::llm::render(
                "sift",
                env!("CARGO_PKG_VERSION"),
                LLM_TOPICS,
                &args.topic,
                cli_std::llm::Format::parse(&args.format),
            )?;
            println!("{output}");
            println!("next: done");
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
        Command::Issue(args) => issue(args).await,
    }
}

async fn serve(args: ServeArgs) -> Result<()> {
    let format = match args.log_format {
        LogFormat::Pretty => service_http::LogFormat::Pretty,
        LogFormat::Json => service_http::LogFormat::Json,
    };
    let config = service_http::HttpConfig::new(
        args.host,
        args.port,
        args.log_level,
        format,
        args.grace_secs,
        args.max_body_bytes,
        args.otlp_endpoint,
    );
    service_http::init_tracing(&config)?;

    let state = Arc::new(ServiceState::open(&args.data_dir)?);
    let app =
        service_http::standard_probe_routes(state.clone(), Some(state.clone()), sift::openapi)
            .merge(
                sift::router(state.clone()).layer(DefaultBodyLimit::max(config.body_limit_bytes)),
            )
            .layer(service_http::trace_layer());
    let listener = tokio::net::TcpListener::bind(config.bind_addr())
        .await
        .context("bind Sift service listener")?;
    tracing::info!(address = %config.bind_addr(), "sift serving HTTP/1.1 and h2c");
    let grace = Duration::from_secs(config.grace_secs);
    service_http::serve(
        listener,
        app,
        service_http::shutdown_with_drain(move || state.start_drain(), grace),
    )
    .await;
    Ok(())
}

fn append_event(args: EventArgs) -> Result<()> {
    let source = std::fs::read_to_string(&args.file)
        .with_context(|| format!("read event file {}", args.file.display()))?;
    let event: EventEnvelope = serde_json::from_str(&source).context("parse EventEnvelope JSON")?;
    let result = DurableJournal::open(&args.data_dir)?.append(event)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    println!("next: done");
    Ok(())
}

fn query(args: QueryArgs) -> Result<()> {
    let rows = DurableJournal::open(&args.data_dir)?.query(EventQuery {
        signal: args.signal,
        after: args.after,
        limit: args.limit,
    })?;
    println!("{}", serde_json::to_string_pretty(&rows)?);
    println!("next: done");
    Ok(())
}

fn replay(args: ReplayArgs) -> Result<()> {
    let rows = DurableJournal::open(&args.data_dir)?.replay(args.after, args.limit)?;
    println!("{}", serde_json::to_string_pretty(&rows)?);
    println!("next: done");
    Ok(())
}

async fn issue(args: IssueArgs) -> Result<()> {
    match args.command {
        IssueCommand::Search(args) => {
            cli_std::issue::search(
                &TOOL,
                cli_std::issue::SearchOptions {
                    query: (!args.query.is_empty()).then(|| args.query.join(" ")),
                    state: args.state,
                    limit: args.limit,
                },
            )
            .await
        }
        IssueCommand::View(args) => cli_std::issue::view(&TOOL, args.number).await,
        IssueCommand::Create(args) => {
            let message = (!args.message.is_empty()).then(|| args.message.join(" "));
            let title = args
                .title
                .unwrap_or_else(|| "sift: issue report".to_string());
            cli_std::issue::create(
                &TOOL,
                cli_std::issue::CreateOptions {
                    title,
                    message,
                    label: vec!["app:sift".to_string()],
                    dry_run: args.dry_run,
                    yes: args.yes,
                    ..Default::default()
                },
            )
            .await
        }
        IssueCommand::Comment(args) => {
            cli_std::issue::comment(
                &TOOL,
                cli_std::issue::CommentOptions {
                    number: args.number,
                    message: (!args.message.is_empty()).then(|| args.message.join(" ")),
                    dry_run: args.dry_run,
                    yes: args.yes,
                    ..Default::default()
                },
            )
            .await
        }
    }
}
// HANDWRITE-END
