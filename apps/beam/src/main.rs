//! beam binary — the GPU-native vector database CLI.
//!
//! Standard agent-facing commands — `beam llm`, `beam upgrade`, `beam issue`
//! (the CONTRIBUTING.md CLI convention, via the shared `cli-std` lib) — sit
//! alongside `beam bench` (a GPU-vs-CPU vector-search demo over the in-memory
//! engine: flat, IVF-flat, and IVF-PQ indexes on wgpu/Metal) and the
//! placeholder service verbs (`serve`, `collections`, `index`, `query`,
//! `dockerfile`, `k8s`) that each exit with a tracked "not implemented yet: …"
//! diagnostic until the real feature lands. Agents start at
//! `beam llm --topic outline`.

use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};

/// Exit code the placeholder service verbs return — a consistent, non-zero
/// "not built yet" signal that is distinct from a hard failure (`ExitCode::FAILURE`).
const NOT_IMPLEMENTED_EXIT: u8 = 3;

/// This binary's identity + build provenance for the standard CLI ops
/// (`upgrade` / `issue`), per the CONTRIBUTING.md CLI convention. The `BEAM_*`
/// values are stamped by `build.rs`.
const TOOL: cli_std::ToolInfo = cli_std::ToolInfo {
    project: "beam",
    repo: "chrischeng-c4/axiom",
    target: env!("BEAM_TARGET"),
    version: env!("CARGO_PKG_VERSION"),
    git_sha: env!("BEAM_GIT_SHA"),
    built_at: env!("BEAM_BUILT_AT"),
};

/// beam's agent-facing `llm` topics — the single in-code source of truth. The
/// `outline` topic + standard-command footer are rendered by `cli-std`; the
/// topic *summaries* below are what an agent sees in `beam llm --topic outline`,
/// so they name Beam as a GPU-native vector DB and the Beam/Lumen boundary.
const TOPICS: &[cli_std::llm::Topic] = &[
    cli_std::llm::Topic {
        id: "architecture",
        summary: "Beam is a GPU-native vector database: vector-first storage, GPU ANN index lifecycle, batch ingest, and GPU vector-query execution",
        body: "# beam — architecture\n\n\
            Beam is the GPU-native vector database in the axiom stack. It owns \
            vector-first collections, GPU approximate-nearest-neighbor (ANN) index \
            build/load, GPU memory tiers with host spill, batch ingest, \
            compaction/rebuild, and vector query execution.\n\n\
            Beam is deliberately separate from lumen. Lumen is the mixed search \
            service (exact, lexical, semantic, perceptual, and duplicate search) and \
            owns ranking and dedup. Beam is the GPU vector engine underneath vector \
            retrieval — it never claims mixed search, ranking, or dedup.\n\n\
            Engine today: an in-memory vector collection with an exact CPU oracle, \
            a GPU flat (brute-force) k-NN index, and a GPU IVF-PQ (IVFADC) \
            approximate-nearest-neighbor index — all on wgpu (Metal on Apple Silicon, \
            Vulkan on Linux/NVIDIA). IVF-PQ prunes to the nprobe nearest cells and \
            product-quantizes residuals, so a query touches a small fraction of the \
            corpus; recall is verified against the flat oracle. `beam bench --index \
            flat|ivfflat|ivfpq` runs the GPU-vs-CPU parity, recall, pruning, and timing \
            demo. Still to come: durable segments, an HTTP/2 query API, and k8s. \
            Capability roots live in projects/beam/README.md (epic #769).\n",
    },
    cli_std::llm::Topic {
        id: "boundaries",
        summary: "Beam owns the GPU vector DB and index lifecycle; Lumen owns mixed search, ranking, and dedup",
        body: "# beam — boundaries\n\n\
            - Beam owns: GPU-native vector collections, GPU ANN index lifecycle, \
            batch ingest, compaction/rebuild, and GPU vector query execution.\n\
            - Lumen owns: mixed search (exact/lexical/semantic/perceptual/duplicate), \
            ranking, and dedup workflows.\n\
            - keep stores large external payloads; Beam stores vectors and the vector \
            metadata needed for ANN.\n\
            - cube owns analytical aggregates; Beam owns nearest-neighbor retrieval.\n",
    },
    cli_std::llm::Topic {
        id: "operations",
        summary: "Drive Beam from an agent: llm/upgrade/issue plus the placeholder serve/collections/index/query/dockerfile/k8s verbs",
        body: "# beam — operations\n\n\
            Standard agent commands (shared cli-std):\n\
            - `beam llm --topic <t>` — offline docs (this).\n\
            - `beam upgrade [--version <tag>] [--check]` — self-update from beam@* releases.\n\
            - `beam issue search|view|create` — read/file project:beam issues.\n\n\
            Service verbs — placeholders in this first slice; each exits with a \
            tracked 'not implemented yet: …' diagnostic until the feature lands:\n\
            - `beam serve` — HTTP service shell.\n\
            - `beam collections` — collection lifecycle.\n\
            - `beam index` — index lifecycle.\n\
            - `beam query` — vector query.\n\
            - `beam dockerfile` — dockerfile render.\n\
            - `beam k8s` — k8s render/operator.\n",
    },
];

