---
id: projects-lumen-src-metrics-rs
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: "This source unit is captured as a per-file rust-source-unit during lumen td_ast standardization."
fill_sections: [overview, source, changes]
---

# Standardized apps/lumen/src/metrics.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/lumen/src/metrics.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Metrics` | apps/lumen/src/metrics.rs | struct | pub | 33 |  |
| `incr_auth_registry_reload_failure` | apps/lumen/src/metrics.rs | function | pub | 178 | incr_auth_registry_reload_failure(&self) |
| `incr_collection_created` | apps/lumen/src/metrics.rs | function | pub | 136 | incr_collection_created(&self, fields: u64) |
| `incr_duplicates` | apps/lumen/src/metrics.rs | function | pub | 127 | incr_duplicates(&self) |
| `incr_index` | apps/lumen/src/metrics.rs | function | pub | 116 | incr_index(&self, items: u64, bytes: u64) |
| `incr_replace_skipped` | apps/lumen/src/metrics.rs | function | pub | 132 | incr_replace_skipped(&self, fields: u64) |
| `incr_scatter_map_version_mismatch` | apps/lumen/src/metrics.rs | function | pub | 152 | incr_scatter_map_version_mismatch(&self) |
| `new` | apps/lumen/src/metrics.rs | function | pub | 107 | new() -> Self |
| `observe_search` | apps/lumen/src/metrics.rs | function | pub | 121 | observe_search(&self, latency_ms: u64) |
| `render` | apps/lumen/src/metrics.rs | function | pub | 191 | render(&self) -> String |
| `set_raft_leader_known` | apps/lumen/src/metrics.rs | function | pub | 162 | set_raft_leader_known(&self, shard: u32, known: bool) |
| `set_reshard_fence_active` | apps/lumen/src/metrics.rs | function | pub | 170 | set_reshard_fence_active(&self, active: bool) |
| `set_shard_map_version` | apps/lumen/src/metrics.rs | function | pub | 146 | set_shard_map_version(&self, version: u64) |
| `set_storage_bytes` | apps/lumen/src/metrics.rs | function | pub | 141 | set_storage_bytes(&self, bytes: u64) |
| `touch_auth_registry_reload_success` | apps/lumen/src/metrics.rs | function | pub | 184 | touch_auth_registry_reload_success(&self) |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// SPEC-MANAGED: apps/lumen/tech-design/semantic/source/apps-lumen-src-metrics-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Lightweight in-process Prometheus exposition.
//!
//! v1 keeps the metric surface narrow and dep-free: a handful of
//! `AtomicU64` counters/gauges + a single `render` that emits the
//! Prometheus text-format. When request volume grows past what
//! lock-free counters can serve, swap in `prometheus`/`metrics` crates
//! without changing the wire format the scraper sees.
//!
//! The counter/gauge primitives and the Prometheus text-format encoder
//! are generic across every service with a `/metrics` scrape endpoint
//! and live in `libs/metrics-prometheus` (#974); this module wires lumen's
//! metric names, HELP text, and `# TYPE` kinds onto them. `Counter`/
//! `Gauge` deref to the underlying `AtomicU64`, so this module's pub API
//! — field names, method names, and `render()`'s byte output — is
//! unchanged for callers, including the `otel` feature's direct
//! `field.load(Ordering::Relaxed)` reads in `src/bin/lumen.rs`.

use metrics_prometheus::{Counter, Gauge, Label, LabeledSample, Sample, SampleGroup};

/// #2475: sentinel `raft_shard` value meaning "never touched by the raft
/// election-state poller" — see [`Metrics::raft_shard`]. Out of range for
/// any real shard index, so it stays distinguishable from every real shard.
const NOT_RAFT: u64 = u64::MAX;

