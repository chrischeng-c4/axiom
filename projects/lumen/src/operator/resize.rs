// HANDWRITE-BEGIN gap="missing-generator:logic:7b95a80b" tracker="pending-tracker" reason="New module: parse_storage_bytes(&str) -> Result<u64> (Kubernetes quantity suffixes Ki/Mi/Gi/Ti and bare bytes), a ResizeAction enum {Grow, NoOp, ShrinkUnsupported, Unparseable} plus decide(current: &str, desired: &str) -> ResizeAction (pure, unit-tested), a PvcResizeOutcome struct (pvc name, action, patched: bool, detail: String), and an impure async resize_instance(client: kube::Client, namespace: &str, name: &str, dry_run: bool) -> Result<Vec<PvcResizeOutcome>> that: fetches the named Lumen CR for spec.serving.raftStorage, lists PVCs labeled app.kubernetes.io/instance=<name> filtered to names starting raft-<name>-, runs decide() per PVC, and for Grow results looks up the bound PersistentVolumeClaim.spec.storage_class_name's StorageClass.spec.allow_volume_expansion — patching spec.resources.requests.storage via Patch::Merge only when allowed and !dry_run, otherwise recording a blocked/would-patch/no-op/shrink-skipped outcome without mutating the PVC."
//! `lumen k8s operator resize-storage` (#809) support module.
//!
//! StatefulSet `volumeClaimTemplates` are immutable after creation, so
//! bumping `spec.serving.raftStorage` on a live `Lumen` CR and letting the
//! operator reconcile does **not** resize anything — the rendered
//! StatefulSet's `apply` is a silent no-op for that field. This module is
//! the detect-and-patch tool for the gap: pure quantity comparison
//! ([`parse_storage_bytes`], [`decide`]) plus an impure driver
//! ([`resize_instance`]) that lists the instance's live `raft-<name>-<n>`
//! PVCs, compares each to the CR's declared size, and patches
//! `spec.resources.requests.storage` directly on PVCs whose bound
//! `StorageClass` allows expansion. PVC shrink is never attempted —
//! Kubernetes does not support it.

use anyhow::{Context, Result};
use k8s_openapi::api::core::v1::PersistentVolumeClaim;
use k8s_openapi::api::storage::v1::StorageClass;
use kube::api::{Api, ListParams, Patch, PatchParams};
use serde::Serialize;
use serde_json::json;

use super::crd::Lumen;

/// Outcome of comparing a PVC's current size against the CR's declared
/// `raftStorage`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResizeAction {
    /// `desired > current`; growing is the only direction Kubernetes PVCs
    /// support.
    Grow { current_bytes: u64, desired_bytes: u64 },
    /// `desired == current`; nothing to do.
    NoOp,
    /// `desired < current`; Kubernetes cannot shrink a bound PVC. Reported,
    /// never attempted.
    ShrinkUnsupported { current_bytes: u64, desired_bytes: u64 },
    /// One of the two quantities did not parse as a Kubernetes storage
    /// quantity.
    Unparseable { detail: String },
}

/// Parse a Kubernetes storage quantity (`"20Gi"`, `"500Mi"`, `"1Ti"`, a
/// decimal SI suffix like `"2G"`, or a bare byte count like `"1024"`) into a
/// byte count. Binary suffixes (`Ki/Mi/Gi/Ti/Pi/Ei`) are powers of 1024;
/// decimal suffixes (`k/M/G/T/P/E`) are powers of 1000, matching
/// `resource.Quantity` semantics.
pub fn parse_storage_bytes(qty: &str) -> Result<u64> {
    let s = qty.trim();
    if s.is_empty() {
        anyhow::bail!("empty storage quantity");
    }
    const SUFFIXES: &[(&str, f64)] = &[
        ("Ei", 1_152_921_504_606_846_976.0),
        ("Pi", 1_125_899_906_842_624.0),
        ("Ti", 1_099_511_627_776.0),
        ("Gi", 1_073_741_824.0),
        ("Mi", 1_048_576.0),
        ("Ki", 1_024.0),
        ("E", 1_000_000_000_000_000_000.0),
        ("P", 1_000_000_000_000_000.0),
        ("T", 1_000_000_000_000.0),
        ("G", 1_000_000_000.0),
        ("M", 1_000_000.0),
        ("k", 1_000.0),
    ];
    for (suffix, multiplier) in SUFFIXES {
        if let Some(num) = s.strip_suffix(suffix) {
            let value: f64 = num
                .trim()
                .parse()
                .with_context(|| format!("invalid numeric part in storage quantity '{qty}'"))?;
            if value < 0.0 {
                anyhow::bail!("negative storage quantity '{qty}'");
            }
            return Ok((value * multiplier).round() as u64);
        }
    }
    s.parse::<u64>()
        .with_context(|| format!("unrecognized storage quantity '{qty}'"))
}

