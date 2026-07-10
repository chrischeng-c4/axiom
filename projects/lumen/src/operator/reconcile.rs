// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reconcile-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! lumen's operator wiring onto the shared `libs/operator` controller.
//!
//! The reconcile loop + leader-election lease now live in `libs/operator`
//! (`operator::run` drives the watch + leader-gated apply over h2c-free kube;
//! `operator::lease` is the elector). lumen supplies only its `ManagedService`
//! impl — what to render, which workloads to poll for readiness, and the
//! `Lumen` status subresource to write.
//!
//! Live per-shard storage-usage measurement (#1319 R1): `ManagedService::
//! status_patch` is synchronous and does no I/O by contract (shared with
//! keep/relay/loom via `libs/operator`), so it cannot itself poll pod
//! `/metrics` endpoints. Instead `run()` spawns a lumen-local background
//! loop (`spawn_shard_usage_loop`) that periodically scrapes every storage
//! pod's `lumen_storage_bytes` gauge over its headless-Service DNS name and
//! writes the per-shard max into an in-process cache; `status_patch` reads
//! that cache synchronously (best-effort — an empty/missing cache falls back
//! to the policy-only [`crate::operator::crd::LumenSpec::reshard_status`]).
//! This keeps the shared `libs/operator` trait untouched.
//!
//! This loop only *reports* a crossed `prepareAtPercent` / `urgentAtPercent`
//! threshold in `status.reshard`; a second, independently leader-gated
//! background loop spawned alongside it — [`crate::operator::reshard_driver::
//! spawn_reshard_driver_loop`] (#1319 R2, #1381) — is what actually drives
//! `workflow.phase` and moves data once a threshold is crossed.
//!
//! ## HPA topology-transition handoff (#1385)
//!
//! [`render::render`] stops emitting a HorizontalPodAutoscaler once the CR's
//! shape no longer wants one ([`render::wants_hpa`] — today, `shardCount >
//! 1`), but `libs/operator`'s shared reconcile contract (`libs/operator::
//! service`) deliberately does not prune children across a render-shape
//! change — that handoff is left to the service. A third independently
//! leader-gated background loop, [`spawn_hpa_handoff_loop`], is lumen's side
//! of that handoff: every tick it lists every `Lumen` CR and, for any whose
//! current shape no longer wants an HPA, deletes the previously-rendered one
//! if it is still there — scoped and idempotent (R2: only an object whose
//! live name *and* labels match what [`render::hpa_labels`] would have
//! stamped; a missing HPA, or one that doesn't look lumen-rendered, is a
//! no-op, not an error). Without this, the stale single-member HPA (clamped
//! to `minReplicas`/`maxReplicas` == 1 by #1317) keeps scaling the serving
//! StatefulSet back down to one ready replica, permanently starving
//! [`crate::operator::reshard_driver`]'s `PrepareSplit` readiness gate
//! (`readyReplicas >= targetShardCount`) — observed live in #1384's kind
//! proof, unblocked there only by a manual `kubectl delete hpa`.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use kube::api::{Api, ApiResource, DeleteParams, DynamicObject};
use kube::{Client, ResourceExt};
use operator::{ManagedService, ReadinessTarget, ReadyFacts};
use serde_json::json;

use crate::operator::crd::Lumen;
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
/// distinct from both `libs/operator`'s own `S::MANAGER`-named apply-loop
/// Lease and `reshard_driver::DRIVER_LEASE_NAME`, so none of the three
/// independently leader-gated loops contend on one Lease object (mirrors the
/// same duplicated `identity`/`lease_namespace` resolution
/// `reshard_driver::spawn_reshard_driver_loop` already uses for the same
/// reason).
const HPA_HANDOFF_LEASE_NAME: &str = "lumen-hpa-handoff";