/// All metrics carry the `{collection, shard, partition}` label set per
/// the README §5 contract. v1 in-memory single-shard reports
/// `shard="0", partition="0"` as constants; future LSM/Raft tiers will
/// vary `partition` and `shard` respectively.
#[derive(Debug, Default)]
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-metrics-rs.md#source
pub struct Metrics {
    pub index_writes_total: Counter,
    pub index_bytes_total: Counter,
    pub search_requests_total: Counter,
    pub search_latency_ms_sum: Counter,
    pub search_latency_ms_count: Counter,
    pub duplicates_requests_total: Counter,
    pub collections_created_total: Counter,
    pub schema_fields_total: Counter,
    pub storage_bytes: Gauge,
    pub posting_cache_hits_total: Counter,
    pub posting_cache_misses_total: Counter,
    /// #1293: `docs:replace` server-side no-op suppression — fields whose
    /// incoming value matched the currently indexed state and were skipped
    /// (no posting-list rewrite, no HNSW tombstone/reinsert). Distinct from
    /// `index_writes_total`, which only ever counts fields actually written.
    pub replace_fields_skipped_total: Counter,
    /// #1467 R5: this pod's live routed shard-map version, `0` for every
    /// non-routed deployment shape (standalone, primary/replica, no
    /// `operator` feature) that never sets it. The reshard driver's
    /// `advance_convergence` scrapes this over `/metrics` — the same
    /// admin-reachable surface its usage loop already polls — to require
    /// every serving pod to actually report the new map version, not just
    /// that its StatefulSet rollout finished (rollout completion alone does
    /// not prove the ConfigMap write each pod reads its map from has
    /// propagated to every pod).
    pub shard_map_version: Gauge,
    /// #1467 R6: count of scatter (routing-key-less) search sub-requests
    /// where the responding pod's live shard-map version differed from the
    /// scattering pod's own declared version. Signal for a mixed-map
    /// rolling-restart window landing a scatter search mid-flight;
    /// non-fatal by design (see `routing_remote.rs`'s scatter exemption
    /// doc — availability over completeness). `0` outside routed
    /// deployments.
    pub scatter_map_version_mismatches_total: Counter,
    /// #2475: this pod's raft shard index, or the `NOT_RAFT` sentinel until
    /// its raft election-state poller (`spawn_cluster_state_poller` in
    /// `src/bin/lumen.rs`) has ticked at least once. `render()` reads the
    /// sentinel to omit `lumen_raft_leader_known` entirely for
    /// standalone/non-raft deployments — publishing a permanently-`0`
    /// series there would be indistinguishable from a genuinely stuck
    /// leaderless shard to `render::prometheus_rule`'s
    /// `LumenRaftLeaderAbsent` alert.
    pub raft_shard: Gauge,
    /// #2475: `1` while this pod's raft election-state poll believes its
    /// shard currently has an elected leader (itself or a peer), `0` while
    /// it does not. Meaningful only once `raft_shard` has left the
    /// `NOT_RAFT` sentinel; see [`Metrics::set_raft_leader_known`].
    pub raft_leader_known: Gauge,
    /// #2475: `1` while this pod believes a reshard-driver write fence
    /// (`POST /admin/reshard:fence`, see `crate::api::WriteFence`) is
    /// currently armed on it, `0` once cleared or never armed.
    pub reshard_fence_active: Gauge,
    /// #2475: unix-epoch seconds the currently (or most recently) armed
    /// write fence was armed at. Pairs with `reshard_fence_active` so
    /// `LumenReshardWorkflowStalled` can tell a long-armed fence (the
    /// driver's fenced final `CatchingUp` pass never came back to clear it)
    /// from one still mid-pass.
    pub reshard_fence_armed_unixtime: Gauge,
    /// #2475: total failed hot-reloads of the bearer-token registry file
    /// (`LUMEN_TOKEN_REGISTRY_FILE`, watched by
    /// `service_auth::spawn_registry_file_watcher`) — read/parse/validation
    /// failures recorded via `crate::auth::LumenAuthEventSink`. The
    /// verifier always keeps serving the last known-good registry on
    /// failure, so this counts silent staleness risk, not live outage.
    pub auth_registry_reload_failures_total: Counter,
    /// #2475: unix-epoch seconds of the last successful bearer-token
    /// registry hot-reload, pairing with the failure counter above as a
    /// cheap staleness signal.
    pub auth_registry_reload_success_unixtime: Gauge,
}

/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-metrics-rs.md#source
impl Metrics {
    pub fn new() -> Self {
        let metrics = Self::default();
        // #2475: seed the "never touched by the raft poller" sentinel;
        // every other field's `Default` (0) is already the right initial
        // value.
        metrics.raft_shard.set(NOT_RAFT);
        metrics
    }

