//! tape HTTP request observability: per-op request counts + latency,
//! rendered into the Prometheus exposition served at `/metrics`.
//!
//! Built on the shared `libs/metrics-prometheus` primitives (`Latency` — a
//! lock-free `sum`/`count` counter pair — plus the text-format encoder
//! [`metrics_prometheus::render`]); this module owns only tape's metric names
//! and the route -> op mapping. Recording happens in [`track`], a
//! `route_layer` middleware over the `/topics` data plane (mirrors
//! relay/keep's `metrics::track`), so the journal hot path never carries a
//! metrics probe.

use std::sync::Arc;
use std::time::Instant;

use axum::extract::{MatchedPath, Request, State};
use axum::middleware::Next;
use axum::response::Response;
use metrics_prometheus::{Counter, Gauge, Latency, Sample};

/// Per-op request metrics for the tape data plane. One [`Latency`]
/// (count + latency-ms sum) per op family; `count` doubles as the request
/// counter the `tape_<op>_requests_total` sample exposes.
#[derive(Debug, Default)]
pub struct TapeMetrics {
    pub append: Latency,
    pub replay: Latency,
    pub checkpoint_get: Latency,
    pub checkpoint_put: Latency,
    /// Everything else on the data plane.
    pub other: Latency,
    /// #2573: `1` while this node is in ENOSPC degraded read-only mode (the
    /// journal persist path hit `io::ErrorKind::StorageFull`), `0` otherwise.
    /// Sticky: stays `1` until the periodic re-probe
    /// (`TAPE_STORAGE_FULL_REPROBE_SECS`, see `src/bin/tape.rs`) confirms the
    /// store directory accepts a write again, or the process restarts.
    /// `crate::server::enforce_storage_writable` reads this to fast-fail
    /// mutating handlers without touching the durable path;
    /// `operator::render::prometheus_rule`'s `TapeStorageDegraded` alert reads
    /// the published `tape_storage_degraded` series.
    pub storage_degraded: Gauge,
    /// #2573: total genuine ENOSPC hits observed on the journal persist path,
    /// monotonic across the process lifetime (never reset when
    /// `storage_degraded` clears) — see [`TapeMetrics::mark_storage_degraded`].
    /// A node that flaps in and out of degraded mode is invisible in the gauge
    /// alone; this counter is what makes that visible.
    pub storage_full_errors_total: Counter,
}

