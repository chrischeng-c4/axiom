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
use std::fmt::Write as _;
use std::time::Duration;

/// #2475: sentinel `raft_shard` value meaning "never touched by the raft
/// election-state poller" — see [`Metrics::raft_shard`]. Out of range for
/// any real shard index, so it stays distinguishable from every real shard.
const NOT_RAFT: u64 = u64::MAX;

/// #2519: number of finite `lumen_search_latency_seconds_bucket{le=...}`
/// rows — see [`SEARCH_LATENCY_BUCKETS_US`].
const SEARCH_LATENCY_BUCKET_COUNT: usize = 12;

/// #2519: search-latency histogram bucket upper bounds, sized for search
/// SLOs from a sub-millisecond fast path up to a 5s outlier tail. Each pair
/// is `(the Prometheus "le" label, the same bound in whole microseconds)` —
/// microseconds because every bound here is an exact conversion of a "nice"
/// decimal-seconds SLO value (e.g. `0.0025s == 2_500us`), so bucket
/// assignment in [`Metrics::observe_search`] is exact integer comparison,
/// never float rounding. Bucket `i` (see `search_latency_buckets`) counts
/// observations in `(bound[i-1], bound[i]]`, with `bound[-1] == 0`.
/// Observations past the last bound aren't stored in a 13th counter — the
/// Prometheus `+Inf` bucket renders as the plain total observation count
/// instead (see `render_search_latency_histogram`).
const SEARCH_LATENCY_BUCKETS_US: [(&str, u64); SEARCH_LATENCY_BUCKET_COUNT] = [
    ("0.001", 1_000),
    ("0.0025", 2_500),
    ("0.005", 5_000),
    ("0.01", 10_000),
    ("0.025", 25_000),
    ("0.05", 50_000),
    ("0.1", 100_000),
    ("0.25", 250_000),
    ("0.5", 500_000),
    ("1", 1_000_000),
    ("2.5", 2_500_000),
    ("5", 5_000_000),
];

/// #2519: `lumen_slow_queries_total`'s threshold (milliseconds) when
/// `LUMEN_SLOW_QUERY_MS` is unset or unparseable — see
/// `slow_query_threshold_ms_from_env`.
const DEFAULT_SLOW_QUERY_THRESHOLD_MS: u64 = 500;

/// All metrics carry the `{collection, shard, partition}` label set per
/// the README §5 contract. v1 in-memory single-shard reports
/// `shard="0", partition="0"` as constants; future LSM/Raft tiers will
/// vary `partition` and `shard` respectively.
#[derive(Debug, Default)]
pub struct Metrics {
    pub index_writes_total: Counter,
    pub index_bytes_total: Counter,
    pub search_requests_total: Counter,
    /// #2519 DEPRECATED: kept only for dashboard back-compat. New
    /// consumers should read the `lumen_search_latency_seconds` histogram
    /// (`search_latency_buckets`/`search_latency_us_sum`) instead — it
    /// carries the same observations at real bucket + second-precision
    /// sum resolution instead of a lossy millisecond-rounded sum/count
    /// pair.
    pub search_latency_ms_sum: Counter,
    /// #2519 DEPRECATED: see `search_latency_ms_sum`. Doubles as the
    /// histogram's total observation count (`+Inf` bucket / `_count`) in
    /// `render_search_latency_histogram` — one `observe_search` call is
    /// exactly one observation of both series, so a second atomic would
    /// only duplicate this one.
    pub search_latency_ms_count: Counter,
    /// #2519: exclusive per-bucket search-latency observation counts
    /// backing `lumen_search_latency_seconds_bucket{le=...}` — see
    /// [`SEARCH_LATENCY_BUCKETS_US`] for the bucket bounds/semantics and
    /// `render_search_latency_histogram` for how these become the
    /// cumulative counts a Prometheus histogram requires.
    pub search_latency_buckets: [Counter; SEARCH_LATENCY_BUCKET_COUNT],
    /// #2519: sum of search latencies in whole microseconds — an integer
    /// atomic (not a float) for lock-free accumulation; `render()` divides
    /// by 1e6 to produce `lumen_search_latency_seconds_sum`.
    pub search_latency_us_sum: Counter,
    /// #2519: total search queries whose latency met or exceeded
    /// `slow_query_threshold_ms` (`LUMEN_SLOW_QUERY_MS`, default
    /// [`DEFAULT_SLOW_QUERY_THRESHOLD_MS`]ms) — see
    /// [`Metrics::observe_search`].
    pub slow_queries_total: Counter,
    /// #2519: the resolved `LUMEN_SLOW_QUERY_MS` threshold in
    /// milliseconds, read once in [`Metrics::new`] (server startup)
    /// rather than per `observe_search` call — mirrors `LUMEN_HNSW_EF`'s
    /// read-once-at-construction convention (`hnsw_search_ef` in
    /// `src/vector_index.rs`) instead of adding env-var lock traffic to
    /// the search hot path. Not published as its own series;
    /// `lumen_slow_queries_total`'s HELP text documents the env var for
    /// operators reading `/metrics` directly.
    pub slow_query_threshold_ms: Gauge,
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
    /// #2516: `1` while this node is in ENOSPC degraded read-only mode
    /// (a durable write path — local AOF/WAL append, segment/RDB
    /// checkpoint save, or raft log append — hit
    /// `io::ErrorKind::StorageFull`), `0` otherwise. Sticky: stays `1`
    /// until the periodic re-probe (`LUMEN_STORAGE_FULL_REPROBE_SECS`, see
    /// `src/bin/lumen.rs`) confirms the data dir accepts a write again, or
    /// the process restarts. `crate::api::enforce_storage_writable` reads
    /// this to fast-fail mutating endpoints without touching the durable
    /// path; `render::prometheus_rule`'s `LumenStorageDegraded` alert reads
    /// the published `lumen_storage_degraded` series.
    pub storage_degraded: Gauge,
    /// #2516: total genuine ENOSPC hits observed on a durable write path,
    /// monotonic across the process lifetime (never reset when
    /// `storage_degraded` clears) — see [`Metrics::mark_storage_degraded`].
    pub storage_full_errors_total: Counter,
}