    pub fn incr_index(&self, items: u64, bytes: u64) {
        self.index_writes_total.add(items);
        self.index_bytes_total.add(bytes);
    }

    pub fn observe_search(&self, latency_ms: u64) {
        self.search_requests_total.incr();
        self.search_latency_ms_sum.add(latency_ms);
        self.search_latency_ms_count.incr();
    }

    pub fn incr_duplicates(&self) {
        self.duplicates_requests_total.incr();
    }

    /// #1293: record `n` `docs:replace` fields skipped as unchanged no-ops.
    pub fn incr_replace_skipped(&self, fields: u64) {
        self.replace_fields_skipped_total.add(fields);
    }

    pub fn incr_collection_created(&self, fields: u64) {
        self.collections_created_total.incr();
        self.schema_fields_total.add(fields);
    }

    pub fn set_storage_bytes(&self, bytes: u64) {
        self.storage_bytes.set(bytes);
    }

    /// #1467 R5: record this pod's live routed shard-map version.
    pub fn set_shard_map_version(&self, version: u64) {
        self.shard_map_version.set(version);
    }

    /// #1467 R6: record one scatter sub-response whose responding pod's map
    /// version differed from the scattering pod's own declared version.
    pub fn incr_scatter_map_version_mismatch(&self) {
        self.scatter_map_version_mismatches_total.incr();
    }

    /// #2475: record this pod's shard index + whether its raft
    /// election-state poll currently believes that shard has an elected
    /// leader. Called every `spawn_cluster_state_poller` tick in raft mode
    /// only; standalone/non-raft pods never call this, so `raft_shard`
    /// stays at the `NOT_RAFT` sentinel forever and `render()` omits
    /// `lumen_raft_leader_known` for them.
    pub fn set_raft_leader_known(&self, shard: u32, known: bool) {
        self.raft_shard.set(shard as u64);
        self.raft_leader_known.set(known as u64);
    }

    /// #2475: arm/clear the reshard write-fence-active signal, called from
    /// `POST /admin/reshard:fence`'s handler. Arming also stamps
    /// `reshard_fence_armed_unixtime` to "now".
    pub fn set_reshard_fence_active(&self, active: bool) {
        self.reshard_fence_active.set(active as u64);
        if active {
            self.reshard_fence_armed_unixtime.set(unix_now_secs());
        }
    }

    /// #2475: record one failed bearer-token registry hot-reload.
    pub fn incr_auth_registry_reload_failure(&self) {
        self.auth_registry_reload_failures_total.incr();
    }

    /// #2475: record one successful bearer-token registry hot-reload's
    /// wall-clock time.
    pub fn touch_auth_registry_reload_success(&self) {
        self.auth_registry_reload_success_unixtime
            .set(unix_now_secs());
    }

