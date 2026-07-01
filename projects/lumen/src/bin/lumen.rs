// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-bin-lumen-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! `lumen` — the single agent-first CLI: `serve` (serving node), `spec` /
//! `llm` (offline integration contract + agent topics), and `k8s` (operator
//! + CRD generation). Agents start here: `lumen llm outline`.
//!
//! A serving node is symmetric: it answers reads from its local
//! materialized index and accepts writes by publishing them to the
//! configured write log. In single-node mode that log is local; in legacy
//! NATS mode it is external; in primary-replica mode Lumen owns ordering and
//! replication via raft_core. Apply happens in the background subscribe loop —
//! see `coordinator` / `wal`.
//!
//! ```text
//! lumen serve                          # single node, in-process log, :7373
//! lumen serve --wal raft               # k8s StatefulSet / HA mode
//! lumen serve --host 0.0.0.0 --port 7373 --log-format json
//! ```

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use tracing_subscriber::EnvFilter;

use lumen::auth::AuthConfig;
use lumen::coordinator::WriteCoordinator;
use lumen::rdb::{LocalFsRdbStore, RdbSnapshot, RdbStore};
use lumen::storage::Engine;
use lumen::wal::{MemWal, SharedWal};
use lumen::wal_nats::NatsWal;

#[derive(Parser)]
#[command(
    name = "lumen",
    version,
    about = "lumen — search specialist (serving node + CLI)"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run a serving node (HTTP API + background apply loop).
    Serve(ServeArgs),
    /// Print lumen's machine-readable integration spec — offline, no server.
    /// Default: the OpenAPI 3 JSON document; `--format openapi-yaml` for
    /// LLM-readable OpenAPI YAML; `--format json-schema` for the data types;
    /// `--shapes` for the query-shape cookbook; `--fields` for the field-type /
    /// analyzer catalog.
    Spec(SpecArgs),
    /// Print agent-facing LLM topics — offline, no server. `outline` maps the
    /// available topics; `workflow` covers mental model +
    /// declare→ingest→search→hydrate; `integration` covers Postgres/AlloyDB
    /// adapter boundaries; `quickstart` is copy-paste end-to-end; `recipes`
    /// are task → ready-to-POST query bodies. Markdown by default; `--format
    /// json` for a machine-readable form.
    Llm(LlmArgs),
    /// Print runtime image Dockerfiles. Image construction is owned here, not
    /// by `k8s`, because the same artifact feeds compose, kind, and real
    /// registries.
    Dockerfile(DockerfileArgs),
    /// Kubernetes artifacts split by layer: cluster-scoped CRD, operator
    /// control plane, and app-namespace Lumen instances.
    K8s(K8sArgs),
    /// Self-update this binary from a published GitHub release. Resolves the
    /// running target + version, downloads the matching `lumen-<target>.tar.gz`,
    /// verifies its sha256, and atomically replaces the running executable.
    /// `--check` reports the available version without changing anything.
    // @spec projects/lumen/tech-design/interfaces/cli/lumen-upgrade-self-update-cli-from-github-releases.md
    Upgrade(UpgradeArgs),
    /// Search, view, and file Lumen issues on the axiom tracker.
    /// `search` and `view` read existing `project:lumen` issues; `create`
    /// files a diagnostics-rich issue tagged `project:lumen`.
    // @spec projects/lumen/tech-design/interfaces/cli/lumen-issue-search-view-create-shared-cli-standard.md
    Issue(IssueArgs),
}

#[derive(clap::Args)]
struct DockerfileArgs {
    #[command(subcommand)]
    cmd: DockerfileCmd,
}

#[derive(Subcommand)]
enum DockerfileCmd {
    /// Render a Dockerfile to stdout or `--out`.
    Render(DockerfileRenderArgs),
}

