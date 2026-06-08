//! Server CLI commands
//!
//! Manages the unified HTTP server lifecycle and project registration:
//! - start: Start server (optionally as daemon)
//! - register: Register a project
//! - unregister: Unregister a project
//! - list: List server status and registered projects
//! - view: Open browser to Plan Viewer URL
//! - shutdown: Stop the server

use crate::registry::Registry;
use crate::Result;
use clap::Subcommand;
use colored::Colorize;
use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[cfg(unix)]
use std::fs::{File, OpenOptions};

#[derive(Subcommand)]
pub enum ServerCommands {
    /// Start unified server
    Start {
        /// HTTP server port (default: 3456)
        #[arg(long, default_value = "3456")]
        port: u16,

        /// Run in daemon mode (background)
        #[arg(long)]
        daemon: bool,

        /// Auto-update client configurations (.gemini/settings.json, ~/.codex/config.toml)
        #[arg(long)]
        update_clients: bool,
    },

    /// Ensure server is running (auto-start if needed)
    Ensure {
        /// HTTP server port (default: 3456)
        #[arg(long, default_value = "3456")]
        port: u16,

        /// Suppress output messages
        #[arg(long, short)]
        quiet: bool,
    },

    /// Register a project with the server
    Register {
        /// Path to project directory (default: current directory)
        path: Option<PathBuf>,
    },

    /// Unregister a project from the server
    Unregister {
        /// Project name or path (default: current directory name)
        project: Option<String>,
    },

    /// List registered projects and server status
    List,

    /// Open Plan Viewer in browser for a specific change
    View {
        /// Project name
        project: String,
        /// Change ID
        change: String,
    },

    /// Shutdown the server
    Shutdown,

    /// Internal: run HTTP server (called by daemon)
    #[command(hide = true)]
    Run {
        #[arg(long)]
        port: u16,
    },
}

pub async fn run(cmd: ServerCommands) -> Result<()> {
    match cmd {
        ServerCommands::Start {
            port,
            daemon,
            update_clients,
        } => start_server(port, daemon, update_clients)?,

        ServerCommands::Ensure { port, quiet } => ensure_server_running(port, quiet)?,

        ServerCommands::Register { path } => register_project(path)?,

        ServerCommands::Unregister { project } => unregister_project(project)?,

        ServerCommands::List => list_status()?,

        ServerCommands::View { project, change } => view_change(&project, &change)?,

        ServerCommands::Shutdown => shutdown_server()?,

        ServerCommands::Run { port } => {
            run_server_daemon(port).await?;
        }
    }

    Ok(())
}

/// Start the server
fn start_server(port: u16, daemon: bool, update_clients: bool) -> Result<()> {
    // Check if server is already running
    let mut registry = match Registry::load() {
        Ok(reg) => {
            if reg.is_server_running() {
                println!("{}", "Server already running".green());
                println!(
                    "{}",
                    format!("  Dashboard: http://localhost:{}/", reg.server.port).cyan()
                );
                return Ok(());
            } else {
                println!(
                    "{}",
                    "Previous server process not found, starting new one".yellow()
                );
                // Clear server info but preserve projects for re-registration (R1)
                Registry::clear_server_info()?;
                start_server_process(port, daemon)?
            }
        }
        Err(_) => start_server_process(port, daemon)?,
    };

    println!(
        "{}",
        format!(
            "Server running on http://localhost:{}",
            registry.server.port
        )
        .green()
    );
    println!(
        "{}",
        format!("  Dashboard: http://localhost:{}/", registry.server.port).cyan()
    );
    println!(
        "{}",
        format!(
            "  MCP endpoint: http://localhost:{}/mcp",
            registry.server.port
        )
        .cyan()
    );

    // Auto-register current project if in a project directory
    let current_dir = env::current_dir()?;
    if crate::registry::is_valid_project(&current_dir) {
        let project_name = current_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        if registry.get_project_path(&project_name).is_none() {
            registry.register_project(project_name.clone(), current_dir.clone())?;
            println!(
                "{}",
                format!("Registered current project: {}", project_name).green()
            );
        }
    }

    if update_clients {
        update_client_configs(&registry)?;
    }

    Ok(())
}

