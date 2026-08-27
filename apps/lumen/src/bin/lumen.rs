// CODEGEN-BEGIN
//! `lumen` — the single agent-first CLI: `serve` (serving node), `spec` /
//! `llm` (offline integration contract + agent topics), and `k8s` (operator
//! + CRD generation). Agents start here: `lumen llm --topic outline`.
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
//!
//! ## Contracts inherited from the retired EC shells
//!
//! This sentence was the whole of the `// Contract:` comment in an AW-EC shell under
//! `apps/lumen/e2e/`, which ran `cargo test -p lumen --bin lumen` in a subprocess and
//! asserted the child's exit status. `cargo test -p lumen` already runs this binary's
//! colocated unit tests directly, so the shell added a second, nested run and nothing
//! else. It was deleted on 2026-08-20 with the EC machinery it belonged to, and the
//! sentence is the only thing it held that nothing else did. The line below is prefixed
//! with the EC id the shell was filed under.
//!
//! - `lumen-claim-topology-empty-pvc-bootstrap-seed` — A fresh serving process restores
//!   a configured SnapshotV1 seed before WAL or raft catch-up.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
#[cfg(feature = "operator")]
use tracing_subscriber::EnvFilter;

use lumen::auth::{AuthConfig, AuthProfile};
use lumen::coordinator::WriteCoordinator;
use lumen::rdb::{LocalFsRdbStore, RdbSnapshot, RdbStore};
use lumen::storage::Engine;
use lumen::wal::{MemWal, SharedWal};
use lumen::wal_nats::NatsWal;

#[path = "lumen/standalone.rs"]
mod standalone;

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
    /// Manage the local single-node Compose deployment.
    Standalone(StandaloneArgs),
    /// Run a serving node (HTTP API + background apply loop).
    Serve(ServeArgs),
    /// Print lumen's machine-readable integration spec — offline, no server.
    /// Default: the OpenAPI 3 JSON document; `--format openapi-yaml` for
    /// LLM-readable OpenAPI YAML; `--format json-schema` for the data types;
    /// `--shapes` for the query-shape cookbook; `--fields` for the field-type /
    /// analyzer catalog.
    Spec(SpecArgs),
    /// Print agent-facing task topics — offline, no server. `outline` maps the
    /// available tasks. Topics that include planned behavior distinguish it
    /// from current support. Tasks name canonical sources and runnable
    /// verification steps. Shared provider content stays owned by its library
    /// and is composed into Lumen.
    /// Markdown is the default; use `--format json` for machine-readable output.
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
    Upgrade(UpgradeArgs),
    /// Search, view, and file Lumen issues on the axiom tracker.
    /// `search` and `view` read existing `app:lumen` issues; `create`
    /// files a diagnostics-rich issue tagged `app:lumen`.
    Issue(IssueArgs),
    /// Fetch a snapshot from a running serving fleet's own `/admin/backup`
    /// and ship it to a destination (`file://`, `s3://`, or `gs://`) via
    /// `libs/service-backup`. GCS uses an explicit access token or GKE Workload
    /// Identity. No new snapshot mechanism — this only
    /// schedules and transports the existing admin API. Typically invoked by
    /// the operator's optional backup CronJob (`spec.serving.backup`, see
    /// `lumen llm --topic storage`), but works standalone. Requires the `backup`
    /// feature (pulled in transitively by `operator`).
    Backup(BackupArgs),
    /// Manage a `kubectl port-forward` for the duration of a wrapped command
    /// against a k8s-deployed Lumen instance — no manually tracked
    /// port-forward process (`lumen llm --topic recipes` has a worked
    /// example). Reachability only: the child is handed a URL and nothing
    /// else. Obtaining a Kubernetes ServiceAccount token for it is #2878's
    /// job, and until then there is no credential to obtain.
    Connect(ConnectArgs),
    /// One-shot query wrappers against a reachable lumen node: `index`,
    /// `search`, `duplicates`, `collections list`. Assembles the exact wire
    /// body `lumen spec --shapes` publishes — no interactive REPL. Requires
    /// the `backup` feature (pulled in transitively by `operator`).
    Query(QueryArgs),
}

#[derive(clap::Args)]
pub(crate) struct StandaloneArgs {
    #[command(subcommand)]
    pub(crate) cmd: StandaloneCmd,
}

#[derive(Subcommand)]
pub(crate) enum StandaloneCmd {
    Compose(StandaloneComposeArgs),
    Gke(StandaloneGkeArgs),
}

#[derive(clap::Args)]
pub(crate) struct StandaloneGkeArgs {
    #[command(subcommand)]
    pub(crate) cmd: StandaloneGkeCmd,
}

#[derive(Subcommand)]
pub(crate) enum StandaloneGkeCmd {
    Init(StandaloneGkeInitArgs),
    Render(StandaloneGkeRenderArgs),
}

