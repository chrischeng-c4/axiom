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

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

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
/// not just the leader — see [`spawn_shard_usage_loop`]) and the autonomous
/// reshard phase driver (#1319 R2, #1381; independently leader-gated — see
/// [`crate::operator::reshard_driver::spawn_reshard_driver_loop`]).
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reconcile-rs.md#source
pub async fn run() -> anyhow::Result<()> {
    match Client::try_default().await {
        Ok(client) => {
            spawn_shard_usage_loop(client.clone());
            crate::operator::reshard_driver::spawn_reshard_driver_loop(client);
        }
        Err(err) => {
            tracing::warn!(
                error = %err,
                "reshard live-usage measurement + phase-driver loops disabled: could not build a kube client"
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
}
// CODEGEN-END