#[derive(clap::Args)]
struct DockerfileRenderArgs {
    /// Which runtime image contract to render.
    #[arg(long, value_enum, default_value_t = DockerfileVariant::Release)]
    variant: DockerfileVariant,
    /// Release tag used by `--variant release`; accepts `0.4.5` or `lumen@0.4.5`.
    #[arg(long)]
    version: Option<String>,
    /// Write to this path instead of stdout. A directory receives
    /// `Dockerfile` or `Dockerfile.release`.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Clone, Copy, ValueEnum)]
enum DockerfileVariant {
    /// Build from the workspace source tree.
    Source,
    /// Fetch and verify a published `lumen@<version>` release binary.
    Release,
}

#[derive(clap::Args)]
struct K8sArgs {
    #[command(subcommand)]
    cmd: K8sCmd,
}

#[derive(Subcommand)]
enum K8sCmd {
    /// Cluster-scoped API layer: render the Lumen CRD.
    Crd(K8sCrdArgs),
    /// Operator control-plane layer: render/install assets or run the controller.
    Operator(K8sOperatorArgs),
    /// App namespace data-plane declaration: render a Lumen custom resource.
    Instance(K8sInstanceArgs),
}

#[derive(clap::Args)]
struct K8sCrdArgs {
    #[command(subcommand)]
    cmd: K8sCrdCmd,
}

#[derive(Subcommand)]
enum K8sCrdCmd {
    /// Render the Lumen CustomResourceDefinition YAML.
    Render(K8sFileOutputArgs),
}

#[derive(clap::Args)]
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

#[derive(clap::Args)]
struct K8sOperatorRenderArgs {
    /// Namespace that owns the operator control plane.
    #[arg(long, default_value = "lumen-system")]
    namespace: String,
    /// Write to this path instead of stdout. A directory receives
    /// `operator.yaml`.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(clap::Args)]
struct K8sInstanceArgs {
    #[command(subcommand)]
    cmd: K8sInstanceCmd,
}

#[derive(Subcommand)]
enum K8sInstanceCmd {
    /// Render a namespaced `kind: Lumen` custom resource.
    Render(K8sInstanceRenderArgs),
}

#[derive(clap::Args)]
struct K8sInstanceRenderArgs {
    /// Built-in instance profile.
    #[arg(long, value_enum, default_value_t = K8sInstanceProfile::Dev)]
    profile: K8sInstanceProfile,
    /// Lumen CR name.
    #[arg(long)]
    name: Option<String>,
    /// Namespace where the app-facing Lumen instance lives.
    #[arg(long)]
    namespace: Option<String>,
    /// Serving image. Defaults are profile-specific.
    #[arg(long)]
    image: Option<String>,
    /// Write to this path instead of stdout. A directory receives `lumen.yaml`.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Clone, Copy, ValueEnum)]
enum K8sInstanceProfile {
    /// Small local/kind CR: one serving pod, embedded WAL, auth disabled.
    Dev,
    /// Pre-prod CR: json logs, raft data-plane shape, observability enabled.
    Staging,
    /// Production-shape CR: auth required, json logs, raft data-plane shape.
    Prod,
    /// Fill-in-the-blanks CR skeleton for app teams.
    Template,
}

#[derive(clap::Args)]
struct K8sFileOutputArgs {
    /// Write to this path instead of stdout.
    #[arg(long)]
    out: Option<PathBuf>,
}

/// `lumen upgrade` flags.
/// @spec projects/lumen/tech-design/interfaces/cli/lumen-upgrade-self-update-cli-from-github-releases.md
#[derive(clap::Args)]
struct UpgradeArgs {
    /// Report the current and latest version without modifying the binary.
    #[arg(long)]
    check: bool,
    /// Install this exact version (`0.4.3` or `lumen@0.4.3`) instead of the latest.
    #[arg(long = "version")]
    tag: Option<String>,
    /// Reinstall even when already on the selected version.
    #[arg(long)]
    force: bool,
    /// Skip the confirmation prompt.
    #[arg(short = 'y', long)]
    yes: bool,
}

/// `lumen issue <search|view|create>` flags.
/// @spec projects/lumen/tech-design/interfaces/cli/lumen-issue-search-view-create-shared-cli-standard.md
#[derive(clap::Args)]
struct IssueArgs {
    #[command(subcommand)]
    command: IssueCommand,
}

#[derive(Subcommand)]
enum IssueCommand {
    /// Search Lumen issues (project:lumen); omit the query to list recent.
    Search(IssueSearchArgs),
    /// Print one issue by number.
    View(IssueViewArgs),
    /// File a diagnostics-rich Lumen issue.
    Create(IssueCreateArgs),
}

#[derive(clap::Args)]
struct IssueSearchArgs {
    /// Search text. Omit to list recent issues.
    #[arg(value_name = "QUERY", num_args = 0..)]
    query: Vec<String>,
    /// Issue state: open, closed, or all.
    #[arg(long, default_value = "open", value_parser = ["open", "closed", "all"])]
    state: String,
    /// Max results.
    #[arg(long, default_value_t = 20)]
    limit: u32,
}

#[derive(clap::Args)]
struct IssueViewArgs {
    /// Issue number.
    number: u64,
}

#[derive(clap::Args)]
struct IssueCreateArgs {
    /// Issue title.
    #[arg(short = 't', long)]
    title: Option<String>,
    /// Free-text description of the problem (trailing words; placed above the
    /// diagnostics block). The only positional — parameters are flags.
    #[arg(value_name = "MSG", num_args = 0..)]
    message: Vec<String>,
    /// Include a running node's `/version`+`/healthz` (e.g. http://localhost:7373).
    #[arg(long)]
    url: Option<String>,
    /// Target repository (`owner/name`); defaults to lumen's release repo.
    #[arg(long)]
    repo: Option<String>,
    /// Add a label (repeatable).
    #[arg(long)]
    label: Vec<String>,
    /// Assemble and print the report without submitting anything.
    #[arg(long)]
    dry_run: bool,
    /// Skip the confirmation prompt.
    #[arg(short = 'y', long)]
    yes: bool,
}

#[derive(Clone, Copy, ValueEnum)]
enum LlmTopic {
    /// Topic map for agent context selection (default).
    Outline,
    /// Product model, declare → ingest → search → hydrate, and non-goals.
    Workflow,
    /// Recommended database/pubsub adapter boundary.
    Integration,
    /// A copy-paste create → index → search walkthrough.
    Quickstart,
    /// Bearer-token auth, token registry schema, and Secret projection.
    Auth,
    /// Task → ready-to-POST query bodies (same source as `spec --shapes`).
    Recipes,
}

#[derive(Clone, Copy, ValueEnum)]
enum LlmFormat {
    /// Human/agent-readable Markdown (default).
    Md,
    /// Machine-readable JSON.
    Json,
}

#[derive(Parser)]
struct LlmArgs {
    /// Which agent-facing topic to print.
    #[arg(long, value_enum, default_value_t = LlmTopic::Outline)]
    topic: LlmTopic,
    /// Output format.
    #[arg(long, value_enum, default_value_t = LlmFormat::Md)]
    format: LlmFormat,
}

#[derive(Clone, Copy, PartialEq, ValueEnum)]
enum WalBackend {
    /// Auto-detect (default, k8s-native): a StatefulSet with
    /// `REPLICAS_PER_SHARD > 1` runs raft (replica/HA mode); a single replica —
    /// or no cluster context (local dev) — runs embedded. An explicit
    /// `--wal <backend>` overrides this.
    Auto,
    /// In-process log. Single-node / dev. No external dependency.
    Embedded,
    /// NATS JetStream legacy backend.
    Nats,
    /// Lumen-owned raft_core replication (#515). HA without an external broker.
    #[cfg(feature = "raft-wal")]
    Raft,
}

/// Resolve `--wal auto` to a concrete backend, k8s-native: a StatefulSet with
/// `REPLICAS_PER_SHARD > 1` (the downward-API value) runs raft; one replica — or
/// no cluster context (the env unset, e.g. local dev) — runs embedded. An
/// explicit `--wal <backend>` passes through unchanged.
fn resolve_wal_backend(requested: WalBackend) -> WalBackend {
    if requested != WalBackend::Auto {
        return requested;
    }
    #[cfg(feature = "raft-wal")]
    if raft_host::cluster::replica_mode() {
        tracing::info!("wal=auto → raft (StatefulSet REPLICAS_PER_SHARD > 1)");
        return WalBackend::Raft;
    }
    tracing::info!("wal=auto → embedded (single replica / no cluster context)");
    WalBackend::Embedded
}

#[derive(Clone, Copy, ValueEnum)]
enum LogFormat {
    Pretty,
    Json,
}

/// Cold-start / snapshot persistence mode for `--data-dir` (Stage 2 Phase 2f-2).
/// Selected at runtime via `--persistence`; defaults to the CBOR RDB, so the
/// default `serve` path is byte-identical to today unless `segment` is passed.
#[derive(Clone, Copy, PartialEq, ValueEnum)]
enum Persistence {
    /// CBOR RDB blob (`rdb-<seq>.lrb`) — the default, byte-identical to today.
    Cbor,
    /// Columnar segment checkpoint (`gen-<seq>/<collection>/...`) — the disk
    /// engine as persistence. Cold start reopens segments WITHOUT a whole-
    /// collection load; the periodic snapshotter re-seals (re-seal-capable).
    Segment,
}

#[derive(Clone, Copy, ValueEnum)]
enum SpecFormat {
    /// Full OpenAPI 3 document as JSON (default).
    Openapi,
    /// Full OpenAPI 3 document as YAML for LLM/agent reading.
    #[value(alias = "yaml", alias = "openapi.yaml")]
    OpenapiYaml,
    /// Just the component schemas (request/response data types).
    JsonSchema,
}

#[derive(Parser)]
struct SpecArgs {
    /// Generate a typed client from this spec instead of printing it.
    /// @spec projects/lumen/tech-design/interfaces/cli/lumen-spec-gen-generate-a-typed-client-ts-py-rust-from-lumen-s-o.md
    #[command(subcommand)]
    gen: Option<SpecSub>,
    /// Schema format to emit when neither `--shapes` nor `--fields` is set.
    #[arg(long, value_enum, default_value_t = SpecFormat::Openapi)]
    format: SpecFormat,
    /// Emit the query-shape cookbook (canonical request examples) instead.
    #[arg(long)]
    shapes: bool,
    /// Emit the field-type / analyzer catalog instead.
    #[arg(long)]
    fields: bool,
}

/// `lumen spec` subcommands.
#[derive(Subcommand)]
enum SpecSub {
    /// Generate a typed API client (TypeScript / Python / Rust) from lumen's
    /// OpenAPI document, written into `--out`.
    Gen(GenArgs),
}

#[derive(Parser)]
struct GenArgs {
    /// Target language for the generated client.
    #[arg(long, value_enum)]
    lang: GenLang,
    /// Output directory for the generated files.
    #[arg(long)]
    out: PathBuf,
    /// HTTP backend for the TypeScript client (ignored for py/rust).
    #[arg(long, value_enum, default_value_t = GenHttp::Fetch)]
    http: GenHttp,
}

#[derive(Clone, Copy, ValueEnum)]
enum GenLang {
    /// TypeScript: types + fetch/axios client + TanStack Query hooks.
    Ts,
    /// Python: pydantic models + generated sync/async HTTP/2 runtime.
    Py,
    /// Rust: serde models + reqwest client.
    Rust,
}

#[derive(Clone, Copy, ValueEnum)]
enum GenHttp {
    Fetch,
    Axios,
}

#[derive(Parser)]
struct ServeArgs {
    /// Bind address. K8s passes 0.0.0.0.
    #[arg(long, env = "LUMEN_HOST", default_value = "127.0.0.1")]
    host: String,
    /// Client API port. 7373 avoids the usual collisions (8080/9200/9000).
    #[arg(long, env = "LUMEN_PORT", default_value_t = 7373)]
    port: u16,
    /// `trace|debug|info|warn|error` (overrides via RUST_LOG still apply).
    #[arg(long, env = "LUMEN_LOG_LEVEL", default_value = "info")]
    log_level: String,
    /// Log output format.
    #[arg(long, env = "LUMEN_LOG_FORMAT", value_enum, default_value_t = LogFormat::Pretty)]
    log_format: LogFormat,
    /// Write-log backend.
    #[arg(long = "wal", env = "LUMEN_WAL", value_enum, default_value_t = WalBackend::Auto)]
    wal: WalBackend,
    /// NATS URL (used when `--wal nats`).
    #[arg(long, env = "LUMEN_NATS_URL", default_value = "nats://localhost:4222")]
    nats_url: String,
    /// Max seconds to keep retrying the initial NATS connect before giving
    /// up. A serving node started before its broker (common during a k8s
    /// rollout) retries with backoff instead of crash-looping.
    #[arg(long, env = "LUMEN_NATS_CONNECT_TIMEOUT_SECS", default_value_t = 120)]
    nats_connect_timeout_secs: u64,
    /// Data dir for raft hard state (used when `--wal raft`). A PVC in k8s.
    #[cfg(feature = "raft-wal")]
    #[arg(
        long,
        env = "LUMEN_RAFT_DATA_DIR",
        default_value = "/var/lib/lumen/raft"
    )]
    raft_data_dir: String,
    /// Peer port for raft RPCs (used when `--wal raft`; multi-pod, Slice 2).
    #[cfg(feature = "raft-wal")]
    #[arg(long, env = "LUMEN_RAFT_PORT", default_value_t = 7374)]
    raft_port: u16,
    /// Shard count for client-side routing (`crc32(collection) % N`).
    /// Install-time topology constant.
    #[arg(long, env = "SHARD_COUNT", default_value_t = 1)]
    shard_count: u32,
    /// Directory for RDB snapshots (cold-start baseline). When unset,
    /// no snapshots are taken and a node rebuilds from the full log.
    #[arg(long, env = "LUMEN_DATA_DIR")]
    data_dir: Option<String>,
    /// Persistence mode for `--data-dir`: `cbor` (the CBOR RDB, default) or
    /// `segment` (the columnar disk-engine checkpoint). Defaults to `cbor`; pass
    /// `--persistence=segment` to opt into the disk tier.
    #[arg(long = "persistence", env = "LUMEN_PERSISTENCE", value_enum, default_value_t = Persistence::Cbor)]
    persistence: Persistence,
    /// Comma-separated segment-checkpoint roots to serve as read shards. Each
    /// root must contain a committed `gen-<seq>/` checkpoint. When set, search
    /// requests fan in across these roots through the API SearchBackend seam;
    /// writes still apply to the node's local engine/log.
    #[arg(long, env = "LUMEN_SEARCH_SHARD_SEGMENT_DIRS", value_delimiter = ',')]
    search_shard_segment_dirs: Vec<PathBuf>,
    /// Seconds between RDB snapshots when `--data-dir` is set.
    #[arg(long, env = "LUMEN_SNAPSHOT_SECS", default_value_t = 300)]
    snapshot_secs: u64,
    /// Graceful drain window on SIGTERM.
    #[arg(long, env = "LUMEN_GRACE_SECS", default_value_t = 30)]
    grace_secs: u64,
    /// OTLP gRPC endpoint for trace export, e.g. `http://otel-collector:4317`.
    /// Opt-in: traces export only when this is set (unset = plain logs, no OTLP,
    /// no collector connection). Requires the `otel` build feature (on in release
    /// builds); a plain dev build ignores it with a warning.
    #[arg(long, env = "LUMEN_OTLP_ENDPOINT")]
    otlp_endpoint: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    lumen::tls::install_default_crypto_provider();
    let cli = Cli::parse();
    match cli.cmd {
        Command::Serve(args) => serve(args).await,
        Command::Spec(args) => {
            // `spec gen` writes a typed client; everything else prints to stdout.
            if let Some(SpecSub::Gen(gen)) = args.gen {
                return spec_gen(gen);
            }
            // Offline self-description: no engine, no server, no I/O beyond stdout.
            let out = if args.shapes {
                serde_json::to_string_pretty(&lumen::spec::query_shapes())?
            } else if args.fields {
                serde_json::to_string_pretty(&lumen::spec::field_catalog())?
            } else {
                match args.format {
                    SpecFormat::Openapi => lumen::spec::openapi_json(),
                    SpecFormat::OpenapiYaml => lumen::spec::openapi_yaml(),
                    SpecFormat::JsonSchema => lumen::spec::json_schema_json(),
                }
            };
            println!("{out}");
            Ok(())
        }
        Command::Llm(args) => {
            // Offline: no engine, no server, no I/O beyond stdout.
            let md = match args.topic {
                LlmTopic::Outline => lumen::spec::llm_outline_md(),
                LlmTopic::Workflow => lumen::spec::llm_workflow_md(),
                LlmTopic::Integration => lumen::spec::llm_integration_md(),
                LlmTopic::Quickstart => lumen::spec::llm_quickstart_md(),
                LlmTopic::Auth => lumen::spec::llm_auth_md(),
                LlmTopic::Recipes => lumen::spec::llm_recipes_md(),
            };
            let out = match args.format {
                LlmFormat::Md => md,
                LlmFormat::Json => match args.topic {
                    // Recipes are inherently structured → emit the canonical
                    // cookbook JSON (single source with `spec --shapes`).
                    LlmTopic::Recipes => {
                        serde_json::to_string_pretty(&lumen::spec::query_shapes())?
                    }
                    LlmTopic::Outline => serde_json::to_string_pretty(
                        &serde_json::json!({ "topic": "outline", "markdown": md }),
                    )?,
                    LlmTopic::Workflow => serde_json::to_string_pretty(
                        &serde_json::json!({ "topic": "workflow", "markdown": md }),
                    )?,
                    LlmTopic::Integration => serde_json::to_string_pretty(
                        &serde_json::json!({ "topic": "integration", "markdown": md }),
                    )?,
                    LlmTopic::Quickstart => serde_json::to_string_pretty(
                        &serde_json::json!({ "topic": "quickstart", "markdown": md }),
                    )?,
                    LlmTopic::Auth => serde_json::to_string_pretty(
                        &serde_json::json!({ "topic": "auth", "markdown": md }),
                    )?,
                },
            };
            println!("{out}");
            Ok(())
        }
        Command::Dockerfile(args) => dockerfile(args),
        Command::K8s(args) => k8s(args).await,
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

/// This binary's identity + build provenance for the standard CLI ops
/// (`upgrade` / `issue`), per the CONTRIBUTING.md CLI convention.
/// @spec projects/lumen/tech-design/interfaces/cli/lumen-upgrade-self-update-cli-from-github-releases.md
/// @spec projects/lumen/tech-design/interfaces/cli/lumen-issue-search-view-create-shared-cli-standard.md
const TOOL: cli_std::ToolInfo = cli_std::ToolInfo {
    project: "lumen",
    repo: "chrischeng-c4/axiom",
    target: env!("LUMEN_TARGET"),
    version: env!("CARGO_PKG_VERSION"),
    git_sha: env!("LUMEN_GIT_SHA"),
    built_at: env!("LUMEN_BUILT_AT"),
};

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
                if let Some(message) = message.as_deref() {
                    let head: String = message
                        .lines()
                        .next()
                        .unwrap_or("")
                        .chars()
                        .take(72)
                        .collect();
                    format!("lumen: {head}")
                } else {
                    "lumen: issue report".to_string()
                }
            });
            cli_std::issue::create(
                &TOOL,
                cli_std::issue::CreateOptions {
                    title,
                    message,
                    url: args.url,
                    repo: args.repo,
                    // Always tag with the project label so reports route
                    // automatically (CLI convention); keep any user labels too.
                    label: std::iter::once("project:lumen".to_string())
                        .chain(args.label)
                        .collect(),
                    dry_run: args.dry_run,
                    yes: args.yes,
                },
            )
            .await
        }
    }
}

