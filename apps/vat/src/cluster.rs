// SPEC-MANAGED: apps/vat/tech-design/semantic/source/projects-vat-src-cluster-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Local Kubernetes cluster drivers (kind / k3d / minikube) behind one enum.
//!
//! vat provisions ephemeral local Kubernetes clusters as run-scoped services
//! and as standalone objects via `vat cluster`. It is a thin orchestrator over
//! the upstream CLIs — it builds no images and runs no daemon. Every cluster
//! gets an *isolated* kubeconfig file; vat never touches `~/.kube/config`.
//!
//! On Apple Silicon every backend needs Docker, so resolution checks both the
//! backend binary on PATH and a reachable Docker daemon; when neither side is
//! satisfiable it reports a structured [`BackendUnavailable`] rather than
//! panicking.

use std::fmt;
use std::io::{Read, Seek, SeekFrom};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};

use crate::config::ClusterBackend;

const CLUSTER_DELETE_TIMEOUT: Duration = Duration::from_secs(30);
const BACKEND_QUERY_TIMEOUT: Duration = Duration::from_secs(30);
const DOCKER_PROBE_TIMEOUT: Duration = Duration::from_secs(5);
const COMMAND_TERM_GRACE: Duration = Duration::from_millis(300);
const COMMAND_REAP_TIMEOUT: Duration = Duration::from_secs(5);
const COMMAND_POLL_INTERVAL: Duration = Duration::from_millis(20);
const MAX_CAPTURE_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Debug)]
struct OwnedCommandCleanupFailed {
    detail: String,
}

impl fmt::Display for OwnedCommandCleanupFailed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.detail)
    }
}

impl std::error::Error for OwnedCommandCleanupFailed {}

pub(crate) fn owned_command_cleanup_failure(error: &anyhow::Error) -> Option<&str> {
    error
        .downcast_ref::<OwnedCommandCleanupFailed>()
        .map(|failure| failure.detail.as_str())
}

#[cfg(test)]
pub(crate) fn injected_owned_command_cleanup_failure(detail: &str) -> anyhow::Error {
    OwnedCommandCleanupFailed {
        detail: detail.to_string(),
    }
    .into()
}

fn never_cancel() -> Result<()> {
    Ok(())
}

/// A concrete cluster backend resolved against the host.
/// @spec apps/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#logic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedBackend {
    Kind,
    K3d,
    Minikube,
}

/// @spec apps/vat/tech-design/semantic/source/projects-vat-src-cluster-rs.md#source
impl ResolvedBackend {
    /// The three backends in `auto` preference order.
    pub const ALL: [ResolvedBackend; 3] = [Self::Kind, Self::K3d, Self::Minikube];