impl TapeMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// #2573: flip into ENOSPC degraded read-only mode and count the hit.
    ///
    /// Called by [`crate::server::AppState::persist`] — the single durable
    /// write path a serving tape node has — when and only when the failure
    /// reports `io::ErrorKind::StorageFull`. Any other persist failure stays a
    /// plain `500`: retrying a transient I/O error can succeed, retrying
    /// against a full disk cannot, and conflating the two is what makes
    /// clients back off against a condition backoff never clears.
    pub fn mark_storage_degraded(&self) {
        self.storage_degraded.set(1);
        self.storage_full_errors_total.incr();
    }

    /// #2573: leave degraded read-only mode after a re-probe write succeeded.
    /// Deliberately does NOT touch `storage_full_errors_total` — the counter
    /// records that the node was once full, which stays true.
    pub fn clear_storage_degraded(&self) {
        self.storage_degraded.set(0);
    }

    /// #2573: `true` while this node is in ENOSPC degraded read-only mode.
    pub fn is_storage_degraded(&self) -> bool {
        self.storage_degraded.get() == 1
    }

    /// Map a matched axum route pattern to its op family. Unknown routes
    /// land in `other`.
    fn op(&self, route: &str) -> &Latency {
        if route.ends_with("/append") {
            &self.append
        } else if route.ends_with("/replay") || route.ends_with("/replay/stream") {
            &self.replay
        } else if route.ends_with("/checkpoint") {
            // GET vs PUT collapse to the same route pattern; `track` picks
            // the concrete field based on the request method instead.
            &self.other
        } else {
            &self.other
        }
    }

    /// Record one request against the route's op family, disambiguating the
    /// shared `/checkpoint` route pattern by HTTP method.
    fn observe_method(&self, method: &axum::http::Method, route: &str, latency_ms: u64) {
        if route.ends_with("/checkpoint") {
            match *method {
                axum::http::Method::GET => self.checkpoint_get.observe(latency_ms),
                axum::http::Method::PUT => self.checkpoint_put.observe(latency_ms),
                _ => self.other.observe(latency_ms),
            }
        } else {
            self.op(route).observe(latency_ms);
        }
    }

    /// Render the Prometheus text exposition (0.0.4) for the recorded
    /// request metrics, through the shared `metrics_prometheus` encoder.
    pub fn render(&self) -> String {
        metrics_prometheus::render(&[
            Sample::new(
                "tape_append_requests_total",
                "counter",
                "Total append requests.",
                self.append.count.get(),
            ),
            Sample::new(
                "tape_append_latency_ms_sum",
                "counter",
                "Sum of append request latencies in milliseconds.",
                self.append.sum.get(),
            ),
            Sample::new(
                "tape_append_latency_ms_count",
                "counter",
                "Count of append latency observations.",
                self.append.count.get(),
            ),
            Sample::new(
                "tape_replay_requests_total",
                "counter",
                "Total replay requests.",
                self.replay.count.get(),
            ),
            Sample::new(
                "tape_replay_latency_ms_sum",
                "counter",
                "Sum of replay request latencies in milliseconds.",
                self.replay.sum.get(),
            ),
            Sample::new(
                "tape_replay_latency_ms_count",
                "counter",
                "Count of replay latency observations.",
                self.replay.count.get(),
            ),
            Sample::new(
                "tape_checkpoint_get_requests_total",
                "counter",
                "Total checkpoint read requests.",
                self.checkpoint_get.count.get(),
            ),
            Sample::new(
                "tape_checkpoint_get_latency_ms_sum",
                "counter",
                "Sum of checkpoint read request latencies in milliseconds.",
                self.checkpoint_get.sum.get(),
            ),
            Sample::new(
                "tape_checkpoint_get_latency_ms_count",
                "counter",
                "Count of checkpoint read latency observations.",
                self.checkpoint_get.count.get(),
            ),
            Sample::new(
                "tape_checkpoint_put_requests_total",
                "counter",
                "Total checkpoint advance requests.",
                self.checkpoint_put.count.get(),
            ),
            Sample::new(
                "tape_checkpoint_put_latency_ms_sum",
                "counter",
                "Sum of checkpoint advance request latencies in milliseconds.",
                self.checkpoint_put.sum.get(),
            ),
            Sample::new(
                "tape_checkpoint_put_latency_ms_count",
                "counter",
                "Count of checkpoint advance latency observations.",
                self.checkpoint_put.count.get(),
            ),
            Sample::new(
                "tape_other_requests_total",
                "counter",
                "Total other data-plane requests.",
                self.other.count.get(),
            ),
            Sample::new(
                "tape_other_latency_ms_sum",
                "counter",
                "Sum of other data-plane request latencies in milliseconds.",
                self.other.sum.get(),
            ),
            Sample::new(
                "tape_other_latency_ms_count",
                "counter",
                "Count of other data-plane latency observations.",
                self.other.count.get(),
            ),
            Sample::new(
                "tape_storage_degraded",
                "gauge",
                "1 while this node is in ENOSPC degraded read-only mode (the journal persist \
                 path hit disk-full); mutating requests are fast-failed with 507.",
                self.storage_degraded.get(),
            ),
            Sample::new(
                "tape_storage_full_errors_total",
                "counter",
                "Total genuine ENOSPC hits observed on the journal persist path.",
                self.storage_full_errors_total.get(),
            ),
        ])
    }
}

