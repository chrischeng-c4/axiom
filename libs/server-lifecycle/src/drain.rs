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

    // <HANDWRITE gap="missing-generator:logic" tracker="#1884" reason="logic section in drain.rs is hand-written pending codegen support">
    pub fn start_drain(&self) {
        // `watch::Sender::send` declines to publish when every receiver has
        // been dropped. Drain is process state, not a best-effort event: a
        // SIGTERM that arrives before a plane subscribes must still be seen by
        // that plane when it starts.
        self.tx.send_replace(DrainState::Draining);
    }
    // </HANDWRITE>
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
        // A receiver subscribed after drain has already begun treats the
        // current watch value as seen. Check it first so startup shutdown
        // futures cannot miss an earlier SIGTERM/drain transition.
        if !self.is_draining() {
            let _ = self.rx.changed().await;
        }
        self.state()
    }
}