    /// Backend name as it appears in vat.toml, evidence, and the CLI.
    pub fn name(self) -> &'static str {
        match self {
            Self::Kind => "kind",
            Self::K3d => "k3d",
            Self::Minikube => "minikube",
        }
    }

    /// The CLI binary that drives this backend.
    pub fn binary(self) -> &'static str {
        self.name()
    }

    /// Resolve a backend name back to the enum.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "kind" => Some(Self::Kind),
            "k3d" => Some(Self::K3d),
            "minikube" => Some(Self::Minikube),
            _ => None,
        }
    }

    /// Whether this backend's CLI is installed (PATH only; Docker is a separate
    /// check during resolution).
    pub fn installed(self) -> bool {
        which(self.binary()).is_some()
    }

    /// `kubectl` argv that succeeds once the cluster's API server answers.
    pub fn ready_argv(self, kubeconfig: &Path) -> Vec<String> {
        vec![
            "kubectl".to_string(),
            "--kubeconfig".to_string(),
            kubeconfig.to_string_lossy().into_owned(),
            "get".to_string(),
            "nodes".to_string(),
        ]
    }

    /// Create the cluster and write its isolated kubeconfig. Bounded by
    /// `timeout`; on overrun the child is killed and a timeout error returned.
    pub fn create(self, spec: &ClusterSpec, timeout: Duration) -> Result<ClusterInfo> {
        self.create_cancellable(spec, timeout, &never_cancel)
    }

    /// Run-scoped create variant. `cancellation` is polled while each backend
    /// command owns a process group; a cancellation error is returned only
    /// after TERM/grace/KILL/reap/group-absence cleanup succeeds.
    pub(crate) fn create_cancellable(
        self,
        spec: &ClusterSpec,
        timeout: Duration,
        cancellation: &dyn Fn() -> Result<()>,
    ) -> Result<ClusterInfo> {
        if let Some(parent) = spec.kubeconfig.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        let info = match self {
            Self::Kind => self.create_kind(spec, timeout, cancellation),
            Self::K3d => self.create_k3d(spec, timeout, cancellation),
            Self::Minikube => self.create_minikube(spec, timeout, cancellation),
        }?;
        cancellation()?;
        Ok(info)
    }

    fn create_kind(
        self,
        spec: &ClusterSpec,
        timeout: Duration,
        cancellation: &dyn Fn() -> Result<()>,
    ) -> Result<ClusterInfo> {
        let mut cmd = Command::new("kind");
        cmd.args(["create", "cluster", "--name", spec.name, "--kubeconfig"]);
        cmd.arg(spec.kubeconfig);
        cmd.arg("--wait")
            .arg(format!("{}s", timeout.as_secs().max(1)));
        if let Some(ver) = spec.k8s_version {
            cmd.arg("--image").arg(format!("kindest/node:v{ver}"));
        }
        if spec.nodes > 1 {
            let cfg_path = spec.kubeconfig.with_file_name("kind-config.yaml");
            std::fs::write(&cfg_path, kind_multinode_config(spec.nodes))
                .with_context(|| format!("write {}", cfg_path.display()))?;
            cmd.arg("--config").arg(&cfg_path);
        }
        run_capture_cancellable(
            cmd,
            timeout + Duration::from_secs(30),
            "kind create cluster",
            cancellation,
        )?;
        Ok(self.info(spec))
    }

    fn create_k3d(
        self,
        spec: &ClusterSpec,
        timeout: Duration,
        cancellation: &dyn Fn() -> Result<()>,
    ) -> Result<ClusterInfo> {
        let mut cmd = Command::new("k3d");
        cmd.args([
            "cluster",
            "create",
            spec.name,
            "--kubeconfig-update-default=false",
            "--kubeconfig-switch-context=false",
            "--wait",
        ]);
        if let Some(ver) = spec.k8s_version {
            cmd.arg("--image").arg(format!("rancher/k3s:v{ver}-k3s1"));
        }
        if spec.nodes > 1 {
            cmd.arg("--agents").arg((spec.nodes - 1).to_string());
        }
        run_capture_cancellable(
            cmd,
            timeout + Duration::from_secs(30),
            "k3d cluster create",
            cancellation,
        )?;
        // k3d writes the kubeconfig to stdout; capture it into the isolated file.
        let mut kubeconfig = Command::new("k3d");
        kubeconfig.args(["kubeconfig", "get", spec.name]);
        let stdout = run_output_cancellable(
            kubeconfig,
            BACKEND_QUERY_TIMEOUT,
            &format!("k3d kubeconfig get for cluster `{}`", spec.name),
            cancellation,
        )?;
        std::fs::write(spec.kubeconfig, stdout)
            .with_context(|| format!("write {}", spec.kubeconfig.display()))?;
        Ok(self.info(spec))
    }

    fn create_minikube(
        self,
        spec: &ClusterSpec,
        timeout: Duration,
        cancellation: &dyn Fn() -> Result<()>,
    ) -> Result<ClusterInfo> {
        let mut cmd = Command::new("minikube");
        cmd.args(["start", "-p", spec.name, "--driver=docker", "--wait=all"]);
        if let Some(ver) = spec.k8s_version {
            cmd.arg(format!("--kubernetes-version=v{ver}"));
        }
        if spec.nodes > 1 {
            cmd.arg("--nodes").arg(spec.nodes.to_string());
        }
        // Point minikube at the isolated kubeconfig via the child's env only —
        // never mutate vat's own process environment.
        cmd.env("KUBECONFIG", spec.kubeconfig);
        run_capture_cancellable(
            cmd,
            timeout + Duration::from_secs(30),
            "minikube start",
            cancellation,
        )?;
        Ok(self.info(spec))
    }

    fn info(self, spec: &ClusterSpec) -> ClusterInfo {
        ClusterInfo {
            backend: self.name(),
            name: spec.name.to_string(),
            kubeconfig: spec.kubeconfig.to_path_buf(),
            node_count: spec.nodes,
        }
    }

    /// Delete the cluster via the backend CLI.
    pub fn delete(self, name: &str) -> Result<()> {
        let cmd = match self {
            Self::Kind => {
                let mut c = Command::new("kind");
                c.args(["delete", "cluster", "--name", name]);
                c
            }
            Self::K3d => {
                let mut c = Command::new("k3d");
                c.args(["cluster", "delete", name]);
                c
            }
            Self::Minikube => {
                let mut c = Command::new("minikube");
                c.args(["delete", "-p", name]);
                c
            }
        };
        run_capture(
            cmd,
            CLUSTER_DELETE_TIMEOUT,
            &format!("{} delete cluster `{name}`", self.name()),
        )
    }

    /// List cluster names this backend currently owns.
    pub fn list(self) -> Result<Vec<String>> {
        match self {
            Self::Kind => {
                let mut command = Command::new("kind");
                command.args(["get", "clusters"]);
                let stdout = run_output(command, BACKEND_QUERY_TIMEOUT, "kind get clusters")?;
                Ok(String::from_utf8_lossy(&stdout)
                    .lines()
                    .map(str::trim)
                    .filter(|l| !l.is_empty() && *l != "No kind clusters found.")
                    .map(String::from)
                    .collect())
            }
            Self::K3d => {
                let mut command = Command::new("k3d");
                command.args(["cluster", "list", "-o", "json"]);
                let stdout = run_output(command, BACKEND_QUERY_TIMEOUT, "k3d cluster list")?;
                let value: serde_json::Value =
                    serde_json::from_slice(&stdout).unwrap_or(serde_json::Value::Null);
                Ok(value
                    .as_array()
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(|c| {
                                c.get("name").and_then(|n| n.as_str()).map(String::from)
                            })
                            .collect()
                    })
                    .unwrap_or_default())
            }
            Self::Minikube => {
                let mut command = Command::new("minikube");
                command.args(["profile", "list", "-o", "json"]);
                let stdout = run_output(command, BACKEND_QUERY_TIMEOUT, "minikube profile list")?;
                let value: serde_json::Value =
                    serde_json::from_slice(&stdout).unwrap_or(serde_json::Value::Null);
                Ok(value
                    .get("valid")
                    .and_then(|v| v.as_array())
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(|p| {
                                p.get("Name").and_then(|n| n.as_str()).map(String::from)
                            })
                            .collect()
                    })
                    .unwrap_or_default())
            }
        }
    }
}