/// Ensure server is running, starting it if needed (for auto-start)
///
/// This is the main function for MCP auto-start. It:
/// 1. Checks if server is already running
/// 2. If not, starts it as a daemon
/// 3. Returns the server URL
pub fn ensure_server_running(port: u16, quiet: bool) -> Result<()> {
    // Check if server is already running
    if let Ok(registry) = Registry::load() {
        if registry.is_server_running() {
            if !quiet {
                println!(
                    "{}",
                    format!(
                        "Server running on http://localhost:{}/mcp",
                        registry.server.port
                    )
                    .green()
                );
            }
            return Ok(());
        }
        // Server not running, clear server info but preserve projects (R1)
        Registry::clear_server_info()?;
    }

    // Start server in daemon mode
    if !quiet {
        println!("{}", "Starting server...".cyan());
    }

    let registry = start_server_daemon_silent(port)?;

    if !quiet {
        println!(
            "{}",
            format!(
                "Server started on http://localhost:{}/mcp",
                registry.server.port
            )
            .green()
        );
    }

    Ok(())
}

/// Get the server log file path (~/.cclab/server.log)
#[cfg(unix)]
fn get_log_file() -> Result<File> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    let cclab_dir = home.join(".cclab");
    std::fs::create_dir_all(&cclab_dir)?;
    let log_path = cclab_dir.join("server.log");

    // Open file for append, create if not exists
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    Ok(file)
}

/// Start server in daemon mode without verbose output (for auto-start)
fn start_server_daemon_silent(port: u16) -> Result<Registry> {
    let exe_path = env::current_exe()?;

    // Create registry with placeholder PID, merging any persisted projects (R4)
    let mut registry = Registry::new(0, port);
    registry.merge_persisted_projects();
    registry.save()?;

    let mut cmd = Command::new(&exe_path);
    cmd.args(["server", "run", "--port", &port.to_string()]);

    #[cfg(unix)]
    {
        let log_file = get_log_file()?;
        let log_file_err = log_file.try_clone()?;
        cmd.stdin(Stdio::null())
            .stdout(Stdio::from(log_file))
            .stderr(Stdio::from(log_file_err));
    }

    let child = cmd.spawn()?;
    let pid = child.id();

    // Update registry with actual PID
    let mut registry = Registry::load()?;
    registry.server.pid = pid;
    registry.save()?;

    // Wait for server to start
    std::thread::sleep(std::time::Duration::from_millis(500));

    Ok(registry)
}

/// Start the server process
fn start_server_process(port: u16, _daemon: bool) -> Result<Registry> {
    let exe_path = env::current_exe()?;

    // Create registry with placeholder PID, merging any persisted projects (R4)
    let mut registry = Registry::new(0, port);
    registry.merge_persisted_projects();
    registry.save()?;

    let mut cmd = Command::new(&exe_path);
    cmd.args(["server", "run", "--port", &port.to_string()]);

    // Always redirect stdout/stderr to log file — child is a long-running
    // background server regardless of --daemon flag
    #[cfg(unix)]
    {
        let log_file = get_log_file()?;
        let log_file_err = log_file.try_clone()?;
        cmd.stdin(Stdio::null())
            .stdout(Stdio::from(log_file))
            .stderr(Stdio::from(log_file_err));
    }

    let child = cmd.spawn()?;
    let pid = child.id();

    // Update registry with actual PID
    let mut registry = Registry::load()?;
    registry.server.pid = pid;
    registry.save()?;

    // Wait for server to start
    std::thread::sleep(std::time::Duration::from_millis(500));

    println!("{}", format!("Server started (PID: {})", pid).green());
    Ok(registry)
}