/// `lumen spec gen` — generate a typed client from lumen's own OpenAPI document
/// (offline; no engine or server) and write it into `--out`.
/// @spec projects/lumen/tech-design/interfaces/cli/lumen-spec-gen-generate-a-typed-client-ts-py-rust-from-lumen-s-o.md
fn spec_gen(args: GenArgs) -> Result<()> {
    use cclab_openapi_codegen::{generate, GenOptions, HttpClient, Lang};
    let lang = match args.lang {
        GenLang::Ts => Lang::Ts,
        GenLang::Py => Lang::Py,
        GenLang::Rust => Lang::Rust,
    };
    let opts = GenOptions {
        lang,
        spec_path: PathBuf::new(),
        out_dir: args.out.clone(),
        client_name: "createClient".to_string(),
        http_client: match args.http {
            GenHttp::Fetch => HttpClient::Fetch,
            GenHttp::Axios => HttpClient::Axios,
        },
        emit_types: true,
        emit_client: true,
        // TanStack Query hooks are a TypeScript-only concern.
        emit_hooks: matches!(lang, Lang::Ts),
    };
    let output = generate(&lumen::spec::openapi_json(), &opts)?;
    std::fs::create_dir_all(&args.out)?;
    for file in &output.files {
        let path = args.out.join(&file.rel_path);
        std::fs::write(&path, &file.contents)?;
        println!("generated {}", path.display());
    }
    Ok(())
}

