use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use tokio::time::Instant;

use crate::deadline::ShutdownDeadline;
use crate::lifecycle::LifecyclePhase;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HookStage {
    AdmissionStop,
    TransportDrain,
    DomainQuiesce,
    BackgroundStop,
    FinalFlush,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookStatus {
    Completed,
    Failed,
    TimedOut,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookOutcome {
    pub stage: HookStage,
    pub name: String,
    pub status: HookStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct ShutdownContext {
    pub deadline: ShutdownDeadline,
}

pub type HookFuture = Pin<Box<dyn Future<Output = Result<(), String>> + Send>>;
pub type HookCallable = Arc<dyn Fn(ShutdownContext) -> HookFuture + Send + Sync>;

pub(crate) struct RegisteredHook {
    pub stage: HookStage,
    pub name: String,
    pub sequence: usize,
    pub call: HookCallable,
}

impl RegisteredHook {
    pub fn new<F, Fut>(stage: HookStage, name: String, sequence: usize, hook: F) -> Self
    where
        F: Fn(ShutdownContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), String>> + Send + 'static,
    {
        Self {
            stage,
            name,
            sequence,
            call: Arc::new(move |context| Box::pin(hook(context))),
        }
    }

    pub fn completed(&self) -> HookOutcome {
        self.outcome(HookStatus::Completed, None)
    }

    pub fn failed(&self, error: String) -> HookOutcome {
        self.outcome(HookStatus::Failed, Some(error))
    }

    pub fn timed_out(&self) -> HookOutcome {
        self.outcome(HookStatus::TimedOut, None)
    }

    fn outcome(&self, status: HookStatus, error: Option<String>) -> HookOutcome {
        HookOutcome {
            stage: self.stage,
            name: self.name.clone(),
            status,
            error,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseTiming {
    pub phase: LifecyclePhase,
    pub started_at: Instant,
    pub finished_at: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShutdownReport {
    pub initiating_generation: u64,
    pub initiating_reason_code: String,
    pub initiating_detail: String,
    pub phase_timings: Vec<PhaseTiming>,
    pub outcomes: Vec<HookOutcome>,
    pub terminal_phase: LifecyclePhase,
    pub started_at: Instant,
    pub finished_at: Instant,
    pub remaining_reserve: Duration,
}
