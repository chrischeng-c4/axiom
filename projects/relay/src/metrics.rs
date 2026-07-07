// HANDWRITE-BEGIN gap="missing-generator:logic:a36ab827" tracker="pending-tracker" reason="RelayMetrics on libs/service-metrics primitives (Latency = count + sum, render): publish / publish-batch / lease / ack / consume / other request counts + latency ms, plus the track route_layer middleware that maps the matched route pattern to its op family (mirrors keep's http/metrics.rs track)."
//! relay HTTP request observability: per-op request counts + latency, rendered
//! into the Prometheus exposition served at `/metrics`.
//!
//! Built on the shared `libs/service-metrics` primitives (`Latency` — a
//! lock-free `sum`/`count` counter pair — plus the text-format encoder
//! [`service_metrics::render`]); this module owns only relay's metric names
//! and the route → op mapping. Recording happens in [`track`], a
//! `route_layer` middleware over the `/v1` data plane (mirroring keep's
//! `http/metrics.rs`), so the engine hot path never carries a metrics probe.

use std::sync::Arc;
use std::time::Instant;

use axum::extract::{MatchedPath, Request, State};
use axum::middleware::Next;
use axum::response::Response;
use service_metrics::{Latency, Sample};

/// Per-op request metrics for the relay data plane. One [`Latency`]
/// (count + latency-ms sum) per op family; `count` doubles as the request
/// counter the `relay_<op>_requests_total` sample exposes.
#[derive(Debug, Default)]
pub struct RelayMetrics {
    pub publish: Latency,
    pub publish_batch: Latency,
    pub lease: Latency,
    pub ack: Latency,
    pub consume: Latency,
    /// Everything else on the data plane (heartbeat, len, ...).
    pub other: Latency,
}

impl RelayMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Map a matched axum route pattern (`/v1/{subject}/<op>`) to its op
    /// family. Batch variants fold into their base op; unknown verbs land in
    /// `other`.
    fn op(&self, route: &str) -> &Latency {
        match route.rsplit('/').next().unwrap_or("") {
            "publish" => &self.publish,
            "publish-batch" => &self.publish_batch,
            "lease" | "lease-batch" => &self.lease,
            "ack" | "ack-batch" => &self.ack,
            "consume" => &self.consume,
            _ => &self.other,
        }
    }

    /// Record one request against the route's op family.
    pub fn observe(&self, route: &str, latency_ms: u64) {
        self.op(route).observe(latency_ms);
    }

    /// Render the Prometheus text exposition (0.0.4) for the recorded
    /// request metrics, through the shared `service_metrics` encoder.
    pub fn render(&self) -> String {
        service_metrics::render(&[
            Sample::new(
                "relay_publish_requests_total",
                "counter",
                "Total publish requests.",
                self.publish.count.get(),
            ),
            Sample::new(
                "relay_publish_latency_ms_sum",
                "counter",
                "Sum of publish request latencies in milliseconds.",
                self.publish.sum.get(),
            ),
            Sample::new(
                "relay_publish_latency_ms_count",
                "counter",
                "Count of publish latency observations.",
                self.publish.count.get(),
            ),
            Sample::new(
                "relay_publish_batch_requests_total",
                "counter",
                "Total publish-batch requests.",
                self.publish_batch.count.get(),
            ),
            Sample::new(
                "relay_publish_batch_latency_ms_sum",
                "counter",
                "Sum of publish-batch request latencies in milliseconds.",
                self.publish_batch.sum.get(),
            ),
            Sample::new(
                "relay_publish_batch_latency_ms_count",
                "counter",
                "Count of publish-batch latency observations.",
                self.publish_batch.count.get(),
            ),
            Sample::new(
                "relay_lease_requests_total",
                "counter",
                "Total lease and lease-batch requests.",
                self.lease.count.get(),
            ),
            Sample::new(
                "relay_lease_latency_ms_sum",
                "counter",
                "Sum of lease request latencies in milliseconds.",
                self.lease.sum.get(),
            ),
            Sample::new(
                "relay_lease_latency_ms_count",
                "counter",
                "Count of lease latency observations.",
                self.lease.count.get(),
            ),
            Sample::new(
                "relay_ack_requests_total",
                "counter",
                "Total ack and ack-batch requests.",
                self.ack.count.get(),
            ),
            Sample::new(
                "relay_ack_latency_ms_sum",
                "counter",
                "Sum of ack request latencies in milliseconds.",
                self.ack.sum.get(),
            ),
            Sample::new(
                "relay_ack_latency_ms_count",
                "counter",
                "Count of ack latency observations.",
                self.ack.count.get(),
            ),
            Sample::new(
                "relay_consume_requests_total",
                "counter",
                "Total streaming consume sessions opened.",
                self.consume.count.get(),
            ),
            Sample::new(
                "relay_consume_latency_ms_sum",
                "counter",
                "Sum of consume handler latencies in milliseconds (stream setup, not stream lifetime).",
                self.consume.sum.get(),
            ),
            Sample::new(
                "relay_consume_latency_ms_count",
                "counter",
                "Count of consume latency observations.",
                self.consume.count.get(),
            ),
            Sample::new(
                "relay_other_requests_total",
                "counter",
                "Total other data-plane requests (heartbeat, len, ...).",
                self.other.count.get(),
            ),
            Sample::new(
                "relay_other_latency_ms_sum",
                "counter",
                "Sum of other data-plane request latencies in milliseconds.",
                self.other.sum.get(),
            ),
            Sample::new(
                "relay_other_latency_ms_count",
                "counter",
                "Count of other data-plane latency observations.",
                self.other.count.get(),
            ),
        ])
    }
}

