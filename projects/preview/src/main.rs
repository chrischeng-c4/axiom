// <HANDWRITE gap="standardize:claim-code" tracker="projects-preview-src-main-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use preview::discover::discover_base_with_kubectl;
use preview::render::{cleanup_plan, mr_comment, preview_environment};
use preview::{
    apply_rendered_manifests, apply_summary_markdown, load_route_table_from_kubectl,
    load_route_table_from_rendered_dir, plan_guarded_cleanup, read_janitor_plan, render_files,
    render_gitops_bundle, resolve_route_with_base, ApplyOptions, BaseRoute, BaseWorkloadContract,
    CleanupApplyOptions, JanitorInput, RenderInput, RouteRequest,
};

#[derive(Debug, Parser)]
#[command(name = "preview", version, about = "MR-scoped UAT previews for GKE")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Render(RenderArgs),
    DiscoverBase(DiscoverBaseArgs),
    Apply(ApplyArgs),
    Gitops {
        #[command(subcommand)]
        command: GitopsCommand,
    },
    Router {
        #[command(subcommand)]
        command: RouterCommand,
    },
    Cleanup {
        #[command(subcommand)]
        command: CleanupCommand,
    },
    Comment(RenderArgs),
    CleanupPlan(CleanupArgs),
    Llm {
        #[arg(long, default_value = "outline")]
        topic: String,
    },
    Upgrade {
        #[arg(long)]
        check: bool,
        #[arg(long)]
        version: Option<String>,
    },
    Issue {
        #[command(subcommand)]
        command: IssueCommand,
    },
}

#[derive(Debug, Subcommand)]
enum GitopsCommand {
    Render(GitopsRenderArgs),
}

#[derive(Debug, Subcommand)]
enum RouterCommand {
    Resolve(RouterResolveArgs),
}

#[derive(Debug, Subcommand)]
enum CleanupCommand {
    Plan(CleanupJanitorPlanArgs),
    Apply(CleanupApplyArgs),
}

#[derive(Debug, Subcommand)]
enum IssueCommand {
    Search {
        query: Vec<String>,
    },
    View {
        number: u64,
    },
    Create {
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        dry_run: bool,
        message: Vec<String>,
    },
}

#[derive(Debug, Args)]
struct RenderArgs {
    #[arg(long)]
    mr: u32,
    #[arg(long)]
    sha: String,
    #[arg(long)]
    image: String,
    #[arg(long, default_value = "app")]
    app: String,
    #[arg(long, default_value = "uat.example.com")]
    host: String,
    #[arg(long, default_value = "uat-base")]
    base_namespace: String,
    #[arg(long, default_value = "sre")]
    owner: String,
    #[arg(long, default_value_t = 48)]
    ttl_hours: u32,
    #[arg(long, default_value = "preview-system")]
    control_namespace: String,
    #[arg(long, default_value = "preview-runner")]
    workload_identity: String,
    #[arg(long)]
    base_contract: Option<PathBuf>,
    #[arg(long, default_value = "dist/preview")]
    out: PathBuf,
}

