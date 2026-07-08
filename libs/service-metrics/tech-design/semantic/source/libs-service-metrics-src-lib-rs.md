---
id: libs-service-metrics-src-lib-rs
summary: Lossless rust-source-unit coverage for `libs/service-metrics/src/lib.rs`.
capability_refs:
  - id: shared-prometheus-metric-primitives
    role: primary
    claim: shared-prometheus-metric-primitives-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Service Metrics library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/service-metrics/src/lib.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/service-metrics/src/lib.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Counter` | libs/service-metrics/src/lib.rs | struct | pub | 24 | pub struct Counter(AtomicU64); |
| `new` | libs/service-metrics/src/lib.rs | const | pub | 27 | pub const fn new() -> Self { |
| `incr` | libs/service-metrics/src/lib.rs | function | pub | 32 | pub fn incr(&self) { |
| `add` | libs/service-metrics/src/lib.rs | function | pub | 37 | pub fn add(&self, n: u64) { |
| `get` | libs/service-metrics/src/lib.rs | function | pub | 42 | pub fn get(&self) -> u64 { |
| `Gauge` | libs/service-metrics/src/lib.rs | struct | pub | 61 | pub struct Gauge(AtomicU64); |
| `set` | libs/service-metrics/src/lib.rs | function | pub | 69 | pub fn set(&self, value: u64) { |
| `Latency` | libs/service-metrics/src/lib.rs | struct | pub | 94 | pub struct Latency { |
| `observe` | libs/service-metrics/src/lib.rs | function | pub | 108 | pub fn observe(&self, value: u64) { |
| `Sample` | libs/service-metrics/src/lib.rs | struct | pub | 118 | pub struct Sample<'a> { |
| `render` | libs/service-metrics/src/lib.rs | function | pub | 140 | pub fn render(samples: &[Sample<'_>]) -> String { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Lock-free Prometheus metric primitives + text-format encoder.
//!
//! Every service in the kit needs the same three shapes — a monotonic
//! counter, a point-in-time gauge, and a `_sum`/`_count` latency
//! observation pair — rendered as Prometheus text format (0.0.4
//! compatible: `# HELP`/`# TYPE` lines followed by the sample). This
//! crate holds only those primitives: no registry side-table, no
//! macros, no dependencies. Callers own their metric structs (typically
//! one field per metric, as plain `Counter`/`Gauge`/`Latency` values)
//! and hand a slice of [`Sample`]s to [`render`] to produce the scrape
//! body.
//!
//! Lifted from lumen's `src/metrics.rs` (#974): lumen's `Metrics`
//! reimplements on top of these primitives with byte-identical
//! `render()` output; keep/relay/loom adoption is a future step.

use std::fmt::Write;
use std::sync::atomic::{AtomicU64, Ordering};

/// A monotonic Prometheus counter: a single `AtomicU64` incremented with
/// `Ordering::Relaxed` (counters have no other state to stay consistent
/// with, so relaxed ordering is sufficient).
#[derive(Debug, Default)]
pub struct Counter(AtomicU64);

impl Counter {
    pub const fn new() -> Self {
        Self(AtomicU64::new(0))
    }

    /// Increment by 1.
    pub fn incr(&self) {
        self.add(1);
    }

    /// Increment by `n`.
    pub fn add(&self, n: u64) {
        self.0.fetch_add(n, Ordering::Relaxed);
    }

    /// Current value.
    pub fn get(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }
}

/// Deref to the underlying `AtomicU64` for callers that need raw
/// atomic ops (e.g. an observable-instrument callback holding only a
/// `&Counter`); `get`/`add`/`incr` above cover the common paths.
impl std::ops::Deref for Counter {
    type Target = AtomicU64;

    fn deref(&self) -> &AtomicU64 {
        &self.0
    }
}

/// A point-in-time Prometheus gauge: a single `AtomicU64` set with
/// `Ordering::Relaxed`.
#[derive(Debug, Default)]
pub struct Gauge(AtomicU64);

impl Gauge {
    pub const fn new() -> Self {
        Self(AtomicU64::new(0))
    }

    /// Overwrite the current value.
    pub fn set(&self, value: u64) {
        self.0.store(value, Ordering::Relaxed);
    }

    /// Current value.
    pub fn get(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }
}

/// Deref to the underlying `AtomicU64`, mirroring [`Counter`]'s escape
/// hatch for raw atomic ops.
impl std::ops::Deref for Gauge {
    type Target = AtomicU64;

    fn deref(&self) -> &AtomicU64 {
        &self.0
    }
}

/// A latency/duration observation: a `sum` + `count` counter pair, the
/// shape a Prometheus summary/histogram takes without bucket
/// boundaries. `observe` records one duration in whatever unit the
/// caller's metric name promises (lumen uses milliseconds).
#[derive(Debug, Default)]
pub struct Latency {
    pub sum: Counter,
    pub count: Counter,
}

impl Latency {
    pub const fn new() -> Self {
        Self {
            sum: Counter::new(),
            count: Counter::new(),
        }
    }

    /// Record one observation of `value`.
    pub fn observe(&self, value: u64) {
        self.sum.add(value);
        self.count.incr();
    }
}

/// One named metric sample ready to render: the Prometheus metric
/// `name`, its `kind` token (`"counter"` or `"gauge"`), the `# HELP`
/// text, and the current `value`.
#[derive(Debug, Clone, Copy)]
pub struct Sample<'a> {
    pub name: &'a str,
    pub kind: &'a str,
    pub help: &'a str,
    pub value: u64,
}