/// Desired cluster shape passed to a backend driver.
/// @spec apps/vat/tech-design/semantic/source/projects-vat-src-cluster-rs.md#source
pub struct ClusterSpec<'a> {
    pub name: &'a str,
    pub k8s_version: Option<&'a str>,
    pub nodes: u32,
    pub kubeconfig: &'a Path,
}

/// Result of creating or inspecting a cluster.
#[derive(Debug, Clone)]
/// @spec apps/vat/tech-design/semantic/source/projects-vat-src-cluster-rs.md#source
pub struct ClusterInfo {
    pub backend: &'static str,
    pub name: String,
    pub kubeconfig: PathBuf,
    pub node_count: u32,
}

/// Structured "no usable cluster backend" report — mirrors the shape of the
/// `docker_unavailable` evidence the service path emits.
/// @spec apps/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#logic
#[derive(Debug, Clone)]
pub struct BackendUnavailable {
    pub requested: ClusterBackend,
    pub installed: Vec<&'static str>,
    pub docker: bool,
}

/// @spec apps/vat/tech-design/semantic/source/projects-vat-src-cluster-rs.md#source
impl BackendUnavailable {
    /// The requested backend as the token used in vat.toml / `--backend`.
    pub fn requested_name(&self) -> &'static str {
        backend_token(self.requested)
    }

    /// Human-readable summary for a bail message.
    pub fn message(&self) -> String {
        format!(
            "no usable cluster backend: requested `{}`, installed [{}], docker daemon {}",
            self.requested_name(),
            self.installed.join(", "),
            if self.docker { "up" } else { "down" }
        )
    }
}

/// The token used for a requested backend in vat.toml and `--backend`.
/// @spec apps/vat/tech-design/semantic/source/projects-vat-src-cluster-rs.md#source
pub fn backend_token(backend: ClusterBackend) -> &'static str {
    match backend {
        ClusterBackend::Auto => "auto",
        ClusterBackend::Kind => "kind",
        ClusterBackend::K3d => "k3d",
        ClusterBackend::Minikube => "minikube",
    }
}

