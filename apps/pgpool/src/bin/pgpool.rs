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
    /// Run the session-mode PostgreSQL proxy: bind the frontend, dial the
    /// configured backend per client, and relay until drain/shutdown.
    Serve(ServeArgs),
    /// Render layered Kubernetes artifacts.
    K8s(K8sArgs),
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

/// Config section of the session-mode-proxy TD: exact env/flag/default
/// surface for `pgpool serve`.
#[derive(clap::Args)]
struct ServeArgs {
    /// Postgres backend host this session-mode proxy dials per client.
    #[arg(long, env = "PGPOOL_BACKEND_HOST", default_value = "127.0.0.1")]
    backend_host: String,
    /// Postgres backend port.
    #[arg(long, env = "PGPOOL_BACKEND_PORT", default_value_t = 5432)]
    backend_port: u16,
    /// Bound on the backend TCP connect, in milliseconds.
    #[arg(
        long,
        env = "PGPOOL_BACKEND_CONNECT_TIMEOUT_MS",
        default_value_t = 5000
    )]
    backend_connect_timeout_ms: u64,
    /// Override the frontend bind address (`host:port`); defaults to the
    /// `RuntimePlan`'s frontend bind (`0.0.0.0:6432`) unchanged.
    #[arg(long, env = "PGPOOL_FRONTEND_BIND")]
    bind: Option<String>,
    /// Grace window for in-flight sessions after SIGTERM/SIGINT, in
    /// milliseconds.
    #[arg(long, env = "PGPOOL_DRAIN_TIMEOUT_MS", default_value_t = 30000)]
    drain_timeout_ms: u64,
    /// Bound on `BackendPool::acquire()`/`acquire_fresh()` waiting for an
    /// idle/freed backend slot before `PoolError::Saturated`, in
    /// milliseconds.
    #[arg(long, env = "PGPOOL_POOL_ACQUIRE_TIMEOUT_MS", default_value_t = 5000)]
    pool_acquire_timeout_ms: u64,
    /// Per-Pod backend connection quota admitted by the control plane.
    #[arg(long, env = "PGPOOL_MAX_BACKEND_CONNECTIONS", default_value_t = 512)]
    max_backend_connections: usize,
    /// Endpoint name used by the asynchronous reserve-lease worker. Empty
    /// disables reserve demand and keeps the historical local pool behavior.
    #[arg(long, env = "PGPOOL_RESERVE_ENDPOINT", default_value = "")]
    reserve_endpoint: String,
    /// Stable Pod identity for the reserve grant. The Deployment renderer
    /// supplies this from metadata.name through the Downward API.
    #[arg(long, env = "PGPOOL_RESERVE_POD", default_value = "")]
    reserve_pod: String,
    /// Stable pod identity used to identify every physical PostgreSQL backend
    /// connection opened by this pgpool process. The Deployment renderer
    /// supplies metadata.name through the Downward API.
    #[arg(long, env = "PGPOOL_POD_NAME", default_value = "local")]
    pod_name: String,
    /// Normal-pool wait before a saturated transaction is allowed to signal
    /// a background reserve-grant request.
    #[arg(long, env = "PGPOOL_RESERVE_POOL_TIMEOUT_MS", default_value_t = 1000)]
    reserve_pool_timeout_ms: u64,
    /// Terminal bounded wait for a normal or already granted reserve lease.
    #[arg(long, env = "PGPOOL_QUEUE_WAIT_TIMEOUT_MS", default_value_t = 5000)]
    queue_wait_timeout_ms: u64,
    /// Idle reserve-grant release timeout, in milliseconds.
    #[arg(long, env = "PGPOOL_RESERVE_IDLE_TIMEOUT_MS", default_value_t = 30000)]
    reserve_idle_timeout_ms: u64,
    /// Lease TTL requested by the background reserve worker, in seconds.
    #[arg(long, env = "PGPOOL_RESERVE_LEASE_TTL_SECONDS", default_value_t = 60)]
    reserve_lease_ttl_seconds: u64,
    /// Maximum reserve units requested in one background allocator exchange.
    #[arg(long, env = "PGPOOL_RESERVE_REQUEST_CHUNK_SIZE", default_value_t = 1)]
    reserve_request_chunk_size: u32,
    /// Override the admin plane bind address (`host:port`); defaults to the
    /// `RuntimePlan`'s admin bind (`0.0.0.0:9080`) unchanged.
    #[arg(long, env = "PGPOOL_ADMIN_BIND")]
    admin_bind: Option<String>,
    /// Operator-facing name for this process's single backend pool; labels
    /// `PoolStats.name`, the `{pool}` path segment in
    /// `GET /pools/{pool}/stats`, and every `/metrics` gauge's `pool=`
    /// label.
    #[arg(long, env = "PGPOOL_POOL_NAME", default_value = "default")]
    pool_name: String,
    /// Bound on the admin HTTP plane's own graceful shutdown once drain
    /// starts, in milliseconds.
    #[arg(long, env = "PGPOOL_ADMIN_DRAIN_TIMEOUT_MS", default_value_t = 30000)]
    admin_drain_timeout_ms: u64,
}