impl Metrics {
    pub fn new() -> Self {
        let metrics = Self::default();
        // #2475: seed the "never touched by the raft poller" sentinel;
        // every other field's `Default` (0) is already the right initial
        // value.
        metrics.raft_shard.set(NOT_RAFT);
        // #2519: resolve `LUMEN_SLOW_QUERY_MS` once at construction time.
        metrics
            .slow_query_threshold_ms
            .set(slow_query_threshold_ms_from_env());
        metrics
    }

    pub fn incr_index(&self, items: u64, bytes: u64) {
        self.index_writes_total.add(items);
        self.index_bytes_total.add(bytes);
    }

    /// Record one search observation of `elapsed`. Updates the deprecated
    /// millisecond sum/count pair (back-compat, see `search_latency_ms_sum`),
    /// the `lumen_search_latency_seconds` histogram (#2519), and
    /// `slow_queries_total` when `elapsed` meets or exceeds
    /// `slow_query_threshold_ms` — `>=`, not `>`, so a
    /// `LUMEN_SLOW_QUERY_MS=0` override (used by tests to force every
    /// search to count as slow) actually fires on a `0`ms observation.
    pub fn observe_search(&self, elapsed: Duration) {
        self.search_requests_total.incr();

        let latency_ms = elapsed.as_millis() as u64;
        self.search_latency_ms_sum.add(latency_ms);
        self.search_latency_ms_count.incr();

        let latency_us = u64::try_from(elapsed.as_micros()).unwrap_or(u64::MAX);
        self.search_latency_us_sum.add(latency_us);
        if let Some(idx) = SEARCH_LATENCY_BUCKETS_US
            .iter()
            .position(|&(_, bound_us)| latency_us <= bound_us)
        {
            self.search_latency_buckets[idx].incr();
        }

        if latency_ms >= self.slow_query_threshold_ms.get() {
            self.slow_queries_total.incr();
        }
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

    /// #2516: flip into ENOSPC degraded read-only mode and count the hit.
    /// Called from every durable-write-path origin that classifies its
    /// failure as `io::ErrorKind::StorageFull` (see
    /// `crate::coordinator::is_storage_full`) — the coordinator apply
    /// loop's local AOF persist, the periodic RDB/segment checkpoint
    /// snapshotters, and (when the `raft-wal` feature is active) raft log
    /// append. Idempotent: calling it while already degraded still counts
    /// the new hit but leaves the gauge at `1`.
    pub fn mark_storage_degraded(&self) {
        self.storage_full_errors_total.incr();
        self.storage_degraded.set(1);
    }

    /// #2516: `true` while this node is in ENOSPC degraded read-only mode.
    pub fn is_storage_degraded(&self) -> bool {
        self.storage_degraded.get() != 0
    }

    /// #2516: clear degraded mode once the periodic re-probe confirms the
    /// data dir accepts a write again.
    pub fn clear_storage_degraded(&self) {
        self.storage_degraded.set(0);
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
                "DEPRECATED (#2519): sum of search latencies in milliseconds; kept for \
                 dashboard back-compat, see lumen_search_latency_seconds for the real \
                 histogram.",
                self.search_latency_ms_sum.get(),
            ),
            Sample::new(
                "lumen_search_latency_ms_count",
                "counter",
                "DEPRECATED (#2519): count of search latency observations; kept for \
                 dashboard back-compat, see lumen_search_latency_seconds for the real \
                 histogram.",
                self.search_latency_ms_count.get(),
            ),
            Sample::new(
                "lumen_slow_queries_total",
                "counter",
                "Total search queries at/above the slow-query threshold \
                 (LUMEN_SLOW_QUERY_MS, default 500ms).",
                self.slow_queries_total.get(),
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
                "lumen_storage_degraded",
                "gauge",
                "1 while this node is in ENOSPC degraded read-only mode (a durable write \
                 path hit disk-full).",
                self.storage_degraded.get(),
            ),
            Sample::new(
                "lumen_storage_full_errors_total",
                "counter",
                "Total genuine ENOSPC hits observed on a durable write path.",
                self.storage_full_errors_total.get(),
            ),
        ];
        let mut out = metrics_prometheus::render(&samples);
        // #2519: real Prometheus histogram for search latency — hand-rolled
        // rather than via `metrics_prometheus::render_labeled` because a
        // histogram's `_bucket`/`_sum`/`_count` sample names all suffix ONE
        // base name under a single `# HELP`/`# TYPE histogram` pair, while
        // `render_labeled` assumes one bare metric name per row.
        out.push_str(&self.render_search_latency_histogram());
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