#[derive(Parser)]
#[command(
    name = "beam",
    version,
    about = "beam — GPU-native vector database (vector-first storage, GPU ANN indexes, vector query)"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print agent-facing LLM topics — offline, no server. `--topic outline`
    /// (default) maps the topics; pass a topic id for detail (`--format json`
    /// for a machine-readable form).
    Llm(LlmArgs),
    /// Self-update this binary from a published `beam@*` GitHub release. Resolves
    /// the running target + version, downloads the matching `beam-<target>.tar.gz`,
    /// verifies its sha256, and atomically replaces the executable. `--check`
    /// reports the available version without changing anything.
    Upgrade(UpgradeArgs),
    /// Search, view, and file beam issues on the axiom tracker (scoped
    /// `project:beam`).
    Issue {
        #[command(subcommand)]
        action: IssueCmd,
    },
    /// Run the GPU vector-search benchmark: build a deterministic in-memory
    /// collection, search it on both the GPU (Metal via wgpu) and the exact CPU
    /// oracle, and print the GPU adapter, GPU-vs-CPU recall, and query timing.
    Bench(BenchArgs),
    /// (Placeholder) HTTP service shell — not implemented yet in this slice.
    Serve,
    /// (Placeholder) Vector collection lifecycle — not implemented yet.
    Collections,
    /// (Placeholder) ANN index lifecycle — not implemented yet.
    Index,
    /// (Placeholder) Vector nearest-neighbor query — not implemented yet.
    Query,
    /// (Placeholder) Dockerfile render — not implemented yet.
    Dockerfile,
    /// (Placeholder) Kubernetes render / operator — not implemented yet.
    #[command(name = "k8s")]
    K8s,
}

/// `beam llm` flags.
#[derive(Args)]
struct LlmArgs {
    /// Topic to print: outline (default), architecture, boundaries, operations.
    #[arg(long, default_value = "outline")]
    topic: String,
    /// Output format: `md` (default) or `json`.
    #[arg(long, value_parser = ["md", "json"], default_value = "md")]
    format: String,
}

