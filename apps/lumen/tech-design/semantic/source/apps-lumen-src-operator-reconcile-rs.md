---
id: projects-lumen-src-operator-reconcile-rs
capability_refs:
  - id: "long-running-stability"
    role: primary
    gap: "kustomize-base-overlays-hpa"
    claim: "kustomize-base-overlays-hpa"
    coverage: partial
    rationale: "This source unit is captured as a per-file rust-source-unit during lumen td_ast standardization."
  - id: "kubernetes-native-deployment"
    role: primary
    gap: "operator-owned-storage-topology-and-reshard-status"
    claim: "operator-owned-storage-topology-and-reshard-status"
    coverage: full
    rationale: "The reconcile loop publishes status from the operator-owned StatefulSet storage topology and reshard policy."
  - id: "dynamic-shard-topology"
    role: primary
    gap: "stale-single-member-hpa-handoff-deletion"
    claim: "stale-single-member-hpa-handoff-deletion"
    coverage: full
    rationale: "The reconcile loop detects and deletes stale single-member or legacy HPA objects during topology handoff."
  - id: "kubernetes-native-deployment"
    role: primary
    gap: "topology-transition-hpa-handoff-deletion"
    claim: "topology-transition-hpa-handoff-deletion"
    coverage: full
    rationale: "The Kubernetes reconcile loop owns stale-HPA deletion during storage-topology transitions."
fill_sections: [overview, source, changes]
---

# Standardized apps/lumen/src/operator/reconcile.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/lumen/src/operator/reconcile.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `PEER_IDENTITY_CONTEXT_KEY` | apps/lumen/src/operator/reconcile.rs | constant | pub | 785 |  |
| `parse_metric` | apps/lumen/src/operator/reconcile.rs | function | pub | 153 | parse_metric(body: &str, metric: &str) -> Option<u64> |
| `run` | apps/lumen/src/operator/reconcile.rs | function | pub | 1285 | run() -> anyhow::Result<()> |
## Source
<!-- type: rust-source-unit lang: rust -->

```rust
// SPEC-MANAGED: apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-reconcile-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! lumen's operator wiring onto the shared `libs/service-k8s` controller.
//!
//! The reconcile loop + leader-election lease now live in `libs/service-k8s`
//! (`service_k8s::run` drives the watch + leader-gated apply over h2c-free kube;
//! `service_k8s::lease` is the elector). lumen supplies only its `ManagedService`
//! impl — what to render, which workloads to poll for readiness, and the
//! `Lumen` status subresource to write.
//!
//! Live per-shard storage-usage measurement (#1319 R1): `ManagedService::
//! status_patch` is synchronous and does no I/O by contract (shared with
//! keep/relay/loom via `libs/service-k8s`), so it cannot itself poll pod
//! `/metrics` endpoints. Instead `run()` spawns a lumen-local background
//! loop (`spawn_shard_usage_loop`) that periodically scrapes every storage
//! pod's `lumen_storage_bytes` gauge over its headless-Service DNS name and
//! writes the per-shard max into an in-process cache; `status_patch` reads
//! that cache synchronously (best-effort — an empty/missing cache falls back
//! to the policy-only [`crate::operator::crd::LumenSpec::reshard_status`]).
//! This keeps the shared `libs/service-k8s` trait untouched.
//!
//! This loop only *reports* a crossed `prepareAtPercent` / `urgentAtPercent`
//! threshold in `status.reshard`; a second, independently leader-gated
//! background loop spawned alongside it — [`crate::operator::reshard_driver::
//! spawn_reshard_driver_loop`] (#1319 R2, #1381) — is what actually drives
//! `workflow.phase` and moves data once a threshold is crossed.
//!
//! ## Post-cutover usage freshness (#1386)
//!
//! Each [`ShardUsageSnapshot`] the loop below writes into the cache is
//! tagged with `spec.shardMap.version` as read off the very same CR the
//! scrape was addressed against — the freshness generation
//! [`crate::operator::crd::LumenSpec::reshard_status_with_usage`] compares
//! against the CR's *current* `shard_map.version` before ever reporting a
//! crossed threshold. Without this, a split's `Complete` cutover (which
//! bumps `shard_map.version`) races this loop's own
//! [`SHARD_USAGE_POLL_INTERVAL`] cadence: the cache can still hold a
//! pre-cutover, pre-eviction reading for up to that whole interval, and
//! [`crate::operator::reshard_driver::should_start_split`] would otherwise
//! re-fire off that stale number the very next driver tick — the live bug
//! #1384's kind proof caught (a second split starting 20s after the first
//! one's `Complete`, purely off a reading taken before the eviction that
//! same cutover had just performed). Neither loop needs to synchronize
//! with the other directly: the generation tag alone is enough, and it
//! survives an operator failover the same way every other reshard
//! checkpoint does — it rides on the CR itself (`status.reshard.
//! usageMeasuredAtMapVersion`, freshly recomputed by whichever replica
//! next runs `status_patch`), never in this loop's or the driver's
//! in-process state.
//!
//! ## HPA topology-transition handoff (#1385)
//!
//! [`render::render`] no longer emits a HorizontalPodAutoscaler for any data
//! topology ([`render::wants_hpa`] is always false), but `libs/service-k8s`'s
//! shared reconcile contract (`libs/service-k8s::
//! service`) deliberately does not prune children across a render-shape
//! change — that handoff is left to the service. A third independently
//! leader-gated background loop, [`spawn_hpa_handoff_loop`], is lumen's side
//! of that handoff: every tick it lists every `Lumen` CR and, for any whose
//! current shape never wants an HPA, deletes any previously-rendered one
//! if it is still there — scoped and idempotent (R2: only an object whose
//! live name *and* labels match what [`render::hpa_labels`] would have
//! stamped; a missing HPA, or one that doesn't look lumen-rendered, is a
//! no-op, not an error). Without this migration cleanup, a stale HPA can keep
//! mutating the data StatefulSet outside the shared membership-aware capacity
//! contract.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use kube::api::{Api, ApiResource, DeleteParams, DynamicObject, ListParams, Patch, PatchParams};
use kube::{Client, ResourceExt};
use serde_json::json;
use service_k8s::{ConditionFact, ConditionStatus, ManagedService, ReadinessTarget, ReadyFacts};

use crate::operator::crd::{Lumen, LumenReshardStatus, ReshardPhase};
use crate::operator::render;

/// The client-facing port lumen's serving Service/StatefulSet expose
/// (`render::CLIENT_PORT`, private to that module). Duplicated here rather
/// than making that constant `pub` — this is the only other file that needs
/// it, and a `pub` const would need its own mirror symbol-table row.
const CLIENT_PORT: u16 = 7373;

/// Poll interval for the live per-shard storage-usage measurement loop
/// (#1319 R1).
const SHARD_USAGE_POLL_INTERVAL: Duration = Duration::from_secs(30);

/// Poll interval for [`spawn_hpa_handoff_loop`] (#1385). Faster than
/// [`SHARD_USAGE_POLL_INTERVAL`] since a lingering stale HPA actively starves
/// the reshard driver's `PrepareSplit` gate — the sooner it's pruned, the
/// sooner that gate can converge.
const HPA_HANDOFF_POLL_INTERVAL: Duration = Duration::from_secs(15);

/// Leader-election Lease name for [`spawn_hpa_handoff_loop`] (#1385) —
/// distinct from both `libs/service-k8s`'s own `S::MANAGER`-named apply-loop
/// Lease and `reshard_driver::DRIVER_LEASE_NAME`, so none of the three
/// independently leader-gated loops contend on one Lease object (mirrors the
/// same duplicated `identity`/`lease_namespace` resolution
/// `reshard_driver::spawn_reshard_driver_loop` already uses for the same
/// reason).
const HPA_HANDOFF_LEASE_NAME: &str = "lumen-hpa-handoff";

/// Poll interval for [`spawn_auth_delegator_sweep_loop`] (#2876). A leftover
/// binding is a standing grant of delegated authentication review to a
/// ServiceAccount whose instance no longer exists, so it is swept on the same
/// brisk cadence as the HPA handoff rather than the slower usage scrape.
const AUTH_DELEGATOR_SWEEP_POLL_INTERVAL: Duration = Duration::from_secs(15);

/// Leader-election Lease name for [`spawn_auth_delegator_sweep_loop`] (#2876),
/// distinct from every other independently leader-gated loop's Lease for the
/// same reason [`HPA_HANDOFF_LEASE_NAME`] is.
const AUTH_DELEGATOR_SWEEP_LEASE_NAME: &str = "lumen-auth-delegator-sweep";

/// One shard-usage measurement (#1386 R1): the raw per-shard bytes plus the
/// `spec.shardMap.version` that was live on the CR at scrape time — the
/// freshness generation [`crate::operator::crd::LumenSpec::
/// reshard_status_with_usage`] compares against the CR's *current*
/// `spec.shardMap.version` to tell a post-cutover measurement apart from a
/// pre-cutover one this cache is still holding right after a split
/// completes.
#[derive(Clone, Debug)]
struct ShardUsageSnapshot {
    measured_at_map_version: u64,
    usage: BTreeMap<u32, u64>,
}

/// `"<namespace>/<name>" -> ShardUsageSnapshot`, refreshed by
/// [`spawn_shard_usage_loop`] and read by [`status_patch`].
type ShardUsageCache = Mutex<BTreeMap<String, ShardUsageSnapshot>>;

fn shard_usage_cache() -> &'static ShardUsageCache {
    static CACHE: OnceLock<ShardUsageCache> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn cache_key(lumen: &Lumen) -> String {
    format!(
        "{}/{}",
        lumen.namespace().unwrap_or_else(|| "default".to_string()),
        lumen.name_any()
    )
}

/// Parse one gauge's value out of Prometheus text exposition (see
/// `crate::metrics::Registry::render`, e.g. `"lumen_storage_bytes 2048\n"`).
/// Ignores comment (`#`) and blank lines; returns `None` if `metric` is not
/// present or its value does not parse. `pub(crate)` (#1467 R5) so
/// `reshard_driver::KubeClusterControl::serving_pods_report_map_version` can
/// reuse it to parse `lumen_shard_map_version` off the same `/metrics`
/// exposition this module already scrapes for `lumen_storage_bytes`.
pub(crate) fn parse_metric(body: &str, metric: &str) -> Option<u64> {
    body.lines().find_map(|line| {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return None;
        }
        let (name, value) = line.split_once(' ')?;
        if name != metric {
            return None;
        }
        value.trim().parse::<f64>().ok().map(|bytes| bytes as u64)
    })
}