#[derive(Debug, Args)]
struct DiscoverBaseArgs {
    #[arg(long)]
    namespace: String,
    #[arg(long)]
    app: String,
    #[arg(long)]
    context: Option<String>,
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct ApplyArgs {
    #[arg(long, default_value = "dist/preview")]
    dir: PathBuf,
    #[arg(long)]
    context: Option<String>,
    #[arg(long, default_value_t = false)]
    dry_run: bool,
    #[arg(long, default_value_t = false)]
    allow_non_kind: bool,
    #[arg(long, default_value_t = false)]
    plan_only: bool,
}

#[derive(Debug, Args)]
struct GitopsRenderArgs {
    #[arg(long, default_value = "dist/preview")]
    dir: PathBuf,
    #[arg(long, default_value = "dist/gitops")]
    out: PathBuf,
}

#[derive(Debug, Args)]
struct RouterResolveArgs {
    #[arg(long)]
    dir: Option<PathBuf>,
    #[arg(long)]
    context: Option<String>,
    #[arg(long, default_value = "preview-system")]
    control_namespace: String,
    #[arg(long)]
    host: String,
    #[arg(long, default_value = "uat-base")]
    base_namespace: String,
    #[arg(long, default_value = "checkout")]
    base_service: String,
    #[arg(long, default_value_t = 80)]
    base_service_port: u16,
    #[arg(long)]
    header_target: Option<String>,
    #[arg(long)]
    cookie_target: Option<String>,
}

#[derive(Debug, Args)]
struct CleanupJanitorPlanArgs {
    #[arg(long)]
    mr: u32,
    #[arg(long)]
    namespace: Option<String>,
    #[arg(long)]
    route_target: Option<String>,
    #[arg(long, default_value = "preview-system")]
    control_namespace: String,
    #[arg(long, default_value = "uat-base")]
    base_namespace: String,
    #[arg(long, default_value_t = false)]
    closed: bool,
    #[arg(long, default_value_t = false)]
    ttl_expired: bool,
    #[arg(long, default_value_t = false)]
    namespace_exists: bool,
    #[arg(long, default_value_t = false)]
    route_binding_exists: bool,
}

#[derive(Debug, Args)]
struct CleanupApplyArgs {
    #[arg(long)]
    plan: PathBuf,
    #[arg(long)]
    context: Option<String>,
    #[arg(long, default_value_t = false)]
    allow_non_kind: bool,
}

#[derive(Debug, Args)]
struct CleanupArgs {
    #[arg(long)]
    mr: u32,
    #[arg(long, default_value_t = false)]
    closed: bool,
    #[arg(long, default_value = "app")]
    app: String,
    #[arg(long, default_value = "unknown")]
    sha: String,
    #[arg(long, default_value = "registry.example.com/app:unknown")]
    image: String,
    #[arg(long, default_value = "uat.example.com")]
    host: String,
    #[arg(long, default_value = "uat-base")]
    base_namespace: String,
    #[arg(long, default_value = "preview-system")]
    control_namespace: String,
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Render(args) => render(args),
        Command::DiscoverBase(args) => discover_base(args),
        Command::Apply(args) => apply(args),
        Command::Gitops { command } => match command {
            GitopsCommand::Render(args) => gitops_render(args),
        },
        Command::Router { command } => match command {
            RouterCommand::Resolve(args) => router_resolve(args),
        },
        Command::Cleanup { command } => match command {
            CleanupCommand::Plan(args) => cleanup_janitor_plan(args),
            CleanupCommand::Apply(args) => cleanup_apply(args),
        },
        Command::Comment(args) => {
            let input = args.into_input()?;
            let env = preview_environment(&input);
            print!("{}", mr_comment(&env));
            Ok(())
        }
        Command::CleanupPlan(args) => {
            let input = RenderInput {
                mr: args.mr,
                sha: args.sha,
                image: args.image,
                app: args.app,
                host: args.host,
                base_namespace: args.base_namespace,
                owner: "sre".to_string(),
                ttl_hours: 48,
                control_namespace: args.control_namespace,
                workload_identity: "preview-runner".to_string(),
                base_contract: None,
            };
            let env = preview_environment(&input);
            println!(
                "{}",
                serde_json::to_string_pretty(&cleanup_plan(&env, args.closed))?
            );
            Ok(())
        }
        Command::Llm { topic } => {
            print_llm(&topic);
            Ok(())
        }
        Command::Upgrade { check, version } => {
            if check {
                println!("preview upgrade check: no release channel configured yet");
            } else {
                println!(
                    "preview upgrade: release channel not configured yet{}",
                    version
                        .as_ref()
                        .map(|v| format!(" (requested {v})"))
                        .unwrap_or_default()
                );
            }
            Ok(())
        }
        Command::Issue { command } => {
            match command {
                IssueCommand::Search { query } => {
                    println!(
                        "preview issue search: backend not configured yet ({})",
                        query.join(" ")
                    );
                }
                IssueCommand::View { number } => {
                    println!("preview issue view: backend not configured yet (#{number})");
                }
                IssueCommand::Create {
                    title,
                    dry_run,
                    message,
                } => {
                    println!("preview issue create: backend not configured yet");
                    println!("dry_run: {dry_run}");
                    if let Some(title) = title {
                        println!("title: {title}");
                    }
                    if !message.is_empty() {
                        println!("message: {}", message.join(" "));
                    }
                }
            }
            Ok(())
        }
    }
}

fn render(args: RenderArgs) -> Result<()> {
    let out = args.out.clone();
    let files = render_files(&args.into_input()?)?;
    for file in files {
        let path = out.join(&file.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create output directory {}", parent.display()))?;
        }
        fs::write(&path, file.contents).with_context(|| format!("write {}", path.display()))?;
    }
    println!("rendered preview contract to {}", out.display());
    Ok(())
}

fn discover_base(args: DiscoverBaseArgs) -> Result<()> {
    let contract = discover_base_with_kubectl(&args.namespace, &args.app, args.context.as_deref())?;
    let output = serde_json::to_string_pretty(&contract)? + "\n";
    if let Some(out) = args.out {
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create output directory {}", parent.display()))?;
        }
        fs::write(&out, output).with_context(|| format!("write {}", out.display()))?;
        println!("discovered base workload contract to {}", out.display());
    } else {
        print!("{output}");
    }
    Ok(())
}

