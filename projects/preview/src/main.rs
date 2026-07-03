// <HANDWRITE gap="standardize:claim-code" tracker="projects-preview-src-main-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use preview::render::{cleanup_plan, mr_comment, preview_environment};
use preview::{render_files, RenderInput};

#[derive(Debug, Parser)]
#[command(name = "preview", version, about = "MR-scoped UAT previews for GKE")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Render(RenderArgs),
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
    #[arg(long, default_value = "sre")]
    owner: String,
    #[arg(long, default_value_t = 48)]
    ttl_hours: u32,
    #[arg(long, default_value = "preview-system")]
    control_namespace: String,
    #[arg(long, default_value = "preview-runner")]
    workload_identity: String,
    #[arg(long, default_value = "dist/preview")]
    out: PathBuf,
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
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Render(args) => render(args),
        Command::Comment(args) => {
            let input = args.into_input();
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
                owner: "sre".to_string(),
                ttl_hours: 48,
                control_namespace: "preview-system".to_string(),
                workload_identity: "preview-runner".to_string(),
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
    let files = render_files(&args.into_input())?;
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

fn print_llm(topic: &str) {
    match topic {
        "outline" => {
            println!("preview manages MR-scoped UAT preview environments for GKE.");
            println!("Use `preview render --mr <id> --sha <sha> --image <image>` in CI.");
            println!("The rendered route binding maps cookie/header target `mr-<id>` to a namespace service.");
        }
        _ => {
            println!("unknown topic `{topic}`; available topics: outline");
        }
    }
}

impl RenderArgs {
    fn into_input(self) -> RenderInput {
        RenderInput {
            mr: self.mr,
            sha: self.sha,
            image: self.image,
            app: self.app,
            host: self.host,
            owner: self.owner,
            ttl_hours: self.ttl_hours,
            control_namespace: self.control_namespace,
            workload_identity: self.workload_identity,
        }
    }
}

// </HANDWRITE>
