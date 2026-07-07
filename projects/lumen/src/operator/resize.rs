// HANDWRITE-BEGIN gap="missing-generator:logic:7b95a80b" tracker="pending-tracker" reason="Thin Lumen-specific adapter over the shared `operator::resize` module (#970): fetches the named Lumen CR for spec.serving.raftStorage and delegates PVC listing/quantity-compare/patch logic (parse_storage_bytes, ResizeAction/decide, PvcResizeOutcome, resize_instance) to libs/operator, supplying the `app.kubernetes.io/instance=<name>` label selector and the `raft-<name>-` PVC name filter. No generator primitive exists yet for this shape, so it stays HANDWRITE per CLAUDE.md until one covers it."
//! `lumen k8s operator resize-storage` (#809) support module.
//!
//! StatefulSet `volumeClaimTemplates` are immutable after creation, so
//! bumping `spec.serving.raftStorage` on a live `Lumen` CR and letting the
//! operator reconcile does **not** resize anything — the rendered
//! StatefulSet's `apply` is a silent no-op for that field. The generic
//! detect-and-patch tool (quantity parsing, grow/no-op/shrink decision, PVC
//! listing + `StorageClass`-gated patch) lives in the shared
//! [`operator::resize`] module (#970); this module is the thin Lumen-specific
//! adapter that fetches the CR's declared `spec.serving.raftStorage` and
//! scopes the PVC listing to the instance's `raft-<name>-<n>` volumes.

use anyhow::{Context, Result};
use kube::api::Api;

use super::crd::Lumen;

pub use operator::resize::{decide, parse_storage_bytes, PvcResizeOutcome, ResizeAction};

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

    let label_selector = format!("app.kubernetes.io/instance={name}");
    let prefix = format!("raft-{name}-");

    operator::resize::resize_instance(
        client,
        namespace,
        &label_selector,
        |pvc_name| pvc_name.starts_with(&prefix),
        |_pvc_name| desired.clone(),
        dry_run,
    )
    .await
}
// HANDWRITE-END