/// Resolve a requested backend against the host: the requested (or, for `auto`,
/// the first installed) backend whose Docker daemon is reachable.
/// @spec apps/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#logic
pub fn resolve_backend(
    requested: ClusterBackend,
) -> std::result::Result<ResolvedBackend, BackendUnavailable> {
    let installed: Vec<ResolvedBackend> = ResolvedBackend::ALL
        .into_iter()
        .filter(|b| b.installed())
        .collect();
    pick_backend(requested, &installed, docker_daemon_up())
}

/// Pure backend selection — split out so it is deterministically testable
/// without touching PATH or the Docker daemon.
fn pick_backend(
    requested: ClusterBackend,
    installed: &[ResolvedBackend],
    docker: bool,
) -> std::result::Result<ResolvedBackend, BackendUnavailable> {
    let pick = match requested {
        ClusterBackend::Auto => installed.first().copied(),
        ClusterBackend::Kind => installed
            .iter()
            .copied()
            .find(|b| *b == ResolvedBackend::Kind),
        ClusterBackend::K3d => installed
            .iter()
            .copied()
            .find(|b| *b == ResolvedBackend::K3d),
        ClusterBackend::Minikube => installed
            .iter()
            .copied()
            .find(|b| *b == ResolvedBackend::Minikube),
    };
    match pick {
        Some(backend) if docker => Ok(backend),
        _ => Err(BackendUnavailable {
            requested,
            installed: installed.iter().map(|b| b.name()).collect(),
            docker,
        }),
    }
}

