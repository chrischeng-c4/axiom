// CODEGEN-BEGIN
//! Lock-free Prometheus metric primitives + text-format encoder.
//!
//! Every service in the kit needs the same three shapes — a monotonic
//! counter, a point-in-time gauge, and a `_sum`/`_count` latency
//! observation pair — rendered as Prometheus text format (0.0.4
//! compatible: `# HELP`/`# TYPE` lines followed by the sample). This
//! crate holds only those primitives: no registry side-table, no
//! macros, no dependencies. Callers own their metric structs (typically
//! one field per metric, as plain `Counter`/`Gauge`/`Latency` values)
//! and hand a slice of [`Sample`]s to [`render`] to produce an unlabeled
//! scrape body, or [`SampleGroup`]s to [`render_labeled`] when one HELP/TYPE
//! declaration owns multiple labeled rows.
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

/// One upper bound of a [`Histogram`], carrying the same bound twice on
/// purpose: `le` is the Prometheus label exactly as it must appear in the
/// exposition, and `max` is that bound in the integer unit observations are
/// recorded in.
///
/// Deriving either side from the other would mean formatting or parsing a
/// decimal at scrape or observe time. The label is a *display* value an SLO
/// is written against (`"0.5"`, never `"0.500"`), and bucket assignment must
/// stay exact integer comparison — so both are stated, and a test pins that
/// they agree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bucket<'a> {
    /// The Prometheus `le` label value, in the metric name's published unit.
    pub le: &'a str,
    /// The same bound in the integer unit passed to [`Histogram::observe`].
    pub max: u64,
}

impl<'a> Bucket<'a> {
    pub const fn new(le: &'a str, max: u64) -> Self {
        Self { le, max }
    }
}

/// A Prometheus histogram: per-bucket observation counts plus `_sum` and
/// `_count`, all lock-free integer atomics.
///
/// Observations are recorded in an integer *base unit* (microseconds,
/// milliseconds, bytes) and published in the unit the metric name promises,
/// via the `divisor` given to [`Histogram::render`]. Accumulating integers
/// and dividing once at scrape time keeps the hot path free of floats and
/// makes `_sum` exactly reproducible — a float accumulator would drift by an
/// amount that depends on the order observations happened to arrive in.
///
/// Buckets are stored exclusively (bucket `i` counts `(bounds[i-1],
/// bounds[i]]`) and made cumulative at render time, which is the form
/// Prometheus requires. Observations above the last bound are not stored in
/// an extra counter: the `+Inf` row is by definition the total count.
#[derive(Debug)]
pub struct Histogram {
    bounds: &'static [Bucket<'static>],
    counts: Vec<Counter>,
    sum: Counter,
    count: Counter,
}

impl Histogram {
    /// Build a histogram over `bounds`, which must be sorted ascending by
    /// `max` (assignment scans for the first bound an observation fits, so an
    /// unsorted list would silently mis-bucket). Allocates once; every later
    /// operation touches only atomics.
    pub fn new(bounds: &'static [Bucket<'static>]) -> Self {
        Self {
            bounds,
            counts: bounds.iter().map(|_| Counter::new()).collect(),
            sum: Counter::new(),
            count: Counter::new(),
        }
    }

    /// Record one observation of `value`, expressed in the base unit the
    /// bounds' `max` fields use.
    pub fn observe(&self, value: u64) {
        if let Some(index) = self.bounds.iter().position(|bound| value <= bound.max) {
            self.counts[index].incr();
        }
        self.sum.add(value);
        self.count.incr();
    }

    /// Total observations recorded — the `_count` series and the `+Inf` bucket.
    pub fn count(&self) -> u64 {
        self.count.get()
    }

    /// Render this histogram as the three series Prometheus expects:
    /// `<name>_bucket{le=…}` (cumulative, including `+Inf`), `<name>_sum`
    /// scaled by `divisor`, and `<name>_count`.
    ///
    /// `divisor` converts the base unit to the published unit and should be a
    /// power of ten (1000 to publish seconds from milliseconds). It is applied
    /// with integer division plus a zero-padded remainder, so the rendered sum
    /// is exact rather than float-rounded. A `divisor` that is not a power of
    /// ten degrades to plain integer division rather than emitting a
    /// misleading fraction.
    pub fn render(&self, name: &str, help: &str, divisor: u64) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "# HELP {name} {help}");
        let _ = writeln!(out, "# TYPE {name} histogram");
        let mut cumulative = 0u64;
        for (bound, counter) in self.bounds.iter().zip(&self.counts) {
            cumulative += counter.get();
            let _ = writeln!(out, "{name}_bucket{{le=\"{}\"}} {cumulative}", bound.le);
        }
        let total = self.count.get();
        let _ = writeln!(out, "{name}_bucket{{le=\"+Inf\"}} {total}");
        let _ = writeln!(out, "{name}_sum {}", scale_decimal(self.sum.get(), divisor));
        let _ = writeln!(out, "{name}_count {total}");
        out
    }
}