/// `lumen dockerfile` — render runtime image artifacts. The checked-in
/// Dockerfiles remain the repo fixtures; CLI output strips ownership markers so
/// the result is the Dockerfile users build.
fn dockerfile(args: DockerfileArgs) -> Result<()> {
    match args.cmd {
        DockerfileCmd::Render(args) => {
            let (file_name, body) = match args.variant {
                DockerfileVariant::Source => ("Dockerfile", render_source_dockerfile()),
                DockerfileVariant::Release => (
                    "Dockerfile.release",
                    render_release_dockerfile(args.version.as_deref()),
                ),
            };
            write_or_print(args.out.as_deref(), file_name, &body)
        }
    }
}

/// `lumen k8s` — cluster artifacts split by lifecycle layer. Only
/// `operator run` needs kube-rs at runtime; the render paths are offline and
/// work from the static manifests/CR templates embedded in the binary.
async fn k8s(args: K8sArgs) -> Result<()> {
    match args.cmd {
        K8sCmd::Crd(args) => match args.cmd {
            K8sCrdCmd::Render(args) => write_or_print(args.out.as_deref(), "crd.yaml", &crd_yaml()),
        },
        K8sCmd::Operator(args) => match args.cmd.unwrap_or(K8sOperatorCmd::Run) {
            K8sOperatorCmd::Run => run_operator().await,
            K8sOperatorCmd::Render(args) => {
                let yaml = render_operator_yaml(&args.namespace);
                write_or_print(args.out.as_deref(), "operator.yaml", &yaml)
            }
        },
        K8sCmd::Instance(args) => match args.cmd {
            K8sInstanceCmd::Render(args) => {
                let yaml = render_instance_yaml(&args);
                write_or_print(args.out.as_deref(), "lumen.yaml", &yaml)
            }
        },
    }
}

#[cfg(feature = "operator")]
async fn run_operator() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    lumen::operator::run().await
}

#[cfg(not(feature = "operator"))]
async fn run_operator() -> Result<()> {
    anyhow::bail!(
        "this lumen build was compiled without operator support; rebuild with \
         `--features operator` (the published image includes it)"
    )
}

#[cfg(feature = "operator")]
fn crd_yaml() -> String {
    lumen::operator::crd_yaml()
}

#[cfg(not(feature = "operator"))]
fn crd_yaml() -> String {
    ensure_trailing_newline(include_str!("../../k8s/operator/crd.yaml"))
}

fn render_source_dockerfile() -> String {
    strip_ownership_markers(include_str!("../../Dockerfile"))
}