/// `"<namespace>/<name>" -> "shard_index -> observed bytes"`, refreshed by
/// [`spawn_shard_usage_loop`] and read by [`status_patch`].
type ShardUsageCache = Mutex<BTreeMap<String, BTreeMap<u32, u64>>>;

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
/// present or its value does not parse.
fn parse_metric(body: &str, metric: &str) -> Option<u64> {
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
/// Ordinal-to-shard mapping matches `libs/raft-host`'s pod placement:
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
/// consults best-effort; the leader-gated `libs/operator` apply loop is
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
                        let mut cache = shard_usage_cache()
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        cache.insert(key, usage);
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
/// `libs/operator::render::horizontal_pod_autoscaler` renders
/// (`autoscaling/v2`) and what `libs/operator::controller`'s generic apply
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

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reconcile-rs.md#source
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
                "HPA handoff: deleted the single-member HPA now that this CR's \
                 rendered shape no longer includes one (shardCount > 1) — the \
                 stale HPA would otherwise keep clamping the serving \
                 StatefulSet back to 1 ready replica and starve the reshard \
                 driver's PrepareSplit readiness gate (#1385)"
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
    // Mirrors `libs/operator::controller`'s own `identity`/`lease_namespace`
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

/// lumen's contribution to the shared operator.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reconcile-rs.md#source
impl ManagedService for Lumen {
    /// Server-side-apply field manager + leader-election Lease name.
    const MANAGER: &'static str = "lumen-operator";

    fn render(&self) -> Vec<serde_json::Value> {
        render::render(self)
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
        let name = self.name_any();
        let serving_ready = ready.ready.get(&name).copied().unwrap_or(0) as i32;
        let desired = self.spec.storage_pod_count();
        let usage = shard_usage_cache()
            .lock()
            .ok()
            .and_then(|cache| cache.get(&cache_key(self)).cloned());
        let reshard = match usage {
            Some(usage) if !usage.is_empty() => self.spec.reshard_status_with_usage(&usage),
            _ => self.spec.reshard_status(),
        };
        let phase = if serving_ready >= desired {
            "Ready"
        } else if serving_ready > 0 {
            "Reconciling"
        } else {
            "Pending"
        };
        json!({ "status": {
            "phase": phase,
            "observedGeneration": self.metadata.generation.unwrap_or(0),
            "servingReadyReplicas": serving_ready,
            "desiredReplicas": desired,
            "shardCount": self.spec.shard_count,
            "reshard": reshard,
            "message": format!("{serving_ready}/{desired} serving pods ready"),
        }})
    }
}

/// `lumen k8s operator run` — run the reconcile controller on the shared
/// `libs/operator` host (leader-gated; safe at `replicas > 1`), alongside
/// the live shard-usage measurement loop (#1319 R1; every replica runs it,
/// not just the leader — see [`spawn_shard_usage_loop`]), the autonomous
/// reshard phase driver (#1319 R2, #1381; independently leader-gated — see
/// [`crate::operator::reshard_driver::spawn_reshard_driver_loop`]), and the
/// HPA topology-transition handoff loop (#1385; independently leader-gated —
/// see [`spawn_hpa_handoff_loop`]).
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reconcile-rs.md#source
pub async fn run() -> anyhow::Result<()> {
    match Client::try_default().await {
        Ok(client) => {
            spawn_shard_usage_loop(client.clone());
            crate::operator::reshard_driver::spawn_reshard_driver_loop(client.clone());
            spawn_hpa_handoff_loop(client);
        }
        Err(err) => {
            tracing::warn!(
                error = %err,
                "reshard live-usage measurement + phase-driver + HPA-handoff loops disabled: could not build a kube client"
            );
        }
    }
    operator::run::<Lumen>().await
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
            shard_count: 2,
            shard_map: ShardMapSpec::default(),
            replicas_per_shard: 3,
            voter_count: 3,
            log_format: Default::default(),
            log_level: None,
            auth: Default::default(),
            tokens_secret: None,
            tokens_secret_provider_class: None,
            serving: ServingSpec::default(),
            reshard_policy: Default::default(),
            observability: false,
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
            shard_count,
            shard_map: ShardMapSpec::default(),
            replicas_per_shard,
            voter_count: replicas_per_shard,
            log_format: Default::default(),
            log_level: None,
            auth: Default::default(),
            tokens_secret: None,
            tokens_secret_provider_class: None,
            serving: ServingSpec::default(),
            reshard_policy: Default::default(),
            observability: false,
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
    async fn prune_stale_hpa_retains_hpa_on_single_member() {
        let lumen = hpa_test_lumen("search", "acme", 1, 1);
        let control = FakeHpaControl::with("acme", "search", render::hpa_labels(&lumen));

        prune_stale_hpa(&control, &lumen).await;

        assert!(control.deletes.lock().unwrap().is_empty());
        assert!(control
            .objects
            .lock()
            .unwrap()
            .contains_key(&("acme".to_string(), "search".to_string())));
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
}
// CODEGEN-END