/// `route_layer` middleware: time each matched data-plane request and record
/// it against its op family. Matched patterns collapse `{subject}`
/// cardinality (`/v1/{subject}/publish`), so the metric set stays bounded.
pub async fn track(
    State(metrics): State<Arc<RelayMetrics>>,
    req: Request,
    next: Next,
) -> Response {
    let route = req
        .extensions()
        .get::<MatchedPath>()
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| req.uri().path().to_string());
    let start = Instant::now();
    let resp = next.run(req).await;
    metrics.observe(&route, start.elapsed().as_millis() as u64);
    resp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_map_to_op_families() {
        let m = RelayMetrics::new();
        m.observe("/v1/{subject}/publish", 3);
        m.observe("/v1/{subject}/publish-batch", 4);
        m.observe("/v1/{subject}/lease", 1);
        m.observe("/v1/{subject}/lease-batch", 1);
        m.observe("/v1/{subject}/ack", 2);
        m.observe("/v1/{subject}/ack-batch", 2);
        m.observe("/v1/{subject}/consume", 5);
        m.observe("/v1/{subject}/heartbeat", 1);
        m.observe("/v1/{subject}/len", 1);
        assert_eq!(m.publish.count.get(), 1);
        assert_eq!(m.publish.sum.get(), 3);
        assert_eq!(m.publish_batch.count.get(), 1);
        assert_eq!(m.lease.count.get(), 2);
        assert_eq!(m.ack.count.get(), 2);
        assert_eq!(m.consume.count.get(), 1);
        assert_eq!(m.other.count.get(), 2);
    }

    #[test]
    fn render_exposes_relay_request_counters() {
        let m = RelayMetrics::new();
        m.observe("/v1/{subject}/publish", 7);
        let out = m.render();
        assert!(out.contains("# TYPE relay_publish_requests_total counter"));
        assert!(out.contains("relay_publish_requests_total 1"));
        assert!(out.contains("relay_publish_latency_ms_sum 7"));
        assert!(out.contains("relay_consume_requests_total 0"));
    }
}
// HANDWRITE-END
