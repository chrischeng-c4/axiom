// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-bin-lumen-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! `lumen` — the single agent-first CLI: `serve` (serving node), `spec` /
//! `llm` (offline integration contract + agent topics), and `k8s` (operator
//! + CRD generation). Agents start here: `lumen llm --topic outline`.
//! @spec projects/lumen/tech-design/interfaces/cli/self-docs-teach-positional-lumen-llm-topic-but-the-cli-only-acce.md#logic
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
    /// Dump a running node's full SnapshotV1 JSON to stdout or `--out`.
    /// Alias of `export`; this is ad hoc data movement, not scheduled backup
    /// sink transport.
    Dump(SnapshotExportArgs),
    /// Export a running node's full SnapshotV1 JSON to stdout or `--out`.
    /// Use `backup` when you need destination sinks and retention.
    Export(SnapshotExportArgs),
    /// Load a SnapshotV1 JSON document from `--file` or stdin into a running
    /// node by replacing all engine state through `/admin/restore`.
    /// Alias of `import`.
    Load(SnapshotImportArgs),
    /// Import a SnapshotV1 JSON document from `--file` or stdin into a running
    /// node by replacing all engine state through `/admin/restore`.
    Import(SnapshotImportArgs),
    /// Self-update this binary from a published GitHub release. Resolves the
    /// running target + version, downloads the matching `lumen-<target>.tar.gz`,
    /// verifies its sha256, and atomically replaces the running executable.
    /// `--check` reports the available version without changing anything.
    // @spec projects/lumen/tech-design/interfaces/cli/lumen-upgrade-self-update-cli-from-github-releases.md
    Upgrade(UpgradeArgs),
    /// Search, view, and file Lumen issues on the axiom tracker.
    /// `search` and `view` read existing `app:lumen` issues; `create`
    /// files a diagnostics-rich issue tagged `app:lumen`.
    // @spec projects/lumen/tech-design/interfaces/cli/lumen-issue-search-view-create-shared-cli-standard.md
    Issue(IssueArgs),
    /// Fetch a snapshot from a running serving fleet's own `/admin/backup`
    /// and ship it to a destination (`file://`, `s3://`, or schema-only
    /// `gs://`) via `libs/service-backup`. No new snapshot mechanism — this only
    /// schedules and transports the existing admin API. Typically invoked by
    /// the operator's optional backup CronJob (`spec.serving.backup`, see
    /// `lumen llm --topic storage`), but works standalone. Requires the `backup`
    /// feature (pulled in transitively by `operator`).
    Backup(BackupArgs),
    /// Manage a `kubectl port-forward` for the duration of a wrapped command
    /// against a k8s-deployed Lumen instance — no manually tracked
    /// port-forward process (`lumen llm --topic recipes` has a worked
    /// example). Resolves a bearer token from the deployment's
    /// token-registry Secret when `--secret`/`--cr` is given (see `lumen llm
    /// --topic auth`).
    // @spec projects/lumen/tech-design/interfaces/cli/cli-connect-query-k8s-agent-workflow.md
    Connect(ConnectArgs),
    /// One-shot query wrappers against a reachable lumen node: `index`,
    /// `search`, `duplicates`, `collections list`. Assembles the exact wire
    /// body `lumen spec --shapes` publishes — no interactive REPL. Requires
    /// the `backup` feature (pulled in transitively by `operator`).
    // @spec projects/lumen/tech-design/interfaces/cli/cli-connect-query-k8s-agent-workflow.md
    Query(QueryArgs),
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
    /// One-shot: grow a running instance's `raft-<name>-<n>` PVCs to match
    /// its CR's `spec.serving.raftStorage` (#809). StatefulSet
    /// `volumeClaimTemplates` are immutable, so a CR edit alone never
    /// resizes existing PVCs; this patches them directly when the bound
    /// `StorageClass` allows expansion. Never shrinks (unsupported by
    /// Kubernetes) and never mutates the CR itself.
    ResizeStorage(K8sOperatorResizeStorageArgs),
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
struct K8sOperatorResizeStorageArgs {
    /// Namespace of the `Lumen` instance to resize.
    #[arg(long)]
    namespace: String,
    /// `Lumen` CR name.
    #[arg(long)]
    name: String,
    /// Report what would be patched without mutating any PVC.
    #[arg(long)]
    dry_run: bool,
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

/// `lumen issue <search|view|create|comment>` flags.
/// @spec projects/lumen/tech-design/interfaces/cli/lumen-issue-search-view-create-shared-cli-standard.md
/// @spec projects/lumen/tech-design/interfaces/cli/lumen-cli-add-issue-comment-auto-reopen-follow-up.md
#[derive(clap::Args)]
struct IssueArgs {
    #[command(subcommand)]
    command: IssueCommand,
}

#[derive(Subcommand)]
enum IssueCommand {
    /// Search Lumen issues (app:lumen); omit the query to list recent.
    Search(IssueSearchArgs),
    /// Print one issue by number.
    View(IssueViewArgs),
    /// File a diagnostics-rich Lumen issue.
    Create(IssueCreateArgs),
    /// Comment on an issue and ensure it is open.
    Comment(IssueCommentArgs),
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

#[derive(clap::Args)]
struct IssueCommentArgs {
    /// Issue number.
    number: u64,
    /// Follow-up note to add after reopening. Omit for cli-std's standard
    /// verification-failed message.
    #[arg(value_name = "MSG", num_args = 0..)]
    message: Vec<String>,
    /// Target repository (`owner/name`); defaults to lumen's release repo.
    #[arg(long)]
    repo: Option<String>,
    /// Print the reopen/comment request without changing GitHub state.
    #[arg(long)]
    dry_run: bool,
    /// Skip the confirmation prompt.
    #[arg(short = 'y', long)]
    yes: bool,
}

/// `lumen backup` flags (#808): pulls a snapshot over HTTP from a running
/// serving fleet and ships it to a destination via `libs/service-backup`.
#[derive(clap::Args)]
struct BackupArgs {
    /// Base URL of a running lumen serving node, e.g.
    /// `http://<name>.<namespace>.svc.cluster.local:7373` (what the operator's
    /// backup CronJob passes) or `http://localhost:7373` for ad hoc use.
    #[arg(long)]
    url: String,
    /// Destination URI: `file:///path`, `s3://bucket/prefix`, or
    /// schema-only `gs://bucket/prefix` (parsed, but the runner supports
    /// `file://` and `s3://` sinks today).
    #[arg(long)]
    dest: String,
    /// Bearer token for the admin API (needs `Role::Admin` on `*`). Falls
    /// back to `LUMEN_BACKUP_TOKEN`; omit entirely when `spec.auth: off`.
    #[arg(long, env = "LUMEN_BACKUP_TOKEN")]
    token: Option<String>,
    /// Drop backup objects older than this many seconds after a successful
    /// put. Omit to keep everything.
    #[arg(long)]
    retention_secs: Option<u64>,
}

/// `lumen connect` flags (#1321): manage a `kubectl port-forward` around a
/// wrapped command so an agent never tracks the port-forward process itself.
/// @spec projects/lumen/tech-design/interfaces/cli/cli-connect-query-k8s-agent-workflow.md
#[derive(clap::Args)]
struct ConnectArgs {
    /// kubectl context to port-forward through. Omit to use the current context.
    #[arg(long)]
    context: Option<String>,
    /// Namespace of the target Service (or `Lumen` CR when `--cr` is set).
    #[arg(long)]
    namespace: String,
    /// Target Service name. Defaults to the `--cr` name when `--cr` is set
    /// (the client Service shares the CR's own metadata name).
    #[arg(long)]
    service: Option<String>,
    /// `Lumen` CR name. When set (and `--service` is omitted) the Service
    /// name defaults to this CR's own name, and its `spec.tokensSecret` (if
    /// any) auto-resolves `--secret`.
    #[arg(long)]
    cr: Option<String>,
    /// Local port to forward to. Omit to pick a free ephemeral port.
    #[arg(long)]
    local_port: Option<u16>,
    /// Remote (Service) port.
    #[arg(long, default_value_t = 7373)]
    remote_port: u16,
    /// Secret name holding a `token-registry.json` key (see `lumen llm
    /// --topic auth`). Auto-resolved from `--cr`'s `spec.tokensSecret` when
    /// omitted and `--cr` is set.
    #[arg(long)]
    secret: Option<String>,
    /// Minimum role the resolved token must cover.
    #[arg(long, value_enum, default_value_t = TokenRole::Admin)]
    role: TokenRole,
    /// Collection id the resolved token must be authorized against. Omit for
    /// the wildcard `*` grant.
    #[arg(long)]
    collection: Option<String>,
    /// The command to run with `LUMEN_URL` (and `LUMEN_TOKEN`, when
    /// resolved) set to the local end of the port-forward. Everything after
    /// `--`, e.g. `lumen connect --namespace prod --cr search -- lumen query
    /// collections list`.
    #[arg(last = true, required = true)]
    command: Vec<String>,
}

/// Bearer-token role required for `lumen connect`/`lumen query`'s Secret
/// token resolution (R2). Mirrors `service_auth::Role`; lumen's own role
/// mapping into `cli_std::connect::Role` (#1376) — every k8s-native service
/// CLI adopting `cli_std::connect` supplies its own such mapping.
#[derive(Clone, Copy, ValueEnum)]
enum TokenRole {
    Read,
    Write,
    Admin,
}

/// @spec projects/lumen/tech-design/interfaces/cli/cli-connect-query-k8s-agent-workflow.md
impl From<TokenRole> for cli_std::connect::Role {
    fn from(role: TokenRole) -> Self {
        match role {
            TokenRole::Read => cli_std::connect::Role::Read,
            TokenRole::Write => cli_std::connect::Role::Write,
            TokenRole::Admin => cli_std::connect::Role::Admin,
        }
    }
}

/// Shared k8s-aware token/target resolution for `lumen connect` and `lumen
/// query *` (R2): an explicit `--token`/`LUMEN_TOKEN` wins; otherwise, when
/// `--namespace`/`--secret` are set, resolve one bearer token from the
/// deployment's token-registry Secret (see `lumen llm --topic auth`) whose
/// role covers `--role` for the query's collection (or the wildcard `*`).
/// @spec projects/lumen/tech-design/interfaces/cli/cli-connect-query-k8s-agent-workflow.md
#[derive(clap::Args, Clone)]
struct QueryTarget {
    /// Base URL of a reachable lumen serving node, e.g. `http://localhost:7373`
    /// — what `lumen connect` sets for the wrapped command.
    #[arg(long, env = "LUMEN_URL")]
    url: Option<String>,
    /// Explicit bearer token. Falls back to `LUMEN_TOKEN`, then to the
    /// token-registry Secret named by `--namespace`/`--secret`.
    #[arg(long, env = "LUMEN_TOKEN")]
    token: Option<String>,
    /// kubectl context for Secret resolution.
    #[arg(long)]
    context: Option<String>,
    /// Namespace holding the token-registry Secret.
    #[arg(long)]
    namespace: Option<String>,
    /// Secret name holding a `token-registry.json` key.
    #[arg(long)]
    secret: Option<String>,
    /// Minimum role the resolved token must cover.
    #[arg(long, value_enum, default_value_t = TokenRole::Admin)]
    role: TokenRole,
}

/// `lumen query <index|search|duplicates|collections>` flags (#1321): thin
/// one-shot wrappers assembling the exact `lumen spec --shapes` wire body.
/// @spec projects/lumen/tech-design/interfaces/cli/cli-connect-query-k8s-agent-workflow.md
#[derive(clap::Args)]
struct QueryArgs {
    #[command(subcommand)]
    command: QueryCommand,
}

#[derive(Subcommand)]
enum QueryCommand {
    /// `POST /collections/{id}/index` — index one or more field values. Wire
    /// body is FLAT: `{"items":[{"external_id","field","value"}]}` — NOT a
    /// nested `{id, fields:{...}}` shape (see `lumen spec --shapes` → "index").
    Index(QueryIndexArgs),
    /// `POST /collections/{id}/search` — term/match/raw-JSON one-shot search.
    Search(QuerySearchArgs),
    /// `POST /collections/{id}/duplicates` — find external_ids sharing a value.
    Duplicates(QueryDuplicatesArgs),
    /// Collection-level read helpers.
    Collections(QueryCollectionsArgs),
}

#[derive(clap::Args)]
struct QueryIndexArgs {
    #[command(flatten)]
    target: QueryTarget,
    /// Target collection id.
    #[arg(long)]
    collection: String,
    /// One item as `EXTERNAL_ID:FIELD=VALUE` (repeatable). `VALUE` is parsed
    /// as JSON when possible (numbers, `[..]` vectors/string-lists), else
    /// kept as a plain string — so `p1:price=79` and
    /// `p1:embedding=[0.1,0.2,0.9]` both work unquoted.
    #[arg(long = "item", value_name = "EXTERNAL_ID:FIELD=VALUE", required = true)]
    items: Vec<String>,
}

#[derive(clap::Args)]
struct QuerySearchArgs {
    #[command(flatten)]
    target: QueryTarget,
    /// Target collection id.
    #[arg(long)]
    collection: String,
    /// Exact term match: `FIELD=VALUE`. Exactly one of `--term`/`--match`/
    /// `--query-json` is required.
    #[arg(long, value_name = "FIELD=VALUE")]
    term: Option<String>,
    /// BM25 text match: `FIELD=TEXT`. Exactly one of `--term`/`--match`/
    /// `--query-json` is required.
    #[arg(long = "match", value_name = "FIELD=TEXT")]
    match_: Option<String>,
    /// Raw `QueryNode` JSON — escape hatch for shapes `--term`/`--match`
    /// don't cover (`lumen spec --shapes` has the full cookbook). Exactly one
    /// of `--term`/`--match`/`--query-json` is required.
    #[arg(long)]
    query_json: Option<String>,
    #[arg(long, default_value_t = 20)]
    limit: u32,
}

#[derive(clap::Args)]
struct QueryDuplicatesArgs {
    #[command(flatten)]
    target: QueryTarget,
    /// Target collection id.
    #[arg(long)]
    collection: String,
    /// Field to find shared values on.
    #[arg(long)]
    field: String,
    #[arg(long, default_value_t = 2)]
    min_group_size: u32,
    #[arg(long, default_value_t = 100)]
    limit: u32,
    #[arg(long, default_value_t = 0)]
    offset: u32,
}

#[derive(clap::Args)]
struct QueryCollectionsArgs {
    #[command(subcommand)]
    command: QueryCollectionsCommand,
}

#[derive(Subcommand)]
enum QueryCollectionsCommand {
    /// `GET /collections` — list collection ids visible to the resolved token.
    List(QueryCollectionsListArgs),
}

#[derive(clap::Args)]
struct QueryCollectionsListArgs {
    #[command(flatten)]
    target: QueryTarget,
}

/// `lumen dump|export` flags (#1095): pulls SnapshotV1 JSON from a running
/// serving fleet and writes the exact bytes to stdout or a local file.
#[derive(clap::Args)]
struct SnapshotExportArgs {
    /// Base URL of a running lumen serving node, e.g. `http://localhost:7373`.
    #[arg(long)]
    url: String,
    /// Write the SnapshotV1 JSON bytes to this path instead of stdout.
    #[arg(long)]
    out: Option<PathBuf>,
    /// Bearer token for the admin API (needs `Role::Admin` on `*`). Falls
    /// back to `LUMEN_BACKUP_TOKEN`; omit entirely when `spec.auth: off`.
    #[arg(long, env = "LUMEN_BACKUP_TOKEN")]
    token: Option<String>,
}

/// `lumen load|import` flags (#1095): reads SnapshotV1 JSON and posts it to
/// `/admin/restore`, replacing the target engine state.
#[derive(clap::Args)]
struct SnapshotImportArgs {
    /// Base URL of a running lumen serving node, e.g. `http://localhost:7373`.
    #[arg(long)]
    url: String,
    /// Read SnapshotV1 JSON bytes from this path. Omit to read stdin.
    #[arg(long)]
    file: Option<PathBuf>,
    /// Bearer token for the admin API (needs `Role::Admin` on `*`). Falls
    /// back to `LUMEN_BACKUP_TOKEN`; omit entirely when `spec.auth: off`.
    #[arg(long, env = "LUMEN_BACKUP_TOKEN")]
    token: Option<String>,
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
    /// Kubernetes-native deployment topology, shard/replica knobs, and bootstrap.
    Deployment,
    /// Operator storage/ops contract: StatefulSet + durable PVC-backed WAL,
    /// including at `replicasPerShard: 1`.
    Storage,
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
    /// Physical storage shard count. Data ownership uses the versioned
    /// virtual-bucket map, not permanent `hash % shardCount` routing.
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
    /// Optional SnapshotV1 JSON seed URI for empty-PVC bootstrap. Supports
    /// exact `file://` paths and, in backup-enabled builds, exact
    /// `s3://bucket/key` objects.
    #[arg(long, env = "LUMEN_BOOTSTRAP_SEED_URI")]
    bootstrap_seed_uri: Option<String>,
    /// Optional seed fetch throttle advertised in CR/env. Exact object fetch is
    /// a one-shot read; streaming throttle belongs in the source adapter.
    #[arg(long, env = "LUMEN_BOOTSTRAP_MAX_BYTES_PER_SEC")]
    bootstrap_max_bytes_per_sec: Option<u64>,
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
                LlmTopic::Deployment => lumen::spec::llm_deployment_md(),
                LlmTopic::Storage => lumen::spec::llm_storage_md(),
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
                    LlmTopic::Deployment => serde_json::to_string_pretty(
                        &serde_json::json!({ "topic": "deployment", "markdown": md }),
                    )?,
                    LlmTopic::Storage => serde_json::to_string_pretty(
                        &serde_json::json!({ "topic": "storage", "markdown": md }),
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
        Command::Backup(args) => dispatch_backup(args).await,
        Command::Dump(args) | Command::Export(args) => dispatch_snapshot_export(args).await,
        Command::Load(args) | Command::Import(args) => dispatch_snapshot_import(args).await,
        Command::Connect(args) => connect(args).await,
        Command::Query(args) => dispatch_query(args).await,
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
                    label: std::iter::once("app:lumen".to_string())
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
    // Chainable output (#963): point at the generated client's entrypoint
    // module — the one file every language always emits (unconditionally
    // pushed by each emitter regardless of `--emit-*` selection).
    let entry_file = match lang {
        Lang::Ts => "index.ts",
        Lang::Py => "__init__.py",
        Lang::Rust => "mod.rs",
    };
    println!("next: {}", args.out.join(entry_file).display());
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
            let variant = args.variant;
            let version = args.version.clone();
            write_or_print(args.out.as_deref(), file_name, &body, move |target| {
                dockerfile_next_command(variant, version.as_deref(), target)
            })
        }
    }
}

/// `next:` builder for `dockerfile render --out` (#963): the matching
/// `docker build` invocation for the variant that was just written.
fn dockerfile_next_command(
    variant: DockerfileVariant,
    version: Option<&str>,
    target: &Path,
) -> String {
    match variant {
        DockerfileVariant::Source => format!("docker build -f {} -t lumen:dev .", target.display()),
        DockerfileVariant::Release => {
            let tag = normalize_lumen_tag(version);
            let ver = tag.trim_start_matches("lumen@");
            format!(
                "docker build -f {} -t lumen:{ver} --build-arg LUMEN_VERSION={tag} .",
                target.display()
            )
        }
    }
}

/// `lumen k8s` — cluster artifacts split by lifecycle layer. `operator run`
/// and `operator resize-storage` need kube-rs at runtime; the render paths
/// are offline and work from the static manifests/CR templates embedded in
/// the binary.
async fn k8s(args: K8sArgs) -> Result<()> {
    match args.cmd {
        K8sCmd::Crd(args) => match args.cmd {
            K8sCrdCmd::Render(args) => write_or_print(
                args.out.as_deref(),
                "crd.yaml",
                &crd_yaml(),
                kubectl_apply_next,
            ),
        },
        K8sCmd::Operator(args) => match args.cmd.unwrap_or(K8sOperatorCmd::Run) {
            K8sOperatorCmd::Run => run_operator().await,
            K8sOperatorCmd::Render(args) => {
                let yaml = render_operator_yaml(&args.namespace);
                write_or_print(
                    args.out.as_deref(),
                    "operator.yaml",
                    &yaml,
                    kubectl_apply_next,
                )
            }
            K8sOperatorCmd::ResizeStorage(args) => resize_storage(args).await,
        },
        K8sCmd::Instance(args) => match args.cmd {
            K8sInstanceCmd::Render(args) => {
                let yaml = render_instance_yaml(&args);
                write_or_print(args.out.as_deref(), "lumen.yaml", &yaml, kubectl_apply_next)
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

/// `lumen k8s operator resize-storage` (#809): one-shot detect-and-patch for
/// the `raft` PVC's `volumeClaimTemplates` immutability gap — see
/// `lumen::operator::resize::resize_instance`.
#[cfg(feature = "operator")]
async fn resize_storage(args: K8sOperatorResizeStorageArgs) -> Result<()> {
    let client = kube::Client::try_default()
        .await
        .context("build a kube client from the in-cluster/kubeconfig context")?;
    let outcomes =
        lumen::operator::resize::resize_instance(client, &args.namespace, &args.name, args.dry_run)
            .await?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "outcomes": outcomes,
            "next": "done",
        }))?
    );
    Ok(())
}

#[cfg(not(feature = "operator"))]
async fn resize_storage(_args: K8sOperatorResizeStorageArgs) -> Result<()> {
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

/// `lumen backup` (#808): fetch `{url}/admin/backup` and ship the bytes to
/// `dest` via `libs/service-backup`, printing the resulting
/// `BackupRunResult` as JSON. This is what the operator's optional backup
/// CronJob (`spec.serving.backup`) invokes on a schedule; it works equally
/// well ad hoc against any running serving node.
#[cfg(feature = "backup")]
async fn dispatch_backup(args: BackupArgs) -> Result<()> {
    let dest = service_backup::BackupDestination::from_uri(&args.dest)?;
    let retention = match args.retention_secs {
        Some(secs) => service_backup::RetentionPolicy::max_age_seconds(secs),
        None => service_backup::RetentionPolicy::default(),
    };
    let result =
        lumen::backup::run_backup(&args.url, args.token.as_deref(), &dest, &retention).await?;
    // Chainable output (#963): `lumen backup` always emits a single JSON
    // object, so the contract's "next" is a top-level field, not a text tail
    // line. `service_backup::BackupRunResult` stays untouched (shared type) —
    // this widens only the ad hoc `Value` this CLI prints.
    let mut out = serde_json::to_value(&result)?;
    if let serde_json::Value::Object(ref mut map) = out {
        map.insert(
            "next".to_string(),
            serde_json::Value::String(restore_next_command(&args, &result)),
        );
    }
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}

/// The matching restore step for a `lumen backup` run (#963): POST the
/// just-written snapshot bytes back to `/admin/restore` on the same fleet.
/// Only `file://` destinations resolve to a concrete local path for a copyable
/// restore command; cloud sinks remain shared `service-backup` behavior and
/// fall back to a generic note here instead of guessing a wrong object-fetch
/// command. The token, when set, is never echoed — the command references the
/// same env var the flag reads.
#[cfg(feature = "backup")]
fn restore_next_command(args: &BackupArgs, result: &service_backup::BackupRunResult) -> String {
    let url = args.url.trim_end_matches('/');
    let auth = if args.token.is_some() {
        " -H \"Authorization: Bearer $LUMEN_BACKUP_TOKEN\""
    } else {
        ""
    };
    match result.object.sink.strip_prefix("local:") {
        Some(root) => format!(
            "curl -sS -X POST {url}/admin/restore{auth} -H 'Content-Type: application/json' --data-binary @{}/{}",
            root.trim_end_matches('/'),
            result.object.key
        ),
        None => format!(
            "fetch {} from {} then: curl -sS -X POST {url}/admin/restore{auth} -H 'Content-Type: application/json' --data-binary @<downloaded-file>",
            result.object.key, result.object.sink
        ),
    }
}

#[cfg(not(feature = "backup"))]
async fn dispatch_backup(_args: BackupArgs) -> Result<()> {
    anyhow::bail!(
        "this lumen build was compiled without backup support; rebuild with \
         `--features backup` (or `operator`, which pulls it in — the published \
         image includes both)"
    )
}

/// `lumen dump|export` (#1095): fetch `{url}/admin/backup` and write exact
/// SnapshotV1 JSON bytes to stdout or `--out`.
#[cfg(feature = "backup")]
async fn dispatch_snapshot_export(args: SnapshotExportArgs) -> Result<()> {
    let payload = lumen::backup::fetch_snapshot_bytes(&args.url, args.token.as_deref()).await?;
    if let Some(out) = args.out {
        if let Some(parent) = out.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        std::fs::write(&out, &payload).with_context(|| format!("write {}", out.display()))?;
        let next =
            restore_file_next_command(args.url.trim_end_matches('/'), &out, args.token.is_some());
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "status": "exported",
                "path": out,
                "bytes": payload.len(),
                "next": next,
            }))?
        );
    } else {
        let mut stdout = std::io::stdout().lock();
        std::io::Write::write_all(&mut stdout, &payload)?;
        std::io::Write::flush(&mut stdout)?;
    }
    Ok(())
}