/// Run the HTTP server (called by daemon process)
async fn run_server_daemon(port: u16) -> Result<()> {
    let registry = Registry::load()?;
    crate::http_server::start_server(port, registry).await?;
    Ok(())
}

/// Register a project
fn register_project(path: Option<PathBuf>) -> Result<()> {
    let mut registry = Registry::load().map_err(|_| {
        anyhow::anyhow!("No server running. Use 'cc server start' to start the server first.")
    })?;

    let project_path = match path {
        Some(p) => p.canonicalize()?,
        None => env::current_dir()?,
    };

    if !crate::registry::is_valid_project(&project_path) {
        anyhow::bail!(
            "Not a valid project directory: {}\nMust contain pyproject.toml, Cargo.toml, package.json, or cclab/",
            project_path.display()
        );
    }

    let project_name = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Cannot determine project name"))?
        .to_string();

    if registry.get_project_path(&project_name).is_some() {
        println!(
            "{}",
            format!("Project '{}' already registered", project_name).yellow()
        );
        return Ok(());
    }

    registry.register_project(project_name.clone(), project_path.clone())?;
    println!(
        "{}",
        format!("Registered project: {}", project_name).green()
    );
    println!("  Path: {}", project_path.display());

    Ok(())
}

/// Unregister a project
fn unregister_project(project: Option<String>) -> Result<()> {
    let mut registry = Registry::load().map_err(|_| anyhow::anyhow!("No server running."))?;

    let project_name = if let Some(name) = project {
        name
    } else {
        let current_dir = env::current_dir()?;
        current_dir
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Cannot determine project name"))?
            .to_string()
    };

    if registry.get_project_path(&project_name).is_none() {
        println!(
            "{}",
            format!("Project '{}' not registered", project_name).yellow()
        );
        return Ok(());
    }

    registry.unregister_project(&project_name)?;
    println!(
        "{}",
        format!("Unregistered project: {}", project_name).green()
    );

    Ok(())
}

/// List server status and registered projects
fn list_status() -> Result<()> {
    let registry = Registry::load().map_err(|_| {
        anyhow::anyhow!("No server running. Use 'cc server start' to start the server.")
    })?;

    println!("\n{}", "Server Status".bold());
    println!("{}", "=".repeat(50));
    println!("  Port:      {}", registry.server.port);
    println!("  PID:       {}", registry.server.pid);
    println!(
        "  Started:   {}",
        registry.server.started_at.format("%Y-%m-%d %H:%M:%S")
    );
    println!("  Uptime:    {}", registry.server_uptime());
    println!(
        "  Status:    {}",
        if registry.is_server_running() {
            "Running".green()
        } else {
            "Dead".red()
        }
    );
    if registry.is_server_outdated() {
        println!(
            "  {}",
            "Binary updated since server start - restart recommended".yellow()
        );
    }
    println!("  Dashboard: http://localhost:{}/", registry.server.port);
    println!("  MCP:       http://localhost:{}/mcp", registry.server.port);
    if let Some(home) = dirs::home_dir() {
        println!("  Log:       {}", home.join(".cclab/server.log").display());
    }

    println!("\n{}", "Registered Projects".bold());
    println!("{}", "=".repeat(50));

    if registry.projects.is_empty() {
        println!("  {}", "No projects registered".yellow());
        println!("  Run 'cc server register' in a project directory to add it.");
    } else {
        for (name, info) in registry.list_projects() {
            println!("  {} {}", "*".green(), name.bold());
            println!("    Path: {}", info.path.display());
            println!(
                "    Registered: {}",
                info.registered_at.format("%Y-%m-%d %H:%M:%S")
            );
        }
    }

    println!();
    Ok(())
}

