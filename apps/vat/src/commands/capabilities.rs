// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-commands-capabilities-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! `vat capabilities` — host/backend capability evidence for agents and CI.

use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};
use std::time::{Duration, Instant};

use anyhow::Result;
use serde::Serialize;

use crate::sandbox;

const DOCKER_TIMEOUT: Duration = Duration::from_millis(900);

#[derive(Debug, Clone, Serialize)]
pub struct CapabilitiesReport {
    pub host: HostInfo,
    pub workspace: WorkspaceCapabilities,
    pub isolation: Vec<IsolationCapability>,
    pub docker: DockerCapability,
    pub services: ServiceCapabilities,
}

#[derive(Debug, Clone, Serialize)]
pub struct HostInfo {
    pub os: String,
    pub arch: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceCapabilities {
    pub cow_clone: bool,
    pub primary_clone_method: String,
    pub fallback_clone_method: String,
    pub diff_basis: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IsolationCapability {
    pub id: String,
    pub implemented: bool,
    pub available: bool,
    pub gpu_native: bool,
    pub write_confinement: bool,
    pub network_egress: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DockerCapability {
    pub cli: bool,
    pub daemon: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceCapabilities {
    pub builtin_emulators: Vec<String>,
    pub external_attach: bool,
    pub docker_services: bool,
    pub native_preset_services: bool,
}

pub fn exec(json: bool) -> Result<ExitCode> {
    let report = report();
    if json {
        crate::commands::print_json(&report, false)?;
    } else {
        print_human(&report);
    }
    Ok(ExitCode::SUCCESS)
}

pub fn report() -> CapabilitiesReport {
    let docker = docker_capability();
    CapabilitiesReport {
        host: HostInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
        },
        workspace: workspace_capabilities(),
        isolation: isolation_capabilities(),
        services: ServiceCapabilities {
            builtin_emulators: vec![
                "pubsub".to_string(),
                "firebase-auth".to_string(),
                "cloud-tasks".to_string(),
                "cloud-scheduler".to_string(),
                "cloud-workflows".to_string(),
                "cloud-storage".to_string(),
                "http-mock".to_string(),
                "openapi".to_string(),
            ],
            external_attach: true,
            docker_services: docker.daemon,
            native_preset_services: true,
        },
        docker,
    }
}

pub fn isolation_available(report: &CapabilitiesReport, id: &str) -> bool {
    report
        .isolation
        .iter()
        .any(|capability| capability.id == id && capability.available)
}

fn workspace_capabilities() -> WorkspaceCapabilities {
    if cfg!(target_os = "macos") {
        WorkspaceCapabilities {
            cow_clone: true,
            primary_clone_method: "apfs_clonefile".to_string(),
            fallback_clone_method: "recursive_copy".to_string(),
            diff_basis: "size_mtime_manifest".to_string(),
        }
    } else if cfg!(target_os = "linux") {
        WorkspaceCapabilities {
            cow_clone: true,
            primary_clone_method: "cp_reflink_auto".to_string(),
            fallback_clone_method: "recursive_copy".to_string(),
            diff_basis: "size_mtime_manifest".to_string(),
        }
    } else {
        WorkspaceCapabilities {
            cow_clone: false,
            primary_clone_method: "recursive_copy".to_string(),
            fallback_clone_method: "recursive_copy".to_string(),
            diff_basis: "size_mtime_manifest".to_string(),
        }
    }
}

fn isolation_capabilities() -> Vec<IsolationCapability> {
    let mut capabilities = vec![IsolationCapability {
        id: "process".to_string(),
        implemented: true,
        available: true,
        gpu_native: true,
        write_confinement: false,
        network_egress: false,
        reason: "host process with copy-on-write workspace only".to_string(),
    }];

    let seatbelt_available = cfg!(target_os = "macos") && sandbox::seatbelt::available();
    capabilities.push(IsolationCapability {
        id: "macos-seatbelt".to_string(),
        implemented: cfg!(target_os = "macos"),
        available: seatbelt_available,
        gpu_native: true,
        write_confinement: seatbelt_available,
        network_egress: seatbelt_available,
        reason: if seatbelt_available {
            "sandbox-exec available: write confinement and egress policy enforceable".to_string()
        } else if cfg!(target_os = "macos") {
            "sandbox-exec is not available on PATH".to_string()
        } else {
            "macOS-only backend".to_string()
        },
    });

    let linux_tooling = which("unshare").is_some() || which("bwrap").is_some();
    capabilities.push(IsolationCapability {
        id: "linux-netns".to_string(),
        implemented: false,
        available: false,
        gpu_native: true,
        write_confinement: false,
        network_egress: false,
        reason: if cfg!(target_os = "linux") && linux_tooling {
            "host has namespace tooling, but vat linux-netns backend is not implemented yet"
                .to_string()
        } else if cfg!(target_os = "linux") {
            "vat linux-netns backend is not implemented yet".to_string()
        } else {
            "Linux-only planned backend".to_string()
        },
    });

    capabilities.push(IsolationCapability {
        id: "vm".to_string(),
        implemented: false,
        available: false,
        gpu_native: false,
        write_confinement: true,
        network_egress: true,
        reason: "out of scope unless a run explicitly trades native GPU access for VM isolation"
            .to_string(),
    });

    capabilities
}

fn docker_capability() -> DockerCapability {
    let cli = which("docker").is_some();
    if !cli {
        return DockerCapability {
            cli: false,
            daemon: false,
            context: None,
            provider: None,
            server_version: None,
            error: Some("docker CLI not found on PATH".to_string()),
        };
    }

    let context = command_stdout_timeout("docker", &["context", "show"], DOCKER_TIMEOUT)
        .ok()
        .filter(|value| !value.is_empty());
    let provider = context.as_deref().and_then(provider_from_context);
    match command_stdout_timeout(
        "docker",
        &["version", "--format", "{{.Server.Version}}"],
        DOCKER_TIMEOUT,
    ) {
        Ok(version) => DockerCapability {
            cli,
            daemon: true,
            context,
            provider,
            server_version: if version.is_empty() {
                None
            } else {
                Some(version)
            },
            error: None,
        },
        Err(error) => DockerCapability {
            cli,
            daemon: false,
            context,
            provider,
            server_version: None,
            error: Some(error),
        },
    }
}

fn provider_from_context(context: &str) -> Option<String> {
    let lower = context.to_ascii_lowercase();
    if lower.contains("orbstack") {
        Some("orbstack".to_string())
    } else if lower.contains("desktop") {
        Some("docker-desktop".to_string())
    } else if lower.contains("colima") {
        Some("colima".to_string())
    } else {
        None
    }
}

fn command_stdout_timeout(
    program: &str,
    args: &[&str],
    timeout: Duration,
) -> Result<String, String> {
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| format!("spawn {program}: {err}"))?;
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                let output = child
                    .wait_with_output()
                    .map_err(|err| format!("wait {program}: {err}"))?;
                if output.status.success() {
                    return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
                }
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                if stderr.is_empty() {
                    return Err(format!("{program} exited with {}", output.status));
                }
                return Err(stderr);
            }
            Ok(None) if start.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "{program} timed out after {}ms",
                    timeout.as_millis()
                ));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(err) => return Err(format!("wait {program}: {err}")),
        }
    }
}

fn which(bin: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(bin))
        .find(|candidate| candidate.is_file())
}

fn print_human(report: &CapabilitiesReport) {
    println!("host {} {}", report.host.os, report.host.arch);
    println!(
        "workspace cow={} primary={} fallback={}",
        report.workspace.cow_clone,
        report.workspace.primary_clone_method,
        report.workspace.fallback_clone_method
    );
    for capability in &report.isolation {
        println!(
            "isolation {} available={} implemented={} gpu_native={} network_egress={}",
            capability.id,
            capability.available,
            capability.implemented,
            capability.gpu_native,
            capability.network_egress
        );
    }
    println!(
        "docker cli={} daemon={} context={}",
        report.docker.cli,
        report.docker.daemon,
        report.docker.context.as_deref().unwrap_or("unknown")
    );
    println!(
        "services builtin_emulators={} external_attach={} docker_services={}",
        report.services.builtin_emulators.join(","),
        report.services.external_attach,
        report.services.docker_services
    );
}
// CODEGEN-END