    /// Prometheus text format (0.0.4 compatible). Always emits the same
    /// set of metric names so scrape configs are stable.
    pub fn render(&self) -> String {
        let samples = [
            Sample::new(
                "lumen_index_writes_total",
                "counter",
                "Total index items applied.",
                self.index_writes_total.get(),
            ),
            Sample::new(
                "lumen_index_bytes_total",
                "counter",
                "Total bytes written across all field indexes.",
                self.index_bytes_total.get(),
            ),
            Sample::new(
                "lumen_search_requests_total",
                "counter",
                "Total search requests served.",
                self.search_requests_total.get(),
            ),
            Sample::new(
                "lumen_search_latency_ms_sum",
                "counter",
                "Sum of search latencies in milliseconds.",
                self.search_latency_ms_sum.get(),
            ),
            Sample::new(
                "lumen_search_latency_ms_count",
                "counter",
                "Count of search latency observations.",
                self.search_latency_ms_count.get(),
            ),
            Sample::new(
                "lumen_duplicates_requests_total",
                "counter",
                "Total duplicate-detection requests.",
                self.duplicates_requests_total.get(),
            ),
            Sample::new(
                "lumen_collections_created_total",
                "counter",
                "Total collections created or extended.",
                self.collections_created_total.get(),
            ),
            Sample::new(
                "lumen_schema_fields_total",
                "counter",
                "Total field declarations registered.",
                self.schema_fields_total.get(),
            ),
            Sample::new(
                "lumen_storage_bytes",
                "gauge",
                "Approximate bytes held by all in-memory field indexes.",
                self.storage_bytes.get(),
            ),
            Sample::new(
                "lumen_posting_cache_hits_total",
                "counter",
                "Posting cache hit count (0 until LSM cache is wired).",
                self.posting_cache_hits_total.get(),
            ),
            Sample::new(
                "lumen_posting_cache_misses_total",
                "counter",
                "Posting cache miss count.",
                self.posting_cache_misses_total.get(),
            ),
            Sample::new(
                "lumen_replace_fields_skipped_total",
                "counter",
                "Total docs:replace fields skipped as unchanged no-ops.",
                self.replace_fields_skipped_total.get(),
            ),
            Sample::new(
                "lumen_shard_map_version",
                "gauge",
                "This pod's live routed shard-map version (0 outside routed deployments).",
                self.shard_map_version.get(),
            ),
            Sample::new(
                "lumen_scatter_map_version_mismatches_total",
                "counter",
                "Scatter search sub-responses whose responding pod's map version differed \
                 from the sender's.",
                self.scatter_map_version_mismatches_total.get(),
            ),
            Sample::new(
                "lumen_reshard_fence_active",
                "gauge",
                "1 while this pod believes a reshard-driver write fence is currently armed \
                 on it.",
                self.reshard_fence_active.get(),
            ),
            Sample::new(
                "lumen_reshard_fence_armed_unixtime",
                "gauge",
                "Unix time the currently (or most recently) armed reshard write fence was \
                 armed at.",
                self.reshard_fence_armed_unixtime.get(),
            ),
            Sample::new(
                "lumen_auth_registry_reload_failures_total",
                "counter",
                "Total failed bearer-token registry hot-reloads.",
                self.auth_registry_reload_failures_total.get(),
            ),
            Sample::new(
                "lumen_auth_registry_reload_success_unixtime",
                "gauge",
                "Unix time of the last successful bearer-token registry hot-reload.",
                self.auth_registry_reload_success_unixtime.get(),
            ),
        ];
        let mut out = metrics_prometheus::render(&samples);
        // #2475: `lumen_raft_leader_known` carries a `shard` label and is
        // omitted entirely (not just left at 0) for standalone/non-raft
        // pods — see `raft_shard`'s doc comment for why a permanent-0
        // series would be a false-positive risk for `LumenRaftLeaderAbsent`.
        let shard = self.raft_shard.get();
        if shard != NOT_RAFT {
            let shard_label = shard.to_string();
            let rows = [LabeledSample::new(
                vec![Label::new("shard", &shard_label)],
                self.raft_leader_known.get(),
            )];
            let groups = [SampleGroup::new(
                "lumen_raft_leader_known",
                "gauge",
                "1 while this pod's raft election-state poll believes its shard currently \
                 has an elected leader, 0 otherwise. Omitted for standalone/non-raft \
                 deployments.",
                &rows,
            )];
            out.push_str(&metrics_prometheus::render_labeled(&groups));
        }
        out
    }
}

