// HANDWRITE-BEGIN gap="missing-generator:logic:k8s-decision-cache" tracker="#2869" reason="A bounded TTL cache whose expiry semantics encode a fail-closed revocation budget; no generator primitive models a stale-only-on-outage window."
//! How long a delegated answer may be reused, and what happens when the
//! apiserver stops answering.
//!
//! Every cached decision is a small window during which a revocation has not
//! taken effect yet. That window is the entire security cost of caching, so it
//! is stated as a number rather than left to emerge: an allow survives
//! [`CachePolicy::allow_ttl`], a deny survives the much shorter
//! [`CachePolicy::deny_ttl`] (a deny that outlives a freshly granted RoleBinding
//! is an availability bug, not a safety one), and when the apiserver is
//! unreachable an already-expired entry may be served for at most
//! [`CachePolicy::stale_window`] beyond that.
//!
//! The stale window is deliberately not reachable on the happy path. [`get`]
//! never returns an expired entry; only [`get_stale`] does, and the caller may
//! only reach for it after a review actually failed. That is what keeps the
//! worst case bounded at `ttl + stale_window` instead of "until the apiserver
//! comes back".
//!
//! [`get`]: TtlCache::get
//! [`get_stale`]: TtlCache::get_stale

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Monotonically-read wall time, in milliseconds.
///
/// Injectable because the properties worth testing here are all temporal:
/// "a revoked allow stops working within six minutes" is only a test if time
/// can be moved without waiting for it.
pub trait Clock: Send + Sync + 'static {
    fn now_millis(&self) -> u64;
}

/// The process clock.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_millis(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

/// A clock a test drives by hand.
///
/// Public because the services that build on this cache have to prove their own
/// revocation bounds, and a private test clock would force each of them to
/// reinvent one.
#[derive(Debug, Default)]
pub struct ManualClock {
    millis: AtomicU64,
}

impl ManualClock {
    pub fn new(millis: u64) -> Self {
        Self {
            millis: AtomicU64::new(millis),
        }
    }

    pub fn advance(&self, by: Duration) {
        self.millis
            .fetch_add(by.as_millis() as u64, Ordering::SeqCst);
    }
}

impl Clock for ManualClock {
    fn now_millis(&self) -> u64 {
        self.millis.load(Ordering::SeqCst)
    }
}

/// How long each class of answer is reusable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CachePolicy {
    /// How long a successful authentication or an allow may be reused.
    pub allow_ttl: Duration,
    /// How long a deny may be reused. Short on purpose: this TTL is the delay
    /// between granting access and access working.
    pub deny_ttl: Duration,
    /// How far past its TTL an entry may be served *only* when the apiserver
    /// failed to answer. Zero disables serving stale entries entirely.
    pub stale_window: Duration,
    /// The hard ceiling on retained entries. An unauthenticated caller chooses
    /// the token, and therefore the key, so an unbounded map is a memory
    /// exhaustion primitive handed to anyone who can reach the port.
    pub max_entries: usize,
}

impl Default for CachePolicy {
    fn default() -> Self {
        Self {
            allow_ttl: Duration::from_secs(300),
            deny_ttl: Duration::from_secs(30),
            stale_window: Duration::from_secs(60),
            max_entries: 8192,
        }
    }
}

impl CachePolicy {
    /// The worst-case delay between a revocation taking effect in Kubernetes
    /// and this process refusing the caller.
    pub fn revocation_bound(&self) -> Duration {
        self.allow_ttl + self.stale_window
    }
}

/// What a lookup found, and how much it should be trusted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheOutcome {
    /// Served from an unexpired entry.
    Hit,
    /// Nothing usable; the caller must ask the apiserver.
    Miss,
    /// Served from an expired entry because the apiserver could not be
    /// reached, and the entry is still inside the stale window.
    Stale,
}

impl CacheOutcome {
    /// A stable label for metrics.
    pub fn label(self) -> &'static str {
        match self {
            Self::Hit => "hit",
            Self::Miss => "miss",
            Self::Stale => "stale",
        }
    }
}

#[derive(Debug, Clone)]
struct Entry<V> {
    value: V,
    /// Absolute expiry. Past this, only [`TtlCache::get_stale`] can see it.
    expires_at_millis: u64,
    /// Absolute end of the stale window. Past this the entry is unreachable.
    discard_at_millis: u64,
}

