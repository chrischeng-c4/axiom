//! tape HTTP request observability: per-op request counts + latency,
//! rendered into the Prometheus exposition served at `/metrics`.
//!
//! Built on the shared `libs/service-metrics` primitives (`Latency` — a
//! lock-free `sum`/`count` counter pair — plus the text-format encoder
//! [`service_metrics::render`]); this module owns only tape's metric names
//! and the route -> op mapping. Recording happens in [`track`], a
//! `route_layer` middleware over the `/topics` data plane (mirrors
//! relay/keep's `metrics::track`), so the journal hot path never carries a
//! metrics probe.

use std::sync::Arc;
use std::time::Instant;

use axum::extract::{MatchedPath, Request, State};
use axum::middleware::Next;
use axum::response::Response;
use service_metrics::{Latency, Sample};

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
}

impl TapeMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Map a matched axum route pattern to its op family. Unknown routes
    /// land in `other`.
    fn op(&self, route: &str) -> &Latency {
        if route.ends_with("/append") {
            &self.append
        } else if route.ends_with("/replay") {
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
    /// request metrics, through the shared `service_metrics` encoder.
    pub fn render(&self) -> String {
        service_metrics::render(&[
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
        assert_eq!(m.replay.count.get(), 1);
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
}