    /// #2519: renders `lumen_search_latency_seconds_bucket{le=...}`
    /// (cumulative, per the Prometheus histogram convention) followed by
    /// `_sum` and `_count`, all under one `# HELP`/`# TYPE histogram`
    /// declaration. See [`SEARCH_LATENCY_BUCKETS_US`] for the bucket
    /// bounds and `search_latency_ms_count`'s doc for why `_count`/`+Inf`
    /// reuse that field instead of a dedicated histogram-count atomic.
    fn render_search_latency_histogram(&self) -> String {
        const NAME: &str = "lumen_search_latency_seconds";
        let mut out = String::new();
        let _ = writeln!(
            out,
            "# HELP {NAME} Search latency histogram in seconds, bucketed for search SLOs."
        );
        let _ = writeln!(out, "# TYPE {NAME} histogram");
        let mut cumulative = 0u64;
        for ((le, _bound_us), bucket) in SEARCH_LATENCY_BUCKETS_US
            .iter()
            .zip(self.search_latency_buckets.iter())
        {
            cumulative += bucket.get();
            let _ = writeln!(out, "{NAME}_bucket{{le=\"{le}\"}} {cumulative}");
        }
        let total = self.search_latency_ms_count.get();
        let _ = writeln!(out, "{NAME}_bucket{{le=\"+Inf\"}} {total}");
        let sum_seconds = self.search_latency_us_sum.get() as f64 / 1_000_000.0;
        let _ = writeln!(out, "{NAME}_sum {sum_seconds}");
        let _ = writeln!(out, "{NAME}_count {total}");
        out
    }
}

