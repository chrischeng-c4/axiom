//! loom binary — one binary, role-per-subcommand (#164):
//! `controller` (scheduler/state), `worker` (resident harness),
//! `run-task` (in-Job entrypoint), `job-controller` (relay → k8s Job bridge),
//! `schema-layer` (worker bidi edge).
//!
//! Alongside the role commands sit the standard agent-facing commands — `loom
//! llm`, `loom upgrade`, `loom issue` (search/view/create) (the CONTRIBUTING.md
//! CLI convention, via the shared `cli-std` lib, #475). Agents start at
//! `loom llm outline`.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "loom",
    version,
    about = "loom — DAG workflow scheduler (control plane over relay + keep)"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Scheduler + sharded, strongly-consistent DAG state; serves the client control API.
    Controller,
    /// Resident pull-loop worker harness (relay lease → keep I/O → ack).
    Worker,
    /// Single-shot in-Job task entrypoint (a k8s Job runs this).
    RunTask,
    /// relay → k8s Job bridge: lease `runner=k8s-job` tasks and create Jobs.
    JobController,
    /// Schema layer: worker-facing bidi edge over the relay work-queue (#432).
    SchemaLayer,
    /// Print loom's OpenAPI control-API contract offline (the `/openapi.json`
    /// twin), or `spec gen` a typed client (ts/py/rust) from it. No server.
    Spec(SpecArgs),
    /// Render loom's container image Dockerfile — `--variant source` (dev/CI
    /// from-source build) or `release` (fetch a published binary). Offline.
    Dockerfile(DockerfileArgs),
    /// Render loom's k8s artifacts (`crd render`, `operator render`,
    /// `instance render --profile`) or run the operator (`operator run`).
    K8s(K8sArgs),
    /// Upload a raft snapshot to a backup destination via `service-backup`
    /// (`--source <file> --destination <uri>`). Needs an `operator` build.
    Backup(BackupArgs),
    /// Print agent-facing LLM topics — offline, no server. `outline` (default)
    /// maps the topics; pass a topic id for detail (`--format json` for a
    /// machine-readable form).
    Llm(LlmArgs),
    /// Self-update this binary from a published GitHub release. Resolves the
    /// running target + version, downloads the matching `loom-<target>.tar.gz`,
    /// verifies its sha256, and atomically replaces the executable. `--check`
    /// reports the available version without changing anything.
    Upgrade(UpgradeArgs),
    /// Search, view, and file loom issues on the axiom tracker. `search` and
    /// `view` read existing `project:loom` issues; `create` files a
    /// diagnostics-rich issue tagged `project:loom` (via `GITHUB_TOKEN`, or a
    /// pre-filled `issues/new` URL when no token is set).
    Issue(IssueArgs),
}

/// `loom spec [--format ...]` / `loom spec gen ...` flags.
#[derive(clap::Args)]
struct SpecArgs {
    /// Generate a typed client from the spec instead of printing it.
    #[command(subcommand)]
    gen: Option<SpecSub>,
    /// Schema format to print (ignored when `gen` is used).
    #[arg(long, value_enum, default_value_t = SpecFormat::Openapi)]
    format: SpecFormat,
}

/// `loom spec` subcommands.
#[derive(Subcommand)]
enum SpecSub {
    /// Generate a typed API client (TypeScript / Python / Rust) from loom's
    /// OpenAPI document, written into `--out`.
    Gen(SpecGenArgs),
}

