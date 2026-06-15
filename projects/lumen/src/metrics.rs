// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-metrics-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Lightweight in-process Prometheus exposition.
//!
//! v1 keeps the metric surface narrow and dep-free: a handful of
//! `AtomicU64` counters/gauges + a single `render` that emits the
//! Prometheus text-format. When request volume grows past what
//! lock-free counters can serve, swap in `prometheus`/`metrics` crates
//! without changing the wire format the scraper sees.

use std::fmt::Write;
use std::sync::atomic::{AtomicU64, Ordering};

/// All metrics carry the `{collection, shard, partition}` label set per
/// the README §5 contract. v1 in-memory single-shard reports
/// `shard="0", partition="0"` as constants; future LSM/Raft tiers will
/// vary `partition` and `shard` respectively.
#[derive(Debug, Default)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-metrics-rs.md#source
pub struct Metrics {
    pub index_writes_total: AtomicU64,
    pub index_bytes_total: AtomicU64,
    pub search_requests_total: AtomicU64,
    pub search_latency_ms_sum: AtomicU64,
    pub search_latency_ms_count: AtomicU64,
    pub duplicates_requests_total: AtomicU64,
    pub collections_created_total: AtomicU64,
    pub schema_fields_total: AtomicU64,
    pub storage_bytes: AtomicU64,
    pub posting_cache_hits_total: AtomicU64,
    pub posting_cache_misses_total: AtomicU64,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-metrics-rs.md#source
impl Metrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn incr_index(&self, items: u64, bytes: u64) {
        self.index_writes_total.fetch_add(items, Ordering::Relaxed);
        self.index_bytes_total.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn observe_search(&self, latency_ms: u64) {
        self.search_requests_total.fetch_add(1, Ordering::Relaxed);
        self.search_latency_ms_sum
            .fetch_add(latency_ms, Ordering::Relaxed);
        self.search_latency_ms_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn incr_duplicates(&self) {
        self.duplicates_requests_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn incr_collection_created(&self, fields: u64) {
        self.collections_created_total
            .fetch_add(1, Ordering::Relaxed);
        self.schema_fields_total
            .fetch_add(fields, Ordering::Relaxed);
    }

    pub fn set_storage_bytes(&self, bytes: u64) {
        self.storage_bytes.store(bytes, Ordering::Relaxed);
    }

    /// Prometheus text format (0.0.4 compatible). Always emits the same
    /// set of metric names so scrape configs are stable.
    pub fn render(&self) -> String {
        let mut s = String::new();
        let pairs: &[(&str, &str, &str, u64)] = &[
            (
                "lumen_index_writes_total",
                "counter",
                "Total index items applied.",
                self.index_writes_total.load(Ordering::Relaxed),
            ),
            (
                "lumen_index_bytes_total",
                "counter",
                "Total bytes written across all field indexes.",
                self.index_bytes_total.load(Ordering::Relaxed),
            ),
            (
                "lumen_search_requests_total",
                "counter",
                "Total search requests served.",
                self.search_requests_total.load(Ordering::Relaxed),
            ),
            (
                "lumen_search_latency_ms_sum",
                "counter",
                "Sum of search latencies in milliseconds.",
                self.search_latency_ms_sum.load(Ordering::Relaxed),
            ),
            (
                "lumen_search_latency_ms_count",
                "counter",
                "Count of search latency observations.",
                self.search_latency_ms_count.load(Ordering::Relaxed),
            ),
            (
                "lumen_duplicates_requests_total",
                "counter",
                "Total duplicate-detection requests.",
                self.duplicates_requests_total.load(Ordering::Relaxed),
            ),
            (
                "lumen_collections_created_total",
                "counter",
                "Total collections created or extended.",
                self.collections_created_total.load(Ordering::Relaxed),
            ),
            (
                "lumen_schema_fields_total",
                "counter",
                "Total field declarations registered.",
                self.schema_fields_total.load(Ordering::Relaxed),
            ),
            (
                "lumen_storage_bytes",
                "gauge",
                "Approximate bytes held by all in-memory field indexes.",
                self.storage_bytes.load(Ordering::Relaxed),
            ),
            (
                "lumen_posting_cache_hits_total",
                "counter",
                "Posting cache hit count (0 until LSM cache is wired).",
                self.posting_cache_hits_total.load(Ordering::Relaxed),
            ),
            (
                "lumen_posting_cache_misses_total",
                "counter",
                "Posting cache miss count.",
                self.posting_cache_misses_total.load(Ordering::Relaxed),
            ),
        ];
        for (name, kind, help, value) in pairs {
            let _ = writeln!(s, "# HELP {name} {help}");
            let _ = writeln!(s, "# TYPE {name} {kind}");
            let _ = writeln!(s, "{name} {value}");
        }
        s
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
        ] {
            assert!(out.contains(name), "expected {name} in:\n{out}");
        }
    }
}
// CODEGEN-END
