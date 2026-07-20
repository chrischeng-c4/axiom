//! beam binary — the GPU-native vector database CLI.
//!
//! Standard agent-facing commands — `beam llm`, `beam upgrade`, `beam issue`
//! (the CONTRIBUTING.md CLI convention, via the shared `cli-std` lib) — sit
//! alongside `beam bench` (a GPU-vs-CPU vector-search demo over the in-memory
//! engine: flat, IVF-flat, and IVF-PQ indexes on wgpu/Metal) and the
//! placeholder service verbs `serve`, `spec`, `dockerfile`, `k8s`, `query`,
//! and `connect`. Agents start at `beam llm --topic outline`.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use clap::{Args, Parser, Subcommand, ValueEnum};
use beam::dx::*;



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
/// topic *summaries* below are what an agent sees in `beam llm --topic outline`.
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
            a GPU flat (brute-force) k-NN index, a GPU IVF-PQ (IVFADC) \
            approximate-nearest-neighbor index, and a CPU HNSW graph ANN index — the \
            GPU paths on wgpu (Metal on Apple Silicon, Vulkan on Linux/NVIDIA). IVF-PQ \
            prunes to the nprobe nearest cells and product-quantizes residuals; HNSW is \
            the default graph ANN every mainstream vector DB ships (build-then-query, \
            L2/Cosine/Dot). A query touches a small fraction of the corpus; recall is \
            verified against the flat oracle. `beam bench --index \
            flat|ivfflat|ivfpq|hnsw` runs the parity, recall, pruning, and timing \
            demo. Still to come: durable segments, an HTTP/2 query API, and k8s. \
            Capability roots live in apps/beam/README.md (epic #769).\n",
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
        summary: "Drive Beam from an agent: llm/upgrade/issue plus the serve/spec/dockerfile/k8s/query/connect verbs",
        body: "# beam — operations\n\n\
            Standard agent commands (shared cli-std):\n\
            - `beam llm --topic <t>` — offline docs (this).\n\
            - `beam upgrade [--version <tag>] [--check]` — self-update from beam@* releases.\n\
            - `beam issue search|view|create` — read/file project:beam issues.\n\n\
            Service:\n\
            - `beam serve [--host 127.0.0.1] [--port 7373]` — run the HTTP/2 (h2c) vector DB.\n\
            - `beam spec [--format openapi-json]` — output OpenAPI spec/schema details.\n\
            - `beam dockerfile render --variant source|release` — output Dockerfiles.\n\
            - `beam k8s crd/operator/instance render` — output Kubernetes manifests.\n\
            - `beam export --url ... --out ...` — dump server state to file.\n\
            - `beam import --url ... --file ...` — restore server state from file.\n\
            - `beam connect` — port-forward helper.\n\
            - `beam query` — one-shot query wrapper.\n",
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
    /// Report the resolved GPU backend + device and build provenance — confirms
    /// which GPU beam is using. wgpu selects it automatically: Metal on Apple
    /// Silicon, Vulkan on NVIDIA/Linux, DX12 on Windows (one WGSL codebase).
    Info,
    /// Run the HTTP/2 (h2c) vector-database service: bind `--host` and `--port`,
    /// and serve the REST API (collections + query) until Ctrl-C.
    Serve(ServeArgs),
    /// Print beam's machine-readable integration spec — offline, no server.
    /// Default: the OpenAPI 3 JSON document; `--format openapi-yaml` for
    /// LLM-readable OpenAPI YAML; `--format json-schema` for the data types.
    Spec(SpecArgs),
    /// Print runtime image Dockerfiles. Image construction is owned here, not
    /// by `k8s`, because the same artifact feeds compose, kind, and real registries.
    Dockerfile(DockerfileArgs),
    /// Kubernetes artifacts split by layer: cluster-scoped CRD, operator
    /// control plane, and app-namespace Beam instances.
    K8s(K8sArgs),
    /// Dump a running node's full snapshot to stdout or `--out`.
    /// Alias of `export`; this is ad hoc data movement, not scheduled backup.
    Dump(SnapshotExportArgs),
    /// Export a running node's full snapshot to stdout or `--out`.
    /// Use `backup` when you need destination sinks and retention.
    Export(SnapshotExportArgs),
    /// Load a snapshot from `--file` or stdin into a running node by replacing
    /// all engine state through `/admin/restore`.
    /// Alias of `import`.
    Load(SnapshotImportArgs),
    /// Import a snapshot from `--file` or stdin into a running node by replacing
    /// all engine state through `/admin/restore`.
    Import(SnapshotImportArgs),
    /// Fetch a snapshot from a running serving fleet's own `/admin/backup`
    /// and ship it to a destination (`file://` or `s3://`) via `libs/service-backup`.
    Backup(BackupArgs),
    /// Manage a `kubectl port-forward` for the duration of a wrapped command
    /// against a k8s-deployed Beam instance. Resolves a bearer token from the
    /// deployment's token-registry Secret when `--secret`/`--cr` is given.
    Connect(ConnectArgs),
    /// One-shot query wrappers against a reachable beam node: `query`, `upsert`,
    /// `delete`, `collections list`. Assembles the exact wire body `beam spec` publishes.
    Query(QueryArgs),
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
    /// residual refine), `ivfpq` (IVF + product-quantized residuals), or `hnsw`
    /// (CPU HNSW graph ANN).
    #[arg(long, value_parser = ["flat", "ivfflat", "ivfpq", "hnsw"], default_value = "flat")]
    index: String,
    /// IVF coarse-cell count (ivfflat / ivfpq). Also seeds the HNSW cluster count.
    #[arg(long, default_value_t = 256)]
    nlist: usize,
    /// Cells probed per query (ivfflat / ivfpq).
    #[arg(long, default_value_t = 16)]
    nprobe: usize,
    /// PQ subvector count (ivfpq); `dim` must be divisible by `m`.
    #[arg(long, default_value_t = 16)]
    m: usize,
    /// HNSW `M` — max connections per node per layer.
    #[arg(long = "hnsw-m", default_value_t = 16)]
    hnsw_m: usize,
    /// HNSW `ef_construction` — build-time beam width.
    #[arg(long = "ef-construction", default_value_t = 200)]
    ef_construction: usize,
    /// HNSW `ef_search` — query-time beam width.
    #[arg(long = "ef-search", default_value_t = 64)]
    ef_search: usize,
    /// Corpus intrinsic dimension.
    #[arg(long, default_value_t = 0)]
    rank: usize,
    /// Filtered k-NN demo.
    #[arg(long = "filter")]
    filter: Option<i64>,
    /// CRUD churn demo.
    #[arg(long, default_value_t = 0.0)]
    churn: f64,
    /// Batched-query size for `--index flat`.
    #[arg(long, default_value_t = 1)]
    batch: usize,
    /// Persistence round-trip demo.
    #[arg(long = "persist")]
    persist: Option<String>,
}