/// Format `value / divisor` exactly, without floating point.
///
/// Returns a plain integer when `divisor` is 1 or is not a power of ten;
/// otherwise an integer part and a remainder zero-padded to the divisor's
/// number of decimal places (`scale_decimal(1_500, 1000) == "1.500"`).
fn scale_decimal(value: u64, divisor: u64) -> String {
    let mut places = 0usize;
    let mut remaining = divisor;
    while remaining > 1 && remaining % 10 == 0 {
        remaining /= 10;
        places += 1;
    }
    if divisor <= 1 || remaining != 1 {
        return (value / divisor.max(1)).to_string();
    }
    format!(
        "{}.{:0width$}",
        value / divisor,
        value % divisor,
        width = places
    )
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

/// One Prometheus label name/value pair. The renderer canonicalizes label
/// order and escapes values, so callers only own label semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Label<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

impl<'a> Label<'a> {
    pub const fn new(name: &'a str, value: &'a str) -> Self {
        Self { name, value }
    }
}

/// One value row within a labeled metric family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabeledSample<'a> {
    pub labels: Vec<Label<'a>>,
    pub value: u64,
}

impl<'a> LabeledSample<'a> {
    pub fn new(labels: Vec<Label<'a>>, value: u64) -> Self {
        Self { labels, value }
    }
}

/// A metric family whose HELP and TYPE declarations are shared by one or more
/// labeled value rows.
#[derive(Debug, Clone, Copy)]
pub struct SampleGroup<'a> {
    pub name: &'a str,
    pub kind: &'a str,
    pub help: &'a str,
    pub samples: &'a [LabeledSample<'a>],
}

