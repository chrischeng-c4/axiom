// HANDWRITE-BEGIN gap="missing-generator:logic:defer-prometheus" tracker="#766" reason="Defer metric names over shared lock-free Prometheus primitives."
use metrics_prometheus::{Counter, Sample};

#[derive(Debug, Default)]
pub struct DeferMetrics {
    pub requests: Counter,
    pub dispatch_acked: Counter,
    pub dispatch_retried: Counter,
    pub dispatch_dead_lettered: Counter,
    pub dispatch_lost_ownership: Counter,
}

impl DeferMetrics {
    pub fn render(&self) -> String {
        metrics_prometheus::render(&[
            Sample::new(
                "defer_requests_total",
                "counter",
                "Total Defer data-plane requests.",
                self.requests.get(),
            ),
            Sample::new(
                "defer_dispatch_acked_total",
                "counter",
                "Target deliveries committed successful.",
                self.dispatch_acked.get(),
            ),
            Sample::new(
                "defer_dispatch_retried_total",
                "counter",
                "Target deliveries committed for retry.",
                self.dispatch_retried.get(),
            ),
            Sample::new(
                "defer_dispatch_dead_lettered_total",
                "counter",
                "Target deliveries moved to the DLQ terminal state.",
                self.dispatch_dead_lettered.get(),
            ),
            Sample::new(
                "defer_dispatch_lost_ownership_total",
                "counter",
                "Target delivery results rejected by executor fencing.",
                self.dispatch_lost_ownership.get(),
            ),
        ])
    }
}
// HANDWRITE-END
