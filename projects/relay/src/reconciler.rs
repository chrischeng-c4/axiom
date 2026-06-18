// SPEC-MANAGED: projects/relay/tech-design/logic/reconciler-lease-reclaim-redeliver-liveness.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:c26b5730" tracker="pending-tracker" reason="Background reconciler: spawn_reconciler(relay, interval) ticks and calls reconcile; ReconcilerHandle to stop it."
//! Background work-queue liveness sweep.
//!
//! A timer task periodically calls [`crate::Relay::reconcile`], reclaiming
//! expired leases so a dead worker's in-flight range is redelivered. The lock
//! is held only for the (synchronous, frontier-only) sweep, never across the
//! timer `.await`.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::Utc;

use crate::engine::Relay;

/// Handle to a running reconciler; drop or [`stop`](ReconcilerHandle::stop) to
/// end the background sweep.
///
/// @spec projects/relay/tech-design/logic/reconciler-lease-reclaim-redeliver-liveness.md#logic
pub struct ReconcilerHandle {
    task: tokio::task::JoinHandle<()>,
}

impl ReconcilerHandle {
    /// Stop the background reconciler.
    pub fn stop(self) {
        self.task.abort();
    }
}

impl Drop for ReconcilerHandle {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// Spawn a background task that reclaims expired leases across all subjects
/// every `interval`.
///
/// @spec projects/relay/tech-design/logic/reconciler-lease-reclaim-redeliver-liveness.md#logic
pub fn spawn_reconciler(relay: Arc<Mutex<Relay>>, interval: Duration) -> ReconcilerHandle {
    let task = tokio::spawn(async move {
        let mut tick = tokio::time::interval(interval);
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tick.tick().await;
            // Synchronous, frontier-only sweep; guard dropped before the next await.
            let _reclaimed = {
                let mut guard = relay.lock().expect("relay mutex");
                guard.reconcile(Utc::now())
            };
        }
    });
    ReconcilerHandle { task }
}
// HANDWRITE-END