/// Build a collision-resistant, backend-safe cluster name from a vat id and a
/// service id. Lowercased, non-`[a-z0-9-]` mapped to `-`, length-capped so the
/// stricter backends (and the Docker resource names they derive) stay legal.
/// @spec apps/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#logic
pub fn cluster_name(vat_id: &str, service_id: &str) -> String {
    let mut name: String = format!("vat-{vat_id}-{service_id}")
        .chars()
        .map(|c| {
            let c = c.to_ascii_lowercase();
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();
    if name.len() > 32 {
        name.truncate(32);
    }
    let trimmed = name.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "vat-cluster".to_string()
    } else {
        trimmed
    }
}

/// A multi-node kind config: one control-plane plus `nodes - 1` workers.
fn kind_multinode_config(nodes: u32) -> String {
    let mut yaml = String::from(
        "kind: Cluster\napiVersion: kind.x-k8s.io/v1alpha4\nnodes:\n  - role: control-plane\n",
    );
    for _ in 1..nodes {
        yaml.push_str("  - role: worker\n");
    }
    yaml
}

/// Run a command to completion bounded by `timeout`, discarding output.
fn run_capture(cmd: Command, timeout: Duration, what: &str) -> Result<()> {
    run_capture_cancellable(cmd, timeout, what, &never_cancel)
}

fn run_capture_cancellable(
    mut cmd: Command,
    timeout: Duration,
    what: &str,
    cancellation: &dyn Fn() -> Result<()>,
) -> Result<()> {
    cmd.stdout(Stdio::null()).stderr(Stdio::null());
    let status = run_owned_command(cmd, timeout, what, cancellation)?;
    if status.success() {
        Ok(())
    } else {
        bail!("{what} failed with {:?}", status.code())
    }
}

fn run_output(cmd: Command, timeout: Duration, what: &str) -> Result<Vec<u8>> {
    run_output_cancellable(cmd, timeout, what, &never_cancel)
}

fn run_output_cancellable(
    mut cmd: Command,
    timeout: Duration,
    what: &str,
    cancellation: &dyn Fn() -> Result<()>,
) -> Result<Vec<u8>> {
    // A real temporary file avoids a pipe-capacity deadlock while the ordinary
    // owner polls cancellation and the deadline. It is unlinked on drop.
    let mut stdout = tempfile::tempfile().with_context(|| format!("capture {what} stdout"))?;
    let child_stdout = stdout
        .try_clone()
        .with_context(|| format!("clone {what} stdout capture"))?;
    cmd.stdout(Stdio::from(child_stdout)).stderr(Stdio::null());
    let enforce_capture_limit = || {
        let captured_len = stdout
            .metadata()
            .with_context(|| format!("inspect {what} stdout capture while command is running"))?
            .len();
        if captured_len > MAX_CAPTURE_BYTES {
            bail!(
                "{what} stdout exceeded the {} byte capture limit",
                MAX_CAPTURE_BYTES
            );
        }
        Ok(())
    };
    let status =
        run_owned_command_observed(cmd, timeout, what, cancellation, &enforce_capture_limit)?;
    if !status.success() {
        bail!("{what} failed with {:?}", status.code());
    }
    let captured_len = stdout
        .metadata()
        .with_context(|| format!("inspect {what} stdout capture"))?
        .len();
    if captured_len > MAX_CAPTURE_BYTES {
        bail!(
            "{what} stdout exceeded the {} byte capture limit",
            MAX_CAPTURE_BYTES
        );
    }
    stdout
        .seek(SeekFrom::Start(0))
        .with_context(|| format!("rewind {what} stdout capture"))?;
    let mut bytes = Vec::with_capacity(captured_len as usize);
    stdout
        .read_to_end(&mut bytes)
        .with_context(|| format!("read {what} stdout capture"))?;
    Ok(bytes)
}

fn run_owned_command(
    cmd: Command,
    timeout: Duration,
    what: &str,
    cancellation: &dyn Fn() -> Result<()>,
) -> Result<ExitStatus> {
    run_owned_command_observed(cmd, timeout, what, cancellation, &never_cancel)
}

fn run_owned_command_observed(
    cmd: Command,
    timeout: Duration,
    what: &str,
    cancellation: &dyn Fn() -> Result<()>,
    observation: &dyn Fn() -> Result<()>,
) -> Result<ExitStatus> {
    run_owned_command_observed_with_leader_probe(
        cmd,
        timeout,
        what,
        cancellation,
        observation,
        &child_has_exited_without_reap,
    )
}

fn run_owned_command_observed_with_leader_probe(
    mut cmd: Command,
    timeout: Duration,
    what: &str,
    cancellation: &dyn Fn() -> Result<()>,
    observation: &dyn Fn() -> Result<()>,
    leader_observation: &dyn Fn(&Child) -> Result<bool>,
) -> Result<ExitStatus> {
    #[cfg(unix)]
    cmd.process_group(0);
    let mut child = cmd
        .stdin(Stdio::null())
        .spawn()
        .with_context(|| format!("spawn {what}"))?;
    let owned_pgid = child.id();
    let deadline = Instant::now() + timeout;
    loop {
        if let Err(observation_error) = observation() {
            return match terminate_owned_command(&mut child, owned_pgid, what) {
                Ok(_) => Err(observation_error),
                Err(cleanup_error) => Err(OwnedCommandCleanupFailed {
                    detail: format!(
                        "{what} observation failed ({observation_error:#}) and owned process-group cleanup is unconfirmed: {cleanup_error:#}"
                    ),
                }
                .into()),
            };
        }
        if let Err(cancellation_error) = cancellation() {
            return match terminate_owned_command(&mut child, owned_pgid, what) {
                Ok(_) => Err(cancellation_error),
                Err(cleanup_error) => Err(OwnedCommandCleanupFailed {
                    detail: format!(
                        "{what} cancellation cleanup is unconfirmed after {cancellation_error:#}: {cleanup_error:#}"
                    ),
                }
                .into()),
            };
        }
        #[cfg(unix)]
        match leader_observation(&child) {
            Ok(true) => {
                return terminate_owned_command(&mut child, owned_pgid, what).map_err(|error| {
                    OwnedCommandCleanupFailed {
                        detail: format!(
                            "{what} completed but owned process-group cleanup is unconfirmed: {error:#}"
                        ),
                    }
                    .into()
                });
            }
            Ok(false) => {}
            Err(observation_error) => {
                return match terminate_owned_command(&mut child, owned_pgid, what) {
                    Ok(_) => Err(observation_error)
                        .with_context(|| format!("observe {what} process-group leader")),
                    Err(cleanup_error) => Err(OwnedCommandCleanupFailed {
                        detail: format!(
                            "{what} leader observation failed ({observation_error:#}) and owned process-group cleanup is unconfirmed: {cleanup_error:#}"
                        ),
                    }
                    .into()),
                };
            }
        }
        #[cfg(not(unix))]
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        if Instant::now() >= deadline {
            return match terminate_owned_command(&mut child, owned_pgid, what) {
                Ok(_) => bail!("{what} timed out after {}ms", timeout.as_millis()),
                Err(cleanup_error) => Err(OwnedCommandCleanupFailed {
                    detail: format!(
                        "{what} timed out after {}ms and owned process-group cleanup is unconfirmed: {cleanup_error:#}",
                        timeout.as_millis()
                    ),
                }
                .into()),
            };
        }
        std::thread::sleep(COMMAND_POLL_INTERVAL);
    }
}

#[cfg(unix)]
fn terminate_owned_command(child: &mut Child, pgid: u32, what: &str) -> Result<ExitStatus> {
    let term_permission_partial = signal_command_group(pgid, libc::SIGTERM, what)?;
    let grace_deadline = Instant::now() + COMMAND_TERM_GRACE;
    while Instant::now() < grace_deadline {
        if !command_group_exists(pgid)? {
            break;
        }
        std::thread::sleep(COMMAND_POLL_INTERVAL);
    }

    let kill_permission_partial = if command_group_exists(pgid)? {
        signal_command_group(pgid, libc::SIGKILL, what)?
    } else {
        false
    };
    // Keep the leader unreaped until every group-directed signal is complete.
    // Reaping releases the numeric PID/PGID identity; sending SIGKILL to that
    // number afterward could target a reused, unrelated process group.
    if (term_permission_partial || kill_permission_partial)
        && !child_has_exited_without_reap(child)?
    {
        match child.kill() {
            Ok(()) => {}
            Err(error) if error.raw_os_error() == Some(libc::ESRCH) => {}
            Err(error) => return Err(error).with_context(|| format!("KILL {what} leader")),
        }
    }

    let reap_deadline = Instant::now() + COMMAND_REAP_TIMEOUT;
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .with_context(|| format!("reap {what} process-group leader {pgid}"))?
        {
            break status;
        }
        if Instant::now() >= reap_deadline {
            bail!("{what} process-group leader {pgid} did not exit after TERM/KILL");
        }
        std::thread::sleep(COMMAND_POLL_INTERVAL);
    };

    let absence_deadline = Instant::now() + COMMAND_REAP_TIMEOUT;
    while command_group_exists(pgid)? && Instant::now() < absence_deadline {
        std::thread::sleep(COMMAND_POLL_INTERVAL);
    }
    if command_group_exists(pgid)? {
        bail!("{what} process group {pgid} remains after TERM/KILL and leader reap");
    }
    Ok(status)
}