/// Classify a `current` PVC size against a `desired` CR-declared size. Pure
/// and unit-tested; no live-cluster dependency.
pub fn decide(current: &str, desired: &str) -> ResizeAction {
    let current_bytes = match parse_storage_bytes(current) {
        Ok(v) => v,
        Err(e) => {
            return ResizeAction::Unparseable {
                detail: format!("current quantity '{current}': {e}"),
            }
        }
    };
    let desired_bytes = match parse_storage_bytes(desired) {
        Ok(v) => v,
        Err(e) => {
            return ResizeAction::Unparseable {
                detail: format!("desired quantity '{desired}': {e}"),
            }
        }
    };
    match desired_bytes.cmp(&current_bytes) {
        std::cmp::Ordering::Greater => ResizeAction::Grow {
            current_bytes,
            desired_bytes,
        },
        std::cmp::Ordering::Equal => ResizeAction::NoOp,
        std::cmp::Ordering::Less => ResizeAction::ShrinkUnsupported {
            current_bytes,
            desired_bytes,
        },
    }
}

/// Per-PVC result of a `resize_instance` run.
#[derive(Debug, Clone, Serialize)]
pub struct PvcResizeOutcome {
    pub pvc_name: String,
    pub current: String,
    pub desired: String,
    pub patched: bool,
    pub detail: String,
}

