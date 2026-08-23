// CODEGEN-BEGIN
//! Service-shell re-exports for protocol-neutral graceful shutdown.
//!
//! The `shutdown_with_drain` and `wait_shutdown_signal` exports are legacy
//! adapters. Production code should use [`LifecycleShutdownTrigger`] with the
//! caller-owned lifecycle and `run_signal_bridge` (or `shutdown_on_signal`).
//!
//! The drain dance every k8s-native service in the ecosystem repeats: on
//! SIGINT/SIGTERM, flip readiness to draining (so `/readyz` → 503 and k8s stops
//! routing), hold a grace window, then let the listener close. Factored out of
//! lumen's / keep's `shutdown_signal`. Ownership lives in `server-lifecycle`.

pub use server_lifecycle::{shutdown_with_drain, wait_shutdown_signal};
use server_lifecycle::{LifecycleController, ShutdownDeadline, ShutdownReport};
use std::{sync::Arc, time::Duration};

#[derive(Clone)]
pub struct LifecycleShutdownTrigger {
    lifecycle: LifecycleController,
    total: Duration,
    reserve: Duration,
}

impl LifecycleShutdownTrigger {
    pub fn new(
        lifecycle: LifecycleController,
        total: Duration,
        reserve: Duration,
    ) -> Result<Self, server_lifecycle::DeadlineError> {
        ShutdownDeadline::from_now(total, reserve)?;
        Ok(Self {
            lifecycle,
            total,
            reserve,
        })
    }

    pub async fn trigger(
        &self,
        reason_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Arc<ShutdownReport> {
        let deadline = ShutdownDeadline::from_now(self.total, self.reserve)
            .expect("validated shutdown durations");
        self.lifecycle.shutdown(deadline, reason_code, detail).await
    }
}

pub async fn run_signal_bridge<F>(
    trigger: LifecycleShutdownTrigger,
    signal: F,
) -> Arc<ShutdownReport>
where
    F: std::future::Future<Output = ()> + Send,
{
    signal.await;
    trigger.trigger("signal", "shutdown signal received").await
}

/// POSIX/CTRL-C convenience bridge for production binaries.
pub async fn shutdown_on_signal(trigger: LifecycleShutdownTrigger) -> Arc<ShutdownReport> {
    run_signal_bridge(trigger, wait_shutdown_signal()).await
}
// CODEGEN-END