/// `route_layer` middleware: time each matched data-plane request and record
/// it against its op family. Matched patterns collapse `{topic}`/`{consumer}`
/// cardinality, so the metric set stays bounded.
pub async fn track(State(metrics): State<Arc<TapeMetrics>>, req: Request, next: Next) -> Response {
    let route = req
        .extensions()
        .get::<MatchedPath>()
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| req.uri().path().to_string());
    let method = req.method().clone();
    let start = Instant::now();
    let resp = next.run(req).await;
    metrics.observe_method(&method, &route, start.elapsed().as_millis() as u64);
    resp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_map_to_op_families() {
        let m = TapeMetrics::new();
        m.observe_method(&axum::http::Method::POST, "/topics/{topic}/append", 3);
        m.observe_method(&axum::http::Method::GET, "/topics/{topic}/replay", 2);
        m.observe_method(&axum::http::Method::GET, "/topics/{topic}/replay/stream", 4);
        m.observe_method(
            &axum::http::Method::GET,
            "/topics/{topic}/consumers/{consumer}/checkpoint",
            1,
        );
        m.observe_method(
            &axum::http::Method::PUT,
            "/topics/{topic}/consumers/{consumer}/checkpoint",
            4,
        );
        assert_eq!(m.append.count.get(), 1);
        assert_eq!(m.append.sum.get(), 3);
        assert_eq!(m.replay.count.get(), 2);
        assert_eq!(m.replay.sum.get(), 6);
        assert_eq!(m.checkpoint_get.count.get(), 1);
        assert_eq!(m.checkpoint_put.count.get(), 1);
        assert_eq!(m.checkpoint_put.sum.get(), 4);
        assert_eq!(m.other.count.get(), 0);
    }

    #[test]
    fn render_exposes_tape_request_counters() {
        let m = TapeMetrics::new();
        m.observe_method(&axum::http::Method::POST, "/topics/{topic}/append", 7);
        let out = m.render();
        assert!(out.contains("# TYPE tape_append_requests_total counter"));
        assert!(out.contains("tape_append_requests_total 1"));
        assert!(out.contains("tape_append_latency_ms_sum 7"));
        assert!(out.contains("tape_replay_requests_total 0"));
    }

    /// #2573 — the degraded gauge is sticky across `mark`, clears only on
    /// `clear`, and the error counter is monotonic across a full flap cycle.
    /// The counter surviving `clear_storage_degraded` is the point: a node
    /// that fills up and re-probes clean every 30s looks healthy in the gauge
    /// at every scrape, and only the counter shows it.
    #[test]
    fn storage_degraded_is_sticky_and_the_error_counter_is_monotonic() {
        let m = TapeMetrics::new();
        assert!(!m.is_storage_degraded());
        assert_eq!(m.storage_full_errors_total.get(), 0);

        m.mark_storage_degraded();
        assert!(m.is_storage_degraded());
        assert_eq!(m.storage_full_errors_total.get(), 1);

        // Sticky: a second hit while already degraded stays degraded and
        // counts again.
        m.mark_storage_degraded();
        assert!(m.is_storage_degraded());
        assert_eq!(m.storage_full_errors_total.get(), 2);

        m.clear_storage_degraded();
        assert!(!m.is_storage_degraded());
        assert_eq!(
            m.storage_full_errors_total.get(),
            2,
            "clearing the gauge must not rewrite history: the node WAS full twice"
        );
    }

    /// #2573 — both series are exposed so the `TapeStorageDegraded` alert and
    /// the flap counter have something to select on.
    #[test]
    fn render_exposes_the_storage_degraded_series() {
        let m = TapeMetrics::new();
        let clean = m.render();
        assert!(clean.contains("# TYPE tape_storage_degraded gauge"));
        assert!(clean.contains("tape_storage_degraded 0"));
        assert!(clean.contains("# TYPE tape_storage_full_errors_total counter"));
        assert!(clean.contains("tape_storage_full_errors_total 0"));

        m.mark_storage_degraded();
        let degraded = m.render();
        assert!(degraded.contains("tape_storage_degraded 1"));
        assert!(degraded.contains("tape_storage_full_errors_total 1"));
    }
}
