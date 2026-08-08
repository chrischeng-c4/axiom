use std::time::Duration;

use tokio::time::Instant;

/// One absolute shutdown budget shared by every lifecycle participant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShutdownDeadline {
    pub expires_at: Instant,
    pub total: Duration,
    pub reserve: Duration,
}

impl ShutdownDeadline {
    pub fn from_now(total: Duration, reserve: Duration) -> Result<Self, DeadlineError> {
        if reserve > total {
            return Err(DeadlineError::ReserveExceedsTotal { total, reserve });
        }
        Ok(Self {
            expires_at: Instant::now() + total,
            total,
            reserve,
        })
    }

    pub fn remaining(self) -> Duration {
        self.expires_at.saturating_duration_since(Instant::now())
    }

    pub fn usable_remaining(self) -> Duration {
        self.remaining().saturating_sub(self.reserve)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum DeadlineError {
    #[error("shutdown reserve {reserve:?} exceeds total {total:?}")]
    ReserveExceedsTotal { total: Duration, reserve: Duration },
}
