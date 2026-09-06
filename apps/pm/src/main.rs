use std::path::PathBuf;
use anyhow::Result;
use clap::{Parser, Subcommand};
use pm::mcp;
use pm::store::PmStore;

#[derive(Parser, Debug)]
#[command(name = "pm")]
#[command(about = "Lightweight, space-saving local Project Management MCP Server for AI Agents")]
#[command(version)]
struct Cli {
    #[arg(long, global = true, help = "Path to PM state file (default: .pm/state.json)")]
    db: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Start the MCP server on stdio or HTTP")]
    Serve {
        #[arg(long, help = "Optional HTTP listen address, e.g. 127.0.0.1:3200 (if omitted, runs on stdio)")]
        http: Option<std::net::SocketAddr>,
    },

    #[command(about = "Initialize .pm repository in the current directory")]
    Init {
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    #[command(about = "Show project summary dashboard in the terminal")]
    Summary {
        #[arg(help = "Project ID to inspect")]
        project_id: String,
    },
}

fn resolve_db_path(explicit: Option<PathBuf>) -> PathBuf {
    if let Some(p) = explicit {
        return p;
    }

    // Default to .pm/state.json in current directory or git root
    let local = PathBuf::from(".pm/state.json");
    if local.exists() {
        return local;
    }

    // Check if we are inside a repo with .git
    let mut curr = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        let candidate = curr.join(".pm").join("state.json");
        if candidate.exists() {
            return candidate;
        }
        if curr.join(".git").exists() {
            return curr.join(".pm").join("state.json");
        }
        if !curr.pop() {
            break;
        }
    }

    PathBuf::from(".pm/state.json")
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let db_path = resolve_db_path(cli.db);

    match cli.command.unwrap_or(Commands::Serve { http: None }) {
        Commands::Serve { http } => {
            let store = PmStore::new(db_path)?;
            if let Some(addr) = http {
                mcp::http::serve_http(store, addr).await?;
            } else {
                mcp::serve(store).await?;
            }
        }

        Commands::Init { path } => {
            let pm_dir = path.join(".pm");
            std::fs::create_dir_all(&pm_dir)?;
            let state_file = pm_dir.join("state.json");
            if !state_file.exists() {
                std::fs::write(&state_file, "{\n  \"projects\": {},\n  \"prds\": {},\n  \"tech_designs\": {},\n  \"features\": {},\n  \"tasks\": {}\n}\n")?;
                println!("Initialized empty PM repository at: {}", state_file.display());
            } else {
                println!("PM repository already exists at: {}", state_file.display());
            }
        }

        Commands::Summary { project_id } => {
            let store = PmStore::new(db_path)?;
            let summary = store.read(|s| s.get_project_summary(&project_id)).await;
            println!("{}", serde_json::to_string_pretty(&summary)?);
        }
    }

    Ok(())
}