/// A bounded, TTL-keyed cache with a separate stale-on-failure read path.
pub struct TtlCache<K, V> {
    entries: Mutex<HashMap<K, Entry<V>>>,
    policy: CachePolicy,
    clock: Arc<dyn Clock>,
}

impl<K, V> TtlCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new(policy: CachePolicy, clock: Arc<dyn Clock>) -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
            policy,
            clock,
        }
    }

    pub fn policy(&self) -> &CachePolicy {
        &self.policy
    }

    /// Store a value under a TTL chosen by the caller — `allow_ttl` or
    /// `deny_ttl`, depending on what the answer was.
    pub fn insert(&self, key: K, value: V, ttl: Duration) {
        let now = self.clock.now_millis();
        let expires_at_millis = now.saturating_add(ttl.as_millis() as u64);
        let discard_at_millis =
            expires_at_millis.saturating_add(self.policy.stale_window.as_millis() as u64);
        let mut entries = self.lock();
        entries.insert(
            key,
            Entry {
                value,
                expires_at_millis,
                discard_at_millis,
            },
        );
        Self::enforce_capacity(&mut entries, self.policy.max_entries, now);
    }

    /// The fast path: an unexpired entry, or nothing. Never returns an expired
    /// entry, so no caller can drift into serving stale data by accident.
    pub fn get(&self, key: &K) -> Option<V> {
        let now = self.clock.now_millis();
        let mut entries = self.lock();
        let entry = entries.get(key)?;
        if now < entry.expires_at_millis {
            return Some(entry.value.clone());
        }
        if now >= entry.discard_at_millis {
            entries.remove(key);
        }
        None
    }

    /// The outage path: an entry that is past its TTL but still inside the
    /// stale window. Only legitimate after a review actually failed — calling
    /// it otherwise silently widens the revocation bound.
    pub fn get_stale(&self, key: &K) -> Option<V> {
        let now = self.clock.now_millis();
        let mut entries = self.lock();
        let entry = entries.get(key)?;
        if now < entry.discard_at_millis {
            return Some(entry.value.clone());
        }
        entries.remove(key);
        None
    }

    pub fn remove(&self, key: &K) {
        self.lock().remove(key);
    }

    pub fn clear(&self) {
        self.lock().clear();
    }

    /// Retained entries, including ones only `get_stale` can still see.
    pub fn len(&self) -> usize {
        self.lock().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// A poisoned mutex here means a panic inside a cache method, which cannot
    /// leave a `HashMap` in a state that threatens correctness. Recovering
    /// beats propagating a panic into every subsequent request.
    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<K, Entry<V>>> {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Drop fully-discardable entries first; only if that is not enough does
    /// the cache evict live ones, oldest expiry first.
    fn enforce_capacity(entries: &mut HashMap<K, Entry<V>>, max_entries: usize, now: u64) {
        if entries.len() <= max_entries {
            return;
        }
        entries.retain(|_, entry| now < entry.discard_at_millis);
        while entries.len() > max_entries {
            let Some(victim) = entries
                .iter()
                .min_by_key(|(_, entry)| entry.expires_at_millis)
                .map(|(key, _)| key.clone())
            else {
                break;
            };
            entries.remove(&victim);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    fn cache(policy: CachePolicy) -> (TtlCache<String, bool>, Arc<ManualClock>) {
        let clock = Arc::new(ManualClock::new(1_000_000));
        (TtlCache::new(policy, clock.clone()), clock)
    }

    #[test]
    fn a_fresh_entry_is_served_and_an_expired_one_is_not() {
        let (cache, clock) = cache(CachePolicy::default());
        cache.insert("k".into(), true, Duration::from_secs(300));
        assert_eq!(cache.get(&"k".to_string()), Some(true));

        clock.advance(Duration::from_secs(299));
        assert_eq!(
            cache.get(&"k".to_string()),
            Some(true),
            "one second before expiry the entry is still fresh"
        );

        clock.advance(Duration::from_secs(2));
        assert_eq!(
            cache.get(&"k".to_string()),
            None,
            "the fast path must never serve an expired entry"
        );
    }

    /// The stale window is only reachable through `get_stale`, which is what
    /// makes "we cache" and "we fail closed" both true.
    #[test]
    fn an_expired_entry_is_reachable_only_through_the_outage_path() {
        let (cache, clock) = cache(CachePolicy::default());
        cache.insert("k".into(), true, Duration::from_secs(300));
        clock.advance(Duration::from_secs(301));

        assert_eq!(cache.get(&"k".to_string()), None);
        assert_eq!(
            cache.get_stale(&"k".to_string()),
            Some(true),
            "30s past expiry is inside the 60s stale window"
        );
    }

    /// AC6: the revocation bound is a number, and past it there is no path
    /// back to the cached answer.
    #[test]
    fn past_the_stale_window_even_the_outage_path_fails_closed() {
        let policy = CachePolicy::default();
        let (cache, clock) = cache(policy);
        cache.insert("k".into(), true, policy.allow_ttl);

        clock.advance(policy.revocation_bound());
        clock.advance(Duration::from_millis(1));

        assert_eq!(cache.get(&"k".to_string()), None);
        assert_eq!(
            cache.get_stale(&"k".to_string()),
            None,
            "there is no read path that outlives allow_ttl + stale_window"
        );
        assert_eq!(
            policy.revocation_bound(),
            Duration::from_secs(360),
            "the documented worst case is six minutes"
        );
    }

    /// A deny gets its own, much shorter TTL: it delays a *grant* taking
    /// effect, so long-lived denies are an availability problem.
    #[test]
    fn a_deny_expires_far_sooner_than_an_allow() {
        let policy = CachePolicy::default();
        let (cache, clock) = cache(policy);
        cache.insert("allow".into(), true, policy.allow_ttl);
        cache.insert("deny".into(), false, policy.deny_ttl);

        clock.advance(Duration::from_secs(31));
        assert_eq!(cache.get(&"deny".to_string()), None);
        assert_eq!(cache.get(&"allow".to_string()), Some(true));
    }

    #[test]
    fn a_zero_stale_window_removes_the_outage_path_entirely() {
        let policy = CachePolicy {
            stale_window: Duration::ZERO,
            ..CachePolicy::default()
        };
        let (cache, clock) = cache(policy);
        cache.insert("k".into(), true, policy.allow_ttl);
        clock.advance(policy.allow_ttl);
        clock.advance(Duration::from_millis(1));

        assert_eq!(cache.get_stale(&"k".to_string()), None);
        assert_eq!(policy.revocation_bound(), policy.allow_ttl);
    }

    /// The key is caller-chosen, so the ceiling has to hold under a flood.
    #[test]
    fn the_entry_ceiling_holds_against_an_unbounded_key_space() {
        let policy = CachePolicy {
            max_entries: 16,
            ..CachePolicy::default()
        };
        let (cache, _clock) = cache(policy);
        for i in 0..1_000 {
            cache.insert(format!("k{i}"), true, Duration::from_secs(300));
        }
        assert!(
            cache.len() <= 16,
            "cache grew to {} entries past a ceiling of 16",
            cache.len()
        );
    }

    #[test]
    fn eviction_prefers_entries_nothing_can_read_any_more() {
        let policy = CachePolicy {
            max_entries: 2,
            ..CachePolicy::default()
        };
        let (cache, clock) = cache(policy);
        cache.insert("unreadable".into(), true, Duration::from_secs(1));
        clock.advance(Duration::from_secs(120));

        cache.insert("a".into(), true, Duration::from_secs(300));
        clock.advance(Duration::from_secs(1));
        cache.insert("b".into(), true, Duration::from_secs(300));

        assert_eq!(
            cache.get_stale(&"unreadable".to_string()),
            None,
            "the entry past its stale window is the one the ceiling should reclaim"
        );
        assert_eq!(cache.get(&"a".to_string()), Some(true));
        assert_eq!(cache.get(&"b".to_string()), Some(true));

        // With nothing discardable left, a third entry costs the entry that
        // expires soonest — the one whose remaining value is lowest.
        clock.advance(Duration::from_secs(1));
        cache.insert("c".into(), true, Duration::from_secs(300));
        assert_eq!(cache.get(&"a".to_string()), None);
        assert_eq!(cache.get(&"b".to_string()), Some(true));
        assert_eq!(cache.get(&"c".to_string()), Some(true));
    }

    #[test]
    fn outcome_labels_are_stable() {
        assert_eq!(CacheOutcome::Hit.label(), "hit");
        assert_eq!(CacheOutcome::Miss.label(), "miss");
        assert_eq!(CacheOutcome::Stale.label(), "stale");
    }
}
// HANDWRITE-END