#[cfg(not(unix))]
fn terminate_owned_command(child: &mut Child, _pgid: u32, what: &str) -> Result<ExitStatus> {
    match child.try_wait()? {
        Some(status) => Ok(status),
        None => {
            child.kill().with_context(|| format!("stop {what} child"))?;
            child.wait().with_context(|| format!("reap {what} child"))
        }
    }
}

#[cfg(unix)]
fn child_has_exited_without_reap(child: &Child) -> Result<bool> {
    let mut info = unsafe { std::mem::zeroed::<libc::siginfo_t>() };
    let result = unsafe {
        libc::waitid(
            libc::P_PID,
            child.id() as libc::id_t,
            &mut info,
            libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
        )
    };
    if result != 0 {
        return Err(std::io::Error::last_os_error())
            .context("observe cluster backend process-group leader without reaping");
    }
    Ok(unsafe { info.si_pid() } != 0)
}

#[cfg(unix)]
fn signal_command_group(pgid: u32, signal: i32, what: &str) -> Result<bool> {
    let result = unsafe { libc::kill(-(pgid as i32), signal) };
    if result == 0 {
        return Ok(false);
    }
    let error = std::io::Error::last_os_error();
    match error.raw_os_error() {
        Some(libc::ESRCH) => Ok(false),
        Some(libc::EPERM) => Ok(true),
        _ => Err(error)
            .with_context(|| format!("send signal {signal} to {what} process group {pgid}")),
    }
}

#[cfg(unix)]
fn command_group_exists(pgid: u32) -> Result<bool> {
    let result = unsafe { libc::kill(-(pgid as i32), 0) };
    if result == 0 {
        return Ok(true);
    }
    let error = std::io::Error::last_os_error();
    match error.raw_os_error() {
        Some(libc::ESRCH) => Ok(false),
        Some(libc::EPERM) => Ok(true),
        _ => Err(error).with_context(|| format!("inspect cluster backend process group {pgid}")),
    }
}

/// Whether the Docker daemon answers — every backend needs it on macOS.
fn docker_daemon_up() -> bool {
    let mut command = Command::new("docker");
    command
        .arg("info")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    run_owned_command(
        command,
        DOCKER_PROBE_TIMEOUT,
        "docker daemon probe",
        &never_cancel,
    )
    .map(|status| status.success())
    .unwrap_or(false)
}