/// Open Plan Viewer for a specific change
fn view_change(project: &str, change: &str) -> Result<()> {
    let registry = Registry::load().map_err(|_| {
        anyhow::anyhow!("No server running. Use 'cc server start' to start the server first.")
    })?;

    if !registry.is_server_running() {
        anyhow::bail!("Server is not running. Use 'cc server start' to start it first.");
    }

    let project_path = registry
        .get_project_path(project)
        .ok_or_else(|| anyhow::anyhow!("Project '{}' is not registered", project))?;

    let change_dir = project_path.join(".aw/changes").join(change);
    if !change_dir.exists() {
        anyhow::bail!(
            "Change '{}' not found in project '{}'\nPath: {}",
            change,
            project,
            change_dir.display()
        );
    }

    let url = format!(
        "http://localhost:{}/view/{}/{}/",
        registry.server.port, project, change
    );

    println!("{}", format!("Opening: {}", url).cyan());

    #[cfg(feature = "ui")]
    {
        if let Err(e) = open::that(&url) {
            eprintln!(
                "Failed to open browser: {}. Please open {} manually.",
                e, url
            );
        }
    }

    #[cfg(not(feature = "ui"))]
    {
        println!("Open this URL in your browser: {}", url);
    }

    Ok(())
}

/// Shutdown the server
fn shutdown_server() -> Result<()> {
    let registry = Registry::load().map_err(|_| anyhow::anyhow!("No server running."))?;

    let pid = registry.server.pid;

    #[cfg(unix)]
    {
        Command::new("kill").arg(pid.to_string()).output()?;
    }

    #[cfg(windows)]
    {
        Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .output()?;
    }

    // Clear server info but preserve projects for next startup (R1)
    Registry::clear_server_info()?;

    println!("{}", format!("Server shut down (PID: {})", pid).green());

    Ok(())
}

/// Update client configurations
fn update_client_configs(registry: &Registry) -> Result<()> {
    println!("\n{}", "Updating client configurations...".cyan());

    // Update .gemini/settings.json for all registered projects
    for (name, info) in registry.list_projects() {
        update_gemini_config(&name, &info.path, registry.server.port)?;
    }

    // Update ~/.codex/config.toml
    update_codex_config(registry.server.port)?;

    println!("{}", "Client configurations updated".green());
    Ok(())
}

/// Update Gemini CLI configuration for a project
fn update_gemini_config(
    project_name: &str,
    project_path: &std::path::Path,
    port: u16,
) -> Result<()> {
    use serde_json::{json, Value};
    use std::fs;

    let gemini_dir = project_path.join(".gemini");
    fs::create_dir_all(&gemini_dir)?;

    let settings_file = gemini_dir.join("settings.json");

    let mut settings: Value = if settings_file.exists() {
        let content = fs::read_to_string(&settings_file)?;
        serde_json::from_str(&content)?
    } else {
        json!({})
    };

    if settings.get("mcpServers").is_none() {
        settings["mcpServers"] = json!({});
    }

    settings["mcpServers"]["cclab"] = json!({
        "url": format!("http://localhost:{}/mcp", port),
        "headers": {
            "X-Project": project_name,
            "X-Project-Path": project_path.to_str().unwrap()
        },
        "timeout": 30000
    });

    let content = serde_json::to_string_pretty(&settings)?;
    fs::write(&settings_file, content)?;

    println!("  Updated {}", settings_file.display());
    Ok(())
}

/// Update Codex configuration
fn update_codex_config(port: u16) -> Result<()> {
    use std::fs;

    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;

    let codex_dir = home.join(".codex");
    fs::create_dir_all(&codex_dir)?;

    let config_file = codex_dir.join("config.toml");

    let mut config_content = if config_file.exists() {
        fs::read_to_string(&config_file)?
    } else {
        String::new()
    };

    let server_config = format!(
        r#"
[mcp_servers.cclab]
url = "http://localhost:{}/mcp"
timeout = 30000
"#,
        port
    );

    if !config_content.contains("[mcp_servers.cclab]") {
        config_content.push_str(&server_config);
        fs::write(&config_file, config_content)?;
        println!("  Updated {}", config_file.display());
    } else {
        println!("  Codex config already has cclab server, skipping");
    }

    Ok(())
}