#[cfg(not(feature = "backup"))]
async fn dispatch_snapshot_export(_args: SnapshotExportArgs) -> Result<()> {
    anyhow::bail!(
        "this lumen build was compiled without backup support; rebuild with \
         `--features backup` (or `operator`, which pulls it in — the published \
         image includes both)"
    )
}

/// `lumen load|import` (#1095): read SnapshotV1 JSON bytes from `--file` or
/// stdin and POST them to `{url}/admin/restore`.
#[cfg(feature = "backup")]
async fn dispatch_snapshot_import(args: SnapshotImportArgs) -> Result<()> {
    let payload = match &args.file {
        Some(path) => std::fs::read(path).with_context(|| format!("read {}", path.display()))?,
        None => {
            let mut buf = Vec::new();
            let mut stdin = std::io::stdin();
            std::io::Read::read_to_end(&mut stdin, &mut buf)?;
            buf
        }
    };
    lumen::backup::restore_snapshot_bytes(&args.url, args.token.as_deref(), &payload).await?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "status": "restored",
            "url": args.url.trim_end_matches('/'),
            "bytes": payload.len(),
            "next": "done",
        }))?
    );
    Ok(())
}

#[cfg(not(feature = "backup"))]
async fn dispatch_snapshot_import(_args: SnapshotImportArgs) -> Result<()> {
    anyhow::bail!(
        "this lumen build was compiled without backup support; rebuild with \
         `--features backup` (or `operator`, which pulls it in — the published \
         image includes both)"
    )
}

