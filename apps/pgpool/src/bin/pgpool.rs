// SPEC-MANAGED: apps/pgpool/tech-design/semantic/source/apps-pgpool-src-bin-pgpool-rs.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-bootstrap" tracker="#pgpool-bootstrap" reason="Initial working-name CLI surface before generated command wiring exists.">
use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use pgpool::spec;

#[derive(Parser)]
#[command(
    name = "pgpool",
    version,
    about = "working-name PostgreSQL pooler service"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print the shared-runtime plan for the pooler data plane and admin plane.
    RuntimePlan,
    /// Print the offline admin API contract.
    Spec(SpecArgs),
    /// Print agent-facing LLM topics, offline.
    Llm(LlmArgs),
    /// Self-update this binary from a published GitHub release.
    Upgrade(UpgradeArgs),
    /// Search, view, file, and comment on pgpool issues.
    Issue(IssueArgs),
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
    /// Install this exact version (`0.4.3` or `pgpool@0.4.3`) instead of latest.
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
    /// Search pgpool issues (`project:pgpool`); omit query to list recent.
    Search(IssueSearchArgs),
    /// Print one issue by number.
    View(IssueViewArgs),
    /// File a diagnostics-rich pgpool issue.
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
    project: "pgpool",
    repo: "chrischeng-c4/axiom",
    target: env!("PGPOOL_TARGET"),
    version: env!("CARGO_PKG_VERSION"),
    git_sha: env!("PGPOOL_GIT_SHA"),
    built_at: env!("PGPOOL_BUILT_AT"),
};

const LLM_TOPICS: &[cli_std::llm::Topic] = &[
    cli_std::llm::Topic {
        id: "workflow",
        summary: "working-name app scaffold and pooler rollout boundaries",
        body: spec::llm_workflow_md(),
    },
    cli_std::llm::Topic {
        id: "api",
        summary: "PostgreSQL frontend and admin API route inventory",
        body: spec::llm_api_md(),
    },
    cli_std::llm::Topic {
        id: "boundaries",
        summary: "platform adapter and shared runtime boundaries",
        body: spec::llm_boundaries_md(),
    },
];

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::RuntimePlan => runtime_plan(),
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

fn runtime_plan() -> Result<()> {
    println!("{}", pgpool::runtime_plan_json());
    println!("next: pgpool spec --format routes");
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
                    .map(|head| format!("pgpool: {}", head.chars().take(72).collect::<String>()))
                    .unwrap_or_else(|| "pgpool: issue report".to_string())
            });
            cli_std::issue::create(
                &TOOL,
                cli_std::issue::CreateOptions {
                    title,
                    message,
                    url: args.url,
                    repo: args.repo,
                    label: std::iter::once("project:pgpool".to_string())
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
// </HANDWRITE>