#[derive(clap::Args)]
pub(crate) struct StandaloneGkeInitArgs {
    #[arg(long)]
    pub(crate) out: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct StandaloneGkeRenderArgs {
    #[arg(long)]
    pub(crate) file: PathBuf,
    #[arg(long)]
    pub(crate) out: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct StandaloneComposeArgs {
    #[command(subcommand)]
    pub(crate) cmd: StandaloneComposeCmd,
}

#[derive(Subcommand)]
pub(crate) enum StandaloneComposeCmd {
    Patch(StandaloneComposePatchArgs),
}

#[derive(clap::Args)]
pub(crate) struct StandaloneComposePatchArgs {
    #[arg(long)]
    pub(crate) file: PathBuf,
    #[arg(long, default_value = "lumen")]
    pub(crate) name: String,
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
    /// Control-plane fleet declaration: render the one cluster-scoped
    /// `LumenFleet` that names every data-plane namespace and its settings.
    /// Use this instead of `instance` when the platform team owns
    /// configuration centrally and app teams only own their own overrides.
    Fleet(K8sFleetArgs),
    /// Client access layer: render the RBAC that lets a named Kubernetes user
    /// mint one client ServiceAccount's token, and that tells Lumen what that
    /// ServiceAccount may do.
    Access(K8sAccessArgs),
}

#[derive(clap::Args)]
struct K8sFleetArgs {
    #[command(subcommand)]
    cmd: K8sFleetCmd,
}

#[derive(Subcommand)]
enum K8sFleetCmd {
    /// Render a cluster-scoped `kind: LumenFleet` declaration.
    Render(K8sFleetRenderArgs),
}

#[derive(clap::Args)]
struct K8sFleetRenderArgs {
    /// Built-in fleet profile.
    #[arg(long, value_enum, default_value_t = K8sFleetProfile::Template)]
    profile: K8sFleetProfile,
    /// LumenFleet name. Also the default name of every `Lumen` it
    /// materializes, so `kubectl get lumen -A` reads as one fleet spread
    /// across namespaces.
    #[arg(long)]
    name: Option<String>,
    /// Serving image for `spec.defaults`. Profile-specific default.
    #[arg(long)]
    image: Option<String>,
    /// Write to this path instead of stdout. A directory receives
    /// `lumenfleet.yaml`.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Clone, Copy, ValueEnum)]
enum K8sFleetProfile {
    /// One local/kind data plane in `default`, auth disabled.
    Dev,
    /// Two tenant namespaces on a dedicated node pool, auth required.
    Prod,
    /// Fill-in-the-blanks skeleton naming every knob a deployer owns:
    /// node pool, StorageClass, ServiceAccount, per-tenant CPU/memory/disk,
    /// and per-tenant credential source.
    Template,
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

#[derive(clap::Args, Clone, Debug, Default)]
struct K8sOperatorRunArgs {}

#[derive(clap::Args)]
struct K8sOperatorArgs {
    #[command(subcommand)]
    cmd: Option<K8sOperatorCmd>,
}

impl Default for K8sOperatorCmd {
    fn default() -> Self {
        Self::Run(K8sOperatorRunArgs::default())
    }
}

#[derive(Subcommand)]
enum K8sOperatorCmd {
    /// Container entrypoint: run the reconcile controller.
    Run(K8sOperatorRunArgs),
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
    /// Operator container image. Supply an immutable registry digest for
    /// reproducible cluster deployment; the default is this build's
    /// published GHCR release, matching the checked-in operator manifest.
    #[arg(long, default_value_t = format!("ghcr.io/chrischeng-c4/lumen:{}", env!("CARGO_PKG_VERSION")))]
    image: String,
    /// Also emit the operator's ServiceMonitor and PrometheusRule (#2621).
    /// Off by default because both are `monitoring.coreos.com/v1` CRDs and a
    /// cluster without prometheus-operator rejects the whole apply; the
    /// scrape *target* Service carries no CRD dependency and is always
    /// rendered. Mirrors the opt-in `k8s/components/operator-monitoring`
    /// kustomize component.
    #[arg(long)]
    monitoring: bool,
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
struct K8sAccessArgs {
    #[command(subcommand)]
    cmd: K8sAccessCmd,
}

#[derive(Subcommand)]
enum K8sAccessCmd {
    /// Render the client access bundle: a ServiceAccount, the RBAC that lets
    /// named users mint its token, and the RBAC Lumen reads to authorize it.
    Render(K8sAccessRenderArgs),
}

/// `lumen k8s access render` flags (#2889).
///
/// Everything here is a name. There is no flag that takes a credential,
/// because the bundle this renders contains none: the caller's identity is
/// minted by the API server on demand, and the only durable objects are the
/// two grants that say who may ask for it and what it may then do.
#[derive(clap::Args)]
struct K8sAccessRenderArgs {
    /// Namespace holding the Lumen instance. The whole bundle lands here:
    /// both grants are namespaced, so an access decision never leaks past the
    /// tenant that made it.
    #[arg(long)]
    namespace: String,
    /// The one ServiceAccount every request to Lumen is made as. Lumen sees
    /// this name — `system:serviceaccount:<namespace>:<name>` — and nothing
    /// about whoever minted the token.
    #[arg(long = "client-sa")]
    client_sa: String,
    /// A Kubernetes user allowed to mint that ServiceAccount's token, spelled
    /// exactly as `kubectl auth whoami` prints it for that principal.
    /// Repeatable. A Google account and a Google service account are both just
    /// strings here: they authenticate to the API server, never to Lumen.
    #[arg(long = "issuer", required = true)]
    issuers: Vec<String>,
    /// `<collection-id>=read|write|admin`. Repeatable, one collection each.
    /// A level grants every verb at or below it, so `write` can read what it
    /// writes.
    #[arg(long = "grant")]
    grants: Vec<String>,
    /// Also grant the instance-wide administrative surface — backup, restore,
    /// reshard, checkpoint. Separate from any collection grant on purpose.
    #[arg(long)]
    instance_admin: bool,
    /// Write to this path instead of stdout. A directory receives
    /// `access.yaml`.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(clap::Args)]
struct K8sFileOutputArgs {
    /// Write to this path instead of stdout.
    #[arg(long)]
    out: Option<PathBuf>,
}

/// `lumen upgrade` flags.
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
    /// `gs://bucket/prefix`. GCS uses an explicit access token or the
    /// GCE/GKE metadata-server Workload Identity token.
    #[arg(long)]
    dest: String,
    /// Drop backup objects older than this many seconds after a successful
    /// put. Omit to keep everything.
    #[arg(long)]
    retention_secs: Option<u64>,
    /// File holding this runner's own audience-bound ServiceAccount token,
    /// read fresh for this run (#2877). In-cluster this is the projected
    /// volume the operator's backup CronJob mounts. A path, never the token:
    /// a credential passed as an argument is visible in the pod spec, in
    /// `ps`, and in every shell history that ever typed it. Omit against a
    /// fleet whose `spec.auth` is `disabled` — it rejects a presented bearer.
    #[arg(long, value_name = "PATH")]
    token_file: Option<std::path::PathBuf>,
}

/// `lumen connect` flags (#1321): manage a `kubectl port-forward` around a
/// wrapped command so an agent never tracks the port-forward process itself.
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
    /// name defaults to this CR's own name.
    #[arg(long)]
    cr: Option<String>,
    /// Local port to forward to. Omit to pick a free ephemeral port.
    #[arg(long)]
    local_port: Option<u16>,
    /// Remote (Service) port.
    #[arg(long, default_value_t = 7373)]
    remote_port: u16,
    /// ServiceAccount to authenticate as (#2878). Named, never inferred: this
    /// CLI will not pick one by listing the namespace or by falling back to
    /// `default`, because a token minted for an account nobody chose is
    /// exactly as authorized as one somebody did choose.
    ///
    /// With it, a short-lived audience-bound token is minted through your
    /// kubeconfig and held in this process, and the wrapped command talks to a
    /// loopback proxy that attaches it — so the token is in no environment
    /// variable, no argument list, and no file. Without it the port-forward
    /// carries no credential at all, which only works against a fleet whose
    /// `spec.auth` is `disabled`.
    ///
    /// You need `create` on this ServiceAccount's `token` subresource; `lumen
    /// k8s access render` emits that grant.
    #[arg(long, value_name = "NAME")]
    client_sa: Option<String>,
    /// PEM bundle of the private CA that signed the fleet's serving certificate
    /// (#3113 R6). The deployment administrator or external certificate
    /// platform distributes this public CA separately from the serving Secret.
    ///
    /// With it, the forwarded socket is spoken to over TLS addressed as
    /// `--server-name`, verified against this bundle and against no public root.
    /// The port-forward is transport; the identity being checked is the
    /// Kubernetes Service's, which is what the certificate actually names.
    ///
    /// Requires `--client-sa`: the verifying connection is made by the local
    /// proxy, and the proxy exists to hold a token. A TLS fleet that accepts no
    /// credential is not a deployment this command has to serve.
    #[arg(
        long,
        value_name = "PATH",
        conflicts_with = "plaintext",
        requires = "client_sa"
    )]
    ca_file: Option<std::path::PathBuf>,
    /// The DNS name the serving certificate must present. Defaults to
    /// `<service>.<namespace>.svc`, which is what the operator requests.
    ///
    /// Override it only when the Service is reached under another of its
    /// certified names (its cluster FQDN, say). `localhost` and `127.0.0.1` are
    /// not among them, and a leaf that carried either would be a leaf usable
    /// against every port-forward anyone ever opens.
    #[arg(long, value_name = "DNS")]
    server_name: Option<String>,
    /// Talk to the forwarded port in cleartext — local and kind development,
    /// where no serving certificate has been issued (#3113 R1).
    ///
    /// Required to be said out loud, because the alternative is a default that
    /// downgrades silently: a production fleet reached without `--ca-file`
    /// would fail somewhere inside the wrapped command's first request instead
    /// of here, and the fix would look like a networking problem.
    #[arg(long)]
    plaintext: bool,
    /// The command to run with `LUMEN_URL` set to the local end of the
    /// port-forward — and nothing else. Everything after `--`, e.g. `lumen
    /// connect --namespace prod --cr search --ca-file ca.crt --client-sa agent
    /// -- lumen query collections list`.
    #[arg(last = true, required = true)]
    command: Vec<String>,
}

/// Where `lumen query *` sends its request, and whose token it carries.
///
/// #2873 removed the bearer flag, the environment variable behind it, and the
/// kubectl Secret lookup behind that — every mechanism whose job was to *find*
/// a credential lying around. #2878 restores a `--context`/`--namespace` pair
/// that looks superficially similar and is not the same thing: nothing here
/// reads a stored credential. `--client-sa` names a ServiceAccount, and the
/// token is minted for it, in memory, for this one command, through the
/// identity already in your kubeconfig.
///
/// There is deliberately no environment variable for `--client-sa`. Which
/// account you act as is a decision each invocation makes out loud.
#[derive(clap::Args, Clone)]
struct QueryTarget {
    /// Base URL of a reachable lumen serving node, e.g. `http://localhost:7373`
    /// — what `lumen connect` sets for the wrapped command.
    #[arg(long, env = "LUMEN_URL")]
    url: Option<String>,
    /// kubeconfig context to mint the token through. Omit for the current one.
    #[arg(long)]
    context: Option<String>,
    /// Namespace of the ServiceAccount named by `--client-sa`.
    #[arg(long)]
    namespace: Option<String>,
    /// ServiceAccount to authenticate as (#2878). Named, never inferred.
    /// Omit to send no credential at all — correct only against a fleet whose
    /// `spec.auth` is `disabled`, and what `lumen connect --client-sa` already
    /// arranges for its wrapped command.
    #[arg(long, value_name = "NAME", requires = "namespace")]
    client_sa: Option<String>,
}

/// `lumen query <index|search|duplicates|collections>` flags (#1321): thin
/// one-shot wrappers assembling the exact `lumen spec --shapes` wire body.
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
    /// `GET /collections` — list collection ids the serving node exposes.
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
}

#[derive(Clone, Copy, ValueEnum)]
enum LlmTopic {
    /// Typed task map for agent context selection (default).
    Outline,
    /// Start one local development or test runtime without Kubernetes.
    RunStandalone,
    /// Inspect the offline search contract before issuing a request.
    LocalSearch,
    /// Declare or review a collection schema.
    ModelSchema,
    /// Select a supported search, filter, range, sort, kNN, or duplicate query.
    SelectQuery,
    /// Separate current query support from the documented 0.5 search target.
    Querying,
    /// Connect a source database, CDC stream, or outbox to Lumen.
    IntegrateSourceDb,
    /// Inspect the request-authentication contract: the Kubernetes
    /// ServiceAccount identity Lumen accepts, and the credential kinds it
    /// refuses.
    Authenticate,
    /// Use a bounded Kubernetes port-forward connection.
    ConnectKubernetes,
    /// Render image, CRD, operator, or instance deployment artifacts.
    DeployKubernetes,
    /// Give an external Kubernetes user access to a Lumen instance through a
    /// client ServiceAccount.
    GrantAccess,
    /// Create or restore an administrative backup.
    BackupRestore,
    /// Generate a typed Rust, Python, or TypeScript client.
    GenerateClient,
    /// Inspect standard operational evidence from a running service.
    Diagnose,
    /// Verify one release candidate through local, kind, and public artifact
    /// evidence.
    VerifyRelease,
}

impl LlmTopic {
    const fn id(self) -> &'static str {
        match self {
            Self::Outline => "outline",
            Self::RunStandalone => "run-standalone",
            Self::LocalSearch => "local-search",
            Self::ModelSchema => "model-schema",
            Self::SelectQuery => "select-query",
            Self::Querying => "querying",
            Self::IntegrateSourceDb => "integrate-source-db",
            Self::Authenticate => "authenticate",
            Self::ConnectKubernetes => "connect-kubernetes",
            Self::DeployKubernetes => "deploy-kubernetes",
            Self::GrantAccess => "grant-access",
            Self::BackupRestore => "backup-restore",
            Self::GenerateClient => "generate-client",
            Self::Diagnose => "diagnose",
            Self::VerifyRelease => "verify-release",
        }
    }
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
    if raft_runtime::cluster::replica_mode() {
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
    /// Pinned generated-client contract, e.g. `python-3.14`. Defaults to
    /// `clients/codegen.toml`; an explicit value overrides that policy once.
    #[arg(long, value_name = "TARGET")]
    target: Option<String>,
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
    /// Deliberately `Option<u32>` with no `default_value_t`: this is the
    /// only clap-native way to tell "the operator/user actually set
    /// `--shard-count`/`SHARD_COUNT`" (`Some`) apart from "nobody set it"
    /// (`None`) — the segment-dirs fan-in path (below) needs that
    /// distinction to default to the loaded-dir count instead of silently
    /// assuming 1 (#1398 R4). Non-fan-in call sites treat `None` as 1.
    #[arg(long, env = "SHARD_COUNT")]
    shard_count: Option<u32>,
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
        Command::Standalone(args) => standalone::run(args),
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
            // Raw spec bytes are a public artifact: this CLI output, the live
            // `/openapi.json` route, `spec gen`, and the committed snapshot all
            // consume `spec::openapi_json()` without an extra wrapper/newline.
            print!("{out}");
            Ok(())
        }
        Command::Llm(args) => {
            // Offline: no engine, no server, no I/O beyond stdout.
            let format = match args.format {
                LlmFormat::Md => cli_std::llm::Format::Md,
                LlmFormat::Json => cli_std::llm::Format::Json,
            };
            let out = lumen::dx::render_llm(args.topic.id(), format)?;
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
fn spec_gen(args: GenArgs) -> Result<()> {
    use openapi_codegen::{
        generate_for_target_with_file_bearer_auth, FileBearerAuth, FileBearerScheme, GenOptions,
        HttpClient, Lang, TargetPolicy, MANIFEST_FILE,
    };

    const TARGET_POLICY: &str = include_str!("../../clients/codegen.toml");

    let lang = match args.lang {
        GenLang::Ts => Lang::Ts,
        GenLang::Py => Lang::Py,
        GenLang::Rust => Lang::Rust,
    };
    let target = TargetPolicy::from_toml(TARGET_POLICY)?.resolve(lang, args.target.as_deref())?;
    let opts = GenOptions {
        lang,
        target: Some(target),
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
    let auth = FileBearerAuth::new(
        "/var/run/secrets/kubernetes.io/serviceaccount/token",
        ".svc.cluster.local",
        [FileBearerScheme::Http, FileBearerScheme::Https],
    )?;
    let output = generate_for_target_with_file_bearer_auth(
        &lumen::spec::openapi_json(),
        &opts,
        target,
        &auth,
    )?;
    output.write_to_dir(&args.out)?;
    for file in &output.files {
        let path = args.out.join(&file.rel_path);
        println!("generated {}", path.display());
    }
    println!("generated {}", args.out.join(MANIFEST_FILE).display());
    let requirements = output.requirements.expect("explicit target requirements");
    println!(
        "target: {} (minimum {} {})",
        requirements.target,
        requirements.language.id(),
        requirements.minimum_version
    );
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
            let tag = cli_std::artifact::release_tag("lumen", version, env!("CARGO_PKG_VERSION"));
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
        K8sCmd::Operator(args) => match args.cmd.unwrap_or_default() {
            K8sOperatorCmd::Run(run_args) => run_operator(run_args).await,
            K8sOperatorCmd::Render(args) => {
                let yaml = render_operator_yaml(&args)?;
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
        K8sCmd::Fleet(args) => match args.cmd {
            K8sFleetCmd::Render(args) => {
                let yaml = render_fleet_yaml(&args);
                write_or_print(
                    args.out.as_deref(),
                    "lumenfleet.yaml",
                    &yaml,
                    kubectl_apply_next,
                )
            }
        },
        K8sCmd::Access(args) => match args.cmd {
            K8sAccessCmd::Render(args) => {
                let yaml = render_access_yaml(&args)?;
                write_or_print(
                    args.out.as_deref(),
                    "access.yaml",
                    &yaml,
                    kubectl_apply_next,
                )
            }
        },
    }
}

#[cfg(feature = "operator")]
async fn run_operator(args: K8sOperatorRunArgs) -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    let _ = args;
    lumen::operator::run().await
}

#[cfg(not(feature = "operator"))]
async fn run_operator(_args: K8sOperatorRunArgs) -> Result<()> {
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
    cli_std::artifact::ensure_trailing_newline(include_str!("../../k8s/operator/crd.yaml"))
}

/// Whatever eventually authenticates `lumen backup` to the admin API, it will
/// not be a string on the command line. #2871 took away the metadata-server
/// fallback; #2873 takes away the bearer flag that was left, because a
/// credential passed as an argument is a credential in `ps`, in shell history,
/// and in the CronJob's own `kubectl describe` — the exact exposure R5 rules
/// out. The `token` parameter on the `lumen::backup` calls below is the seam
/// #2877 fills with a projected, audience-bound ServiceAccount token read from
/// a file; until then it is `None` and the request carries no `Authorization`
/// header at all.
///
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
    // Read here, not at parse time: one backup run is one read of the
    // projected file, so a CronJob pod that starts minutes after the kubelet
    // last rotated the token still presents the current one (#2877 R3).
    // Missing, empty, expired, or minted for the wrong audience all fail the
    // run before any bytes move, with a message naming the path rather than
    // the material (#2877 R5).
    let token = match args.token_file.as_deref() {
        Some(path) => Some(
            service_auth::k8s::ProjectedTokenFile::new(path, lumen::auth::AUDIENCE)
                .read()
                .with_context(|| "the backup runner cannot authenticate to this Lumen fleet")?,
        ),
        None => None,
    };
    let result = lumen::backup::run_backup(
        &args.url,
        token
            .as_ref()
            .map(service_auth::k8s::ProjectedToken::expose),
        &dest,
        &retention,
    )
    .await?;
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
/// command. The command carries no `Authorization` header: there is no
/// credential for it to carry (#2873), and a placeholder one would read as an
/// instruction to go find a token that does not exist.
#[cfg(feature = "backup")]
fn restore_next_command(args: &BackupArgs, result: &service_backup::BackupRunResult) -> String {
    let url = args.url.trim_end_matches('/');
    match result.object.sink.strip_prefix("local:") {
        Some(root) => format!(
            "curl -sS -X POST {url}/admin/restore -H 'Content-Type: application/json' --data-binary @{}/{}",
            root.trim_end_matches('/'),
            result.object.key
        ),
        None => format!(
            "fetch {} from {} then: curl -sS -X POST {url}/admin/restore -H 'Content-Type: application/json' --data-binary @<downloaded-file>",
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
    let payload = lumen::backup::fetch_snapshot_bytes(&args.url, None).await?;
    if let Some(out) = args.out {
        if let Some(parent) = out.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        std::fs::write(&out, &payload).with_context(|| format!("write {}", out.display()))?;
        let next = restore_file_next_command(args.url.trim_end_matches('/'), &out);
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
    lumen::backup::restore_snapshot_bytes(&args.url, None, &payload).await?;
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
fn restore_file_next_command(url: &str, path: &Path) -> String {
    format!("lumen import --url {url} --file {}", path.display())
}

// ---------------------------------------------------------------------------
// `lumen connect` / `lumen query` (#1321) — thin adapter over
// `cli_std::connect` (#1376): the `kubectl port-forward` process lifecycle
// (`ChildGuard`, `free_local_port`, `wait_for_local_port_ready`) lives in
// `libs/cli-std/src/connect.rs`, reusable by any k8s-native service CLI.
// This file keeps only its own flag surface (`ConnectArgs`/`QueryTarget`) and
// the `Lumen` CRD-name lookup convention (`"lumen"` passed as
// `resource_kind`).
//
// #2873 cut the credential half away entirely. The shared module's resolver
// chain — kubectl-get the Secret named by the CR, base64-decode the registry
// key inside it, pick an entry whose role covers the request — is still there
// for the services that have not migrated, but lumen no longer calls any of
// it: the registry it decoded stopped existing in #2871, and the CR field
// naming the Secret stopped existing in #2872. What remains here is a
// port-forward, and nothing that reads, derives, prints, or passes on a
// credential.
// ---------------------------------------------------------------------------

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
/// local end is reachable, run the wrapped command with `LUMEN_URL` set, then
/// tear the port-forward down (`ChildGuard::drop`) once the wrapped command
/// exits — regardless of its exit status — so no port-forward process is left
/// for the caller to track.
///
/// The child's environment gains exactly one variable, and it is a URL. It
/// used to also gain a bearer token, which meant every descendant of the
/// wrapped command — and anything that could read `/proc/<pid>/environ` —
/// inherited a bearer nobody had scoped to them. #2873 deleted that token;
/// #2878 gives the credential back without giving it to the child: with
/// `--client-sa`, the token is minted by `TokenRequest` through the caller's
/// own kubeconfig, held in this process, and attached to the child's requests
/// as they pass through a loopback proxy. The URL the child gets points at the
/// proxy rather than at the port-forward, and that is the only difference the
/// child can observe.
async fn connect(args: ConnectArgs) -> Result<()> {
    let service = args
        .service
        .clone()
        .or_else(|| args.cr.clone())
        .context("--service or --cr is required")?;

    // Which name this connection is *for*, as opposed to which socket it goes
    // through (#3113 R6). In TLS mode the upstream URL carries the Service's
    // own DNS name, so SNI, hostname verification, and the `Host` header all
    // address the identity the serving certificate asserts; only address
    // resolution points at the tunnel.
    let server_name = args
        .server_name
        .clone()
        .unwrap_or_else(|| format!("{service}.{}.svc", args.namespace));

    // Settled before anything is spawned: a transport nobody chose is not a
    // thing to discover after a port-forward is running.
    if args.ca_file.is_none() && !args.plaintext {
        anyhow::bail!(
            "say how this connection is secured: `--ca-file <PATH>` verifies {server_name} \
             against the fleet's published trust bundle, and `--plaintext` talks to a \
             development instance that serves no certificate.\nThere is no default because the \
             wrong one is silent: cleartext against a TLS fleet fails inside the wrapped \
             command's first request, and reads as a networking problem rather than as a \
             transport that was never chosen."
        );
    }

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

    let trust = match &args.ca_file {
        Some(path) => Some(serving_trust(path, &server_name, local_port)?),
        None => None,
    };
    let upstream = match &trust {
        Some(_) => format!("https://{server_name}"),
        None => format!("http://127.0.0.1:{local_port}"),
    };

    // One probe before anything else runs. Every way TLS goes wrong here —
    // the wrong CA, the wrong name, an expired leaf, a fleet still serving
    // cleartext — is a fact about the deployment, and it should be reported
    // as one rather than surfacing as the wrapped command's first request
    // failing with a transport error (#3113 R7).
    if let Some(client) = &trust {
        probe_serving_tls(client, &server_name, args.ca_file.as_deref()).await?;
    }

    // Mint before spawning anything. A missing RBAC grant is the most common
    // way this command fails, and it should fail here — where the error can
    // name the account and the check — rather than three layers down inside
    // the child's first HTTP response (R6).
    let mut proxy = match &args.client_sa {
        Some(client_sa) => Some(start_client_proxy(&args, client_sa, &upstream, trust).await?),
        None => None,
    };

    let child_url = match &proxy {
        Some(proxy) => {
            eprintln!(
                "lumen connect: forwarding {} -> {upstream} -> svc/{service}:{} in {}, \
                 authenticating as serviceaccount {}/{}{}. The token is held here; the wrapped \
                 command sees only the local URL.",
                proxy.local_url(),
                args.remote_port,
                args.namespace,
                args.namespace,
                args.client_sa.as_deref().unwrap_or_default(),
                match &args.ca_file {
                    Some(path) => format!(", verifying {server_name} against {}", path.display()),
                    None => ", over cleartext (--plaintext)".to_string(),
                },
            );
            proxy.local_url()
        }
        None => {
            // R4: say out loud that this connection is unauthenticated, on
            // stderr so the wrapped command's own stdout stays
            // machine-readable. A caller who gets a 401 deserves to be told
            // why here, not left to infer it from the server's response.
            eprintln!(
                "lumen connect: forwarding {upstream} -> svc/{service}:{} in {} with no \
                 credential. Pass --client-sa <NAME> to authenticate as a ServiceAccount; \
                 without it a serving instance with `auth: required` refuses every request, and \
                 `auth: disabled` accepts them all.",
                args.remote_port, args.namespace
            );
            upstream.clone()
        }
    };

    let (program, rest) = args
        .command
        .split_first()
        .context("wrapped command is empty")?;
    let mut child_cmd = tokio::process::Command::new(program);
    child_cmd.args(rest);
    child_cmd.env("LUMEN_URL", &child_url);
    let mut child = child_cmd.spawn().context("run wrapped command")?;

    // Three ways this ends, and all three must tear down the port-forward and
    // the proxy (R7). The child exiting is the ordinary one. A token refresh
    // that fails means the grant was revoked mid-session: keeping the child
    // alive would leave it talking to a proxy that can only answer 503, so we
    // end it. Ctrl-C is the caller ending it.
    enum Ending {
        Child(std::process::ExitStatus),
        Refresh(String),
        Interrupted,
    }
    let ending = tokio::select! {
        status = child.wait() => Ending::Child(status.context("wait for the wrapped command")?),
        fatal = next_fatal(proxy.as_mut()) => Ending::Refresh(fatal),
        signal = tokio::signal::ctrl_c() => {
            signal.context("listen for interrupt")?;
            Ending::Interrupted
        }
    };

    // `_forward` and `proxy` drop at the end of this scope on every path, so
    // the port-forward process and the loopback listener go with them.
    match ending {
        Ending::Child(status) if status.success() => Ok(()),
        Ending::Child(status) => anyhow::bail!("wrapped command exited with {status}"),
        Ending::Refresh(detail) => {
            let _ = child.start_kill();
            let _ = child.wait().await;
            anyhow::bail!(
                "the wrapped command was stopped because its token could not be renewed: {detail}"
            )
        }
        Ending::Interrupted => {
            let _ = child.start_kill();
            let _ = child.wait().await;
            anyhow::bail!("interrupted")
        }
    }
}

/// The proxy's fatal-error arm of `connect`'s `select!`. Without a proxy there
/// is nothing to wait for, and a future that never resolves is the honest way
/// to say so — `select!` then simply has one fewer way to finish.
async fn next_fatal(proxy: Option<&mut service_auth::k8s::LoopbackProxy>) -> String {
    match proxy {
        Some(proxy) => match proxy.next_fatal().await {
            Some(error) => error.to_string(),
            None => std::future::pending().await,
        },
        None => std::future::pending().await,
    }
}

/// Mint a token for `client_sa` and stand up the loopback proxy that lends it
/// to the wrapped command (#2878, R1/R2/R4).
///
/// The first mint happens here rather than lazily on the first proxied
/// request, so `lumen connect --client-sa` fails immediately and legibly when
/// the caller lacks the grant.
#[cfg(feature = "delegated-auth")]
async fn start_client_proxy(
    args: &ConnectArgs,
    client_sa: &str,
    upstream: &str,
    trust: Option<ServingTrust>,
) -> Result<service_auth::k8s::LoopbackProxy> {
    let tokens = client_token_source(args.context.as_deref(), &args.namespace, client_sa).await?;
    first_mint(&tokens, &args.namespace, client_sa).await?;
    match trust {
        Some(client) => {
            service_auth::k8s::LoopbackProxy::start_with_client(upstream, tokens, client).await
        }
        None => service_auth::k8s::LoopbackProxy::start(upstream, tokens).await,
    }
    .context("start the local authenticated proxy")
}

/// The verifying client, where there is one to have.
///
/// `--ca-file` requires `--client-sa`, and `--client-sa` requires the
/// `delegated-auth` feature, so a build without it cannot reach a state where
/// this holds anything — which is why the placeholder is a unit rather than a
/// second implementation to keep in step.
#[cfg(feature = "delegated-auth")]
type ServingTrust = reqwest::Client;
#[cfg(not(feature = "delegated-auth"))]
type ServingTrust = ();

/// The client the proxy forwards through when the fleet serves TLS (#3113 R6).
///
/// Two things are deliberately absent. There is no option to skip verification:
/// the failure this command is most likely to hit is a *correct* refusal, and a
/// flag that turned it off would turn the private trust domain into decoration.
/// And the public root store is not merely augmented but switched off — with it
/// on, any public CA could still vouch for this name.
#[cfg(feature = "delegated-auth")]
fn serving_trust(
    ca_file: &std::path::Path,
    server_name: &str,
    local_port: u16,
) -> Result<ServingTrust> {
    let pem = std::fs::read_to_string(ca_file).with_context(|| {
        format!(
            "read the serving trust bundle {}. Obtain the public CA separately \
             from the deployment administrator or external certificate platform",
            ca_file.display()
        )
    })?;
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], local_port));
    service_auth::k8s::verifying_client(&pem, server_name, addr).with_context(|| {
        format!(
            "build a client that verifies {server_name} against {}",
            ca_file.display()
        )
    })
}

/// Reach the far end once, and turn whatever went wrong into something the
/// caller can act on (#3113 R7).
///
/// Each branch names a different deployment fact, because they have different
/// fixes and a single "handshake failed" would send a caller looking in the
/// wrong place. None of them offers to stop verifying, and none of them
/// suggests a certificate for `localhost` — that leaf would be valid against
/// every port-forward anyone opens, which is the opposite of what naming the
/// Service buys.
#[cfg(feature = "delegated-auth")]
async fn probe_serving_tls(
    client: &ServingTrust,
    server_name: &str,
    ca_file: Option<&std::path::Path>,
) -> Result<()> {
    let ca = ca_file
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "<none>".to_string());
    let error = match client
        .get(format!("https://{server_name}/healthz"))
        .timeout(Duration::from_secs(15))
        .send()
        .await
    {
        // Any HTTP answer means the handshake completed, which is all this
        // probe is about. `/healthz` needs no credential, so a status other
        // than 200 is the fleet's business and not this command's.
        Ok(_) => return Ok(()),
        Err(error) => error,
    };

    // The useful text is in the source chain — rustls' rejection reason is
    // several layers below reqwest's "error sending request".
    let mut detail = error.to_string();
    let mut source: Option<&(dyn std::error::Error + 'static)> = std::error::Error::source(&error);
    while let Some(inner) = source {
        detail = format!("{detail}: {inner}");
        source = inner.source();
    }
    let lowered = detail.to_ascii_lowercase();

    let diagnosis = if lowered.contains("unknownissuer") || lowered.contains("unknown issuer") {
        format!(
            "the certificate {server_name} presented was not signed by anything in {ca}. Either \
             the bundle is from another cluster or CA pool, or the fleet's leaf was issued \
             outside it — obtain the public CA separately from the deployment administrator"
        )
    } else if lowered.contains("notvalidforname") || lowered.contains("not valid for name") {
        format!(
            "the far end holds a certificate that does not name {server_name}. Pass \
             `--server-name` with one of the names it does hold (the operator requests \
             `<service>.<namespace>.svc` and its cluster FQDN), or check that `--service`/`--cr` \
             names the Service you meant"
        )
    } else if lowered.contains("expired") {
        format!(
            "the certificate {server_name} presented has expired. Ask the deployment administrator \
             or external certificate platform to rotate the externally provisioned serving Secret \
             and distribute its public CA; keep verification enabled and do not downgrade to plaintext"
        )
    } else if lowered.contains("corrupt message")
        || lowered.contains("handshake eof")
        || lowered.contains("unexpected eof")
        || lowered.contains("http instead of https")
    {
        // The far end read a ClientHello, made nothing of it, and hung up.
        // That is overwhelmingly one thing: a port serving cleartext.
        format!(
            "svc/{server_name} did not complete a TLS handshake and closed the connection, which \
             is what a port still answering in cleartext does. Use `--plaintext` if that is a \
             development instance; in production, have the deployment administrator or external \
             certificate platform provision the serving TLS Secret and set `spec.servingTlsSecret`"
        )
    } else {
        format!("could not complete a TLS handshake with {server_name}: {detail}")
    };

    anyhow::bail!(
        "{diagnosis}.\n\
         The port-forward is only transport: the socket is on 127.0.0.1, but the identity being \
         verified is the Kubernetes Service's, and that is the one the certificate names. There \
         is no option to skip that check."
    )
}

/// Without `delegated-auth` there is no token to carry and so no proxy to
/// verify through. Refusing here keeps the flag from looking like it worked.
#[cfg(not(feature = "delegated-auth"))]
fn serving_trust(
    _ca_file: &std::path::Path,
    _server_name: &str,
    _local_port: u16,
) -> Result<ServingTrust> {
    anyhow::bail!(
        "--ca-file needs the `delegated-auth` feature (rebuild with \
         `cargo build -p lumen --features delegated-auth`); this binary has no client that can \
         verify a private serving certificate"
    )
}

/// Unreachable: the only producer of a [`ServingTrust`] in this build fails.
#[cfg(not(feature = "delegated-auth"))]
async fn probe_serving_tls(
    _client: &ServingTrust,
    _server_name: &str,
    _ca_file: Option<&std::path::Path>,
) -> Result<()> {
    Ok(())
}

/// The first mint, with Lumen's own remediation attached (#2878, R6).
///
/// `service-auth` already names the caller, the ServiceAccount, and the
/// `kubectl auth can-i` question — everything a provider-neutral library can
/// know. What it cannot know is that this repository ships a command that
/// writes the missing grant, so the CLI adds that here rather than teaching the
/// library about Lumen.
#[cfg(feature = "delegated-auth")]
async fn first_mint(
    tokens: &service_auth::k8s::TokenSource,
    namespace: &str,
    client_sa: &str,
) -> Result<service_auth::k8s::ProjectedToken> {
    match tokens.token().await {
        Ok(token) => Ok(token),
        Err(error @ service_auth::k8s::TokenRequestError::Forbidden { .. }) => {
            anyhow::bail!(
                "{error}. `lumen k8s access render --namespace {namespace} --client-sa \
                 {client_sa} --issuer <you>` emits exactly that grant"
            )
        }
        Err(other) => Err(other.into()),
    }
}

/// Same signature, no minter: a build without `delegated-auth` has no
/// TokenRequest client linked in, and pretending otherwise would mean either
/// running unauthenticated behind the caller's back or inventing a credential.
/// Naming the missing feature is the only honest answer.
#[cfg(not(feature = "delegated-auth"))]
async fn start_client_proxy(
    _args: &ConnectArgs,
    _client_sa: &str,
    _upstream: &str,
    _trust: Option<ServingTrust>,
) -> Result<service_auth::k8s::LoopbackProxy> {
    anyhow::bail!(
        "--client-sa needs the `delegated-auth` feature (rebuild with \
         `cargo build -p lumen --features delegated-auth`); this binary cannot mint a \
         ServiceAccount token"
    )
}

/// One short-lived audience-bound token, minted for an explicitly named
/// ServiceAccount through whatever identity the kubeconfig already holds
/// (#2878, R1/R3).
///
/// The audience is `lumen`'s own, not a parameter: a token a serving node will
/// accept is exactly a token minted for that audience, and letting a flag
/// choose otherwise would only produce credentials that fail at the far end.
#[cfg(feature = "delegated-auth")]
async fn client_token_source(
    context: Option<&str>,
    namespace: &str,
    client_sa: &str,
) -> Result<std::sync::Arc<service_auth::k8s::TokenSource>> {
    let target =
        service_auth::k8s::TokenRequestTarget::new(namespace, client_sa, lumen::auth::AUDIENCE)?;
    let minter = service_auth::k8s::KubeTokenMinter::from_context(context).await?;
    Ok(std::sync::Arc::new(service_auth::k8s::TokenSource::new(
        std::sync::Arc::new(minter),
        target,
    )))
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
        offset: 0,
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

/// The `token` parameter is the whole of `lumen query`'s credential handling
/// (#2878). It is a value passed in, minted moments earlier for this one
/// command: there is no environment variable, no file, and no Secret lookup
/// behind it. #2873 removed all three, and the parameter is deliberately
/// explicit rather than ambient so that a future change which starts sending a
/// credential from somewhere else has to say so in this signature.
#[cfg(feature = "backup")]
async fn http_post_json(
    base_url: &str,
    path: &str,
    body: serde_json::Value,
    token: Option<&service_auth::k8s::ProjectedToken>,
) -> Result<serde_json::Value> {
    let url = format!("{}{path}", base_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let mut req = client.post(&url).json(&body);
    if let Some(token) = token {
        req = req.bearer_auth(token.expose());
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
    path: &str,
    token: Option<&service_auth::k8s::ProjectedToken>,
) -> Result<serde_json::Value> {
    let url = format!("{}{path}", base_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let mut req = client.get(&url);
    if let Some(token) = token {
        req = req.bearer_auth(token.expose());
    }
    let resp = req.send().await.with_context(|| format!("GET {url}"))?;
    let status = resp.status();
    let payload: serde_json::Value = resp.json().await.unwrap_or(serde_json::Value::Null);
    if !status.is_success() {
        anyhow::bail!("GET {url} returned {status}: {payload}");
    }
    Ok(payload)
}

/// Mint the token `lumen query` will carry, if the caller named an account to
/// mint it for (#2878, R3/R4).
///
/// The token is returned by value and lives only as long as the command that
/// asked for it. It is never written anywhere: not to a cache file, not to an
/// environment variable, not to stdout.
#[cfg(all(feature = "backup", feature = "delegated-auth"))]
async fn query_token(target: &QueryTarget) -> Result<Option<service_auth::k8s::ProjectedToken>> {
    let Some(client_sa) = target.client_sa.as_deref() else {
        return Ok(None);
    };
    // clap's `requires = "namespace"` already guarantees this pair arrives
    // together; the message is for the code path, not for the user.
    let namespace = target
        .namespace
        .as_deref()
        .context("--client-sa requires --namespace")?;
    let tokens = client_token_source(target.context.as_deref(), namespace, client_sa).await?;
    Ok(Some(first_mint(&tokens, namespace, client_sa).await?))
}

/// Without `delegated-auth` there is no TokenRequest client to mint with, so
/// `--client-sa` is refused rather than silently ignored. Sending an
/// unauthenticated request in response to an explicit request to authenticate
/// is the one behaviour that must not happen.
#[cfg(all(feature = "backup", not(feature = "delegated-auth")))]
async fn query_token(target: &QueryTarget) -> Result<Option<service_auth::k8s::ProjectedToken>> {
    if target.client_sa.is_some() {
        anyhow::bail!(
            "--client-sa needs the `delegated-auth` feature (rebuild with \
             `cargo build -p lumen --features delegated-auth`); this binary cannot mint a \
             ServiceAccount token"
        );
    }
    Ok(None)
}

/// `lumen query` dispatch (#1321, R3): resolves `--url` via `QueryTarget`,
/// assembles the exact wire body, and POSTs/GETs it. No REPL, no new HTTP
/// endpoint.
///
/// Credential resolution is a single line and it is not a lookup (#2878): with
/// `--client-sa`, a token is minted for the named account, carried in memory
/// for this one request, and dropped with the command. Without it the request
/// goes out as whoever the network says it is and a serving instance under
/// `auth: required` answers 401 — the honest failure (AC4). What #2873 removed
/// and did not come back is the silent path: quietly reaching into a Secret
/// for a shared token nobody named.
#[cfg(feature = "backup")]
async fn dispatch_query(args: QueryArgs) -> Result<()> {
    match args.command {
        QueryCommand::Index(args) => {
            let base = resolve_base_url(&args.target)?;
            let token = query_token(&args.target).await?;
            let (path, body) = build_index_body(&args.collection, &args.items)?;
            let resp = http_post_json(&base, &path, body, token.as_ref()).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
            Ok(())
        }
        QueryCommand::Search(args) => {
            let base = resolve_base_url(&args.target)?;
            let token = query_token(&args.target).await?;
            let (path, body) = build_search_body(&args)?;
            let resp = http_post_json(&base, &path, body, token.as_ref()).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
            Ok(())
        }
        QueryCommand::Duplicates(args) => {
            let base = resolve_base_url(&args.target)?;
            let token = query_token(&args.target).await?;
            let (path, body) = build_duplicates_body(&args)?;
            let resp = http_post_json(&base, &path, body, token.as_ref()).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
            Ok(())
        }
        QueryCommand::Collections(args) => match args.command {
            QueryCollectionsCommand::List(args) => {
                let base = resolve_base_url(&args.target)?;
                let token = query_token(&args.target).await?;
                let resp = http_get_json(&base, "/collections", token.as_ref()).await?;
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
    cli_std::artifact::strip_source_ownership_markers(include_str!("../../Dockerfile"))
}

fn render_release_dockerfile(version: Option<&str>) -> String {
    let tag = cli_std::artifact::release_tag("lumen", version, env!("CARGO_PKG_VERSION"));
    let version = tag.trim_start_matches("lumen@");
    let template =
        cli_std::artifact::strip_source_ownership_markers(include_str!("../../Dockerfile.release"));
    let mut out = String::new();
    for line in template.lines() {
        if line.starts_with("#   docker build -f apps/lumen/Dockerfile.release -t lumen:") {
            out.push_str(&format!(
                "#   docker build -f apps/lumen/Dockerfile.release -t lumen:{version} \\"
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

fn render_operator_yaml(args: &K8sOperatorRenderArgs) -> Result<String> {
    let namespace = &args.namespace;
    let image = &args.image;
    let monitoring = args.monitoring;
    if image.is_empty()
        || image.starts_with('-')
        || image
            .chars()
            .any(|character| character.is_whitespace() || character.is_control())
    {
        anyhow::bail!(
            "operator image must be a non-empty, whitespace-free OCI image reference that does not start with '-'"
        );
    }
    let mut out = String::new();
    out.push_str(&cli_std::artifact::replace_kubernetes_namespace(
        &cli_std::artifact::strip_source_ownership_markers(include_str!(
            "../../k8s/operator/rbac.yaml"
        )),
        "lumen-system",
        namespace,
    ));
    out.push_str("\n---\n");
    let mut deployment = cli_std::artifact::replace_kubernetes_namespace(
        &cli_std::artifact::strip_source_ownership_markers(include_str!(
            "../../k8s/operator/deployment.yaml"
        )),
        "lumen-system",
        namespace,
    );

    // Derived, not hardcoded (#2532): the checked-in manifest pins this
    // workspace's own version, so a release bump that misses `deployment.yaml`
    // fails this render instead of silently handing out a stale image.
    let checked_in_image = format!(
        "          image: ghcr.io/chrischeng-c4/lumen:{}",
        env!("CARGO_PKG_VERSION")
    );
    if !deployment.contains(&checked_in_image) {
        anyhow::bail!(
            "checked-in operator manifest does not pin this build's image \
             (`{checked_in_image}`) — bump k8s/operator/deployment.yaml with the release"
        );
    }
    out.push_str(&deployment.replacen(&checked_in_image, &format!("          image: {image}"), 1));
    // The operator's own scrape target (#2621). Unconditional: it is plain
    // core/v1, so it applies on a cluster with no monitoring stack at all, and
    // shipping it always means turning monitoring on later never has to come
    // back and add the target. Kept in the same order as
    // k8s/operator/kustomization.yaml so the two paths stay comparable.
    out.push_str("\n---\n");
    out.push_str(&cli_std::artifact::replace_kubernetes_namespace(
        &cli_std::artifact::strip_source_ownership_markers(include_str!(
            "../../k8s/operator/service.yaml"
        )),
        "lumen-system",
        namespace,
    ));
    // The PDB ships with the Deployment: `replicas: 2` only survives a node
    // drain if evictions are serialized (#2602). Render consumers get the same
    // operator layer the kustomize consumers get.
    out.push_str("\n---\n");
    out.push_str(&cli_std::artifact::replace_kubernetes_namespace(
        &cli_std::artifact::strip_source_ownership_markers(include_str!(
            "../../k8s/operator/pdb.yaml"
        )),
        "lumen-system",
        namespace,
    ));
    // Opt-in tail, byte-identical to the `operator-monitoring` component so a
    // kustomize consumer and a render consumer get the same alerts. Gated
    // because these two are monitoring.coreos.com CRDs: emitting them
    // unconditionally would make `kubectl apply` of the whole render fail on
    // any cluster without prometheus-operator, taking the operator down with
    // the alerts.
    if monitoring {
        for manifest in [
            include_str!("../../k8s/components/operator-monitoring/servicemonitor.yaml"),
            include_str!("../../k8s/components/operator-monitoring/prometheusrule.yaml"),
        ] {
            out.push_str("\n---\n");
            out.push_str(&rewrite_monitoring_namespace(
                &cli_std::artifact::strip_source_ownership_markers(manifest),
                namespace,
            ));
        }
    }
    Ok(cli_std::artifact::ensure_trailing_newline(&out))
}

/// Rewrite the control-plane namespace in the two monitoring manifests.
///
/// `cli_std::artifact::replace_kubernetes_namespace` rewrites the `name:` and
/// `namespace:` keys, which is everything the RBAC/Deployment/Service/PDB
/// layer carries. The monitoring layer carries the namespace in three further
/// shapes, and every one of them fails *silently* if left behind on a
/// `--namespace` render:
///
/// - the ServiceMonitor's `namespaceSelector.matchNames` list item — a
///   selector pointed at an empty namespace discovers no target, so the
///   operator simply never appears in Prometheus;
/// - the PromQL `namespace="..."` matchers in both alert expressions — an
///   expression that matches nothing can never fire, which is exactly the
///   false green row 4 exists to prevent;
/// - the `-n <ns>` in the runbook annotations — commands an on-call would
///   paste against the wrong namespace mid-incident.
fn rewrite_monitoring_namespace(manifest: &str, namespace: &str) -> String {
    cli_std::artifact::replace_kubernetes_namespace(manifest, "lumen-system", namespace)
        .replace("- lumen-system", &format!("- {namespace}"))
        .replace(
            "namespace=\"lumen-system\"",
            &format!("namespace=\"{namespace}\""),
        )
        .replace("-n lumen-system", &format!("-n {namespace}"))
}

/// Standalone custom resource manifests for `--profile <dev|staging|prod|template>`.
///
/// Shared by `lumen k8s instance render` and by tests asserting the four profile shapes.
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
            // Published releases live at ghcr.io/chrischeng-c4/lumen:<version>
            // (digest in each release's notes); this is the handed-out default.
            format!("ghcr.io/chrischeng-c4/lumen:{default_version}"),
            InstanceBody::Staging,
        ),
        K8sInstanceProfile::Prod => (
            "lumen",
            "production",
            // Same published GHCR default as staging; override with `--image`
            // to pin @sha256 or point at a mirrored registry.
            format!("ghcr.io/chrischeng-c4/lumen:{default_version}"),
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

    let header_comment = "# TLS Secrets are provisioned by the deployment administrator or an external platform.\n# The operator consumes named serving/peer Secrets and performs no issuance.\n";

    let yaml = format!(
        "{header_comment}apiVersion: lumen.dev/v1alpha1\nkind: Lumen\nmetadata:\n  name: {name}\n  namespace: {namespace}\nspec:\n{}",
        profile_spec_body(body, image)
    );
    cli_std::artifact::ensure_trailing_newline(&yaml)
}

/// The `spec:` body for one profile, at two-space indent. Shared by
/// `k8s instance render` and `k8s fleet render` so a fleet's `defaults` and a
/// standalone CR cannot drift into disagreeing about what "prod" means.
fn profile_spec_body(body: InstanceBody, image: &str) -> String {
    let mut yaml = format!("  image: {image}\n");
    match body {
        InstanceBody::Dev => {
            // Every profile states its auth posture out loud, even when it
            // agrees with the CRD default (#2678). `auth` fails closed, so a
            // rendered CR that stayed silent would be a `required` instance
            // with no token source: a pod that never passes readiness.
            yaml.push_str("  shardCount: 1\n  replicasPerShard: 1\n  voterCount: 1\n  logFormat: pretty\n  auth: disabled\n  placement:\n    nodeSelector:\n      kubernetes.io/os: linux\n  serving:\n    cpu: \"1\"\n    memory: 4Gi\n");
        }
        InstanceBody::Staging => {
            // #3113 R8: `servingTlsSecret` is stated for the same reason as
            // `peerTlsSecret` below, but the failure it prevents is the
            // opposite one. An unstated peer Secret fails closed — the pods
            // refuse to start and say so. An unstated serving Secret fails
            // *open*: the client port quietly stays h2c, and a fleet serves
            // KSA-bearing requests in cleartext while looking healthy.
            yaml.push_str("  shardCount: 3\n  replicasPerShard: 3\n  voterCount: 3\n  logFormat: json\n  auth: required\n  peerTlsSecret: lumen-peer-tls\n  servingTlsSecret: lumen-serving-tls\n  serving:\n    cpu: \"1\"\n    memory: 4Gi\n  observability: true\n");
        }
        InstanceBody::Prod => {
            // #2890 R7: `peerTlsSecret` is stated, not defaulted. A replicated
            // profile that stayed silent about it would render a CR whose pods
            // refuse to start — the same reasoning as `auth` above, one port
            // over.
            yaml.push_str("  imagePullPolicy: Always\n  shardCount: 6\n  replicasPerShard: 3\n  voterCount: 3\n  logFormat: json\n  logLevel: warn\n  auth: required\n  peerTlsSecret: lumen-peer-tls\n  servingTlsSecret: lumen-serving-tls\n  serving:\n    cpu: \"1\"\n    memory: 4Gi\n    graceSecs: 45\n  observability: true\n");
        }
        InstanceBody::Template => {
            yaml.push_str("  imagePullPolicy: IfNotPresent\n  shardCount: REPLACE_ME__SHARD_COUNT\n  replicasPerShard: REPLACE_ME__REPLICAS_PER_SHARD\n  voterCount: REPLACE_ME__VOTER_COUNT\n  logFormat: json\n  auth: required\n  peerTlsSecret: REPLACE_ME__PEER_TLS_SECRET\n  servingTlsSecret: REPLACE_ME__SERVING_TLS_SECRET\n  serving:\n    cpu: \"1\"\n    memory: 4Gi\n");
        }
    }
    yaml
}

#[derive(Clone, Copy)]
enum InstanceBody {
    Dev,
    Staging,
    Prod,
    Template,
}

/// `spec.defaults` for `k8s fleet render --profile template`, at four-space
/// indent.
///
/// Names no token source, because there is no longer one to name: a caller's
/// identity comes from the cluster's TokenRequest/TokenReview path, not from
/// anything a fleet or its instances configure (#2872). A template that still
/// carried a credential field would hand every app team a spec the API server
/// now rejects outright.
const FLEET_TEMPLATE_DEFAULTS: &str = "\
    \x20   image: __IMAGE__\n\
    \x20   imagePullPolicy: IfNotPresent\n\
    \x20   shardCount: REPLACE_ME__SHARD_COUNT\n\
    \x20   replicasPerShard: REPLACE_ME__REPLICAS_PER_SHARD\n\
    \x20   voterCount: REPLACE_ME__VOTER_COUNT\n\
    \x20   logFormat: json\n\
    \x20   auth: required\n\
    \x20   # Serving leaf every data plane presents on :7373, one Secret name\n\
    \x20   # per tenant namespace. Stated rather than left out: an unset\n\
    \x20   # serving Secret is not an error, it is cleartext (#3113).\n\
    \x20   servingTlsSecret: REPLACE_ME__SERVING_TLS_SECRET\n\
    \x20   # Workload Identity KSA every data plane runs as.\n\
    \x20   serviceAccountName: REPLACE_ME__WORKLOAD_IDENTITY_KSA\n\
    \x20   placement:\n\
    \x20     # Which node pool every data plane lands on.\n\
    \x20     nodeSelector:\n\
    \x20       REPLACE_ME__NODE_POOL_LABEL: REPLACE_ME__NODE_POOL_VALUE\n\
    \x20     # Uncomment when the pool is tainted to repel other workloads.\n\
    \x20     # tolerations:\n\
    \x20     #   - key: REPLACE_ME__TAINT_KEY\n\
    \x20     #     operator: Equal\n\
    \x20     #     value: REPLACE_ME__TAINT_VALUE\n\
    \x20     #     effect: NoSchedule\n\
    \x20   serving:\n\
    \x20     cpu: \"1\"\n\
    \x20     memory: 4Gi\n\
    \x20     # SSD vs standard disk is the StorageClass; omit to take the\n\
    \x20     # cluster default.\n\
    \x20     raftStorageClass: REPLACE_ME__STORAGE_CLASS\n";

/// Render a `LumenFleet` — the single cluster-scoped object the platform team
/// applies into the control-plane namespace to declare every data plane.
///
/// The split the document is built to teach: `defaults` holds what the
/// platform owns and every tenant shares (image, node pool, StorageClass,
/// ServiceAccount); each `instances[].spec` holds what one app team owns (its
/// CPU/memory request — which is what makes replica-shard autoscaling
/// trigger — its disk size, and its credential source).
fn render_fleet_yaml(args: &K8sFleetRenderArgs) -> String {
    let default_version = env!("CARGO_PKG_VERSION");
    let (default_name, default_image, body) = match args.profile {
        K8sFleetProfile::Dev => ("search", "lumen:latest".to_string(), InstanceBody::Dev),
        K8sFleetProfile::Prod => (
            "lumen",
            format!("ghcr.io/chrischeng-c4/lumen:{default_version}"),
            InstanceBody::Prod,
        ),
        K8sFleetProfile::Template => (
            "REPLACE_ME__FLEET_NAME",
            "REPLACE_ME__REGISTRY/lumen:REPLACE_ME__IMAGE_TAG".to_string(),
            InstanceBody::Template,
        ),
    };
    let name = args.name.as_deref().unwrap_or(default_name);
    let image = args.image.as_deref().unwrap_or(&default_image);

    // `defaults` is a whole LumenSpec. dev/prod reuse the instance profile
    // bodies verbatim, one level deeper, so a fleet's "prod" and a standalone
    // "prod" CR cannot come to disagree. `template` gets its own body: every
    // value there is a REPLACE_ME with no semantics to keep in step, and it
    // has to name knobs (node pool, StorageClass, ServiceAccount) that only
    // make sense on the fleet — appending them to the shared body would emit
    // `serving:` twice and silently drop the first one.
    let defaults: String = match args.profile {
        K8sFleetProfile::Template => FLEET_TEMPLATE_DEFAULTS.replace("__IMAGE__", image),
        _ => profile_spec_body(body, image)
            .lines()
            .map(|line| format!("  {line}\n"))
            .collect(),
    };

    let mut yaml = format!(
        "# One object, applied once by the platform team. Every data-plane\n\
         # namespace this cluster serves is declared below; the operator\n\
         # materializes one `Lumen` per entry into the namespace named.\n\
         #\n\
         # The namespaces must already exist — the fleet never creates them.\n\
         apiVersion: lumen.dev/v1alpha1\n\
         kind: LumenFleet\n\
         metadata:\n  name: {name}\n\
         spec:\n\
         \x20 # Platform-owned: what every tenant shares.\n\
         \x20 defaults:\n{defaults}"
    );

    match args.profile {
        K8sFleetProfile::Dev => {
            yaml.push_str(
                "  instances:\n\
                 \x20   - namespace: default\n",
            );
        }
        K8sFleetProfile::Prod => {
            yaml.push_str(
                "    placement:\n\
                 \x20     nodeSelector:\n\
                 \x20       cloud.google.com/gke-nodepool: lumen\n\
                 \x20 # App-team-owned: what one tenant sets for itself. Each\n\
                 \x20 # `spec` is a merge patch over `defaults` above — name only\n\
                 \x20 # what differs; everything unnamed is inherited.\n\
                 \x20 instances:\n\
                 \x20   - namespace: team-a\n\
                 \x20     spec:\n\
                 \x20       serving:\n\
                 \x20         cpu: \"4\"\n\
                 \x20         memory: 16Gi\n\
                 \x20         raftStorage: 200Gi\n\
                 \x20   - namespace: team-b\n\
                 \x20     spec:\n\
                 \x20       serving:\n\
                 \x20         cpu: \"1\"\n\
                 \x20         memory: 4Gi\n",
            );
        }
        K8sFleetProfile::Template => {
            yaml.push_str(
                "  # App-team-owned: what one tenant sets for itself. Each\n\
                 \x20 # `spec` is a merge patch over `defaults` above — name only\n\
                 \x20 # what differs; everything unnamed is inherited. A `null`\n\
                 \x20 # value removes an inherited field.\n\
                 \x20 instances:\n\
                 \x20   - namespace: REPLACE_ME__APP_NAMESPACE\n\
                 \x20     spec:\n\
                 \x20       serving:\n\
                 \x20         # Requests, not just limits: replica-shard\n\
                 \x20         # autoscaling triggers off these.\n\
                 \x20         cpu: REPLACE_ME__CPU\n\
                 \x20         memory: REPLACE_ME__MEMORY\n\
                 \x20         raftStorage: REPLACE_ME__DISK\n\
                 \x20 # Retain (default) leaves an instance running when its entry\n\
                 \x20 # is removed; Delete removes it and its PVCs.\n\
                 \x20 prunePolicy: Retain\n",
            );
        }
    }
    cli_std::artifact::ensure_trailing_newline(&yaml)
}

/// `lumen k8s access render` (#2889) — the client access handoff, as five
/// objects and no credential.
///
/// The boundary this renders has two hops, and the reason to render it rather
/// than describe it in a runbook is that the two are easy to collapse into
/// one:
///
/// 1. a human account or a Google service account authenticates to
///    *kube-apiserver* through its kubeconfig credential plugin, and
///    Kubernetes RBAC decides whether that principal may create a TokenRequest
///    for one named ServiceAccount;
/// 2. the short-lived, audience-bound token that comes back is the only
///    credential Lumen ever sees, and Lumen asks Kubernetes RBAC what *that
///    ServiceAccount* may do.
///
/// Binding the Google principal straight to the Lumen role authorizes the same
/// human and looks like it worked — right until the first request, which
/// arrives carrying a ServiceAccount token nobody granted anything to. The two
/// RoleBindings here take deliberately different subject kinds so that
/// shortcut is not expressible.
///
/// `serviceaccounts/token` is the namespace's privilege-escalation surface:
/// `create` on it without `resourceNames` mints a token for *every*
/// ServiceAccount in the namespace, the operator's included. So the issuer
/// Role always names its ServiceAccount, and the bundle is scanned with
/// [`service_k8s::render::rbac::first_wildcard`] before it is emitted — a
/// wildcard that reached any field of any object is a bug in this function,
/// and the render fails rather than hands one out.
#[cfg(feature = "operator")]
fn render_access_yaml(args: &K8sAccessRenderArgs) -> Result<String> {
    use service_k8s::render::rbac::{
        first_wildcard, role, role_binding, NamedRule, Role, RoleBinding, RoleSubject,
        ServiceAccountSubject,
    };

    let namespace = object_name("--namespace", &args.namespace)?;
    let client = object_name("--client-sa", &args.client_sa)?;
    let issuers = parse_issuers(&args.issuers)?;
    let grants = parse_grants(&args.grants)?;
    if grants.is_empty() && !args.instance_admin {
        anyhow::bail!(
            "nothing would be granted: pass at least one `--grant \
             <collection-id>=read|write|admin` or `--instance-admin`"
        );
    }

    let issuer_role_name = format!("{client}-token-issuer");
    let lumen_role_name = format!("{client}-lumen-access");
    let labels = serde_json::json!({
        "app.kubernetes.io/name": "lumen",
        "app.kubernetes.io/instance": client,
        "app.kubernetes.io/component": "access",
        "app.kubernetes.io/managed-by": "lumen-cli",
        "app.kubernetes.io/part-of": "lumen",
    });

    // Hop 1: who may mint this ServiceAccount's token.
    let issuer_rules = [NamedRule {
        api_groups: &[""],
        resources: &["serviceaccounts/token"],
        resource_names: &[client],
        verbs: &["create"],
    }];
    let issuer_subjects: Vec<RoleSubject<'_>> =
        issuers.iter().copied().map(RoleSubject::User).collect();

    // Hop 2: what that ServiceAccount may do, in Lumen's own vocabulary. The
    // resource and verb names come from `lumen::auth`, which is what the
    // serving side puts in its SubjectAccessReview — one definition, so a
    // rendered grant cannot describe a check Lumen does not make.
    let api_groups = [lumen::auth::API_GROUP];
    let collections = [lumen::auth::COLLECTIONS_RESOURCE];
    let admin = [lumen::auth::ADMIN_RESOURCE];
    let admin_verbs = [lumen::auth::verb(service_auth::role_map::Role::Admin)];
    let grant_names: Vec<[&str; 1]> = grants
        .iter()
        .map(|grant| [grant.collection.as_str()])
        .collect();
    let mut lumen_rules: Vec<NamedRule<'_>> = grants
        .iter()
        .zip(&grant_names)
        .map(|(grant, names)| NamedRule {
            api_groups: &api_groups,
            resources: &collections,
            resource_names: names,
            verbs: &grant.verbs,
        })
        .collect();
    if args.instance_admin {
        lumen_rules.push(NamedRule {
            api_groups: &api_groups,
            resources: &admin,
            // The admin surface is one namespace-wide object, not a set of
            // named ones — `AuthTarget::Admin` sends no resource name — so
            // there is nothing to enumerate here.
            resource_names: &[],
            // And it is checked at exactly one role: every `ensure_admin` call
            // site asks for `Role::Admin`. Granting the lower verbs too would
            // widen the grant past anything Lumen can ask for.
            verbs: &admin_verbs,
        });
    }

    let documents = vec![
        serde_json::json!({
            "apiVersion": "v1",
            "kind": "ServiceAccount",
            "metadata": { "name": client, "namespace": namespace, "labels": labels },
        }),
        role(Role {
            name: &issuer_role_name,
            namespace,
            labels: labels.clone(),
            rules: &issuer_rules,
        }),
        role_binding(RoleBinding {
            name: &issuer_role_name,
            namespace,
            labels: labels.clone(),
            role: &issuer_role_name,
            subjects: &issuer_subjects,
        }),
        role(Role {
            name: &lumen_role_name,
            namespace,
            labels: labels.clone(),
            rules: &lumen_rules,
        }),
        role_binding(RoleBinding {
            name: &lumen_role_name,
            namespace,
            labels,
            role: &lumen_role_name,
            subjects: &[RoleSubject::ServiceAccount(ServiceAccountSubject {
                namespace,
                name: client,
            })],
        }),
    ];

    let mut out = String::from(ACCESS_BUNDLE_HEADER);
    for (index, document) in documents.iter().enumerate() {
        if let Some(field) = first_wildcard(document) {
            anyhow::bail!(
                "refusing to render a wildcard RBAC grant: `{}` in the {} object",
                field,
                document["kind"].as_str().unwrap_or("rendered")
            );
        }
        if index > 0 {
            out.push_str("---\n");
        }
        out.push_str(&serde_yaml::to_string(document).context("render access bundle YAML")?);
    }
    Ok(cli_std::artifact::ensure_trailing_newline(&out))
}

/// Leads the rendered bundle so the object list is readable without the
/// issue that produced it. Comments, not a wrapper: the body stays raw
/// multi-document YAML that `kubectl apply -f -` accepts unchanged.
#[cfg(feature = "operator")]
const ACCESS_BUNDLE_HEADER: &str = "\
# Lumen client access (#2889). Two hops, five objects, no credential.
#
#   1. `<client-sa>-token-issuer` lets the named Kubernetes users create a
#      TokenRequest for exactly one ServiceAccount. Those users authenticate
#      to kube-apiserver, never to Lumen.
#   2. `<client-sa>-lumen-access` is what Lumen's SubjectAccessReview reads
#      once that ServiceAccount's token arrives.
#
# Mint a token with:
#   kubectl create token <client-sa> -n <namespace> \\
#     --audience lumen.axiom.dev --duration 10m
";

#[cfg(not(feature = "operator"))]
fn render_access_yaml(_args: &K8sAccessRenderArgs) -> Result<String> {
    anyhow::bail!(
        "this lumen build was compiled without operator support; rebuild with \
         `--features operator` (the published binary and image include it)"
    )
}

/// A Kubernetes object name (RFC 1123 label), checked here rather than left to
/// `kubectl apply`. A rejected name is a CLI error; an accepted-but-wrong one
/// becomes a `resourceNames` entry that matches nothing, which RBAC reports as
/// an ordinary denial with no hint that the grant was misspelled.
#[cfg(feature = "operator")]
fn object_name<'a>(flag: &str, value: &'a str) -> Result<&'a str> {
    let shaped = !value.is_empty()
        && value.len() <= 63
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !value.starts_with('-')
        && !value.ends_with('-');
    if !shaped {
        anyhow::bail!(
            "{flag} must be a DNS-1123 label — 1-63 lowercase letters, digits, \
             or `-`, not starting or ending with `-` — got `{value}`"
        );
    }
    Ok(value)
}

/// One collection's grant: the id RBAC will match on, and the verbs it earns.
#[cfg(feature = "operator")]
struct CollectionGrant {
    collection: String,
    verbs: Vec<&'static str>,
}

/// Parse `--grant <collection-id>=read|write|admin`, rejecting duplicates.
///
/// Two `--grant` flags for one collection would render two rules that RBAC
/// unions, so the narrower one is silently irrelevant — exactly the case where
/// a deployer believes they tightened a grant and did not.
#[cfg(feature = "operator")]
fn parse_grants(specs: &[String]) -> Result<Vec<CollectionGrant>> {
    let mut grants: Vec<CollectionGrant> = Vec::new();
    for spec in specs {
        let (collection, level) = spec.split_once('=').ok_or_else(|| {
            anyhow::anyhow!("--grant expects `<collection-id>=read|write|admin`, got `{spec}`")
        })?;
        if collection.is_empty()
            || collection.len() > 253
            || collection
                .chars()
                .any(|c| c.is_whitespace() || c.is_control() || c == '*')
        {
            anyhow::bail!(
                "--grant collection id must be 1-253 characters with no whitespace \
                 and no `*` — got `{collection}` in `{spec}`"
            );
        }
        let level = match level {
            "read" => service_auth::role_map::Role::Read,
            "write" => service_auth::role_map::Role::Write,
            "admin" => service_auth::role_map::Role::Admin,
            other => anyhow::bail!(
                "--grant level must be `read`, `write`, or `admin` — got `{other}` in `{spec}`"
            ),
        };
        if grants.iter().any(|grant| grant.collection == collection) {
            anyhow::bail!(
                "--grant names `{collection}` twice; RBAC unions the rules, so the \
                 narrower grant would have no effect"
            );
        }
        grants.push(CollectionGrant {
            collection: collection.to_string(),
            verbs: granted_verbs(level),
        });
    }
    Ok(grants)
}

/// Every verb Lumen can ask for at or below `level`.
///
/// The role-to-verb mapping is one-to-one (`read` -> `get`, `write` ->
/// `update`, `admin` -> `delete`) and lives in `lumen::auth`; the *grant* is
/// cumulative because `Role::covers` is — a writer that could not read the
/// collection it writes would be denied by the same check that lets it write.
#[cfg(feature = "operator")]
fn granted_verbs(level: service_auth::role_map::Role) -> Vec<&'static str> {
    use service_auth::role_map::Role;
    [Role::Read, Role::Write, Role::Admin]
        .into_iter()
        .filter(|needed| level.covers(*needed))
        .map(lumen::auth::verb)
        .collect()
}

/// Validate `--issuer` names without interpreting them.
///
/// A Kubernetes username is whatever the API server's authenticator produced:
/// a Google address, an OIDC subject, a certificate CN. Parsing it here would
/// invent a distinction the authorizer does not make, so the only rejections
/// are the ones that would produce a binding matching the wrong set of people
/// — an empty name, a wildcard, stray control characters, or surrounding
/// whitespace that YAML would keep and the API server would not.
#[cfg(feature = "operator")]
fn parse_issuers(issuers: &[String]) -> Result<Vec<&str>> {
    let mut names: Vec<&str> = Vec::new();
    for issuer in issuers {
        if issuer.is_empty()
            || issuer.contains('*')
            || issuer.chars().any(char::is_control)
            || issuer.trim() != issuer
        {
            anyhow::bail!(
                "--issuer must be the username `kubectl auth whoami` prints, with no \
                 `*`, no control characters, and no surrounding whitespace — got `{issuer}`"
            );
        }
        if names.contains(&issuer.as_str()) {
            anyhow::bail!("--issuer names `{issuer}` twice");
        }
        names.push(issuer);
    }
    Ok(names)
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
    if let Some(target) = cli_std::artifact::write_or_print(out, default_file, body)? {
        println!("next: {}", next(&target));
    }
    Ok(())
}

/// `next:` builder shared by every k8s render verb: the rendered manifest's
/// only sensible follow-up is applying it.
fn kubectl_apply_next(target: &Path) -> String {
    format!("kubectl apply -f {}", target.display())
}

/// Real [`lumen::api::CheckpointSink`] wiring for segment-persistence mode
/// (#1389): forces the same synchronous stage-then-rename checkpoint the
/// periodic snapshotter performs (`SegmentRdbStore::save`), but synchronously
/// on demand — this is what `POST /admin/checkpoint` answers, and what the
/// reshard driver's cutover gate (`service_k8s::reshard_driver::
/// checkpoint_touched_shards`) awaits per touched shard before triggering the
/// cutover rolling restart. Also prunes + trims the AOF through the
/// checkpointed sequence, mirroring the periodic path exactly, so an
/// on-demand checkpoint leaves the AOF in the same state a periodic one
/// would (and a reshard cutover right after one doesn't leave a redundant,
/// ever-growing AOF tail).
struct SegmentCheckpointSink {
    engine: Arc<Engine>,
    store: Arc<lumen::segment_rdb::SegmentRdbStore>,
    writer: Arc<dyn lumen::coordinator::WriteSink>,
    aof: Option<lumen::coordinator::SharedAof>,
}

fn segment_restore_sink(
    segment_mode: bool,
    backend: WalBackend,
    engine: Arc<Engine>,
    store: Option<Arc<lumen::segment_rdb::SegmentRdbStore>>,
    writer: Arc<dyn lumen::coordinator::WriteSink>,
    aof: Option<lumen::coordinator::SharedAof>,
) -> Result<Option<Arc<dyn lumen::api::RestoreSink>>> {
    if !segment_mode {
        return Ok(None);
    }

    const UNAVAILABLE_REASON: &str =
        "durable segment restore requires wal=embedded, a configured data directory, and the local AOF";
    if backend != WalBackend::Embedded {
        return Ok(Some(Arc::new(
            lumen::segment_restore::UnavailableRestoreSink::new(UNAVAILABLE_REASON),
        )));
    }

    match (store, aof) {
        (Some(store), Some(aof)) => Ok(Some(Arc::new(
            lumen::segment_restore::SegmentRestoreSink::new(engine, store, writer, aof)?,
        ))),
        _ => Ok(Some(Arc::new(
            lumen::segment_restore::UnavailableRestoreSink::new(UNAVAILABLE_REASON),
        ))),
    }
}

#[async_trait::async_trait]
impl lumen::api::CheckpointSink for SegmentCheckpointSink {
    async fn checkpoint_now(&self) -> Result<bool> {
        // The sink owns the checkpoint permit. Do not acquire this in the API
        // handler: an exclusive restore may already be queued ahead of this
        // request, and a nested read would then deadlock.
        let _checkpoint_permit = if let Some(gate) = self.writer.mutation_gate() {
            Some(gate.shared().await?)
        } else {
            None
        };
        let seq = self.writer.applied_seq();
        let store = self.store.clone();
        let engine = self.engine.clone();
        tokio::task::spawn_blocking(move || -> Result<()> {
            store.save(&engine, seq)?;
            store.prune(3)?;
            Ok(())
        })
        .await
        .context("checkpoint task panicked")??;

        if let Some(aof) = &self.aof {
            let aof = aof.clone();
            let trim = tokio::task::spawn_blocking(move || {
                aof.lock()
                    .map_err(|_| anyhow::anyhow!("aof writer poisoned"))?
                    .truncate_through(seq)
            })
            .await;
            match trim {
                Ok(Ok(())) => tracing::info!(through = seq, "AOF trimmed to on-demand checkpoint"),
                Ok(Err(e)) => {
                    tracing::warn!(error = %e, "AOF trim after on-demand checkpoint failed")
                }
                Err(e) => tracing::warn!(error = %e, "AOF trim task panicked"),
            }
        }
        tracing::info!(
            up_to_seq = seq,
            "on-demand checkpoint written (admin request)"
        );
        Ok(true)
    }
}

async fn serve(args: ServeArgs) -> Result<()> {
    init_tracing(
        &args.log_level,
        args.log_format,
        args.otlp_endpoint.as_deref(),
    )?;

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
    let mut raft_host: Option<Arc<raft_runtime::RaftHost>> = None;
    #[cfg(feature = "raft-wal")]
    let mut raft_peer_transport: Option<raft_runtime::PeerTransport> = None;
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
            // Constructed below (#1486), once the final restore watermark
            // (`start_seq`, after any checkpoint + AOF-tail replay) is
            // known — an embedded `MemWal` must start its sequence domain
            // above that watermark, not at 0, or the apply loop's
            // redelivery-dedup guard silently strands the first N
            // post-restart writes.
            None
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
            // Peers are always addressed on the dedicated authenticated Raft
            // port over `https`.
            let headless = std::env::var("LUMEN_HEADLESS_SERVICE")
                .unwrap_or_else(|_| "lumen-headless".to_string());
            let peer_transport = lumen::tls::PeerTlsConfig::from_env()
                .context("raft: load peer TLS configuration")?
                .map(|config| config.peer_transport())
                .transpose()
                .context("raft: build shared peer mTLS transport")?;
            // #2890 R3/R4: no plaintext fallback. This used to route peer RPCs
            // at the *client* port over h2c whenever TLS material was absent —
            // a replicated group replicating committed index mutations between
            // pods with nothing on the wire saying who either end is, reachable
            // by anything that can open a TCP connection to the Service. The
            // failure it replaced (a pod that will not start) is loud, local,
            // and names the field to set; the one it created was silent.
            let Some(peer_transport) = peer_transport else {
                anyhow::bail!(
                    "raft: replicated mode needs peer mTLS material — set \
                     LUMEN_PEER_TLS_CERT / LUMEN_PEER_TLS_KEY / LUMEN_PEER_TLS_CA \
                     (+ LUMEN_PEER_MTLS=on), or under Kubernetes name a Secret with \
                     tls.crt/tls.key/ca.crt in the Lumen CR's spec.peerTlsSecret. \
                     Raft peer traffic has no plaintext path"
                );
            };
            let topo = raft_runtime::ClusterTopology::from_env_with_scheme(
                "lumen",
                &headless,
                args.raft_port,
                "LUMEN_PEERS",
                "https",
            )
            .context("raft: cluster topology from env")?;
            tracing::info!(
                node_id = topo.node_id,
                voters = ?topo.membership.voters,
                peers = ?topo.peers.keys().collect::<Vec<_>>(),
                data_dir = %args.raft_data_dir,
                "wal=raft (raft_core; multi-pod)"
            );
            let store = raft_runtime::RaftStore::open(
                &args.raft_data_dir,
                topo.node_id,
                raft_runtime::FsyncPolicy::Always,
            )
            .context("open raft store")?;
            // The host is the sole applier: committed entries fold straight into
            // the engine (via `EngineSm`), so there is no `WalLog`/coordinator
            // seam for the raft path. Cold-start (restore + replay) happens in
            // `RaftHost::spawn`; snapshot/compaction is driven externally below.
            let sm = lumen::raft_sm::EngineSm::new(engine.clone(), 0);
            let host_config = raft_runtime::HostConfig {
                snapshot: raft_runtime::SnapshotPolicy::External,
                ..Default::default()
            };
            let host = Arc::new(raft_runtime::RaftHost::spawn_with_peer_transport(
                topo.node_id,
                topo.membership,
                topo.peers,
                store,
                sm.clone() as Arc<dyn raft_runtime::RaftStateMachine>,
                host_config,
                peer_transport.clone(),
            ));
            raft_peer_transport = Some(peer_transport);

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
                engine.clone(),
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

    // Embedded backend: build the `MemWal` now that `start_seq` reflects the
    // final restore watermark (checkpoint restore, then AOF-tail replay if
    // any — whichever is higher). Every other backend either owns its own
    // sequence domain externally (NATS) or bypasses `wal` entirely (raft)
    // (#1486).
    let wal: Option<SharedWal> = match backend {
        WalBackend::Embedded => Some(Arc::new(MemWal::starting_at(start_seq))),
        _ => wal,
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

    // `LUMEN_AUTH=required|in-cluster` delegates both halves of the decision to the
    // apiserver: TokenReview says who is calling, SubjectAccessReview says
    // whether they may (#2869). Building the verifier proves both grants before
    // the listener opens, so a missing `system:auth-delegator` binding is a
    // startup failure rather than a fleet that serves 503s while looking ready.
    //
    // The transport lives behind the `delegated-auth` feature. A build without
    // it can still parse either required mode — and must refuse to start
    // rather than quietly serve unauthenticated traffic under that setting.
    #[cfg(feature = "delegated-auth")]
    async fn serving_verifier(
        auth: &AuthConfig,
    ) -> anyhow::Result<Arc<lumen::auth::LumenVerifier>> {
        Ok(Arc::new(lumen::auth::LumenVerifier::connect(auth).await?))
    }
    #[cfg(not(feature = "delegated-auth"))]
    async fn serving_verifier(
        auth: &AuthConfig,
    ) -> anyhow::Result<Arc<lumen::auth::LumenVerifier>> {
        anyhow::bail!(
            "LUMEN_AUTH={} needs the Kubernetes TokenReview/SubjectAccessReview transport, \
             which this binary was built without. Rebuild with `--features delegated-auth`, or \
             unset LUMEN_AUTH to serve without authentication. Refusing to start.",
            auth.profile().env_value()
        )
    }

    let auth = Arc::new(AuthConfig::from_env()?);
    let verifier = if auth.required {
        let verifier = serving_verifier(&auth).await?;
        match auth.profile() {
            AuthProfile::ManagedAudience => tracing::info!(
                namespace = %auth.namespace,
                audience = lumen::auth::AUDIENCE,
                "auth=required — every request is authenticated by Kubernetes TokenReview and \
                 authorized by SubjectAccessReview; only audience-bound ServiceAccount tokens are \
                 accepted"
            ),
            AuthProfile::KubernetesDefault => tracing::info!(
                namespace = %auth.namespace,
                "auth=in-cluster — every request is authenticated by Kubernetes TokenReview \
                 against the apiserver's configured audiences and authorized by \
                 SubjectAccessReview; only Kubernetes ServiceAccount identities are accepted"
            ),
            AuthProfile::Off => unreachable!("required auth cannot use the off profile"),
        }
        Some(verifier)
    } else {
        tracing::warn!(
            "auth=off — requests are not authenticated. Set LUMEN_AUTH=required for Managed or \
             LUMEN_AUTH=in-cluster for Standalone (plus LUMEN_AUTH_NAMESPACE when needed) to \
             delegate to Kubernetes"
        );
        None
    };
    let admission = service_http::AdmissionConfig::from_env("LUMEN")?.controller(
        "lumen.read",
        "lumen.write",
        "lumen.admin",
    );
    if admission.is_some() {
        tracing::info!("request admission enabled (LUMEN_ADMISSION_*; probes stay exempt)");
    }

    let mut state = lumen::api::AppState::with_components(engine.clone(), auth, writer.clone());
    if let Some(verifier) = verifier {
        state = state.with_verifier(verifier);
    }
    if let Some(restore_sink) = segment_restore_sink(
        segment_mode,
        backend,
        engine.clone(),
        segment_store.clone(),
        writer.clone(),
        aof_writer.clone(),
    )? {
        state = state.with_restore_sink(restore_sink);
    }
    // #1389: wire a real on-demand checkpoint (`POST /admin/checkpoint`) only
    // when segment persistence is actually configured — the raft path has
    // its own snapshot mechanism and is out of the reshard driver's scope
    // (single-member only), and the default CBOR/no-data-dir path stays
    // `NoopCheckpoint` (nothing durable to force). Cloned from `segment_store`
    // before the periodic-snapshotter block below consumes it.
    if let Some(store) = segment_store.clone() {
        state = state.with_checkpoint(Arc::new(SegmentCheckpointSink {
            engine: engine.clone(),
            store,
            writer: writer.clone(),
            aof: aof_writer.clone(),
        }));
    }
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
        //
        // #1398 R4: the physical shard count that map is built for defaults
        // to the number of loaded dirs (matching `EngineShardSearch::new`'s
        // original behavior) unless `--shard-count`/`SHARD_COUNT` was
        // explicitly set — see `fan_in_shard_count`'s doc comment for why
        // `args.shard_count` has to be `Option<u32>` to make that
        // distinction. A mismatch between the resolved count and the
        // actual loaded-dir count fails startup instead of silently
        // under-routing.
        let fan_in_shard_count = lumen::config::fan_in_shard_count(args.shard_count, shards.len());
        let shard_map = lumen::config::shard_map_from_env(fan_in_shard_count).context(
            "shard map from env (SHARD_MAP_VERSION/SHARD_MAP_ASSIGNMENTS/VIRTUAL_BUCKET_COUNT)",
        )?;
        lumen::config::check_fan_in_shard_count(&shard_map, shards.len())?;
        tracing::info!(
            shard_count = shards.len(),
            shard_map_version = shard_map.version(),
            "search backend=segment-sharded"
        );
        state = state.with_search_backend(Arc::new(
            lumen::routing::EngineShardSearch::new_with_shard_map(shards, shard_map),
        ));
    }
    // #1398 R1: activate cross-pod routing only in the operator/k8s routed
    // serving topology (`SHARD_COUNT` env > 1 at `replicasPerShard <= 1`) —
    // `routed_activation_shard_count` returns `None` for every other
    // deployment shape, so `shardCount:1` serving never even constructs a
    // `RoutedRouter` (AC5). It also folds in the fan-in mutual-exclusion
    // guard (#1442 R3): the fan-in path above already built its own local
    // `EngineShardSearch`/`state.search_backend` when
    // `args.search_shard_segment_dirs` is non-empty, so a fan-in invocation
    // must never also reach the routed block — pulled into
    // `config::routed_activation_shard_count` so that guarantee is
    // unit-tested directly instead of only by inline control flow here.
    #[cfg(feature = "operator")]
    if let Some(shard_count) =
        lumen::config::routed_activation_shard_count(args.search_shard_segment_dirs.is_empty())
            .context("routed shard count from env (SHARD_COUNT)")?
    {
        let shard_map = lumen::config::shard_map_from_env(shard_count).context(
            "shard map from env (SHARD_MAP_VERSION/SHARD_MAP_ASSIGNMENTS/VIRTUAL_BUCKET_COUNT)",
        )?;
        let (prefix, local_shard) = lumen::config::routed_pod_topology(shard_count)
            .context("routed pod topology from env (POD_NAME)")?;
        let headless = std::env::var("LUMEN_HEADLESS_SERVICE")
            .unwrap_or_else(|_| "lumen-headless".to_string());
        let shard_urls: Vec<String> = (0..shard_count)
            .map(|shard| {
                format!(
                    "http://{}:{}",
                    lumen::routing::shard_host(&prefix, shard, &headless),
                    args.port
                )
            })
            .collect();
        tracing::info!(
            shard_count,
            local_shard,
            shard_map_version = shard_map.version(),
            "cross-pod shard routing active"
        );
        // #1467 R5: publish this pod's live shard-map version on `/metrics`
        // so the reshard driver's `advance_convergence` can require every
        // serving pod to actually report the new map, not just that its
        // StatefulSet rollout finished.
        engine.metrics().set_shard_map_version(shard_map.version());
        let router = lumen::routing_remote::RoutedRouter::new(
            engine.clone(),
            state.write_backend.clone(),
            shard_map,
            local_shard,
            shard_urls,
        )
        .context("construct routed shard router")?;
        state = state.with_routed(Arc::new(router));
    }
    #[cfg_attr(not(feature = "raft-wal"), allow(unused_mut))]
    let mut app = lumen::api::router_with_admission(state, admission);
    // Plain local/backward-compatible Raft RPCs share the public h2c port.
    // Configured mTLS peers are served only by the dedicated listener below.
    #[cfg(feature = "raft-wal")]
    if raft_peer_transport.is_none() {
        if let Some(host) = &raft_host {
            app = app.merge(host.router());
        }
    }

    // Periodic snapshotter. Raft mode: the host captures the engine RDB AND
    // compacts the raft log (bounding it + arming InstallSnapshot for a fresh
    // replica) — the shared backup layer (#524, closes #522 by construction).
    // Otherwise the RDB snapshotter writes the `--data-dir` checkpoints the apply
    // loop tails from on restart.
    #[cfg(feature = "raft-wal")]
    if let Some(host) = raft_host.clone() {
        let period = Duration::from_secs(args.snapshot_secs.max(1));
        let snap_engine = engine.clone();
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
                    Err(e) => {
                        // #2516: the raft snapshot is itself a durable
                        // checkpoint write — same ENOSPC treatment as the
                        // non-raft RDB/segment snapshotters below.
                        if lumen::coordinator::is_storage_full(&e) {
                            tracing::error!(error = %e, "raft snapshot/compact hit ENOSPC — entering degraded read-only mode");
                            snap_engine.metrics().mark_storage_degraded();
                        } else {
                            tracing::warn!(error = %e, "raft snapshot/compact failed");
                        }
                    }
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
                let _checkpoint_permit = match snap_writer.mutation_gate() {
                    Some(gate) => match gate.shared().await {
                        Ok(permit) => Some(permit),
                        Err(e) => {
                            tracing::error!(error = %e, "RDB checkpoint blocked by durability state");
                            continue;
                        }
                    },
                    None => None,
                };
                let seq = snap_writer.applied_seq();
                match RdbSnapshot::capture(&snap_engine, seq) {
                    Ok(rdb) => {
                        if let Err(e) = store.save(&rdb).await {
                            // #2516: a checkpoint write is a durable write
                            // path too — ENOSPC here must enter the same
                            // sticky degraded read-only mode as an AOF
                            // ENOSPC, not just a one-off warn.
                            if lumen::coordinator::is_storage_full(&e) {
                                tracing::error!(error = %e, "RDB snapshot save hit ENOSPC — entering degraded read-only mode");
                                snap_engine.metrics().mark_storage_degraded();
                            } else {
                                tracing::warn!(error = %e, "RDB snapshot save failed");
                            }
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
                let _checkpoint_permit = match snap_writer.mutation_gate() {
                    Some(gate) => match gate.shared().await {
                        Ok(permit) => Some(permit),
                        Err(e) => {
                            tracing::error!(error = %e, "segment checkpoint blocked by durability state");
                            continue;
                        }
                    },
                    None => None,
                };
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
                    Ok(Err(e)) => {
                        // #2516: same treatment as the RDB path above — a
                        // segment checkpoint ENOSPC is a durable write path
                        // failure and must flip the sticky degraded flag.
                        if lumen::coordinator::is_storage_full(&e) {
                            tracing::error!(error = %e, "segment checkpoint save hit ENOSPC — entering degraded read-only mode");
                            snap_engine.metrics().mark_storage_degraded();
                        } else {
                            tracing::warn!(error = %e, "segment checkpoint save failed");
                        }
                    }
                    Err(e) => tracing::warn!(error = %e, "segment checkpoint task panicked"),
                }
            }
        });
    }

    // #2516: periodic ENOSPC re-probe. While this node is in degraded
    // read-only mode (`Metrics::storage_degraded`), attempt a small write
    // into `--data-dir` every `LUMEN_STORAGE_FULL_REPROBE_SECS` (default 30s)
    // and clear the sticky flag once one succeeds — the automatic recovery
    // path for a PVC that was resized or freed up without a pod restart.
    // (Operators can also just restart the pod: the flag is process-local
    // and starts clear on a fresh process.) Only probes while degraded, so a
    // healthy node pays nothing extra.
    if let Some(dir) = args.data_dir.clone() {
        let probe_engine = engine.clone();
        let reprobe_secs: u64 = std::env::var("LUMEN_STORAGE_FULL_REPROBE_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .filter(|&v| v > 0)
            .unwrap_or(30);
        let probe_path = std::path::Path::new(&dir).join(".storage_full_probe");
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(reprobe_secs));
            ticker.tick().await; // skip immediate fire
            loop {
                ticker.tick().await;
                if !probe_engine.metrics().is_storage_degraded() {
                    continue;
                }
                match tokio::fs::write(&probe_path, b"ok").await {
                    Ok(()) => {
                        probe_engine.metrics().clear_storage_degraded();
                        tracing::info!(
                            "storage re-probe write succeeded; leaving degraded read-only mode"
                        );
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "storage re-probe still failing");
                    }
                }
            }
        });
    }

    // #3113 R1: the client port's own identity, loaded before the socket binds
    // so a pod with unusable material fails startup instead of accepting
    // connections it cannot serve. `None` is the h2c posture — local and kind
    // development, and the only path to cleartext on this port.
    let serving_tls = lumen::tls::ServingTlsConfig::from_env()
        .context("load serving TLS configuration")?
        .map(|config| {
            let names = config.dns_names.clone();
            config.reloadable().map(|tls| (tls, names))
        })
        .transpose()
        .context("activate the serving certificate")?;

    let bind = format!("{}:{}", args.host, args.port);
    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .with_context(|| format!("bind {bind}"))?;
    match &serving_tls {
        Some((tls, names)) => tracing::info!(
            addr = %bind,
            shard_count = args.shard_count.unwrap_or(1),
            tls_generation = tls.generation(),
            server_names = %names.join(","),
            "lumen serve listening over TLS"
        ),
        None => tracing::info!(
            addr = %bind,
            shard_count = args.shard_count.unwrap_or(1),
            "lumen serve listening"
        ),
    }

    #[cfg(feature = "raft-wal")]
    let peer_server = if let (Some(host), Some(transport)) =
        (raft_host.as_ref(), raft_peer_transport.as_ref())
    {
        let peer_bind = format!("{}:{}", args.host, args.raft_port);
        let peer_listener = tokio::net::TcpListener::bind(&peer_bind)
            .await
            .with_context(|| format!("bind authenticated raft peer listener {peer_bind}"))?;
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let peer_router = host.router();
        let transport = transport.clone();
        tracing::info!(addr = %peer_bind, tls_generation = transport.generation(), "lumen raft peer mTLS listening");
        let task = tokio::spawn(async move {
            transport
                .serve(peer_listener, peer_router, async {
                    let _ = shutdown_rx.await;
                })
                .await
        });
        Some((shutdown_tx, task))
    } else {
        None
    };

    let grace = Duration::from_secs(args.grace_secs);
    let shutdown_engine = engine.clone();
    let shutdown_aof = aof_writer.clone();
    // Serve HTTP/1.1 + h2c on one port through the shared service HTTP shell,
    // with the standard SIGTERM drain sequence flipping `/readyz` to 503
    // before the listener closes. The single-replica segment AOF is synced at
    // the *start* of termination, not after the full drain window: the pod's
    // termination grace can equal that window, so a post-drain sync may lose
    // the race with Kubernetes' SIGKILL.
    let shutdown = service_http::shutdown_with_drain(
        move || {
            shutdown_engine.start_drain();
            if let Some(aof) = shutdown_aof {
                match aof.lock() {
                    Ok(mut writer) => {
                        if let Err(e) = writer.sync() {
                            tracing::warn!(error = %e, "sync local AOF during shutdown failed");
                        }
                    }
                    Err(_) => tracing::warn!("local AOF writer poisoned during shutdown"),
                }
            }
        },
        grace,
    );
    match serving_tls {
        // #3113 R1/R9: the configuration is read per accepted connection, so a
        // renewed leaf reaches connection N+1 with no rebind and no restart.
        // `None` from the source refuses the connection — there is deliberately
        // no branch here that answers it in cleartext instead.
        Some((tls, _)) => {
            service_http::serve_tls(
                listener,
                app,
                service_http::config_source(move || tls.server_config()),
                shutdown,
            )
            .await;
        }
        None => service_http::serve(listener, app, shutdown).await,
    }
    #[cfg(feature = "raft-wal")]
    if let Some((shutdown_tx, task)) = peer_server {
        let _ = shutdown_tx.send(());
        task.await.context("raft peer listener task panicked")??;
    }
    // Flush any batched spans before exit (no-op when OTLP was never enabled).
    #[cfg(feature = "otel")]
    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}

/// Keeps `AppState.cluster` (#1310's read-consistency enforcement seam)
/// current for the process lifetime (#1349): polls the already-running
/// `RaftHost` for its live role/leader view (`is_leader`/`leader`, both
/// pre-existing — no new raft-runtime surface added) and republishes it onto
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
    host: Arc<raft_runtime::RaftHost>,
    cluster: Arc<lumen::raft::ClusterState>,
    is_voter: bool,
    engine: Arc<Engine>,
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
            // #2475: publish `lumen_raft_leader_known{shard}` off the same
            // live election read `enforce_read_consistency` trusts, so
            // `render::prometheus_rule`'s `LumenRaftLeaderAbsent` alerts on
            // a real signal rather than a synthesized one.
            engine
                .metrics()
                .set_raft_leader_known(cluster.shard_index, leader.is_some());
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

fn init_tracing(level: &str, format: LogFormat, otlp_endpoint: Option<&str>) -> Result<()> {
    let format = match format {
        LogFormat::Pretty => service_http::LogFormat::Pretty,
        LogFormat::Json => service_http::LogFormat::Json,
    };
    let config = service_http::HttpConfig::new(
        "127.0.0.1",
        0,
        format!("info,lumen={level}"),
        format,
        0,
        0,
        otlp_endpoint.map(str::to_owned),
    );
    let identity = service_http::ServiceIdentity::new("lumen", env!("CARGO_PKG_VERSION"))?;
    service_http::init_tracing_with_identity(&config, &identity)
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
    use lumen::coordinator::MutationGate;
    use lumen::types::{
        CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
        QueryNode, SearchRequest, TermQuery,
    };
    use std::collections::BTreeMap;

    fn test_schema() -> CreateCollectionRequest {
        serde_json::from_value(serde_json::json!({
            "fields": { "value": { "type": "keyword" } }
        }))
        .expect("valid test schema")
    }

    fn test_writer(
        engine: Arc<Engine>,
    ) -> (
        Arc<dyn lumen::coordinator::WriteSink>,
        lumen::coordinator::SharedAof,
        tempfile::TempDir,
    ) {
        let dir = tempfile::tempdir().expect("AOF tempdir");
        let aof = Arc::new(std::sync::Mutex::new(
            lumen::aof::AofWriter::open(dir.path().join("aof.log")).expect("open AOF"),
        ));
        let writer = lumen::coordinator::WriteCoordinator::start_from_with_aof(
            Arc::new(MemWal::new()),
            engine,
            0,
            aof.clone(),
        );
        (writer, aof, dir)
    }

    #[tokio::test]
    async fn segment_restore_sink_is_not_selected_for_non_segment_mode() {
        let engine = Arc::new(Engine::new());
        let (writer, aof, _dir) = test_writer(engine.clone());
        assert!(
            segment_restore_sink(false, WalBackend::Embedded, engine, None, writer, Some(aof))
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn segment_restore_sink_replaces_current_and_live_state() {
        let live = Arc::new(Engine::new());
        live.create_collection("old", test_schema()).unwrap();
        let source = Engine::new();
        source.create_collection("new", test_schema()).unwrap();
        let snapshot = source.snapshot().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(lumen::segment_rdb::SegmentRdbStore::new(dir.path()).unwrap());
        let aof = Arc::new(std::sync::Mutex::new(
            lumen::aof::AofWriter::open(dir.path().join("aof.log")).unwrap(),
        ));
        let writer = lumen::coordinator::WriteCoordinator::start_from_with_aof(
            Arc::new(MemWal::new()),
            live.clone(),
            0,
            aof.clone(),
        );
        let sink = segment_restore_sink(
            true,
            WalBackend::Embedded,
            live.clone(),
            Some(store.clone()),
            writer,
            Some(aof),
        )
        .unwrap()
        .expect("embedded segment sink");

        sink.restore(snapshot).await.unwrap();
        assert_eq!(live.list_collections().unwrap(), vec!["new".to_string()]);
        let loaded = store.load_current_generation().unwrap().unwrap();
        assert_eq!(
            loaded.engine.list_collections().unwrap(),
            vec!["new".to_string()]
        );
    }

    #[tokio::test]
    async fn segment_nats_restore_is_unavailable_over_real_router() {
        use axum::body::Body;
        use axum::http::{Method, Request, StatusCode};
        use tower::ServiceExt;

        let engine = Arc::new(Engine::new());
        let (writer, aof, _dir) = test_writer(engine.clone());
        let restore_sink = segment_restore_sink(
            true,
            WalBackend::Nats,
            engine.clone(),
            None,
            writer.clone(),
            Some(aof),
        )
        .unwrap()
        .expect("fail-closed segment sink");
        let state = lumen::api::AppState::with_components(
            engine,
            Arc::new(lumen::auth::AuthConfig::open()),
            writer,
        )
        .with_restore_sink(restore_sink);
        let response = lumen::api::router(state)
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/admin/restore")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({"version": 1, "collections": {}}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let envelope: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(envelope["error"], "restore_unavailable");
    }

    #[tokio::test]
    async fn segment_restore_sink_is_unavailable_without_store_or_aof() {
        use axum::body::Body;
        use axum::http::{Method, Request, StatusCode};
        use tower::ServiceExt;

        let engine = Arc::new(Engine::new());
        let writer = lumen::coordinator::WriteCoordinator::start_from(
            Arc::new(MemWal::new()),
            engine.clone(),
            0,
        );
        let restore_sink = segment_restore_sink(
            true,
            WalBackend::Embedded,
            engine.clone(),
            None,
            writer.clone(),
            None,
        )
        .unwrap()
        .expect("missing durable resources must install a fail-closed sink");
        let state = lumen::api::AppState::with_components(
            engine,
            Arc::new(lumen::auth::AuthConfig::open()),
            writer,
        )
        .with_restore_sink(restore_sink);
        let response = lumen::api::router(state)
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/admin/restore")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({"version": 1, "collections": {}}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn checkpoint_shared_permit_blocks_exclusive_restore_until_checkpoint_finishes() {
        let gate = MutationGate::default();
        let checkpoint = gate.shared().await.unwrap();
        let mut restore = Box::pin(gate.exclusive());
        assert!(
            tokio::time::timeout(Duration::from_millis(25), &mut restore)
                .await
                .is_err()
        );
        drop(checkpoint);
        tokio::time::timeout(Duration::from_secs(1), restore)
            .await
            .expect("exclusive restore must proceed after checkpoint permit is released")
            .unwrap();
    }

    #[tokio::test]
    async fn manual_checkpoint_can_join_before_queued_exclusive_restore_without_deadlock() {
        let gate = MutationGate::default();
        let checkpoint = gate.shared().await.unwrap();
        let mut restore = Box::pin(gate.exclusive());
        tokio::task::yield_now().await;
        assert!(
            tokio::time::timeout(Duration::from_millis(25), &mut restore)
                .await
                .is_err()
        );
        let mut next_checkpoint = Box::pin(gate.shared());
        assert!(
            tokio::time::timeout(Duration::from_millis(25), &mut next_checkpoint)
                .await
                .is_err()
        );
        drop(checkpoint);
        // Once the active checkpoint releases, the fair gate lets the queued
        // exclusive restore run. A handler must not hold a second read guard.
        let _restore = tokio::time::timeout(Duration::from_secs(1), restore)
            .await
            .expect("queued restore must not deadlock")
            .unwrap();
        drop(_restore);
        tokio::time::timeout(Duration::from_secs(1), next_checkpoint)
            .await
            .expect("checkpoint must finish after restore")
            .unwrap();
    }

    /// Every handed-out `LumenFleet` must actually materialize. A rendered
    /// template is the first thing a deployer applies, so a duplicated
    /// `serving:` key (last wins, the CPU/memory request silently gone) or a
    /// `defaults`/`instances` pair that merges into two conflicting shard
    /// topologies would ship as a cluster that comes up wrong — or not at all —
    /// with the
    /// mistake in our YAML rather than theirs.
    #[cfg(feature = "operator")]
    #[test]
    fn every_rendered_fleet_profile_materializes_its_instances() {
        use lumen::operator::fleet::{plan, PlanOutcome};

        for profile in [
            K8sFleetProfile::Dev,
            K8sFleetProfile::Prod,
            K8sFleetProfile::Template,
        ] {
            let yaml = render_fleet_yaml(&K8sFleetRenderArgs {
                profile,
                name: None,
                image: None,
                out: None,
            });
            // The template is meant to be edited before it applies, so its
            // required-decision placeholders are filled in here rather than
            // pretending an unedited skeleton is deployable. Everything else
            // about it — key structure, the defaults/instances merge, the
            // per-instance topology pairing — is exactly what ships.
            let yaml = yaml
                .replace("REPLACE_ME__SHARD_COUNT", "1")
                .replace("REPLACE_ME__REPLICAS_PER_SHARD", "1")
                .replace("REPLACE_ME__VOTER_COUNT", "1");
            // Parsing is itself the duplicate-key check: serde_yaml rejects a
            // mapping that names one key twice, which is how a second
            // `serving:` block silently eating the CPU/memory request gets
            // caught rather than shipped.
            let fleet: lumen::operator::LumenFleet = serde_yaml::from_str(&yaml)
                .unwrap_or_else(|err| panic!("profile does not parse: {err}\n{yaml}"));

            let planned = plan(&fleet);
            assert!(
                !planned.is_empty(),
                "a fleet that declares no data plane is not a usable starting point\n{yaml}"
            );
            for instance in &planned {
                if let PlanOutcome::Rejected(reason) = &instance.outcome {
                    panic!(
                        "namespace {} would be rejected: {reason}\n{yaml}",
                        instance.namespace
                    );
                }
            }
        }
    }

    /// The template exists to name the knobs a deployer owns; a knob that
    /// silently drops out of it is a knob nobody knows to set.
    #[test]
    fn the_fleet_template_names_every_deployer_owned_knob() {
        let yaml = render_fleet_yaml(&K8sFleetRenderArgs {
            profile: K8sFleetProfile::Template,
            name: None,
            image: None,
            out: None,
        });
        for knob in [
            "nodeSelector",       // which node pool
            "raftStorageClass",   // SSD vs standard disk
            "serviceAccountName", // the KSA the data plane runs as
            "cpu:",               // request — what triggers shard autoscaling
            "memory:",
            "raftStorage:", // per-tenant disk size
            "prunePolicy",
        ] {
            assert!(yaml.contains(knob), "template lost `{knob}`\n{yaml}");
        }
    }

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
                    offset: 0,
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
            context: None,
            namespace: None,
            client_sa: None,
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

    // `wait_for_local_port_ready`/`ChildGuard` unit tests moved to
    // `libs/cli-std/src/connect.rs` (#1376) along with the primitives
    // themselves; lumen's own coverage is the thin-adapter tests above
    // (`resolve_base_url_requires_explicit_url`, `build_*_body_*`) plus
    // `cargo test -p cli-std --features k8s`. The credential half of that
    // shared module (`select_token`, `cr_tokens_secret`, `secret_data_bytes`)
    // keeps its own tests there and is simply no longer called from here
    // (#2873) — the integration gate for that is
    // `e2e/cli_credential_paths_retired.rs`.

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
        let host = Arc::new(raft_runtime::RaftHost::spawn(
            0,
            raft_runtime::Membership {
                voters: vec![0],
                learners: vec![],
            },
            std::collections::HashMap::new(),
            raft_runtime::RaftStore::open(tmp.to_str().unwrap(), 0, raft_runtime::FsyncPolicy::Os)
                .unwrap(),
            sm.clone() as Arc<dyn raft_runtime::RaftStateMachine>,
            raft_runtime::HostConfig::default(),
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

        // #2475: a fresh `Engine`'s `lumen_raft_leader_known` starts
        // unpublished (sentinel `raft_shard`, see `metrics.rs`) until this
        // poller ticks; asserted below alongside role convergence.
        let poller_engine = Arc::new(Engine::new());
        assert!(
            !poller_engine
                .metrics()
                .render()
                .contains("lumen_raft_leader_known"),
            "raft leader metric must be absent before the poller's first tick"
        );

        spawn_cluster_state_poller(host, cluster.clone(), true, poller_engine.clone());

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

        // #2475: the poller's real election read also publishes
        // `lumen_raft_leader_known{shard="0"} 1` on `/metrics` (a
        // single-voter host always wins its own election) — the metric
        // `render::prometheus_rule`'s `LumenRaftLeaderAbsent` alert reads.
        let metrics_out = poller_engine.metrics().render();
        assert!(
            metrics_out.contains("lumen_raft_leader_known{shard=\"0\"} 1"),
            "expected lumen_raft_leader_known in:\n{metrics_out}"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
// CODEGEN-END