#[derive(clap::Args)]
struct K8sArgs {
    #[command(subcommand)]
    command: K8sCommand,
}

#[derive(Subcommand)]
enum K8sCommand {
    /// Render the cluster-scoped Pgpool CustomResourceDefinition.
    Crd(K8sCrdArgs),
    /// Render or run the Pgpool operator control plane.
    Operator(K8sOperatorArgs),
    /// Render app-namespace Pgpool instance artifacts.
    Instance(K8sInstanceArgs),
}

#[derive(clap::Args)]
struct K8sCrdArgs {
    #[command(subcommand)]
    command: K8sCrdCommand,
}

#[derive(Subcommand)]
enum K8sCrdCommand {
    /// Render the Pgpool CustomResourceDefinition YAML.
    Render(K8sOutputArgs),
}

#[derive(clap::Args)]
struct K8sOperatorArgs {
    #[command(subcommand)]
    command: K8sOperatorCommand,
}

#[derive(Subcommand)]
enum K8sOperatorCommand {
    /// Render operator namespace, RBAC, and Deployment YAML.
    Render(K8sOperatorRenderArgs),
    /// Run the shared leader-elected reconcile controller.
    Run,
}

#[derive(clap::Args)]
struct K8sOutputArgs {
    /// Write YAML to a path instead of stdout.
    #[arg(long)]
    out: Option<std::path::PathBuf>,
}

#[derive(clap::Args)]
struct K8sOperatorRenderArgs {
    /// Namespace containing the operator control plane.
    #[arg(long, default_value = "pgpool-system")]
    namespace: String,
    /// Write YAML to a path instead of stdout.
    #[arg(long)]
    out: Option<std::path::PathBuf>,
}

#[derive(clap::Args)]
struct K8sInstanceArgs {
    #[command(subcommand)]
    command: K8sInstanceCommand,
}

#[derive(Subcommand)]
enum K8sInstanceCommand {
    /// Render the stateless Deployment, ClusterIP Service, ServiceAccount, and PDB.
    Render(K8sInstanceRenderArgs),
}

#[derive(clap::Args)]
struct K8sInstanceRenderArgs {
    #[arg(long, value_enum, default_value_t = K8sProfile::Dev)]
    profile: K8sProfile,
    /// Write YAML to a path instead of stdout.
    #[arg(long)]
    out: Option<std::path::PathBuf>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum K8sProfile {
    Dev,
    Staging,
    Prod,
    Template,
}

impl From<K8sProfile> for pgpool::k8s::InstanceProfile {
    fn from(value: K8sProfile) -> Self {
        match value {
            K8sProfile::Dev => Self::Dev,
            K8sProfile::Staging => Self::Staging,
            K8sProfile::Prod => Self::Prod,
            K8sProfile::Template => Self::Template,
        }
    }
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

#[tokio::main(flavor = "multi_thread")]
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
        Command::Serve(args) => serve(args).await,
        Command::K8s(args) => k8s(args).await,
    }
}

