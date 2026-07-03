//! `OutcomeWindow<T>` — a bounded index→outcome retention map.
//!
//! Every raft_core write path needs to hand the *rich* result of an apply
//! back to the handler that proposed it: `raft_host::RaftHost::propose`
//! already gives read-your-write on the committed `Index`, but the payload
//! (the outcome the local state machine computed while applying) has to be
//! stashed somewhere in the interim. This is that stash: a small
//! `BTreeMap<Index, T>` that retains only the most recent `capacity` entries,
//! so a value nobody ever claims (a write that originated on another node)
//! ages out instead of growing the map forever.
//!
//! Not thread-safe on its own — callers hold it behind their own `Mutex`
//! alongside whatever else needs the same critical section (lumen's
//! `WriteCoordinator`, for instance, also guards a waiter map in the same
//! lock).

use std::collections::BTreeMap;

/// The retention window's default size, carried over from lumen's original
/// `OUTCOME_WINDOW` constant: far larger than the sub-millisecond gap between
/// an apply landing and its local claimant reading it, so only outcomes
/// nobody was waiting on (writes that originated on other nodes) age out.
pub const DEFAULT_CAPACITY: u64 = 8192;

/// A bounded `index -> T` retention map for read-your-write outcome claiming.
///
/// Usage is always `insert` then `advance` with the same index (the index
/// just inserted), mirroring "insert before advancing applied" — the
/// external "applied" cursor a caller tracks alongside this window should
/// only move forward *after* both calls, so a reader never observes an
/// applied index whose outcome isn't in the window yet.
pub struct OutcomeWindow<T> {
    capacity: u64,
    entries: BTreeMap<u64, T>,
}

impl<T> OutcomeWindow<T> {
    /// A window retaining at most `capacity` recent indices (entries at or
    /// above `index - capacity` survive an `advance(index)`).
    pub fn new(capacity: u64) -> Self {
        Self {
            capacity,
            entries: BTreeMap::new(),
        }
    }

    /// Store `value` at `index`, overwriting any existing entry there.
    pub fn insert(&mut self, index: u64, value: T) {
        self.entries.insert(index, value);
    }

    /// Evict every entry strictly below `index.saturating_sub(capacity)` —
    /// the retention window's cutoff as of `index`. Call with the same
    /// index just passed to `insert` so the entry that was just inserted is
    /// never itself evicted by its own advance.
    pub fn advance(&mut self, index: u64) {
        let cutoff = index.saturating_sub(self.capacity);
        while let Some((&k, _)) = self.entries.iter().next() {
            if k < cutoff {
                self.entries.remove(&k);
            } else {
                break;
            }
        }
    }

    /// Remove and return the value at `index`, if it's still in the window
    /// (claiming twice, or claiming after eviction, returns `None`).
    pub fn claim(&mut self, index: u64) -> Option<T> {
        self.entries.remove(&index)
    }
}

impl<T> Default for OutcomeWindow<T> {
    /// A window sized to [`DEFAULT_CAPACITY`].
    fn default() -> Self {
        Self::new(DEFAULT_CAPACITY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_then_claim_round_trips() {
        let mut w: OutcomeWindow<&str> = OutcomeWindow::new(4);
        w.insert(1, "a");
        w.advance(1);
        assert_eq!(w.claim(1), Some("a"));
        // Claiming again returns None — the entry is gone.
        assert_eq!(w.claim(1), None);
    }

    #[test]
    fn advance_evicts_strictly_below_cutoff_inclusive_boundary_survives() {
        let mut w: OutcomeWindow<u64> = OutcomeWindow::new(4);
        for i in 1..=4 {
            w.insert(i, i * 10);
            w.advance(i);
        }
        // capacity 4, advanced to 4 → cutoff = 0, nothing evicted yet.
        for i in 1..=4 {
            assert_eq!(w.claim(i), Some(i * 10), "index {i} should survive");
        }
    }

    #[test]
    fn advance_evicts_entries_below_the_cutoff() {
        let mut w: OutcomeWindow<u64> = OutcomeWindow::new(4);
        for i in 1..=10 {
            w.insert(i, i * 10);
            w.advance(i);
        }
        // cutoff = 10 - 4 = 6, so indices < 6 are evicted; 6..=10 survive.
        for i in 1..6 {
            assert_eq!(w.claim(i), None, "index {i} should have been evicted");
        }
        for i in 6..=10 {
            assert_eq!(w.claim(i), Some(i * 10), "index {i} should survive");
        }
    }

    #[test]
    fn claim_after_evict_returns_none() {
        let mut w: OutcomeWindow<&str> = OutcomeWindow::new(2);
        w.insert(1, "first");
        w.advance(1);
        w.insert(2, "second");
        w.advance(2);
        w.insert(3, "third");
        w.advance(3);
        // cutoff = 4 - 2 = 2 once index 4 lands → index 1 evicted, 2 survives.
        w.insert(4, "fourth");
        w.advance(4);
        assert_eq!(w.claim(1), None, "evicted entries stay gone");
        assert_eq!(w.claim(2), Some("second"));
        assert_eq!(w.claim(4), Some("fourth"));
    }

    #[test]
    fn default_uses_the_documented_capacity() {
        let mut w: OutcomeWindow<u64> = OutcomeWindow::default();
        w.insert(DEFAULT_CAPACITY, 1);
        w.advance(DEFAULT_CAPACITY);
        // cutoff = DEFAULT_CAPACITY - DEFAULT_CAPACITY = 0, so index
        // DEFAULT_CAPACITY (>= 0) survives.
        assert_eq!(w.claim(DEFAULT_CAPACITY), Some(1));

        w.insert(1, 2);
        w.insert(DEFAULT_CAPACITY + 1, 3);
        w.advance(DEFAULT_CAPACITY + 1);
        // cutoff = (DEFAULT_CAPACITY + 1) - DEFAULT_CAPACITY = 1, so index 1
        // itself survives (not strictly below cutoff) but nothing lower would.
        assert_eq!(w.claim(1), Some(2));
        assert_eq!(w.claim(DEFAULT_CAPACITY + 1), Some(3));
    }
}
