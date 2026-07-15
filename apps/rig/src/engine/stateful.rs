// HANDWRITE-BEGIN gap="missing-generator:logic:09145022" tracker="pending-tracker" reason="Add the reusable typed warmup/observe/fault/recover/verify/teardown runner with bounded actions, retained evidence, and deterministic reports. generator gap: missing-generator:stateful-harness (#1645)."
//! Reusable bounded lifecycle for stateful-service soak and fault scenarios.
//!
//! Rig owns phase ordering, time budgets, evidence retention, and cleanup. A
//! consumer owns the actions and its domain assertions. Timed-out actions are
//! detached, so actions must also use bounded I/O or cooperative cancellation;
//! their evidence sink is closed as soon as the runner records the timeout.

use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One fixed phase in a stateful-service lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatefulPhase {
    Warmup,
    Observe,
    Fault,
    Recover,
    Verify,
    Teardown,
}

impl StatefulPhase {
    fn thread_name(self) -> &'static str {
        match self {
            Self::Warmup => "warmup",
            Self::Observe => "observe",
            Self::Fault => "fault",
            Self::Recover => "recover",
            Self::Verify => "verify",
            Self::Teardown => "teardown",
        }
    }
}

/// Terminal outcome of one lifecycle phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseOutcome {
    Passed,
    Failed,
    TimedOut,
    Skipped,
}

/// A typed, ordered observation retained in the terminal report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceRecord {
    pub sequence: u64,
    pub phase: StatefulPhase,
    pub kind: String,
    pub value: Value,
}

/// Result and timing of one fixed phase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhaseRecord {
    pub phase: StatefulPhase,
    pub outcome: PhaseOutcome,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Machine-readable terminal record for one stateful scenario.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatefulReport {
    pub protocol: String,
    pub scenario_id: String,
    pub passed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_phase: Option<StatefulPhase>,
    pub total_duration_ms: u64,
    pub phases: Vec<PhaseRecord>,
    pub evidence: Vec<EvidenceRecord>,
}

impl StatefulReport {
    /// Find the record for one phase. All six phases are always represented.
    pub fn phase(&self, phase: StatefulPhase) -> Option<&PhaseRecord> {
        self.phases.iter().find(|record| record.phase == phase)
    }
}

/// Time budgets for one lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatefulLimits {
    /// Maximum duration of any individual action.
    pub phase_timeout: Duration,
    /// Maximum duration reserved for all five primary phases together.
    pub scenario_timeout: Duration,
    /// Independent cleanup reserve; teardown always receives this budget.
    pub teardown_timeout: Duration,
}

impl Default for StatefulLimits {
    fn default() -> Self {
        Self {
            phase_timeout: Duration::from_secs(30),
            scenario_timeout: Duration::from_secs(120),
            teardown_timeout: Duration::from_secs(30),
        }
    }
}

/// Evidence handle scoped to the phase currently running.
#[derive(Clone)]
pub struct EvidenceSink {
    phase: StatefulPhase,
    active: Arc<AtomicBool>,
    sequence: Arc<AtomicU64>,
    records: Arc<Mutex<Vec<EvidenceRecord>>>,
}

impl EvidenceSink {
    /// Retain one typed observation. Writes after phase completion or timeout
    /// are intentionally ignored so detached actions cannot mutate a report.
    pub fn record(&self, kind: impl Into<String>, value: Value) {
        if !self.active.load(Ordering::Acquire) {
            return;
        }
        let record = EvidenceRecord {
            sequence: self.sequence.fetch_add(1, Ordering::Relaxed),
            phase: self.phase,
            kind: kind.into(),
            value,
        };
        self.records
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(record);
    }
}

/// Consumer-supplied work for a single phase.
pub type PhaseAction = Box<dyn FnOnce(EvidenceSink) -> Result<()> + Send + 'static>;

/// The six actions bound to the fixed lifecycle.
pub struct StatefulActions {
    pub warmup: PhaseAction,
    pub observe: PhaseAction,
    pub fault: PhaseAction,
    pub recover: PhaseAction,
    pub verify: PhaseAction,
    pub teardown: PhaseAction,
}

/// One runnable stateful-service scenario.
pub struct StatefulScenario {
    pub id: String,
    pub limits: StatefulLimits,
    pub actions: StatefulActions,
}

impl StatefulScenario {
    pub fn new(id: impl Into<String>, actions: StatefulActions) -> Self {
        Self {
            id: id.into(),
            limits: StatefulLimits::default(),
            actions,
        }
    }

    pub fn with_limits(mut self, limits: StatefulLimits) -> Self {
        self.limits = limits;
        self
    }
}