/// #2475: current unix-epoch seconds, saturating to `0` on a pre-epoch
/// clock rather than panicking (metrics rendering must never fail).
fn unix_now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_emits_every_metric() {
        let m = Metrics::new();
        m.incr_index(3, 100);
        m.observe_search(7);
        m.set_raft_leader_known(2, true);
        m.set_reshard_fence_active(true);
        m.incr_auth_registry_reload_failure();
        m.touch_auth_registry_reload_success();
        let out = m.render();
        for name in [
            "lumen_index_writes_total",
            "lumen_search_latency_ms_sum",
            "lumen_storage_bytes",
            "lumen_posting_cache_hits_total",
            "lumen_replace_fields_skipped_total",
            "lumen_shard_map_version",
            "lumen_scatter_map_version_mismatches_total",
            "lumen_reshard_fence_active",
            "lumen_reshard_fence_armed_unixtime",
            "lumen_auth_registry_reload_failures_total",
            "lumen_auth_registry_reload_success_unixtime",
            "lumen_raft_leader_known",
        ] {
            assert!(out.contains(name), "expected {name} in:\n{out}");
        }
        assert!(
            out.contains("lumen_raft_leader_known{shard=\"2\"} 1"),
            "expected labeled raft series in:\n{out}"
        );
    }

    /// #2475: a pod whose raft election-state poller never ticked (every
    /// standalone/non-raft deployment) must not publish
    /// `lumen_raft_leader_known` at all — a permanent `0` there would be
    /// indistinguishable from a genuinely stuck leaderless shard to
    /// `LumenRaftLeaderAbsent`.
    #[test]
    fn render_omits_raft_leader_known_when_never_set() {
        let m = Metrics::new();
        let out = m.render();
        assert!(
            !out.contains("lumen_raft_leader_known"),
            "unexpected raft series in:\n{out}"
        );
    }

    /// Byte-identical golden-render check (#974): fixed inputs must
    /// reproduce the exact pre-refactor `render()` capture, byte for
    /// byte — this is the AC2 contract the observability EC claim
    /// (`lumen_claim_observability_prometheus_metrics`) depends on.
    #[test]
    fn render_is_byte_identical_to_pre_refactor_capture() {
        let m = Metrics::new();
        m.incr_index(3, 100);
        m.observe_search(7);
        m.observe_search(9);
        m.incr_duplicates();
        m.incr_collection_created(4);
        m.set_storage_bytes(2048);
        m.posting_cache_hits_total.add(5);
        m.posting_cache_misses_total.add(2);
        m.incr_replace_skipped(6);
        m.set_shard_map_version(3);
        m.incr_scatter_map_version_mismatch();
        // #2475: reach past the wall-clock-stamping setters and set the raw
        // gauges directly so this golden capture stays deterministic.
        m.reshard_fence_active.set(1);
        m.reshard_fence_armed_unixtime.set(1_700_000_000);
        m.incr_auth_registry_reload_failure();
        m.incr_auth_registry_reload_failure();
        m.auth_registry_reload_success_unixtime.set(1_700_000_001);
        m.set_raft_leader_known(2, true);
        let out = m.render();
        let golden = "# HELP lumen_index_writes_total Total index items applied.\n\
# TYPE lumen_index_writes_total counter\n\
lumen_index_writes_total 3\n\
# HELP lumen_index_bytes_total Total bytes written across all field indexes.\n\
# TYPE lumen_index_bytes_total counter\n\
lumen_index_bytes_total 100\n\
# HELP lumen_search_requests_total Total search requests served.\n\
# TYPE lumen_search_requests_total counter\n\
lumen_search_requests_total 2\n\
# HELP lumen_search_latency_ms_sum Sum of search latencies in milliseconds.\n\
# TYPE lumen_search_latency_ms_sum counter\n\
lumen_search_latency_ms_sum 16\n\
# HELP lumen_search_latency_ms_count Count of search latency observations.\n\
# TYPE lumen_search_latency_ms_count counter\n\
lumen_search_latency_ms_count 2\n\
# HELP lumen_duplicates_requests_total Total duplicate-detection requests.\n\
# TYPE lumen_duplicates_requests_total counter\n\
lumen_duplicates_requests_total 1\n\
# HELP lumen_collections_created_total Total collections created or extended.\n\
# TYPE lumen_collections_created_total counter\n\
lumen_collections_created_total 1\n\
# HELP lumen_schema_fields_total Total field declarations registered.\n\
# TYPE lumen_schema_fields_total counter\n\
lumen_schema_fields_total 4\n\
# HELP lumen_storage_bytes Approximate bytes held by all in-memory field indexes.\n\
# TYPE lumen_storage_bytes gauge\n\
lumen_storage_bytes 2048\n\
# HELP lumen_posting_cache_hits_total Posting cache hit count (0 until LSM cache is wired).\n\
# TYPE lumen_posting_cache_hits_total counter\n\
lumen_posting_cache_hits_total 5\n\
# HELP lumen_posting_cache_misses_total Posting cache miss count.\n\
# TYPE lumen_posting_cache_misses_total counter\n\
lumen_posting_cache_misses_total 2\n\
# HELP lumen_replace_fields_skipped_total Total docs:replace fields skipped as unchanged no-ops.\n\
# TYPE lumen_replace_fields_skipped_total counter\n\
lumen_replace_fields_skipped_total 6\n\
# HELP lumen_shard_map_version This pod's live routed shard-map version (0 outside routed deployments).\n\
# TYPE lumen_shard_map_version gauge\n\
lumen_shard_map_version 3\n\
# HELP lumen_scatter_map_version_mismatches_total Scatter search sub-responses whose responding pod's map version differed from the sender's.\n\
# TYPE lumen_scatter_map_version_mismatches_total counter\n\
lumen_scatter_map_version_mismatches_total 1\n\
# HELP lumen_reshard_fence_active 1 while this pod believes a reshard-driver write fence is currently armed on it.\n\
# TYPE lumen_reshard_fence_active gauge\n\
lumen_reshard_fence_active 1\n\
# HELP lumen_reshard_fence_armed_unixtime Unix time the currently (or most recently) armed reshard write fence was armed at.\n\
# TYPE lumen_reshard_fence_armed_unixtime gauge\n\
lumen_reshard_fence_armed_unixtime 1700000000\n\
# HELP lumen_auth_registry_reload_failures_total Total failed bearer-token registry hot-reloads.\n\
# TYPE lumen_auth_registry_reload_failures_total counter\n\
lumen_auth_registry_reload_failures_total 2\n\
# HELP lumen_auth_registry_reload_success_unixtime Unix time of the last successful bearer-token registry hot-reload.\n\
# TYPE lumen_auth_registry_reload_success_unixtime gauge\n\
lumen_auth_registry_reload_success_unixtime 1700000001\n\
# HELP lumen_raft_leader_known 1 while this pod's raft election-state poll believes its shard currently has an elected leader, 0 otherwise. Omitted for standalone/non-raft deployments.\n\
# TYPE lumen_raft_leader_known gauge\n\
lumen_raft_leader_known{shard=\"2\"} 1\n";
        assert_eq!(
            out, golden,
            "render() diverged from the pre-refactor capture (#2475 added \
             lumen_reshard_fence_active + lumen_reshard_fence_armed_unixtime + \
             lumen_auth_registry_reload_failures_total + \
             lumen_auth_registry_reload_success_unixtime + lumen_raft_leader_known)"
        );
    }
}
// CODEGEN-END
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/lumen/src/metrics.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Canonical lossless source unit for lumen's in-process Prometheus
      exposition. Runtime behavior is regenerated exactly from the
      authoritative rust-source-unit captured above.

      #1467 R5/R6 added `shard_map_version` (gauge) +
      `set_shard_map_version` (this pod's live routed shard-map version,
      scraped by the reshard driver's `advance_convergence`) and
      `scatter_map_version_mismatches_total` (counter) +
      `incr_scatter_map_version_mismatch` (keyless scatter sub-requests
      whose responding pod's map version disagreed with the sender's).

      #2475 added six fields + methods backing the expanded operator
      PrometheusRule (`render::prometheus_rule`). `raft_shard` (gauge, with
      a `NOT_RAFT` = `u64::MAX` sentinel) + `raft_leader_known` (gauge) +
      `set_raft_leader_known` publish `lumen_raft_leader_known{shard}` off
      `spawn_cluster_state_poller` so `LumenRaftLeaderAbsent` reads a real
      signal; the sentinel makes `render()` omit the labeled series
      entirely for standalone/non-raft pods rather than publish a
      permanent false-positive `0`. `reshard_fence_active` (gauge) +
      `reshard_fence_armed_unixtime` (gauge) + `set_reshard_fence_active`
      are set from `POST /admin/reshard:fence`'s handler and back
      `LumenReshardWorkflowStalled`. `auth_registry_reload_failures_total`
      (counter) + `auth_registry_reload_success_unixtime` (gauge) +
      `incr_auth_registry_reload_failure` /
      `touch_auth_registry_reload_success` are driven by
      `crate::auth::LumenAuthEventSink` and back
      `LumenAuthRegistryReloadFailing`. `Metrics::new()` now seeds the
      `raft_shard` sentinel explicitly (previously `Self::default()`
      sufficed since every field defaulted to a valid `0`). `render()`'s
      golden-output test extended; new test
      `render_omits_raft_leader_known_when_never_set` added.
```
