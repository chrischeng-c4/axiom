// SPEC-MANAGED: apps/vat/tech-design/semantic/source/projects-vat-src-commands-cluster-rs.md#rust-source-unit
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
/// @spec apps/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterRecord {
    pub backend: String,
    pub name: String,
    pub kubeconfig: String,
    pub node_count: u32,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub create_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cleanup_error: Option<String>,
}

/// `vat cluster create` — resolve a backend and create a standalone cluster.
/// @spec apps/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#cli
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
    ensure_backend_name_available(&name, resolved.name(), resolved.list())?;

    std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    let kubeconfig = dir.join("kubeconfig");
    let spec = ClusterSpec {
        name: &name,
        k8s_version: k8s_version.as_deref(),
        nodes,
        kubeconfig: &kubeconfig,
    };
    // Persist the backend/name ownership identity before the backend can
    // create anything. If create and compensating delete both fail, this is
    // the durable recovery handle used by `vat cluster delete`.
    let mut record = ClusterRecord {
        backend: resolved.name().to_string(),
        name: name.clone(),
        kubeconfig: kubeconfig.to_string_lossy().into_owned(),
        node_count: nodes,
        created_at: Utc::now().to_rfc3339(),
        create_error: None,
        cleanup_error: None,
    };
    persist_cluster_record(&dir, &record)?;
    let info = match resolved.create(&spec, CREATE_TIMEOUT) {
        Ok(info) => info,
        Err(err) => {
            retain_failed_create(&dir, &mut record, &err)?;
            return Err(err).context(format!(
                "create cluster `{name}` outcome is unconfirmed; automatic same-name cleanup was skipped and registry retained at {}",
                dir.display()
            ));
        }
    };

    record = ClusterRecord {
        backend: info.backend.to_string(),
        name: info.name.clone(),
        kubeconfig: info.kubeconfig.to_string_lossy().into_owned(),
        node_count: info.node_count,
        created_at: record.created_at,
        create_error: None,
        cleanup_error: None,
    };
    persist_cluster_record(&dir, &record)?;

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
/// @spec apps/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#cli
pub fn ls(json: bool) -> Result<ExitCode> {
    let records = read_registry()?;
    // Reconcile against each backend's live list once.
    let mut entries = Vec::new();
    for record in records {
        let backend = resolve_record_backend(&record)
            .context("refusing to mark an unknown-backend registry entry stale")?;
        let live = backend.list().with_context(|| {
            format!(
                "list {} backend while reconciling cluster `{}`",
                record.backend, record.name
            )
        })?;
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
                    "create_error": record.create_error,
                    "cleanup_error": record.cleanup_error,
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
/// @spec apps/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#cli
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
/// @spec apps/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#cli
pub fn delete(name: String, json: bool) -> Result<ExitCode> {
    let record = load_record(&name)?;
    if record.name != name {
        bail!(
            "cluster registry directory `{name}` claims a different backend identity `{}`; refusing destructive cleanup",
            record.name
        );
    }
    let backend = resolve_record_backend(&record).context("registry retained")?;
    if record
        .cleanup_error
        .as_deref()
        .is_some_and(|error| error.contains(STANDALONE_COMMAND_CLEANUP_START))
    {
        bail!(
            "cluster `{name}` retains an unconfirmed create-command process group; a resource delete cannot prove that PGID absent, so registry is retained for manual recovery"
        );
    }
    let live = backend.list().with_context(|| {
        format!(
            "list {} backend before deleting cluster `{name}`; registry retained",
            record.backend
        )
    })?;
    if live.iter().any(|candidate| candidate == &record.name) {
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

fn persist_cluster_record(dir: &std::path::Path, record: &ClusterRecord) -> Result<()> {
    let destination = dir.join("cluster.json");
    let temporary = dir.join(format!(".cluster.{}.tmp", id::fresh()));
    let result = (|| -> Result<()> {
        std::fs::write(&temporary, serde_json::to_vec_pretty(record)?)
            .with_context(|| format!("write temporary registry for cluster `{}`", record.name))?;
        std::fs::rename(&temporary, &destination)
            .with_context(|| format!("publish registry for cluster `{}`", record.name))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

const STANDALONE_COMMAND_CLEANUP_START: &str = "[vat-cluster-command-cleanup]";
const STANDALONE_COMMAND_CLEANUP_END: &str = "[/vat-cluster-command-cleanup]";

fn ensure_backend_name_available(
    name: &str,
    backend_name: &str,
    live: Result<Vec<String>>,
) -> Result<()> {
    let live = live
        .with_context(|| format!("list {backend_name} backend before creating cluster `{name}`"))?;
    if live.iter().any(|cluster| cluster == name) {
        bail!("cluster `{name}` already exists in the {backend_name} backend");
    }
    Ok(())
}

fn resolve_record_backend(record: &ClusterRecord) -> Result<ResolvedBackend> {
    ResolvedBackend::from_name(&record.backend).with_context(|| {
        format!(
            "cluster `{}` registry names unknown backend `{}`",
            record.name, record.backend
        )
    })
}

fn retain_failed_create(
    dir: &std::path::Path,
    record: &mut ClusterRecord,
    create_error: &anyhow::Error,
) -> Result<()> {
    record.create_error = Some(format!("{create_error:#}"));
    record.cleanup_error = Some(
        cluster::owned_command_cleanup_failure(create_error)
            .map(|detail| {
                format!(
                    "{STANDALONE_COMMAND_CLEANUP_START} create-command process-group cleanup unconfirmed: {detail}{STANDALONE_COMMAND_CLEANUP_END}"
                )
            })
            .unwrap_or_else(|| {
                "cluster create outcome unknown; automatic same-name delete skipped because no unique backend ownership token exists; inspect the backend and recover manually"
                    .to_string()
            }),
    );
    persist_cluster_record(dir, record).with_context(|| {
        format!(
            "persist failed create outcome for cluster `{}`; initial registry remains at {}",
            record.name,
            dir.display()
        )
    })
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
        if !manifest.exists() {
            continue;
        }
        let bytes = std::fs::read(&manifest)
            .with_context(|| format!("read cluster registry {}", manifest.display()))?;
        let record = serde_json::from_slice::<ClusterRecord>(&bytes)
            .with_context(|| format!("parse cluster registry {}", manifest.display()))?;
        records.push(record);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn creating_record(dir: &std::path::Path) -> ClusterRecord {
        ClusterRecord {
            backend: "kind".to_string(),
            name: "owned-cluster".to_string(),
            kubeconfig: dir.join("kubeconfig").to_string_lossy().into_owned(),
            node_count: 1,
            created_at: Utc::now().to_rfc3339(),
            create_error: None,
            cleanup_error: None,
        }
    }

    #[test]
    fn generic_create_failure_retains_outcome_unknown_identity_without_auto_delete() {
        let temp = tempfile::tempdir().expect("cluster registry tempdir");
        let dir = temp.path().join("owned-cluster");
        std::fs::create_dir_all(&dir).expect("create registry dir");
        let mut record = creating_record(&dir);
        persist_cluster_record(&dir, &record).expect("persist pre-create identity");

        retain_failed_create(
            &dir,
            &mut record,
            &anyhow::anyhow!("injected create failure"),
        )
        .expect("retain outcome-unknown create");
        assert!(dir.exists());
        let persisted: ClusterRecord = serde_json::from_slice(
            &std::fs::read(dir.join("cluster.json")).expect("read retained registry"),
        )
        .expect("parse retained registry");
        assert_eq!(persisted.backend, "kind");
        assert_eq!(persisted.name, "owned-cluster");
        assert_eq!(
            persisted.create_error.as_deref(),
            Some("injected create failure")
        );
        assert!(persisted
            .cleanup_error
            .as_deref()
            .is_some_and(|error| error.contains("automatic same-name delete skipped")));
    }

    #[test]
    fn owned_command_cleanup_failure_retains_registry_and_cli_pgid_obligation() {
        let temp = tempfile::tempdir().expect("cluster registry tempdir");
        let dir = temp.path().join("owned-cluster");
        std::fs::create_dir_all(&dir).expect("create registry dir");
        let mut record = creating_record(&dir);
        persist_cluster_record(&dir, &record).expect("persist pre-create identity");

        let create_error = cluster::injected_owned_command_cleanup_failure(
            "create command PGID 4242 remains after TERM/KILL",
        );
        retain_failed_create(&dir, &mut record, &create_error)
            .expect("retain command cleanup obligation");

        assert!(dir.exists());
        let persisted: ClusterRecord = serde_json::from_slice(
            &std::fs::read(dir.join("cluster.json")).expect("read retained registry"),
        )
        .expect("parse retained registry");
        assert!(persisted.cleanup_error.as_deref().is_some_and(|error| {
            error.contains(STANDALONE_COMMAND_CLEANUP_START) && error.contains("PGID 4242")
        }));
    }

    #[test]
    fn backend_list_failure_is_not_an_empty_absence_proof() {
        let error = ensure_backend_name_available(
            "owned-cluster",
            "kind",
            Err(anyhow::anyhow!("injected backend list failure")),
        )
        .expect_err("list failure must block create");
        assert!(error.to_string().contains("list kind backend"));

        let duplicate = ensure_backend_name_available(
            "owned-cluster",
            "kind",
            Ok(vec!["owned-cluster".to_string()]),
        )
        .expect_err("exact backend name must block create");
        assert!(duplicate.to_string().contains("already exists"));
    }

    #[test]
    fn unknown_backend_registry_is_not_a_delete_or_stale_proof() {
        let temp = tempfile::tempdir().expect("cluster registry tempdir");
        let dir = temp.path().join("owned-cluster");
        let mut record = creating_record(&dir);
        record.backend = "future-backend".to_string();
        let error = resolve_record_backend(&record).expect_err("unknown backend must fail closed");
        assert!(error.to_string().contains("unknown backend"));
    }
}
// CODEGEN-END