#[derive(clap::Args)]
struct SpecGenArgs {
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

#[derive(Clone, Copy, clap::ValueEnum)]
enum SpecFormat {
    /// OpenAPI 3 as JSON — the offline twin of `/openapi.json`.
    Openapi,
    /// OpenAPI 3 as YAML.
    #[value(alias = "yaml", alias = "openapi.yaml")]
    OpenapiYaml,
    /// Just the component schemas (request/response data types).
    JsonSchema,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum GenLang {
    /// TypeScript: types + fetch/axios client + TanStack Query hooks.
    Ts,
    /// Python: pydantic models + a generated sync/async HTTP/2 runtime.
    Py,
    /// Rust: serde models + a reqwest client.
    Rust,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum GenHttp {
    Fetch,
    Axios,
}

/// `loom dockerfile render …` flags.
#[derive(clap::Args)]
struct DockerfileArgs {
    #[command(subcommand)]
    cmd: DockerfileCmd,
}

#[derive(Subcommand)]
enum DockerfileCmd {
    /// Print (or write with `--out`) the Dockerfile for a build variant.
    Render(DockerfileRenderArgs),
}

#[derive(clap::Args)]
struct DockerfileRenderArgs {
    /// `source` (from-source dev/CI build) or `release` (fetch published binary).
    #[arg(long, value_enum, default_value_t = DockerfileVariant::Source)]
    variant: DockerfileVariant,
    /// Pin the release image to this version (`0.4.3` or `loom@0.4.3`).
    #[arg(long)]
    version: Option<String>,
    /// Write to this path or directory instead of stdout.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum DockerfileVariant {
    Source,
    Release,
}

/// `loom k8s …` flags. Kubernetes output split by lifecycle layer: `crd`
/// (cluster-scoped API), `operator` (control plane), `instance` (app CR).
#[derive(clap::Args)]
struct K8sArgs {
    #[command(subcommand)]
    cmd: K8sCmd,
}

#[derive(Subcommand)]
enum K8sCmd {
    /// The cluster-scoped `Loom` CRD.
    Crd(K8sCrdArgs),
    /// The operator control plane (RBAC + Deployment), or run it.
    Operator(K8sOperatorArgs),
    /// An app-namespace `Loom` custom resource.
    Instance(K8sInstanceArgs),
}

#[derive(clap::Args)]
struct K8sCrdArgs {
    #[command(subcommand)]
    cmd: K8sCrdCmd,
}
#[derive(Subcommand)]
enum K8sCrdCmd {
    /// Print (or write with `--out`) the CRD YAML.
    Render(K8sOutArgs),
}

#[derive(clap::Args)]
struct K8sOperatorArgs {
    #[command(subcommand)]
    cmd: K8sOperatorCmd,
}
#[derive(Subcommand)]
enum K8sOperatorCmd {
    /// Print (or write) the operator manifests (RBAC + Deployment).
    Render(K8sOperatorRenderArgs),
    /// Run the reconcile controller (needs an `operator` build + a cluster).
    Run,
}
#[derive(clap::Args)]
struct K8sOperatorRenderArgs {
    /// Namespace to deploy the operator control plane into.
    #[arg(long, default_value = "loom-system")]
    namespace: String,
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
    /// Print (or write) a `Loom` CR for a deployment profile.
    Render(K8sInstanceRenderArgs),
}
#[derive(clap::Args)]
struct K8sInstanceRenderArgs {
    /// Deployment profile: dev / staging / prod / template.
    #[arg(long, value_enum, default_value_t = K8sProfile::Dev)]
    profile: K8sProfile,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    namespace: Option<String>,
    #[arg(long)]
    image: Option<String>,
    #[arg(long)]
    out: Option<PathBuf>,
}
#[derive(Clone, Copy, clap::ValueEnum)]
enum K8sProfile {
    Dev,
    Staging,
    Prod,
    Template,
}

#[derive(clap::Args)]
struct K8sOutArgs {
    #[arg(long)]
    out: Option<PathBuf>,
}

/// `loom backup` flags.
#[derive(clap::Args)]
struct BackupArgs {
    /// Snapshot file to upload (the controller's `runs.snapshot.json`).
    #[arg(long)]
    source: PathBuf,
    /// Destination URI: `file:///path`, `s3://bucket/prefix`, `gs://bucket/prefix`.
    #[arg(long)]
    destination: String,
    /// Prune uploaded objects older than this many seconds.
    #[arg(long)]
    max_age_secs: Option<u64>,
}

/// `loom llm` flags.
#[derive(clap::Args)]
struct LlmArgs {
    /// Topic id (`outline` lists them all).
    #[arg(default_value = "outline")]
    topic: String,
    /// Output format: `md` (default) or `json`.
    #[arg(long, default_value = "md")]
    format: String,
}

/// `loom upgrade` flags.
#[derive(clap::Args)]
struct UpgradeArgs {
    /// Report the current and latest version without modifying the binary.
    #[arg(long)]
    check: bool,
    /// Install this exact version (`0.1.0` or `loom@0.1.0`) instead of the latest.
    #[arg(long)]
    tag: Option<String>,
    /// Reinstall even when already on the selected version.
    #[arg(long)]
    force: bool,
    /// Skip the confirmation prompt.
    #[arg(short = 'y', long)]
    yes: bool,
}

/// `loom issue <search|view|create>` flags.
#[derive(clap::Args)]
struct IssueArgs {
    #[command(subcommand)]
    command: IssueCommand,
}

#[derive(Subcommand)]
enum IssueCommand {
    /// Search loom issues (`project:loom`); omit the query to list recent.
    Search(IssueSearchArgs),
    /// Print one issue by number.
    View(IssueViewArgs),
    /// File a diagnostics-rich loom issue.
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
    /// Issue title (defaults to a `loom: <first line>` summary of the message).
    #[arg(short = 't', long)]
    title: Option<String>,
    /// Free-text description of the problem (trailing words; placed above the
    /// diagnostics block). The only positional — parameters are flags.
    #[arg(value_name = "MSG", num_args = 0..)]
    message: Vec<String>,
    /// Include a running node's `/version`+`/healthz` (e.g. http://localhost:7474).
    #[arg(long)]
    url: Option<String>,
    /// Target repository (`owner/name`); defaults to loom's release repo.
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

/// This binary's identity + build provenance for the standard CLI ops
/// (`upgrade` / `issue`), per the CONTRIBUTING.md CLI convention (#475).
const TOOL: cli_std::ToolInfo = cli_std::ToolInfo {
    project: "loom",
    repo: "chrischeng-c4/axiom",
    target: env!("LOOM_TARGET"),
    version: env!("CARGO_PKG_VERSION"),
    git_sha: env!("LOOM_GIT_SHA"),
    built_at: env!("LOOM_BUILT_AT"),
};

/// loom's agent-facing `llm` topics — the single in-code source of truth.
const TOPICS: &[cli_std::llm::Topic] = &[
    cli_std::llm::Topic {
        id: "architecture",
        summary: "control plane over relay + keep — never on the data path (claim-check)",
        body: "# loom — architecture\n\n\
            loom is the DAG workflow scheduler: it composes per-task lifecycles into a \
            dynamic DAG. It coordinates with small messages only — payload bytes never \
            traverse loom (claim-check via keep). keep (result store) and relay (broker) \
            stay deliberately simple; all orchestration complexity is concentrated here.\n\n\
            - **client** knows loom (control) + keep (data, claim-check); never relay.\n\
            - **worker** knows relay + keep; never loom. loom observes completion via relay acks.\n\
            - **relay / keep** stay passive; loom is the only role that coordinates both.\n\n\
            Node lifecycle: loom picks the next ready node → assembles input refs from keep → \
            publishes the task to relay (tagged with its `runner` class) → a worker leases, runs, \
            writes the result to keep, acks → loom updates DAG state → repeat. Fan-in barrier = \
            done counter (by task id, not attempt). State is sharded + strongly consistent (Raft, \
            #110). Transport is HTTP/2 cleartext (h2c). Epic: #106.\n",
    },
    cli_std::llm::Topic {
        id: "roles",
        summary: "the role-per-subcommand binary + the key env vars each role reads",
        body: "# loom — roles (subcommands)\n\n\
            One binary, one role per subcommand:\n\n\
            - `loom controller` — scheduler + DAG state; serves the client control API. \
            Env: `LOOM_ADDR`, `LOOM_RELAY`, `LOOM_KEEP`, `LOOM_COMPLETION_SHARDS`, \
            `LOOM_DATA_DIR`/`LOOM_RAFT_DIR`, `LOOM_DISPATCH_DEADLINE_SECS`, `LOOM_GC_RETENTION_SECS`.\n\
            - `loom worker` — resident pull loop (relay lease → keep I/O → run handler → ack). \
            Env: `LOOM_RELAY`, `LOOM_KEEP`, `LOOM_RUNNER`, `LOOM_WORKER_CONCURRENCY`, \
            `LOOM_INLINE_MAX_BYTES`; set `LOOM_SCHEMA_LAYER` to use the bidi edge instead of \
            direct relay leasing.\n\
            - `loom run-task` — single-shot in-Job entrypoint (a k8s Job runs this), driven by \
            `LOOM_TASK_*` env (run/node/attempt/name/input-refs).\n\
            - `loom job-controller` — relay → k8s Job bridge: leases `runner=k8s-job` tasks and \
            creates Jobs (`LOOM_JOB_IMAGE`, `LOOM_JOB_NAMESPACE`).\n\
            - `loom schema-layer` — worker-facing bidi edge over the relay work-queue (#432); \
            `LOOM_ADDR` (default `0.0.0.0:7475`), `LOOM_RELAY`, `LOOM_KEEP`, \
            `LOOM_KEEP_TOKEN_SECRET`.\n",
    },
    cli_std::llm::Topic {
        id: "control-api",
        summary: "the client control API — submit a run, poll status, complete a node",
        body: "# loom — control API (the controller's HTTP surface)\n\n\
            Small JSON over HTTP/2 cleartext; payload bytes stay in keep.\n\n\
            - `POST /runs` — submit a workflow run (DAG of nodes); returns the run view.\n\
            - `GET  /runs/{id}` — poll run + per-node status.\n\
            - `POST /runs/{id}/nodes/{node}/complete` — report a node completion \
            (`result_ref`/`result_inline`, `attempt`, `failed`, and runtime `fan_out` children). \
            Workers normally reach this indirectly: they ack relay and loom folds the completion \
            off the `loom.completions` subject — but the endpoint also drives manual completion.\n\n\
            Standard archetype surface on the same port (via `service-http`): \
            `GET /healthz` (liveness), `GET /readyz` (readiness — 503 while draining), \
            `GET /metrics` (Prometheus), `GET /openapi.json` (machine OpenAPI), `GET /docs` \
            (Swagger UI). The OpenAPI is also emitted offline by `loom spec`.\n\n\
            Dynamic fan-out (#116): a completing node may carry `fan_out` children that loom \
            splices into the DAG; the fan-in barrier waits for all siblings before readying the \
            join. At-least-once completions are idempotent (deduped by run/node/attempt, #437).\n",
    },
];

// <HANDWRITE gap="missing-generator:logic" tracker="pending-tracker" reason="logic section in main.rs is hand-written pending codegen support">
fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Controller => loom::controller::run(),
        Command::Worker => loom::worker::run(),
        Command::RunTask => loom::runtask::run(),
        Command::JobController => loom::jobcontroller::run(),
        Command::SchemaLayer => loom::schema_layer::run(),
        // Offline: emit the OpenAPI contract (or a typed client), no server.
        Command::Spec(args) => spec(args),
        // Offline: emit a container image Dockerfile, no server.
        Command::Dockerfile(args) => dockerfile(args),
        // k8s artifacts (offline render, except `operator run`).
        Command::K8s(args) => k8s(args),
        // Upload a raft snapshot via service-backup (operator build only).
        Command::Backup(args) => backup(args),
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
            Ok(())
        }
        // The standard ops are async; loom's role commands each build their own
        // runtime, so main stays sync and these block on a local one.
        Command::Upgrade(args) => block_on(cli_std::upgrade::run(
            &TOOL,
            cli_std::upgrade::Options {
                check: args.check,
                tag: args.tag,
                force: args.force,
                yes: args.yes,
            },
        )),
        Command::Issue(args) => block_on(issue(args)),
    }
}
// </HANDWRITE>

/// Dispatch `loom issue <search|view|create>` to the shared `cli_std::issue`
/// implementation. `search`/`view` are read-only; `create` files (or previews)
/// a diagnostics-rich issue always tagged `project:loom` (CLI convention).
async fn issue(args: IssueArgs) -> anyhow::Result<()> {
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
                    let head: String = message.lines().next().unwrap_or("").chars().take(72).collect();
                    format!("loom: {head}")
                } else {
                    "loom: issue report".to_string()
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
                    label: std::iter::once("project:loom".to_string())
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

/// `loom spec` — print the control-API OpenAPI offline (the `/openapi.json`
/// twin the controller also serves), or `spec gen` a typed client. Offline: the
/// document is built from the in-binary `utoipa` derive; no server or network.
fn spec(args: SpecArgs) -> anyhow::Result<()> {
    if let Some(SpecSub::Gen(gen)) = args.gen {
        return spec_gen(gen);
    }
    let doc = loom::controller::openapi();
    let out = match args.format {
        SpecFormat::Openapi => doc.to_pretty_json()?,
        SpecFormat::OpenapiYaml => serde_yaml::to_string(&doc)?,
        SpecFormat::JsonSchema => {
            let v = serde_json::to_value(&doc)?;
            let schemas = v
                .get("components")
                .and_then(|c| c.get("schemas"))
                .cloned()
                .unwrap_or(serde_json::Value::Null);
            serde_json::to_string_pretty(&schemas)?
        }
    };
    println!("{out}");
    Ok(())
}

/// `loom spec gen` — emit a typed client (ts/py/rust) from loom's own OpenAPI via
/// the shared `cclab-openapi-codegen` polyglot core. No external codegen step.
fn spec_gen(args: SpecGenArgs) -> anyhow::Result<()> {
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
    let output = generate(&loom::controller::openapi().to_pretty_json()?, &opts)?;
    std::fs::create_dir_all(&args.out)?;
    for file in &output.files {
        let path = args.out.join(&file.rel_path);
        std::fs::write(&path, &file.contents)?;
        println!("generated {}", path.display());
    }
    Ok(())
}

/// `loom dockerfile render` — emit the checked-in Dockerfile for a build
/// variant (the checked-in files are the fixtures; `--version` rebinds the
/// release image tag). Offline: the templates are baked into the binary.
fn dockerfile(args: DockerfileArgs) -> anyhow::Result<()> {
    match args.cmd {
        DockerfileCmd::Render(args) => {
            let (name, body) = match args.variant {
                DockerfileVariant::Source => {
                    ("Dockerfile".to_string(), include_str!("../Dockerfile").to_string())
                }
                DockerfileVariant::Release => (
                    "Dockerfile.release".to_string(),
                    render_release_dockerfile(args.version.as_deref()),
                ),
            };
            write_or_print(args.out.as_deref(), &name, &body)
        }
    }
}

/// Rebind the `LOOM_VERSION` build-arg (and the example `docker build` line) in
/// the release Dockerfile to `version` (defaults to the checked-in tag).
fn render_release_dockerfile(version: Option<&str>) -> String {
    let template = include_str!("../Dockerfile.release");
    let Some(version) = version else {
        return template.to_string();
    };
    let tag = if version.starts_with("loom@") {
        version.to_string()
    } else {
        format!("loom@{version}")
    };
    let bare = tag.trim_start_matches("loom@");
    let mut out = String::new();
    for line in template.lines() {
        if line.starts_with("ARG LOOM_VERSION=") {
            out.push_str(&format!("ARG LOOM_VERSION={tag}"));
        } else if line.starts_with("#   docker build -f projects/loom/Dockerfile.release -t loom:") {
            out.push_str(&format!(
                "#   docker build -f projects/loom/Dockerfile.release -t loom:{bare} \\"
            ));
        } else if line.starts_with("#     --build-arg LOOM_VERSION=") {
            out.push_str(&format!("#     --build-arg LOOM_VERSION={tag} ."));
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

/// `loom k8s …` — render the cluster-scoped CRD, the operator control plane, or
/// an app-namespace `Loom` CR; or run the reconcile controller. The render paths
/// are offline (templates baked into the binary); `operator run` needs a build
/// with the `operator` feature and a cluster.
fn k8s(args: K8sArgs) -> anyhow::Result<()> {
    match args.cmd {
        K8sCmd::Crd(a) => match a.cmd {
            K8sCrdCmd::Render(o) => write_or_print(o.out.as_deref(), "crd.yaml", &crd_yaml()),
        },
        K8sCmd::Operator(a) => match a.cmd {
            K8sOperatorCmd::Render(r) => {
                let yaml = render_operator_yaml(&r.namespace);
                write_or_print(r.out.as_deref(), "operator.yaml", &yaml)
            }
            K8sOperatorCmd::Run => block_on(run_operator()),
        },
        K8sCmd::Instance(a) => match a.cmd {
            K8sInstanceCmd::Render(r) => {
                let yaml = render_instance_yaml(&r);
                write_or_print(r.out.as_deref(), "loom.yaml", &yaml)
            }
        },
    }
}

/// The `Loom` CRD YAML. An `operator` build derives it from the source type; a
/// serving build emits the checked-in copy (kept in sync by the operator build).
#[cfg(feature = "operator")]
fn crd_yaml() -> String {
    loom::operator::crd_yaml()
}
#[cfg(not(feature = "operator"))]
fn crd_yaml() -> String {
    include_str!("../k8s/operator/crd.yaml").to_string()
}

/// The operator control plane (RBAC + Deployment) as one document, with the
/// system namespace substituted.
fn render_operator_yaml(namespace: &str) -> String {
    let sub = |s: &str| {
        s.replace("namespace: loom-system", &format!("namespace: {namespace}"))
            .replace("name: loom-system", &format!("name: {namespace}"))
    };
    format!(
        "{}\n---\n{}",
        sub(include_str!("../k8s/operator/rbac.yaml")).trim_end(),
        sub(include_str!("../k8s/operator/deployment.yaml")).trim_end(),
    )
}

/// Run the reconcile controller. Only an `operator` build links the kube runtime.
#[cfg(feature = "operator")]
fn run_operator() -> impl std::future::Future<Output = anyhow::Result<()>> {
    tracing_subscriber::fmt().init();
    loom::operator::run()
}
#[cfg(not(feature = "operator"))]
async fn run_operator() -> anyhow::Result<()> {
    anyhow::bail!(
        "this loom build has no operator support; rebuild with `--features operator` \
         (the published operator image includes it)"
    )
}

/// A `Loom` custom resource for a deployment profile — dev (single-node),
/// staging / prod (raft HA), or a REPLACE_ME template.
fn render_instance_yaml(args: &K8sInstanceRenderArgs) -> String {
    let version = env!("CARGO_PKG_VERSION");
    let (name, ns, image, body) = match args.profile {
        K8sProfile::Dev => ("loom", "default", "loom:dev".to_string(), Profile::Dev),
        K8sProfile::Staging => ("loom", "loom-staging", "loom:staging".to_string(), Profile::Staging),
        K8sProfile::Prod => ("loom", "loom", format!("loom:{version}"), Profile::Prod),
        K8sProfile::Template => (
            "REPLACE_ME__NAME",
            "REPLACE_ME__NAMESPACE",
            "REPLACE_ME__REGISTRY/loom:REPLACE_ME__TAG".to_string(),
            Profile::Template,
        ),
    };
    let name = args.name.as_deref().unwrap_or(name);
    let ns = args.namespace.as_deref().unwrap_or(ns);
    let image = args.image.as_deref().unwrap_or(&image);
    let mut y = format!(
        "apiVersion: loom.dev/v1alpha1\nkind: Loom\nmetadata:\n  name: {name}\n  namespace: {ns}\nspec:\n  image: {image}\n"
    );
    match body {
        Profile::Dev => y.push_str(
            "  shardCount: 1\n  replicasPerShard: 1\n  voterCount: 1\n  storage: 2Gi\n",
        ),
        Profile::Staging => y.push_str(
            "  shardCount: 1\n  replicasPerShard: 3\n  voterCount: 3\n  storage: 5Gi\n  cpu: \"500m\"\n  memory: 512Mi\n",
        ),
        Profile::Prod => y.push_str(
            "  imagePullPolicy: Always\n  shardCount: 1\n  replicasPerShard: 3\n  voterCount: 3\n  storage: 10Gi\n  cpu: \"1\"\n  memory: 1Gi\n  gcRetentionSecs: 86400\n  backupSchedule: \"0 */6 * * *\"\n  backupDestination: \"s3://loom-backups/prod\"\n",
        ),
        Profile::Template => y.push_str(
            "  imagePullPolicy: IfNotPresent\n  shardCount: 1\n  replicasPerShard: REPLACE_ME__REPLICAS\n  voterCount: REPLACE_ME__VOTERS\n  storage: REPLACE_ME__STORAGE\n",
        ),
    }
    y
}

enum Profile {
    Dev,
    Staging,
    Prod,
    Template,
}

/// `loom backup` — upload a raft snapshot via `service-backup` (operator build).
#[cfg(feature = "operator")]
fn backup(args: BackupArgs) -> anyhow::Result<()> {
    loom::operator::backup::run(&args.source, &args.destination, args.max_age_secs)
}
#[cfg(not(feature = "operator"))]
fn backup(_args: BackupArgs) -> anyhow::Result<()> {
    anyhow::bail!(
        "this loom build has no backup support; rebuild with `--features operator`"
    )
}

/// Write `body` to `out` (a file, or `<dir>/<name>` when `out` is a directory),
/// or print it to stdout when `out` is `None`. Shared by the render verbs.
fn write_or_print(out: Option<&std::path::Path>, name: &str, body: &str) -> anyhow::Result<()> {
    match out {
        None => {
            print!("{body}");
            Ok(())
        }
        Some(p) => {
            let path = if p.is_dir() { p.join(name) } else { p.to_path_buf() };
            std::fs::write(&path, body)?;
            println!("wrote {}", path.display());
            Ok(())
        }
    }
}

/// Run a future to completion on a fresh runtime. The standard CLI ops
/// (`upgrade`/`issue`) are async, but loom's role subcommands each build
/// their own runtime, so `main` stays sync.
fn block_on<F: std::future::Future<Output = anyhow::Result<()>>>(fut: F) -> anyhow::Result<()> {
    tokio::runtime::Runtime::new()?.block_on(fut)
}
