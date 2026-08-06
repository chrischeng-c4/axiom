// HANDWRITE-BEGIN gap="missing-generator:logic:5a75efe3" tracker="pending-tracker" reason="Own MetricsProvider and Prometheus-backed lifecycle connection counters."
//! Protocol-neutral metric-provider seam and lifecycle event bridge.

use metrics_prometheus::{render, Counter, Sample};
use server_lifecycle::{
    ConnectionMetrics, LifecycleEventError, LifecycleEventSubscription, LifecycleObservation,
};
use std::sync::{Arc, Mutex};

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
    lifecycle: Mutex<Option<LifecycleMetricsSnapshot>>,
}

#[derive(Debug, Clone)]
pub struct LifecycleMetricsSnapshot {
    pub phase: server_lifecycle::LifecyclePhase,
    pub generation: u64,
    pub transition_count: u64,
    pub age_seconds: u64,
    pub reason_code: String,
    transition_at: std::time::Instant,
}

impl LifecycleMetrics {
    pub const fn new() -> Self {
        Self {
            accepted: Counter::new(),
            rejected: Counter::new(),
            closed: Counter::new(),
            lifecycle: Mutex::new(None),
        }
    }

    pub fn lifecycle_snapshot(&self) -> Option<LifecycleMetricsSnapshot> {
        let mut snapshot = self.lifecycle.lock().unwrap().clone()?;
        snapshot.age_seconds = snapshot.transition_at.elapsed().as_secs();
        Some(snapshot)
    }

    pub fn record_lifecycle(&self, observation: &LifecycleObservation) {
        let mut current = self.lifecycle.lock().unwrap();
        if let Some(previous) = current.as_ref() {
            // Watch channels may replay an observation. A duplicate generation
            // is not a transition and must not reset its reason or age.
            if previous.generation == observation.generation {
                return;
            }
            // Coalesced updates can skip generations; account for each boundary.
            let delta = observation
                .generation
                .saturating_sub(previous.generation)
                .max(1);
            let count = previous.transition_count.saturating_add(delta);
            *current = Some(LifecycleMetricsSnapshot {
                phase: observation.phase,
                generation: observation.generation,
                transition_count: count,
                age_seconds: 0,
                reason_code: observation.reason_code.clone(),
                transition_at: observation.transitioned_at.into_std(),
            });
            return;
        }
        *current = Some(LifecycleMetricsSnapshot {
            phase: observation.phase,
            generation: observation.generation,
            transition_count: 1,
            age_seconds: 0,
            reason_code: observation.reason_code.clone(),
            transition_at: observation.transitioned_at.into_std(),
        });
    }

    pub async fn observe_lifecycle(
        self: Arc<Self>,
        mut events: LifecycleEventSubscription,
    ) -> Result<(), LifecycleEventError> {
        loop {
            match events.next().await {
                Ok(observation) => {
                    self.record_lifecycle(&observation);
                    tracing::info!(target: "service.lifecycle", phase = ?observation.phase, generation = observation.generation, reason_code = %observation.reason_code, "lifecycle transition");
                    if matches!(
                        observation.phase,
                        server_lifecycle::LifecyclePhase::Stopped
                            | server_lifecycle::LifecyclePhase::Fatal
                    ) {
                        return Ok(());
                    }
                }
                Err(error) => return Err(error),
            }
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
        let mut samples = vec![
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
        ];
        samples.extend(self.lifecycle_samples());
        render(&samples)
    }
}

impl LifecycleMetrics {
    fn lifecycle_samples(&self) -> [Sample<'_>; 4] {
        let known = self.lifecycle_snapshot();
        let mut snapshot = known.clone().unwrap_or(LifecycleMetricsSnapshot {
            phase: server_lifecycle::LifecyclePhase::Starting,
            generation: 0,
            transition_count: 0,
            age_seconds: 0,
            reason_code: "unknown".into(),
            transition_at: std::time::Instant::now(),
        });
        snapshot.age_seconds = snapshot.transition_at.elapsed().as_secs();
        [
            Sample::new(
                "service_lifecycle_phase",
                "gauge",
                "Current lifecycle phase code.",
                known.map(|_| phase_code(snapshot.phase)).unwrap_or(255),
            ),
            Sample::new(
                "service_lifecycle_generation",
                "gauge",
                "Current lifecycle generation.",
                snapshot.generation,
            ),
            Sample::new(
                "service_lifecycle_transitions_total",
                "counter",
                "Lifecycle transitions observed.",
                snapshot.transition_count,
            ),
            Sample::new(
                "service_lifecycle_transition_age_seconds",
                "gauge",
                "Age of current lifecycle phase.",
                snapshot.age_seconds,
            ),
        ]
    }
}

fn phase_code(phase: server_lifecycle::LifecyclePhase) -> u64 {
    match phase {
        server_lifecycle::LifecyclePhase::Starting => 0,
        server_lifecycle::LifecyclePhase::Recovering => 1,
        server_lifecycle::LifecyclePhase::Serving => 2,
        server_lifecycle::LifecyclePhase::Degraded => 3,
        server_lifecycle::LifecyclePhase::Draining => 4,
        server_lifecycle::LifecyclePhase::Stopping => 5,
        server_lifecycle::LifecyclePhase::Stopped => 6,
        server_lifecycle::LifecyclePhase::Fatal => 7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lifecycle_metrics_use_canonical_prometheus_encoder() {
        let metrics = LifecycleMetrics::new();
        let rendered = metrics.render_metrics();
        assert!(rendered.contains("service_lifecycle_phase 255\n"));
        assert!(rendered.contains("service_lifecycle_generation 0\n"));
        assert!(rendered.contains("service_lifecycle_transitions_total 0\n"));
        assert!(rendered.contains("service_lifecycle_transition_age_seconds 0\n"));
    }
}
// HANDWRITE-END
