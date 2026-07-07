// HANDWRITE-BEGIN gap="missing-generator:logic:1eefd229" tracker="pending-tracker" reason="Single relay CLI bin (clap): bare relay (no subcommand) runs the h2c server with ServeArgs flags falling back to RELAY_BIND/RELAY_DATA_DIR env (the relay_server.rs behavior verbatim); Command::Llm/Upgrade/Issue dispatch to cli_std::{llm,upgrade,issue} with relay's ToolInfo; mirrors projects/keep/src/bin/keep.rs."
//! relay — cloud-native work-queue broker (HTTP/2 + OpenAPI).
//!
//! Bare `relay` (no subcommand) runs the server — the former `relay-server`
//! entrypoint verbatim (env-driven; flags override). The standard agent-facing
//! commands — `relay llm`, `relay upgrade`, `relay issue` (all offline-safe,
//! network paths behind the `self-update`/`issue` features via the shared
//! `cli-std` lib) — sit alongside it per the CONTRIBUTING.md CLI convention.
//! Agents start at `relay llm`.

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

use relay::server::{router, AppState};
use relay::server_config::RelayServerConfig;
use relay::spawn_reconciler;

#[path = "../llm.rs"]
mod llm;

use llm::{TOOL, TOPICS};

#[derive(Parser, Debug)]
#[command(
    name = "relay",
    version,
    about = "relay — durable single-cast work-queue broker (HTTP/2 + OpenAPI)"
)]
struct Cli {
    /// Standard agent-facing command. Omit it to run the server (the default).
    #[command(subcommand)]
    cmd: Option<Command>,
    /// Server flags — used when no subcommand is given (`relay <flags>`).
    #[command(flatten)]
    serve: ServeArgs,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Print agent-facing LLM topics — offline, no server. `outline` (default)
    /// maps the topics; pass a topic id for detail (`--format json` for a
    /// machine-readable form).
    Llm(LlmArgs),
    /// Self-update this binary from a published GitHub release. Resolves the
    /// running target + version, downloads the matching `relay-<target>.tar.gz`,
    /// verifies its sha256, and atomically replaces the executable. `--check`
    /// reports the available version without changing anything.
    Upgrade(UpgradeArgs),
    /// Search, view, and file relay issues on the axiom tracker
    /// (`search`/`view`/`create`). `create` auto-tags `project:relay`;
    /// `search` is filtered to relay's own issues.
    Issue(IssueArgs),
    /// Kubernetes artifacts split by layer: the cluster-scoped CRD, the
    /// operator control plane, and app-namespace Relay instances. Render paths
    /// are offline (they work from the binary); only `operator run` needs the
    /// `operator` build feature.
    K8s(K8sArgs),
    /// Render relay's runtime image Dockerfiles — offline, no server. Image
    /// construction is owned here (not by `k8s`) because the same artifact
    /// feeds compose, kind, and real registries.
    Dockerfile(DockerfileArgs),
    /// Print relay's machine-readable integration spec — offline, no server:
    /// the same OpenAPI document `GET /openapi.json` serves. `spec gen --lang
    /// ts|py|rust` generates a typed client from it (#1209, keep #777).
    Spec(SpecArgs),
    /// Write a consistent snapshot of a RUNNING node's live (un-acked) state
    /// to a backup destination through the shared libs/service-backup runner
    /// (#1209): fetches `GET /admin/backup` and ships the bytes to `--dest`
    /// (`file://` always; `s3://` via the lib). Needs a build with
    /// `--features backup`.
    Backup(BackupArgs),
}

/// `relay spec [--format ...]` or `relay spec gen ...`. Positional slots are
/// reserved for the `gen` subcommand; everything else is a flag (the CLI
/// convention). relay has no request-shape cookbook / value-type catalog, so
/// keep's `--shapes`/`--fields` are deliberately absent (never faked).
#[derive(clap::Args, Debug)]
struct SpecArgs {
    /// Generate a typed client from the spec instead of printing it.
    #[command(subcommand)]
    gen: Option<SpecSub>,
    /// Schema format to emit.
    #[arg(long, value_enum, default_value_t = SpecFormat::Openapi)]
    format: SpecFormat,
}

#[derive(Subcommand, Debug)]
enum SpecSub {
    /// Generate a typed API client (TypeScript / Python / Rust) from relay's
    /// OpenAPI document, written into `--out`.
    Gen(GenArgs),
}

#[derive(clap::Args, Debug)]
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

