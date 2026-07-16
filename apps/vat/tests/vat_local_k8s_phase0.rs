// HANDWRITE-BEGIN gap="missing-generator:e2e-test:afed504d" tracker="#1539" reason="Phase 0 has opt-in real-host probes. The durable systemd-machine control preflights the source-backed fixture, owns one temporary machine, records exact evidence, requires stop-to-run re-execution, and cleans up explicitly and through Drop. The separate disposable probe follows only the inspect-returned backing container ID, verifies root/systemd, a Ready k3s node, and a completed Job, then removes the job and machine. Its second host-API opt-in copies a short-lived 0600 kubeconfig plus owned kubectl cache, reads nodes from macOS through the exact inspected IP, and removes credentials before machine cleanup. An ephemeral pass never substitutes for durable control or remaining Phase 0 journeys."

//! Real-host gates for the Apple-container Local Kubernetes feasibility spike.
//!
//! This first gate proves that the host can create a persistent *systemd*
//! machine and execute one command through the machine API before Phase 0
//! attempts k3s. Apple Container machines boot the image init system, so a
//! minimal application image is not a valid k3s substrate control. A failed
//! control is a meaningful NO-GO for the persistent backend: it writes
//! evidence and fails the opt-in test rather than letting a disposable guest
//! observation obscure the restart requirement. A separate opt-in diagnostic
//! below can prove a one-boot k3s node through `container exec`, but that is
//! not a completed Kubernetes product proof.

use serde::Serialize;
use std::fs;
use std::net::IpAddr;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::str::FromStr;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tempfile::TempDir;

const E2E_OPT_IN: &str = "VAT_LOCAL_K8S_E2E";
const DISPOSABLE_E2E_OPT_IN: &str = "VAT_LOCAL_K8S_DISPOSABLE_E2E";
const HOST_API_E2E_OPT_IN: &str = "VAT_LOCAL_K8S_HOST_API_E2E";
const MACHINE_IMAGE_ENV: &str = "VAT_LOCAL_K8S_MACHINE_IMAGE";
const DEFAULT_MACHINE_IMAGE: &str = "local/vat-k8s-systemd:phase0";
const MACHINE_COMMAND_TIMEOUT: Duration = Duration::from_secs(90);
const MACHINE_CLEANUP_TIMEOUT: Duration = Duration::from_secs(45);
const MACHINE_CLEANUP_COMMAND_TIMEOUT: Duration = Duration::from_secs(15);
const MACHINE_CREATE_FAILURE_STABILIZATION: Duration = Duration::from_secs(20);
const MACHINE_CLEANUP_RETRY_DELAY: Duration = Duration::from_secs(1);
const K3S_INSTALL_TIMEOUT: Duration = Duration::from_secs(300);
const K3S_READY_TIMEOUT: Duration = Duration::from_secs(240);
const HOST_KUBECTL_TIMEOUT: Duration = Duration::from_secs(60);
const K3S_KUBECONFIG_PATH: &str = "/etc/rancher/k3s/k3s.yaml";

#[derive(Debug, Clone, Serialize)]
struct CommandEvidence {
    label: String,
    argv: Vec<String>,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl CommandEvidence {
    fn passed(&self) -> bool {
        self.exit_code == Some(0)
    }

    fn not_run(label: &str, args: &[&str], reason: impl Into<String>) -> Self {
        Self::not_run_for("container", label, args, reason)
    }

    fn not_run_for(program: &str, label: &str, args: &[&str], reason: impl Into<String>) -> Self {
        Self {
            label: label.to_string(),
            argv: std::iter::once(program.to_string())
                .chain(args.iter().map(|arg| (*arg).to_string()))
                .collect(),
            exit_code: None,
            stdout: String::new(),
            stderr: format!("not run: {}", reason.into()),
        }
    }
}

#[derive(Debug, Serialize)]
struct LocalEvidence {
    label: String,
    passed: bool,
    detail: String,
}

impl LocalEvidence {
    fn not_run(label: &str, reason: impl Into<String>) -> Self {
        Self {
            label: label.to_string(),
            passed: false,
            detail: format!("not run: {}", reason.into()),
        }
    }

    fn passed(label: &str, detail: impl Into<String>) -> Self {
        Self {
            label: label.to_string(),
            passed: true,
            detail: detail.into(),
        }
    }

    fn failed(label: &str, detail: impl Into<String>) -> Self {
        Self {
            label: label.to_string(),
            passed: false,
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Serialize)]
struct ControlReport {
    schema: &'static str,
    phase: &'static str,
    machine: String,
    machine_image: String,
    container_version: CommandEvidence,
    machine_image_inspect: CommandEvidence,
    machine_create: CommandEvidence,
    machine_init: CommandEvidence,
    machine_systemd: CommandEvidence,
    machine_stop: CommandEvidence,
    machine_reexec: CommandEvidence,
    machine_inspect: CommandEvidence,
    machine_logs: CommandEvidence,
    machine_delete: CommandEvidence,
    machine_cleanup_confirmed: bool,
    machine_cleanup_attempts: Vec<CommandEvidence>,
    verdict: &'static str,
    blocker: Option<&'static str>,
}

#[derive(Debug, Serialize)]
struct DisposableK3sReport {
    schema: &'static str,
    phase: &'static str,
    machine: String,
    machine_image: String,
    backing_container_id: Option<String>,
    backing_container_status: Option<String>,
    container_version: CommandEvidence,
    machine_image_inspect: CommandEvidence,
    machine_create: CommandEvidence,
    machine_inspect: CommandEvidence,
    guest_systemd: CommandEvidence,
    guest_root: CommandEvidence,
    k3s_install: CommandEvidence,
    k3s_node_ready: CommandEvidence,
    k3s_cluster_state: CommandEvidence,
    k3s_workload: CommandEvidence,
    k3s_workload_cleanup: CommandEvidence,
    k3s_logs: CommandEvidence,
    host_api: HostApiReport,
    machine_delete: CommandEvidence,
    machine_cleanup_confirmed: bool,
    machine_cleanup_attempts: Vec<CommandEvidence>,
    verdict: &'static str,
    limitation: &'static str,
    blocker: Option<String>,
}

#[derive(Debug, Serialize)]
struct HostApiReport {
    requested: bool,
    machine_ip: Option<String>,
    server_endpoint: Option<String>,
    machine_reinspect: CommandEvidence,
    kubeconfig_copy: CommandEvidence,
    kubeconfig_rewrite: LocalEvidence,
    kubeconfig_permissions: LocalEvidence,
    kubectl_cache: LocalEvidence,
    kubectl_client: CommandEvidence,
    kubectl_get_nodes: CommandEvidence,
    credential_cleanup: LocalEvidence,
    credential_cleanup_confirmed: bool,
    verdict: &'static str,
    blocker: Option<String>,
}

#[derive(Debug, Clone)]
struct BackingContainer {
    id: String,
    status: String,
    ip_address: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MachinePresence {
    Present,
    Absent,
    Unknown,
}

fn command_argv(program: &str, args: &[&str]) -> Vec<String> {
    std::iter::once(program.to_string())
        .chain(args.iter().map(|arg| (*arg).to_string()))
        .collect()
}

/// Runs one `container` invocation without a shell so the evidence records the
/// exact argv. The test intentionally does not use an ambient default machine.
fn container_command(label: &str, args: &[&str]) -> CommandEvidence {
    container_command_with_timeout(label, args, MACHINE_COMMAND_TIMEOUT)
}

/// Run one host command with a bounded wall-clock deadline. The disposable
/// k3s probe downloads and waits on a real guest, so an opt-in test must never
/// leave a blocked host-side process behind indefinitely.
fn command_with_timeout(
    program: &str,
    label: &str,
    args: &[&str],
    timeout: Duration,
    remove_env: &[&str],
) -> CommandEvidence {
    let mut command = Command::new(program);
    command
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for key in remove_env {
        command.env_remove(key);
    }
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            return CommandEvidence {
                label: label.to_string(),
                argv: command_argv(program, args),
                exit_code: None,
                stdout: String::new(),
                stderr: format!("spawn {program} {}: {error}", args.join(" ")),
            };
        }
    };
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                return match child.wait_with_output() {
                    Ok(output) => evidence_from_output(program, label, args, output),
                    Err(error) => CommandEvidence {
                        label: label.to_string(),
                        argv: command_argv(program, args),
                        exit_code: None,
                        stdout: String::new(),
                        stderr: format!("collect {program} {}: {error}", args.join(" ")),
                    },
                };
            }
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(100)),
            Ok(None) => {
                let _ = child.kill();
                return match child.wait_with_output() {
                    Ok(output) => {
                        let mut evidence = evidence_from_output(program, label, args, output);
                        if !evidence.stderr.is_empty() {
                            evidence.stderr.push('\n');
                        }
                        evidence
                            .stderr
                            .push_str(&format!("timed out after {} seconds", timeout.as_secs()));
                        evidence
                    }
                    Err(error) => CommandEvidence {
                        label: label.to_string(),
                        argv: command_argv(program, args),
                        exit_code: None,
                        stdout: String::new(),
                        stderr: format!(
                            "timed out after {} seconds; collect {program} {}: {error}",
                            timeout.as_secs(),
                            args.join(" ")
                        ),
                    },
                };
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return CommandEvidence {
                    label: label.to_string(),
                    argv: command_argv(program, args),
                    exit_code: None,
                    stdout: String::new(),
                    stderr: format!("wait for {program} {}: {error}", args.join(" ")),
                };
            }
        }
    }
}