fn render_release_dockerfile(version: Option<&str>) -> String {
    let tag = normalize_lumen_tag(version);
    let version = tag.trim_start_matches("lumen@");
    let template = strip_ownership_markers(include_str!("../../Dockerfile.release"));
    let mut out = String::new();
    for line in template.lines() {
        if line.starts_with("#   docker build -f projects/lumen/Dockerfile.release -t lumen:") {
            out.push_str(&format!(
                "#   docker build -f projects/lumen/Dockerfile.release -t lumen:{version} \\"
            ));
        } else if line.starts_with("#     --build-arg LUMEN_VERSION=") {
            out.push_str(&format!("#     --build-arg LUMEN_VERSION={tag} ."));
        } else if line.starts_with("ARG LUMEN_VERSION=") {
            out.push_str(&format!("ARG LUMEN_VERSION={tag}"));
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

fn normalize_lumen_tag(version: Option<&str>) -> String {
    let raw = version
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(env!("CARGO_PKG_VERSION"))
        .trim();
    if raw.starts_with("lumen@") {
        raw.to_string()
    } else {
        format!("lumen@{raw}")
    }
}

fn render_operator_yaml(namespace: &str) -> String {
    let mut out = String::new();
    out.push_str(&replace_operator_namespace(
        &strip_ownership_markers(include_str!("../../k8s/operator/rbac.yaml")),
        namespace,
    ));
    out.push_str("\n---\n");
    out.push_str(&replace_operator_namespace(
        &strip_ownership_markers(include_str!("../../k8s/operator/deployment.yaml")),
        namespace,
    ));
    ensure_trailing_newline(&out)
}

fn replace_operator_namespace(input: &str, namespace: &str) -> String {
    input
        .replace("name: lumen-system", &format!("name: {namespace}"))
        .replace(
            "namespace: lumen-system",
            &format!("namespace: {namespace}"),
        )
}

fn render_instance_yaml(args: &K8sInstanceRenderArgs) -> String {
    let default_version = env!("CARGO_PKG_VERSION");
    let (default_name, default_namespace, default_image, body) = match args.profile {
        K8sInstanceProfile::Dev => (
            "search",
            "default",
            "lumen:latest".to_string(),
            InstanceBody::Dev,
        ),
        K8sInstanceProfile::Staging => (
            "lumen",
            "staging",
            format!("lumen:{default_version}"),
            InstanceBody::Staging,
        ),
        K8sInstanceProfile::Prod => (
            "lumen",
            "production",
            format!("registry.example.com/lumen:{default_version}"),
            InstanceBody::Prod,
        ),
        K8sInstanceProfile::Template => (
            "REPLACE_ME__LUMEN_NAME",
            "REPLACE_ME__APP_NAMESPACE",
            "REPLACE_ME__REGISTRY/lumen:REPLACE_ME__IMAGE_TAG".to_string(),
            InstanceBody::Template,
        ),
    };
    let name = args.name.as_deref().unwrap_or(default_name);
    let namespace = args.namespace.as_deref().unwrap_or(default_namespace);
    let image = args.image.as_deref().unwrap_or(&default_image);

    let mut yaml = format!(
        "apiVersion: lumen.dev/v1alpha1\nkind: Lumen\nmetadata:\n  name: {name}\n  namespace: {namespace}\nspec:\n  image: {image}\n"
    );
    match body {
        InstanceBody::Dev => {
            yaml.push_str("  shardCount: 1\n  replicasPerShard: 1\n  voterCount: 1\n  logFormat: pretty\n  serving:\n    autoscaling:\n      minReplicas: 1\n      maxReplicas: 3\n      targetCpuUtilization: 70\n");
        }
        InstanceBody::Staging => {
            yaml.push_str("  shardCount: 3\n  replicasPerShard: 3\n  voterCount: 3\n  logFormat: json\n  serving:\n    autoscaling:\n      minReplicas: 3\n      maxReplicas: 6\n      targetCpuUtilization: 70\n  observability: true\n");
        }
        InstanceBody::Prod => {
            yaml.push_str("  imagePullPolicy: Always\n  shardCount: 6\n  replicasPerShard: 3\n  voterCount: 3\n  logFormat: json\n  logLevel: warn\n  auth: required\n  tokensSecret: lumen-tokens\n  serving:\n    autoscaling:\n      minReplicas: 6\n      maxReplicas: 12\n      targetCpuUtilization: 65\n    cpu: \"4\"\n    memory: 16Gi\n    graceSecs: 45\n  observability: true\n");
        }
        InstanceBody::Template => {
            yaml.push_str("  imagePullPolicy: IfNotPresent\n  shardCount: REPLACE_ME__SHARD_COUNT\n  replicasPerShard: REPLACE_ME__REPLICAS_PER_SHARD\n  voterCount: REPLACE_ME__VOTER_COUNT\n  logFormat: json\n  serving:\n    autoscaling:\n      minReplicas: 2\n      maxReplicas: 8\n      targetCpuUtilization: 70\n");
        }
    }
    ensure_trailing_newline(&yaml)
}

enum InstanceBody {
    Dev,
    Staging,
    Prod,
    Template,
}

fn strip_ownership_markers(input: &str) -> String {
    let mut out = String::new();
    for line in input.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("# SPEC-MANAGED:")
            || trimmed == "# CODEGEN-BEGIN"
            || trimmed == "# CODEGEN-END"
        {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn write_or_print(out: Option<&Path>, default_file: &str, body: &str) -> Result<()> {
    if let Some(path) = out {
        let target = if path.extension().is_some() {
            path.to_path_buf()
        } else {
            path.join(default_file)
        };
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&target, body)?;
        println!("wrote {}", target.display());
    } else {
        print!("{body}");
    }
    Ok(())
}

fn ensure_trailing_newline(input: &str) -> String {
    if input.ends_with('\n') {
        input.to_string()
    } else {
        format!("{input}\n")
    }
}

async fn serve(args: ServeArgs) -> Result<()> {
    init_tracing(
        &args.log_level,
        args.log_format,
        args.otlp_endpoint.as_deref(),
    );

    let engine = Arc::new(Engine::new());

    // OTLP metrics push (opt-in, same endpoint as traces): observable
    // instruments read the engine's atomic counters and push to the collector.
    #[cfg(feature = "otel")]
    if let Some(endpoint) = args.otlp_endpoint.as_deref() {
        match init_otel_meter(endpoint, engine.clone()) {
            Ok(()) => tracing::info!(otlp_endpoint = endpoint, "OTLP metrics push enabled"),
            Err(e) => {
                tracing::error!(error = %e, "OTLP metrics init failed; /metrics pull still works")
            }
        }
    }

    // Select the write log. `--wal raft` also yields a driver whose router is
    // merged into the serve app below (peer RPCs ride the h2c port).
    #[cfg(feature = "raft-wal")]
    let mut raft_host: Option<Arc<raft_host::RaftHost>> = None;
    #[cfg(feature = "raft-wal")]
    let mut raft_writer: Option<Arc<dyn lumen::coordinator::WriteSink>> = None;
    // k8s-native auto-detect: `--wal auto` (the default) picks raft when the
    // StatefulSet runs >1 replica per shard, else embedded — so single-node /
    // local dev needs no flags or cluster env.
    let backend = resolve_wal_backend(args.wal);
    let wal: Option<SharedWal> = match backend {
        WalBackend::Auto => unreachable!("auto is resolved by resolve_wal_backend"),
        WalBackend::Embedded => {
            tracing::info!("wal=embedded (in-process; single-node)");
            Some(Arc::new(MemWal::new()))
        }
        WalBackend::Nats => {
            tracing::info!(url = %args.nats_url, "wal=nats (JetStream)");
            Some(Arc::new(
                connect_nats_with_retry(&args.nats_url, args.nats_connect_timeout_secs)
                    .await
                    .context("connect NATS write log")?,
            ))
        }
        #[cfg(feature = "raft-wal")]
        WalBackend::Raft => {
            // Topology from the StatefulSet downward API via the shared helper
            // (node id + membership + peers — no hand-rolled ordinal/DNS math).
            // Raft RPCs ride the client port (the host's router merges into the
            // serve app), so the peer port is `args.port`; `LUMEN_PEERS` overrides
            // host:port to run a multi-node group on one machine.
            let headless = std::env::var("LUMEN_HEADLESS_SERVICE")
                .unwrap_or_else(|_| "lumen-headless".to_string());
            let topo =
                raft_host::ClusterTopology::from_env("lumen", &headless, args.port, "LUMEN_PEERS")
                    .context("raft: cluster topology from env")?;
            tracing::info!(
                node_id = topo.node_id,
                voters = ?topo.membership.voters,
                peers = ?topo.peers.keys().collect::<Vec<_>>(),
                data_dir = %args.raft_data_dir,
                "wal=raft (raft_core; multi-pod)"
            );
            let store = raft_host::RaftStore::open(
                &args.raft_data_dir,
                topo.node_id,
                raft_host::FsyncPolicy::Always,
            )
            .context("open raft store")?;
            // The host is the sole applier: committed entries fold straight into
            // the engine (via `EngineSm`), so there is no `WalLog`/coordinator
            // seam for the raft path. Cold-start (restore + replay) happens in
            // `RaftHost::spawn`; snapshot/compaction is driven externally below.
            let sm = lumen::raft_sm::EngineSm::new(engine.clone(), 0);
            let host = Arc::new(raft_host::RaftHost::spawn(
                topo.node_id,
                topo.membership,
                topo.peers,
                store,
                sm.clone() as Arc<dyn raft_host::RaftStateMachine>,
                raft_host::HostConfig {
                    snapshot: raft_host::SnapshotPolicy::External,
                    ..Default::default()
                },
            ));
            raft_host = Some(Arc::clone(&host));
            raft_writer = Some(Arc::new(lumen::raft_sm::RaftWriteSink::new(host, sm)));
            None
        }
    };

    // The raft path is the sole applier (no WalLog/coordinator seam): it
    // cold-starts inside `RaftHost::spawn` and uses the host as its `WriteSink`.
    #[cfg(feature = "raft-wal")]
    let is_raft = raft_writer.is_some();
    #[cfg(not(feature = "raft-wal"))]
    let is_raft = false;

    // Persistence bootstrap: load the latest checkpoint (if any) so we tail from
    // its sequence instead of replaying the whole log. Two modes share the
    // `--data-dir`: the default CBOR RDB and (opt-in) the columnar segment
    // checkpoint. `segment_mode` is `false` unless `--persistence=segment` is
    // passed, so the block below is byte-identical to today in the default mode.
    let segment_mode = use_segment_persistence(&args);

    // The CBOR RDB store — built unless segment persistence is selected.
    let rdb_store = if segment_mode {
        None
    } else {
        match &args.data_dir {
            Some(dir) => Some(Arc::new(
                LocalFsRdbStore::new(dir).context("open RDB store")?,
            )),
            None => None,
        }
    };

    // The segment-checkpoint store — built only in segment mode.
    let segment_store: Option<Arc<lumen::segment_rdb::SegmentRdbStore>> = if segment_mode {
        match &args.data_dir {
            Some(dir) => Some(Arc::new(
                lumen::segment_rdb::SegmentRdbStore::new(dir)
                    .context("open segment-checkpoint store")?,
            )),
            None => None,
        }
    } else {
        None
    };

    // Cold-start sequence: the WAL position the checkpoint is current as of, so
    // the apply loop tails from `start_seq + 1`.
    let mut start_seq = {
        if is_raft {
            // Raft cold-starts inside `RaftHost::spawn` (snapshot restore + replay
            // of committed entries); the engine here is fresh and the host owns
            // the applied seq, so there is nothing to load from `--data-dir`.
            0
        } else if let Some(store) = &segment_store {
            // Segment mode: reopen every collection from the newest checkpoint
            // INTO `engine` (no whole-collection load), replacing the CBOR restore.
            match store
                .reopen_into(&engine)
                .context("load latest segment checkpoint")?
            {
                Some(seq) => {
                    tracing::info!(up_to_seq = seq, "restored segment-checkpoint baseline");
                    seq
                }
                None => 0,
            }
        } else {
            cbor_cold_start(&rdb_store, &engine).await?
        }
    };

    // Local AOF (segment mode only): RDB (segment checkpoint, up to `start_seq`)
    // → AOF replay (`start_seq+1 .. A`) → broker tail (`A+1 ..`). After replay the
    // apply loop keeps appending to this same writer, and the checkpoint
    // snapshotter trims it. The default CBOR path never builds one.
    let aof_writer: Option<lumen::coordinator::SharedAof> = if segment_mode && !is_raft {
        match &args.data_dir {
            Some(dir) => {
                let aof_path = std::path::Path::new(dir).join("aof.log");
                // (b) Replay the AOF over the RDB baseline, advancing the cold-start
                // sequence to the AOF head `A` so the loop tails the broker from `A+1`.
                let replayed = lumen::aof::replay_aof_into(&engine, &aof_path, start_seq)
                    .context("replay AOF over segment baseline")?;
                if replayed > start_seq {
                    tracing::info!(from = start_seq, to = replayed, "replayed AOF tail");
                    start_seq = replayed;
                }
                // Open the same AOF for continued appends (truncates any torn tail).
                let w = lumen::aof::AofWriter::open(&aof_path).context("open AOF")?;
                Some(std::sync::Arc::new(std::sync::Mutex::new(w)))
            }
            None => None,
        }
    } else {
        None
    };

    // (c) Start the apply loop. In segment mode with an AOF, the loop appends
    // every applied record to it; otherwise the default loop runs unchanged.
    // The raft path uses the `RaftHost` as its `WriteSink`; every other backend
    // uses the `WriteCoordinator` (sole applier over a `WalLog`). Both are erased
    // to `Arc<dyn WriteSink>` so the API binds to a single write seam.
    #[cfg(feature = "raft-wal")]
    let raft_writer = raft_writer.take();
    #[cfg(not(feature = "raft-wal"))]
    let raft_writer: Option<Arc<dyn lumen::coordinator::WriteSink>> = None;
    let writer: Arc<dyn lumen::coordinator::WriteSink> = if let Some(rw) = raft_writer {
        rw
    } else {
        let wal = wal.expect("non-raft backend yields a WAL");
        match aof_writer.clone() {
            Some(aof) => WriteCoordinator::start_from_with_aof(wal, engine.clone(), start_seq, aof),
            None => WriteCoordinator::start_from(wal, engine.clone(), start_seq),
        }
    };

    let auth = Arc::new(AuthConfig::from_env()?);
    if auth.required {
        tracing::info!(tokens = auth.tokens.len(), "auth required");
    } else {
        tracing::warn!("auth=off — set LUMEN_AUTH=required for production");
    }

    let mut state = lumen::api::AppState::with_components(engine.clone(), auth, writer.clone());
    if !args.search_shard_segment_dirs.is_empty() {
        let shards = load_search_shard_segment_roots(&args.search_shard_segment_dirs)?;
        tracing::info!(shard_count = shards.len(), "search backend=segment-sharded");
        state = state.with_search_backend(Arc::new(lumen::routing::EngineShardSearch::new(shards)));
    }
    #[cfg_attr(not(feature = "raft-wal"), allow(unused_mut))]
    let mut app = lumen::api::router(state);
    // Peer raft RPCs (`/raft/*`, `/raftz`) share the h2c serve port.
    #[cfg(feature = "raft-wal")]
    if let Some(host) = &raft_host {
        app = app.merge(host.router());
    }

    // Periodic snapshotter. Raft mode: the host captures the engine RDB AND
    // compacts the raft log (bounding it + arming InstallSnapshot for a fresh
    // replica) — the shared backup layer (#524, closes #522 by construction).
    // Otherwise the RDB snapshotter writes the `--data-dir` checkpoints the apply
    // loop tails from on restart.
    #[cfg(feature = "raft-wal")]
    if let Some(host) = raft_host.clone() {
        let period = Duration::from_secs(args.snapshot_secs.max(1));
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(period);
            ticker.tick().await; // skip immediate fire
            loop {
                ticker.tick().await;
                match host.snapshot_and_compact().await {
                    Ok(idx) if idx > 0 => {
                        tracing::info!(snapshot_index = idx, "raft snapshot taken + log compacted")
                    }
                    Ok(_) => {}
                    Err(e) => tracing::warn!(error = %e, "raft snapshot/compact failed"),
                }
            }
        });
    }
    if let (false, Some(store)) = (is_raft, rdb_store) {
        let snap_engine = engine.clone();
        let snap_writer = writer.clone();
        let period = Duration::from_secs(args.snapshot_secs.max(1));
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(period);
            ticker.tick().await; // skip immediate fire
            loop {
                ticker.tick().await;
                let seq = snap_writer.applied_seq();
                match RdbSnapshot::capture(&snap_engine, seq) {
                    Ok(rdb) => {
                        if let Err(e) = store.save(&rdb).await {
                            tracing::warn!(error = %e, "RDB snapshot save failed");
                        } else {
                            tracing::info!(up_to_seq = seq, "RDB snapshot written");
                            let _ = store.prune(3).await;
                        }
                    }
                    Err(e) => tracing::warn!(error = %e, "RDB capture failed"),
                }
            }
        });
    }

    // Periodic segment-checkpoint snapshotter (segment mode only). Re-seals every
    // collection into a fresh generation, tagged with the applied seq, atomically
    // (stage + rename). The seal is CPU-bound (re-materializes columns) and takes
    // the per-collection state write lock, so it runs on a blocking thread to keep
    // the async runtime free — mirroring the apply loop's `spawn_blocking`.
    if let Some(store) = segment_store {
        let snap_engine = engine.clone();
        let snap_writer = writer.clone();
        let snap_aof = aof_writer.clone();
        let period = Duration::from_secs(args.snapshot_secs.max(1));
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(period);
            ticker.tick().await; // skip immediate fire
            loop {
                ticker.tick().await;
                let seq = snap_writer.applied_seq();
                let store2 = store.clone();
                let eng2 = snap_engine.clone();
                let res = tokio::task::spawn_blocking(move || {
                    store2
                        .save(&eng2, seq)
                        .map(|()| store2.prune(3).map(|_| ()))
                })
                .await;
                match res {
                    Ok(Ok(_)) => {
                        tracing::info!(up_to_seq = seq, "segment checkpoint written");
                        // The checkpoint at `seq` is now durable in the segment
                        // RDB, so every AOF frame with `seq <= C` is redundant —
                        // trim it (crash-safe rewrite-survivors + rename). Off the
                        // hot path: a blocking thread, since it rewrites the file.
                        if let Some(aof) = &snap_aof {
                            let aof2 = aof.clone();
                            let trim = tokio::task::spawn_blocking(move || {
                                aof2.lock()
                                    .map_err(|_| anyhow::anyhow!("aof writer poisoned"))?
                                    .truncate_through(seq)
                            })
                            .await;
                            match trim {
                                Ok(Ok(())) => {
                                    tracing::info!(through = seq, "AOF trimmed to checkpoint")
                                }
                                Ok(Err(e)) => tracing::warn!(error = %e, "AOF trim failed"),
                                Err(e) => tracing::warn!(error = %e, "AOF trim task panicked"),
                            }
                        }
                    }
                    Ok(Err(e)) => tracing::warn!(error = %e, "segment checkpoint save failed"),
                    Err(e) => tracing::warn!(error = %e, "segment checkpoint task panicked"),
                }
            }
        });
    }

    let bind = format!("{}:{}", args.host, args.port);
    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .with_context(|| format!("bind {bind}"))?;
    tracing::info!(addr = %bind, shard_count = args.shard_count, "lumen serve listening");

    let grace = Duration::from_secs(args.grace_secs);
    // Serve HTTP/1.1 + h2c on one port through the shared service HTTP shell,
    // with the standard SIGTERM drain sequence flipping `/readyz` to 503
    // before the listener closes.
    service_http::serve(
        listener,
        app,
        service_http::shutdown_with_drain(move || engine.start_drain(), grace),
    )
    .await;
    // Flush any batched spans before exit (no-op when OTLP was never enabled).
    #[cfg(feature = "otel")]
    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}

