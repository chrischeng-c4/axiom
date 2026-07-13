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
        Command::Serve(args) => serve(args).await,
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

/// `serve_entry` in the TD Logic flowchart: build a `TcpServerConfig` from
/// `RuntimePlan` with NO tcp-server-level `ConnectionBudget` wired in (the
/// `SessionHandler`/`TransactionHandler` enforce their own admission so a
/// rejection can write a wire-level `ErrorResponse` before closing), share
/// ONE `server_core::DrainController` between the TCP frontend and the
/// admin plane (`share_drain`), spawn the SIGTERM/SIGINT signal task
/// (`spawn_signal_task`), build the admin router (`build_admin_router`),
/// then run both planes concurrently (`run_both_planes`) until they both
/// drain and exit (R1-R7, AC1-AC4).
async fn serve(args: ServeArgs) -> Result<()> {
    let mut plan = pgpool::default_runtime_plan();

    let frontend_bind = match &args.bind {
        Some(addr) => {
            let addr: std::net::SocketAddr = addr.parse()?;
            server_core::BindConfig {
                host: addr.ip(),
                port: addr.port(),
            }
        }
        None => plan.frontend_bind.clone(),
    };

    if let Some(addr) = &args.admin_bind {
        let addr: std::net::SocketAddr = addr.parse()?;
        plan.admin_bind = server_core::BindConfig {
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
    let backend_pool = pgpool::pool::BackendPool::new(pgpool::pool::PoolConfig {
        endpoint: pgpool::proxy::BackendEndpointConfig {
            host: args.backend_host.clone(),
            port: args.backend_port,
        },
        max_backend_connections: plan.max_backend_connections,
        acquire_timeout: std::time::Duration::from_millis(args.pool_acquire_timeout_ms),
        backend_connect_timeout,
        wire,
    });
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
    let drain = server_core::DrainController::new();

    let server_config = tcp_server::TcpServerConfig::new(frontend_bind)
        .with_socket_options(plan.frontend_socket)
        .with_drain_timeout(drain_timeout);
    let server_config = pgpool::admin::wire_tcp_server_drain(server_config, &drain);

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

    // `spawn_signal_task` (TD Logic section): SIGTERM/SIGINT flips the SAME
    // shared controller `POST /drain` flips, so `/readyz` and the TCP
    // accept loop react identically to either trigger (R2).
    tokio::spawn(pgpool::admin::drain_on_shutdown_signal(
        drain.clone(),
        server_core::signal::wait_shutdown_signal(),
    ));

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
    let admin_listener = tokio::net::TcpListener::bind(plan.admin_bind.socket_addr()).await?;

    let listener = tcp_server::bind(&server_config).await?;
    println!("pgpool serve: listening on {}", listener.local_addr()?);
    println!(
        "pgpool serve: backend {}:{}",
        args.backend_host, args.backend_port
    );
    println!(
        "pgpool serve: admin plane on {}",
        admin_listener.local_addr()?
    );

    // `run_both_planes` (TD Logic section): each plane gets its OWN
    // one-shot shutdown future awaiting `drain.signal().changed()` — safe
    // and idempotent since both resolve on the exact same shared
    // controller, whether it was flipped by SIGTERM/SIGINT or `POST
    // /drain` (R2, AC2).
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

    tokio::join!(
        tcp_server::serve(listener, server_config, handler, tcp_shutdown),
        http_server::serve_h2c_with_options(
            admin_listener,
            admin_router,
            plan.admin_h2c,
            admin_shutdown,
        ),
    );

    Ok(())
}
// </HANDWRITE>