/// `beam bench` flags — deterministic GPU-vs-CPU vector-search demo.
#[derive(Args)]
struct BenchArgs {
    /// Number of database vectors to generate.
    #[arg(long, default_value_t = 100_000)]
    n: usize,
    /// Vector dimension.
    #[arg(long, default_value_t = 128)]
    dim: usize,
    /// Neighbors per query (top-k).
    #[arg(long, default_value_t = 10)]
    k: usize,
    /// Number of queries to run.
    #[arg(long, default_value_t = 20)]
    queries: usize,
    /// Distance metric.
    #[arg(long, value_parser = ["l2", "dot", "cosine"], default_value = "l2")]
    metric: String,
    /// Index backend: `flat` (exact GPU brute force), `ivfflat` (IVF + exact
    /// residual refine), or `ivfpq` (IVF + product-quantized residuals). IVF
    /// backends use a clustered corpus and print the candidates-scanned ratio.
    #[arg(long, value_parser = ["flat", "ivfflat", "ivfpq"], default_value = "flat")]
    index: String,
    /// IVF coarse-cell count (ivfflat / ivfpq).
    #[arg(long, default_value_t = 256)]
    nlist: usize,
    /// Cells probed per query (ivfflat / ivfpq).
    #[arg(long, default_value_t = 16)]
    nprobe: usize,
    /// PQ subvector count (ivfpq); `dim` must be divisible by `m`.
    #[arg(long, default_value_t = 16)]
    m: usize,
    /// Corpus intrinsic dimension. `0` (default) = isotropic clustered data (PQ's
    /// worst case). `> 0` (e.g. 16 for dim=128) = embedding-like low-rank data
    /// where IVF-PQ recall is high. Only affects the ivfflat / ivfpq backends.
    #[arg(long, default_value_t = 0)]
    rank: usize,
    /// Filtered k-NN demo: tag every row `category = i % 8` and keep only rows
    /// with `category == <this>` (~1/8 selectivity), reporting filtered recall
    /// vs the filtered CPU oracle. Omit for the unfiltered bench.
    #[arg(long = "filter")]
    filter: Option<i64>,
}

/// `beam upgrade` flags (the convention surface: `--version` + `--check`).
#[derive(Args)]
struct UpgradeArgs {
    /// Install a specific release tag, e.g. `beam@0.4.3` or `0.4.3`.
    #[arg(long = "version")]
    tag: Option<String>,
    /// Only report whether a newer release exists; do not install.
    #[arg(long)]
    check: bool,
}

/// `beam issue <search|view|create>`.
#[derive(Subcommand)]
enum IssueCmd {
    /// Search beam issues (project:beam); omit the query to list recent.
    Search(IssueSearchArgs),
    /// Print a single issue by number.
    View(IssueViewArgs),
    /// File a structured diagnostics-rich issue (auto-tagged project:beam).
    Create(IssueCreateArgs),
}

#[derive(Args)]
struct IssueSearchArgs {
    /// Search text (omit to list recent issues).
    #[arg(num_args = 0..)]
    query: Vec<String>,
    /// Issue state filter.
    #[arg(long, default_value = "open", value_parser = ["open", "closed", "all"])]
    state: String,
    /// Max results.
    #[arg(long, default_value_t = 20)]
    limit: u32,
}

#[derive(Args)]
struct IssueViewArgs {
    /// Issue number.
    number: u64,
}

#[derive(Args)]
struct IssueCreateArgs {
    /// Issue title (default: derived from the message).
    #[arg(long)]
    title: Option<String>,
    /// Print the issue that would be filed (and its URL) without creating it.
    #[arg(long)]
    dry_run: bool,
    /// Free-text description of the problem.
    #[arg(num_args = 0.., allow_hyphen_values = true)]
    message: Vec<String>,
}