/// Whether segment persistence is selected. Driven purely by `--persistence`:
/// `false` for the default `cbor` mode (the binary's cold-start + snapshotter are
/// byte-identical to today), `true` only when `--persistence=segment` is passed.
fn use_segment_persistence(args: &ServeArgs) -> bool {
    args.persistence == Persistence::Segment
}

fn load_search_shard_segment_roots(dirs: &[PathBuf]) -> Result<Vec<Arc<Engine>>> {
    let mut shards = Vec::with_capacity(dirs.len());
    for dir in dirs {
        let store = lumen::segment_rdb::SegmentRdbStore::new(dir)
            .with_context(|| format!("open search shard segment root {}", dir.display()))?;
        let Some((engine, seq)) = store
            .load_latest()
            .with_context(|| format!("load search shard segment root {}", dir.display()))?
        else {
            anyhow::bail!(
                "search shard segment root {} has no committed gen-<seq> checkpoint",
                dir.display()
            );
        };
        tracing::info!(
            root = %dir.display(),
            up_to_seq = seq,
            "loaded search shard segment root"
        );
        shards.push(engine);
    }
    Ok(shards)
}

/// The CBOR-RDB cold start: load the latest `rdb-<seq>.lrb` (if any) into
/// `engine` and return its sequence so the apply loop tails from there. This is
/// the exact restore the binary has always done; factored out so the segment
/// branch can sit beside it without duplicating it.
async fn cbor_cold_start(
    rdb_store: &Option<Arc<LocalFsRdbStore>>,
    engine: &Arc<Engine>,
) -> Result<u64> {
    if let Some(store) = rdb_store {
        match store.load_latest().await? {
            Some(rdb) => {
                let seq = rdb.up_to_seq;
                rdb.restore_into(engine).context("restore RDB")?;
                tracing::info!(up_to_seq = seq, "restored RDB baseline");
                Ok(seq)
            }
            None => Ok(0),
        }
    } else {
        Ok(0)
    }
}

