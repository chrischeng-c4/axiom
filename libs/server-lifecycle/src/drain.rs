use crate::lifecycle::{LifecycleController, LifecyclePhase, LifecycleSubscription};

/// Compatibility admission state projected from the authoritative lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrainState {
    Ready,
    Draining,
}

impl DrainState {
    pub fn is_draining(self) -> bool {
        matches!(self, Self::Draining)
    }
}

#[derive(Debug, Clone)]
pub struct DrainController {
    lifecycle: LifecycleController,
}

impl DrainController {
    pub fn new() -> Self {
        Self {
            lifecycle: LifecycleController::serving(),
        }
    }

    pub fn from_lifecycle(lifecycle: LifecycleController) -> Self {
        Self { lifecycle }
    }

    pub fn lifecycle(&self) -> LifecycleController {
        self.lifecycle.clone()
    }

    pub fn signal(&self) -> DrainSignal {
        DrainSignal {
            subscription: self.lifecycle.subscribe(),
        }
    }

    pub fn state(&self) -> DrainState {
        if self.lifecycle.observation().phase.is_draining_or_later() {
            DrainState::Draining
        } else {
            DrainState::Ready
        }
    }

    pub fn is_draining(&self) -> bool {
        self.state().is_draining()
    }

    pub fn start_drain(&self) {
        let phase = self.lifecycle.observation().phase;
        if !phase.is_draining_or_later() {
            let _ = self.lifecycle.transition(
                LifecyclePhase::Draining,
                "drain",
                "compatibility drain requested",
            );
        }
    }
}

impl Default for DrainController {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct DrainSignal {
    subscription: LifecycleSubscription,
}

impl DrainSignal {
    pub fn state(&self) -> DrainState {
        if self.subscription.observation().phase.is_draining_or_later() {
            DrainState::Draining
        } else {
            DrainState::Ready
        }
    }

    pub fn is_draining(&self) -> bool {
        self.state().is_draining()
    }

    pub async fn changed(&mut self) -> DrainState {
        loop {
            if self.is_draining() {
                return DrainState::Draining;
            }
            let _ = self.subscription.changed().await;
        }
    }
}
