// HANDWRITE-BEGIN gap="missing-generator:logic:raft-runtime-proposal-cache" tracker="#1854" reason="Bounded replicated proposal outcome cache shared by effectful Raft state machines to make ambiguous client retries idempotent."
//! Bounded proposal-id outcome retention for deterministic state machines.
//!
//! A service embeds a stable proposal id in its replicated command, looks it
//! up before applying domain state, and snapshots [`ProposalCache::snapshot`]
//! with the rest of the state machine. This closes the common ambiguity where
//! a follower's request committed on the leader but the response was lost.

use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

pub const DEFAULT_PROPOSAL_CACHE_CAPACITY: usize = 4096;

pub struct ProposalCache<K, V> {
    capacity: usize,
    entries: HashMap<K, V>,
    order: VecDeque<K>,
}

impl<K, V> ProposalCache<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity,
            entries: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    pub fn get(&self, id: &K) -> Option<V> {
        self.entries.get(id).cloned()
    }

    pub fn insert(&mut self, id: K, outcome: V) {
        if self.entries.contains_key(&id) {
            return;
        }
        self.order.push_back(id.clone());
        self.entries.insert(id, outcome);
        while self.order.len() > self.capacity {
            if let Some(oldest) = self.order.pop_front() {
                self.entries.remove(&oldest);
            }
        }
    }

    pub fn snapshot(&self) -> Vec<(K, V)> {
        self.order
            .iter()
            .filter_map(|id| self.entries.get(id).cloned().map(|out| (id.clone(), out)))
            .collect()
    }

    pub fn restore(&mut self, entries: Vec<(K, V)>) {
        self.entries.clear();
        self.order.clear();
        for (id, outcome) in entries {
            self.insert(id, outcome);
        }
    }
}

impl<K, V> Default for ProposalCache<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    fn default() -> Self {
        Self::with_capacity(DEFAULT_PROPOSAL_CACHE_CAPACITY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retains_first_outcome_and_restores_in_order() {
        let mut cache = ProposalCache::with_capacity(2);
        cache.insert("a", 1);
        cache.insert("a", 9);
        cache.insert("b", 2);
        assert_eq!(cache.get(&"a"), Some(1));
        cache.insert("c", 3);
        assert_eq!(cache.get(&"a"), None);

        let snapshot = cache.snapshot();
        let mut restored = ProposalCache::with_capacity(2);
        restored.restore(snapshot);
        assert_eq!(restored.get(&"b"), Some(2));
        assert_eq!(restored.get(&"c"), Some(3));
    }
}
// HANDWRITE-END
