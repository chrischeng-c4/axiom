// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-commands-cluster-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! `vat cluster` — manage standalone local Kubernetes clusters.
//!
//! Unlike a run-scoped cluster service, these clusters outlive a single run so
//! an agent can iterate against one. vat does not *supervise* them (no daemon,
//! no restart policy) — it only creates, lists, deletes, and reports kubeconfig
//! on explicit command, exactly like kind/k3d/minikube themselves. Each cluster
//! gets a registry directory under `<root>/clusters/<name>/` holding its
//! metadata and an isolated kubeconfig; vat never touches `~/.kube/config`.

use std::process::ExitCode;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::cluster::{self, ClusterSpec, ResolvedBackend};
use crate::config::ClusterBackend;
use crate::{id, paths};

/// Default standalone create timeout — clusters take minutes to come up.
const CREATE_TIMEOUT: Duration = Duration::from_secs(600);

/// Persisted registry entry for a standalone cluster
/// (`<root>/clusters/<name>/cluster.json`).
/// @spec projects/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterRecord {
    pub backend: String,
    pub name: String,
    pub kubeconfig: String,
    pub node_count: u32,
    pub created_at: String,
}

/// `vat cluster create` — resolve a backend and create a standalone cluster.
/// @spec projects/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#cli
pub fn create(
    name: Option<String>,
    backend: ClusterBackend,
    k8s_version: Option<String>,
    nodes: u32,
    json: bool,
) -> Result<ExitCode> {
    let resolved = match cluster::resolve_backend(backend) {
        Ok(resolved) => resolved,
        Err(unavailable) => {
            println!(
                "{}",
                serde_json::to_string(&serde_json::json!({
                    "type": "error",
                    "code": "cluster_backend_unavailable",
                    "requested": unavailable.requested_name(),
                    "installed": unavailable.installed,
                    "docker": unavailable.docker,
                }))?
            );
            return Ok(ExitCode::FAILURE);
        }
    };

    let name = match name {
        Some(name) => name,
        None => default_cluster_name(),
    };
    let dir = paths::cluster_dir(&name)?;
    if dir.exists() {
        bail!("cluster `{name}` already exists in the vat registry");
    }
    if resolved
        .list()
        .unwrap_or_default()
        .iter()
        .any(|c| c == &name)
    {
        bail!(
            "cluster `{name}` already exists in the {} backend",
            resolved.name()
        );
    }

    std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    let kubeconfig = dir.join("kubeconfig");
    let spec = ClusterSpec {
        name: &name,
        k8s_version: k8s_version.as_deref(),
        nodes,
        kubeconfig: &kubeconfig,
    };
    let info = match resolved.create(&spec, CREATE_TIMEOUT) {
        Ok(info) => info,
        Err(err) => {
            // Leave nothing behind on a failed create.
            let _ = resolved.delete(&name);
            let _ = std::fs::remove_dir_all(&dir);
            return Err(err).with_context(|| format!("create cluster `{name}`"));
        }
    };

    let record = ClusterRecord {
        backend: info.backend.to_string(),
        name: info.name.clone(),
        kubeconfig: info.kubeconfig.to_string_lossy().into_owned(),
        node_count: info.node_count,
        created_at: Utc::now().to_rfc3339(),
    };
    std::fs::write(
        dir.join("cluster.json"),
        serde_json::to_vec_pretty(&record)?,
    )
    .with_context(|| format!("write registry for cluster `{name}`"))?;

    if json {
        crate::commands::print_json(&record, false)?;
    } else {
        println!("created {} cluster `{}`", record.backend, record.name);
        println!("kubeconfig {}", record.kubeconfig);
    }
    Ok(ExitCode::SUCCESS)
}

/// `vat cluster ls` — list registry clusters, marking any missing from their
/// backend as stale.
/// @spec projects/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#cli
pub fn ls(json: bool) -> Result<ExitCode> {
    let records = read_registry()?;
    // Reconcile against each backend's live list once.
    let mut entries = Vec::new();
    for record in records {
        let live = ResolvedBackend::from_name(&record.backend)
            .map(|b| b.list().unwrap_or_default())
            .unwrap_or_default();
        let stale = !live.iter().any(|c| c == &record.name);
        entries.push((record, stale));
    }

    if json {
        let value: Vec<serde_json::Value> = entries
            .iter()
            .map(|(record, stale)| {
                serde_json::json!({
                    "backend": record.backend,
                    "name": record.name,
                    "kubeconfig": record.kubeconfig,
                    "node_count": record.node_count,
                    "created_at": record.created_at,
                    "stale": stale,
                })
            })
            .collect();
        crate::commands::print_json(&value, false)?;
    } else if entries.is_empty() {
        println!("no vat-managed clusters");
    } else {
        for (record, stale) in &entries {
            let mark = if *stale { " (stale)" } else { "" };
            println!(
                "{}  {}  {}{}",
                record.name, record.backend, record.kubeconfig, mark
            );
        }
    }
    Ok(ExitCode::SUCCESS)
}

/// `vat cluster kubeconfig` — print the isolated kubeconfig path for a cluster.
/// @spec projects/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#cli
pub fn kubeconfig(name: String, json: bool) -> Result<ExitCode> {
    let record = load_record(&name)?;
    if json {
        crate::commands::print_json(&record, false)?;
    } else {
        println!("{}", record.kubeconfig);
    }
    Ok(ExitCode::SUCCESS)
}

/// `vat cluster delete` — delete the cluster via its backend, then remove the
/// registry entry.
/// @spec projects/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#cli
pub fn delete(name: String, json: bool) -> Result<ExitCode> {
    let record = load_record(&name)?;
    if let Some(backend) = ResolvedBackend::from_name(&record.backend) {
        backend
            .delete(&record.name)
            .with_context(|| format!("delete cluster `{name}`"))?;
    }
    let dir = paths::cluster_dir(&name)?;
    std::fs::remove_dir_all(&dir).with_context(|| format!("remove registry for `{name}`"))?;
    if json {
        crate::commands::print_json(
            &serde_json::json!({ "deleted": name, "backend": record.backend }),
            false,
        )?;
    } else {
        println!("deleted {} cluster `{}`", record.backend, name);
    }
    Ok(ExitCode::SUCCESS)
}

fn default_cluster_name() -> String {
    let id = id::fresh();
    format!("vat-cluster-{}", id.strip_prefix("vat-").unwrap_or(&id))
}

fn read_registry() -> Result<Vec<ClusterRecord>> {
    let dir = paths::clusters_dir()?;
    let mut records = Vec::new();
    if !dir.exists() {
        return Ok(records);
    }
    for entry in std::fs::read_dir(&dir).with_context(|| format!("read {}", dir.display()))? {
        let entry = entry?;
        let manifest = entry.path().join("cluster.json");
        if let Ok(bytes) = std::fs::read(&manifest) {
            if let Ok(record) = serde_json::from_slice::<ClusterRecord>(&bytes) {
                records.push(record);
            }
        }
    }
    records.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(records)
}

fn load_record(name: &str) -> Result<ClusterRecord> {
    let manifest = paths::cluster_dir(name)?.join("cluster.json");
    if !manifest.exists() {
        bail!("unknown cluster `{name}` (not in the vat registry)");
    }
    let bytes = std::fs::read(&manifest).with_context(|| format!("read {}", manifest.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse registry for `{name}`"))
}
// CODEGEN-END