fn container_command_with_timeout(
    label: &str,
    args: &[&str],
    timeout: Duration,
) -> CommandEvidence {
    command_with_timeout("container", label, args, timeout, &[])
}

fn kubectl_command(label: &str, args: &[&str], timeout: Duration) -> CommandEvidence {
    command_with_timeout(
        "kubectl",
        label,
        args,
        timeout,
        &[
            "KUBECONFIG",
            "HTTP_PROXY",
            "HTTPS_PROXY",
            "ALL_PROXY",
            "NO_PROXY",
            "http_proxy",
            "https_proxy",
            "all_proxy",
            "no_proxy",
        ],
    )
}

fn container_exec_command(
    label: &str,
    container_id: &str,
    args: &[&str],
    timeout: Duration,
) -> CommandEvidence {
    let mut exec_args = vec!["exec", container_id];
    exec_args.extend_from_slice(args);
    container_command_with_timeout(label, &exec_args, timeout)
}

fn backing_container_from_machine_inspect(
    machine_inspect: &CommandEvidence,
) -> Result<BackingContainer, String> {
    if !machine_inspect.passed() {
        return Err("machine inspect failed".to_string());
    }
    let value: serde_json::Value = serde_json::from_str(&machine_inspect.stdout)
        .map_err(|error| format!("machine inspect was not JSON: {error}"))?;
    let record = match &value {
        serde_json::Value::Array(records) => records
            .first()
            .ok_or_else(|| "machine inspect returned no records".to_string())?,
        serde_json::Value::Object(_) => &value,
        _ => return Err("machine inspect returned neither an object nor an array".to_string()),
    };
    let status = record
        .get("status")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "machine inspect did not return a status".to_string())?
        .to_string();
    if !status.eq_ignore_ascii_case("running") {
        return Err(format!(
            "machine inspect returned status {status:?}, not running"
        ));
    }
    let id = record
        .get("containerId")
        .and_then(serde_json::Value::as_str)
        .filter(|id| !id.is_empty())
        .ok_or_else(|| "machine inspect did not return a backing containerId".to_string())?
        .to_string();
    let ip_address = record
        .get("ipAddress")
        .and_then(serde_json::Value::as_str)
        .filter(|ip| !ip.is_empty())
        .ok_or_else(|| "machine inspect did not return an ipAddress".to_string())?
        .to_string();
    IpAddr::from_str(&ip_address).map_err(|error| {
        format!("machine inspect returned invalid ipAddress {ip_address:?}: {error}")
    })?;
    Ok(BackingContainer {
        id,
        status,
        ip_address,
    })
}

fn evidence_from_output(
    program: &str,
    label: &str,
    args: &[&str],
    output: Output,
) -> CommandEvidence {
    CommandEvidence {
        label: label.to_string(),
        argv: command_argv(program, args),
        exit_code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn unique_machine_name() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after Unix epoch")
        .as_millis();
    format!("vat-k8s-phase0-control-{}-{millis}", std::process::id())
}

fn machine_image() -> String {
    std::env::var(MACHINE_IMAGE_ENV).unwrap_or_else(|_| DEFAULT_MACHINE_IMAGE.to_string())
}

fn evidence_path(machine: &str) -> PathBuf {
    let root = std::env::var_os("VAT_LOCAL_K8S_EVIDENCE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/private/tmp"));
    root.join(format!("{machine}.json"))
}

fn host_api_report_not_run(requested: bool, reason: impl Into<String>) -> HostApiReport {
    let reason = reason.into();
    HostApiReport {
        requested,
        machine_ip: None,
        server_endpoint: None,
        machine_reinspect: CommandEvidence::not_run("host_machine_reinspect", &[], &reason),
        kubeconfig_copy: CommandEvidence::not_run("host_kubeconfig_copy", &[], &reason),
        kubeconfig_rewrite: LocalEvidence::not_run("host_kubeconfig_rewrite", &reason),
        kubeconfig_permissions: LocalEvidence::not_run("host_kubeconfig_permissions", &reason),
        kubectl_cache: LocalEvidence::not_run("host_kubectl_cache", &reason),
        kubectl_client: CommandEvidence::not_run_for(
            "kubectl",
            "host_kubectl_client",
            &[],
            &reason,
        ),
        kubectl_get_nodes: CommandEvidence::not_run_for(
            "kubectl",
            "host_kubectl_get_nodes",
            &[],
            &reason,
        ),
        credential_cleanup: LocalEvidence::passed(
            "host_kubeconfig_cleanup",
            "no private credential directory was created",
        ),
        credential_cleanup_confirmed: true,
        verdict: if requested { "no-go" } else { "not-requested" },
        blocker: requested.then_some(reason),
    }
}