// <HANDWRITE gap="missing-generator:logic--6da9b12e" tracker="pending-tracker" reason="logic section in main.rs is hand-written pending codegen support">
/// `beam serve` flags.
#[derive(Args)]
pub struct ServeArgs {
    /// Bind address. K8s passes 0.0.0.0.
    #[arg(long, env = "BEAM_HOST", default_value = "127.0.0.1")]
    pub host: String,
    /// Client API port.
    #[arg(long, env = "BEAM_PORT", default_value_t = 7373)]
    pub port: u16,
    /// `trace|debug|info|warn|error`
    #[arg(long, env = "BEAM_LOG_LEVEL", default_value = "info")]
    pub log_level: String,
    /// Graceful drain window on SIGTERM.
    #[arg(long, env = "BEAM_GRACE_SECS", default_value_t = 30)]
    pub grace_secs: u64,
    /// Data directory for durable local persistence.
    #[arg(long, env = "BEAM_DATA_DIR")]
    pub data_dir: Option<String>,
}
// </HANDWRITE>

/// `beam spec` flags.
#[derive(Args)]
struct SpecArgs {
    /// Spec format: `openapi-json` (default), `openapi-yaml`, `json-schema`, `shapes`, or `fields`.
    #[arg(long, default_value = "openapi-json", value_parser = ["openapi-json", "openapi-yaml", "json-schema", "shapes", "fields"])]
    format: String,
    /// Return the field catalog. Alias for --format fields.
    #[arg(long)]
    fields: bool,
    /// Return the query-shape cookbook. Alias for --format shapes.
    #[arg(long)]
    shapes: bool,
}

