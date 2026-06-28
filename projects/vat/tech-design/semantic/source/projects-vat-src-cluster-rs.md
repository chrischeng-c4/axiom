---
id: projects-vat-src-cluster-rs
summary: >
  rust-source-unit TD AST payload for projects/vat/src/cluster.rs.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This rust-source-unit TD preserves vat source ownership while migrating #39 off group-level source replay."
---

# Standardized projects/vat/src/cluster.rs

## Overview
<!-- type: overview lang: markdown -->

Rust source-unit TD for `projects/vat/src/cluster.rs`, captured during #39 vat migration onto td_ast lossless source generation.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
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

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};

use crate::config::ClusterBackend;

/// A concrete cluster backend resolved against the host.
/// @spec projects/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#logic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedBackend {
    Kind,
    K3d,
    Minikube,
}

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
        if let Some(parent) = spec.kubeconfig.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        match self {
            Self::Kind => self.create_kind(spec, timeout),
            Self::K3d => self.create_k3d(spec, timeout),
            Self::Minikube => self.create_minikube(spec, timeout),
        }
    }

    fn create_kind(self, spec: &ClusterSpec, timeout: Duration) -> Result<ClusterInfo> {
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
        run_capture(
            cmd,
            timeout + Duration::from_secs(30),
            "kind create cluster",
        )?;
        Ok(self.info(spec))
    }

    fn create_k3d(self, spec: &ClusterSpec, timeout: Duration) -> Result<ClusterInfo> {
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
        run_capture(cmd, timeout + Duration::from_secs(30), "k3d cluster create")?;
        // k3d writes the kubeconfig to stdout; capture it into the isolated file.
        let out = Command::new("k3d")
            .args(["kubeconfig", "get", spec.name])
            .stderr(Stdio::null())
            .output()
            .context("k3d kubeconfig get")?;
        if !out.status.success() {
            bail!("k3d kubeconfig get failed for cluster `{}`", spec.name);
        }
        std::fs::write(spec.kubeconfig, &out.stdout)
            .with_context(|| format!("write {}", spec.kubeconfig.display()))?;
        Ok(self.info(spec))
    }

    fn create_minikube(self, spec: &ClusterSpec, timeout: Duration) -> Result<ClusterInfo> {
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
        run_capture(cmd, timeout + Duration::from_secs(30), "minikube start")?;
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
        let mut cmd = match self {
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
        let status = cmd
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .with_context(|| format!("{} delete cluster", self.name()))?;
        if !status.success() {
            bail!("{} failed to delete cluster `{name}`", self.name());
        }
        Ok(())
    }

    /// List cluster names this backend currently owns.
    pub fn list(self) -> Result<Vec<String>> {
        match self {
            Self::Kind => {
                let out = Command::new("kind")
                    .args(["get", "clusters"])
                    .stderr(Stdio::null())
                    .output()
                    .context("kind get clusters")?;
                Ok(String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .map(str::trim)
                    .filter(|l| !l.is_empty() && *l != "No kind clusters found.")
                    .map(String::from)
                    .collect())
            }
            Self::K3d => {
                let out = Command::new("k3d")
                    .args(["cluster", "list", "-o", "json"])
                    .stderr(Stdio::null())
                    .output()
                    .context("k3d cluster list")?;
                let value: serde_json::Value =
                    serde_json::from_slice(&out.stdout).unwrap_or(serde_json::Value::Null);
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
                let out = Command::new("minikube")
                    .args(["profile", "list", "-o", "json"])
                    .stderr(Stdio::null())
                    .output()
                    .context("minikube profile list")?;
                let value: serde_json::Value =
                    serde_json::from_slice(&out.stdout).unwrap_or(serde_json::Value::Null);
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
pub struct ClusterSpec<'a> {
    pub name: &'a str,
    pub k8s_version: Option<&'a str>,
    pub nodes: u32,
    pub kubeconfig: &'a Path,
}

/// Result of creating or inspecting a cluster.
#[derive(Debug, Clone)]
pub struct ClusterInfo {
    pub backend: &'static str,
    pub name: String,
    pub kubeconfig: PathBuf,
    pub node_count: u32,
}

/// Structured "no usable cluster backend" report — mirrors the shape of the
/// `docker_unavailable` evidence the service path emits.
/// @spec projects/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#logic
#[derive(Debug, Clone)]
pub struct BackendUnavailable {
    pub requested: ClusterBackend,
    pub installed: Vec<&'static str>,
    pub docker: bool,
}

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
/// @spec projects/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#logic
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
/// @spec projects/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#logic
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

/// Run a command to completion bounded by `timeout`, discarding output. The
/// child is killed on overrun. stdout/stderr go to null to avoid pipe
/// deadlocks on long-running creates.
fn run_capture(mut cmd: Command, timeout: Duration, what: &str) -> Result<()> {
    let mut child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("spawn {what}"))?;
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait()? {
            if status.success() {
                return Ok(());
            }
            bail!("{what} failed with {:?}", status.code());
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            bail!("{what} timed out after {}s", timeout.as_secs());
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// Whether the Docker daemon answers — every backend needs it on macOS.
fn docker_daemon_up() -> bool {
    Command::new("docker")
        .arg("info")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
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
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/cluster.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/vat/src/cluster.rs` captured during #39 vat standardization.
```