fn kube_api_endpoint(machine_ip: &str) -> Result<String, String> {
    let ip = IpAddr::from_str(machine_ip)
        .map_err(|error| format!("invalid machine IP {machine_ip:?}: {error}"))?;
    Ok(match ip {
        IpAddr::V4(_) => format!("https://{ip}:6443"),
        IpAddr::V6(_) => format!("https://[{ip}]:6443"),
    })
}

fn private_kubeconfig_dir() -> Result<TempDir, String> {
    let directory = tempfile::Builder::new()
        .prefix("vat-k8s-hostapi-")
        .tempdir()
        .map_err(|error| format!("create private host kubeconfig directory: {error}"))?;
    fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700))
        .map_err(|error| format!("restrict host kubeconfig directory: {error}"))?;
    Ok(directory)
}

fn rewrite_kubeconfig_server(path: &Path, endpoint: &str) -> LocalEvidence {
    const LOOPBACK_ENDPOINT: &str = "https://127.0.0.1:6443";
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            return LocalEvidence::failed(
                "host_kubeconfig_rewrite",
                format!("read copied kubeconfig: {error}"),
            );
        }
    };
    let matches = source.matches(LOOPBACK_ENDPOINT).count();
    if matches != 1 {
        return LocalEvidence::failed(
            "host_kubeconfig_rewrite",
            format!("expected exactly one loopback API endpoint, found {matches}"),
        );
    }
    match fs::write(path, source.replacen(LOOPBACK_ENDPOINT, endpoint, 1)) {
        Ok(()) => LocalEvidence::passed(
            "host_kubeconfig_rewrite",
            "rewrote the single loopback API endpoint without recording credential contents",
        ),
        Err(error) => LocalEvidence::failed(
            "host_kubeconfig_rewrite",
            format!("write rewritten kubeconfig: {error}"),
        ),
    }
}

fn restrict_kubeconfig_permissions(path: &Path) -> LocalEvidence {
    match fs::set_permissions(path, fs::Permissions::from_mode(0o600)) {
        Ok(()) => LocalEvidence::passed(
            "host_kubeconfig_permissions",
            "restricted the copied credential to owner read/write",
        ),
        Err(error) => LocalEvidence::failed(
            "host_kubeconfig_permissions",
            format!("restrict copied kubeconfig permissions: {error}"),
        ),
    }
}

fn create_private_kubectl_cache(directory: &TempDir) -> Result<(PathBuf, LocalEvidence), String> {
    let cache_path = directory.path().join("kubectl-cache");
    fs::create_dir(&cache_path)
        .map_err(|error| format!("create private kubectl cache: {error}"))?;
    fs::set_permissions(&cache_path, fs::Permissions::from_mode(0o700))
        .map_err(|error| format!("restrict private kubectl cache: {error}"))?;
    Ok((
        cache_path,
        LocalEvidence::passed(
            "host_kubectl_cache",
            "created a private kubectl discovery cache under the owned credential directory",
        ),
    ))
}

fn cleanup_private_kubeconfig_dir(directory: TempDir, owned_path: PathBuf) -> LocalEvidence {
    match directory.close() {
        Ok(()) => LocalEvidence::passed(
            "host_kubeconfig_cleanup",
            "removed the private credential directory",
        ),
        Err(close_error) => {
            let mut last_error = format!("TempDir close failed: {close_error}");
            for attempt in 1..=3 {
                match fs::remove_dir_all(&owned_path) {
                    Ok(()) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => {
                        last_error = format!(
                            "remove private credential directory attempt {attempt}: {error}"
                        );
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                }
                match fs::symlink_metadata(&owned_path) {
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                        return LocalEvidence::passed(
                            "host_kubeconfig_cleanup",
                            format!(
                                "removed the private credential directory after TempDir close retry {attempt}"
                            ),
                        );
                    }
                    Ok(_) => {
                        last_error = format!(
                            "private credential directory remains after cleanup attempt {attempt}"
                        );
                    }
                    Err(error) => {
                        last_error = format!(
                            "verify private credential directory cleanup attempt {attempt}: {error}"
                        );
                    }
                }
                thread::sleep(Duration::from_millis(100));
            }
            LocalEvidence::failed("host_kubeconfig_cleanup", last_error)
        }
    }
}