/// `beam dockerfile` flags.
#[derive(Args)]
struct DockerfileArgs {
    #[command(subcommand)]
    cmd: DockerfileCmd,
}

#[derive(Subcommand)]
enum DockerfileCmd {
    /// Render a Dockerfile to stdout or `--out`.
    Render(DockerfileRenderArgs),
}

#[derive(Args)]
struct DockerfileRenderArgs {
    /// Which runtime image contract to render.
    #[arg(long, value_enum, default_value_t = DockerfileVariant::Release)]
    variant: DockerfileVariant,
    /// Release tag used by `--variant release`; accepts `0.4.7` or `beam@0.4.7`.
    #[arg(long)]
    version: Option<String>,
    /// Write to this path instead of stdout.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Clone, Copy, ValueEnum)]
enum DockerfileVariant {
    Source,
    Release,
}

/// `beam k8s` flags.
#[derive(Args)]
struct K8sArgs {
    #[command(subcommand)]
    cmd: K8sCmd,
}

#[derive(Subcommand)]
enum K8sCmd {
    /// Cluster-scoped API layer: render the Beam CRD.
    Crd(K8sCrdArgs),
    /// Operator control-plane layer: render or run the operator reconciler.
    Operator(K8sOperatorArgs),
    /// App namespace data-plane declaration: render a Beam custom resource.
    Instance(K8sInstanceArgs),
}

#[derive(Args)]
struct K8sCrdArgs {
    #[command(subcommand)]
    cmd: K8sCrdCmd,
}

#[derive(Subcommand)]
enum K8sCrdCmd {
    /// Render the Beam CustomResourceDefinition YAML.
    Render(K8sFileOutputArgs),
}

#[derive(Args)]
struct K8sOperatorArgs {
    #[command(subcommand)]
    cmd: Option<K8sOperatorCmd>,
}

#[derive(Subcommand)]
enum K8sOperatorCmd {
    /// Container entrypoint: run the reconcile controller.
    Run,
    /// Render operator namespace/RBAC/deployment YAML.
    Render(K8sOperatorRenderArgs),
}

#[derive(Args)]
struct K8sOperatorRenderArgs {
    /// Namespace that owns the operator control plane.
    #[arg(long, default_value = "beam-system")]
    namespace: String,
    /// Write to this path instead of stdout.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Args)]
struct K8sInstanceArgs {
    #[command(subcommand)]
    cmd: K8sInstanceCmd,
}

#[derive(Subcommand)]
enum K8sInstanceCmd {
    /// Render a namespaced `kind: Beam` custom resource.
    Render(K8sInstanceRenderArgs),
}

#[derive(Args)]
struct K8sInstanceRenderArgs {
    /// Built-in instance profile.
    #[arg(long, value_enum, default_value_t = K8sInstanceProfile::Dev)]
    profile: K8sInstanceProfile,
    /// Beam CR name.
    #[arg(long)]
    name: Option<String>,
    /// Namespace where the app-facing Beam instance lives.
    #[arg(long)]
    namespace: Option<String>,
    /// Serving image.
    #[arg(long)]
    image: Option<String>,
    /// Write to this path instead of stdout.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Clone, Copy, ValueEnum)]
enum K8sInstanceProfile {
    Dev,
    Staging,
    Prod,
    Template,
}

#[derive(Args)]
struct K8sFileOutputArgs {
    /// Write to this path instead of stdout.
    #[arg(long)]
    out: Option<PathBuf>,
}

/// `beam dump|export` flags: pulls Snapshot from a running serving fleet and writes exact bytes to stdout or a file.
#[derive(Args)]
struct SnapshotExportArgs {
    /// Base URL of a running beam serving node, e.g. `http://localhost:7373`.
    #[arg(long)]
    url: String,
    /// Write the Snapshot bytes to this path instead of stdout.
    #[arg(long)]
    out: Option<PathBuf>,
    /// Bearer token for the admin API.
    #[arg(long, env = "BEAM_BACKUP_TOKEN")]
    token: Option<String>,
}

/// `beam load|import` flags: reads Snapshot bytes and posts it to `/admin/restore`.
#[derive(Args)]
struct SnapshotImportArgs {
    /// Base URL of a running beam serving node, e.g. `http://localhost:7373`.
    #[arg(long)]
    url: String,
    /// Read Snapshot bytes from this path. Omit to read stdin.
    #[arg(long)]
    file: Option<PathBuf>,
    /// Bearer token for the admin API.
    #[arg(long, env = "BEAM_BACKUP_TOKEN")]
    token: Option<String>,
}

/// `beam backup` flags.
#[derive(Args)]
struct BackupArgs {
    /// Base URL of a running beam serving node.
    #[arg(long)]
    url: String,
    /// Destination URI: `file:///path` or `s3://bucket/prefix`.
    #[arg(long)]
    dest: String,
    /// Bearer token.
    #[arg(long, env = "BEAM_BACKUP_TOKEN")]
    token: Option<String>,
    /// Drop backup objects older than this many seconds.
    #[arg(long)]
    retention_secs: Option<u64>,
}

/// `beam connect` flags.
#[derive(Args)]
struct ConnectArgs {
    /// kubectl context to port-forward through.
    #[arg(long)]
    context: Option<String>,
    /// Namespace of the target Service.
    #[arg(long)]
    namespace: String,
    /// Target Service name.
    #[arg(long)]
    service: Option<String>,
    /// `Beam` CR name.
    #[arg(long)]
    cr: Option<String>,
    /// Local port to forward to.
    #[arg(long)]
    local_port: Option<u16>,
    /// Remote port.
    #[arg(long, default_value_t = 7373)]
    remote_port: u16,
    /// Secret name holding a `token-registry.json` key.
    #[arg(long)]
    secret: Option<String>,
    /// Minimum role the resolved token must cover.
    #[arg(long, value_enum, default_value_t = TokenRole::Admin)]
    role: TokenRole,
    /// Collection id the resolved token must be authorized against.
    #[arg(long)]
    collection: Option<String>,
    /// The command to run with `BEAM_URL` (and `BEAM_TOKEN`) set.
    #[arg(last = true, required = true)]
    command: Vec<String>,
}

#[derive(Clone, Copy, ValueEnum)]
enum TokenRole {
    Read,
    Write,
    Admin,
}

impl From<TokenRole> for cli_std::connect::Role {
    fn from(r: TokenRole) -> Self {
        match r {
            TokenRole::Read => cli_std::connect::Role::Read,
            TokenRole::Write => cli_std::connect::Role::Write,
            TokenRole::Admin => cli_std::connect::Role::Admin,
        }
    }
}

/// `beam query` flags.
#[derive(Args)]
struct QueryArgs {
    #[command(subcommand)]
    command: QueryCommand,
}

#[derive(Subcommand)]
enum QueryCommand {
    /// Collection-level read helpers.
    Collections(QueryCollectionsArgs),
    /// Run a k-NN query.
    Query(QueryCollectionArgs),
    /// Batch upsert vectors.
    Upsert(QueryUpsertArgs),
    /// Delete one vector.
    Delete(QueryDeleteArgs),
}

#[derive(Args)]
struct QueryCollectionsArgs {
    #[command(subcommand)]
    command: QueryCollectionsCommand,
}

#[derive(Subcommand)]
enum QueryCollectionsCommand {
    /// List collection names and sizes.
    List(QueryCollectionsListArgs),
}

#[derive(Args)]
struct QueryCollectionsListArgs {
    #[command(flatten)]
    target: QueryTarget,
}

#[derive(Args)]
struct QueryCollectionArgs {
    #[command(flatten)]
    target: QueryTarget,
    /// Target collection name.
    #[arg(long)]
    collection: String,
    /// Query vector elements, comma-separated (e.g. `0.1,0.2,0.9`).
    #[arg(long, value_delimiter = ',', required = true)]
    vector: Vec<f32>,
    /// Top-k neighbors to return.
    #[arg(long, default_value_t = 10)]
    k: usize,
}

#[derive(Args)]
struct QueryUpsertArgs {
    #[command(flatten)]
    target: QueryTarget,
    /// Target collection name.
    #[arg(long)]
    collection: String,
    /// Vector item to upsert as `ID:VEC` (e.g. `doc_1=[0.1,0.2]`).
    #[arg(long = "item", value_name = "ID:VEC", required = true)]
    items: Vec<String>,
}

#[derive(Args)]
struct QueryDeleteArgs {
    #[command(flatten)]
    target: QueryTarget,
    /// Target collection name.
    #[arg(long)]
    collection: String,
    /// Vector ID to delete.
    #[arg(long, required = true)]
    id: String,
}

#[derive(Args)]
struct QueryTarget {
    /// Base URL of a running beam serving node.
    #[arg(long, env = "BEAM_URL")]
    url: Option<String>,
    /// Bearer token.
    #[arg(long, env = "BEAM_TOKEN")]
    token: Option<String>,
    /// kubectl context.
    #[arg(long)]
    context: Option<String>,
    /// Namespace of the target Service.
    #[arg(long)]
    namespace: Option<String>,
    /// Secret name holding a `token-registry.json` key.
    #[arg(long)]
    secret: Option<String>,
    /// Role to verify or resolve a token for.
    #[arg(long, value_enum, default_value_t = TokenRole::Admin)]
    role: TokenRole,
}

/// `beam upgrade` flags (the convention surface: `--version` + `--check`).
#[derive(Args)]
struct UpgradeArgs {
    /// Install a specific release tag, e.g. `beam@0.4.7` or `0.4.7`.
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





use anyhow::Context;

// <HANDWRITE gap="missing-generator:logic" tracker="#2147" reason="logic section in main.rs is hand-written pending codegen support">
fn main() -> ExitCode {
    match dispatch(Cli::parse().command) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("beam: {err:#}");
            ExitCode::FAILURE
        }
    }
}
// </HANDWRITE>

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
        // Real GPU vector-search benchmark. Prints the Metal adapter, GPU-vs-CPU
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
                hnsw_m: args.hnsw_m,
                ef_construction: args.ef_construction,
                ef_search: args.ef_search,
                rank: args.rank,
                filter_category: args.filter,
                churn: args.churn,
                persist: args.persist,
                batch: args.batch,
            })
        }
        // Report the resolved GPU backend + device — the dual-platform proof.
        Command::Info => {
            println!("beam {} ({}, git {})", TOOL.version, TOOL.target, TOOL.git_sha);
            match beam::gpu::GpuContext::new() {
                Some(gpu) => {
                    let (backend, name) = gpu.adapter_info();
                    println!("GPU backend: {backend}");
                    println!("GPU device:  {name}");
                    println!(
                        "wgpu selects the backend automatically: Metal (Apple Silicon), \
                         Vulkan (NVIDIA/Linux), DX12 (Windows) — one WGSL codebase."
                    );
                }
                None => println!("GPU: no adapter available on this host (CPU-only)"),
            }
            Ok(ExitCode::SUCCESS)
        }
        // Real HTTP/2 (h2c) vector-database service. Blocks until Ctrl-C/SIGTERM.
        Command::Serve(args) => {
            let addr = format!("{}:{}", args.host, args.port);
            // <HANDWRITE gap="missing-generator:logic--6da9b12e" tracker="pending-tracker" reason="wire data_dir to serve call">
            let data_path = args.data_dir.map(std::path::PathBuf::from);
            block_on(beam::service::serve(&addr, data_path))?;
            // </HANDWRITE>
            Ok(ExitCode::SUCCESS)
        }
        Command::Spec(args) => {
            let out = if args.shapes || args.fields {
                serde_json::to_string_pretty(&serde_json::json!({}))?
            } else {
                match args.format.as_str() {
                    "openapi-json" => beam::spec::openapi_json(),
                    "openapi-yaml" => beam::spec::openapi_yaml(),
                    "json-schema" => beam::spec::json_schema_json(),
                    _ => beam::spec::openapi_json(),
                }
            };
            println!("{out}");
            Ok(ExitCode::SUCCESS)
        }
        Command::Dockerfile(args) => {
            match args.cmd {
                DockerfileCmd::Render(render) => {
                    let (file_name, body) = match render.variant {
                        DockerfileVariant::Source => ("Dockerfile", render_source_dockerfile()),
                        DockerfileVariant::Release => (
                            "Dockerfile.release",
                            render_release_dockerfile(render.version.as_deref()),
                        ),
                    };
                    let release = matches!(render.variant, DockerfileVariant::Release);
                    let version = render.version.clone();
                    write_or_print(render.out.as_deref(), file_name, &body, move |target| {
                        dockerfile_next_command(release, version.as_deref(), target)
                    })?;
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::K8s(args) => {
            match args.cmd {
                K8sCmd::Crd(crd) => {
                    match crd.cmd {
                        K8sCrdCmd::Render(out) => {
                            write_or_print(out.out.as_deref(), "crd.yaml", &render_crd_yaml(), |target| {
                                format!("kubectl apply -f {}", target.display())
                            })?;
                        }
                    }
                }
                K8sCmd::Operator(operator) => {
                    match operator.cmd.unwrap_or(K8sOperatorCmd::Run) {
                        K8sOperatorCmd::Run => {
                            println!("beam operator: reconcile controller running (placeholder)");
                        }
                        K8sOperatorCmd::Render(render) => {
                            let body = render_operator_yaml(&render.namespace);
                            write_or_print(render.out.as_deref(), "operator.yaml", &body, |target| {
                                format!("kubectl apply -f {}", target.display())
                            })?;
                        }
                    }
                }
                K8sCmd::Instance(instance) => {
                    match instance.cmd {
                        K8sInstanceCmd::Render(render) => {
                            let name = render.name.as_deref().unwrap_or("beam");
                            let namespace = render.namespace.as_deref().unwrap_or("default");
                            let profile_str = match render.profile {
                                K8sInstanceProfile::Dev => "dev",
                                K8sInstanceProfile::Staging => "staging",
                                K8sInstanceProfile::Prod => "prod",
                                K8sInstanceProfile::Template => "template",
                            };
                            let body = render_instance_yaml(profile_str, name, namespace, render.image.as_deref());
                            write_or_print(render.out.as_deref(), "beam.yaml", &body, |target| {
                                format!("kubectl apply -f {}", target.display())
                            })?;
                        }
                    }
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Dump(args) | Command::Export(args) => {
            let client = reqwest::blocking::Client::new();
            let mut req = client.get(format!("{}/admin/backup", args.url.trim_end_matches('/')));
            if let Some(tok) = args.token {
                req = req.bearer_auth(tok);
            }
            let resp = req.send()?;
            if !resp.status().is_success() {
                anyhow::bail!("export failed (status {}): {}", resp.status(), resp.text()?);
            }
            let payload = resp.bytes()?;
            if let Some(out) = args.out {
                if let Some(parent) = out.parent().filter(|p| !p.as_os_str().is_empty()) {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&out, &payload)?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "status": "exported",
                        "path": out,
                        "bytes": payload.len(),
                    }))?
                );
            } else {
                std::io::Write::write_all(&mut std::io::stdout().lock(), &payload)?;
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Load(args) | Command::Import(args) => {
            let payload = match &args.file {
                Some(path) => std::fs::read(path)?,
                None => {
                    let mut buf = Vec::new();
                    std::io::Read::read_to_end(&mut std::io::stdin(), &mut buf)?;
                    buf
                }
            };
            let client = reqwest::blocking::Client::new();
            let mut req = client.post(format!("{}/admin/restore", args.url.trim_end_matches('/')));
            if let Some(tok) = args.token {
                req = req.bearer_auth(tok);
            }
            let resp = req.body(payload).send()?;
            if !resp.status().is_success() {
                anyhow::bail!("import failed (status {}): {}", resp.status(), resp.text()?);
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "restored",
                    "url": args.url.trim_end_matches('/'),
                }))?
            );
            Ok(ExitCode::SUCCESS)
        }
        Command::Backup(args) => {
            block_on(async {
                let dest = service_backup::BackupDestination::from_uri(&args.dest)?;
                let retention = match args.retention_secs {
                    Some(secs) => service_backup::RetentionPolicy::max_age_seconds(secs),
                    None => service_backup::RetentionPolicy::default(),
                };
                let result = beam::backup::run_backup(
                    &args.url,
                    args.token.as_deref(),
                    &dest,
                    &retention,
                )
                .await?;
                let mut out = serde_json::to_value(&result)?;
                if let serde_json::Value::Object(ref mut map) = out {
                    map.insert(
                        "next".to_string(),
                        serde_json::Value::String(format!(
                            "curl -sS -X POST {}/admin/restore -H 'Content-Type: application/json' --data-binary @...",
                            args.url.trim_end_matches('/')
                        )),
                    );
                }
                println!("{}", serde_json::to_string_pretty(&out)?);
                Ok(())
            })?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Connect(args) => {
            block_on(async {
                let service = args.service.clone().or_else(|| args.cr.clone()).context("--service or --cr is required")?;
                let secret = match args.secret.clone() {
                    Some(s) => Some(s),
                    None => match &args.cr {
                        Some(cr) => cli_std::connect::resolve_cr_tokens_secret(
                            args.context.as_deref(),
                            &args.namespace,
                            "beam",
                            cr,
                        )?,
                        None => None,
                    },
                };
                let local_port = match args.local_port {
                    Some(p) => p,
                    None => cli_std::connect::free_local_port()?,
                };
                let mut pf_cmd = std::process::Command::new("kubectl");
                if let Some(ctx) = &args.context {
                    pf_cmd.args(["--context", ctx]);
                }
                pf_cmd.args([
                    "port-forward",
                    "-n",
                    &args.namespace,
                    &format!("svc/{service}"),
                    &format!("{local_port}:{}", args.remote_port),
                ]);
                pf_cmd.stdout(std::process::Stdio::null());
                pf_cmd.stderr(std::process::Stdio::null());
                let _forward = cli_std::connect::ChildGuard::spawn(&mut pf_cmd).context("start kubectl port-forward")?;
                cli_std::connect::wait_for_local_port_ready(local_port, Duration::from_secs(30))?;

                let target = QueryTarget {
                    url: Some(format!("http://127.0.0.1:{local_port}")),
                    token: args.collection.as_deref().and_then(|c| {
                        cli_std::connect::resolve_token(
                            None,
                            args.context.as_deref(),
                            Some(&args.namespace),
                            secret.as_deref(),
                            args.role.into(),
                            Some(c),
                        ).ok().flatten()
                    }),
                    context: args.context.clone(),
                    namespace: Some(args.namespace.clone()),
                    secret,
                    role: args.role,
                };

                let base_url = format!("http://127.0.0.1:{local_port}");
                let (program, rest) = args.command.split_first().context("wrapped command is empty")?;
                let mut child_cmd = std::process::Command::new(program);
                child_cmd.args(rest);
                child_cmd.env("BEAM_URL", &base_url);
                if let Some(tok) = &target.token {
                    child_cmd.env("BEAM_TOKEN", tok);
                }
                let status = child_cmd.status().context("run wrapped command")?;
                if !status.success() {
                    anyhow::bail!("wrapped command exited with {status}");
                }
                Ok(())
            })?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Query(args) => {
            block_on(async {
                match args.command {
                    QueryCommand::Collections(col_args) => {
                        match col_args.command {
                            QueryCollectionsCommand::List(list_args) => {
                                let url = list_args.target.url.clone().context("--url or BEAM_URL is required")?;
                                let client = reqwest::Client::new();
                                let mut req = client.get(format!("{}/v1/collections", url.trim_end_matches('/')));
                                if let Some(tok) = &list_args.target.token {
                                    req = req.bearer_auth(tok);
                                }
                                let resp = req.send().await?;
                                if !resp.status().is_success() {
                                    anyhow::bail!("list collections failed (status {}): {}", resp.status(), resp.text().await?);
                                }
                                println!("{}", resp.text().await?);
                            }
                        }
                    }
                    QueryCommand::Query(query_args) => {
                        let url = query_args.target.url.clone().context("--url or BEAM_URL is required")?;
                        let client = reqwest::Client::new();
                        let mut req = client.post(format!("{}/v1/collections/{}/query", url.trim_end_matches('/'), query_args.collection));
                        if let Some(tok) = &query_args.target.token {
                            req = req.bearer_auth(tok);
                        }
                        req = req.json(&serde_json::json!({
                            "vector": query_args.vector,
                            "k": query_args.k,
                        }));
                        let resp = req.send().await?;
                        if !resp.status().is_success() {
                            anyhow::bail!("query failed (status {}): {}", resp.status(), resp.text().await?);
                        }
                        println!("{}", resp.text().await?);
                    }
                    QueryCommand::Upsert(upsert_args) => {
                        let url = upsert_args.target.url.clone().context("--url or BEAM_URL is required")?;
                        let mut items = Vec::new();
                        for raw in upsert_args.items {
                            let (id, vec_str) = raw.split_once(':').context("invalid item format (expected ID:VEC)")?;
                            let vector: Vec<f32> = serde_json::from_str(vec_str).context("failed to parse vector list")?;
                            items.push(serde_json::json!({ "id": id, "vector": vector }));
                        }
                        let client = reqwest::Client::new();
                        let req = client.post(format!("{}/v1/collections/{}/vectors", url.trim_end_matches('/'), upsert_args.collection));
                        let req = if let Some(tok) = &upsert_args.target.token {
                            req.bearer_auth(tok)
                        } else {
                            req
                        };
                        let req = req.json(&serde_json::json!({ "items": items }));
                        let resp = req.send().await?;
                        if !resp.status().is_success() {
                            anyhow::bail!("upsert failed (status {}): {}", resp.status(), resp.text().await?);
                        }
                        println!("{}", resp.text().await?);
                    }
                    QueryCommand::Delete(delete_args) => {
                        let url = delete_args.target.url.clone().context("--url or BEAM_URL is required")?;
                        let client = reqwest::Client::new();
                        let mut req = client.delete(format!("{}/v1/collections/{}/vectors/{}", url.trim_end_matches('/'), delete_args.collection, delete_args.id));
                        if let Some(tok) = &delete_args.target.token {
                            req = req.bearer_auth(tok);
                        }
                        let resp = req.send().await?;
                        if !resp.status().is_success() {
                            anyhow::bail!("delete failed (status {}): {}", resp.status(), resp.text().await?);
                        }
                        println!("{}", resp.text().await?);
                    }
                }
                Ok(())
            })?;
            Ok(ExitCode::SUCCESS)
        }
    }
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