/// Connect to NATS, retrying the initial connect with exponential backoff
/// (capped at 5s/attempt) until `timeout_secs` elapses. Once connected,
/// `async-nats` auto-reconnects on its own, so only the initial connect needs
/// this — it stops a serving node from crash-looping when it starts before
/// the broker (e.g. mid-rollout). The last error is returned on timeout.
async fn connect_nats_with_retry(url: &str, timeout_secs: u64) -> Result<NatsWal> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(timeout_secs);
    let mut backoff = Duration::from_millis(250);
    let mut attempt = 0u32;
    loop {
        attempt += 1;
        match NatsWal::connect(url).await {
            Ok(wal) => {
                if attempt > 1 {
                    tracing::info!(attempt, "connected to NATS write log");
                }
                return Ok(wal);
            }
            Err(e) => {
                let now = tokio::time::Instant::now();
                if now >= deadline {
                    return Err(e).with_context(|| {
                        format!("NATS unreachable after {timeout_secs}s ({attempt} attempts)")
                    });
                }
                let sleep_for = backoff.min(deadline.saturating_duration_since(now));
                tracing::warn!(
                    attempt,
                    retry_in_ms = sleep_for.as_millis() as u64,
                    error = %e,
                    "NATS connect failed; retrying"
                );
                tokio::time::sleep(sleep_for).await;
                backoff = (backoff * 2).min(Duration::from_secs(5));
            }
        }
    }
}