/// Execute a bounded lifecycle and always emit a complete report.
///
/// Primary phases stop at the first failure or timeout. Remaining primary
/// phases are recorded as skipped, while teardown always runs with its own
/// reserve even if the primary lifecycle exhausted its total budget.
pub fn run_stateful(scenario: StatefulScenario) -> StatefulReport {
    let StatefulScenario {
        id,
        limits,
        actions,
    } = scenario;
    let StatefulActions {
        warmup,
        observe,
        fault,
        recover,
        verify,
        teardown,
    } = actions;

    let started = Instant::now();
    let records = Arc::new(Mutex::new(Vec::new()));
    let sequence = Arc::new(AtomicU64::new(0));
    let mut phases = Vec::with_capacity(6);
    let mut failed_phase = None;

    let primary = [
        (StatefulPhase::Warmup, warmup),
        (StatefulPhase::Observe, observe),
        (StatefulPhase::Fault, fault),
        (StatefulPhase::Recover, recover),
        (StatefulPhase::Verify, verify),
    ];

    for (phase, action) in primary {
        if failed_phase.is_some() {
            phases.push(skipped(phase));
            continue;
        }
        let remaining = limits.scenario_timeout.saturating_sub(started.elapsed());
        let record = if remaining.is_zero() {
            PhaseRecord {
                phase,
                outcome: PhaseOutcome::TimedOut,
                duration_ms: 0,
                error: Some("primary lifecycle exhausted its total time budget".into()),
            }
        } else {
            run_action(
                phase,
                action,
                limits.phase_timeout.min(remaining),
                Arc::clone(&records),
                Arc::clone(&sequence),
            )
        };
        if record.outcome != PhaseOutcome::Passed {
            failed_phase = Some(phase);
        }
        phases.push(record);
    }

    let teardown_record = run_action(
        StatefulPhase::Teardown,
        teardown,
        limits.teardown_timeout,
        Arc::clone(&records),
        Arc::clone(&sequence),
    );
    if failed_phase.is_none() && teardown_record.outcome != PhaseOutcome::Passed {
        failed_phase = Some(StatefulPhase::Teardown);
    }
    phases.push(teardown_record);

    let mut evidence = records
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    evidence.sort_by_key(|record| record.sequence);
    let passed = phases
        .iter()
        .all(|record| record.outcome == PhaseOutcome::Passed);

    StatefulReport {
        protocol: "rig.stateful.v1".into(),
        scenario_id: id,
        passed,
        failed_phase,
        total_duration_ms: millis(started.elapsed()),
        phases,
        evidence,
    }
}

fn run_action(
    phase: StatefulPhase,
    action: PhaseAction,
    timeout: Duration,
    records: Arc<Mutex<Vec<EvidenceRecord>>>,
    sequence: Arc<AtomicU64>,
) -> PhaseRecord {
    if timeout.is_zero() {
        return PhaseRecord {
            phase,
            outcome: PhaseOutcome::TimedOut,
            duration_ms: 0,
            error: Some("phase received no remaining time budget".into()),
        };
    }

    let started = Instant::now();
    let active = Arc::new(AtomicBool::new(true));
    let sink = EvidenceSink {
        phase,
        active: Arc::clone(&active),
        sequence,
        records,
    };
    let (tx, rx) = mpsc::sync_channel(1);
    let worker = match thread::Builder::new()
        .name(format!("rig-stateful-{}", phase.thread_name()))
        .spawn(move || {
            let outcome = catch_unwind(AssertUnwindSafe(|| action(sink)))
                .map_err(panic_message)
                .and_then(|result| result.map_err(|error| format!("{error:#}")));
            let _ = tx.send(outcome);
        }) {
        Ok(worker) => worker,
        Err(error) => {
            active.store(false, Ordering::Release);
            return PhaseRecord {
                phase,
                outcome: PhaseOutcome::Failed,
                duration_ms: millis(started.elapsed()),
                error: Some(format!("failed to start phase worker: {error}")),
            };
        }
    };

    let (outcome, error, join) = match rx.recv_timeout(timeout) {
        Ok(Ok(())) => (PhaseOutcome::Passed, None, true),
        Ok(Err(error)) => (PhaseOutcome::Failed, Some(error), true),
        Err(mpsc::RecvTimeoutError::Timeout) => (
            PhaseOutcome::TimedOut,
            Some(format!("phase exceeded its {} ms budget", millis(timeout))),
            false,
        ),
        Err(mpsc::RecvTimeoutError::Disconnected) => (
            PhaseOutcome::Failed,
            Some("phase worker disconnected without an outcome".into()),
            true,
        ),
    };
    active.store(false, Ordering::Release);
    if join {
        let _ = worker.join();
    }

    PhaseRecord {
        phase,
        outcome,
        duration_ms: millis(started.elapsed()),
        error,
    }
}

fn skipped(phase: StatefulPhase) -> PhaseRecord {
    PhaseRecord {
        phase,
        outcome: PhaseOutcome::Skipped,
        duration_ms: 0,
        error: Some("skipped after an earlier phase failed".into()),
    }
}

fn panic_message(payload: Box<dyn Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        return format!("phase panicked: {message}");
    }
    if let Some(message) = payload.downcast_ref::<String>() {
        return format!("phase panicked: {message}");
    }
    "phase panicked with a non-string payload".into()
}

fn millis(duration: Duration) -> u64 {
    duration.as_millis().min(u64::MAX as u128) as u64
}
// HANDWRITE-END