fn probe_host_api(
    requested: bool,
    machine_name: &str,
    initial_backing: Option<&BackingContainer>,
    k3s_node_ready: &CommandEvidence,
) -> HostApiReport {
    if !requested {
        return host_api_report_not_run(false, "host API probe was not requested");
    }
    if !k3s_node_ready.passed() {
        return host_api_report_not_run(
            true,
            "guest k3s did not reach Node Ready, so host API access is not meaningful",
        );
    }
    let Some(initial_backing) = initial_backing else {
        return host_api_report_not_run(
            true,
            "the owned running machine did not expose a backing container and IP address",
        );
    };
    let endpoint = match kube_api_endpoint(&initial_backing.ip_address) {
        Ok(endpoint) => endpoint,
        Err(error) => return host_api_report_not_run(true, error),
    };
    let machine_reinspect = container_command(
        "host_machine_reinspect",
        &["machine", "inspect", machine_name],
    );
    let current_backing = match backing_container_from_machine_inspect(&machine_reinspect) {
        Ok(current_backing) => current_backing,
        Err(error) => {
            let mut report = host_api_report_not_run(true, error);
            report.machine_ip = Some(initial_backing.ip_address.clone());
            report.server_endpoint = Some(endpoint);
            report.machine_reinspect = machine_reinspect;
            return report;
        }
    };
    if current_backing.id != initial_backing.id
        || current_backing.ip_address != initial_backing.ip_address
    {
        let mut report = host_api_report_not_run(
            true,
            "the owned machine backing container or IP changed before kubeconfig export",
        );
        report.machine_ip = Some(current_backing.ip_address);
        report.server_endpoint = Some(endpoint);
        report.machine_reinspect = machine_reinspect;
        return report;
    }

    let mut report = HostApiReport {
        requested: true,
        machine_ip: Some(current_backing.ip_address.clone()),
        server_endpoint: Some(endpoint.clone()),
        machine_reinspect,
        kubeconfig_copy: CommandEvidence::not_run(
            "host_kubeconfig_copy",
            &[],
            "private credential directory was not created",
        ),
        kubeconfig_rewrite: LocalEvidence::not_run(
            "host_kubeconfig_rewrite",
            "private credential directory was not created",
        ),
        kubeconfig_permissions: LocalEvidence::not_run(
            "host_kubeconfig_permissions",
            "private credential directory was not created",
        ),
        kubectl_cache: LocalEvidence::not_run(
            "host_kubectl_cache",
            "private credential directory was not created",
        ),
        kubectl_client: CommandEvidence::not_run_for(
            "kubectl",
            "host_kubectl_client",
            &[],
            "private kubeconfig was not ready",
        ),
        kubectl_get_nodes: CommandEvidence::not_run_for(
            "kubectl",
            "host_kubectl_get_nodes",
            &[],
            "private kubeconfig was not ready",
        ),
        credential_cleanup: LocalEvidence::not_run(
            "host_kubeconfig_cleanup",
            "private credential directory was not created",
        ),
        credential_cleanup_confirmed: false,
        verdict: "no-go",
        blocker: None,
    };

    let credential_dir = match private_kubeconfig_dir() {
        Ok(directory) => directory,
        Err(error) => {
            report.kubeconfig_rewrite =
                LocalEvidence::failed("host_kubeconfig_rewrite", error.clone());
            report.kubeconfig_permissions = LocalEvidence::not_run(
                "host_kubeconfig_permissions",
                "private credential directory could not be created",
            );
            report.kubectl_cache = LocalEvidence::not_run(
                "host_kubectl_cache",
                "private credential directory could not be created",
            );
            report.credential_cleanup = LocalEvidence::passed(
                "host_kubeconfig_cleanup",
                "no private credential directory was created",
            );
            report.credential_cleanup_confirmed = true;
            report.blocker = Some(error);
            return report;
        }
    };
    let credential_path = credential_dir.path().to_path_buf();
    let kubeconfig_path = credential_dir.path().join("kubeconfig");
    let kubeconfig_source = format!("{}:{K3S_KUBECONFIG_PATH}", current_backing.id);
    let kubeconfig_destination = kubeconfig_path.to_string_lossy().into_owned();
    report.kubeconfig_copy = container_command_with_timeout(
        "host_kubeconfig_copy",
        &["copy", &kubeconfig_source, &kubeconfig_destination],
        MACHINE_COMMAND_TIMEOUT,
    );
    if report.kubeconfig_copy.passed() {
        report.kubeconfig_rewrite = rewrite_kubeconfig_server(&kubeconfig_path, &endpoint);
        if report.kubeconfig_rewrite.passed {
            report.kubeconfig_permissions = restrict_kubeconfig_permissions(&kubeconfig_path);
        }
    }
    if report.kubeconfig_copy.passed()
        && report.kubeconfig_rewrite.passed
        && report.kubeconfig_permissions.passed
    {
        match create_private_kubectl_cache(&credential_dir) {
            Ok((cache_path, evidence)) => {
                report.kubectl_cache = evidence;
                let cache_destination = cache_path.to_string_lossy().into_owned();
                report.kubectl_client = kubectl_command(
                    "host_kubectl_client",
                    &["version", "--client", "--output=json"],
                    HOST_KUBECTL_TIMEOUT,
                );
                if report.kubectl_client.passed() {
                    report.kubectl_get_nodes = kubectl_command(
                        "host_kubectl_get_nodes",
                        &[
                            "--kubeconfig",
                            &kubeconfig_destination,
                            "--cache-dir",
                            &cache_destination,
                            "--request-timeout=20s",
                            "get",
                            "nodes",
                            "-o",
                            "json",
                        ],
                        HOST_KUBECTL_TIMEOUT,
                    );
                }
            }
            Err(error) => {
                report.kubectl_cache = LocalEvidence::failed("host_kubectl_cache", error);
            }
        }
    }

    report.credential_cleanup = cleanup_private_kubeconfig_dir(credential_dir, credential_path);
    report.credential_cleanup_confirmed = report.credential_cleanup.passed;
    let host_api_passed = report.machine_reinspect.passed()
        && report.kubeconfig_copy.passed()
        && report.kubeconfig_rewrite.passed
        && report.kubeconfig_permissions.passed
        && report.kubectl_cache.passed
        && report.kubectl_client.passed()
        && report.kubectl_get_nodes.passed()
        && report.credential_cleanup_confirmed;
    if host_api_passed {
        report.verdict = "ephemeral-host-api-go";
    } else if report.blocker.is_none() {
        report.blocker = Some(if !report.credential_cleanup_confirmed {
            "VAT could not remove its private host kubeconfig directory after bounded retries."
                .to_string()
        } else if !report.kubeconfig_copy.passed() {
            "Apple Container could not copy the kubeconfig from the exact owned backing container."
                .to_string()
        } else if !report.kubeconfig_rewrite.passed {
            "VAT could not safely rewrite exactly one loopback API endpoint in the copied kubeconfig."
                .to_string()
        } else if !report.kubeconfig_permissions.passed {
            "VAT could not restrict the copied kubeconfig to owner read/write.".to_string()
        } else if !report.kubectl_cache.passed {
            "VAT could not create a private kubectl cache under its owned credential directory."
                .to_string()
        } else if !report.kubectl_client.passed() {
            "The host kubectl client is unavailable or failed before the API reachability check."
                .to_string()
        } else {
            "macOS kubectl could not read the disposable guest API through the inspected machine IP."
                .to_string()
        });
    }
    report
}

/// Apple Container 1.1.0 reports an absent machine as a structured `notFound`
/// error that includes the requested name. Treat every other inspect failure
/// as unknown so cleanup fails closed if the CLI or daemon cannot establish
/// exact-name absence.
fn machine_inspect_reports_absence(name: &str, evidence: &CommandEvidence) -> bool {
    if evidence.passed() {
        return false;
    }
    let diagnostic = format!("{}\n{}", evidence.stdout, evidence.stderr).to_ascii_lowercase();
    diagnostic.contains("notfound:")
        && diagnostic.contains("container machine")
        && diagnostic.contains(&name.to_ascii_lowercase())
}

/// Deletes only the exact, test-created machine. This is called explicitly so
/// cleanup is recorded, and from Drop as a last resort if an assertion panics.
struct MachineCleanup {
    name: String,
    create_attempted: bool,
    create_succeeded: Option<bool>,
    cleanup_attempted: bool,
    deleted: bool,
    attempts: Vec<CommandEvidence>,
}

impl MachineCleanup {
    fn new(name: String) -> Self {
        Self {
            name,
            create_attempted: false,
            create_succeeded: None,
            cleanup_attempted: false,
            deleted: false,
            attempts: Vec::new(),
        }
    }

    /// Reserve deletion ownership before invoking `machine create`: the
    /// runtime may allocate the uniquely named machine and then return an
    /// error or time out on the client side.
    fn mark_create_attempted(&mut self) {
        self.create_attempted = true;
    }

    fn record_create_result(&mut self, succeeded: bool) {
        self.create_succeeded = Some(succeeded);
    }

    fn cleanup_confirmed(&self) -> bool {
        self.deleted
    }

    fn cleanup_attempts(&self) -> Vec<CommandEvidence> {
        self.attempts.clone()
    }

