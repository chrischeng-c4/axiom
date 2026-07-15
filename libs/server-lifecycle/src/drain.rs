use tokio::sync::watch;

/// Server admission state.
/// @spec apps/agentic-workflow/tech-design/logic/shared-server-substrate-performance-layers.md#logic
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

/// Write-side drain controller owned by the server runtime.
#[derive(Debug, Clone)]
pub struct DrainController {
    tx: watch::Sender<DrainState>,
}

impl DrainController {
    pub fn new() -> Self {
        let (tx, _rx) = watch::channel(DrainState::Ready);
        Self { tx }
    }

    pub fn signal(&self) -> DrainSignal {
        DrainSignal {
            rx: self.tx.subscribe(),
        }
    }

    pub fn state(&self) -> DrainState {
        *self.tx.borrow()
    }

    pub fn is_draining(&self) -> bool {
        self.state().is_draining()
    }

    pub fn start_drain(&self) {
        let _ = self.tx.send(DrainState::Draining);
    }
}

impl Default for DrainController {
    fn default() -> Self {
        Self::new()
    }
}

/// Read-side drain signal cloned into accept loops and connection tasks.
#[derive(Debug, Clone)]
pub struct DrainSignal {
    rx: watch::Receiver<DrainState>,
}

impl DrainSignal {
    pub fn state(&self) -> DrainState {
        *self.rx.borrow()
    }

    pub fn is_draining(&self) -> bool {
        self.state().is_draining()
    }

    pub async fn changed(&mut self) -> DrainState {
        let _ = self.rx.changed().await;
        self.state()
    }
}