/// #2519: `LUMEN_SLOW_QUERY_MS` (milliseconds) if set and parseable to a
/// `u64`, else [`DEFAULT_SLOW_QUERY_THRESHOLD_MS`]. Read once at
/// `Metrics::new()` construction — mirrors `hnsw_search_ef`'s
/// read-once-per-construction convention in `src/vector_index.rs`.
fn slow_query_threshold_ms_from_env() -> u64 {
    std::env::var("LUMEN_SLOW_QUERY_MS")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(DEFAULT_SLOW_QUERY_THRESHOLD_MS)
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
        m.observe_search(Duration::from_millis(7));
        m.set_raft_leader_known(2, true);
        m.set_reshard_fence_active(true);
        let out = m.render();
        for name in [
            "lumen_index_writes_total",
            // #2519: deprecated back-compat series must still be present.
            "lumen_search_latency_ms_sum",
            "lumen_search_latency_ms_count",
            // #2519: the new histogram + slow-query series.
            "lumen_search_latency_seconds_bucket",
            "lumen_search_latency_seconds_sum",
            "lumen_search_latency_seconds_count",
            "lumen_slow_queries_total",
            "lumen_storage_bytes",
            "lumen_posting_cache_hits_total",
            "lumen_replace_fields_skipped_total",
            "lumen_shard_map_version",
            "lumen_scatter_map_version_mismatches_total",
            "lumen_reshard_fence_active",
            "lumen_reshard_fence_armed_unixtime",
            "lumen_raft_leader_known",
            "lumen_storage_degraded",
            "lumen_storage_full_errors_total",
        ] {
            assert!(out.contains(name), "expected {name} in:\n{out}");
        }
        assert!(
            out.contains("lumen_raft_leader_known{shard=\"2\"} 1"),
            "expected labeled raft series in:\n{out}"
        );
    }

    /// #2519: a search observation lands in the correct cumulative
    /// histogram bucket, and every bucket >= its own falls in too (a
    /// Prometheus histogram bucket is "le", not exclusive).
    #[test]
    fn observe_search_updates_histogram_buckets() {
        let m = Metrics::new();
        m.observe_search(Duration::from_micros(1_500)); // 1.5ms -> le=0.0025 bucket
        let out = m.render();
        assert!(
            out.contains("lumen_search_latency_seconds_bucket{le=\"0.001\"} 0"),
            "1.5ms observation must not count in the 1ms bucket:\n{out}"
        );
        assert!(
            out.contains("lumen_search_latency_seconds_bucket{le=\"0.0025\"} 1"),
            "1.5ms observation must count in the 2.5ms bucket:\n{out}"
        );
        assert!(
            out.contains("lumen_search_latency_seconds_bucket{le=\"5\"} 1"),
            "cumulative buckets past the observation's own bucket must include it:\n{out}"
        );
        assert!(
            out.contains("lumen_search_latency_seconds_bucket{le=\"+Inf\"} 1"),
            "+Inf bucket must equal the total observation count:\n{out}"
        );
        assert!(
            out.contains("lumen_search_latency_seconds_count 1"),
            "histogram _count must equal the total observation count:\n{out}"
        );
    }

    /// #2519 AC: an artificially-slow observation (here, `Duration::MAX`
    /// against a default threshold) increments `lumen_slow_queries_total`,
    /// and it stays folded into the `+Inf` bucket rather than a stored
    /// 13th bucket counter.
    #[test]
    fn observe_search_increments_slow_queries_over_threshold() {
        let m = Metrics::new();
        m.observe_search(Duration::from_secs(10)); // past every finite bucket
        assert_eq!(m.slow_queries_total.get(), 1);
        let out = m.render();
        assert!(out.contains("lumen_slow_queries_total 1"), "{out}");
        assert!(
            out.contains("lumen_search_latency_seconds_bucket{le=\"5\"} 0"),
            "a 10s observation must not land in the 5s bucket:\n{out}"
        );
        assert!(out.contains("lumen_search_latency_seconds_bucket{le=\"+Inf\"} 1"));
    }

    // Process-global env mutex shared across LUMEN_SLOW_QUERY_MS-mutating
    // tests (mirrors `auth.rs`'s `AUTH_ENV_LOCK`).
    static SLOW_QUERY_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// #2519 AC: the "threshold=0 trick" — `LUMEN_SLOW_QUERY_MS=0` makes
    /// every search, however fast, count as slow. `Metrics::new()` reads
    /// the env var once at construction, so it must be set before that
    /// call.
    #[test]
    fn slow_query_threshold_zero_counts_every_search_as_slow() {
        let _g = SLOW_QUERY_ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::set_var("LUMEN_SLOW_QUERY_MS", "0");
        }
        let m = Metrics::new();
        unsafe {
            std::env::remove_var("LUMEN_SLOW_QUERY_MS");
        }
        m.observe_search(Duration::from_micros(1));
        assert_eq!(
            m.slow_queries_total.get(),
            1,
            "a threshold=0 override must count even a ~0ms search as slow"
        );
    }

    /// A fast search under the default 500ms threshold must NOT count as
    /// slow (regression guard against an inverted comparison).
    #[test]
    fn fast_search_under_default_threshold_is_not_slow() {
        let _g = SLOW_QUERY_ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::remove_var("LUMEN_SLOW_QUERY_MS");
        }
        let m = Metrics::new();
        m.observe_search(Duration::from_millis(5));
        assert_eq!(m.slow_queries_total.get(), 0);
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

    /// #2516: `mark_storage_degraded` flips the gauge to `1` and counts the
    /// hit; `clear_storage_degraded` (the periodic re-probe) flips it back
    /// without touching the counter — the counter is a lifetime total, not
    /// a "currently degraded" signal.
    #[test]
    fn storage_degraded_marks_and_clears() {
        let m = Metrics::new();
        assert!(!m.is_storage_degraded());
        assert_eq!(m.storage_full_errors_total.get(), 0);

        m.mark_storage_degraded();
        assert!(m.is_storage_degraded());
        assert_eq!(m.storage_full_errors_total.get(), 1);

        // A second hit while already degraded still counts, but the gauge
        // stays at 1 (sticky, not a counter).
        m.mark_storage_degraded();
        assert!(m.is_storage_degraded());
        assert_eq!(m.storage_full_errors_total.get(), 2);

        m.clear_storage_degraded();
        assert!(!m.is_storage_degraded());
        assert_eq!(
            m.storage_full_errors_total.get(),
            2,
            "clearing degraded mode must not reset the lifetime error counter"
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
        m.observe_search(Duration::from_millis(7));
        m.observe_search(Duration::from_millis(9));
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
        m.set_raft_leader_known(2, true);
        m.mark_storage_degraded();
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
# HELP lumen_search_latency_ms_sum DEPRECATED (#2519): sum of search latencies in milliseconds; kept for dashboard back-compat, see lumen_search_latency_seconds for the real histogram.\n\
# TYPE lumen_search_latency_ms_sum counter\n\
lumen_search_latency_ms_sum 16\n\
# HELP lumen_search_latency_ms_count DEPRECATED (#2519): count of search latency observations; kept for dashboard back-compat, see lumen_search_latency_seconds for the real histogram.\n\
# TYPE lumen_search_latency_ms_count counter\n\
lumen_search_latency_ms_count 2\n\
# HELP lumen_slow_queries_total Total search queries at/above the slow-query threshold (LUMEN_SLOW_QUERY_MS, default 500ms).\n\
# TYPE lumen_slow_queries_total counter\n\
lumen_slow_queries_total 0\n\
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
# HELP lumen_storage_degraded 1 while this node is in ENOSPC degraded read-only mode (a durable write path hit disk-full).\n\
# TYPE lumen_storage_degraded gauge\n\
lumen_storage_degraded 1\n\
# HELP lumen_storage_full_errors_total Total genuine ENOSPC hits observed on a durable write path.\n\
# TYPE lumen_storage_full_errors_total counter\n\
lumen_storage_full_errors_total 1\n\
# HELP lumen_search_latency_seconds Search latency histogram in seconds, bucketed for search SLOs.\n\
# TYPE lumen_search_latency_seconds histogram\n\
lumen_search_latency_seconds_bucket{le=\"0.001\"} 0\n\
lumen_search_latency_seconds_bucket{le=\"0.0025\"} 0\n\
lumen_search_latency_seconds_bucket{le=\"0.005\"} 0\n\
lumen_search_latency_seconds_bucket{le=\"0.01\"} 2\n\
lumen_search_latency_seconds_bucket{le=\"0.025\"} 2\n\
lumen_search_latency_seconds_bucket{le=\"0.05\"} 2\n\
lumen_search_latency_seconds_bucket{le=\"0.1\"} 2\n\
lumen_search_latency_seconds_bucket{le=\"0.25\"} 2\n\
lumen_search_latency_seconds_bucket{le=\"0.5\"} 2\n\
lumen_search_latency_seconds_bucket{le=\"1\"} 2\n\
lumen_search_latency_seconds_bucket{le=\"2.5\"} 2\n\
lumen_search_latency_seconds_bucket{le=\"5\"} 2\n\
lumen_search_latency_seconds_bucket{le=\"+Inf\"} 2\n\
lumen_search_latency_seconds_sum 0.016\n\
lumen_search_latency_seconds_count 2\n\
# HELP lumen_raft_leader_known 1 while this pod's raft election-state poll believes its shard currently has an elected leader, 0 otherwise. Omitted for standalone/non-raft deployments.\n\
# TYPE lumen_raft_leader_known gauge\n\
lumen_raft_leader_known{shard=\"2\"} 1\n";
        assert_eq!(
            out, golden,
            "render() diverged from the pre-refactor capture (#2475 added \
             lumen_reshard_fence_active + lumen_reshard_fence_armed_unixtime + \
             lumen_raft_leader_known; \
             #2519 added lumen_slow_queries_total + the \
             lumen_search_latency_seconds histogram, and marked \
             lumen_search_latency_ms_sum/_count deprecated in HELP text; \
             #2516 added lumen_storage_degraded + lumen_storage_full_errors_total)"
        );
    }
}
// CODEGEN-END
