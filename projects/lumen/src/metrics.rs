// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-metrics-rs.md#rust-source-unit
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
//! and live in `libs/service-metrics` (#974); this module wires lumen's
//! metric names, HELP text, and `# TYPE` kinds onto them. `Counter`/
//! `Gauge` deref to the underlying `AtomicU64`, so this module's pub API
//! — field names, method names, and `render()`'s byte output — is
//! unchanged for callers, including the `otel` feature's direct
//! `field.load(Ordering::Relaxed)` reads in `src/bin/lumen.rs`.

use service_metrics::{Counter, Gauge, Sample};

/// All metrics carry the `{collection, shard, partition}` label set per
/// the README §5 contract. v1 in-memory single-shard reports
/// `shard="0", partition="0"` as constants; future LSM/Raft tiers will
/// vary `partition` and `shard` respectively.
#[derive(Debug, Default)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-metrics-rs.md#source
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
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-metrics-rs.md#source
impl Metrics {
    pub fn new() -> Self {
        Self::default()
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
        ];
        service_metrics::render(&samples)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_emits_every_metric() {
        let m = Metrics::new();
        m.incr_index(3, 100);
        m.observe_search(7);
        let out = m.render();
        for name in [
            "lumen_index_writes_total",
            "lumen_search_latency_ms_sum",
            "lumen_storage_bytes",
            "lumen_posting_cache_hits_total",
            "lumen_replace_fields_skipped_total",
        ] {
            assert!(out.contains(name), "expected {name} in:\n{out}");
        }
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
lumen_replace_fields_skipped_total 6\n";
        assert_eq!(
            out, golden,
            "render() diverged from the pre-refactor capture (#1293 added lumen_replace_fields_skipped_total)"
        );
    }
}
// CODEGEN-END