impl<'a> SampleGroup<'a> {
    pub const fn new(
        name: &'a str,
        kind: &'a str,
        help: &'a str,
        samples: &'a [LabeledSample<'a>],
    ) -> Self {
        Self {
            name,
            kind,
            help,
            samples,
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

// <HANDWRITE gap="missing-generator:logic" tracker="#1892" reason="Expose the shared Prometheus label-value escaping primitive for custom metric families.">
/// Render labeled metric families as Prometheus text format 0.0.4.
///
/// Groups and rows preserve caller order. Labels within a row are sorted by
/// name then value, and label values escape backslash, double quote, and
/// newline as required by the Prometheus exposition format.
pub fn render_labeled(groups: &[SampleGroup<'_>]) -> String {
    let mut out = String::new();
    for group in groups {
        let _ = writeln!(out, "# HELP {} {}", group.name, group.help);
        let _ = writeln!(out, "# TYPE {} {}", group.name, group.kind);
        for sample in group.samples {
            let _ = write!(out, "{}", group.name);
            if !sample.labels.is_empty() {
                let mut labels = sample.labels.iter().collect::<Vec<_>>();
                labels.sort_unstable_by(|left, right| {
                    left.name
                        .cmp(right.name)
                        .then_with(|| left.value.cmp(right.value))
                });
                out.push('{');
                for (index, label) in labels.into_iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    let _ = write!(out, "{}=\"", label.name);
                    write_escaped_label_value(&mut out, label.value);
                    out.push('"');
                }
                out.push('}');
            }
            let _ = writeln!(out, " {}", sample.value);
        }
    }
    out
}
// </HANDWRITE>

/// Escapes a label value for Prometheus text exposition. Custom metric
/// families should use this instead of duplicating the escaping rules.
pub fn escape_label_value(value: &str) -> String {
    let mut escaped = String::new();
    write_escaped_label_value(&mut escaped, value);
    escaped
}

fn write_escaped_label_value(out: &mut String, value: &str) {
    for character in value.chars() {
        match character {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            other => out.push(other),
        }
    }
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

    #[test]
    fn labeled_render_sorts_and_escapes_labels() {
        let rows = [LabeledSample::new(
            vec![Label::new("zone", "a\\b\nc"), Label::new("pool", "x\"y")],
            7,
        )];
        let groups = [SampleGroup::new(
            "demo_active",
            "gauge",
            "Active demo resources.",
            &rows,
        )];

        assert_eq!(
            render_labeled(&groups),
            "# HELP demo_active Active demo resources.\n\
# TYPE demo_active gauge\n\
demo_active{pool=\"x\\\"y\",zone=\"a\\\\b\\nc\"} 7\n"
        );
        assert_eq!(rows[0].labels[0].name, "zone");
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

    const MS: [Bucket<'static>; 3] = [
        Bucket::new("0.1", 100),
        Bucket::new("0.5", 500),
        Bucket::new("1", 1_000),
    ];

    /// Each bound's `le` label and its integer `max` are written by hand, so
    /// nothing but a test stops them from disagreeing — and a disagreement is
    /// invisible in the scrape body while quietly assigning observations to
    /// the wrong bucket.
    #[test]
    fn bucket_label_and_integer_bound_agree() {
        for bound in MS {
            let expected = scale_decimal(bound.max, 1_000);
            let matches = expected == bound.le
                || expected.trim_end_matches('0').trim_end_matches('.') == bound.le;
            assert!(
                matches,
                "le={:?} does not denote {} in the base unit (got {expected})",
                bound.le, bound.max
            );
        }
    }

    /// Prometheus buckets are cumulative, but storing them that way would mean
    /// touching every counter above an observation on the hot path. Counts are
    /// stored exclusively and summed at render.
    #[test]
    fn buckets_render_cumulatively_from_exclusive_counts() {
        let h = Histogram::new(&MS);
        h.observe(50); // → le=0.1
        h.observe(300); // → le=0.5
        h.observe(300); // → le=0.5
        let out = h.render("op_seconds", "help", 1_000);
        assert!(out.contains("op_seconds_bucket{le=\"0.1\"} 1"), "{out}");
        assert!(out.contains("op_seconds_bucket{le=\"0.5\"} 3"), "{out}");
        assert!(out.contains("op_seconds_bucket{le=\"1\"} 3"), "{out}");
    }

    /// An observation past the last bound belongs to `+Inf` only. It must
    /// still reach `_sum`/`_count`, or a histogram would under-report exactly
    /// the outliers it exists to expose.
    #[test]
    fn an_observation_past_the_last_bound_lands_only_in_inf() {
        let h = Histogram::new(&MS);
        h.observe(9_999);
        let out = h.render("op_seconds", "help", 1_000);
        assert!(out.contains("op_seconds_bucket{le=\"1\"} 0"), "{out}");
        assert!(out.contains("op_seconds_bucket{le=\"+Inf\"} 1"), "{out}");
        assert!(out.contains("op_seconds_sum 9.999"), "{out}");
        assert!(out.contains("op_seconds_count 1"), "{out}");
    }

    /// The whole reason observations accumulate as integers: the rendered sum
    /// is exact and order-independent, where a float accumulator drifts by an
    /// amount that depends on arrival order.
    #[test]
    fn sum_scaling_is_exact_integer_arithmetic() {
        assert_eq!(scale_decimal(1_500, 1_000), "1.500");
        assert_eq!(scale_decimal(7, 1_000), "0.007");
        assert_eq!(scale_decimal(0, 1_000), "0.000");
        // Not a power of ten → integer division rather than a bogus fraction.
        assert_eq!(scale_decimal(3_000, 1_024), "2");
        // No scaling requested.
        assert_eq!(scale_decimal(42, 1), "42");

        let forward = Histogram::new(&MS);
        let backward = Histogram::new(&MS);
        for value in [1u64, 33, 250, 999] {
            forward.observe(value);
        }
        for value in [999u64, 250, 33, 1] {
            backward.observe(value);
        }
        assert_eq!(
            forward.render("op_seconds", "h", 1_000),
            backward.render("op_seconds", "h", 1_000)
        );
    }

    /// An empty histogram must still expose every series, otherwise a rate()
    /// over a freshly started process reports "no data" instead of zero and
    /// an absence alert cannot distinguish the two.
    #[test]
    fn a_histogram_with_no_observations_still_renders_every_series() {
        let out = Histogram::new(&MS).render("op_seconds", "help", 1_000);
        assert_eq!(
            out,
            "# HELP op_seconds help\n\
             # TYPE op_seconds histogram\n\
             op_seconds_bucket{le=\"0.1\"} 0\n\
             op_seconds_bucket{le=\"0.5\"} 0\n\
             op_seconds_bucket{le=\"1\"} 0\n\
             op_seconds_bucket{le=\"+Inf\"} 0\n\
             op_seconds_sum 0.000\n\
             op_seconds_count 0\n"
        );
    }
}
// CODEGEN-END
