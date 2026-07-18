// SPEC-MANAGED: apps/mesh/tech-design/interfaces/cli/scaffold-service-crate-and-standard-cli-shell.md#logic
// <HANDWRITE gap="mesh-cli-shell-scaffold" tracker="#1970" reason="Initial Mesh CLI shell: standard llm/upgrade/issue verbs plus placeholder domain verbs before any graph engine exists.">
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mesh", version, about = "mesh - relationship/property-graph service")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print agent-facing LLM topics, offline.
    Llm(LlmArgs),
    /// Self-update this binary from a published GitHub release.
    Upgrade(UpgradeArgs),
    /// Search, view, file, and comment on Mesh issues.
    Issue(IssueArgs),
    /// Serve the graph service over HTTP. Not implemented yet.
    Serve(PlaceholderArgs),
    /// Manage graph collections (namespaces). Not implemented yet.
    Collections(PlaceholderArgs),
    /// Manage graph nodes. Not implemented yet.
    Nodes(PlaceholderArgs),
    /// Manage graph edges. Not implemented yet.
    Edges(PlaceholderArgs),
    /// Run a traversal/path query. Not implemented yet.
    Query(PlaceholderArgs),
    /// Render mesh's runtime image Dockerfiles. Not implemented yet.
    Dockerfile(PlaceholderArgs),
    /// Kubernetes deployment artifacts. Not implemented yet.
    K8s(PlaceholderArgs),
}

#[derive(clap::Args)]
struct LlmArgs {
    /// Topic: outline or boundaries.
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
    /// Install this exact version (`0.4.8` or `mesh@0.4.8`) instead of latest.
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
    /// Search Mesh issues (`app:mesh`); omit query to list recent.
    Search(IssueSearchArgs),
    /// Print one issue by number.
    View(IssueViewArgs),
    /// File a diagnostics-rich Mesh issue.
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

/// Shared args for domain verbs that don't exist yet: swallow any trailing
/// tokens so `mesh <verb> --help`/`mesh <verb> <anything>` both parse, and
/// the handler always reports "not implemented yet" instead of panicking.
#[derive(clap::Args)]
struct PlaceholderArgs {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    _rest: Vec<String>,
}

const TOOL: cli_std::ToolInfo = cli_std::ToolInfo {
    project: "mesh",
    repo: "chrischeng-c4/axiom",
    target: env!("MESH_TARGET"),
    version: env!("CARGO_PKG_VERSION"),
    git_sha: env!("MESH_GIT_SHA"),
    built_at: env!("MESH_BUILT_AT"),
};

const LLM_TOPICS: &[cli_std::llm::Topic] = &[cli_std::llm::Topic {
    id: "boundaries",
    summary: "what mesh owns vs. lumen/beam/cube, and the log-driven derived-index model",
    body: "\
# mesh \u{2014} service boundaries

Mesh is the relationship/property-graph service in the Axiom service stack.
It owns typed node/edge storage with properties, and traversal/path query
over that graph.

Like `lumen`, mesh is log-driven and derived: writes fold through a
raft-replicated log into a separate, rebuildable local index. The caller
owns the system of record; mesh never becomes the durable owner of
relationship data (unlike `beam`, which is a durable source of truth for
vectors).

Mesh is intentionally separate from its siblings:
- `beam` owns vector ANN search.
- `lumen` owns lexical/semantic/perceptual search and dedup.
- `cube` owns OLAP-style columnar aggregation.
- `mesh` owns the graph shape: nodes, typed edges, properties, and
  traversal/path query.

This is the first CLI shell slice (#1970): only `llm`/`upgrade`/`issue` and
placeholder domain verbs exist today. The graph storage engine, raft state
machine, shard routing, and traversal executor are future work under the
mesh epic (#1969).
",
}];

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
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
        Command::Serve(_) => not_implemented_yet("serve"),
        Command::Collections(_) => not_implemented_yet("collections"),
        Command::Nodes(_) => not_implemented_yet("nodes"),
        Command::Edges(_) => not_implemented_yet("edges"),
        Command::Query(_) => not_implemented_yet("query"),
        Command::Dockerfile(_) => not_implemented_yet("dockerfile"),
        Command::K8s(_) => not_implemented_yet("k8s"),
    }
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
                    .map(|head| format!("mesh: {}", head.chars().take(72).collect::<String>()))
                    .unwrap_or_else(|| "mesh: issue report".to_string())
            });
            cli_std::issue::create(
                &TOOL,
                cli_std::issue::CreateOptions {
                    title,
                    message,
                    url: args.url,
                    repo: args.repo,
                    label: std::iter::once("app:mesh".to_string())
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

/// Every domain verb before the graph engine exists (#1969) reports this
/// instead of panicking or silently no-oping. Exit code 3 distinguishes
/// "known gap" from a generic clap usage error (2) or an anyhow bail (1).
fn not_implemented_yet(thing: &str) -> Result<()> {
    eprintln!("mesh: not implemented yet: {thing}");
    eprintln!("next: track progress at https://github.com/chrischeng-c4/axiom/issues/1969");
    std::process::exit(3);
}
// </HANDWRITE>