fn apply(args: ApplyArgs) -> Result<()> {
    let summary = apply_rendered_manifests(&ApplyOptions {
        dir: args.dir,
        context: args.context,
        dry_run: args.dry_run,
        allow_non_kind: args.allow_non_kind,
        plan_only: args.plan_only,
    })?;
    print!("{}", apply_summary_markdown(&summary));
    Ok(())
}

fn gitops_render(args: GitopsRenderArgs) -> Result<()> {
    let files = render_gitops_bundle(&args.dir)?;
    for file in files {
        let path = args.out.join(&file.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create output directory {}", parent.display()))?;
        }
        fs::write(&path, file.contents).with_context(|| format!("write {}", path.display()))?;
    }
    println!("rendered preview GitOps bundle to {}", args.out.display());
    Ok(())
}

fn router_resolve(args: RouterResolveArgs) -> Result<()> {
    let bindings = if let Some(dir) = &args.dir {
        load_route_table_from_rendered_dir(dir)
    } else {
        load_route_table_from_kubectl(&args.control_namespace, args.context.as_deref())
    }?;
    let base = BaseRoute {
        host: args.host.clone(),
        namespace: args.base_namespace,
        service: args.base_service,
        service_port: args.base_service_port,
    };
    let mut request = RouteRequest {
        host: args.host,
        ..RouteRequest::default()
    };
    if let Some(target) = args.header_target {
        request.headers.insert("X-UAT-Target".to_string(), target);
    }
    if let Some(target) = args.cookie_target {
        request.cookies.insert("uat_target".to_string(), target);
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&resolve_route_with_base(&bindings, &base, &request))?
    );
    Ok(())
}

fn cleanup_janitor_plan(args: CleanupJanitorPlanArgs) -> Result<()> {
    let namespace = args
        .namespace
        .unwrap_or_else(|| format!("uat-mr-{}", args.mr));
    let route_target = args
        .route_target
        .unwrap_or_else(|| format!("mr-{}", args.mr));
    let plan = plan_guarded_cleanup(JanitorInput {
        mr: args.mr,
        namespace,
        route_target,
        control_namespace: args.control_namespace.clone(),
        protected_namespaces: vec![args.base_namespace, args.control_namespace],
        mr_closed: args.closed,
        ttl_expired: args.ttl_expired,
        namespace_exists: args.namespace_exists,
        route_binding_exists: args.route_binding_exists,
    });
    println!("{}", serde_json::to_string_pretty(&plan)?);
    Ok(())
}

fn cleanup_apply(args: CleanupApplyArgs) -> Result<()> {
    let plan = read_janitor_plan(&args.plan)?;
    let summary = preview::apply_guarded_cleanup(
        &plan,
        &CleanupApplyOptions {
            context: args.context,
            allow_non_kind: args.allow_non_kind,
        },
    )?;
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}

fn print_llm(topic: &str) {
    match topic {
        "outline" => {
            println!("preview manages MR-scoped UAT preview environments for GKE.");
            println!("Use `preview render --mr <id> --sha <sha> --image <image>` in CI.");
            println!(
                "Use `preview apply --dir <dir> --dry-run` for server-side validation in kind."
            );
            println!("Use `preview gitops render --dir <dir> --out <dir>` for PR-based delivery.");
            println!("Use `preview router resolve --dir <dir> --host <host>` to prove base fallback and preview target routing locally.");
            println!("Use `preview cleanup plan` and `preview cleanup apply --plan <json>` for guarded janitor cleanup.");
            println!("The rendered route binding maps cookie/header target `mr-<id>` to a namespace service.");
        }
        _ => {
            println!("unknown topic `{topic}`; available topics: outline");
        }
    }
}

impl RenderArgs {
    fn into_input(self) -> Result<RenderInput> {
        let base_contract = self
            .base_contract
            .as_ref()
            .map(|path| {
                let contents = fs::read_to_string(path)
                    .with_context(|| format!("read base contract {}", path.display()))?;
                serde_json::from_str::<BaseWorkloadContract>(&contents)
                    .with_context(|| format!("parse base contract {}", path.display()))
            })
            .transpose()?;
        Ok(RenderInput {
            mr: self.mr,
            sha: self.sha,
            image: self.image,
            app: self.app,
            host: self.host,
            base_namespace: self.base_namespace,
            owner: self.owner,
            ttl_hours: self.ttl_hours,
            control_namespace: self.control_namespace,
            workload_identity: self.workload_identity,
            base_contract,
        })
    }
}

// </HANDWRITE>