/// Fetch one pod's `/metrics` and read its `lumen_storage_bytes` gauge.
/// `None` on any network error, non-2xx status, or missing/unparseable
/// metric — an unreachable pod (e.g. mid-rollout) contributes nothing rather
/// than failing the whole measurement tick.
async fn pod_storage_bytes(http: &reqwest::Client, url: &str) -> Option<u64> {
    let resp = http.get(url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body = resp.text().await.ok()?;
    parse_metric(&body, "lumen_storage_bytes")
}

/// Every storage pod's `(shard_index, /metrics URL)`, addressed by its
/// StatefulSet headless-Service DNS name
/// (`<name>-<ordinal>.<name>-headless.<ns>.svc.cluster.local:<port>`).
/// Ordinal-to-shard mapping matches `libs/raft-runtime`'s pod placement:
/// `shard_index = ordinal % shard_count`, `replica_index = ordinal /
/// shard_count`. Pod count and shard-count clamping follow
/// [`crate::operator::crd::LumenSpec::storage_pod_count`] exactly, including
/// its single-shard-single-replica edge case (#1317).
fn pod_metrics_urls(lumen: &Lumen) -> Vec<(u32, String)> {
    let name = lumen.name_any();
    let ns = lumen.namespace().unwrap_or_else(|| "default".to_string());
    let headless = format!("{name}-headless");
    let shard_count = lumen.spec.shard_count.max(1);
    let total = lumen.spec.storage_pod_count().max(0) as u32;
    (0..total)
        .map(|ordinal| {
            let shard_index = ordinal % shard_count;
            let url = format!(
                "http://{name}-{ordinal}.{headless}.{ns}.svc.cluster.local:{CLIENT_PORT}/metrics"
            );
            (shard_index, url)
        })
        .collect()
}

/// Scrape every `(shard_index, url)` pair and reduce to the per-shard
/// maximum observed byte count (a shard's busiest replica, not the sum —
/// replicas of the same shard hold the same raft-replicated data).
/// Unreachable pods are skipped, not treated as zero usage.
async fn aggregate_shard_usage(
    http: &reqwest::Client,
    pod_urls: &[(u32, String)],
) -> BTreeMap<u32, u64> {
    let mut usage: BTreeMap<u32, u64> = BTreeMap::new();
    for (shard_index, url) in pod_urls {
        let Some(bytes) = pod_storage_bytes(http, url).await else {
            continue;
        };
        usage
            .entry(*shard_index)
            .and_modify(|max| *max = (*max).max(bytes))
            .or_insert(bytes);
    }
    usage
}

/// One measurement tick for `lumen`: [`pod_metrics_urls`] +
/// [`aggregate_shard_usage`].
async fn measure_shard_usage(http: &reqwest::Client, lumen: &Lumen) -> BTreeMap<u32, u64> {
    let urls = pod_metrics_urls(lumen);
    aggregate_shard_usage(http, &urls).await
}

/// Background loop (#1319 R1): every [`SHARD_USAGE_POLL_INTERVAL`], list
/// every `Lumen` CR cluster-wide, measure its live per-shard storage usage,
/// and refresh [`shard_usage_cache`]. Runs on every replica (not just the
/// leader) since it only populates a local read cache that `status_patch`
/// consults best-effort; the leader-gated `libs/service-k8s` apply loop is
/// still the only writer of the CR's status subresource.
fn spawn_shard_usage_loop(client: Client) {
    tokio::spawn(async move {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        let api: kube::Api<Lumen> = kube::Api::all(client);
        loop {
            match api.list(&Default::default()).await {
                Ok(list) => {
                    for lumen in list.items {
                        let usage = measure_shard_usage(&http, &lumen).await;
                        if usage.is_empty() {
                            continue;
                        }
                        let key = cache_key(&lumen);
                        // #1386 R1: tag this measurement with the map
                        // version live on the *same* CR read the scrape
                        // itself was addressed against, so a status
                        // computed later can tell whether it predates the
                        // next cutover.
                        let snapshot = ShardUsageSnapshot {
                            measured_at_map_version: lumen.spec.shard_map.version,
                            usage,
                        };
                        let mut cache = shard_usage_cache()
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        cache.insert(key, snapshot);
                    }
                }
                Err(err) => {
                    tracing::warn!(error = %err, "shard usage measurement: list Lumen failed");
                }
            }
            tokio::time::sleep(SHARD_USAGE_POLL_INTERVAL).await;
        }
    });
}

/// The `ApiResource` for a live HorizontalPodAutoscaler, matching what
/// `libs/service-k8s::render::horizontal_pod_autoscaler` renders
/// (`autoscaling/v2`) and what `libs/service-k8s::controller`'s generic apply
/// loop server-side-applies it as.
fn hpa_api_resource() -> ApiResource {
    ApiResource {
        group: "autoscaling".to_string(),
        version: "v2".to_string(),
        api_version: "autoscaling/v2".to_string(),
        kind: "HorizontalPodAutoscaler".to_string(),
        plural: "horizontalpodautoscalers".to_string(),
    }
}

/// Seam for [`prune_stale_hpa`]'s only two k8s side effects (#1385) —
/// abstracted the same way `reshard_driver::ClusterControl` abstracts its
/// cluster calls, so the handoff decision is testable without a live k8s API
/// server. [`KubeHpaControl`] is the production implementation; tests supply
/// an in-memory fake.
#[async_trait::async_trait]
trait HpaControl: Send + Sync {
    /// The live HPA's labels at `(namespace, name)`, or `None` if it does not
    /// exist. A missing object is the idempotent no-op case (R2), never an
    /// error.
    async fn hpa_labels(
        &self,
        namespace: &str,
        name: &str,
    ) -> anyhow::Result<Option<BTreeMap<String, String>>>;

    /// Delete the HPA at `(namespace, name)`. Only called after
    /// [`Self::hpa_labels`] has confirmed lumen rendered it. Idempotent: a
    /// concurrent deletion between the two calls (404 on delete) is treated
    /// as success, not an error.
    async fn delete_hpa(&self, namespace: &str, name: &str) -> anyhow::Result<()>;
}

/// Production [`HpaControl`]: real `kube::Client` calls.
struct KubeHpaControl {
    client: Client,
}

/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-reconcile-rs.md#source
#[async_trait::async_trait]
impl HpaControl for KubeHpaControl {
    async fn hpa_labels(
        &self,
        namespace: &str,
        name: &str,
    ) -> anyhow::Result<Option<BTreeMap<String, String>>> {
        let api: Api<DynamicObject> =
            Api::namespaced_with(self.client.clone(), namespace, &hpa_api_resource());
        let obj = api.get_opt(name).await?;
        Ok(obj.and_then(|o| o.metadata.labels))
    }

    async fn delete_hpa(&self, namespace: &str, name: &str) -> anyhow::Result<()> {
        let api: Api<DynamicObject> =
            Api::namespaced_with(self.client.clone(), namespace, &hpa_api_resource());
        match api.delete(name, &DeleteParams::default()).await {
            Ok(_) => Ok(()),
            // Already gone (raced with another deletion, or a watch-triggered
            // reconcile fired again before the cache caught up) — idempotent
            // no-op, matching R2.
            Err(kube::Error::Api(e)) if e.code == 404 => Ok(()),
            Err(err) => Err(err.into()),
        }
    }
}

/// One CR's HPA-handoff check (#1385): if `lumen`'s currently-rendered shape
/// no longer wants an HPA ([`render::wants_hpa`], R1), delete the
/// previously-rendered one if it is still there and lumen actually rendered
/// it (R2 — live name *and* labels match [`render::hpa_labels`]; a missing
/// HPA, or a live one that doesn't look lumen-rendered, is left alone). Logs
/// the handoff (why the HPA vanished) so an operator reading logs
/// understands it (R3/AC3). Never panics; a failed cluster call is logged
/// and retried next tick, same as the other background loops in this file.
async fn prune_stale_hpa(control: &dyn HpaControl, lumen: &Lumen) {
    if render::wants_hpa(lumen) {
        // Current shape still wants one (or keeps wanting one) — nothing to
        // hand off.
        return;
    }
    let namespace = lumen.namespace().unwrap_or_else(|| "default".to_string());
    let name = lumen.name_any();
    let live_labels = match control.hpa_labels(&namespace, &name).await {
        Ok(labels) => labels,
        Err(err) => {
            tracing::warn!(
                %namespace, %name, error = %err,
                "HPA handoff: failed to read live HPA, will retry next tick"
            );
            return;
        }
    };
    let Some(live_labels) = live_labels else {
        // R2: no HPA to hand off is a no-op, not an error.
        return;
    };
    let expected_labels = render::hpa_labels(lumen);
    if live_labels != expected_labels {
        // R2 scope guard: an object happens to share this CR's name (and
        // namespace) but its labels don't match what lumen would have
        // stamped — never touch it, it wasn't rendered by us.
        tracing::warn!(
            %namespace, %name,
            "HPA handoff: found an HPA at this CR's name whose labels don't \
             match lumen's render — leaving it alone (not operator-rendered)"
        );
        return;
    }
    match control.delete_hpa(&namespace, &name).await {
        Ok(()) => {
            tracing::info!(
                %namespace, %name,
                "HPA handoff: deleted a legacy operator-rendered data-plane \
                 HPA — direct StatefulSet HPA cannot preserve whole per-shard \
                 layers or perform Raft membership transitions"
            );
        }
        Err(err) => {
            tracing::warn!(
                %namespace, %name, error = %err,
                "HPA handoff: failed to delete stale HPA, will retry next tick"
            );
        }
    }
}

/// Background loop (#1385): every [`HPA_HANDOFF_POLL_INTERVAL`], while
/// holding the [`HPA_HANDOFF_LEASE_NAME`] Lease, list every `Lumen` CR
/// cluster-wide and run [`prune_stale_hpa`] against each. Independently
/// leader-gated (like [`crate::operator::reshard_driver::
/// spawn_reshard_driver_loop`]) since deletion is a cluster write, unlike
/// [`spawn_shard_usage_loop`]'s read-only cache population.
fn spawn_hpa_handoff_loop(client: Client) {
    // Mirrors `libs/service-k8s::controller`'s own `identity`/`lease_namespace`
    // helpers (private to that crate, so duplicated here, same as
    // `reshard_driver::spawn_reshard_driver_loop` already does) so every
    // independently-leader-gated loop resolves the same pod identity and
    // Lease namespace from the same env vars.
    let identity = std::env::var("POD_NAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| HPA_HANDOFF_LEASE_NAME.to_string());
    let namespace =
        std::env::var("POD_NAMESPACE").unwrap_or_else(|_| "lumen-operator-system".to_string());
    let election = crate::operator::lease::Election::new(identity);
    crate::operator::lease::spawn(
        client.clone(),
        namespace,
        HPA_HANDOFF_LEASE_NAME.to_string(),
        election.clone(),
    );
    let control = KubeHpaControl {
        client: client.clone(),
    };
    tokio::spawn(async move {
        let api: kube::Api<Lumen> = kube::Api::all(client);
        loop {
            if election
                .is_leader
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                match api.list(&Default::default()).await {
                    Ok(list) => {
                        for lumen in list.items {
                            prune_stale_hpa(&control, &lumen).await;
                        }
                    }
                    Err(err) => {
                        tracing::warn!(error = %err, "HPA handoff: list Lumen failed");
                    }
                }
            }
            tokio::time::sleep(HPA_HANDOFF_POLL_INTERVAL).await;
        }
    });
}

/// The `ApiResource` for a cluster-scoped ClusterRoleBinding (#2876).
fn cluster_role_binding_api_resource() -> ApiResource {
    ApiResource {
        group: "rbac.authorization.k8s.io".to_string(),
        version: "v1".to_string(),
        api_version: "rbac.authorization.k8s.io/v1".to_string(),
        kind: "ClusterRoleBinding".to_string(),
        plural: "clusterrolebindings".to_string(),
    }
}

/// Seam for the auth-delegator binding's three cluster side effects (#2876),
/// abstracted the same way [`HpaControl`] abstracts the HPA handoff's, so both
/// the apply decision and the sweep decision are testable without an API
/// server. [`KubeAuthDelegatorControl`] is the production implementation.
#[async_trait::async_trait]
trait AuthDelegatorControl: Send + Sync {
    /// Server-side-apply the binding. Cluster-scoped: `Api::all_with`, not
    /// `Api::namespaced_with`, which is why this cannot ride the shared
    /// controller's child-apply loop.
    async fn apply_binding(&self, binding: &serde_json::Value) -> anyhow::Result<()>;

    /// Every ClusterRoleBinding lumen's operator manages, as
    /// `(name, labels)`. Selected server-side on the managed-by/component
    /// labels so the sweep never even sees a binding belonging to something
    /// else.
    async fn managed_bindings(&self) -> anyhow::Result<Vec<(String, BTreeMap<String, String>)>>;

    /// Delete one binding by name. A 404 is success: the sweep runs every tick
    /// and races itself across operator replicas.
    async fn delete_binding(&self, name: &str) -> anyhow::Result<()>;
}

/// Production [`AuthDelegatorControl`]: real `kube::Client` calls.
struct KubeAuthDelegatorControl {
    client: Client,
}

/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-reconcile-rs.md#source
#[async_trait::async_trait]
impl AuthDelegatorControl for KubeAuthDelegatorControl {
    async fn apply_binding(&self, binding: &serde_json::Value) -> anyhow::Result<()> {
        let name = binding["metadata"]["name"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("rendered binding has no metadata.name"))?
            .to_string();
        let api: Api<DynamicObject> =
            Api::all_with(self.client.clone(), &cluster_role_binding_api_resource());
        let obj: DynamicObject = serde_json::from_value(binding.clone())?;
        api.patch(
            &name,
            &PatchParams::apply(<Lumen as ManagedService>::MANAGER).force(),
            &Patch::Apply(&obj),
        )
        .await?;
        Ok(())
    }

    async fn managed_bindings(&self) -> anyhow::Result<Vec<(String, BTreeMap<String, String>)>> {
        let api: Api<DynamicObject> =
            Api::all_with(self.client.clone(), &cluster_role_binding_api_resource());
        let params = ListParams::default().labels(&format!(
            "app.kubernetes.io/managed-by={},app.kubernetes.io/component=auth-delegation",
            <Lumen as ManagedService>::MANAGER
        ));
        Ok(api
            .list(&params)
            .await?
            .items
            .into_iter()
            .filter_map(|obj| {
                let name = obj.metadata.name.clone()?;
                Some((name, obj.metadata.labels.unwrap_or_default()))
            })
            .collect())
    }

    async fn delete_binding(&self, name: &str) -> anyhow::Result<()> {
        let api: Api<DynamicObject> =
            Api::all_with(self.client.clone(), &cluster_role_binding_api_resource());
        match api.delete(name, &DeleteParams::default()).await {
            Ok(_) => Ok(()),
            Err(kube::Error::Api(e)) if e.code == 404 => Ok(()),
            Err(err) => Err(err.into()),
        }
    }
}

/// Apply this instance's auth-delegator binding, returning the failure message
/// to publish if the write was refused (#2876 R5, AC4).
///
/// The error is returned rather than propagated because of where it has to
/// land. Failing the reconcile outright aborts before the status patch, so the
/// CR would go on reporting whatever it last said while its serving pods could
/// not authenticate a single request — a silent failure with a healthy-looking
/// status. Carrying the message forward instead makes the CR say the true
/// thing: not Ready, and *why*, naming the ClusterRoleBinding operation that
/// was refused. The 30-second requeue retries it either way, so nothing is
/// lost by not erroring.
async fn apply_auth_delegator_binding(control: &dyn AuthDelegatorControl, lumen: &Lumen) -> Option<String> {
    let binding = render::auth_delegator_binding(lumen);
    let name = render::auth_delegator_binding_name(lumen);
    match control.apply_binding(&binding).await {
        Ok(()) => None,
        Err(err) => {
            tracing::warn!(
                binding = %name, error = %err,
                "auth delegation: apply of the serving ServiceAccount's \
                 system:auth-delegator ClusterRoleBinding failed"
            );
            Some(format!(
                "apply ClusterRoleBinding {name} (system:auth-delegator): {err}"
            ))
        }
    }
}

/// Delete auth-delegator bindings whose owning `Lumen` is gone (#2876 R3/R5,
/// AC3).
///
/// A cluster-scoped object cannot be owned by a namespaced CR, so there is no
/// cascading delete to rely on and — since a deleted CR is never reconciled
/// again — no reconcile that could clean up after itself either. This sweep is
/// the replacement: it runs cluster-wide against the live `Lumen` list, so the
/// object that authorizes an instance disappears with the instance rather than
/// when something remembers to look.
///
/// `live` is every `Lumen` in the cluster. A binding survives only if some
/// live instance would render *exactly* it — same name and same full label
/// set. Matching on the full label set, rather than on the name alone, is the
/// same guard [`prune_stale_hpa`] uses and for the same reason: a name is not
/// proof of authorship, and this one deletes an RBAC object.
async fn sweep_stale_auth_delegator_bindings(control: &dyn AuthDelegatorControl, live: &[Lumen]) {
    let bindings = match control.managed_bindings().await {
        Ok(bindings) => bindings,
        Err(err) => {
            tracing::warn!(
                error = %err,
                "auth delegation sweep: listing managed ClusterRoleBindings failed, will retry next tick"
            );
            return;
        }
    };
    let wanted: BTreeMap<String, BTreeMap<String, String>> = live
        .iter()
        .map(|lumen| {
            (
                render::auth_delegator_binding_name(lumen),
                render::auth_delegator_labels(lumen),
            )
        })
        .collect();
    for (name, labels) in bindings {
        match wanted.get(&name) {
            // Still wanted, and it looks like ours — the reconcile path keeps
            // its subject current, so there is nothing to do here.
            Some(expected) if *expected == labels => continue,
            Some(_) => {
                // A live instance claims this name but the labels are not the
                // ones lumen stamps. Deleting would be acting on an object we
                // cannot show we created; the apply path will correct the
                // fields it owns.
                tracing::warn!(
                    binding = %name,
                    "auth delegation sweep: a binding at a live instance's name carries labels \
                     lumen does not render — leaving it alone (not operator-rendered)"
                );
                continue;
            }
            None => {}
        }
        match control.delete_binding(&name).await {
            Ok(()) => tracing::info!(
                binding = %name,
                "auth delegation sweep: deleted a system:auth-delegator ClusterRoleBinding whose \
                 Lumen instance no longer exists"
            ),
            Err(err) => tracing::warn!(
                binding = %name, error = %err,
                "auth delegation sweep: delete failed, will retry next tick"
            ),
        }
    }
}

/// Background loop (#2876): every [`AUTH_DELEGATOR_SWEEP_POLL_INTERVAL`],
/// while holding the [`AUTH_DELEGATOR_SWEEP_LEASE_NAME`] Lease, list every
/// `Lumen` cluster-wide and hand the list to
/// [`sweep_stale_auth_delegator_bindings`]. Independently leader-gated for the
/// same reason [`spawn_hpa_handoff_loop`] is: it performs cluster writes.
fn spawn_auth_delegator_sweep_loop(client: Client) {
    let identity = std::env::var("POD_NAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| AUTH_DELEGATOR_SWEEP_LEASE_NAME.to_string());
    let namespace =
        std::env::var("POD_NAMESPACE").unwrap_or_else(|_| "lumen-operator-system".to_string());
    let election = crate::operator::lease::Election::new(identity);
    crate::operator::lease::spawn(
        client.clone(),
        namespace,
        AUTH_DELEGATOR_SWEEP_LEASE_NAME.to_string(),
        election.clone(),
    );
    let control = KubeAuthDelegatorControl {
        client: client.clone(),
    };
    tokio::spawn(async move {
        let api: kube::Api<Lumen> = kube::Api::all(client);
        loop {
            if election
                .is_leader
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                match api.list(&Default::default()).await {
                    Ok(list) => {
                        sweep_stale_auth_delegator_bindings(&control, &list.items).await;
                    }
                    Err(err) => {
                        // Deliberately no sweep on a failed list: an empty
                        // `live` set would read as "no instance wants any
                        // binding" and delete every one of them.
                        tracing::warn!(error = %err, "auth delegation sweep: list Lumen failed");
                    }
                }
            }
            tokio::time::sleep(AUTH_DELEGATOR_SWEEP_POLL_INTERVAL).await;
        }
    });
}

/// Why this replicated instance has no usable Raft peer identity, if it has
/// none (#2890 R4).
///
/// `Some(_)` is a fail-closed verdict: a `replicasPerShard > 1` instance whose
/// `spec.peerTlsSecret` is unset.
///
/// Single-replica instances run no consensus link and are always `None`: there
/// is no peer to authenticate.
fn check_peer_identity(lumen: &Lumen) -> Option<String> {
    if !lumen.spec.peer_identity_required() {
        return None;
    }
    if lumen.spec.peer_tls_secret.is_none() {
        return Some(format!(
            "replicasPerShard={} requires spec.peerTlsSecret naming a Secret with {}; \
             replicated Raft traffic has no plaintext fallback",
            lumen.spec.replicas_per_shard,
            render::PEER_TLS_KEYS.join(", ")
        ));
    }
    None
}


/// The reconcile-context key carrying [`apply_auth_delegator_binding`]'s
/// verdict from the plan hook to the condition projection (#2876).
const AUTH_DELEGATION_CONTEXT_KEY: &str = "authDelegationError";

/// The same channel for [`check_peer_identity`]'s verdict (#2890). Same reason:
/// the check is a Secret read, and `observe` is I/O-free by contract.
///
/// Public so the render-gate tests project a condition through the key the
/// reconcile loop actually writes, rather than a string literal that would keep
/// passing after a rename.
pub const PEER_IDENTITY_CONTEXT_KEY: &str = "peerIdentityError";

/// Read that verdict back out. Absent key = peer identity is satisfied (or not
/// required).
fn peer_identity_error(context: &serde_json::Value) -> Option<String> {
    context
        .get(PEER_IDENTITY_CONTEXT_KEY)?
        .as_str()
        .map(str::to_string)
}

/// Read that verdict back out. Absent key = the binding applied.
fn auth_delegation_error(context: &serde_json::Value) -> Option<String> {
    context
        .get(AUTH_DELEGATION_CONTEXT_KEY)?
        .as_str()
        .map(str::to_string)
}

/// lumen's contribution to the shared operator.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-reconcile-rs.md#source
impl ManagedService for Lumen {
    /// Server-side-apply field manager + leader-election Lease name.
    const MANAGER: &'static str = "lumen-operator";

    fn render(&self) -> Vec<serde_json::Value> {
        render::render(self)
    }

    /// #2678 R7: reject a spec that names two credential sources before any
    /// child object is applied.
    ///
    /// `render` is infallible by contract — six services share that signature —
    /// so the refusal lives here, the one hook on the reconcile path that can
    /// return an error. Failing here means the reconcile fails and the CR does
    /// not converge; the alternative, picking one source by precedence, leaves
    /// an operator reading the credentials they deployed while lumen serves the
    /// other ones. The CRD carries the same rule as CEL, so on a current
    /// cluster this never fires — it is the backstop for an older CRD or an
    /// object written before the rule existed.
    fn reconcile_plan(
        &self,
        client: kube::Client,
    ) -> impl std::future::Future<Output = anyhow::Result<service_k8s::service::ReconcilePlan>> + Send
    {
        let validation = self.spec.validate();
        let children = render::render(self);
        // #2876: the serving ServiceAccount's `system:auth-delegator` binding
        // is cluster-scoped, so it cannot be one of `children` — those are all
        // applied into the CR's namespace. This hook is where it goes: it is
        // the one place on the reconcile path with a client, and applying the
        // grant *before* the workload means the pods do not start serving
        // ahead of their ability to authenticate anyone.
        let lumen = self.clone();
        async move {
            validation.map_err(|why| anyhow::anyhow!(why))?;
            let control = KubeAuthDelegatorControl {
                client: client.clone(),
            };
            let mut context = serde_json::Map::new();
            if let Some(error) = apply_auth_delegator_binding(&control, &lumen).await {
                context.insert(
                    AUTH_DELEGATION_CONTEXT_KEY.to_string(),
                    serde_json::Value::String(error),
                );
            }
            if let Some(error) = check_peer_identity(&lumen) {
                context.insert(
                    PEER_IDENTITY_CONTEXT_KEY.to_string(),
                    serde_json::Value::String(error),
                );
            }
            Ok(service_k8s::service::ReconcilePlan {
                children,
                context: serde_json::Value::Object(context),
            })
        }
    }

    fn prunes(&self) -> Vec<service_k8s::service::PruneTarget> {
        render::prunes(self)
    }

    fn readiness_targets(&self) -> Vec<ReadinessTarget> {
        // The serving fleet is always a StatefulSet (render::render), whether
        // or not raft consensus (`replicasPerShard > 1`) is active.
        let name = self.name_any();
        vec![ReadinessTarget {
            kind: "StatefulSet",
            name,
        }]
    }

    fn status_patch(&self, ready: &ReadyFacts) -> serde_json::Value {
        let obs = self.observe(ready);
        json!({ "status": {
            "phase": obs.phase,
            "observedGeneration": self.metadata.generation.unwrap_or(0),
            "servingReadyReplicas": obs.serving_ready,
            "desiredReplicas": obs.desired,
            "shardCount": self.spec.shard_count,
            "reshard": obs.reshard,
            "message": format!("{}/{} serving pods ready", obs.serving_ready, obs.desired),
        }})
    }

    /// #2601: the Kubernetes-convention convergence surface, derived from the
    /// same [`Observation`] [`Self::status_patch`] projects — so the flat
    /// fields and the conditions can never disagree about whether this CR has
    /// converged.
    ///
    /// Clock-free by construction: the caller stamps `lastTransitionTime`, which
    /// is what keeps this side of the projection deterministic (see the module
    /// doc's no-I/O `status_patch` contract).
    fn conditions(&self, ready: &ReadyFacts, context: &serde_json::Value) -> Vec<ConditionFact> {
        let mut observation = self.observe(ready);
        // #2876 AC4: the plan hook's verdict on the cluster-scoped RBAC write
        // reaches the status here. It cannot come from `observe`, which is
        // I/O-free by contract and has no way to know what the apply did.
        observation.auth_delegation = auth_delegation_error(context);
        // #2890 R4: same channel, same reason — the peer-identity verdict is a
        // Secret read, and `observe` does no I/O.
        observation.peer_identity = peer_identity_error(context);
        observation.conditions()
    }

    /// #2601: the conditions already persisted on this object, so the shared
    /// projection can carry each `lastTransitionTime` forward. `Patch::Merge`
    /// replaces arrays wholesale, so nothing survives server-side unless it is
    /// read back off the watched object and re-sent.
    fn observed_conditions(&self) -> Vec<service_k8s::Condition> {
        self.status
            .as_ref()
            .map(|status| status.conditions.clone())
            .unwrap_or_default()
    }
}

/// The reshard blocking conditions that mean *wedged*, as opposed to the policy
/// states [`crate::operator::crd::LumenSpec::reshard_status`] reports at plain
/// defaults (#2601).
///
/// `maxShardBytesUnset` is present on every CR that has not opted into
/// auto-splitting, and `maxShardsReached` is a configured ceiling rather than a
/// fault; gating `Ready` on the raw `blockingConditions` list would therefore
/// report every default install as permanently not-ready.
const RESHARD_WEDGE_CONDITIONS: [&str; 2] =
    ["reshardOversizedDocument", "topologyConvergenceStalled"];

/// One reconcile's worth of observed facts about a `Lumen` (#2601).
///
/// Both status surfaces — the flat legacy fields and `status.conditions[]` —
/// project from this single computation rather than each re-deriving the
/// reshard state, so they cannot drift apart.
struct Observation {
    serving_ready: i32,
    desired: i32,
    reshard: LumenReshardStatus,
    /// The post-cutover write-pause fence is still armed (#1458 R1).
    awaiting_convergence: bool,
    phase: &'static str,
    /// Why this reconcile could not apply the serving ServiceAccount's
    /// `system:auth-delegator` binding, if it could not (#2876). Filled from
    /// the reconcile context, never from `observe` — the apply is I/O and
    /// `observe` is not allowed to do any.
    auth_delegation: Option<String>,
    /// Why this replicated instance has no usable Raft peer identity, if it
    /// has none (#2890). Same provenance as `auth_delegation`, and empty for
    /// every single-replica instance.
    peer_identity: Option<String>,
    /// Whether this instance runs a replicated Raft group at all — the one
    /// piece of the peer-identity story `observe` *can* derive, since it is
    /// pure spec.
    peer_identity_required: bool,
}

impl Observation {
    /// Is a reshard workflow actually in flight? Either a non-`Complete` phase,
    /// or the post-cutover fence still waiting to clear — the latter happens
    /// *at* phase `Complete`, which is why it is a separate disjunct.
    fn reshard_active(&self) -> bool {
        self.reshard.phase != ReshardPhase::Complete.as_str() || self.awaiting_convergence
    }

    /// The clock-free condition facts for this observation, in printed order.
    fn conditions(&self) -> Vec<ConditionFact> {
        let replicas = format!("{}/{} serving pods ready", self.serving_ready, self.desired);
        let replicas_ready = self.serving_ready >= self.desired;
        let wedge = self
            .reshard
            .blocking_conditions
            .iter()
            .find(|c| RESHARD_WEDGE_CONDITIONS.contains(&c.as_str()));

        let ready = match (
            self.auth_delegation
                .as_deref()
                .map(|error| ("AuthDelegationNotGranted", error))
                // #2890 R4 AC2: outranks everything below for the same reason
                // the auth-delegation verdict does — a replicated group with no
                // peer identity has no authenticated way to replicate, and its
                // pods refuse to start rather than fall back to plaintext.
                // Ordered after it only because a broken TokenReview grant
                // fails every request, while this one fails replication.
                .or_else(|| {
                    self.peer_identity
                        .as_deref()
                        .map(|error| ("PeerIdentityNotConfigured", error))
                }),
            wedge,
            replicas_ready,
        ) {
            // #2876 AC4. This outranks everything below it: without the
            // `system:auth-delegator` binding the serving pods cannot run a
            // TokenReview, so every request fails authentication no matter how
            // many pods are Ready or how healthy the shard map is. Reporting
            // Ready here would be reporting on a data plane that answers 401
            // to its own operator.
            (Some((reason, error)), _, _) => {
                ConditionFact::new("Ready", ConditionStatus::False, reason, error.to_string())
            }
            // A wedge outranks a healthy replica count: every pod can be Ready
            // while writes are fenced or a batch is unappliable.
            (None, Some(wedge), _) => ConditionFact::new(
                "Ready",
                ConditionStatus::False,
                "ReshardWedged",
                format!("{wedge}: {}", self.reshard.message),
            ),
            (None, None, true) => ConditionFact::new(
                "Ready",
                ConditionStatus::True,
                "AllReplicasReady",
                replicas.clone(),
            ),
            (None, None, false) => ConditionFact::new(
                "Ready",
                ConditionStatus::False,
                "ReplicasNotReady",
                replicas.clone(),
            ),
        };

        // #2876 AC4: a condition of its own, not just a Ready reason. A
        // watcher that only sees `Ready=False/AuthDelegationNotGranted`
        // learns nothing once something else takes over the Ready slot; this
        // one keeps reporting the RBAC state on its own terms, and names the
        // exact operation that was refused.
        let auth_delegation = match &self.auth_delegation {
            Some(error) => ConditionFact::new(
                "AuthDelegationReady",
                ConditionStatus::False,
                "ClusterRoleBindingFailed",
                error.clone(),
            ),
            None => ConditionFact::new(
                "AuthDelegationReady",
                ConditionStatus::True,
                "AuthDelegatorBound",
                "serving ServiceAccount is bound to system:auth-delegator".to_string(),
            ),
        };

        // #2890 R4 AC2: its own condition for the same reason
        // `AuthDelegationReady` is one — a watcher that only reads `Ready`
        // learns nothing about peer identity once something else takes the
        // Ready slot, and this is the condition that names the spec field and required keys.
        let peer_identity = match &self.peer_identity {
            Some(error) => ConditionFact::new(
                "PeerIdentityReady",
                ConditionStatus::False,
                "PeerTlsSecretNotNamed",
                error.clone(),
            ),
            // True for a single-replica instance too, with a reason that says
            // why rather than implying material was found: there is no peer to
            // authenticate, so nothing is outstanding.
            None if !self.peer_identity_required => ConditionFact::new(
                "PeerIdentityReady",
                ConditionStatus::True,
                "NoReplicatedPeers",
                "single-member shard: no Raft peer transport to authenticate".to_string(),
            ),
            None => ConditionFact::new(
                "PeerIdentityReady",
                ConditionStatus::True,
                "PeerTlsSecretProjected",
                "spec.peerTlsSecret is configured; peer TLS material is required at member startup".to_string(),
            ),
        };

        let progressing = if !replicas_ready {
            ConditionFact::new(
                "Progressing",
                ConditionStatus::True,
                "ReplicasConverging",
                replicas,
            )
        } else if self.reshard_active() {
            ConditionFact::new(
                "Progressing",
                ConditionStatus::True,
                "ReshardInFlight",
                self.reshard.message.clone(),
            )
        } else {
            ConditionFact::new(
                "Progressing",
                ConditionStatus::False,
                "Converged",
                "spec is fully reconciled".to_string(),
            )
        };

        let reshard = if self.reshard_active() {
            // Reason tracks the workflow's own vocabulary so a watcher can read
            // the phase straight off the condition; the fence-only case has no
            // phase of its own to report.
            let reason = if self.reshard.phase == ReshardPhase::Complete.as_str() {
                "AwaitingTopologyConvergence".to_string()
            } else {
                self.reshard.phase.clone()
            };
            ConditionFact::new(
                "ReshardInProgress",
                ConditionStatus::True,
                reason,
                self.reshard.message.clone(),
            )
        } else {
            ConditionFact::new(
                "ReshardInProgress",
                ConditionStatus::False,
                "Complete",
                self.reshard.message.clone(),
            )
        };

        vec![
            ready,
            progressing,
            reshard,
            auth_delegation,
            peer_identity,
        ]
    }
}

impl Lumen {
    /// Compute this reconcile's [`Observation`] — the single source both status
    /// surfaces project from (#2601). Synchronous and I/O-free, per the module
    /// doc's contract; the live usage read is a cache lookup, not a scrape.
    fn observe(&self, ready: &ReadyFacts) -> Observation {
        let name = self.name_any();
        let serving_ready = ready.ready.get(&name).copied().unwrap_or(0) as i32;
        let desired = self.spec.storage_pod_count();
        let usage = shard_usage_cache()
            .lock()
            .ok()
            .and_then(|cache| cache.get(&cache_key(self)).cloned());
        let mut reshard = match usage {
            Some(snapshot) if !snapshot.usage.is_empty() => self
                .spec
                .reshard_status_with_usage(&snapshot.usage, snapshot.measured_at_map_version),
            _ => self.spec.reshard_status(),
        };
        // #1444 R2: an oversized single-document batch the reshard driver
        // cannot apply is a distinct, named blocking condition (not the
        // generic threshold/policy conditions `reshard_status*` already
        // computes above) with its own remediation text — layered on here
        // rather than inside `LumenSpec::reshard_status*` because it comes
        // from the driver's own live apply attempts, not from spec/usage.
        let namespace = self.namespace().unwrap_or_else(|| "default".to_string());
        let uid = self.uid().unwrap_or_default();
        if let Some(block) =
            crate::operator::reshard_driver::oversize_block_condition(&namespace, &name, &uid)
        {
            reshard
                .blocking_conditions
                .push("reshardOversizedDocument".to_string());
            reshard.message = block.to_string();
        }
        // #1458 R1: post-cutover topology-convergence pending — derived
        // purely from persisted spec state (`shardMap.version` vs
        // `workflow.convergedShardMapVersion`), the same check
        // `reshard_driver::advance_convergence` runs each tick, so this
        // needs no driver-side cache read. Only sets the message if a more
        // severe oversize wedge did not already claim it above.
        //
        // #1467 R7: also require `workflow.lastCutoverShardMapVersion ==
        // shardMap.version` — the same gate `advance_convergence` itself
        // uses to decide whether to engage the fence loop at all. Without
        // this, a manually-authored/edited `shardMap.version` (one the
        // driver never cut over to, so it never arms a fence or advances
        // convergence) would report `awaitingTopologyConvergence` forever,
        // even though nothing is actually blocking writes.
        let map_version = self.spec.shard_map.version;
        let workflow = &self.spec.reshard_policy.workflow;
        let awaiting_convergence = map_version > 0
            && workflow.converged_shard_map_version != Some(map_version)
            && workflow.last_cutover_shard_map_version == Some(map_version);
        if awaiting_convergence {
            reshard
                .blocking_conditions
                .push("awaitingTopologyConvergence".to_string());
            if !reshard
                .blocking_conditions
                .contains(&"reshardOversizedDocument".to_string())
            {
                reshard.message = format!(
                    "waiting for every serving pod to become Ready on shardMap version \
                     {map_version} before the post-cutover write-pause fence is cleared"
                );
            }
        }
        // #1467 R7: distinct, named condition once the wait above has
        // exceeded the stall budget — the driver keeps re-arming the fence
        // (never silently drops it), but operators need a visible signal
        // that convergence has been pending unusually long, not just that
        // it's pending.
        //
        // #1485 R2: computed purely from the persisted `workflow.
        // convergenceWaitStartedAt` checkpoint (the same field `advance_
        // convergence` itself stamps and reads), not the driver's
        // process-local stall-tracking cache — so this condition, and the
        // budget it is derived from, survive an operator restart mid-wait
        // rather than resetting to "not stalled" until the cache re-fills.
        if awaiting_convergence
            && crate::operator::reshard_driver::convergence_stall_condition(
                workflow.convergence_wait_started_at,
            )
        {
            reshard
                .blocking_conditions
                .push("topologyConvergenceStalled".to_string());
            // #1485 R1: surface the bounded remediation restart's own
            // re-trigger count/timestamp alongside the condition, so
            // operators can see the self-heal fired without reading driver
            // logs.
            reshard.convergence_remediation_restart_count =
                workflow.convergence_remediation_restart_count;
            reshard.convergence_remediation_restarted_at =
                workflow.convergence_remediation_restarted_at;
            if !reshard
                .blocking_conditions
                .contains(&"reshardOversizedDocument".to_string())
            {
                reshard.message = format!(
                    "topology convergence on shardMap version {map_version} has not been \
                     confirmed after an extended wait; the write-pause fence remains armed \
                     and is being kept re-armed"
                );
            }
        }
        let phase = if serving_ready >= desired {
            "Ready"
        } else if serving_ready > 0 {
            "Reconciling"
        } else {
            "Pending"
        };
        Observation {
            serving_ready,
            desired,
            reshard,
            awaiting_convergence,
            phase,
            // Not knowable here: applying the binding is I/O, and this
            // function is synchronous and I/O-free by the module's contract.
            // `conditions` fills it from the reconcile context (#2876).
            auth_delegation: None,
            // Same for the Secret read behind this one (#2890) — but whether
            // the instance owes peer identity at all is pure spec, so that half
            // is derivable here.
            peer_identity: None,
            peer_identity_required: self.spec.peer_identity_required(),
        }
    }
}

/// `lumen k8s operator run` — run the reconcile controller on the shared
/// `libs/service-k8s` host (leader-gated; safe at `replicas > 1`), alongside
/// the live shard-usage measurement loop (#1319 R1; every replica runs it,
/// not just the leader — see [`spawn_shard_usage_loop`]), the autonomous
/// reshard phase driver (#1319 R2, #1381; independently leader-gated — see
/// [`crate::operator::reshard_driver::spawn_reshard_driver_loop`]), and the
/// HPA topology-transition handoff loop (#1385; independently leader-gated —
/// see [`spawn_hpa_handoff_loop`]), the fleet materialization loop
/// (independently leader-gated — see
/// [`crate::operator::fleet::spawn_fleet_loop`]), and the auth-delegator
/// binding sweep (#2876; independently leader-gated — see
/// [`spawn_auth_delegator_sweep_loop`], which cleans up the one child no owner
/// reference can reach).
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-reconcile-rs.md#source
pub async fn run() -> anyhow::Result<()> {
    match Client::try_default().await {
        Ok(client) => {
            spawn_shard_usage_loop(client.clone());
            crate::operator::reshard_driver::spawn_reshard_driver_loop(client.clone());
            crate::operator::fleet::spawn_fleet_loop(client.clone());
            spawn_hpa_handoff_loop(client.clone());
            spawn_auth_delegator_sweep_loop(client);
        }
        Err(err) => {
            tracing::warn!(
                error = %err,
                "reshard live-usage measurement + phase-driver + fleet + HPA-handoff + auth-delegator-sweep loops disabled: could not build a kube client"
            );
        }
    }
    service_k8s::run::<Lumen>().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn body_with(bytes: u64) -> String {
        format!(
            "# HELP lumen_storage_bytes docs\n\
             # TYPE lumen_storage_bytes gauge\n\
             lumen_storage_bytes {bytes}\n"
        )
    }

    #[test]
    fn parse_metric_reads_matching_gauge_line() {
        let body = body_with(2048);
        assert_eq!(parse_metric(&body, "lumen_storage_bytes"), Some(2048));
    }

    #[test]
    fn parse_metric_missing_metric_is_none() {
        let body = "lumen_docs_total 3\n";
        assert_eq!(parse_metric(body, "lumen_storage_bytes"), None);
    }

    #[tokio::test]
    async fn pod_storage_bytes_fetches_and_parses() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/metrics"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body_with(4096)))
            .mount(&server)
            .await;
        let http = reqwest::Client::new();
        let url = format!("{}/metrics", server.uri());
        assert_eq!(pod_storage_bytes(&http, &url).await, Some(4096));
    }

    #[tokio::test]
    async fn pod_storage_bytes_unreachable_pod_is_none() {
        let http = reqwest::Client::new();
        // No listener on this port; connection should fail promptly.
        let url = "http://127.0.0.1:1/metrics";
        assert_eq!(pod_storage_bytes(&http, url).await, None);
    }

    #[tokio::test]
    async fn aggregate_shard_usage_takes_max_within_a_shard() {
        // shard 0 has two replicas (2048, 8192 bytes); max, not sum, wins.
        let a = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/metrics"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body_with(2048)))
            .mount(&a)
            .await;
        let b = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/metrics"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body_with(8192)))
            .mount(&b)
            .await;
        let http = reqwest::Client::new();
        let urls = vec![
            (0u32, format!("{}/metrics", a.uri())),
            (0u32, format!("{}/metrics", b.uri())),
        ];
        let usage = aggregate_shard_usage(&http, &urls).await;
        assert_eq!(usage.get(&0), Some(&8192));
    }

    #[tokio::test]
    async fn aggregate_shard_usage_skips_unreachable_pods() {
        let a = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/metrics"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body_with(1024)))
            .mount(&a)
            .await;
        let http = reqwest::Client::new();
        let urls = vec![
            (0u32, "http://127.0.0.1:1/metrics".to_string()),
            (1u32, format!("{}/metrics", a.uri())),
        ];
        let usage = aggregate_shard_usage(&http, &urls).await;
        assert_eq!(usage.get(&0), None);
        assert_eq!(usage.get(&1), Some(&1024));
    }

    #[test]
    fn pod_metrics_urls_covers_every_storage_pod_by_headless_dns() {
        use crate::operator::crd::{LumenSpec, ServingSpec, ShardMapSpec};
        let spec = LumenSpec {
            image: "lumen:latest".into(),
            image_pull_policy: None,
            placement: Default::default(),
            shard_count: 2,
            shard_map: ShardMapSpec::default(),
            replicas_per_shard: 3,
            voter_count: 3,
            log_format: Default::default(),
            log_level: None,
            auth: Default::default(),
            serving: ServingSpec::default(),
            reshard_policy: Default::default(),
            observability: false,
            network_policy: false,
            admission: None,
            service_account_name: None,
            service_account_annotations: BTreeMap::new(),
            peer_tls_secret: None,
            serving_tls_secret: None,
            body_limit_bytes: None,
        };
        let mut lumen = Lumen::new("search", spec);
        lumen.metadata.namespace = Some("acme".to_string());

        let urls = pod_metrics_urls(&lumen);
        assert_eq!(urls.len(), 6);
        assert!(urls.contains(&(
            0,
            "http://search-0.search-headless.acme.svc.cluster.local:7373/metrics".to_string()
        )));
        // ordinal 3 = replica_index 1, shard_index 1 (3 % 2 == 1).
        assert!(urls.contains(&(
            1,
            "http://search-3.search-headless.acme.svc.cluster.local:7373/metrics".to_string()
        )));
    }

    // ---- #1444 R2: oversized-doc blocking condition in status.reshard -----

    #[test]
    fn status_patch_surfaces_oversize_block_as_distinct_reshard_condition() {
        let lumen = hpa_test_lumen("search", "acme-status-oversize", 2, 1);
        let namespace = "acme-status-oversize";
        let name = "search";
        crate::operator::reshard_driver::record_oversize_block(
            namespace,
            name,
            "",
            crate::operator::reshard_driver::OversizedDocumentBlock {
                collection: "widgets".to_string(),
                external_id: "doc-42".to_string(),
                bytes: 9_000_000,
            },
        );

        let ready = ReadyFacts {
            ready: std::collections::HashMap::new(),
        };
        let patch = lumen.status_patch(&ready);
        let reshard = &patch["status"]["reshard"];
        let blocking = reshard["blockingConditions"]
            .as_array()
            .expect("blockingConditions must be present");
        assert!(
            blocking
                .iter()
                .any(|c| c.as_str() == Some("reshardOversizedDocument")),
            "status.reshard.blockingConditions must include reshardOversizedDocument, got: {reshard}"
        );
        let message = reshard["message"].as_str().unwrap_or_default();
        assert!(
            message.contains("widgets") && message.contains("doc-42"),
            "status.reshard.message must name the collection and external_id, got: {message}"
        );

        crate::operator::reshard_driver::clear_oversize_block(namespace, name);
    }

    #[test]
    fn status_patch_has_no_oversize_condition_when_none_recorded() {
        let lumen = hpa_test_lumen("search", "acme-status-clean", 2, 1);
        let ready = ReadyFacts {
            ready: std::collections::HashMap::new(),
        };
        let patch = lumen.status_patch(&ready);
        let reshard = &patch["status"]["reshard"];
        let blocking = reshard["blockingConditions"].as_array();
        let has_condition = blocking
            .map(|arr| {
                arr.iter()
                    .any(|c| c.as_str() == Some("reshardOversizedDocument"))
            })
            .unwrap_or(false);
        assert!(
            !has_condition,
            "no oversize wedge was recorded for this namespace/name; \
             status.reshard.blockingConditions must not report one, got: {reshard}"
        );
    }

    /// #1458 R4/AC4: a CR recorded an oversize wedge under `uid` "old-uid";
    /// a deleted-and-recreated CR under the *same* namespace/name gets a
    /// fresh `uid` from the API server, so its status must be clean
    /// immediately — not wait for `prune_oversize_cache`'s next poll to
    /// catch up with the driver-loop's live-CR listing.
    #[test]
    fn status_patch_is_clean_for_a_recreated_cr_with_a_stale_cached_uid() {
        let namespace = "acme-status-recreated";
        let name = "search";
        crate::operator::reshard_driver::record_oversize_block(
            namespace,
            name,
            "old-uid",
            crate::operator::reshard_driver::OversizedDocumentBlock {
                collection: "widgets".to_string(),
                external_id: "doc-42".to_string(),
                bytes: 9_000_000,
            },
        );

        let mut lumen = hpa_test_lumen(name, namespace, 2, 1);
        lumen.metadata.uid = Some("new-uid".to_string());

        let ready = ReadyFacts {
            ready: std::collections::HashMap::new(),
        };
        let patch = lumen.status_patch(&ready);
        let reshard = &patch["status"]["reshard"];
        let blocking = reshard["blockingConditions"].as_array();
        let has_condition = blocking
            .map(|arr| {
                arr.iter()
                    .any(|c| c.as_str() == Some("reshardOversizedDocument"))
            })
            .unwrap_or(false);
        assert!(
            !has_condition,
            "a recreated CR (new uid) must not inherit the deleted CR's stale oversize wedge \
             cached under the old uid, got: {reshard}"
        );

        crate::operator::reshard_driver::clear_oversize_block(namespace, name);
    }

    // ---- #1467 R7: bounded topology-convergence stall escalation ----------

    /// A `shardMap.version` the workflow actually cut over to
    /// (`lastCutoverShardMapVersion == shardMap.version`), still unconverged
    /// (`convergedShardMapVersion` absent): the shape `status_patch`'s
    /// `awaitingTopologyConvergence` gate requires.
    fn cutover_pending_convergence_lumen(name: &str, ns: &str, map_version: u64) -> Lumen {
        let mut lumen = hpa_test_lumen(name, ns, 2, 1);
        lumen.spec.shard_map.version = map_version;
        lumen
            .spec
            .reshard_policy
            .workflow
            .last_cutover_shard_map_version = Some(map_version);
        lumen
    }

    #[test]
    fn status_patch_reports_awaiting_convergence_without_a_stall_condition_before_the_budget() {
        let lumen = cutover_pending_convergence_lumen("search", "acme-convergence-fresh", 1);
        let ready = ReadyFacts {
            ready: std::collections::HashMap::new(),
        };
        let patch = lumen.status_patch(&ready);
        let reshard = &patch["status"]["reshard"];
        let blocking = reshard["blockingConditions"]
            .as_array()
            .expect("blockingConditions must be present");
        assert!(
            blocking
                .iter()
                .any(|c| c.as_str() == Some("awaitingTopologyConvergence")),
            "got: {reshard}"
        );
        assert!(
            !blocking
                .iter()
                .any(|c| c.as_str() == Some("topologyConvergenceStalled")),
            "a freshly-awaiting convergence (no recorded stall ticks yet) must not report the \
             stalled condition, got: {reshard}"
        );
    }

    /// Wall-clock "epoch seconds" helper duplicated from `reshard_driver`'s
    /// own private one (not exposed beyond `pub fn
    /// convergence_stall_budget_secs`) — this test only needs `now`, not the
    /// budget constant itself, to back-date `convergenceWaitStartedAt`.
    fn test_now_epoch_secs() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    #[test]
    fn status_patch_surfaces_topology_convergence_stall_as_distinct_condition() {
        let namespace = "acme-convergence-stalled";
        let name = "search";
        let map_version = 1u64;
        let mut lumen = cutover_pending_convergence_lumen(name, namespace, map_version);

        // #1485 R2: the stall budget is now computed purely from the
        // persisted `workflow.convergenceWaitStartedAt` checkpoint — no
        // driver-side cache to drive here — so simulate an extended wait by
        // back-dating it past `convergence_stall_budget_secs()` directly,
        // exactly what a real long-running wait (or a wait that started
        // before the operator restarted) would leave behind in the CR.
        let stall_budget = crate::operator::reshard_driver::convergence_stall_budget_secs();
        lumen
            .spec
            .reshard_policy
            .workflow
            .convergence_wait_started_at =
            Some(test_now_epoch_secs().saturating_sub(stall_budget + 1));

        let ready = ReadyFacts {
            ready: std::collections::HashMap::new(),
        };
        let patch = lumen.status_patch(&ready);
        let reshard = &patch["status"]["reshard"];
        let blocking = reshard["blockingConditions"]
            .as_array()
            .expect("blockingConditions must be present");
        assert!(
            blocking
                .iter()
                .any(|c| c.as_str() == Some("awaitingTopologyConvergence")),
            "topologyConvergenceStalled must be layered on top of, not instead of, \
             awaitingTopologyConvergence, got: {reshard}"
        );
        assert!(
            blocking
                .iter()
                .any(|c| c.as_str() == Some("topologyConvergenceStalled")),
            "expected a distinct topologyConvergenceStalled condition once the stall budget \
             is exceeded, got: {reshard}"
        );
    }

    /// #1485 R2/AC2: the raised `topologyConvergenceStalled` condition is a
    /// pure function of the persisted CR (`workflow.convergenceWaitStartedAt`
    /// alone) — it is computed identically whether or not the operator
    /// process serving this reconcile has ever seen this CR before, i.e. it
    /// survives an operator restart by construction, unlike the pre-#1485
    /// process-local-cache-only computation which required 30+ consecutive
    /// in-process ticks to re-accumulate before re-raising.
    #[test]
    fn status_patch_stalled_condition_survives_a_fresh_process_seeing_the_cr_for_the_first_time() {
        let map_version = 1u64;
        let mut lumen = cutover_pending_convergence_lumen(
            "search",
            "acme-convergence-restart-durable",
            map_version,
        );
        let stall_budget = crate::operator::reshard_driver::convergence_stall_budget_secs();
        lumen
            .spec
            .reshard_policy
            .workflow
            .convergence_wait_started_at =
            Some(test_now_epoch_secs().saturating_sub(stall_budget + 1));
        // #1485 R1: also prove a completed remediation restart's own
        // bookkeeping round-trips through status untouched by process
        // identity — status_patch never resets it.
        lumen
            .spec
            .reshard_policy
            .workflow
            .convergence_remediation_restart_count = 1;
        lumen
            .spec
            .reshard_policy
            .workflow
            .convergence_remediation_restarted_at = Some(test_now_epoch_secs());

        // No driver-side cache is ever populated in this test process for
        // this namespace/name — `status_patch` (called by whichever operator
        // replica happens to reconcile this CR next) must still report the
        // stall purely from `lumen.spec` above.
        let ready = ReadyFacts {
            ready: std::collections::HashMap::new(),
        };
        let patch = lumen.status_patch(&ready);
        let reshard = &patch["status"]["reshard"];
        let blocking = reshard["blockingConditions"]
            .as_array()
            .expect("blockingConditions must be present");
        assert!(
            blocking
                .iter()
                .any(|c| c.as_str() == Some("topologyConvergenceStalled")),
            "stalled condition must be derived purely from persisted spec state, got: {reshard}"
        );
        assert_eq!(
            reshard["convergenceRemediationRestartCount"].as_u64(),
            Some(1),
            "convergenceRemediationRestartCount must be surfaced in status.reshard, got: {reshard}"
        );
        assert!(
            reshard["convergenceRemediationRestartedAt"].is_number(),
            "convergenceRemediationRestartedAt must be surfaced in status.reshard, got: {reshard}"
        );
    }

    #[test]
    fn status_patch_never_reports_awaiting_convergence_for_a_manually_authored_map_version() {
        // #1467 R7: a shardMap.version the driver never itself cut over to
        // (lastCutoverShardMapVersion absent/stale) must not report
        // awaitingTopologyConvergence at all — status_patch uses the same
        // gate advance_convergence does, so a manually-edited map version
        // never wedges status forever waiting on a fence the driver never
        // armed.
        let mut lumen = hpa_test_lumen("search", "acme-convergence-manual", 2, 1);
        lumen.spec.shard_map.version = 5;
        // last_cutover_shard_map_version left at its default (None).
        let ready = ReadyFacts {
            ready: std::collections::HashMap::new(),
        };
        let patch = lumen.status_patch(&ready);
        let reshard = &patch["status"]["reshard"];
        let blocking = reshard["blockingConditions"].as_array();
        let has_condition = blocking
            .map(|arr| {
                arr.iter()
                    .any(|c| c.as_str() == Some("awaitingTopologyConvergence"))
            })
            .unwrap_or(false);
        assert!(
            !has_condition,
            "a shardMap.version with no matching lastCutoverShardMapVersion must not report \
             awaitingTopologyConvergence, got: {reshard}"
        );
    }

    // ---- #2601: metav1.Condition convergence surface -----------------------

    /// `ReadyFacts` reporting `count` ready pods for `name`'s StatefulSet.
    fn ready_facts(name: &str, count: i64) -> ReadyFacts {
        let mut ready = std::collections::HashMap::new();
        ready.insert(name.to_string(), count);
        ReadyFacts { ready }
    }

    fn condition<'a>(facts: &'a [ConditionFact], type_: &str) -> &'a ConditionFact {
        facts
            .iter()
            .find(|c| c.type_ == type_)
            .unwrap_or_else(|| panic!("expected a `{type_}` condition, got: {facts:?}"))
    }

    #[test]
    fn a_fully_ready_default_cr_reports_ready_true() {
        // The load-bearing case: at plain defaults `reshard_status` always
        // reports `maxShardBytesUnset` in `blockingConditions`. Gating `Ready`
        // on that list would leave every install that never opted into
        // auto-splitting permanently not-ready.
        let lumen = hpa_test_lumen("search", "acme-cond-ready", 2, 1);
        let facts = lumen.conditions(&ready_facts("search", 2), &serde_json::Value::Null);

        let ready = condition(&facts, "Ready");
        assert_eq!(ready.status, ConditionStatus::True, "got: {facts:?}");
        assert_eq!(ready.reason, "AllReplicasReady");
        assert_eq!(
            condition(&facts, "Progressing").status,
            ConditionStatus::False,
            "a settled CR is not progressing, got: {facts:?}"
        );
        assert_eq!(
            condition(&facts, "ReshardInProgress").status,
            ConditionStatus::False,
            "no reshard workflow is in flight, got: {facts:?}"
        );
    }

    #[test]
    fn short_replicas_report_ready_false_and_progressing_true() {
        let lumen = hpa_test_lumen("search", "acme-cond-short", 2, 1);
        let facts = lumen.conditions(&ready_facts("search", 1), &serde_json::Value::Null);

        let ready = condition(&facts, "Ready");
        assert_eq!(ready.status, ConditionStatus::False, "got: {facts:?}");
        assert_eq!(ready.reason, "ReplicasNotReady");
        assert!(
            ready.message.contains("1/2"),
            "the message must name the counts, got: {ready:?}"
        );

        let progressing = condition(&facts, "Progressing");
        assert_eq!(progressing.status, ConditionStatus::True);
        assert_eq!(progressing.reason, "ReplicasConverging");
    }

    #[test]
    fn a_reshard_wedge_outranks_a_healthy_replica_count() {
        // Every pod Ready, but writes are unappliable: `Ready=True` here would
        // tell `kubectl wait` the CR converged while it is in fact stuck.
        let namespace = "acme-cond-wedged";
        let name = "search";
        let lumen = hpa_test_lumen(name, namespace, 2, 1);
        crate::operator::reshard_driver::record_oversize_block(
            namespace,
            name,
            "",
            crate::operator::reshard_driver::OversizedDocumentBlock {
                collection: "widgets".to_string(),
                external_id: "doc-42".to_string(),
                bytes: 9_000_000,
            },
        );

        let facts = lumen.conditions(&ready_facts(name, 2), &serde_json::Value::Null);
        crate::operator::reshard_driver::clear_oversize_block(namespace, name);

        let ready = condition(&facts, "Ready");
        assert_eq!(ready.status, ConditionStatus::False, "got: {facts:?}");
        assert_eq!(ready.reason, "ReshardWedged");
        assert!(
            ready.message.contains("reshardOversizedDocument") && ready.message.contains("doc-42"),
            "the message must name the wedge and its remediation detail, got: {ready:?}"
        );
    }

    #[test]
    fn the_post_cutover_fence_reports_reshard_in_progress_at_phase_complete() {
        // `awaitingTopologyConvergence` happens *at* phase `Complete`, which is
        // why `reshard_active` cannot be a phase comparison alone.
        let lumen = cutover_pending_convergence_lumen("search", "acme-cond-fence", 1);
        let facts = lumen.conditions(&ready_facts("search", 2), &serde_json::Value::Null);

        let reshard = condition(&facts, "ReshardInProgress");
        assert_eq!(reshard.status, ConditionStatus::True, "got: {facts:?}");
        assert_eq!(reshard.reason, "AwaitingTopologyConvergence");
        assert_eq!(
            condition(&facts, "Progressing").reason,
            "ReshardInFlight",
            "got: {facts:?}"
        );
        assert_eq!(
            condition(&facts, "Ready").status,
            ConditionStatus::True,
            "an armed fence is expected mid-reshard, not a wedge — only a stall is, \
             got: {facts:?}"
        );
    }

    #[test]
    fn a_stalled_fence_is_a_wedge() {
        let mut lumen = cutover_pending_convergence_lumen("search", "acme-cond-stalled", 1);
        let stall_budget = crate::operator::reshard_driver::convergence_stall_budget_secs();
        lumen
            .spec
            .reshard_policy
            .workflow
            .convergence_wait_started_at =
            Some(test_now_epoch_secs().saturating_sub(stall_budget + 1));

        let facts = lumen.conditions(&ready_facts("search", 2), &serde_json::Value::Null);
        let ready = condition(&facts, "Ready");
        assert_eq!(ready.status, ConditionStatus::False, "got: {facts:?}");
        assert_eq!(ready.reason, "ReshardWedged");
    }

    #[test]
    fn conditions_are_a_pure_function_of_spec_and_observed_facts() {
        // The determinism the clock-free split exists to preserve: no wall
        // clock is read here, so repeated projection is byte-identical.
        let lumen = hpa_test_lumen("search", "acme-cond-deterministic", 2, 1);
        let ready = ready_facts("search", 1);
        assert_eq!(
            lumen.conditions(&ready, &serde_json::Value::Null),
            lumen.conditions(&ready, &serde_json::Value::Null)
        );
    }

    #[test]
    fn the_flat_status_and_the_conditions_agree_on_readiness() {
        // Both surfaces project from one `Observation`; this pins that they
        // cannot drift as either side grows.
        let lumen = hpa_test_lumen("search", "acme-cond-agree", 2, 1);
        for count in [0i64, 1, 2] {
            let ready = ready_facts("search", count);
            let phase = lumen.status_patch(&ready)["status"]["phase"]
                .as_str()
                .expect("phase")
                .to_string();
            let ready_condition =
                condition(&lumen.conditions(&ready, &serde_json::Value::Null), "Ready").status;
            assert_eq!(
                phase == "Ready",
                ready_condition == ConditionStatus::True,
                "phase {phase:?} disagrees with the Ready condition at {count} ready pods"
            );
        }
    }

    #[test]
    fn observed_conditions_round_trip_through_the_projection() {
        // The `Patch::Merge` array-replacement trap: unless prior conditions
        // are read back off the watched object, every reconcile would restamp
        // `lastTransitionTime` and every watcher would see the 30s requeue as
        // a state change.
        let mut lumen = hpa_test_lumen("search", "acme-cond-transition", 2, 1);
        let ready = ready_facts("search", 2);
        let first = service_k8s::service::project(
            &lumen.observed_conditions(),
            lumen.conditions(&ready, &serde_json::Value::Null),
            1,
            "2026-07-25T00:00:00Z",
        );
        assert!(lumen.observed_conditions().is_empty());

        lumen.status = Some(crate::operator::crd::LumenStatus {
            conditions: first.clone(),
            ..Default::default()
        });
        assert_eq!(lumen.observed_conditions(), first);

        let second = service_k8s::service::project(
            &lumen.observed_conditions(),
            lumen.conditions(&ready, &serde_json::Value::Null),
            2,
            "2026-07-25T01:00:00Z",
        );
        assert_eq!(
            second[0].last_transition_time, "2026-07-25T00:00:00Z",
            "an unchanged status must keep its original transition time"
        );
        assert_eq!(
            second[0].observed_generation,
            Some(2),
            "observedGeneration tracks every reconcile, unlike lastTransitionTime"
        );
    }

    // ---- HPA topology-transition handoff (#1385, AC1) ----------------------

    /// In-memory [`HpaControl`]: a `(namespace, name) -> labels` map plus a
    /// record of every `delete_hpa` call, mirroring
    /// `reshard_driver::tests::FakeControl`'s role for that module's
    /// `ClusterControl` seam.
    #[derive(Default)]
    struct FakeHpaControl {
        objects: Mutex<BTreeMap<(String, String), BTreeMap<String, String>>>,
        deletes: Mutex<Vec<(String, String)>>,
    }

    impl FakeHpaControl {
        fn with(ns: &str, name: &str, labels: BTreeMap<String, String>) -> Self {
            let control = Self::default();
            control
                .objects
                .lock()
                .unwrap()
                .insert((ns.to_string(), name.to_string()), labels);
            control
        }
    }

    #[async_trait::async_trait]
    impl HpaControl for FakeHpaControl {
        async fn hpa_labels(
            &self,
            namespace: &str,
            name: &str,
        ) -> anyhow::Result<Option<BTreeMap<String, String>>> {
            Ok(self
                .objects
                .lock()
                .unwrap()
                .get(&(namespace.to_string(), name.to_string()))
                .cloned())
        }

        async fn delete_hpa(&self, namespace: &str, name: &str) -> anyhow::Result<()> {
            let key = (namespace.to_string(), name.to_string());
            self.objects.lock().unwrap().remove(&key);
            self.deletes.lock().unwrap().push(key);
            Ok(())
        }
    }

    fn hpa_test_spec(shard_count: u32, replicas_per_shard: u32) -> crate::operator::crd::LumenSpec {
        use crate::operator::crd::{LumenSpec, ServingSpec, ShardMapSpec};
        LumenSpec {
            image: "lumen:latest".into(),
            image_pull_policy: None,
            placement: Default::default(),
            shard_count,
            shard_map: ShardMapSpec::default(),
            replicas_per_shard,
            voter_count: replicas_per_shard,
            log_format: Default::default(),
            log_level: None,
            auth: Default::default(),
            serving: ServingSpec::default(),
            reshard_policy: Default::default(),
            observability: false,
            network_policy: false,
            admission: None,
            service_account_name: None,
            service_account_annotations: std::collections::BTreeMap::new(),
            peer_tls_secret: None,
            serving_tls_secret: None,
            body_limit_bytes: None,
        }
    }

    fn hpa_test_lumen(name: &str, ns: &str, shard_count: u32, replicas_per_shard: u32) -> Lumen {
        let mut lumen = Lumen::new(name, hpa_test_spec(shard_count, replicas_per_shard));
        lumen.metadata.namespace = Some(ns.to_string());
        lumen
    }

    #[tokio::test]
    async fn prune_stale_hpa_deletes_operator_rendered_hpa_on_multi_shard() {
        let lumen = hpa_test_lumen("search", "acme", 3, 1);
        let control = FakeHpaControl::with("acme", "search", render::hpa_labels(&lumen));

        prune_stale_hpa(&control, &lumen).await;

        assert_eq!(
            control.deletes.lock().unwrap().as_slice(),
            &[("acme".to_string(), "search".to_string())]
        );
        assert!(control
            .objects
            .lock()
            .unwrap()
            .get(&("acme".to_string(), "search".to_string()))
            .is_none());
    }

    #[tokio::test]
    async fn prune_stale_hpa_deletes_legacy_hpa_on_single_member() {
        let lumen = hpa_test_lumen("search", "acme", 1, 1);
        let control = FakeHpaControl::with("acme", "search", render::hpa_labels(&lumen));

        prune_stale_hpa(&control, &lumen).await;

        assert_eq!(
            control.deletes.lock().unwrap().as_slice(),
            &[("acme".to_string(), "search".to_string())]
        );
        assert!(control
            .objects
            .lock()
            .unwrap()
            .get(&("acme".to_string(), "search".to_string()))
            .is_none());
    }

    #[tokio::test]
    async fn prune_stale_hpa_leaves_missing_hpa_as_noop() {
        let lumen = hpa_test_lumen("search", "acme", 3, 1);
        let control = FakeHpaControl::default();

        prune_stale_hpa(&control, &lumen).await;

        assert!(control.deletes.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn prune_stale_hpa_leaves_unrelated_hpa_name_untouched() {
        let lumen = hpa_test_lumen("search", "acme", 3, 1);
        // A user's own, differently-named HPA lives in the same namespace —
        // the handoff loop only ever looks up the CR's own name, so it must
        // never be inspected or deleted.
        let control = FakeHpaControl::with("acme", "my-other-hpa", render::hpa_labels(&lumen));

        prune_stale_hpa(&control, &lumen).await;

        assert!(control.deletes.lock().unwrap().is_empty());
        assert!(control
            .objects
            .lock()
            .unwrap()
            .contains_key(&("acme".to_string(), "my-other-hpa".to_string())));
    }

    #[tokio::test]
    async fn prune_stale_hpa_leaves_foreign_labeled_hpa_at_same_name_untouched() {
        let lumen = hpa_test_lumen("search", "acme", 3, 1);
        // Same namespace/name as the CR would render, but labels that don't
        // match lumen's stamp (e.g. a different `managed-by`) — R2's scope
        // guard, not just a name check.
        let mut foreign_labels = render::hpa_labels(&lumen);
        foreign_labels.insert(
            "app.kubernetes.io/managed-by".to_string(),
            "some-other-operator".to_string(),
        );
        let control = FakeHpaControl::with("acme", "search", foreign_labels);

        prune_stale_hpa(&control, &lumen).await;

        assert!(control.deletes.lock().unwrap().is_empty());
        assert!(control
            .objects
            .lock()
            .unwrap()
            .contains_key(&("acme".to_string(), "search".to_string())));
    }

    // ---- auth-delegator ClusterRoleBinding (#2876) -------------------------

    /// In-memory [`AuthDelegatorControl`]: a `name -> labels` map of the
    /// bindings the cluster currently holds, plus switches to make either the
    /// apply or the list fail the way a 403 or an apiserver outage would.
    #[derive(Default)]
    struct FakeAuthDelegatorControl {
        objects: Mutex<BTreeMap<String, BTreeMap<String, String>>>,
        applied: Mutex<Vec<serde_json::Value>>,
        deletes: Mutex<Vec<String>>,
        apply_fails: bool,
        list_fails: bool,
    }

    impl FakeAuthDelegatorControl {
        fn with(bindings: &[(&str, BTreeMap<String, String>)]) -> Self {
            let control = Self::default();
            for (name, labels) in bindings {
                control
                    .objects
                    .lock()
                    .unwrap()
                    .insert((*name).to_string(), labels.clone());
            }
            control
        }
    }

    #[async_trait::async_trait]
    impl AuthDelegatorControl for FakeAuthDelegatorControl {
        async fn apply_binding(&self, binding: &serde_json::Value) -> anyhow::Result<()> {
            if self.apply_fails {
                anyhow::bail!("clusterrolebindings.rbac.authorization.k8s.io is forbidden");
            }
            self.applied.lock().unwrap().push(binding.clone());
            let name = binding["metadata"]["name"].as_str().unwrap().to_string();
            let labels = serde_json::from_value(binding["metadata"]["labels"].clone())?;
            self.objects.lock().unwrap().insert(name, labels);
            Ok(())
        }

        async fn managed_bindings(&self) -> anyhow::Result<Vec<(String, BTreeMap<String, String>)>> {
            if self.list_fails {
                anyhow::bail!("the server was unable to return a response");
            }
            Ok(self
                .objects
                .lock()
                .unwrap()
                .iter()
                .map(|(name, labels)| (name.clone(), labels.clone()))
                .collect())
        }

        async fn delete_binding(&self, name: &str) -> anyhow::Result<()> {
            self.objects.lock().unwrap().remove(name);
            self.deletes.lock().unwrap().push(name.to_string());
            Ok(())
        }
    }

    #[tokio::test]
    async fn apply_auth_delegator_binding_applies_the_instance_binding() {
        let lumen = hpa_test_lumen("search", "acme", 1, 1);
        let control = FakeAuthDelegatorControl::default();

        assert_eq!(apply_auth_delegator_binding(&control, &lumen).await, None);

        let applied = control.applied.lock().unwrap();
        assert_eq!(applied.len(), 1);
        assert_eq!(applied[0]["kind"], "ClusterRoleBinding");
        assert_eq!(applied[0]["roleRef"]["name"], "system:auth-delegator");
        assert_eq!(
            applied[0]["subjects"],
            serde_json::json!([{
                "kind": "ServiceAccount",
                "name": "search",
                "namespace": "acme",
            }]),
            "exactly one subject: this instance's own serving ServiceAccount"
        );
    }

    /// A second reconcile of an unchanged CR must not produce a second object:
    /// the apply is server-side and keyed by a name derived from the CR, so it
    /// converges on the same binding (#2876 AC3).
    #[tokio::test]
    async fn apply_auth_delegator_binding_is_idempotent_across_reconciles() {
        let lumen = hpa_test_lumen("search", "acme", 1, 1);
        let control = FakeAuthDelegatorControl::default();

        apply_auth_delegator_binding(&control, &lumen).await;
        apply_auth_delegator_binding(&control, &lumen).await;

        assert_eq!(
            control.objects.lock().unwrap().len(),
            1,
            "the binding name is a function of the CR, so re-applying overwrites rather than adds"
        );
    }

    /// AC4: a refused write does not vanish into a log line, and does not fail
    /// the reconcile before the status is written either — it comes back as the
    /// message the CR will publish.
    #[tokio::test]
    async fn apply_auth_delegator_binding_reports_a_refused_write() {
        let lumen = hpa_test_lumen("search", "acme", 1, 1);
        let control = FakeAuthDelegatorControl {
            apply_fails: true,
            ..Default::default()
        };

        let error = apply_auth_delegator_binding(&control, &lumen)
            .await
            .expect("a refused apply must produce a message");

        assert!(
            error.contains("ClusterRoleBinding") && error.contains("system:auth-delegator"),
            "the message must name the operation that was refused, got: {error}"
        );
        assert!(
            error.contains(&render::auth_delegator_binding_name(&lumen)),
            "the message must name the object, got: {error}"
        );
    }

    /// AC4, the other half: that message reaches `status.conditions` as a
    /// not-Ready CR. Reporting Ready while the serving pods cannot authenticate
    /// anyone is the failure this exists to prevent.
    #[test]
    fn a_refused_binding_makes_the_cr_not_ready_and_says_why() {
        // Every replica is up: without the refused binding this CR would report
        // `Ready=True`, which is exactly the lie AC4 forbids.
        let lumen = hpa_test_lumen("search", "acme-cond-delegation", 1, 2);
        let context = serde_json::json!({
            AUTH_DELEGATION_CONTEXT_KEY: "apply ClusterRoleBinding lumen.acme-cond-delegation.search.auth-delegator (system:auth-delegator): forbidden",
        });

        let facts = lumen.conditions(&ready_facts("search", 2), &context);

        let ready = condition(&facts, "Ready");
        assert_eq!(ready.status, ConditionStatus::False, "got: {facts:?}");
        assert_eq!(ready.reason, "AuthDelegationNotGranted");
        assert!(
            ready.message.contains("ClusterRoleBinding"),
            "the Ready message must name the refused operation, got: {facts:?}"
        );
        let delegation = condition(&facts, "AuthDelegationReady");
        assert_eq!(delegation.status, ConditionStatus::False);
        assert_eq!(delegation.reason, "ClusterRoleBindingFailed");
    }

    #[test]
    fn a_granted_binding_leaves_readiness_to_the_workload() {
        let lumen = hpa_test_lumen("search", "acme-cond-delegation-ok", 1, 2);

        let facts = lumen.conditions(&ready_facts("search", 2), &serde_json::json!({}));

        let delegation = condition(&facts, "AuthDelegationReady");
        assert_eq!(delegation.status, ConditionStatus::True);
        assert_eq!(delegation.reason, "AuthDelegatorBound");
        assert_eq!(
            condition(&facts, "Ready").status,
            ConditionStatus::True,
            "got: {facts:?}"
        );
    }

    /// AC3: the binding a deleted CR left behind is the whole reason this loop
    /// exists — nothing else can reach it, since a cluster-scoped object cannot
    /// name a namespaced owner.
    #[tokio::test]
    async fn sweep_deletes_a_binding_whose_instance_is_gone() {
        let gone = hpa_test_lumen("retired", "acme", 1, 1);
        let live = hpa_test_lumen("search", "acme", 1, 1);
        let control = FakeAuthDelegatorControl::with(&[
            (
                &render::auth_delegator_binding_name(&gone),
                render::auth_delegator_labels(&gone),
            ),
            (
                &render::auth_delegator_binding_name(&live),
                render::auth_delegator_labels(&live),
            ),
        ]);

        sweep_stale_auth_delegator_bindings(&control, &[live.clone()]).await;

        assert_eq!(
            control.deletes.lock().unwrap().as_slice(),
            &[render::auth_delegator_binding_name(&gone)]
        );
        assert!(control
            .objects
            .lock()
            .unwrap()
            .contains_key(&render::auth_delegator_binding_name(&live)));
    }

    /// A rename is a delete plus a create as far as the CR is concerned, and
    /// the old name is not derivable from the new object — only from the
    /// cluster-wide diff this sweep computes.
    #[tokio::test]
    async fn sweep_deletes_the_binding_left_by_a_renamed_instance() {
        let old = hpa_test_lumen("old-name", "acme", 1, 1);
        let new = hpa_test_lumen("new-name", "acme", 1, 1);
        let control = FakeAuthDelegatorControl::with(&[(
            &render::auth_delegator_binding_name(&old),
            render::auth_delegator_labels(&old),
        )]);

        sweep_stale_auth_delegator_bindings(&control, &[new]).await;

        assert_eq!(
            control.deletes.lock().unwrap().as_slice(),
            &[render::auth_delegator_binding_name(&old)]
        );
    }

    /// Full label-set equality, not name equality, is what proves authorship —
    /// the same guard `prune_stale_hpa` uses, and this one deletes an RBAC
    /// object.
    #[tokio::test]
    async fn sweep_leaves_a_foreign_labeled_binding_at_a_live_name_untouched() {
        let lumen = hpa_test_lumen("search", "acme", 1, 1);
        let mut foreign = render::auth_delegator_labels(&lumen);
        foreign.insert(
            "app.kubernetes.io/managed-by".to_string(),
            "some-other-operator".to_string(),
        );
        let control = FakeAuthDelegatorControl::with(&[(
            &render::auth_delegator_binding_name(&lumen),
            foreign,
        )]);

        sweep_stale_auth_delegator_bindings(&control, &[lumen]).await;

        assert!(control.deletes.lock().unwrap().is_empty());
    }

    /// An unreadable list is not an empty cluster. Treating the two alike would
    /// make one apiserver blip revoke delegated review for every Lumen at once.
    #[tokio::test]
    async fn sweep_deletes_nothing_when_it_cannot_list() {
        let orphan = hpa_test_lumen("retired", "acme", 1, 1);
        let mut control = FakeAuthDelegatorControl::with(&[(
            &render::auth_delegator_binding_name(&orphan),
            render::auth_delegator_labels(&orphan),
        )]);
        control.list_fails = true;

        sweep_stale_auth_delegator_bindings(&control, &[]).await;

        assert!(control.deletes.lock().unwrap().is_empty());
    }

    // ---- peer TLS Secret check (#2890 R4) ----------------------------------

    /// A replicated instance naming `secret`, in namespace `acme`.
    fn peer_test_lumen(secret: Option<&str>) -> Lumen {
        let mut lumen = hpa_test_lumen("search", "acme", 1, 3);
        lumen.spec.peer_tls_secret = secret.map(str::to_string);
        lumen
    }

    #[test]
    fn a_single_replica_instance_is_never_asked_for_peer_material() {
        let mut lumen = peer_test_lumen(None);
        lumen.spec.replicas_per_shard = 1;
        assert_eq!(check_peer_identity(&lumen), None);
    }

    #[test]
    fn a_replicated_instance_with_no_secret_named_says_which_keys_it_needs() {
        let lumen = peer_test_lumen(None);

        let error = check_peer_identity(&lumen)
            .expect("a replicated instance owes peer identity");

        assert!(error.contains("spec.peerTlsSecret"), "got: {error}");
        for key in render::PEER_TLS_KEYS {
            assert!(error.contains(key), "the message must name {key}: {error}");
        }
    }

    #[test]
    fn complete_peer_material_is_no_finding_at_all() {
        let lumen = peer_test_lumen(Some("search-peer-tls"));
        assert_eq!(check_peer_identity(&lumen), None);
    }

    #[test]
    fn check_peer_identity_message_pins_spec_field_required_keys_and_replica_count() {
        let lumen = peer_test_lumen(None);
        let error = check_peer_identity(&lumen).expect("replicated CR without peerTlsSecret must return error");
        assert!(error.contains("spec.peerTlsSecret"), "message must name spec field: {error}");
        assert!(
            error.contains(&format!("replicasPerShard={}", lumen.spec.replicas_per_shard)),
            "message must state replicasPerShard value: {error}"
        );
        for key in render::PEER_TLS_KEYS {
            assert!(error.contains(key), "message must name required key {key}: {error}");
        }
        assert!(
            error.contains("replicated Raft traffic has no plaintext fallback"),
            "message must state tail rationale: {error}"
        );

        // Single-replica instance owes no peer identity
        let single = hpa_test_lumen("search", "acme", 1, 1);
        assert_eq!(check_peer_identity(&single), None);

        // Replicated instance naming spec.peerTlsSecret returns None
        let configured = peer_test_lumen(Some("search-peer-tls"));
        assert_eq!(check_peer_identity(&configured), None);
    }
}
// CODEGEN-END
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/lumen/src/operator/reconcile.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Canonical lossless source unit for the Lumen operator reconciliation loop.
      Runtime behavior is regenerated exactly from the authoritative
      rust-source-unit captured above.
```