impl<'a> Sample<'a> {
    pub const fn new(name: &'a str, kind: &'a str, help: &'a str, value: u64) -> Self {
        Self {
            name,
            kind,
            help,
            value,
        }
    }
}

/// Render `samples` as Prometheus text format (0.0.4 compatible): each
/// sample emits `# HELP <name> <help>`, `# TYPE <name> <kind>`, then
/// `<name> <value>`, in the order given. Always emits the same set of
/// lines for the same input so scrape configs stay stable.
pub fn render(samples: &[Sample<'_>]) -> String {
    let mut out = String::new();
    for sample in samples {
        let _ = writeln!(out, "# HELP {} {}", sample.name, sample.help);
        let _ = writeln!(out, "# TYPE {} {}", sample.name, sample.kind);
        let _ = writeln!(out, "{} {}", sample.name, sample.value);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_add_and_incr_accumulate() {
        let c = Counter::new();
        c.incr();
        c.add(4);
        assert_eq!(c.get(), 5);
    }

    #[test]
    fn counter_and_gauge_deref_to_raw_atomic() {
        let c = Counter::new();
        c.add(2);
        assert_eq!(c.load(Ordering::Relaxed), 2);

        let g = Gauge::new();
        g.set(9);
        assert_eq!(g.load(Ordering::Relaxed), 9);
    }

    #[test]
    fn gauge_set_overwrites() {
        let g = Gauge::new();
        g.set(10);
        g.set(3);
        assert_eq!(g.get(), 3);
    }

    #[test]
    fn latency_observe_tracks_sum_and_count() {
        let l = Latency::new();
        l.observe(7);
        l.observe(9);
        assert_eq!(l.sum.get(), 16);
        assert_eq!(l.count.get(), 2);
    }

    #[test]
    fn render_emits_help_type_value_per_sample() {
        let samples = [
            Sample::new("demo_total", "counter", "A demo counter.", 3),
            Sample::new("demo_bytes", "gauge", "A demo gauge.", 100),
        ];
        let out = render(&samples);
        assert_eq!(
            out,
            "# HELP demo_total A demo counter.\n\
             # TYPE demo_total counter\n\
             demo_total 3\n\
             # HELP demo_bytes A demo gauge.\n\
             # TYPE demo_bytes gauge\n\
             demo_bytes 100\n"
        );
    }

    /// Golden-render test derived from lumen's `src/metrics.rs` (#974):
    /// reproduces lumen's exact metric set (names, HELP text, `# TYPE`
    /// kinds, ordering) at fixed counter states and asserts the encoder
    /// output is byte-identical to the pre-refactor capture. lumen's own
    /// `Metrics::render` test asserts the same string against the live
    /// `Metrics` struct so the two stay locked together.
    #[test]
    fn golden_render_matches_lumen_metrics_capture() {
        let index_writes_total = Counter::new();
        let index_bytes_total = Counter::new();
        let search = Latency::new();
        let duplicates_requests_total = Counter::new();
        let collections_created_total = Counter::new();
        let schema_fields_total = Counter::new();
        let storage_bytes = Gauge::new();
        let posting_cache_hits_total = Counter::new();
        let posting_cache_misses_total = Counter::new();

        index_writes_total.add(3);
        index_bytes_total.add(100);
        search.observe(7);
        search.observe(9);
        duplicates_requests_total.incr();
        collections_created_total.incr();
        schema_fields_total.add(4);
        storage_bytes.set(2048);
        posting_cache_hits_total.add(5);
        posting_cache_misses_total.add(2);

        let samples = [
            Sample::new(
                "lumen_index_writes_total",
                "counter",
                "Total index items applied.",
                index_writes_total.get(),
            ),
            Sample::new(
                "lumen_index_bytes_total",
                "counter",
                "Total bytes written across all field indexes.",
                index_bytes_total.get(),
            ),
            Sample::new(
                "lumen_search_requests_total",
                "counter",
                "Total search requests served.",
                search.count.get(),
            ),
            Sample::new(
                "lumen_search_latency_ms_sum",
                "counter",
                "Sum of search latencies in milliseconds.",
                search.sum.get(),
            ),
            Sample::new(
                "lumen_search_latency_ms_count",
                "counter",
                "Count of search latency observations.",
                search.count.get(),
            ),
            Sample::new(
                "lumen_duplicates_requests_total",
                "counter",
                "Total duplicate-detection requests.",
                duplicates_requests_total.get(),
            ),
            Sample::new(
                "lumen_collections_created_total",
                "counter",
                "Total collections created or extended.",
                collections_created_total.get(),
            ),
            Sample::new(
                "lumen_schema_fields_total",
                "counter",
                "Total field declarations registered.",
                schema_fields_total.get(),
            ),
            Sample::new(
                "lumen_storage_bytes",
                "gauge",
                "Approximate bytes held by all in-memory field indexes.",
                storage_bytes.get(),
            ),
            Sample::new(
                "lumen_posting_cache_hits_total",
                "counter",
                "Posting cache hit count (0 until LSM cache is wired).",
                posting_cache_hits_total.get(),
            ),
            Sample::new(
                "lumen_posting_cache_misses_total",
                "counter",
                "Posting cache miss count.",
                posting_cache_misses_total.get(),
            ),
        ];

        let out = render(&samples);
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
lumen_posting_cache_misses_total 2\n";
        assert_eq!(
            out, golden,
            "encoder output diverged from lumen's pre-refactor capture"
        );
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/service-metrics/src/lib.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/service-metrics/src/lib.rs` captured during libs codegen standardization.
```
