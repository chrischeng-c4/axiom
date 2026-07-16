// HANDWRITE-BEGIN gap="missing-generator:logic:5a75efe3" tracker="pending-tracker" reason="Own MetricsProvider and Prometheus-backed lifecycle connection counters."
//! Protocol-neutral metric-provider seam and lifecycle event bridge.

use metrics_prometheus::{render, Counter, Sample};
use server_lifecycle::ConnectionMetrics;

/// Renders a complete Prometheus text-format exposition body.
pub trait MetricsProvider: Send + Sync {
    fn render_metrics(&self) -> String {
        String::new()
    }
}

/// Canonical connection counters for any `server-tcp` based runtime.
#[derive(Debug, Default)]
pub struct LifecycleMetrics {
    accepted: Counter,
    rejected: Counter,
    closed: Counter,
}

impl LifecycleMetrics {
    pub const fn new() -> Self {
        Self {
            accepted: Counter::new(),
            rejected: Counter::new(),
            closed: Counter::new(),
        }
    }

    pub fn accepted(&self) -> u64 {
        self.accepted.get()
    }

    pub fn rejected(&self) -> u64 {
        self.rejected.get()
    }

    pub fn closed(&self) -> u64 {
        self.closed.get()
    }
}

impl ConnectionMetrics for LifecycleMetrics {
    fn connection_accepted(&self) {
        self.accepted.incr();
    }

    fn connection_rejected(&self) {
        self.rejected.incr();
    }

    fn connection_closed(&self) {
        self.closed.incr();
    }
}

impl MetricsProvider for LifecycleMetrics {
    fn render_metrics(&self) -> String {
        render(&[
            Sample::new(
                "service_connections_accepted_total",
                "counter",
                "Total accepted service connections.",
                self.accepted(),
            ),
            Sample::new(
                "service_connections_rejected_total",
                "counter",
                "Total service connections rejected by admission.",
                self.rejected(),
            ),
            Sample::new(
                "service_connections_closed_total",
                "counter",
                "Total completed or failed service connections.",
                self.closed(),
            ),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_metrics_use_canonical_prometheus_encoder() {
        let metrics = LifecycleMetrics::new();
        metrics.connection_accepted();
        metrics.connection_accepted();
        metrics.connection_rejected();
        metrics.connection_closed();

        assert_eq!(metrics.accepted(), 2);
        assert_eq!(metrics.rejected(), 1);
        assert_eq!(metrics.closed(), 1);
        assert_eq!(
            metrics.render_metrics(),
            "# HELP service_connections_accepted_total Total accepted service connections.\n\
# TYPE service_connections_accepted_total counter\n\
service_connections_accepted_total 2\n\
# HELP service_connections_rejected_total Total service connections rejected by admission.\n\
# TYPE service_connections_rejected_total counter\n\
service_connections_rejected_total 1\n\
# HELP service_connections_closed_total Total completed or failed service connections.\n\
# TYPE service_connections_closed_total counter\n\
service_connections_closed_total 1\n"
        );
    }
}
// HANDWRITE-END