fn init_tracing(level: &str, format: LogFormat, otlp_endpoint: Option<&str>) {
    use tracing_subscriber::prelude::*;
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("info,lumen={level}")));
    let fmt_layer = match format {
        LogFormat::Pretty => tracing_subscriber::fmt::layer().boxed(),
        LogFormat::Json => tracing_subscriber::fmt::layer().json().boxed(),
    };
    let registry = tracing_subscriber::registry().with(filter).with(fmt_layer);

    #[cfg(feature = "otel")]
    {
        if let Some(endpoint) = otlp_endpoint {
            match build_otel_tracer(endpoint) {
                Ok(tracer) => {
                    registry
                        .with(tracing_opentelemetry::layer().with_tracer(tracer))
                        .init();
                    tracing::info!(otlp_endpoint = endpoint, "OTLP trace export enabled");
                }
                Err(e) => {
                    registry.init();
                    tracing::error!(error = %e, "OTLP init failed; continuing without trace export");
                }
            }
        } else {
            registry.init();
        }
        return;
    }

    #[cfg(not(feature = "otel"))]
    {
        if otlp_endpoint.is_some() {
            registry.init();
            tracing::warn!(
                "LUMEN_OTLP_ENDPOINT is set but this binary was built without the `otel` \
                 feature — no trace export (rebuild with --features otel)"
            );
        } else {
            registry.init();
        }
    }
}

/// Build a batch OTLP (tonic/gRPC, plaintext) tracer exporting to `endpoint`.
/// Runs inside the tokio runtime (`serve` is `#[tokio::main]`-driven).
#[cfg(feature = "otel")]
fn build_otel_tracer(
    endpoint: &str,
) -> std::result::Result<opentelemetry_sdk::trace::Tracer, Box<dyn std::error::Error>> {
    use opentelemetry_otlp::WithExportConfig;
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(endpoint.to_string());
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(opentelemetry_sdk::trace::Config::default().with_resource(
            opentelemetry_sdk::Resource::new(vec![
                opentelemetry::KeyValue::new("service.name", "lumen"),
                opentelemetry::KeyValue::new(
                    "service.version",
                    env!("CARGO_PKG_VERSION").to_string(),
                ),
            ]),
        ))
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;
    Ok(tracer)
}

/// Build + install a global OTLP (tonic) meter provider that PUSHES lumen's
/// counters to `endpoint` every 60s. The observable instruments read the
/// engine's existing atomic counters, so the OTLP push and the `/metrics` pull
/// share one source of truth (no double counting). This is what lets a fleet of
/// stateless replicas report without anyone scraping each pod — the collector
/// aggregates and Prometheus scrapes only the collector.
#[cfg(feature = "otel")]
fn init_otel_meter(
    endpoint: &str,
    engine: Arc<Engine>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    use opentelemetry::metrics::MeterProvider as _;
    use opentelemetry_otlp::WithExportConfig;
    use std::sync::atomic::Ordering;

    let provider = opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry_sdk::runtime::Tokio)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint.to_string()),
        )
        .with_resource(opentelemetry_sdk::Resource::new(vec![
            opentelemetry::KeyValue::new("service.name", "lumen"),
            opentelemetry::KeyValue::new("service.version", env!("CARGO_PKG_VERSION").to_string()),
        ]))
        .with_period(Duration::from_secs(60))
        .build()?;

    let meter = provider.meter("lumen");

    // Each atomic counter → an observable instrument whose callback reads the
    // live value at every collection interval. Closures own an Arc<Engine>.
    macro_rules! obs_counter {
        ($name:literal, $field:ident, $desc:literal) => {{
            let eng = engine.clone();
            let _ = meter
                .u64_observable_counter($name)
                .with_description($desc)
                .with_callback(move |o| {
                    o.observe(eng.metrics().$field.load(Ordering::Relaxed), &[])
                })
                .init();
        }};
    }
    obs_counter!(
        "lumen_index_writes_total",
        index_writes_total,
        "Total index writes"
    );
    obs_counter!(
        "lumen_index_bytes_total",
        index_bytes_total,
        "Total bytes indexed"
    );
    obs_counter!(
        "lumen_search_requests_total",
        search_requests_total,
        "Total search requests"
    );
    obs_counter!(
        "lumen_search_latency_ms_sum",
        search_latency_ms_sum,
        "Search latency ms sum"
    );
    obs_counter!(
        "lumen_search_latency_ms_count",
        search_latency_ms_count,
        "Search latency count"
    );
    obs_counter!(
        "lumen_duplicates_requests_total",
        duplicates_requests_total,
        "Total duplicates requests"
    );
    obs_counter!(
        "lumen_collections_created_total",
        collections_created_total,
        "Total collections created"
    );
    obs_counter!(
        "lumen_schema_fields_total",
        schema_fields_total,
        "Total schema fields"
    );
    obs_counter!(
        "lumen_posting_cache_hits_total",
        posting_cache_hits_total,
        "Posting cache hits"
    );
    obs_counter!(
        "lumen_posting_cache_misses_total",
        posting_cache_misses_total,
        "Posting cache misses"
    );

    // storage_bytes is a gauge (can decrease).
    {
        let eng = engine.clone();
        let _ = meter
            .u64_observable_gauge("lumen_storage_bytes")
            .with_description("Current storage bytes")
            .with_callback(move |o| {
                o.observe(eng.metrics().storage_bytes.load(Ordering::Relaxed), &[])
            })
            .init();
    }

    opentelemetry::global::set_meter_provider(provider);
    Ok(())
}
// CODEGEN-END
