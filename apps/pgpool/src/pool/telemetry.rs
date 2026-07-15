// SPEC-MANAGED: apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-transaction-phase-telemetry" tracker="#1750" reason="The pool's bounded opt-in phase counters need a generator primitive for atomic diagnostic aggregates.">
//! Bounded, opt-in aggregate transaction-pool phase telemetry.
//!
//! This module intentionally stores no client identity, SQL text, or individual
//! samples.  It is enabled only for an explicit diagnostic process and keeps a
//! fixed three-phase, two-outcome counter set.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

const PHASES: usize = 3;
const OUTCOMES: usize = 2;

#[derive(Debug, Clone, Copy)]
pub(crate) enum TransactionPhase {
    Acquire = 0,
    Relay = 1,
    Release = 2,
}

impl TransactionPhase {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Acquire => "acquire",
            Self::Relay => "relay",
            Self::Release => "release",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum TransactionPhaseOutcome {
    Success = 0,
    Failure = 1,
}

impl TransactionPhaseOutcome {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failure => "failure",
        }
    }
}

#[derive(Debug)]
struct PhaseAggregate {
    count: AtomicU64,
    total_nanoseconds: AtomicU64,
}

impl PhaseAggregate {
    const fn new() -> Self {
        Self {
            count: AtomicU64::new(0),
            total_nanoseconds: AtomicU64::new(0),
        }
    }

    fn record(&self, elapsed: Duration) {
        self.count.fetch_add(1, Ordering::Relaxed);
        let nanos = elapsed.as_nanos().min(u128::from(u64::MAX)) as u64;
        self.total_nanoseconds.fetch_add(nanos, Ordering::Relaxed);
    }

    fn snapshot(&self) -> (u64, u64) {
        (
            self.count.load(Ordering::Relaxed),
            self.total_nanoseconds.load(Ordering::Relaxed),
        )
    }
}

/// A single bounded Prometheus sample pair.  `phase` and `outcome` have fixed
/// vocabularies, so one pool can produce at most six label combinations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransactionPhaseMetric {
    pub phase: &'static str,
    pub outcome: &'static str,
    pub count: u64,
    pub total_seconds: f64,
}

/// Read-only snapshot consumed by the admin metrics renderer.
#[derive(Debug, Clone, PartialEq)]
pub struct TransactionPhaseTelemetrySnapshot {
    pub metrics: [TransactionPhaseMetric; PHASES * OUTCOMES],
}

/// One process-local telemetry set shared by a pool's cheap cloned handles.
#[derive(Debug)]
pub struct TransactionPhaseTelemetry {
    aggregates: [[PhaseAggregate; OUTCOMES]; PHASES],
}

impl TransactionPhaseTelemetry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            aggregates: std::array::from_fn(|_| std::array::from_fn(|_| PhaseAggregate::new())),
        })
    }

    pub(crate) fn from_environment() -> Option<Arc<Self>> {
        let value = std::env::var("PGPOOL_TRANSACTION_PHASE_TELEMETRY").ok()?;
        matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES").then(Self::new)
    }

    pub(crate) fn record(
        &self,
        phase: TransactionPhase,
        outcome: TransactionPhaseOutcome,
        elapsed: Duration,
    ) {
        self.aggregates[phase as usize][outcome as usize].record(elapsed);
    }

    pub fn snapshot(&self) -> TransactionPhaseTelemetrySnapshot {
        let phases = [
            TransactionPhase::Acquire,
            TransactionPhase::Relay,
            TransactionPhase::Release,
        ];
        let outcomes = [
            TransactionPhaseOutcome::Success,
            TransactionPhaseOutcome::Failure,
        ];
        let mut index = 0;
        let mut metrics = [TransactionPhaseMetric {
            phase: "acquire",
            outcome: "success",
            count: 0,
            total_seconds: 0.0,
        }; PHASES * OUTCOMES];
        for phase in phases {
            for outcome in outcomes {
                let (count, total_nanoseconds) =
                    self.aggregates[phase as usize][outcome as usize].snapshot();
                metrics[index] = TransactionPhaseMetric {
                    phase: phase.as_str(),
                    outcome: outcome.as_str(),
                    count,
                    total_seconds: total_nanoseconds as f64 / 1_000_000_000.0,
                };
                index += 1;
            }
        }
        TransactionPhaseTelemetrySnapshot { metrics }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_is_bounded_and_preserves_phase_outcome_totals() {
        let telemetry = TransactionPhaseTelemetry::new();
        telemetry.record(
            TransactionPhase::Acquire,
            TransactionPhaseOutcome::Success,
            Duration::from_nanos(25),
        );
        telemetry.record(
            TransactionPhase::Acquire,
            TransactionPhaseOutcome::Failure,
            Duration::from_nanos(5),
        );

        let snapshot = telemetry.snapshot();
        assert_eq!(snapshot.metrics.len(), 6);
        assert_eq!(snapshot.metrics[0].phase, "acquire");
        assert_eq!(snapshot.metrics[0].outcome, "success");
        assert_eq!(snapshot.metrics[0].count, 1);
        assert_eq!(snapshot.metrics[0].total_seconds, 0.000_000_025);
        assert_eq!(snapshot.metrics[1].outcome, "failure");
        assert_eq!(snapshot.metrics[1].count, 1);
        assert_eq!(snapshot.metrics[1].total_seconds, 0.000_000_005);
    }
}
// </HANDWRITE>