    fn remaining_cleanup_timeout(deadline: Instant) -> Option<Duration> {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            None
        } else {
            Some(remaining.min(MACHINE_CLEANUP_COMMAND_TIMEOUT))
        }
    }

    fn inspect_presence(&mut self, timeout: Duration) -> MachinePresence {
        let evidence = container_command_with_timeout(
            "machine_cleanup_inspect",
            &["machine", "inspect", &self.name],
            timeout,
        );
        let presence = if evidence.passed() {
            MachinePresence::Present
        } else if machine_inspect_reports_absence(&self.name, &evidence) {
            MachinePresence::Absent
        } else {
            MachinePresence::Unknown
        };
        self.attempts.push(evidence);
        presence
    }

    fn delete(&mut self) -> CommandEvidence {
        if !self.create_attempted {
            self.deleted = true;
            return CommandEvidence::not_run(
                "machine_delete",
                &["machine", "delete", &self.name],
                "machine creation did not succeed",
            );
        }

        self.cleanup_attempted = true;
        let cleanup_started = Instant::now();
        let deadline = cleanup_started + MACHINE_CLEANUP_TIMEOUT;
        // A timed-out or failed create can still finish booting after the
        // client returns. Do not accept an early `notFound`: keep reconciling
        // this exact name through a stabilization window, then require a
        // final exact-name absence result.
        let stabilization_deadline = if self.create_succeeded == Some(false) {
            Some(cleanup_started + MACHINE_CREATE_FAILURE_STABILIZATION)
        } else {
            None
        };
        let mut last_delete = CommandEvidence::not_run(
            "machine_delete",
            &["machine", "delete", &self.name],
            "the owned machine was not observed before cleanup",
        );
        loop {
            let Some(inspect_timeout) = Self::remaining_cleanup_timeout(deadline) else {
                return last_delete;
            };
            match self.inspect_presence(inspect_timeout) {
                MachinePresence::Absent => {
                    let stabilized = stabilization_deadline
                        .map(|not_before| Instant::now() >= not_before)
                        .unwrap_or(true);
                    if stabilized {
                        self.deleted = true;
                        return last_delete;
                    }
                }
                MachinePresence::Present | MachinePresence::Unknown => {
                    let Some(delete_timeout) = Self::remaining_cleanup_timeout(deadline) else {
                        return last_delete;
                    };
                    let evidence = container_command_with_timeout(
                        "machine_delete",
                        &["machine", "delete", &self.name],
                        delete_timeout,
                    );
                    last_delete = evidence.clone();
                    self.attempts.push(evidence);
                }
            }

            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return last_delete;
            }
            thread::sleep(MACHINE_CLEANUP_RETRY_DELAY.min(remaining));
        }
    }
}

impl Drop for MachineCleanup {
    fn drop(&mut self) {
        // Explicit cleanup records every reconciliation attempt in the report.
        // Drop is strictly a panic fallback so a bounded failed explicit pass
        // does not silently spend a second deadline outside that evidence.
        if self.create_attempted && !self.deleted && !self.cleanup_attempted {
            let _ = self.delete();
        }
    }
}

#[test]
fn machine_cleanup_requires_a_structured_missing_machine_result() {
    let missing = CommandEvidence {
        label: "machine_cleanup_inspect".to_string(),
        argv: vec!["container".to_string(), "machine".to_string()],
        exit_code: Some(1),
        stdout: String::new(),
        stderr: "Error: failed to inspect container machine (cause: \"notFound: \"container machine with ID owned-machine not found\"\")".to_string(),
    };
    let unavailable = CommandEvidence {
        label: "machine_cleanup_inspect".to_string(),
        argv: vec!["container".to_string(), "machine".to_string()],
        exit_code: Some(1),
        stdout: String::new(),
        stderr: "Error: failed to connect to the machine API".to_string(),
    };

    assert!(machine_inspect_reports_absence("owned-machine", &missing));
    assert!(!machine_inspect_reports_absence("other-machine", &missing));
    assert!(!machine_inspect_reports_absence(
        "owned-machine",
        &unavailable
    ));
}

#[test]
fn skipped_host_api_evidence_is_not_reported_as_success() {
    assert!(!CommandEvidence::not_run_for(
        "kubectl",
        "host_kubectl_get_nodes",
        &[],
        "host API probe was not requested",
    )
    .passed());
    assert!(
        !LocalEvidence::not_run("host_kubectl_cache", "host API probe was not requested").passed
    );
}

