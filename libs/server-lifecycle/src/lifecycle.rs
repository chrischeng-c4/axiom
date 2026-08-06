use std::fmt;
use std::sync::{Arc, Mutex};

use tokio::sync::watch;
use tokio::time::Instant;

use crate::deadline::ShutdownDeadline;
use crate::hooks::{HookStage, PhaseTiming, ShutdownContext, ShutdownReport};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecyclePhase {
    Starting,
    Recovering,
    Serving,
    Degraded,
    Draining,
    Stopping,
    Stopped,
    Fatal,
}

impl LifecyclePhase {
    pub fn is_draining_or_later(self) -> bool {
        matches!(
            self,
            Self::Draining | Self::Stopping | Self::Stopped | Self::Fatal
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleObservation {
    pub phase: LifecyclePhase,
    pub generation: u64,
    pub transitioned_at: Instant,
    pub reason_code: String,
    pub detail: String,
}

impl LifecycleObservation {
    pub fn startup_succeeded(&self) -> bool {
        matches!(
            self.phase,
            LifecyclePhase::Serving
                | LifecyclePhase::Degraded
                | LifecyclePhase::Draining
                | LifecyclePhase::Stopping
                | LifecyclePhase::Stopped
        )
    }

    pub fn is_healthy(&self) -> bool {
        matches!(
            self.phase,
            LifecyclePhase::Starting
                | LifecyclePhase::Recovering
                | LifecyclePhase::Serving
                | LifecyclePhase::Degraded
                | LifecyclePhase::Draining
                | LifecyclePhase::Stopping
        )
    }

    pub fn is_ready(&self) -> bool {
        matches!(
            self.phase,
            LifecyclePhase::Serving | LifecyclePhase::Degraded
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum LifecycleError {
    #[error("invalid lifecycle transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: LifecyclePhase,
        to: LifecyclePhase,
    },
    #[error("lifecycle hook registration is closed")]
    RegistrationClosed,
}

struct Inner {
    observation: Mutex<LifecycleObservation>,
    watch: watch::Sender<LifecycleObservation>,
    hooks: Mutex<HookRegistry>,
    completion: watch::Sender<Option<Arc<ShutdownReport>>>,
    shutdown_started: Mutex<bool>,
}

struct HookRegistry {
    closed: bool,
    next: usize,
    hooks: Vec<crate::hooks::RegisteredHook>,
}

#[derive(Clone)]
pub struct LifecycleController {
    inner: Arc<Inner>,
}

impl fmt::Debug for LifecycleController {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LifecycleController")
            .field("observation", &self.observation())
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct LifecycleSubscription {
    pub(crate) rx: watch::Receiver<LifecycleObservation>,
}

impl LifecycleSubscription {
    pub fn observation(&self) -> LifecycleObservation {
        self.rx.borrow().clone()
    }

    pub async fn changed(&mut self) -> LifecycleObservation {
        let _ = self.rx.changed().await;
        self.observation()
    }
}

impl LifecycleController {
    pub fn new() -> Self {
        Self::with_phase(LifecyclePhase::Starting, "starting", "controller created")
    }

    pub fn serving() -> Self {
        Self::with_phase(LifecyclePhase::Serving, "serving", "admission open")
    }

    fn with_phase(phase: LifecyclePhase, reason: &str, detail: &str) -> Self {
        let observation = LifecycleObservation {
            phase,
            generation: 0,
            transitioned_at: Instant::now(),
            reason_code: reason.into(),
            detail: detail.into(),
        };
        let (watch, _) = watch::channel(observation.clone());
        let (completion, _) = watch::channel(None);
        Self {
            inner: Arc::new(Inner {
                observation: Mutex::new(observation),
                watch,
                hooks: Mutex::new(HookRegistry {
                    closed: false,
                    next: 0,
                    hooks: Vec::new(),
                }),
                completion,
                shutdown_started: Mutex::new(false),
            }),
        }
    }

    pub fn observation(&self) -> LifecycleObservation {
        self.inner.observation.lock().unwrap().clone()
    }

    pub fn subscribe(&self) -> LifecycleSubscription {
        LifecycleSubscription {
            rx: self.inner.watch.subscribe(),
        }
    }

    pub fn transition(
        &self,
        phase: LifecyclePhase,
        reason_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<LifecycleObservation, LifecycleError> {
        let mut current = self.inner.observation.lock().unwrap();
        if current.phase == phase {
            return Ok(current.clone());
        }
        if !valid_edge(current.phase, phase) {
            return Err(LifecycleError::InvalidTransition {
                from: current.phase,
                to: phase,
            });
        }
        *current = LifecycleObservation {
            phase,
            generation: current.generation + 1,
            transitioned_at: Instant::now(),
            reason_code: reason_code.into(),
            detail: detail.into(),
        };
        self.inner.watch.send_replace(current.clone());
        Ok(current.clone())
    }

    pub fn register_hook<F, Fut>(
        &self,
        stage: HookStage,
        name: impl Into<String>,
        hook: F,
    ) -> Result<(), LifecycleError>
    where
        F: Fn(ShutdownContext) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<(), String>> + Send + 'static,
    {
        let mut registry = self.inner.hooks.lock().unwrap();
        if registry.closed {
            return Err(LifecycleError::RegistrationClosed);
        }
        let sequence = registry.next;
        registry.next += 1;
        registry.hooks.push(crate::hooks::RegisteredHook::new(
            stage,
            name.into(),
            sequence,
            hook,
        ));
        Ok(())
    }

    pub async fn shutdown(
        &self,
        deadline: ShutdownDeadline,
        reason_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Arc<ShutdownReport> {
        let reason_code = reason_code.into();
        let detail = detail.into();
        let first = {
            let mut started = self.inner.shutdown_started.lock().unwrap();
            if *started {
                false
            } else {
                *started = true;
                true
            }
        };
        if first {
            let hooks = {
                let mut registry = self.inner.hooks.lock().unwrap();
                registry.closed = true;
                let mut hooks = std::mem::take(&mut registry.hooks);
                hooks.sort_by_key(|hook| (hook.stage, hook.sequence));
                hooks
            };
            let initial = self.observation();
            if !initial.phase.is_draining_or_later() {
                let _ = self.transition(LifecyclePhase::Draining, &reason_code, &detail);
            }
            let controller = self.clone();
            tokio::spawn(async move {
                let report = Arc::new(
                    controller
                        .run_shutdown(deadline, reason_code, detail, hooks)
                        .await,
                );
                controller.inner.completion.send_replace(Some(report));
            });
        }
        let mut receiver = self.inner.completion.subscribe();
        loop {
            if let Some(report) = receiver.borrow().clone() {
                return report;
            }
            if receiver.changed().await.is_err() {
                unreachable!("completion sender is owned by lifecycle controller");
            }
        }
    }

    async fn run_shutdown(
        &self,
        deadline: ShutdownDeadline,
        reason_code: String,
        detail: String,
        hooks: Vec<crate::hooks::RegisteredHook>,
    ) -> ShutdownReport {
        let started_at = Instant::now();
        let initiating_generation = self.observation().generation;
        let mut outcomes = Vec::with_capacity(hooks.len());
        for hook in hooks {
            let usable = deadline.usable_remaining();
            if usable.is_zero() {
                outcomes.push(hook.timed_out());
                continue;
            }
            let context = ShutdownContext { deadline };
            // Run each user future in an owned task. A panic is converted into
            // a failed outcome instead of aborting the runner, while timeout
            // explicitly aborts the task so no user future is detached.
            let call = hook.call.clone();
            let mut task = tokio::spawn(async move { (call)(context).await });
            let outcome = match tokio::time::timeout(usable, &mut task).await {
                Ok(Ok(Ok(()))) => hook.completed(),
                Ok(Ok(Err(error))) => hook.failed(error),
                Ok(Err(join_error)) if join_error.is_panic() => {
                    hook.failed("lifecycle hook panicked".into())
                }
                Ok(Err(_)) => hook.failed("lifecycle hook task cancelled".into()),
                Err(_) => {
                    task.abort();
                    let _ = task.await;
                    hook.timed_out()
                }
            };
            outcomes.push(outcome);
        }
        let current = self.observation();
        let stopping_at = Instant::now();
        if current.phase == LifecyclePhase::Draining {
            let _ = self.transition(LifecyclePhase::Stopping, "shutdown", "stopping hooks");
            let _ = self.transition(
                LifecyclePhase::Stopped,
                "shutdown-complete",
                "shutdown complete",
            );
        }
        let finished_at = Instant::now();
        ShutdownReport {
            initiating_generation,
            initiating_reason_code: reason_code,
            initiating_detail: detail,
            phase_timings: vec![
                PhaseTiming {
                    phase: LifecyclePhase::Draining,
                    started_at,
                    finished_at: stopping_at,
                },
                PhaseTiming {
                    phase: LifecyclePhase::Stopping,
                    started_at: stopping_at,
                    finished_at,
                },
            ],
            outcomes,
            terminal_phase: self.observation().phase,
            started_at,
            finished_at,
            remaining_reserve: deadline.remaining().min(deadline.reserve),
        }
    }
}

impl Default for LifecycleController {
    fn default() -> Self {
        Self::new()
    }
}

fn valid_edge(from: LifecyclePhase, to: LifecyclePhase) -> bool {
    matches!(
        (from, to),
        (
            LifecyclePhase::Starting,
            LifecyclePhase::Recovering
                | LifecyclePhase::Serving
                | LifecyclePhase::Degraded
                | LifecyclePhase::Draining
                | LifecyclePhase::Fatal
        ) | (
            LifecyclePhase::Recovering,
            LifecyclePhase::Serving
                | LifecyclePhase::Degraded
                | LifecyclePhase::Draining
                | LifecyclePhase::Fatal
        ) | (
            LifecyclePhase::Serving,
            LifecyclePhase::Degraded | LifecyclePhase::Draining | LifecyclePhase::Fatal
        ) | (
            LifecyclePhase::Degraded,
            LifecyclePhase::Serving | LifecyclePhase::Draining | LifecyclePhase::Fatal
        ) | (
            LifecyclePhase::Draining,
            LifecyclePhase::Stopping | LifecyclePhase::Fatal
        ) | (
            LifecyclePhase::Stopping,
            LifecyclePhase::Stopped | LifecyclePhase::Fatal
        )
    )
}