#[derive(Clone, Copy, Debug, ValueEnum)]
enum GenLang {
    /// TypeScript: types + fetch/axios client + TanStack Query hooks.
    Ts,
    /// Python: pydantic models + a generated sync/async HTTP/2 runtime.
    Py,
    /// Rust: serde models + a reqwest client.
    Rust,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum GenHttp {
    Fetch,
    Axios,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum SpecFormat {
    /// Full OpenAPI 3 document as JSON (default).
    Openapi,
    /// Full OpenAPI 3 document as YAML for LLM/agent reading.
    #[value(alias = "yaml", alias = "openapi.yaml")]
    OpenapiYaml,
    /// Just the component schemas (honest: relay registers no named schemas
    /// today, so this serializes a null `components`).
    JsonSchema,
}

/// `relay backup` flags (#1209): pulls a snapshot over HTTP from a running
/// node and ships it to a destination via `libs/service-backup` (lumen #808).
#[derive(clap::Args, Debug)]
struct BackupArgs {
    /// Base URL of a running relay node, e.g.
    /// `http://<name>.<namespace>.svc.cluster.local:7000` (what the operator's
    /// backup CronJob passes) or `http://localhost:7000` for ad hoc use.
    #[arg(long)]
    url: String,
    /// Destination URI: `file:///path`, `s3://bucket/prefix`, or schema-only
    /// `gs://bucket/prefix` (parses, but the runner supports `file://` and
    /// `s3://` sinks today).
    #[arg(long)]
    dest: String,
    /// Bearer token for `/admin/backup` (needs `admin` on `*`). Falls back to
    /// `RELAY_BACKUP_TOKEN`; omit entirely when the node runs `--auth off`.
    #[arg(long, env = "RELAY_BACKUP_TOKEN")]
    token: Option<String>,
    /// Drop backup objects older than this many seconds after a successful
    /// put. Omit to keep everything.
    #[arg(long)]
    retention_secs: Option<u64>,
}

/// `relay k8s <crd|operator|instance>` — cluster artifacts split by lifecycle
/// layer.
#[derive(clap::Args, Debug)]
struct K8sArgs {
    #[command(subcommand)]
    cmd: K8sCmd,
}

#[derive(Subcommand, Debug)]
enum K8sCmd {
    /// Cluster-scoped API layer: render the Relay CRD.
    Crd(K8sCrdArgs),
    /// Operator control-plane layer: render assets or run the controller.
    Operator(K8sOperatorArgs),
    /// App-namespace declaration: render a Relay custom resource.
    Instance(K8sInstanceArgs),
}

#[derive(clap::Args, Debug)]
struct K8sCrdArgs {
    #[command(subcommand)]
    cmd: K8sCrdCmd,
}

#[derive(Subcommand, Debug)]
enum K8sCrdCmd {
    /// Render the Relay CustomResourceDefinition YAML.
    Render(K8sFileOutputArgs),
}

#[derive(clap::Args, Debug)]
struct K8sOperatorArgs {
    #[command(subcommand)]
    cmd: Option<K8sOperatorCmd>,
}

#[derive(Subcommand, Debug)]
enum K8sOperatorCmd {
    /// Container entrypoint: run the reconcile controller (needs `--features
    /// operator`). The default when no subcommand is given.
    Run,
    /// Render operator namespace/RBAC/deployment YAML.
    Render(K8sOperatorRenderArgs),
}

#[derive(clap::Args, Debug)]
struct K8sOperatorRenderArgs {
    /// Namespace that owns the operator control plane.
    #[arg(long, default_value = "relay-system")]
    namespace: String,
    /// Write to this path instead of stdout. A directory receives
    /// `operator.yaml`.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(clap::Args, Debug)]
struct K8sInstanceArgs {
    #[command(subcommand)]
    cmd: K8sInstanceCmd,
}

#[derive(Subcommand, Debug)]
enum K8sInstanceCmd {
    /// Render a namespaced `kind: Relay` custom resource.
    Render(K8sInstanceRenderArgs),
}

#[derive(clap::Args, Debug)]
struct K8sInstanceRenderArgs {
    /// Built-in instance profile.
    #[arg(long, value_enum, default_value_t = K8sInstanceProfile::Dev)]
    profile: K8sInstanceProfile,
    /// Relay CR name. HA (replicasPerShard > 1) instances must keep the
    /// default `relay` — serve derives raft peer DNS as
    /// `relay-<ordinal>.<peer-service>`.
    #[arg(long)]
    name: Option<String>,
    /// Namespace where the app-facing Relay instance lives.
    #[arg(long)]
    namespace: Option<String>,
    /// Broker image. Defaults are profile-specific.
    #[arg(long)]
    image: Option<String>,
    /// Write to this path instead of stdout. A directory receives `relay.yaml`.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum K8sInstanceProfile {
    /// Small local/kind CR: one broker pod, small disk, verbose logs.
    Dev,
    /// Pre-prod CR: prod-shaped single node, info logs, mid disk.
    Staging,
    /// Production-shape CR: 3-replica raft-HA group, large disk, auth on.
    Prod,
    /// Fill-in-the-blanks CR skeleton for app teams.
    Template,
}

#[derive(clap::Args, Debug)]
struct K8sFileOutputArgs {
    /// Write to this path instead of stdout.
    #[arg(long)]
    out: Option<PathBuf>,
}

/// `relay dockerfile <render>` — render relay's runtime image Dockerfiles.
#[derive(clap::Args, Debug)]
struct DockerfileArgs {
    #[command(subcommand)]
    cmd: DockerfileCmd,
}

#[derive(Subcommand, Debug)]
enum DockerfileCmd {
    /// Render a Dockerfile to stdout or `--out`.
    Render(DockerfileRenderArgs),
}

#[derive(clap::Args, Debug)]
struct DockerfileRenderArgs {
    /// Which runtime image contract to render.
    #[arg(long, value_enum, default_value_t = DockerfileVariant::Source)]
    variant: DockerfileVariant,
    /// Release tag used by `--variant release`; accepts `0.4.3` or
    /// `relay@0.4.3`.
    #[arg(long)]
    version: Option<String>,
    /// Write to this path instead of stdout. A directory receives `Dockerfile`
    /// or `Dockerfile.release`.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum DockerfileVariant {
    /// Build from the workspace source tree.
    Source,
    /// Fetch and verify a published `relay@<version>` release binary.
    Release,
}

/// Server flags (the bare `relay` path) — the `relay-server` env knobs
/// surfaced as flags with env fallback.
#[derive(clap::Args, Debug)]
struct ServeArgs {
    /// h2c listen address for this shard.
    #[arg(long, env = "RELAY_BIND", default_value = "0.0.0.0:7000")]
    bind: String,
    /// Durable log directory (defaults to the core config's data dir).
    #[arg(long, env = "RELAY_DATA_DIR")]
    data_dir: Option<String>,
    /// Graceful-drain window (seconds) held after SIGTERM before the listener
    /// closes, while `/readyz` reports 503 so k8s stops routing.
    #[arg(long, env = "RELAY_GRACE_SECS", default_value_t = 10)]
    grace_secs: u64,
    /// Request-auth mode for the /v1 data plane: `off` (tokenless dev,
    /// the default) or `required` (bearer tokens from the registry file).
    /// Probes stay tokenless either way.
    #[arg(long, env = "RELAY_AUTH", default_value = "off")]
    auth: String,
    /// Bearer-token registry file (JSON `{token: {subject, roles}}`),
    /// mounted from a Secret in production. Required (and validated at
    /// startup) when `--auth required`.
    #[arg(long, env = "RELAY_TOKEN_REGISTRY_FILE")]
    token_registry_file: Option<String>,
    /// Headless-Service name for raft peer DNS in replica/HA mode
    /// (`relay-<ordinal>.<peer-service>:<serve port>`). Only read when the
    /// standard downward-API env says `REPLICAS_PER_SHARD > 1`.
    #[arg(long, env = "RELAY_PEER_SERVICE", default_value = "relay")]
    peer_service: String,
}

/// `relay llm` flags.
#[derive(clap::Args, Debug)]
struct LlmArgs {
    /// Topic id (`outline` lists them all).
    #[arg(default_value = "outline")]
    topic: String,
    /// Output format: `md` (default) or `json`.
    #[arg(long, default_value = "md")]
    format: String,
}

/// `relay upgrade` flags.
#[derive(clap::Args, Debug)]
struct UpgradeArgs {
    /// Report the current and latest version without modifying the binary.
    #[arg(long)]
    check: bool,
    /// Install this exact version (`0.4.3` or `relay@0.4.3`) instead of the latest.
    #[arg(long)]
    tag: Option<String>,
    /// Reinstall even when already on the selected version.
    #[arg(long)]
    force: bool,
    /// Skip the confirmation prompt.
    #[arg(short = 'y', long)]
    yes: bool,
}

/// `relay issue <search|view|create>` — search, read, and file relay issues.
/// Positional slots are reserved for the verb + its primary object, so the rest
/// are flags (the CLI convention).
#[derive(clap::Args, Debug)]
struct IssueArgs {
    #[command(subcommand)]
    cmd: IssueCommand,
}

#[derive(Subcommand, Debug)]
enum IssueCommand {
    /// Search relay's issues (`project:relay`); omit the query to list recent.
    Search(IssueSearchArgs),
    /// Print a single issue by number.
    View(IssueViewArgs),
    /// File a structured issue (auto-tagged `project:relay`).
    Create(IssueCreateArgs),
}

/// `relay issue search [query] [--state] [--limit]` flags.
#[derive(clap::Args, Debug)]
struct IssueSearchArgs {
    /// Search text (omit to list recent issues).
    #[arg(num_args = 0..)]
    query: Vec<String>,
    /// Issue state filter.
    #[arg(long, value_parser = ["open", "closed", "all"], default_value = "open")]
    state: String,
    /// Max results.
    #[arg(long, default_value_t = 20)]
    limit: u32,
}

/// `relay issue view <number>` flags.
#[derive(clap::Args, Debug)]
struct IssueViewArgs {
    /// Issue number.
    number: u64,
}

/// `relay issue create [--title <t>] [message...]` flags.
#[derive(clap::Args, Debug)]
struct IssueCreateArgs {
    /// Issue title (default: derived from the message).
    #[arg(long)]
    title: Option<String>,
    /// Print the issue that would be filed (and its URL) without creating it.
    #[arg(long)]
    dry_run: bool,
    /// Free-text description of the problem.
    #[arg(num_args = 0..)]
    message: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Install the process-level rustls crypto provider before anything parses
    // or dials TLS (the operator/kube and online CLI paths link rustls, which
    // panics without a default provider). A no-op in the default,
    // provider-free build. See `relay::tls`.
    relay::tls::install_default_crypto_provider();
    let cli = Cli::parse();
    match cli.cmd {
        // Default (no subcommand): run the server.
        None => serve_main(cli.serve).await,
        Some(cmd) => dispatch(cmd).await,
    }
}

async fn dispatch(cmd: Command) -> Result<()> {
    match cmd {
        Command::Llm(args) => {
            // Offline: no engine, no server, no I/O beyond stdout.
            let out = cli_std::llm::render(
                TOOL.project,
                TOOL.version,
                TOPICS,
                &args.topic,
                cli_std::llm::Format::parse(&args.format),
            )?;
            println!("{out}");
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
        Command::Issue(args) => dispatch_issue(args).await,
        Command::K8s(args) => k8s(args).await,
        Command::Dockerfile(args) => dockerfile(args),
        Command::Spec(args) => spec(args),
        Command::Backup(args) => dispatch_backup(args).await,
    }
}

/// `relay spec` — offline OpenAPI (JSON / YAML / component schemas), or
/// `spec gen` to generate a typed client. No engine, no server, no I/O beyond
/// stdout / `--out` (#1209, keep #777).
fn spec(args: SpecArgs) -> Result<()> {
    // `spec gen` writes a typed client; everything else prints to stdout.
    if let Some(SpecSub::Gen(gen)) = args.gen {
        return spec_gen(gen);
    }
    let out = match args.format {
        SpecFormat::Openapi => relay::openapi::api_doc_json(),
        SpecFormat::OpenapiYaml => relay::openapi::openapi_yaml(),
        SpecFormat::JsonSchema => relay::openapi::json_schema_json(),
    };
    println!("{out}");
    Ok(())
}

/// `relay spec gen` — generate a typed client from relay's own OpenAPI
/// document (offline; no engine or server) via the shared
/// `libs/openapi-codegen`, written into `--out`. One codegen path, no
/// external tool (keep's spec_gen verbatim).
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
    let output = generate(&relay::openapi::api_doc_json(), &opts)?;
    std::fs::create_dir_all(&args.out)?;
    for file in &output.files {
        let path = args.out.join(&file.rel_path);
        std::fs::write(&path, &file.contents)?;
        println!("generated {}", path.display());
    }
    Ok(())
}

/// `relay backup` (#1209): fetch `{url}/admin/backup` and ship the bytes to
/// `--dest` via `libs/service-backup`, printing the resulting
/// `BackupRunResult` as JSON. This is what the operator's optional backup
/// CronJob (`spec.backup`) invokes on a schedule; it works equally ad hoc.
#[cfg(feature = "backup")]
async fn dispatch_backup(args: BackupArgs) -> Result<()> {
    let dest = service_backup::BackupDestination::from_uri(&args.dest)?;
    let retention = match args.retention_secs {
        Some(secs) => service_backup::RetentionPolicy::max_age_seconds(secs),
        None => service_backup::RetentionPolicy::default(),
    };
    let result =
        relay::backup::run_backup(&args.url, args.token.as_deref(), &dest, &retention).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#[cfg(not(feature = "backup"))]
async fn dispatch_backup(_args: BackupArgs) -> Result<()> {
    anyhow::bail!(
        "this relay build was compiled without backup support; rebuild with \
         `--features backup` (the published image includes it)"
    )
}

/// `relay k8s` — cluster artifacts split by lifecycle layer. Only `operator
/// run` needs kube-rs at runtime; the render paths are offline and work from
/// the binary (the generated CRD is embedded, the operator manifests are
/// string-templated, the instance CRs are profile-templated).
async fn k8s(args: K8sArgs) -> Result<()> {
    match args.cmd {
        K8sCmd::Crd(a) => match a.cmd {
            K8sCrdCmd::Render(a) => write_or_print(a.out.as_deref(), "crd.yaml", &crd_yaml()),
        },
        K8sCmd::Operator(a) => match a.cmd.unwrap_or(K8sOperatorCmd::Run) {
            K8sOperatorCmd::Run => run_operator().await,
            K8sOperatorCmd::Render(a) => {
                let yaml = render_operator_yaml(&a.namespace);
                write_or_print(a.out.as_deref(), "operator.yaml", &yaml)
            }
        },
        K8sCmd::Instance(a) => match a.cmd {
            K8sInstanceCmd::Render(a) => {
                let yaml = render_instance_yaml(&a);
                write_or_print(a.out.as_deref(), "relay.yaml", &yaml)
            }
        },
    }
}

#[cfg(feature = "operator")]
async fn run_operator() -> Result<()> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
    relay::operator::run().await
}

#[cfg(not(feature = "operator"))]
async fn run_operator() -> Result<()> {
    anyhow::bail!(
        "this relay build was compiled without operator support; rebuild with \
         `--features operator` (the published image includes it)"
    )
}

#[cfg(feature = "operator")]
fn crd_yaml() -> String {
    relay::operator::crd_yaml()
}

#[cfg(not(feature = "operator"))]
fn crd_yaml() -> String {
    ensure_trailing_newline(include_str!("../../k8s/operator/crd.yaml"))
}

/// Render the operator control-plane manifests (RBAC + Deployment) with the
/// namespace substituted, from the checked-in fixtures.
fn render_operator_yaml(namespace: &str) -> String {
    let mut out = String::new();
    out.push_str(&replace_operator_namespace(
        include_str!("../../k8s/operator/rbac.yaml"),
        namespace,
    ));
    out.push_str("\n---\n");
    out.push_str(&replace_operator_namespace(
        include_str!("../../k8s/operator/deployment.yaml"),
        namespace,
    ));
    ensure_trailing_newline(&out)
}

fn replace_operator_namespace(input: &str, namespace: &str) -> String {
    input
        .replace("name: relay-system", &format!("name: {namespace}"))
        .replace(
            "namespace: relay-system",
            &format!("namespace: {namespace}"),
        )
}

/// Render a `kind: Relay` custom resource for the selected profile.
fn render_instance_yaml(args: &K8sInstanceRenderArgs) -> String {
    let default_version = env!("CARGO_PKG_VERSION");
    let (default_name, default_namespace, default_image, body) = match args.profile {
        K8sInstanceProfile::Dev => (
            "relay",
            "default",
            "relay:latest".to_string(),
            InstanceBody::Dev,
        ),
        K8sInstanceProfile::Staging => (
            "relay",
            "staging",
            format!("relay:{default_version}"),
            InstanceBody::Staging,
        ),
        K8sInstanceProfile::Prod => (
            "relay",
            "production",
            format!("registry.example.com/relay:{default_version}"),
            InstanceBody::Prod,
        ),
        K8sInstanceProfile::Template => (
            "relay",
            "REPLACE_ME__APP_NAMESPACE",
            "REPLACE_ME__REGISTRY/relay:REPLACE_ME__IMAGE_TAG".to_string(),
            InstanceBody::Template,
        ),
    };
    let name = args.name.as_deref().unwrap_or(default_name);
    let namespace = args.namespace.as_deref().unwrap_or(default_namespace);
    let image = args.image.as_deref().unwrap_or(&default_image);

    let mut yaml = format!(
        "apiVersion: relay.dev/v1alpha1\nkind: Relay\nmetadata:\n  name: {name}\n  namespace: {namespace}\nspec:\n  image: {image}\n"
    );
    match body {
        InstanceBody::Dev => {
            yaml.push_str(
                "  replicasPerShard: 1\n  voterCount: 1\n  logLevel: debug\n  storage: 1Gi\n  resources:\n    cpu: \"250m\"\n    memory: 256Mi\n",
            );
        }
        InstanceBody::Staging => {
            yaml.push_str(
                "  replicasPerShard: 1\n  voterCount: 1\n  logLevel: info\n  storage: 20Gi\n  resources:\n    cpu: \"1\"\n    memory: 2Gi\n",
            );
        }
        InstanceBody::Prod => {
            yaml.push_str(
                "  imagePullPolicy: Always\n  replicasPerShard: 3\n  voterCount: 3\n  logLevel: info\n  storage: 100Gi\n  graceSecs: 30\n  auth: required\n  tokensSecret: relay-token-registry\n  resources:\n    cpu: \"4\"\n    memory: 8Gi\n",
            );
        }
        InstanceBody::Template => {
            yaml.push_str(
                "  imagePullPolicy: IfNotPresent\n  replicasPerShard: REPLACE_ME__REPLICAS_PER_SHARD\n  voterCount: REPLACE_ME__VOTER_COUNT\n  storage: 10Gi\n  resources:\n    cpu: \"1\"\n    memory: 1Gi\n",
            );
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

/// `relay dockerfile render` — render relay's runtime image Dockerfiles. The
/// checked-in Dockerfiles are the fixtures; the CLI is their in-binary form
/// (marker stripping + `relay@version` substitution), so `render` stays the
/// source of truth (keep #777 pattern).
fn dockerfile(args: DockerfileArgs) -> Result<()> {
    match args.cmd {
        DockerfileCmd::Render(a) => {
            let (file_name, body) = match a.variant {
                DockerfileVariant::Source => ("Dockerfile", render_source_dockerfile()),
                DockerfileVariant::Release => (
                    "Dockerfile.release",
                    render_release_dockerfile(a.version.as_deref()),
                ),
            };
            write_or_print(a.out.as_deref(), file_name, &body)
        }
    }
}

fn render_source_dockerfile() -> String {
    strip_ownership_markers(include_str!("../../Dockerfile"))
}

fn render_release_dockerfile(version: Option<&str>) -> String {
    let tag = normalize_relay_tag(version);
    let version = tag.trim_start_matches("relay@");
    let template = strip_ownership_markers(include_str!("../../Dockerfile.release"));
    let mut out = String::new();
    for line in template.lines() {
        if line.starts_with("#   docker build -f projects/relay/Dockerfile.release -t relay:") {
            out.push_str(&format!(
                "#   docker build -f projects/relay/Dockerfile.release -t relay:{version} \\"
            ));
        } else if line.starts_with("#     --build-arg RELAY_VERSION=") {
            out.push_str(&format!("#     --build-arg RELAY_VERSION={tag} ."));
        } else if line.starts_with("ARG RELAY_VERSION=") {
            out.push_str(&format!("ARG RELAY_VERSION={tag}"));
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

/// Normalize a version input into a `relay@<version>` release tag, defaulting
/// to the compiled crate version.
fn normalize_relay_tag(version: Option<&str>) -> String {
    let raw = version
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(env!("CARGO_PKG_VERSION"))
        .trim();
    if raw.starts_with("relay@") {
        raw.to_string()
    } else {
        format!("relay@{raw}")
    }
}

/// Strip AW source-ownership markers so the rendered Dockerfile is the one
/// users build (a no-op for relay's marker-free fixtures; kept for parity).
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

/// Write `body` to `out` (a file, or `default_file` inside a directory) or
/// print it to stdout.
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

/// `relay issue <verb>` — dispatch search/view/create to cli-std. `create`
/// always tags `project:relay`; `search` is filtered to relay's own issues.
async fn dispatch_issue(args: IssueArgs) -> Result<()> {
    match args.cmd {
        IssueCommand::Search(m) => {
            let joined = m.query.join(" ");
            let query = (!joined.trim().is_empty()).then_some(joined);
            cli_std::issue::search(
                &TOOL,
                cli_std::issue::SearchOptions {
                    query,
                    state: m.state,
                    limit: m.limit,
                },
            )
            .await
        }
        IssueCommand::View(m) => cli_std::issue::view(&TOOL, m.number).await,
        IssueCommand::Create(m) => {
            let msg = m.message.join(" ");
            let title = m.title.unwrap_or_else(|| {
                if msg.trim().is_empty() {
                    "relay: issue report".to_string()
                } else {
                    let head: String = msg.lines().next().unwrap_or("").chars().take(72).collect();
                    format!("relay: {head}")
                }
            });
            let message = (!msg.trim().is_empty()).then_some(msg);
            cli_std::issue::create(
                &TOOL,
                cli_std::issue::CreateOptions {
                    title,
                    message,
                    url: None,
                    repo: None,
                    label: vec!["project:relay".to_string()],
                    dry_run: m.dry_run,
                    yes: true,
                },
            )
            .await
        }
    }
}

/// Run the relay server (the default, no-subcommand path) — the former
/// `relay-server` entrypoint: load config, spawn the lease reconciler, serve
/// the app through the shared service shell (#1205): HTTP/1.1 + h2c on one
/// port with a SIGTERM-aware graceful drain (`--grace-secs`).
async fn serve_main(args: ServeArgs) -> Result<()> {
    // RUST_LOG wins; otherwise default to info (keep's pattern — relay's
    // single `--bind` string doesn't map onto service_http::HttpConfig).
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    // Resolve the bearer-auth contract (#1206) BEFORE anything serves: with
    // --auth required a missing/unparseable/empty registry file is a startup
    // error (nonzero exit), never a per-request 401.
    let auth = relay::auth::AuthConfig::resolve(
        &args.auth,
        args.token_registry_file.as_deref(),
        std::env::var(relay::auth::LEGACY_TOKENS_ENV)
            .ok()
            .as_deref(),
    )?;
    tracing::info!(
        required = auth.required,
        "request auth resolved (RELAY_AUTH; probes stay tokenless)"
    );

    let mut config = RelayServerConfig::default();
    config.bind = args.bind;
    if let Some(data_dir) = args.data_dir {
        config.core.data_dir = data_dir;
    }
    let bind = config.bind.clone();
    let reconcile_interval = Duration::from_millis(config.reconcile_interval_ms);

    let mut state = AppState::with_auth(config, auth);
    // Held for the process lifetime; aborts on drop (i.e. never, since serve runs forever).
    let _reconciler = spawn_reconciler(state.relay_handle(), reconcile_interval);

    // Auto-mode HA (#544): the standard downward-API quartet flips replica
    // mode (REPLICAS_PER_SHARD > 1) — no relay-specific flags. Topology comes
    // from raft-host (never re-derive the ordinal math locally); the raft
    // group replicates publishes into this process's engine and its peer
    // router rides the serve port OUTSIDE the bearer-auth data plane (cluster
    // traffic, tokenless like probes; mTLS is a later slice). Held for the
    // process lifetime — dropping it would abort the tick/pump tasks.
    let raft = if raft_host::cluster::replica_mode() {
        // Peer-mTLS material (#1209): load + validate BEFORE the raft group
        // spawns, so a misconfigured deployment (partial RELAY_PEER_TLS_* set,
        // mis-pointed path, unusable PEM) exits nonzero at startup instead of
        // failing at dial time. Termination on the peer port is NOT yet
        // applied — raft-host's h2c transport has no TLS seam (the filed gap
        // in the TD); this proves the mounted material is usable today.
        match relay::peer_tls::PeerTlsConfig::from_env()? {
            Some(tls) => {
                tls.rustls_server_config()?;
                tls.rustls_client_config()?;
                if tls.required {
                    tracing::warn!(
                        cert = %tls.cert.display(),
                        "peer TLS material validated; RELAY_PEER_MTLS=on requested but mTLS \
                         termination on the raft peer port is not yet applied (raft-host/h2c \
                         TLS seam gap) — peer RPCs stay h2c"
                    );
                } else {
                    tracing::info!(
                        cert = %tls.cert.display(),
                        "peer TLS material validated (not required); peer RPCs stay h2c until \
                         the raft-host TLS seam lands"
                    );
                }
            }
            None => tracing::info!(
                "no peer TLS material configured (RELAY_PEER_TLS_*); peer RPCs are plain h2c"
            ),
        }
        let peer_port = bind
            .rsplit(':')
            .next()
            .and_then(|p| p.parse::<u16>().ok())
            .ok_or_else(|| {
                anyhow::anyhow!("cannot derive the raft peer port from --bind {bind}")
            })?;
        let topo = raft_host::ClusterTopology::from_env(
            "relay",
            &args.peer_service,
            peer_port,
            "RELAY_PEERS",
        )?;
        let data_dir = state.config().core.data_dir.clone();
        anyhow::ensure!(
            !data_dir.is_empty(),
            "replica/HA mode requires a durable --data-dir (RELAY_DATA_DIR)"
        );
        let raft = std::sync::Arc::new(relay::RelayRaft::from_topology(
            state.relay_handle(),
            std::path::Path::new(&data_dir),
            &topo,
            relay::RelayRaft::host_config(relay::raft::SNAPSHOT_EVERY),
        )?);
        state.set_raft(std::sync::Arc::clone(&raft));
        tracing::info!(
            node_id = topo.node_id,
            replicas = topo.replicas_per_shard,
            voters = topo.membership.voters.len(),
            "raft: replica/HA mode — publishes replicate; peer RPCs on the serve port"
        );
        Some(raft)
    } else {
        None
    };

    let mut app = router(state.clone());
    if let Some(raft) = &raft {
        app = app.merge(raft.router());
    }
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!(
        addr = %listener.local_addr()?,
        "relay listening (HTTP/1.1 + HTTP/2 cleartext)"
    );

    // Serve HTTP/1.1 + h2c on one port and drain on SIGTERM through the shared
    // service shell: `start_drain` flips `/readyz` to 503 for the grace window
    // before the listener closes.
    let grace = Duration::from_secs(args.grace_secs);
    service_http::serve(
        listener,
        app,
        service_http::shutdown_with_drain(move || state.start_drain(), grace),
    )
    .await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    /// R1: the convention verbs + bare-serve default all parse.
    #[test]
    fn cli_parse_surface() {
        Cli::command().debug_assert();
        assert!(Cli::try_parse_from(["relay"]).unwrap().cmd.is_none());
        assert!(matches!(
            Cli::try_parse_from(["relay", "llm"]).unwrap().cmd,
            Some(Command::Llm(_))
        ));
        assert!(matches!(
            Cli::try_parse_from(["relay", "llm", "http-api", "--format", "json"])
                .unwrap()
                .cmd,
            Some(Command::Llm(_))
        ));
        assert!(matches!(
            Cli::try_parse_from(["relay", "upgrade", "--check"])
                .unwrap()
                .cmd,
            Some(Command::Upgrade(_))
        ));
        assert!(matches!(
            Cli::try_parse_from(["relay", "issue", "search", "lease"])
                .unwrap()
                .cmd,
            Some(Command::Issue(_))
        ));
        assert!(matches!(
            Cli::try_parse_from(["relay", "issue", "view", "42"])
                .unwrap()
                .cmd,
            Some(Command::Issue(_))
        ));
        assert!(matches!(
            Cli::try_parse_from(["relay", "issue", "create", "--dry-run", "it", "broke"])
                .unwrap()
                .cmd,
            Some(Command::Issue(_))
        ));
        // R2: relay-server's env knobs surface as flags on the bare path.
        let cli = Cli::try_parse_from(["relay", "--bind", "127.0.0.1:0", "--data-dir", "/tmp/x"])
            .unwrap();
        assert!(cli.cmd.is_none());
        assert_eq!(cli.serve.bind, "127.0.0.1:0");
        assert_eq!(cli.serve.data_dir.as_deref(), Some("/tmp/x"));
        // #1205: graceful-drain window — default 10, overridable via flag.
        assert_eq!(cli.serve.grace_secs, 10);
        let cli = Cli::try_parse_from(["relay", "--grace-secs", "3"]).unwrap();
        assert_eq!(cli.serve.grace_secs, 3);
        // #1206: the service-auth contract surfaces as serve flags with env
        // fallback — default off (tokenless dev), `required` + registry file.
        assert_eq!(cli.serve.auth, "off");
        assert!(cli.serve.token_registry_file.is_none());
        let cli = Cli::try_parse_from([
            "relay",
            "--auth",
            "required",
            "--token-registry-file",
            "/var/run/secrets/relay/token-registry.json",
        ])
        .unwrap();
        assert_eq!(cli.serve.auth, "required");
        assert_eq!(
            cli.serve.token_registry_file.as_deref(),
            Some("/var/run/secrets/relay/token-registry.json")
        );
        // #544: the raft peer-DNS service name surfaces as a serve flag with
        // env fallback; default matches the headless Service in k8s/.
        assert_eq!(cli.serve.peer_service, "relay");
        let cli = Cli::try_parse_from(["relay", "--peer-service", "relay-peer"]).unwrap();
        assert_eq!(cli.serve.peer_service, "relay-peer");
    }

    /// #1208: `relay k8s crd/operator/instance` verbs parse with their
    /// convention flags. Positionals name subcommands; profile/namespace/out
    /// are flags.
    #[test]
    fn k8s_verbs_parse() {
        Cli::try_parse_from(["relay", "k8s", "crd", "render"]).expect("crd render");
        Cli::try_parse_from(["relay", "k8s", "operator", "run"]).expect("operator run");
        Cli::try_parse_from([
            "relay",
            "k8s",
            "operator",
            "render",
            "--namespace",
            "relay-system",
        ])
        .expect("operator render");

        let cli = Cli::try_parse_from([
            "relay",
            "k8s",
            "instance",
            "render",
            "--profile",
            "prod",
            "--namespace",
            "production",
        ])
        .expect("instance render");
        match cli.cmd {
            Some(Command::K8s(K8sArgs {
                cmd:
                    K8sCmd::Instance(K8sInstanceArgs {
                        cmd: K8sInstanceCmd::Render(a),
                    }),
            })) => {
                assert!(matches!(a.profile, K8sInstanceProfile::Prod));
                assert_eq!(a.namespace.as_deref(), Some("production"));
            }
            other => panic!("expected k8s instance render, got {other:?}"),
        }

        // `operator` with no subcommand defaults to `run`.
        let cli = Cli::try_parse_from(["relay", "k8s", "operator"]).expect("operator default");
        match cli.cmd {
            Some(Command::K8s(K8sArgs {
                cmd: K8sCmd::Operator(K8sOperatorArgs { cmd }),
            })) => assert!(cmd.is_none()),
            other => panic!("expected k8s operator, got {other:?}"),
        }
    }

    /// #1208: `relay dockerfile render` parses with variant/version/out flags.
    #[test]
    fn dockerfile_verbs_parse() {
        let cli = Cli::try_parse_from([
            "relay",
            "dockerfile",
            "render",
            "--variant",
            "release",
            "--version",
            "1.2.3",
        ])
        .expect("dockerfile render should parse");
        match cli.cmd {
            Some(Command::Dockerfile(DockerfileArgs {
                cmd: DockerfileCmd::Render(a),
            })) => {
                assert!(matches!(a.variant, DockerfileVariant::Release));
                assert_eq!(a.version.as_deref(), Some("1.2.3"));
            }
            other => panic!("expected dockerfile render, got {other:?}"),
        }
        // Version tag normalization: bare and prefixed forms converge.
        assert_eq!(normalize_relay_tag(Some("1.2.3")), "relay@1.2.3");
        assert_eq!(normalize_relay_tag(Some("relay@1.2.3")), "relay@1.2.3");
        assert_eq!(
            normalize_relay_tag(None),
            format!("relay@{}", env!("CARGO_PKG_VERSION"))
        );
    }

    /// #1209: `relay spec` / `relay spec gen` / `relay backup` parse with
    /// their convention flags; the keep-only `--shapes`/`--fields` do NOT
    /// parse (relay has no catalogs — omitted, never faked).
    #[test]
    fn spec_and_backup_verbs_parse() {
        Cli::try_parse_from(["relay", "spec"]).expect("spec default");
        Cli::try_parse_from(["relay", "spec", "--format", "openapi-yaml"]).expect("spec yaml");
        Cli::try_parse_from(["relay", "spec", "--format", "json-schema"])
            .expect("spec json-schema");
        assert!(
            Cli::try_parse_from(["relay", "spec", "--shapes"]).is_err(),
            "--shapes is keep-only"
        );
        assert!(
            Cli::try_parse_from(["relay", "spec", "--fields"]).is_err(),
            "--fields is keep-only"
        );

        let cli = Cli::try_parse_from(["relay", "spec", "gen", "--lang", "ts", "--out", "/tmp/x"])
            .expect("spec gen should parse");
        match cli.cmd {
            Some(Command::Spec(SpecArgs {
                gen: Some(SpecSub::Gen(a)),
                ..
            })) => {
                assert!(matches!(a.lang, GenLang::Ts));
                assert_eq!(a.out, PathBuf::from("/tmp/x"));
            }
            other => panic!("expected spec gen, got {other:?}"),
        }

        let cli = Cli::try_parse_from([
            "relay",
            "backup",
            "--url",
            "http://localhost:7000",
            "--dest",
            "file:///tmp/backups",
            "--retention-secs",
            "3600",
        ])
        .expect("backup should parse");
        match cli.cmd {
            Some(Command::Backup(a)) => {
                assert_eq!(a.url, "http://localhost:7000");
                assert_eq!(a.dest, "file:///tmp/backups");
                assert_eq!(a.retention_secs, Some(3600));
            }
            other => panic!("expected backup, got {other:?}"),
        }
    }

    /// R3: build-stamp envs populate ToolInfo (never empty; "unknown" is the
    /// stamped fallback outside a git checkout).
    #[test]
    fn toolinfo_is_stamped() {
        assert_eq!(TOOL.project, "relay");
        assert!(!TOOL.version.is_empty());
        assert!(!TOOL.target.is_empty());
        assert!(!TOOL.git_sha.is_empty());
        assert!(!TOOL.built_at.is_empty());
    }
}
// HANDWRITE-END
