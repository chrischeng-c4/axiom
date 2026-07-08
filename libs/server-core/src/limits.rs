use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Shared connection budget with RAII accounting.
/// @spec projects/agentic-workflow/tech-design/logic/shared-server-substrate-performance-layers.md#logic
#[derive(Debug, Clone)]
pub struct ConnectionBudget {
    inner: Arc<ConnectionBudgetInner>,
}

#[derive(Debug)]
struct ConnectionBudgetInner {
    max: usize,
    active: AtomicUsize,
}

impl ConnectionBudget {
    pub fn new(max: usize) -> Self {
        Self {
            inner: Arc::new(ConnectionBudgetInner {
                max: max.max(1),
                active: AtomicUsize::new(0),
            }),
        }
    }

    pub fn max(&self) -> usize {
        self.inner.max
    }

    pub fn active(&self) -> usize {
        self.inner.active.load(Ordering::Relaxed)
    }

    pub fn available(&self) -> usize {
        self.max().saturating_sub(self.active())
    }

    pub fn try_acquire(&self) -> Result<ConnectionPermit, ConnectionLimitExceeded> {
        loop {
            let active = self.inner.active.load(Ordering::Acquire);
            if active >= self.inner.max {
                return Err(ConnectionLimitExceeded {
                    max: self.inner.max,
                    active,
                });
            }
            if self
                .inner
                .active
                .compare_exchange(active, active + 1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Ok(ConnectionPermit {
                    budget: self.clone(),
                });
            }
        }
    }
}

/// Permit held for the lifetime of an accepted connection.
#[derive(Debug)]
pub struct ConnectionPermit {
    budget: ConnectionBudget,
}

impl Drop for ConnectionPermit {
    fn drop(&mut self) {
        self.budget.inner.active.fetch_sub(1, Ordering::AcqRel);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("connection limit exceeded: active={active} max={max}")]
pub struct ConnectionLimitExceeded {
    pub max: usize,
    pub active: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permit_tracks_active_connections() {
        // @spec projects/agentic-workflow/tech-design/logic/shared-server-substrate-performance-layers.md#unit-test
        let budget = ConnectionBudget::new(1);
        let permit = budget.try_acquire().expect("first permit");
        assert_eq!(budget.active(), 1);
        assert!(budget.try_acquire().is_err());
        drop(permit);
        assert_eq!(budget.active(), 0);
        assert!(budget.try_acquire().is_ok());
    }
}