fn main() -> ExitCode {
    match dispatch(Cli::parse().command) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("beam: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn dispatch(command: Command) -> anyhow::Result<ExitCode> {
    match command {
        // Offline: render the in-code topics, no runtime/server/I/O beyond stdout.
        Command::Llm(args) => {
            let out = cli_std::llm::render(
                TOOL.project,
                TOOL.version,
                TOPICS,
                &args.topic,
                cli_std::llm::Format::parse(&args.format),
            )?;
            println!("{out}");
            Ok(ExitCode::SUCCESS)
        }
        // The standard ops are async; main stays sync and blocks on a local runtime.
        Command::Upgrade(args) => {
            block_on(cli_std::upgrade::run(
                &TOOL,
                cli_std::upgrade::Options {
                    check: args.check,
                    tag: args.tag,
                    force: false,
                    yes: true,
                },
            ))?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Issue { action } => {
            block_on(handle_issue(action))?;
            Ok(ExitCode::SUCCESS)
        }
        // Real GPU vector-search demo. Prints the Metal adapter, GPU-vs-CPU
        // recall, and timing; exits non-zero if no GPU adapter is available.
        Command::Bench(args) => {
            let metric = beam::collection::Metric::parse(&args.metric)
                .ok_or_else(|| anyhow::anyhow!("unknown metric: {}", args.metric))?;
            let index = beam::bench::IndexKind::parse(&args.index)
                .ok_or_else(|| anyhow::anyhow!("unknown index: {}", args.index))?;
            beam::bench::run(&beam::bench::BenchConfig {
                n: args.n,
                dim: args.dim,
                k: args.k,
                queries: args.queries,
                metric,
                index,
                nlist: args.nlist,
                nprobe: args.nprobe,
                m: args.m,
                rank: args.rank,
                filter_category: args.filter,
            })
        }
        // Placeholder service verbs — a consistent, tracked "not built yet" exit.
        Command::Serve => Ok(placeholder("HTTP service shell")),
        Command::Collections => Ok(placeholder("collection lifecycle")),
        Command::Index => Ok(placeholder("index lifecycle")),
        Command::Query => Ok(placeholder("vector query")),
        Command::Dockerfile => Ok(placeholder("dockerfile render")),
        Command::K8s => Ok(placeholder("k8s render/operator")),
    }
}

/// Emit the tracked diagnostic and the shared not-implemented exit code.
fn placeholder(feature: &str) -> ExitCode {
    eprintln!("beam: {}", beam::not_implemented(feature));
    ExitCode::from(NOT_IMPLEMENTED_EXIT)
}

/// `beam issue <verb>` — dispatch search/view/create to cli-std. `create` always
/// tags `project:beam`; `search` defaults to beam's own issues.
async fn handle_issue(action: IssueCmd) -> anyhow::Result<()> {
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
            .await
        }
        IssueCmd::View(args) => cli_std::issue::view(&TOOL, args.number).await,
        IssueCmd::Create(args) => {
            let message = join_words(args.message);
            cli_std::issue::create(
                &TOOL,
                cli_std::issue::CreateOptions {
                    title: issue_title(args.title, message.as_deref()),
                    message,
                    url: None,
                    repo: None,
                    label: vec![format!("project:{}", TOOL.project)],
                    dry_run: args.dry_run,
                    yes: true,
                },
            )
            .await
        }
    }
}

/// Join trailing words into a single optional message (`None` when blank).
fn join_words(words: Vec<String>) -> Option<String> {
    let joined = words.join(" ");
    (!joined.trim().is_empty()).then_some(joined)
}

/// The explicit title, else a `beam: <first line>` title derived from the message.
fn issue_title(explicit: Option<String>, message: Option<&str>) -> String {
    if let Some(title) = explicit.filter(|t| !t.trim().is_empty()) {
        return title;
    }
    let Some(message) = message.map(str::trim).filter(|m| !m.is_empty()) else {
        return "beam: issue report".to_string();
    };
    let head: String = message.lines().next().unwrap_or(message).chars().take(72).collect();
    format!("beam: {head}")
}

/// Run a future to completion on a fresh runtime. The standard CLI ops
/// (`upgrade`/`issue`) are async, but beam's other subcommands are sync, so
/// `main` stays sync and these block on a local runtime.
fn block_on<F: std::future::Future<Output = anyhow::Result<()>>>(fut: F) -> anyhow::Result<()> {
    tokio::runtime::Runtime::new()?.block_on(fut)
}