#[cfg(feature = "backup")]
fn restore_file_next_command(url: &str, path: &Path, has_token: bool) -> String {
    let auth = if has_token {
        " --token $LUMEN_BACKUP_TOKEN"
    } else {
        ""
    };
    format!("lumen import --url {url}{auth} --file {}", path.display())
}

// ---------------------------------------------------------------------------
// `lumen connect` / `lumen query` (#1321) — thin adapter over
// `cli_std::connect` (#1376): the `kubectl port-forward` process lifecycle
// (`ChildGuard`, `free_local_port`, `wait_for_local_port_ready`) and the
// token-registry Secret resolution chain (`kubectl_get_json`,
// `resolve_cr_tokens_secret`, `resolve_token`) now live in
// `libs/cli-std/src/connect.rs`, reusable by any k8s-native service CLI.
// This file keeps only its own flag surface (`ConnectArgs`/`QueryTarget`),
// the `Lumen` CRD-name lookup convention (`"lumen"` passed as
// `resource_kind`), and the `TokenRole` -> `cli_std::connect::Role` mapping.
// ---------------------------------------------------------------------------

/// R2: resolve a usable bearer token without the caller decoding the
/// Secret/JSON by hand. Precedence: `target.token` (explicit flag or
/// `LUMEN_TOKEN`) wins; otherwise, when `--namespace`/`--secret` are both
/// set, fetch the Secret via kubectl, decode its `token-registry.json` key
/// (the same schema `lumen llm --topic auth` documents), and pick a token
/// whose role covers `target.role` for `collection` (or `*`). Returns `None`
/// when no token can be resolved (e.g. `spec.auth: off` deployments). Thin
/// wrapper over `cli_std::connect::resolve_token` (#1376).
/// @spec projects/lumen/tech-design/interfaces/cli/cli-connect-query-k8s-agent-workflow.md
fn resolve_token(target: &QueryTarget, collection: Option<&str>) -> Result<Option<String>> {
    cli_std::connect::resolve_token(
        target.token.as_deref(),
        target.context.as_deref(),
        target.namespace.as_deref(),
        target.secret.as_deref(),
        target.role.into(),
        collection,
    )
}

