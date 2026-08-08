// HANDWRITE-BEGIN gap="missing-generator:logic:bf3dae37" tracker="pending-tracker" reason="Own readiness and draining observation."
//! Protocol-neutral readiness observation.

use crate::drain::{DrainController, DrainSignal};
use crate::lifecycle::{LifecycleController, LifecycleObservation, LifecycleSubscription};

/// Reports whether a server is draining and should reject new work.
pub trait Readiness: Send + Sync {
    fn is_draining(&self) -> bool;
}

impl Readiness for LifecycleController {
    fn is_draining(&self) -> bool {
        self.observation().phase.is_draining_or_later()
    }
}

impl Readiness for LifecycleSubscription {
    fn is_draining(&self) -> bool {
        self.observation().phase.is_draining_or_later()
    }
}

pub fn observation_is_healthy(observation: &LifecycleObservation) -> bool {
    observation.is_healthy()
}

impl Readiness for DrainController {
    fn is_draining(&self) -> bool {
        DrainController::is_draining(self)
    }
}

impl Readiness for DrainSignal {
    fn is_draining(&self) -> bool {
        DrainSignal::is_draining(self)
    }
}

#[cfg(test)]
mod tests {
    use super::Readiness;
    use crate::DrainController;

    fn assert_ready(readiness: &dyn Readiness, expected: bool) {
        assert_eq!(readiness.is_draining(), expected);
    }

    #[test]
    fn controller_and_signal_share_drain_state() {
        let controller = DrainController::new();
        let signal = controller.signal();
        assert_ready(&controller, false);
        assert_ready(&signal, false);

        controller.start_drain();
        assert_ready(&controller, true);
        assert_ready(&signal, true);
    }
}
// HANDWRITE-END