/// List the live `raft-<name>-<n>` PVCs for the named `Lumen` instance,
/// compare each to the CR's declared `spec.serving.raftStorage`, and patch
/// `spec.resources.requests.storage` on PVCs that are safe to grow (bound
/// `StorageClass.allowVolumeExpansion == true`, `dry_run == false`).
/// No other PVC field is touched, and the CR itself is never mutated — the
/// deployer edits `spec.serving.raftStorage` separately, the same source of
/// truth `render()` already reads.
pub async fn resize_instance(
    client: kube::Client,
    namespace: &str,
    name: &str,
    dry_run: bool,
) -> Result<Vec<PvcResizeOutcome>> {
    let lumens: Api<Lumen> = Api::namespaced(client.clone(), namespace);
    let cr = lumens
        .get(name)
        .await
        .with_context(|| format!("get Lumen '{namespace}/{name}'"))?;
    let desired = cr.spec.serving.raft_storage.clone();

    let pvc_api: Api<PersistentVolumeClaim> = Api::namespaced(client.clone(), namespace);
    let list_params = ListParams::default().labels(&format!("app.kubernetes.io/instance={name}"));
    let all = pvc_api
        .list(&list_params)
        .await
        .with_context(|| format!("list PVCs in '{namespace}' for instance '{name}'"))?;

    let prefix = format!("raft-{name}-");
    let mut outcomes = Vec::new();
    for pvc in all.items {
        let pvc_name = match pvc.metadata.name.as_deref() {
            Some(n) if n.starts_with(&prefix) => n.to_string(),
            _ => continue,
        };
        let current = pvc
            .spec
            .as_ref()
            .and_then(|s| s.resources.as_ref())
            .and_then(|r| r.requests.as_ref())
            .and_then(|m| m.get("storage"))
            .map(|q| q.0.clone())
            .unwrap_or_default();

        let outcome = match decide(&current, &desired) {
            ResizeAction::Unparseable { detail } => PvcResizeOutcome {
                pvc_name,
                current,
                desired: desired.clone(),
                patched: false,
                detail,
            },
            ResizeAction::NoOp => PvcResizeOutcome {
                pvc_name,
                current: current.clone(),
                desired: desired.clone(),
                patched: false,
                detail: "already at desired size".to_string(),
            },
            ResizeAction::ShrinkUnsupported { .. } => PvcResizeOutcome {
                pvc_name,
                current: current.clone(),
                desired: desired.clone(),
                patched: false,
                detail: "desired size is smaller than current; Kubernetes cannot shrink a \
                          bound PVC, recreate it instead"
                    .to_string(),
            },
            ResizeAction::Grow { .. } => {
                let storage_class_name =
                    pvc.spec.as_ref().and_then(|s| s.storage_class_name.clone());
                let expandable = match &storage_class_name {
                    Some(sc_name) => {
                        let sc_api: Api<StorageClass> = Api::all(client.clone());
                        match sc_api.get(sc_name).await {
                            Ok(sc) => sc.allow_volume_expansion.unwrap_or(false),
                            Err(_) => false,
                        }
                    }
                    None => false,
                };
                if !expandable {
                    PvcResizeOutcome {
                        pvc_name,
                        current: current.clone(),
                        desired: desired.clone(),
                        patched: false,
                        detail: format!(
                            "StorageClass '{}' does not allow volume expansion; recreate the \
                             PVC/StatefulSet manually",
                            storage_class_name.as_deref().unwrap_or("<none>")
                        ),
                    }
                } else if dry_run {
                    PvcResizeOutcome {
                        pvc_name,
                        current: current.clone(),
                        desired: desired.clone(),
                        patched: false,
                        detail: "dry run: would patch spec.resources.requests.storage"
                            .to_string(),
                    }
                } else {
                    let patch = json!({
                        "spec": { "resources": { "requests": { "storage": desired } } }
                    });
                    pvc_api
                        .patch(&pvc_name, &PatchParams::default(), &Patch::Merge(&patch))
                        .await
                        .with_context(|| {
                            format!("patch PVC '{pvc_name}' storage to '{desired}'")
                        })?;
                    PvcResizeOutcome {
                        pvc_name,
                        current: current.clone(),
                        desired: desired.clone(),
                        patched: true,
                        detail: "patched spec.resources.requests.storage".to_string(),
                    }
                }
            }
        };
        outcomes.push(outcome);
    }
    Ok(outcomes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_binary_and_decimal_suffixes_and_bare_bytes() {
        assert_eq!(parse_storage_bytes("20Gi").unwrap(), 20 * 1_073_741_824);
        assert_eq!(parse_storage_bytes("500Mi").unwrap(), 500 * 1_048_576);
        assert_eq!(parse_storage_bytes("1Ti").unwrap(), 1_099_511_627_776);
        assert_eq!(parse_storage_bytes("2G").unwrap(), 2_000_000_000);
        assert_eq!(parse_storage_bytes("1024").unwrap(), 1024);
    }

    #[test]
    fn rejects_unparseable_quantities() {
        assert!(parse_storage_bytes("").is_err());
        assert!(parse_storage_bytes("banana").is_err());
        assert!(parse_storage_bytes("-5Gi").is_err());
    }

    #[test]
    fn decide_detects_grow() {
        match decide("20Gi", "30Gi") {
            ResizeAction::Grow {
                current_bytes,
                desired_bytes,
            } => {
                assert_eq!(current_bytes, 20 * 1_073_741_824);
                assert_eq!(desired_bytes, 30 * 1_073_741_824);
            }
            other => panic!("expected Grow, got {other:?}"),
        }
    }

    #[test]
    fn decide_detects_no_op() {
        assert_eq!(decide("20Gi", "20Gi"), ResizeAction::NoOp);
    }

    #[test]
    fn decide_detects_shrink_unsupported() {
        match decide("20Gi", "10Gi") {
            ResizeAction::ShrinkUnsupported {
                current_bytes,
                desired_bytes,
            } => {
                assert_eq!(current_bytes, 20 * 1_073_741_824);
                assert_eq!(desired_bytes, 10 * 1_073_741_824);
            }
            other => panic!("expected ShrinkUnsupported, got {other:?}"),
        }
    }

    #[test]
    fn decide_detects_unparseable() {
        assert!(matches!(
            decide("not-a-size", "20Gi"),
            ResizeAction::Unparseable { .. }
        ));
        assert!(matches!(
            decide("20Gi", "not-a-size"),
            ResizeAction::Unparseable { .. }
        ));
    }
}
// HANDWRITE-END
