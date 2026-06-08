// HANDWRITE-BEGIN gap="missing-generator:hand-written:f07dd5c5" tracker="2087" reason="CLI dispatch — daemon {start,stop,restart,status} + register/unregister/projects + sync + query {coverage,impact}."
mod discovery;
mod output;

use std::io::{BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use cgdb_core::rpc::{read_message, write_message, RpcRequest, RpcResponse};
use clap::{Parser, Subcommand};
use serde_json::{json, Value};

#[derive(Parser)]
#[command(name = "cgdb", about = "cgdb — local graph daemon CLI")]
struct Cli {
    #[arg(long, env = "CGDB_SOCKET")]
    socket: Option<PathBuf>,
    #[arg(long, env = "CGDB_PORT")]
    port: Option<u16>,
    #[arg(long, default_value = "json")]
    format: OutputFormat,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum OutputFormat {
    Json,
    Mermaid,
}

#[derive(Subcommand)]
enum Cmd {
    Daemon {
        #[command(subcommand)]
        cmd: DaemonCmd,
    },
    Register {
        project: Option<String>,
        #[arg(long)]
        all: bool,
    },
    Unregister {
        project: String,
    },
    Projects,
    Sync {
        project: String,
        #[arg(long)]
        rebuild: bool,
    },
    Query {
        #[command(subcommand)]
        cmd: QueryCmd,
    },
    Lens {
        #[command(subcommand)]
        cmd: LensCmd,
    },
}

#[derive(Subcommand)]
enum LensCmd {
    Overview { project: String },
    ZoomIn { project: String, node_id: String },
    ZoomOut { project: String, node_id: String },
    Focus {
        project: String,
        node_id: String,
        #[arg(long, default_value_t = 1)]
        depth: u8,
        #[arg(long)]
        include_semantic: bool,
    },
    Breadcrumb { project: String, node_id: String },
}

#[derive(Subcommand)]
enum DaemonCmd {
    Start {
        #[arg(long, default_value_t = 5455)]
        port: u16,
        #[arg(long)]
        tcp: bool,
        #[arg(long)]
        foreground: bool,
    },
    Stop,
    Restart,
    Status,
}

#[derive(Subcommand)]
enum QueryCmd {
    Coverage { project: String },
    Impact { project: String, spec_section: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let Cli { socket, port: _, format, cmd } = cli;
    let ctx = ConnCtx { socket };
    match cmd {
        Cmd::Daemon { cmd } => match cmd {
            DaemonCmd::Start { port, tcp, foreground } => daemon_start(port, tcp, foreground),
            DaemonCmd::Stop => daemon_stop(),
            DaemonCmd::Restart => {
                let _ = daemon_stop();
                daemon_start(5455, false, false)
            }
            DaemonCmd::Status => rpc_call(&ctx, "daemon.status", json!({}), format),
        },
        Cmd::Register { project, all } => {
            let repo_root = find_repo_root()?;
            if all {
                let projects = list_score_projects(&repo_root)?;
                for name in projects {
                    rpc_call_only(
                        &ctx,
                        "project.register",
                        json!({ "name": name, "repo_root": repo_root.to_string_lossy() }),
                    )?;
                }
                Ok(())
            } else {
                let name = project.ok_or_else(|| anyhow!("project name required"))?;
                rpc_call(
                    &ctx,
                    "project.register",
                    json!({ "name": name, "repo_root": repo_root.to_string_lossy() }),
                    format,
                )
            }
        }
        Cmd::Unregister { project } => {
            rpc_call(&ctx, "project.unregister", json!({ "name": project }), format)
        }
        Cmd::Projects => rpc_call(&ctx, "project.list", json!({}), format),
        Cmd::Sync { project, rebuild } => rpc_call(
            &ctx,
            "project.sync",
            json!({ "name": project, "rebuild": rebuild }),
            format,
        ),
        Cmd::Query { cmd } => match cmd {
            QueryCmd::Coverage { project } => rpc_call(
                &ctx,
                "query.coverage",
                json!({ "name": project }),
                format,
            ),
            QueryCmd::Impact { project, spec_section } => rpc_call(
                &ctx,
                "query.impact",
                json!({ "name": project, "spec_section": spec_section }),
                format,
            ),
        },
        Cmd::Lens { cmd } => {
            let fmt_str = match format {
                OutputFormat::Json => "json",
                OutputFormat::Mermaid => "mermaid",
            };
            match cmd {
                LensCmd::Overview { project } => rpc_call(
                    &ctx,
                    "lens.overview",
                    json!({ "project": project, "format": fmt_str }),
                    format,
                ),
                LensCmd::ZoomIn { project, node_id } => rpc_call(
                    &ctx,
                    "lens.zoom_in",
                    json!({ "project": project, "node_id": node_id, "format": fmt_str }),
                    format,
                ),
                LensCmd::ZoomOut { project, node_id } => rpc_call(
                    &ctx,
                    "lens.zoom_out",
                    json!({ "project": project, "node_id": node_id, "format": fmt_str }),
                    format,
                ),
                LensCmd::Focus { project, node_id, depth, include_semantic } => rpc_call(
                    &ctx,
                    "lens.focus",
                    json!({
                        "project": project,
                        "node_id": node_id,
                        "depth": depth,
                        "include_semantic": include_semantic,
                        "format": fmt_str,
                    }),
                    format,
                ),
                LensCmd::Breadcrumb { project, node_id } => rpc_call(
                    &ctx,
                    "lens.breadcrumb",
                    json!({ "project": project, "node_id": node_id, "format": fmt_str }),
                    format,
                ),
            }
        }
    }
}

struct ConnCtx {
    socket: Option<PathBuf>,
}

fn daemon_start(port: u16, tcp: bool, foreground: bool) -> Result<()> {
    let bin = std::env::current_exe()?
        .parent()
        .ok_or_else(|| anyhow!("no parent dir for cgdb binary"))?
        .join("cgdb-daemon");
    if !bin.exists() {
        return Err(anyhow!("daemon binary not found at {}", bin.display()));
    }
    let mut cmd = Command::new(&bin);
    cmd.arg("--port").arg(port.to_string());
    if tcp {
        cmd.arg("--tcp");
    }
    if foreground {
        let status = cmd.arg("--foreground").status()?;
        std::process::exit(status.code().unwrap_or(0));
    } else {
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());
        let child = cmd.spawn().with_context(|| "spawn daemon")?;
        // give the daemon a moment to bind
        for _ in 0..40 {
            if discovery::sock_path().exists() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        println!("cgdb-daemon spawned (pid {})", child.id());
        Ok(())
    }
}

fn daemon_stop() -> Result<()> {
    let pid_path = discovery::pid_path();
    if !pid_path.exists() {
        return Err(anyhow!("no daemon.pid at {}", pid_path.display()));
    }
    let pid: i32 = std::fs::read_to_string(&pid_path)?.trim().parse()?;
    // SIGTERM via libc-free std: write to /proc/<pid>/... won't work cross-platform.
    // Use the standard `kill` command for portability.
    let status = Command::new("kill").arg(pid.to_string()).status()?;
    if !status.success() {
        return Err(anyhow!("kill {} failed", pid));
    }
    Ok(())
}

fn rpc_call(ctx: &ConnCtx, method: &str, params: Value, format: OutputFormat) -> Result<()> {
    let resp = send(ctx, method, params)?;
    output::render(method, &resp, format)
}

fn rpc_call_only(ctx: &ConnCtx, method: &str, params: Value) -> Result<()> {
    let _ = send(ctx, method, params)?;
    Ok(())
}

fn send(ctx: &ConnCtx, method: &str, params: Value) -> Result<RpcResponse> {
    let sock_path = ctx.socket.clone().unwrap_or_else(discovery::sock_path);
    let stream = UnixStream::connect(&sock_path)
        .with_context(|| format!("connect UDS {}", sock_path.display()))?;
    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);
    let req = RpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(1),
        method: method.into(),
        params,
    };
    let body = serde_json::to_vec(&req)?;
    write_message(&mut writer, &body)?;
    writer.flush()?;
    let raw = read_message(&mut reader)?
        .ok_or_else(|| anyhow!("daemon closed connection without response"))?;
    let resp: RpcResponse = serde_json::from_slice(&raw)?;
    if let Some(err) = &resp.error {
        return Err(anyhow!("{}", err.message));
    }
    Ok(resp)
}

fn find_repo_root() -> Result<PathBuf> {
    let mut cur = std::env::current_dir()?;
    loop {
        if cur.join(".git").exists() {
            return Ok(cur);
        }
        if !cur.pop() {
            return Err(anyhow!("not inside a git repo"));
        }
    }
}

fn list_score_projects(repo_root: &std::path::Path) -> Result<Vec<String>> {
    let cfg_path = repo_root.join(".score").join("config.toml");
    let body = std::fs::read_to_string(&cfg_path)?;
    let cfg: toml::Value = toml::from_str(&body)?;
    let mut out = Vec::new();
    if let Some(arr) = cfg.get("projects").and_then(|v| v.as_array()) {
        for p in arr {
            if let Some(n) = p.get("name").and_then(|v| v.as_str()) {
                out.push(n.to_string());
            }
        }
    }
    Ok(out)
}
// HANDWRITE-END
// SPEC-MANAGED: .score/tech_design/projects/cgdb/specs/cgdb-v0-1.md#changes