#[test]
fn host_kubeconfig_rewrite_is_exact_and_private_cleanup_removes_every_artifact() {
    let directory = private_kubeconfig_dir().expect("create private kubeconfig directory");
    let owned_path = directory.path().to_path_buf();
    let kubeconfig = owned_path.join("kubeconfig");
    fs::write(
        &kubeconfig,
        "clusters:\n  - cluster:\n      server: https://127.0.0.1:6443\n    name: default\n",
    )
    .expect("write fixture kubeconfig");

    let rewrite = rewrite_kubeconfig_server(&kubeconfig, "https://192.168.64.17:6443");
    assert!(rewrite.passed, "{rewrite:?}");
    let permissions = restrict_kubeconfig_permissions(&kubeconfig);
    assert!(permissions.passed, "{permissions:?}");
    let (cache, cache_evidence) =
        create_private_kubectl_cache(&directory).expect("create private cache");
    assert!(cache_evidence.passed, "{cache_evidence:?}");
    assert!(cache.starts_with(&owned_path));
    assert!(fs::read_to_string(&kubeconfig)
        .expect("read rewritten kubeconfig")
        .contains("https://192.168.64.17:6443"));

    let cleanup = cleanup_private_kubeconfig_dir(directory, owned_path.clone());
    assert!(cleanup.passed, "{cleanup:?}");
    assert!(matches!(
        fs::symlink_metadata(owned_path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound
    ));
}

#[test]
fn kube_api_endpoint_formats_machine_ips_without_accepting_invalid_input() {
    assert_eq!(
        kube_api_endpoint("192.168.64.17").expect("IPv4 endpoint"),
        "https://192.168.64.17:6443"
    );
    assert_eq!(
        kube_api_endpoint("fd00::17").expect("IPv6 endpoint"),
        "https://[fd00::17]:6443"
    );
    assert!(kube_api_endpoint("not-an-ip").is_err());
}

#[test]
#[ignore = "real Apple-container machine probe; run only with VAT_LOCAL_K8S_E2E=1"]
fn apple_machine_exec_control_is_usable_before_k3s() {
    if std::env::var(E2E_OPT_IN).as_deref() != Ok("1") {
        eprintln!("{E2E_OPT_IN}=1 is required; skipping destructive real-host probe");
        return;
    }

    let container_version = container_command("container_version", &["--version"]);
    if !container_version.passed() {
        eprintln!("container CLI unavailable; skipping Apple-machine feasibility probe");
        return;
    }

    let name = unique_machine_name();
    let image = machine_image();
    let report_path = evidence_path(&name);
    let mut cleanup = MachineCleanup::new(name.clone());
    let machine_image_inspect =
        container_command("machine_image_inspect", &["image", "inspect", &image]);

    // `--no-boot` makes the first boot attributable to the exact commands
    // below. Apple Container machines boot the image init, so this uses a
    // systemd image built from tests/fixtures/local-k8s-phase0-machine rather
    // than a minimal application image such as alpine.
    let (
        machine_create,
        machine_init,
        machine_systemd,
        machine_stop,
        machine_reexec,
        machine_inspect,
        machine_logs,
    ) = if machine_image_inspect.passed() {
        cleanup.mark_create_attempted();
        let machine_create = container_command(
            "machine_create",
            &[
                "machine",
                "create",
                "--no-boot",
                "--name",
                &name,
                "--home-mount",
                "none",
                "--cpus",
                "2",
                "--memory",
                "4G",
                &image,
            ],
        );
        cleanup.record_create_result(machine_create.passed());
        if machine_create.passed() {
            let machine_init = container_command(
                "machine_init",
                &[
                    "machine",
                    "run",
                    "--name",
                    &name,
                    "--",
                    "cat",
                    "/proc/1/comm",
                ],
            );
            let machine_systemd = container_command(
                "machine_systemd",
                &[
                    "machine",
                    "run",
                    "--name",
                    &name,
                    "--",
                    "sh",
                    "-ec",
                    "test \"$(cat /proc/1/comm)\" = systemd && systemctl is-system-running --wait",
                ],
            );
            let machine_stop = container_command("machine_stop", &["machine", "stop", &name]);
            let machine_reexec = container_command(
                "machine_reexec",
                &[
                    "machine",
                    "run",
                    "--name",
                    &name,
                    "--",
                    "sh",
                    "-ec",
                    "test \"$(cat /proc/1/comm)\" = systemd && echo vat-k8s-phase0-control-ok",
                ],
            );
            let machine_inspect =
                container_command("machine_inspect", &["machine", "inspect", &name]);
            let machine_logs = container_command("machine_logs", &["machine", "logs", &name]);
            (
                machine_create,
                machine_init,
                machine_systemd,
                machine_stop,
                machine_reexec,
                machine_inspect,
                machine_logs,
            )
        } else {
            (
                machine_create,
                CommandEvidence::not_run("machine_init", &[], "machine creation failed"),
                CommandEvidence::not_run("machine_systemd", &[], "machine creation failed"),
                CommandEvidence::not_run("machine_stop", &[], "machine creation failed"),
                CommandEvidence::not_run("machine_reexec", &[], "machine creation failed"),
                CommandEvidence::not_run("machine_inspect", &[], "machine creation failed"),
                CommandEvidence::not_run("machine_logs", &[], "machine creation failed"),
            )
        }
    } else {
        (
            CommandEvidence::not_run("machine_create", &[], "systemd image is unavailable"),
            CommandEvidence::not_run("machine_init", &[], "systemd image is unavailable"),
            CommandEvidence::not_run("machine_systemd", &[], "systemd image is unavailable"),
            CommandEvidence::not_run("machine_stop", &[], "systemd image is unavailable"),
            CommandEvidence::not_run("machine_reexec", &[], "systemd image is unavailable"),
            CommandEvidence::not_run("machine_inspect", &[], "systemd image is unavailable"),
            CommandEvidence::not_run("machine_logs", &[], "systemd image is unavailable"),
        )
    };
    let machine_delete = cleanup.delete();
    let machine_cleanup_confirmed = cleanup.cleanup_confirmed();
    let machine_cleanup_attempts = cleanup.cleanup_attempts();

    let control_passed = machine_image_inspect.passed()
        && machine_create.passed()
        && machine_init.passed()
        && machine_init.stdout.trim() == "systemd"
        && machine_systemd.passed()
        && machine_stop.passed()
        && machine_reexec.passed()
        && machine_reexec.stdout.contains("vat-k8s-phase0-control-ok")
        && machine_cleanup_confirmed;
    let blocker = if control_passed {
        None
    } else if !machine_cleanup_confirmed {
        Some(
            "VAT could not confirm that its exact temporary machine is absent after bounded reconciliation. Inspect the recorded cleanup attempts before retrying.",
        )
    } else if !machine_image_inspect.passed() {
        Some(
            "The required systemd Phase 0 image is unavailable. Build local/vat-k8s-systemd:phase0 from apps/vat/tests/fixtures/local-k8s-phase0-machine before retrying.",
        )
    } else if machine_logs.stdout.contains("can't run '/sbin/openrc'")
        || machine_logs.stderr.contains("can't run '/sbin/openrc'")
    {
        Some(
            "The selected machine image cannot boot its init system; this is an image-fixture failure, not yet evidence of a broken Apple Container host runtime.",
        )
    } else if machine_logs.stdout.contains("running in system mode")
        && (!machine_init.passed() || !machine_reexec.passed())
    {
        Some(
            "Apple Container booted systemd, but its machine-run command transport failed. Container exec can reach an already-running backing container, but cannot restart a stopped machine, so local Kubernetes remains NO-GO until machine run is repaired.",
        )
    } else {
        Some(
            "Apple Container must boot and preserve a systemd machine before k3s, host kubeconfig, or workload probes are meaningful.",
        )
    };
    let report = ControlReport {
        schema: "vat.local-k8s.phase0.control.v3",
        phase: "systemd-machine-control",
        machine: name,
        machine_image: image,
        container_version,
        machine_image_inspect,
        machine_create,
        machine_init,
        machine_systemd,
        machine_stop,
        machine_reexec,
        machine_inspect,
        machine_logs,
        machine_delete,
        machine_cleanup_confirmed,
        machine_cleanup_attempts,
        verdict: if control_passed { "go" } else { "no-go" },
        blocker,
    };

    fs::write(
        &report_path,
        serde_json::to_vec_pretty(&report).expect("serialize Phase 0 control evidence"),
    )
    .unwrap_or_else(|error| panic!("write {}: {error}", report_path.display()));
    println!(
        "vat local-k8s Phase 0 control evidence: {}\n{}",
        report_path.display(),
        serde_json::to_string_pretty(&report).expect("render Phase 0 control evidence")
    );

    assert!(
        report.machine_cleanup_confirmed,
        "Phase 0 cleanup failed; inspect {} before retrying",
        report_path.display()
    );
    assert!(
        control_passed,
        "Apple Container systemd-machine control failed; Phase 0 remains NO-GO. Evidence: {}",
        report_path.display()
    );
}

/// This is intentionally a separate probe from the durable stop/run control
/// above. It observes whether an already-running Apple machine can host a
/// disposable k3s node through the documented `container exec` surface. A pass
/// does not claim that VAT can restart, reconcile, or productize that node.
#[test]
#[ignore = "real disposable k3s probe; run only with VAT_LOCAL_K8S_DISPOSABLE_E2E=1"]
fn apple_machine_bootstraps_disposable_k3s_via_backing_container_exec() {
    if std::env::var(DISPOSABLE_E2E_OPT_IN).as_deref() != Ok("1") {
        eprintln!("{DISPOSABLE_E2E_OPT_IN}=1 is required; skipping disposable k3s substrate probe");
        return;
    }
    let host_api_requested = std::env::var(HOST_API_E2E_OPT_IN).as_deref() == Ok("1");

    let container_version = container_command("container_version", &["--version"]);
    if !container_version.passed() {
        eprintln!("container CLI unavailable; skipping disposable k3s substrate probe");
        return;
    }

    let name = format!("{}-disposable", unique_machine_name());
    let image = machine_image();
    let report_path = evidence_path(&name);
    let mut cleanup = MachineCleanup::new(name.clone());
    let machine_image_inspect =
        container_command("machine_image_inspect", &["image", "inspect", &image]);

    // This deliberately omits `--no-boot`: current Apple Container hosts can
    // auto-boot this systemd image even where `machine run` cannot transport a
    // command. The only command path below is `container exec` against the
    // backing ID returned by inspect for this exact owned machine.
    let (
        machine_create,
        machine_inspect,
        backing_container,
        guest_systemd,
        guest_root,
        k3s_install,
        k3s_node_ready,
        k3s_cluster_state,
        k3s_workload,
        k3s_workload_cleanup,
        k3s_logs,
    ) = if machine_image_inspect.passed() {
        cleanup.mark_create_attempted();
        let machine_create = container_command(
            "machine_create",
            &[
                "machine",
                "create",
                "--name",
                &name,
                "--home-mount",
                "none",
                "--cpus",
                "2",
                "--memory",
                "4G",
                &image,
            ],
        );
        cleanup.record_create_result(machine_create.passed());
        if machine_create.passed() {
            let machine_inspect =
                container_command("machine_inspect", &["machine", "inspect", &name]);
            match backing_container_from_machine_inspect(&machine_inspect) {
                Ok(backing_container) => {
                    let guest_systemd = container_exec_command(
                        "guest_systemd",
                        &backing_container.id,
                        &[
                            "sh",
                            "-ec",
                            "set -eu; test \"$(cat /proc/1/comm)\" = systemd; state=\"$(systemctl is-system-running --wait || true)\"; test \"$state\" = running || test \"$state\" = degraded; printf '%s\\n' \"$state\"",
                        ],
                        MACHINE_COMMAND_TIMEOUT,
                    );
                    let guest_root = container_exec_command(
                        "guest_root",
                        &backing_container.id,
                        &["id", "-u"],
                        MACHINE_COMMAND_TIMEOUT,
                    );
                    let can_install = guest_systemd.passed()
                        && guest_root.passed()
                        && guest_root.stdout.trim() == "0";
                    let k3s_install = if can_install {
                        let install_script = if host_api_requested {
                            format!(
                                "set -eu; curl -fsSL https://get.k3s.io -o /tmp/install-k3s.sh; INSTALL_K3S_VERSION=v1.36.2+k3s1 INSTALL_K3S_EXEC='server --write-kubeconfig-mode 0600 --tls-san {}' sh /tmp/install-k3s.sh; k3s --version",
                                backing_container.ip_address
                            )
                        } else {
                            "set -eu; curl -fsSL https://get.k3s.io -o /tmp/install-k3s.sh; INSTALL_K3S_VERSION=v1.36.2+k3s1 INSTALL_K3S_EXEC='server --write-kubeconfig-mode 0600' sh /tmp/install-k3s.sh; k3s --version".to_string()
                        };
                        container_exec_command(
                            "k3s_install",
                            &backing_container.id,
                            &["sh", "-ec", &install_script],
                            K3S_INSTALL_TIMEOUT,
                        )
                    } else {
                        CommandEvidence::not_run(
                            "k3s_install",
                            &[],
                            "systemd or root preflight failed",
                        )
                    };
                    let k3s_node_ready = if k3s_install.passed() {
                        container_exec_command(
                            "k3s_node_ready",
                            &backing_container.id,
                            &[
                                "sh",
                                "-ec",
                                "set -eu; systemctl is-active --quiet k3s; k3s kubectl wait --for=condition=Ready node --all --timeout=180s; k3s kubectl get nodes -o wide",
                            ],
                            K3S_READY_TIMEOUT,
                        )
                    } else {
                        CommandEvidence::not_run("k3s_node_ready", &[], "k3s installation failed")
                    };
                    let k3s_cluster_state = if k3s_node_ready.passed() {
                        container_exec_command(
                            "k3s_cluster_state",
                            &backing_container.id,
                            &[
                                "sh",
                                "-ec",
                                "set -eu; k3s kubectl get nodes -o wide; k3s kubectl get pods --all-namespaces -o wide",
                            ],
                            MACHINE_COMMAND_TIMEOUT,
                        )
                    } else {
                        CommandEvidence::not_run(
                            "k3s_cluster_state",
                            &[],
                            "node did not reach Ready",
                        )
                    };
                    let k3s_workload = if k3s_node_ready.passed() {
                        container_exec_command(
                            "k3s_workload",
                            &backing_container.id,
                            &[
                                "sh",
                                "-ec",
                                "set -eu; k3s kubectl delete job vat-phase0-smoke --ignore-not-found=true; k3s kubectl create job vat-phase0-smoke --image=busybox:1.36.1 -- /bin/sh -c 'echo vat-k8s-phase0-workload-ok'; k3s kubectl wait --for=condition=complete job/vat-phase0-smoke --timeout=180s; k3s kubectl logs job/vat-phase0-smoke",
                            ],
                            K3S_READY_TIMEOUT,
                        )
                    } else {
                        CommandEvidence::not_run("k3s_workload", &[], "node did not reach Ready")
                    };
                    let k3s_workload_cleanup = if k3s_install.passed() {
                        container_exec_command(
                            "k3s_workload_cleanup",
                            &backing_container.id,
                            &[
                                "sh",
                                "-ec",
                                "k3s kubectl delete job vat-phase0-smoke --ignore-not-found=true --wait=true --timeout=60s",
                            ],
                            MACHINE_COMMAND_TIMEOUT,
                        )
                    } else {
                        CommandEvidence::not_run(
                            "k3s_workload_cleanup",
                            &[],
                            "k3s installation failed",
                        )
                    };
                    let k3s_logs = container_exec_command(
                        "k3s_logs",
                        &backing_container.id,
                        &["journalctl", "-u", "k3s", "-b", "--no-pager", "--lines=200"],
                        MACHINE_COMMAND_TIMEOUT,
                    );
                    (
                        machine_create,
                        machine_inspect,
                        Some(backing_container),
                        guest_systemd,
                        guest_root,
                        k3s_install,
                        k3s_node_ready,
                        k3s_cluster_state,
                        k3s_workload,
                        k3s_workload_cleanup,
                        k3s_logs,
                    )
                }
                Err(reason) => (
                    machine_create,
                    machine_inspect,
                    None,
                    CommandEvidence::not_run("guest_systemd", &[], &reason),
                    CommandEvidence::not_run("guest_root", &[], &reason),
                    CommandEvidence::not_run("k3s_install", &[], &reason),
                    CommandEvidence::not_run("k3s_node_ready", &[], &reason),
                    CommandEvidence::not_run("k3s_cluster_state", &[], &reason),
                    CommandEvidence::not_run("k3s_workload", &[], &reason),
                    CommandEvidence::not_run("k3s_workload_cleanup", &[], &reason),
                    CommandEvidence::not_run("k3s_logs", &[], &reason),
                ),
            }
        } else {
            (
                machine_create,
                CommandEvidence::not_run("machine_inspect", &[], "machine creation failed"),
                None,
                CommandEvidence::not_run("guest_systemd", &[], "machine creation failed"),
                CommandEvidence::not_run("guest_root", &[], "machine creation failed"),
                CommandEvidence::not_run("k3s_install", &[], "machine creation failed"),
                CommandEvidence::not_run("k3s_node_ready", &[], "machine creation failed"),
                CommandEvidence::not_run("k3s_cluster_state", &[], "machine creation failed"),
                CommandEvidence::not_run("k3s_workload", &[], "machine creation failed"),
                CommandEvidence::not_run("k3s_workload_cleanup", &[], "machine creation failed"),
                CommandEvidence::not_run("k3s_logs", &[], "machine creation failed"),
            )
        }
    } else {
        (
            CommandEvidence::not_run("machine_create", &[], "systemd image is unavailable"),
            CommandEvidence::not_run("machine_inspect", &[], "systemd image is unavailable"),
            None,
            CommandEvidence::not_run("guest_systemd", &[], "systemd image is unavailable"),
            CommandEvidence::not_run("guest_root", &[], "systemd image is unavailable"),
            CommandEvidence::not_run("k3s_install", &[], "systemd image is unavailable"),
            CommandEvidence::not_run("k3s_node_ready", &[], "systemd image is unavailable"),
            CommandEvidence::not_run("k3s_cluster_state", &[], "systemd image is unavailable"),
            CommandEvidence::not_run("k3s_workload", &[], "systemd image is unavailable"),
            CommandEvidence::not_run("k3s_workload_cleanup", &[], "systemd image is unavailable"),
            CommandEvidence::not_run("k3s_logs", &[], "systemd image is unavailable"),
        )
    };
    let host_api = probe_host_api(
        host_api_requested,
        &name,
        backing_container.as_ref(),
        &k3s_node_ready,
    );
    let machine_delete = cleanup.delete();
    let machine_cleanup_confirmed = cleanup.cleanup_confirmed();
    let machine_cleanup_attempts = cleanup.cleanup_attempts();

    let disposable_passed = machine_image_inspect.passed()
        && machine_create.passed()
        && machine_inspect.passed()
        && backing_container.is_some()
        && guest_systemd.passed()
        && guest_root.passed()
        && guest_root.stdout.trim() == "0"
        && k3s_install.passed()
        && k3s_node_ready.passed()
        && k3s_cluster_state.passed()
        && k3s_workload.passed()
        && k3s_workload.stdout.contains("vat-k8s-phase0-workload-ok")
        && k3s_workload_cleanup.passed()
        && (!host_api_requested || host_api.verdict == "ephemeral-host-api-go")
        && host_api.credential_cleanup_confirmed
        && machine_cleanup_confirmed;
    let blocker = if !machine_cleanup_confirmed {
        Some(
            "VAT could not confirm that its exact temporary machine is absent after bounded reconciliation. Inspect the recorded cleanup attempts before retrying."
                .to_string(),
        )
    } else if !machine_image_inspect.passed() {
        Some(
            "The required systemd Phase 0 image is unavailable. Build local/vat-k8s-systemd:phase0 from apps/vat/tests/fixtures/local-k8s-phase0-machine before retrying."
                .to_string(),
        )
    } else if !machine_create.passed() {
        Some("Apple Container could not create the disposable systemd machine.".to_string())
    } else if backing_container.is_none() {
        Some(
            "Apple Container did not expose a running backing container ID for the owned machine, so the disposable exec probe fails closed."
                .to_string(),
        )
    } else if !guest_systemd.passed() || !guest_root.passed() || guest_root.stdout.trim() != "0" {
        Some(
            "The auto-booted machine did not provide a usable root systemd guest through container exec."
                .to_string(),
        )
    } else if !k3s_install.passed() {
        Some("k3s installation did not complete inside the disposable guest.".to_string())
    } else if !k3s_node_ready.passed() {
        Some("k3s did not produce a Ready node within the bounded probe.".to_string())
    } else if host_api_requested && host_api.verdict != "ephemeral-host-api-go" {
        host_api.blocker.clone().or_else(|| {
            Some("The requested host kubeconfig/API probe did not complete.".to_string())
        })
    } else if !k3s_workload.passed() || !k3s_workload.stdout.contains("vat-k8s-phase0-workload-ok")
    {
        Some(
            "A disposable k3s node started, but the bounded workload did not complete and emit its marker."
                .to_string(),
        )
    } else if !k3s_workload_cleanup.passed() || !machine_cleanup_confirmed {
        Some("The probe could not clean up every owned resource.".to_string())
    } else {
        None
    };
    let report = DisposableK3sReport {
        schema: "vat.local-k8s.phase0.disposable-k3s.v3",
        phase: "disposable-k3s-substrate",
        machine: name,
        machine_image: image,
        backing_container_id: backing_container.as_ref().map(|backing| backing.id.clone()),
        backing_container_status: backing_container
            .as_ref()
            .map(|backing| backing.status.clone()),
        container_version,
        machine_image_inspect,
        machine_create,
        machine_inspect,
        guest_systemd,
        guest_root,
        k3s_install,
        k3s_node_ready,
        k3s_cluster_state,
        k3s_workload,
        k3s_workload_cleanup,
        k3s_logs,
        host_api,
        machine_delete,
        machine_cleanup_confirmed,
        machine_cleanup_attempts,
        verdict: if disposable_passed {
            "ephemeral-go"
        } else {
            "no-go"
        },
        limitation: "The guest substrate remains one-machine and one-boot. When explicitly requested, a successful host sub-probe proves only temporary macOS kubeconfig/API access through the inspected machine IP; it does not prove durable kubeconfig, port exposure, local image delivery, storage, multi-node networking, or stop/run durability.",
        blocker,
    };

    fs::write(
        &report_path,
        serde_json::to_vec_pretty(&report).expect("serialize disposable k3s evidence"),
    )
    .unwrap_or_else(|error| panic!("write {}: {error}", report_path.display()));
    println!(
        "vat local-k8s disposable k3s evidence: {}\n{}",
        report_path.display(),
        serde_json::to_string_pretty(&report).expect("render disposable k3s evidence")
    );

    assert!(
        report.machine_cleanup_confirmed,
        "disposable k3s cleanup failed; inspect {} before retrying",
        report_path.display()
    );
    assert!(
        disposable_passed,
        "Apple Container disposable k3s probe failed. Evidence: {}",
        report_path.display()
    );
}

// HANDWRITE-END
