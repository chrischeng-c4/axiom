// HANDWRITE-BEGIN gap="missing-generator:contract:ff9d0c0a" tracker="pending-tracker" reason="Implement the versioned registry and line-delimited request/response contract, typed JSON result envelopes, bounded PNG validation, and atomic caller-directed writes."
//! Local, read-only Workbench observability CLI.

use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    path::{Path, PathBuf},
    process,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};

const PROTOCOL_VERSION: u32 = 1;
const DEFAULT_LOG_LINES: usize = 100;
const MAX_LOG_LINES: usize = 1_000;
const MAX_RESPONSE_BYTES: usize = 16 * 1024 * 1024;
const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

#[derive(Debug)]
pub struct CliError {
    pub code: &'static str,
    pub message: String,
    pub next: &'static str,
}

impl CliError {
    fn new(code: &'static str, message: impl Into<String>, next: &'static str) -> Self {
        Self { code, message: message.into(), next }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CliResult {
    Snapshot { instance_id: String, path: String, bytes: usize },
    Logs { path: String, lines: Vec<String>, truncated: bool },
}

#[derive(Debug, Deserialize)]
struct RuntimeRegistry {
    #[serde(rename = "protocolVersion")]
    protocol_version: u32,
    #[serde(rename = "instanceId")]
    instance_id: String,
    pid: u32,
    port: u16,
    token: String,
}

#[derive(Serialize)]
struct RuntimeRequest<'a> {
    #[serde(rename = "protocolVersion")]
    protocol_version: u32,
    #[serde(rename = "requestId")]
    request_id: u64,
    token: &'a str,
    method: &'a str,
}

#[derive(Deserialize)]
struct RuntimeResponse {
    #[serde(rename = "protocolVersion")]
    protocol_version: u32,
    #[serde(rename = "requestId")]
    request_id: u64,
    #[serde(rename = "instanceId")]
    instance_id: String,
    ok: bool,
    #[serde(rename = "mimeType")]
    mime_type: Option<String>,
    #[serde(rename = "dataBase64")]
    data_base64: Option<String>,
    error: Option<String>,
}

/// Run only explicit read-only Workbench CLI commands. `None` means launch the desktop host.
pub fn run_if_requested(args: &[String]) -> Option<i32> {
    if args.len() < 2 || !matches!(args[1].as_str(), "snapshot" | "logs") {
        return None;
    }

    let result = run_with_paths(&args[1..], &runtime_root(), &diagnostic_log_path());
    match result {
        Ok(result) => {
            println!("{}", serde_json::to_string(&result).expect("CLI results serialize"));
            Some(0)
        }
        Err(error) => {
            eprintln!(
                "{}",
                serde_json::json!({
                    "kind": "error",
                    "code": error.code,
                    "message": error.message,
                    "next": error.next,
                })
            );
            Some(2)
        }
    }
}

pub fn run_with_paths(args: &[String], runtime_root: &Path, log_path: &Path) -> Result<CliResult, CliError> {
    match args {
        [command, rest @ ..] if command == "logs" => logs(rest, log_path),
        [command, rest @ ..] if command == "snapshot" => snapshot(rest, runtime_root),
        _ => Err(invalid_arguments("expected `workbench snapshot --out <png-path>` or `workbench logs [--tail <count>]`")),
    }
}

fn logs(args: &[String], log_path: &Path) -> Result<CliResult, CliError> {
    let tail = match args {
        [] => DEFAULT_LOG_LINES,
        [flag, value] if flag == "--tail" => value.parse::<usize>().ok().filter(|count| *count > 0)
            .ok_or_else(|| invalid_arguments("--tail must be a positive integer"))?,
        _ => return Err(invalid_arguments("logs accepts only an optional `--tail <count>`")),
    }.min(MAX_LOG_LINES);

    let text = match fs::read_to_string(log_path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(CliError::new("log_unavailable", format!("cannot read {}: {error}", log_path.display()), "workbench logs --tail 100")),
    };
    let all_lines = text.lines().map(str::to_owned).collect::<Vec<_>>();
    let start = all_lines.len().saturating_sub(tail);
    Ok(CliResult::Logs {
        path: log_path.display().to_string(),
        truncated: start > 0,
        lines: all_lines[start..].to_vec(),
    })
}

fn snapshot(args: &[String], runtime_root: &Path) -> Result<CliResult, CliError> {
    let output_path = match args {
        [flag, path] if flag == "--out" && !path.is_empty() => PathBuf::from(path),
        _ => return Err(invalid_arguments("snapshot requires exactly `--out <png-path>`")),
    };
    let parent = output_path.parent().filter(|path| path.is_dir()).ok_or_else(|| {
        CliError::new("output_write_failed", "snapshot output parent does not exist", "mkdir -p <output-parent> && workbench snapshot --out <png-path>")
    })?;
    if parent.as_os_str().is_empty() {
        return Err(CliError::new("output_write_failed", "snapshot output requires a parent directory", "workbench snapshot --out /tmp/workbench.png"));
    }

    let registry = read_registry(runtime_root)?;
    let request_id = next_request_id();
    let response = request_snapshot(&registry, request_id)?;
    if response.protocol_version != PROTOCOL_VERSION {
        return Err(CliError::new("runtime_protocol_mismatch", "runtime returned an unsupported protocol version", "rebuild and reopen Workbench"));
    }
    if response.request_id != request_id || response.instance_id != registry.instance_id {
        return Err(CliError::new("runtime_protocol_mismatch", "runtime response identity did not match the request", "reopen Workbench and retry snapshot"));
    }
    if !response.ok {
        return Err(CliError::new("snapshot_failed", response.error.unwrap_or_else(|| "runtime rejected snapshot request".into()), "reopen Workbench and retry snapshot"));
    }
    if response.mime_type.as_deref() != Some("image/png") {
        return Err(CliError::new("snapshot_failed", "runtime response was not image/png", "reopen Workbench and retry snapshot"));
    }
    let data = BASE64.decode(response.data_base64.unwrap_or_default()).map_err(|_| CliError::new("snapshot_failed", "runtime returned invalid PNG encoding", "reopen Workbench and retry snapshot"))?;
    if data.len() > MAX_RESPONSE_BYTES || !data.starts_with(&PNG_SIGNATURE) {
        return Err(CliError::new("snapshot_failed", "runtime returned an invalid or oversized PNG", "reopen Workbench and retry snapshot"));
    }
    atomic_write(&output_path, &data)?;
    Ok(CliResult::Snapshot { instance_id: registry.instance_id, path: output_path.display().to_string(), bytes: data.len() })
}

fn read_registry(runtime_root: &Path) -> Result<RuntimeRegistry, CliError> {
    let path = runtime_root.join("current.json");
    let bytes = fs::read(&path).map_err(|error| CliError::new("runtime_unavailable", format!("Workbench runtime registry is unavailable: {error}"), "open Workbench and retry `workbench snapshot --out <png-path>`"))?;
    let registry: RuntimeRegistry = serde_json::from_slice(&bytes).map_err(|_| CliError::new("runtime_unavailable", "Workbench runtime registry is malformed", "reopen Workbench and retry snapshot"))?;
    if registry.protocol_version != PROTOCOL_VERSION || registry.instance_id.is_empty() || registry.token.len() < 16 || registry.port < 1024 || registry.pid == 0 {
        return Err(CliError::new("runtime_unavailable", "Workbench runtime registry has invalid fields", "reopen Workbench and retry snapshot"));
    }
    Ok(registry)
}

fn request_snapshot(registry: &RuntimeRegistry, request_id: u64) -> Result<RuntimeResponse, CliError> {
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), registry.port);
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(2)).map_err(|_| CliError::new("runtime_unavailable", "Workbench runtime is not reachable", "open Workbench and retry `workbench snapshot --out <png-path>`"))?;
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(2))).ok();
    let request = RuntimeRequest { protocol_version: PROTOCOL_VERSION, request_id, token: &registry.token, method: "snapshot" };
    serde_json::to_writer(&mut stream, &request).map_err(|error| CliError::new("snapshot_failed", format!("cannot encode snapshot request: {error}"), "reopen Workbench and retry snapshot"))?;
    stream.write_all(b"\n").map_err(|_| CliError::new("runtime_unavailable", "Workbench runtime closed the request", "reopen Workbench and retry snapshot"))?;
    let mut bytes = Vec::new();
    BufReader::new(stream).take((MAX_RESPONSE_BYTES + 4096) as u64).read_until(b'\n', &mut bytes).map_err(|_| CliError::new("runtime_unavailable", "Workbench runtime response timed out", "reopen Workbench and retry snapshot"))?;
    if bytes.len() > MAX_RESPONSE_BYTES + 4096 {
        return Err(CliError::new("snapshot_failed", "Workbench runtime response exceeded its bound", "reopen Workbench and retry snapshot"));
    }
    serde_json::from_slice(&bytes).map_err(|_| CliError::new("runtime_protocol_mismatch", "Workbench runtime returned malformed JSON", "rebuild and reopen Workbench"))
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), CliError> {
    let nonce = next_request_id();
    let temporary = path.with_extension(format!("workbench-{nonce}.tmp"));
    let result = (|| -> std::io::Result<()> {
        let mut file = OpenOptions::new().write(true).create_new(true).open(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result.map_err(|error| CliError::new("output_write_failed", format!("cannot write {}: {error}", path.display()), "choose a writable output path and retry snapshot"))
}

fn runtime_root() -> PathBuf {
    home_directory().join(".axiom-workbench/runtime")
}

fn diagnostic_log_path() -> PathBuf {
    home_directory().join(".axiom-workbench/logs/workbench.log")
}

fn home_directory() -> PathBuf {
    env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/tmp"))
}

fn next_request_id() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos() as u64 ^ process::id() as u64
}

fn invalid_arguments(message: impl Into<String>) -> CliError {
    CliError::new("invalid_arguments", message, "workbench snapshot --out /tmp/workbench.png")
}

<!-- marker: missing-generator:contract:ff9d0c0a path: apps/workbench/src/observability_cli.rs reason: Implement the versioned registry and line-delimited request/response contract, typed JSON result envelopes, bounded PNG validation, and atomic caller-directed writes. -->
// HANDWRITE-END