async fn k8s(args: K8sArgs) -> Result<()> {
    match args.command {
        K8sCommand::Crd(args) => match args.command {
            K8sCrdCommand::Render(args) => write_or_print(args.out, pgpool::operator::crd_yaml()),
        },
        K8sCommand::Operator(args) => match args.command {
            K8sOperatorCommand::Render(args) => {
                write_or_print(args.out, pgpool::operator::operator_yaml(&args.namespace))
            }
            K8sOperatorCommand::Run => pgpool::operator::run().await,
        },
        K8sCommand::Instance(args) => match args.command {
            K8sInstanceCommand::Render(args) => write_or_print(
                args.out,
                pgpool::operator::instance_yaml(args.profile.into()),
            ),
        },
    }
}

fn write_or_print(path: Option<std::path::PathBuf>, yaml: String) -> Result<()> {
    if let Some(path) = path {
        std::fs::write(&path, yaml)?;
        println!("wrote {}", path.display());
    } else {
        print!("{yaml}");
    }
    Ok(())
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

// <HANDWRITE gap="missing-generator:logic" tracker="#1882" reason="logic section in pgpool.rs is hand-written pending codegen support">
/// `serve_entry` in the TD Logic flowchart: build a `TcpServerConfig` from
/// `RuntimePlan` with NO server-tcp-level `ConnectionBudget` wired in (the
/// `SessionHandler`/`TransactionHandler` enforce their own admission so a
/// rejection can write a wire-level `ErrorResponse` before closing), share
/// ONE `server_lifecycle::DrainController` between the TCP frontend and the
/// admin plane (`share_drain`), spawn the SIGTERM/SIGINT signal task
/// (`spawn_signal_task`), build the admin router (`build_admin_router`),
/// then run both planes concurrently (`run_both_planes`) until they both
/// drain and exit (R1-R7, AC1-AC4).
async fn serve(args: ServeArgs) -> Result<()> {
    let mut plan = pgpool::default_runtime_plan();
    plan.max_backend_connections = args.max_backend_connections;

    let frontend_bind = match &args.bind {
        Some(addr) => {
            let addr: std::net::SocketAddr = addr.parse()?;
            server_lifecycle::BindConfig {
                host: addr.ip(),
                port: addr.port(),
            }
        }
        None => plan.frontend_bind.clone(),
    };

    if let Some(addr) = &args.admin_bind {
        let addr: std::net::SocketAddr = addr.parse()?;
        plan.admin_bind = server_lifecycle::BindConfig {
            host: addr.ip(),
            port: addr.port(),
        };
    }
    plan.admin_h2c.drain_timeout = std::time::Duration::from_millis(args.admin_drain_timeout_ms);

    let backend_connect_timeout = std::time::Duration::from_millis(args.backend_connect_timeout_ms);
    let drain_timeout = std::time::Duration::from_millis(args.drain_timeout_ms);
    let wire = pgpool::wire::WireCodecConfig::default();

    // Shared backend pool (WI #1289): both pool modes dial/return connections
    // through the same capacity-bounded `BackendPool` (R1), and the admin
    // plane's `NamedPool` holds a clone of this SAME pool for live stats
    // (R3) — cloned before the handler match arm below consumes it.
    let pool_config = pgpool::pool::PoolConfig {
        endpoint: pgpool::proxy::BackendEndpointConfig {
            host: args.backend_host.clone(),
            port: args.backend_port,
        },
        max_backend_connections: plan.max_backend_connections,
        acquire_timeout: std::time::Duration::from_millis(args.pool_acquire_timeout_ms),
        backend_connect_timeout,
        wire,
    };
    let backend_application_name = format!("pgpool-{}", args.pod_name);
    let backend_pool = (if args.reserve_endpoint.is_empty() || args.reserve_pod.is_empty() {
        pgpool::pool::BackendPool::new(pool_config)
    } else {
        pgpool::pool::BackendPool::new_with_reserve(
            pool_config,
            pgpool::pool::ReserveLeaseRuntimeConfig {
                endpoint: args.reserve_endpoint.clone(),
                pod: args.reserve_pod.clone(),
                policy: pgpool::pool::ReserveLeasePolicy {
                    reserve_pool_timeout_seconds: args.reserve_pool_timeout_ms / 1000,
                    queue_wait_timeout_seconds: args.queue_wait_timeout_ms / 1000,
                    reserve_idle_timeout_seconds: args.reserve_idle_timeout_ms / 1000,
                    lease_ttl_seconds: args.reserve_lease_ttl_seconds,
                    request_chunk_size: args.reserve_request_chunk_size,
                },
            },
        )
    })
    .with_backend_application_name(backend_application_name);
    let admin_backend_pool = backend_pool.clone();

    // Called exactly once: the SAME `ConnectionBudget` is shared into
    // whichever `PoolHandler` arm is built below AND into the admin
    // plane's `NamedPool` (R3 Schema: "never constructing a second
    // budget").
    let frontend_budget = plan.frontend_budget();

    let proxy_config = pgpool::proxy::SessionProxyConfig {
        backend: pgpool::proxy::BackendEndpointConfig {
            host: args.backend_host.clone(),
            port: args.backend_port,
        },
        frontend_budget: frontend_budget.clone(),
        backend_connect_timeout,
        drain_timeout,
        wire,
        backend_pool: backend_pool.clone(),
    };

    // `share_drain` (TD Logic section): one `DrainController` for the whole
    // process, cloned into `TcpServerConfig.drain`, `AdminState`, and the
    // signal task below — never a second, independent controller (R2, R7).
    let drain = server_lifecycle::DrainController::new();

    let server_config = server_tcp::TcpServerConfig::new(frontend_bind)
        .with_socket_options(plan.frontend_socket)
        .with_drain_timeout(drain_timeout);
    let server_config = pgpool::admin::wire_server_tcp_drain(server_config, &drain);

    // `PoolHandler` dispatch (TD Schema section): selected once at process
    // start from `RuntimePlan::pool_mode`, never re-evaluated per
    // connection.
    let handler = match plan.pool_mode {
        pgpool::PoolMode::Session => {
            pgpool::pool::PoolHandler::Session(pgpool::proxy::SessionHandler::new(proxy_config))
        }
        pgpool::PoolMode::Transaction => pgpool::pool::PoolHandler::Transaction(
            pgpool::pool::TransactionHandler::new(pgpool::pool::TransactionProxyConfig {
                frontend_budget: frontend_budget.clone(),
                backend_pool,
                wire,
                drain_timeout,
            }),
        ),
    };

    // `build_admin_router` (TD Logic section): one named pool per process
    // today (R3 Schema note), labeled via `--pool-name`/`PGPOOL_POOL_NAME`.
    let admin_state = pgpool::admin::AdminState::new(
        drain.clone(),
        vec![pgpool::admin::NamedPool {
            name: args.pool_name.clone(),
            mode: plan.pool_mode.clone(),
            budget: frontend_budget,
            pool: admin_backend_pool,
        }],
    );
    let admin_router = pgpool::admin::build_router(admin_state);

    // Subscribe both serving planes before the signal task can publish a
    // drain. `DrainSignal::changed` also observes an already-published state,
    // so a drain during either bind below resolves both shutdown futures.
    let tcp_shutdown = {
        let mut signal = drain.signal();
        async move {
            signal.changed().await;
        }
    };
    let admin_shutdown = {
        let mut signal = drain.signal();
        async move {
            signal.changed().await;
        }
    };

    // `spawn_signal_task` (TD Logic section): this is deliberately after
    // AdminState and both plane subscriptions exist, so SIGTERM/SIGINT flips
    // the same controller every startup participant already observes.
    tokio::spawn(pgpool::admin::drain_on_shutdown_signal(
        drain.clone(),
        server_lifecycle::signal::wait_shutdown_signal(),
    ));
    let admin_listener = tokio::net::TcpListener::bind(plan.admin_bind.socket_addr()).await?;

    let listener = server_tcp::bind(&server_config).await?;
    println!("pgpool serve: listening on {}", listener.local_addr()?);
    println!(
        "pgpool serve: backend {}:{}",
        args.backend_host, args.backend_port
    );
    println!(
        "pgpool serve: admin plane on {}",
        admin_listener.local_addr()?
    );

    tokio::join!(
        server_tcp::serve(listener, server_config, handler, tcp_shutdown),
        server_http::serve_h2c_with_options(
            admin_listener,
            admin_router,
            plan.admin_h2c,
            admin_shutdown,
        ),
    );

    Ok(())
}
// </HANDWRITE>
// </HANDWRITE>