fn which(binary: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(binary);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct FakeCancellation;

    impl fmt::Display for FakeCancellation {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("fake cluster cancellation")
        }
    }

    impl std::error::Error for FakeCancellation {}

    #[test]
    fn cluster_name_sanitizes_and_bounds() {
        let name = cluster_name("vat-7F3.k1q9", "my.E2E/svc");
        assert!(name.len() <= 32);
        assert!(!name.is_empty());
        assert!(name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'));
        assert!(!name.starts_with('-') && !name.ends_with('-'));
    }

    #[test]
    fn pick_backend_auto_prefers_first_installed() {
        let got = pick_backend(ClusterBackend::Auto, &[ResolvedBackend::K3d], true);
        assert!(matches!(got, Ok(ResolvedBackend::K3d)));
    }

    #[test]
    fn pick_backend_forced_must_match_installed() {
        let err = pick_backend(ClusterBackend::Kind, &[ResolvedBackend::K3d], true);
        assert!(err.is_err());
    }

    #[test]
    fn pick_backend_unavailable_without_docker() {
        let err = pick_backend(ClusterBackend::Auto, &[ResolvedBackend::Kind], false);
        let unavailable = err.expect_err("no docker means unavailable");
        assert!(!unavailable.docker);
        assert_eq!(unavailable.installed, vec!["kind"]);
    }

    #[test]
    fn pick_backend_unavailable_without_any_backend() {
        let err = pick_backend(ClusterBackend::Auto, &[], true);
        let unavailable = err.expect_err("no backend means unavailable");
        assert!(unavailable.installed.is_empty());
        assert_eq!(unavailable.requested_name(), "auto");
    }

    #[test]
    fn resolve_backend_does_not_panic() {
        // Whatever the host looks like, resolution returns a value, never panics.
        let _ = resolve_backend(ClusterBackend::Auto);
    }

    #[test]
    fn kind_multinode_config_has_workers() {
        let cfg = kind_multinode_config(3);
        assert!(cfg.contains("control-plane"));
        assert_eq!(cfg.matches("role: worker").count(), 2);
    }

    #[cfg(unix)]
    #[test]
    fn captured_command_timeout_is_bounded() {
        let temp = tempfile::tempdir().expect("tempdir");
        let leader_marker = temp.path().join("leader.pid");
        let descendant_marker = temp.path().join("descendant.pid");
        let mut command = Command::new("/bin/sh");
        command
            .env("LEADER_MARKER", &leader_marker)
            .env("DESCENDANT_MARKER", &descendant_marker)
            .args([
                "-c",
                "echo $$ > \"$LEADER_MARKER\"; /bin/sh -c 'trap : TERM; echo $$ > \"$DESCENDANT_MARKER\"; while :; do sleep 1; done' & trap : TERM; while :; do sleep 1; done",
            ]);
        let started = Instant::now();
        let error = run_capture(
            command,
            Duration::from_millis(200),
            "injected cluster cleanup",
        )
        .expect_err("command must time out");
        assert!(error.to_string().contains("timed out"));
        assert!(started.elapsed() < Duration::from_secs(2));
        assert_pid_markers_absent(&[leader_marker, descendant_marker]);
    }

    #[cfg(unix)]
    #[test]
    fn captured_command_fake_cancellation_cleans_term_resistant_descendant() {
        let temp = tempfile::tempdir().expect("tempdir");
        let leader_marker = temp.path().join("leader.pid");
        let descendant_marker = temp.path().join("descendant.pid");
        let mut command = Command::new("/bin/sh");
        command
            .env("LEADER_MARKER", &leader_marker)
            .env("DESCENDANT_MARKER", &descendant_marker)
            .args([
                "-c",
                "echo $$ > \"$LEADER_MARKER\"; /bin/sh -c 'trap : TERM; echo $$ > \"$DESCENDANT_MARKER\"; while :; do sleep 1; done' & trap : TERM; while :; do sleep 1; done",
            ]);
        let cancellation = || {
            if leader_marker.exists() && descendant_marker.exists() {
                Err(FakeCancellation.into())
            } else {
                Ok(())
            }
        };
        let started = Instant::now();
        let error = run_capture_cancellable(
            command,
            Duration::from_secs(5),
            "fake-cancelled cluster create",
            &cancellation,
        )
        .expect_err("command must observe fake cancellation");
        assert!(error.downcast_ref::<FakeCancellation>().is_some());
        assert!(started.elapsed() >= COMMAND_TERM_GRACE);
        assert!(started.elapsed() < Duration::from_secs(2));
        assert_pid_markers_absent(&[leader_marker, descendant_marker]);
    }

    #[cfg(unix)]
    #[test]
    fn leader_observation_error_still_cleans_owned_process_group() {
        let temp = tempfile::tempdir().expect("tempdir");
        let leader_marker = temp.path().join("leader.pid");
        let descendant_marker = temp.path().join("descendant.pid");
        let mut command = Command::new("/bin/sh");
        command
            .env("LEADER_MARKER", &leader_marker)
            .env("DESCENDANT_MARKER", &descendant_marker)
            .args([
                "-c",
                "echo $$ > \"$LEADER_MARKER\"; /bin/sh -c 'trap : TERM; echo $$ > \"$DESCENDANT_MARKER\"; while :; do sleep 1; done' & trap : TERM; while :; do sleep 1; done",
            ]);
        let leader_probe = |_: &Child| {
            if leader_marker.exists() && descendant_marker.exists() {
                bail!("injected waitid observation failure");
            }
            Ok(false)
        };
        let error = run_owned_command_observed_with_leader_probe(
            command,
            Duration::from_secs(5),
            "observation-error cluster command",
            &never_cancel,
            &never_cancel,
            &leader_probe,
        )
        .expect_err("leader observation error must propagate after cleanup");
        assert!(error
            .to_string()
            .contains("observe observation-error cluster command"));
        assert_pid_markers_absent(&[leader_marker, descendant_marker]);
    }

    #[test]
    fn captured_backend_output_is_bounded_and_complete() {
        let mut command = Command::new("/bin/sh");
        command.args(["-c", "printf 'isolated-kubeconfig'"]);
        let output = run_output(command, Duration::from_secs(1), "fake kubeconfig capture")
            .expect("capture fake kubeconfig");
        assert_eq!(output, b"isolated-kubeconfig");
    }

    #[cfg(unix)]
    #[test]
    fn captured_output_limit_terminates_command_before_its_runtime_deadline() {
        let temp = tempfile::tempdir().expect("tempdir");
        let leader_marker = temp.path().join("leader.pid");
        let mut command = Command::new("/bin/sh");
        command.env("LEADER_MARKER", &leader_marker).args([
            "-c",
            "echo $$ > \"$LEADER_MARKER\"; dd if=/dev/zero bs=1048576 count=5 2>/dev/null; sleep 10",
        ]);
        let started = Instant::now();
        let error = run_output(command, Duration::from_secs(5), "oversized cluster output")
            .expect_err("capture must stop at its byte budget while the child is live");
        assert!(error.to_string().contains("capture limit"));
        assert!(started.elapsed() < Duration::from_secs(2));
        assert_pid_markers_absent(&[leader_marker]);
    }

    #[cfg(unix)]
    #[test]
    fn term_exited_leader_stays_unreaped_until_descendant_group_is_killed() {
        let temp = tempfile::tempdir().expect("tempdir");
        let leader_marker = temp.path().join("leader.pid");
        let descendant_marker = temp.path().join("descendant.pid");
        let mut command = Command::new("/bin/sh");
        command
            .env("LEADER_MARKER", &leader_marker)
            .env("DESCENDANT_MARKER", &descendant_marker)
            .args([
                "-c",
                "echo $$ > \"$LEADER_MARKER\"; /bin/sh -c 'trap \"\" TERM; echo $$ > \"$DESCENDANT_MARKER\"; while :; do :; done' & trap 'exit 0' TERM; while :; do :; done",
            ]);
        let error = run_capture(
            command,
            Duration::from_millis(150),
            "leader-exits-before-descendant",
        )
        .expect_err("fixture must time out and clean the whole group");
        assert!(error.to_string().contains("timed out"));
        assert_pid_markers_absent(&[leader_marker, descendant_marker]);
    }

    #[cfg(unix)]
    fn assert_pid_markers_absent(markers: &[PathBuf]) {
        for marker in markers {
            let pid = std::fs::read_to_string(marker)
                .unwrap_or_else(|error| panic!("read {}: {error}", marker.display()))
                .trim()
                .parse::<i32>()
                .expect("fixture pid");
            let deadline = Instant::now() + Duration::from_secs(1);
            loop {
                let result = unsafe { libc::kill(pid, 0) };
                if result == -1
                    && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
                {
                    break;
                }
                assert!(Instant::now() < deadline, "pid {pid} survived cleanup");
                std::thread::sleep(COMMAND_POLL_INTERVAL);
            }
        }
    }
}
// CODEGEN-END