// The body-builder / URL-resolution helpers below are exercised directly by
// `dispatch_query`'s `backup`-gated real implementation and by this file's
// unit tests; `cfg(any(test, feature = "backup"))` keeps them from tripping
// dead-code warnings in a plain default (non-`backup`) build while staying
// available to `cargo test -p lumen` without requiring `--features backup`.
#[cfg(any(test, feature = "backup"))]
fn resolve_base_url(target: &QueryTarget) -> Result<String> {
    target
        .url
        .clone()
        .context("--url is required (or set LUMEN_URL, or run inside `lumen connect ... -- ...`)")
}

/// `lumen connect` (#1321, R1): spawn `kubectl port-forward`, wait until the
/// local end is reachable, run the wrapped command with `LUMEN_URL` (and
/// `LUMEN_TOKEN`, when resolved) set, then tear the port-forward down
/// (`ChildGuard::drop`) once the wrapped command exits — regardless of its
/// exit status — so no port-forward process is left for the caller to track.
/// @spec projects/lumen/tech-design/interfaces/cli/cli-connect-query-k8s-agent-workflow.md
async fn connect(args: ConnectArgs) -> Result<()> {
    let service = args
        .service
        .clone()
        .or_else(|| args.cr.clone())
        .context("--service or --cr is required")?;

    let secret = match args.secret.clone() {
        Some(secret) => Some(secret),
        None => match &args.cr {
            // "lumen" is the `Lumen` CRD's kubectl resource name — lumen's
            // own CR-kind lookup convention (R2).
            Some(cr) => cli_std::connect::resolve_cr_tokens_secret(
                args.context.as_deref(),
                &args.namespace,
                "lumen",
                cr,
            )?,
            None => None,
        },
    };

    let local_port = match args.local_port {
        Some(port) => port,
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
    let _forward =
        cli_std::connect::ChildGuard::spawn(&mut pf_cmd).context("start kubectl port-forward")?;

    cli_std::connect::wait_for_local_port_ready(local_port, Duration::from_secs(30))?;

    let target = QueryTarget {
        url: None,
        token: None,
        context: args.context.clone(),
        namespace: Some(args.namespace.clone()),
        secret,
        role: args.role,
    };
    let token = resolve_token(&target, args.collection.as_deref())?;

    let base_url = format!("http://127.0.0.1:{local_port}");
    let (program, rest) = args
        .command
        .split_first()
        .context("wrapped command is empty")?;
    let mut child_cmd = std::process::Command::new(program);
    child_cmd.args(rest);
    child_cmd.env("LUMEN_URL", &base_url);
    if let Some(token) = &token {
        child_cmd.env("LUMEN_TOKEN", token);
    }
    let status = child_cmd.status().context("run wrapped command")?;
    // `_forward` drops here (end of scope), tearing the port-forward down
    // whether the wrapped command succeeded or not (AC1).
    if !status.success() {
        anyhow::bail!("wrapped command exited with {status}");
    }
    Ok(())
}

/// Parse a CLI-supplied value into a `FieldValue`: JSON first (so
/// `--item p1:price=79` and `--item p1:embedding=[0.1,0.2,0.9]` work
/// unquoted), else the raw string.
#[cfg(any(test, feature = "backup"))]
fn parse_field_value(raw: &str) -> lumen::types::FieldValue {
    serde_json::from_str::<lumen::types::FieldValue>(raw)
        .unwrap_or_else(|_| lumen::types::FieldValue::String(raw.to_string()))
}

/// Parse one `--item EXTERNAL_ID:FIELD=VALUE` flag into an `IndexItem`.
#[cfg(any(test, feature = "backup"))]
fn parse_index_item(spec: &str) -> Result<lumen::types::IndexItem> {
    let (external_id, rest) = spec
        .split_once(':')
        .with_context(|| format!("--item `{spec}` must be EXTERNAL_ID:FIELD=VALUE"))?;
    let (field, value) = rest
        .split_once('=')
        .with_context(|| format!("--item `{spec}` must be EXTERNAL_ID:FIELD=VALUE"))?;
    Ok(lumen::types::IndexItem {
        external_id: external_id.to_string(),
        field: field.to_string(),
        value: parse_field_value(value),
        version: None,
    })
}

/// AC3: build the exact flat `POST /collections/{id}/index` body —
/// `{"items":[{"external_id","field","value"}]}` — matching the shape
/// `lumen::spec::query_shapes()`'s "index" entry publishes, NOT a nested
/// `{id, fields:{...}}` shape.
#[cfg(any(test, feature = "backup"))]
fn build_index_body(collection: &str, items: &[String]) -> Result<(String, serde_json::Value)> {
    let parsed: Result<Vec<_>> = items.iter().map(|s| parse_index_item(s)).collect();
    let request = lumen::types::IndexRequest {
        items: parsed?,
        request_id: None,
    };
    let body = serde_json::to_value(&request).context("serialize IndexRequest")?;
    Ok((format!("/collections/{collection}/index"), body))
}

/// Build the `QueryNode` for `lumen query search` from exactly one of
/// `--term`/`--match`/`--query-json`.
#[cfg(any(test, feature = "backup"))]
fn build_search_query_node(args: &QuerySearchArgs) -> Result<lumen::types::QueryNode> {
    let set_count = [
        args.term.is_some(),
        args.match_.is_some(),
        args.query_json.is_some(),
    ]
    .into_iter()
    .filter(|set| *set)
    .count();
    if set_count != 1 {
        anyhow::bail!("exactly one of --term, --match, --query-json is required");
    }
    if let Some(term) = &args.term {
        let (field, value) = term.split_once('=').context("--term must be FIELD=VALUE")?;
        return Ok(lumen::types::QueryNode::Term(lumen::types::TermQuery {
            field: field.to_string(),
            value: parse_field_value(value),
        }));
    }
    if let Some(m) = &args.match_ {
        let (field, text) = m.split_once('=').context("--match must be FIELD=TEXT")?;
        return Ok(lumen::types::QueryNode::Match(lumen::types::MatchQuery {
            field: field.to_string(),
            text: text.to_string(),
            op: lumen::types::MatchOp::And,
        }));
    }
    let raw = args
        .query_json
        .as_deref()
        .expect("exactly one branch checked above");
    serde_json::from_str(raw).context("--query-json is not a valid QueryNode")
}

/// Build the `POST /collections/{id}/search` body for `lumen query search`.
#[cfg(any(test, feature = "backup"))]
fn build_search_body(args: &QuerySearchArgs) -> Result<(String, serde_json::Value)> {
    let query = build_search_query_node(args)?;
    let request = lumen::types::SearchRequest {
        query,
        limit: args.limit,
        cursor: None,
        routing_key: None,
        sort: None,
        track_total: true,
        collapse: None,
    };
    let body = serde_json::to_value(&request).context("serialize SearchRequest")?;
    Ok((format!("/collections/{}/search", args.collection), body))
}

/// Build the `POST /collections/{id}/duplicates` body for `lumen query duplicates`.
#[cfg(any(test, feature = "backup"))]
fn build_duplicates_body(args: &QueryDuplicatesArgs) -> Result<(String, serde_json::Value)> {
    let request = lumen::types::DuplicatesRequest {
        field: args.field.clone(),
        min_group_size: args.min_group_size,
        limit: args.limit,
        offset: args.offset,
    };
    let body = serde_json::to_value(&request).context("serialize DuplicatesRequest")?;
    Ok((format!("/collections/{}/duplicates", args.collection), body))
}

#[cfg(feature = "backup")]
async fn http_post_json(
    base_url: &str,
    token: Option<&str>,
    path: &str,
    body: serde_json::Value,
) -> Result<serde_json::Value> {
    let url = format!("{}{path}", base_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let mut req = client.post(&url).json(&body);
    if let Some(token) = token {
        req = req.bearer_auth(token);
    }
    let resp = req.send().await.with_context(|| format!("POST {url}"))?;
    let status = resp.status();
    let payload: serde_json::Value = resp.json().await.unwrap_or(serde_json::Value::Null);
    if !status.is_success() {
        anyhow::bail!("POST {url} returned {status}: {payload}");
    }
    Ok(payload)
}

#[cfg(feature = "backup")]
async fn http_get_json(
    base_url: &str,
    token: Option<&str>,
    path: &str,
) -> Result<serde_json::Value> {
    let url = format!("{}{path}", base_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let mut req = client.get(&url);
    if let Some(token) = token {
        req = req.bearer_auth(token);
    }
    let resp = req.send().await.with_context(|| format!("GET {url}"))?;
    let status = resp.status();
    let payload: serde_json::Value = resp.json().await.unwrap_or(serde_json::Value::Null);
    if !status.is_success() {
        anyhow::bail!("GET {url} returned {status}: {payload}");
    }
    Ok(payload)
}

/// `lumen query` dispatch (#1321, R3): resolves `--url`/token via
/// `QueryTarget` (R2, shared with `lumen connect`), assembles the exact wire
/// body, and POSTs/GETs it. No REPL, no new HTTP endpoint.
/// @spec projects/lumen/tech-design/interfaces/cli/cli-connect-query-k8s-agent-workflow.md
#[cfg(feature = "backup")]
async fn dispatch_query(args: QueryArgs) -> Result<()> {
    match args.command {
        QueryCommand::Index(args) => {
            let base = resolve_base_url(&args.target)?;
            let token = resolve_token(&args.target, Some(&args.collection))?;
            let (path, body) = build_index_body(&args.collection, &args.items)?;
            let resp = http_post_json(&base, token.as_deref(), &path, body).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
            Ok(())
        }
        QueryCommand::Search(args) => {
            let base = resolve_base_url(&args.target)?;
            let token = resolve_token(&args.target, Some(&args.collection))?;
            let (path, body) = build_search_body(&args)?;
            let resp = http_post_json(&base, token.as_deref(), &path, body).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
            Ok(())
        }
        QueryCommand::Duplicates(args) => {
            let base = resolve_base_url(&args.target)?;
            let token = resolve_token(&args.target, Some(&args.collection))?;
            let (path, body) = build_duplicates_body(&args)?;
            let resp = http_post_json(&base, token.as_deref(), &path, body).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
            Ok(())
        }
        QueryCommand::Collections(args) => match args.command {
            QueryCollectionsCommand::List(args) => {
                let base = resolve_base_url(&args.target)?;
                let token = resolve_token(&args.target, None)?;
                let resp = http_get_json(&base, token.as_deref(), "/collections").await?;
                println!("{}", serde_json::to_string_pretty(&resp)?);
                Ok(())
            }
        },
    }
}

#[cfg(not(feature = "backup"))]
async fn dispatch_query(_args: QueryArgs) -> Result<()> {
    anyhow::bail!(
        "this lumen build was compiled without backup support; rebuild with \
         `--features backup` (or `operator`, which pulls it in — the published \
         image includes both)"
    )
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

/// Write `body` to `--out` (or stream it to stdout when `out` is `None`).
/// Chainable output (#963): the file-writing branch ends with exactly one
/// deterministic `next: <command>` line built from the resolved target path,
/// so an agent can copy-paste the follow-up; the stream-to-stdout branch
/// never emits one (nothing would separate it from the artifact bytes).
fn write_or_print(
    out: Option<&Path>,
    default_file: &str,
    body: &str,
    next: impl FnOnce(&Path) -> String,
) -> Result<()> {
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
        println!("next: {}", next(&target));
    } else {
        print!("{body}");
    }
    Ok(())
}

/// `next:` builder shared by every k8s render verb: the rendered manifest's
/// only sensible follow-up is applying it.
fn kubectl_apply_next(target: &Path) -> String {
    format!("kubectl apply -f {}", target.display())
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

    if apply_bootstrap_seed(&engine, args.bootstrap_seed_uri.as_deref())? {
        if let Some(limit) = args.bootstrap_max_bytes_per_sec {
            tracing::info!(
                max_bytes_per_sec = limit,
                "bootstrap seed applied; read throttle reserved for object-store fetchers"
            );
        }
    }

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
    // Live `ClusterState` for `AppState::with_cluster` (#1349): populated only
    // in raft mode, kept current for the process lifetime by
    // `spawn_cluster_state_poller` below. `None` here (standalone/legacy-log
    // backends) is correct — `enforce_read_consistency` no-ops when
    // `state.cluster` is `None`.
    #[cfg(feature = "raft-wal")]
    let mut raft_cluster: Option<Arc<lumen::raft::ClusterState>> = None;
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

            // Live cluster state (#1349): the same `ClusterConfig`/`RaftGroup`
            // shape `AppState::with_cluster`'s consumer (`enforce_read_consistency`,
            // `GET /debug/cluster`) already expects, seeded with the same
            // topology math as `topo` above (#1002 delegation keeps them from
            // drifting) so `group.peers` names line up with raft `NodeId`s
            // 1:1 by replica index.
            let cluster_cfg =
                lumen::config::ClusterConfig::from_env().context("raft: cluster config")?;
            let group = lumen::raft::RaftGroup::from_config(
                &cluster_cfg,
                "lumen",
                &headless,
                args.port,
                args.port,
            )
            .context("raft: build raft group")?;
            let cluster_state = Arc::new(
                lumen::raft::ClusterState::new(&cluster_cfg, group)
                    .context("raft: build cluster state")?,
            );
            spawn_cluster_state_poller(
                host.clone(),
                cluster_state.clone(),
                cluster_cfg.is_voter()?,
            );
            raft_cluster = Some(cluster_state);

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
    // Populate #1310's read-consistency enforcement seam with live cluster
    // state (#1349) — only in raft mode; standalone/legacy-log backends
    // correctly leave `state.cluster` `None` (single authoritative copy).
    #[cfg(feature = "raft-wal")]
    if let Some(cluster) = raft_cluster {
        state = state.with_cluster(cluster);
    }
    if !args.search_shard_segment_dirs.is_empty() {
        let shards = load_search_shard_segment_roots(&args.search_shard_segment_dirs)?;
        // #1384: route by the operator/reshard-driver-committed shard map
        // (SHARD_MAP_VERSION/SHARD_MAP_ASSIGNMENTS/VIRTUAL_BUCKET_COUNT env)
        // instead of always assuming the balanced default — queries for
        // buckets moved by a completed autonomous split must land on their
        // new physical shard.
        let shard_map = lumen::config::shard_map_from_env(args.shard_count).context(
            "shard map from env (SHARD_MAP_VERSION/SHARD_MAP_ASSIGNMENTS/VIRTUAL_BUCKET_COUNT)",
        )?;
        tracing::info!(
            shard_count = shards.len(),
            shard_map_version = shard_map.version(),
            "search backend=segment-sharded"
        );
        state = state.with_search_backend(Arc::new(
            lumen::routing::EngineShardSearch::new_with_shard_map(shards, shard_map),
        ));
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

/// Keeps `AppState.cluster` (#1310's read-consistency enforcement seam)
/// current for the process lifetime (#1349): polls the already-running
/// `RaftHost` for its live role/leader view (`is_leader`/`leader`, both
/// pre-existing — no new raft-host surface added) and republishes it onto
/// the shared `ClusterState` via its atomic setters, so every concurrently
/// running request handler observes the latest election result without a
/// restart. Runs for the life of the `serve` process; errors from the raft
/// host (e.g. transient watch-channel lag) are not fatal to serving and are
/// simply retried on the next tick.
///
/// Replication lag is reported as `0` on the leader and `u64::MAX`
/// ("unknown") on every follower/learner: deriving a true milliseconds-lag
/// figure would need new peer RPC surface this WI intentionally does not
/// add (see #1349's scope guardrail), so `ReadConsistency::Bounded` is kept
/// conservative — an unknown lag always fails the bound rather than
/// silently serving a stale follower.
#[cfg(feature = "raft-wal")]
fn spawn_cluster_state_poller(
    host: Arc<raft_host::RaftHost>,
    cluster: Arc<lumen::raft::ClusterState>,
    is_voter: bool,
) {
    use lumen::raft::RaftRole;
    let applied_rx = host.applied_watch();
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_millis(200));
        loop {
            ticker.tick().await;
            let is_leader = host.is_leader().await;
            let leader = host.leader().await;
            let role = if !is_voter {
                RaftRole::Learner
            } else if is_leader {
                RaftRole::Leader
            } else if leader.is_some() {
                RaftRole::Follower
            } else {
                RaftRole::Candidate
            };
            let prev = cluster.role();
            cluster.set_role(role);
            cluster.set_leader_index(leader.map(|n| n as u32));
            cluster.replication_lag_ms.store(
                if role == RaftRole::Leader {
                    0
                } else {
                    u64::MAX
                },
                std::sync::atomic::Ordering::Relaxed,
            );
            cluster
                .applied_index
                .store(*applied_rx.borrow(), std::sync::atomic::Ordering::Relaxed);
            if role != prev {
                tracing::info!(pod = %cluster.pod_name, from = ?prev, to = ?role, "raft role changed");
            }
        }
    });
}

fn apply_bootstrap_seed(engine: &Engine, seed_uri: Option<&str>) -> Result<bool> {
    let Some(seed_uri) = seed_uri else {
        return Ok(false);
    };
    let bytes = service_backup::fetch_backup_object(seed_uri)
        .with_context(|| format!("read bootstrap seed {seed_uri}"))?;
    let snap: lumen::storage::SnapshotV1 =
        serde_json::from_slice(&bytes).context("decode bootstrap SnapshotV1 JSON")?;
    engine.restore(snap).context("apply bootstrap seed")?;
    tracing::info!(
        seed_uri,
        bytes = bytes.len(),
        "bootstrap seed restored before WAL/raft catch-up"
    );
    Ok(true)
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

#[cfg(test)]
mod tests {
    use super::*;
    use lumen::types::{
        CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
        QueryNode, SearchRequest, TermQuery,
    };
    use std::collections::BTreeMap;

    #[test]
    fn bootstrap_seed_file_restores_snapshot_before_catchup() {
        let source = Engine::new();
        let mut fields = BTreeMap::new();
        fields.insert(
            "tag".to_string(),
            FieldSpec {
                field_type: FieldType::Keyword,
                analyzer: None,
                multi: None,
                dim: None,
                metric: None,
                backend: None,
                quantize: None,
            },
        );
        source
            .create_collection("c", CreateCollectionRequest { fields })
            .unwrap();
        source
            .index(
                "c",
                IndexRequest {
                    items: vec![IndexItem {
                        external_id: "doc-1".into(),
                        field: "tag".into(),
                        value: FieldValue::String("seeded".into()),
                        version: None,
                    }],
                    request_id: None,
                },
            )
            .unwrap();

        let path = std::env::temp_dir().join(format!(
            "lumen-bootstrap-seed-{}-{}.json",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        std::fs::write(
            &path,
            serde_json::to_vec(&source.snapshot().unwrap()).unwrap(),
        )
        .unwrap();

        let target = Engine::new();
        let uri = format!("file://{}", path.display());
        assert!(apply_bootstrap_seed(&target, Some(&uri)).unwrap());
        let result = target
            .search(
                "c",
                SearchRequest {
                    query: QueryNode::Term(TermQuery {
                        field: "tag".into(),
                        value: FieldValue::String("seeded".into()),
                    }),
                    limit: 10,
                    cursor: None,
                    routing_key: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
            .unwrap();
        assert_eq!(result.total, 1);
        assert_eq!(result.hits[0].external_id, "doc-1");

        let _ = std::fs::remove_file(path);
    }

    // -----------------------------------------------------------------
    // `lumen connect` / `lumen query` (#1321)
    // -----------------------------------------------------------------

    fn test_query_target() -> QueryTarget {
        QueryTarget {
            url: None,
            token: None,
            context: None,
            namespace: None,
            secret: None,
            role: TokenRole::Admin,
        }
    }

    #[test]
    fn resolve_base_url_requires_explicit_url() {
        let mut target = test_query_target();
        assert!(resolve_base_url(&target).is_err());
        target.url = Some("http://127.0.0.1:7373".to_string());
        assert_eq!(resolve_base_url(&target).unwrap(), "http://127.0.0.1:7373");
    }

    #[test]
    fn parse_field_value_prefers_json_then_falls_back_to_string() {
        assert!(matches!(parse_field_value("79"), FieldValue::Number(n) if n == 79.0));
        assert!(matches!(parse_field_value("acme"), FieldValue::String(s) if s == "acme"));
        assert!(
            matches!(parse_field_value("[0.1,0.2,0.9]"), FieldValue::Vector(v) if v.len() == 3)
        );
        assert!(matches!(
            parse_field_value(r#"["a","b"]"#),
            FieldValue::StringList(v) if v == vec!["a".to_string(), "b".to_string()]
        ));
    }

    #[test]
    fn parse_index_item_splits_external_id_field_value() {
        let item = parse_index_item("row-42:email=person@example.com").unwrap();
        assert_eq!(item.external_id, "row-42");
        assert_eq!(item.field, "email");
        assert!(matches!(item.value, FieldValue::String(ref s) if s == "person@example.com"));

        assert!(parse_index_item("missing-colon").is_err());
        assert!(parse_index_item("row-42:missing-equals").is_err());
    }

    /// AC3: `lumen query index`'s assembled body must match the FLAT shape
    /// `lumen spec --shapes` publishes for "index" — the reporter's bug was
    /// assuming a nested `{id, fields:{...}}` shape.
    #[test]
    fn build_index_body_matches_published_index_shape() {
        let (path, body) = build_index_body(
            "products",
            &[
                "row-42:email=person@example.com".to_string(),
                "row-42:price=79".to_string(),
            ],
        )
        .unwrap();
        assert_eq!(path, "/collections/products/index");

        let shapes = lumen::spec::query_shapes();
        let index_shape = shapes["shapes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["name"] == "index")
            .expect("query_shapes() publishes an `index` shape");
        let published = &index_shape["request"];

        assert!(body["items"].is_array());
        assert!(published["items"].is_array());
        assert_eq!(
            body["items"][0]
                .as_object()
                .unwrap()
                .keys()
                .collect::<std::collections::BTreeSet<_>>(),
            published["items"][0]
                .as_object()
                .unwrap()
                .keys()
                .collect::<std::collections::BTreeSet<_>>(),
            "assembled item keys must match the published flat {{external_id,field,value}} shape"
        );
        assert_eq!(body["items"][0]["external_id"], "row-42");
        assert_eq!(body["items"][0]["field"], "email");
        assert_eq!(body["items"][0]["value"], "person@example.com");
        assert_eq!(body["items"][1]["value"], 79.0);
    }

    #[test]
    fn build_search_query_node_requires_exactly_one_of_term_match_query_json() {
        let mut args = QuerySearchArgs {
            target: test_query_target(),
            collection: "products".into(),
            term: None,
            match_: None,
            query_json: None,
            limit: 20,
        };
        assert!(
            build_search_query_node(&args).is_err(),
            "none set should be rejected"
        );

        args.term = Some("status=active".to_string());
        let node = build_search_query_node(&args).unwrap();
        assert!(matches!(node, QueryNode::Term(TermQuery { ref field, .. }) if field == "status"));

        args.term = None;
        args.match_ = Some("title=earbuds".to_string());
        let node = build_search_query_node(&args).unwrap();
        assert!(matches!(node, QueryNode::Match(_)));

        args.term = Some("status=active".to_string());
        assert!(
            build_search_query_node(&args).is_err(),
            "both --term and --match set should be rejected"
        );
    }

    #[test]
    fn build_search_body_assembles_search_request_wire_shape() {
        let args = QuerySearchArgs {
            target: test_query_target(),
            collection: "products".into(),
            term: None,
            match_: Some("title=earbuds".to_string()),
            query_json: None,
            limit: 10,
        };
        let (path, body) = build_search_body(&args).unwrap();
        assert_eq!(path, "/collections/products/search");
        assert_eq!(body["query"]["match"]["field"], "title");
        assert_eq!(body["query"]["match"]["text"], "earbuds");
        assert_eq!(body["limit"], 10);
    }

    #[test]
    fn build_duplicates_body_matches_duplicates_request_shape() {
        let args = QueryDuplicatesArgs {
            target: test_query_target(),
            collection: "products".into(),
            field: "email".into(),
            min_group_size: 2,
            limit: 100,
            offset: 0,
        };
        let (path, body) = build_duplicates_body(&args).unwrap();
        assert_eq!(path, "/collections/products/duplicates");
        assert_eq!(body["field"], "email");
        assert_eq!(body["min_group_size"], 2);
        assert_eq!(body["limit"], 100);
        assert_eq!(body["offset"], 0);
    }

    // `select_token`/`cr_tokens_secret`/`secret_data_bytes`/
    // `wait_for_local_port_ready`/`ChildGuard` unit tests moved to
    // `libs/cli-std/src/connect.rs` (#1376) along with the primitives
    // themselves; lumen's own coverage is the thin-adapter tests above
    // (`resolve_base_url_requires_explicit_url`, `build_*_body_*`) plus
    // `cargo test -p cli-std --features k8s`.

    // -----------------------------------------------------------------
    // `spawn_cluster_state_poller` (#1349)
    // -----------------------------------------------------------------

    /// #1349 AC1/AC2 (unit-level): a single-voter `RaftHost` always wins its
    /// own election, so `spawn_cluster_state_poller` must converge the
    /// shared `ClusterState` from its pre-poller bootstrap value to
    /// `RaftRole::Leader` — driven by the real raft engine's own
    /// `is_leader`/`leader` results, not a manually-set role. This is the
    /// same seam `enforce_read_consistency` (#1310, `src/api.rs`) reads via
    /// `AppState.cluster`; the live 3-node localhost cluster in this WI's
    /// report additionally proves the HTTP-facing accept/reject behavior
    /// end-to-end.
    #[cfg(feature = "raft-wal")]
    #[tokio::test]
    async fn cluster_state_poller_converges_role_to_live_election_result() {
        use lumen::raft::{ClusterState, PeerAddr, RaftGroup, RaftRole};

        let tmp = std::env::temp_dir().join(format!(
            "lumen-cluster-poller-test-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("t")
        ));
        let _ = std::fs::create_dir_all(&tmp);
        let sm = lumen::raft_sm::EngineSm::new(Arc::new(Engine::new()), 0);
        let host = Arc::new(raft_host::RaftHost::spawn(
            0,
            raft_host::Membership {
                voters: vec![0],
                learners: vec![],
            },
            std::collections::HashMap::new(),
            raft_host::RaftStore::open(tmp.to_str().unwrap(), 0, raft_host::FsyncPolicy::Os)
                .unwrap(),
            sm.clone() as Arc<dyn raft_host::RaftStateMachine>,
            raft_host::HostConfig::default(),
        ));

        // Bootstrap value deliberately wrong (Follower/no-leader), matching
        // how a real pod starts before its first poller tick — proves the
        // assertion below observes the poller's live update, not the
        // constructor's static default.
        let cluster = Arc::new(ClusterState::from_snapshot(
            "lumen-0".to_string(),
            0,
            0,
            RaftRole::Follower,
            RaftGroup {
                shard_index: 0,
                peers: vec![PeerAddr {
                    pod_name: "lumen-0".to_string(),
                    host: "127.0.0.1".to_string(),
                    raft_port: 0,
                    client_port: 0,
                    role: RaftRole::Follower,
                }],
            },
            0,
            1,
            u64::MAX,
        ));
        assert_eq!(cluster.role(), RaftRole::Follower, "bootstrap sanity check");

        spawn_cluster_state_poller(host, cluster.clone(), true);

        let converged = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if cluster.role() == RaftRole::Leader {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        })
        .await;
        assert!(
            converged.is_ok(),
            "poller did not converge role to Leader within bound"
        );
        assert_eq!(cluster.leader_index(), Some(0));
        assert_eq!(
            cluster
                .replication_lag_ms
                .load(std::sync::atomic::Ordering::Relaxed),
            0,
            "leader reports zero lag, not the unknown sentinel"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
// CODEGEN-END
