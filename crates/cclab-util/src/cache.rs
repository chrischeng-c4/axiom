//! LRU cache with optional TTL (time-to-live) support.
//!
//! A thread-safe, pure Rust LRU cache that evicts least-recently-used entries
//! when `max_size` is exceeded, and optionally expires entries after a
//! configurable TTL.
//!
//! # Examples
//!
//! ```
//! use cclab_util::cache::LruCache;
//!
//! let mut cache = LruCache::new(3);
//! cache.put("a", 1);
//! cache.put("b", 2);
//! cache.put("c", 3);
//!
//! assert_eq!(cache.get(&"a"), Some(&1));
//!
//! // Adding a 4th item evicts the LRU entry ("b", since "a" was just accessed)
//! cache.put("d", 4);
//! assert_eq!(cache.get(&"b"), None);
//! ```

use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

/// An entry in the LRU cache.
struct CacheEntry<V> {
    value: V,
    inserted_at: Instant,
    /// Position in the access order (higher = more recent)
    access_order: u64,
}

/// A Least-Recently-Used (LRU) cache with optional TTL.
///
/// - **max_size**: Maximum number of entries. When exceeded, the
///   least-recently-used entry is evicted.
/// - **ttl**: Optional time-to-live. Entries older than this duration
///   are considered expired and removed on access.
pub struct LruCache<K, V> {
    entries: HashMap<K, CacheEntry<V>>,
    max_size: usize,
    ttl: Option<Duration>,
    access_counter: u64,
}

impl<K: Eq + Hash + Clone, V> LruCache<K, V> {
    /// Create a new LRU cache with the given maximum size and no TTL.
    ///
    /// # Examples
    ///
    /// ```
    /// use cclab_util::cache::LruCache;
    ///
    /// let cache: LruCache<String, i32> = LruCache::new(100);
    /// assert_eq!(cache.len(), 0);
    /// assert_eq!(cache.capacity(), 100);
    /// ```
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(max_size),
            max_size: max_size.max(1),
            ttl: None,
            access_counter: 0,
        }
    }

    /// Create a new LRU cache with the given maximum size and TTL.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use cclab_util::cache::LruCache;
    ///
    /// let cache: LruCache<String, i32> = LruCache::with_ttl(100, Duration::from_secs(60));
    /// ```
    pub fn with_ttl(max_size: usize, ttl: Duration) -> Self {
        Self {
            entries: HashMap::with_capacity(max_size),
            max_size: max_size.max(1),
            ttl: Some(ttl),
            access_counter: 0,
        }
    }

    /// Insert a key-value pair. If the key exists, the value is updated.
    ///
    /// If the cache is full, the least-recently-used entry is evicted.
    ///
    /// Returns the old value if the key was already present.
    pub fn put(&mut self, key: K, value: V) -> Option<V> {
        self.access_counter += 1;
        let order = self.access_counter;

        if let Some(entry) = self.entries.get_mut(&key) {
            entry.access_order = order;
            entry.inserted_at = Instant::now();
            let old = std::mem::replace(&mut entry.value, value);
            return Some(old);
        }

        // Evict if at capacity
        if self.entries.len() >= self.max_size {
            self.evict_lru();
        }

        self.entries.insert(
            key,
            CacheEntry {
                value,
                inserted_at: Instant::now(),
                access_order: order,
            },
        );

        None
    }

    /// Look up a key, returning a reference to its value if present and not expired.
    ///
    /// This counts as an access for LRU ordering.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        // Check TTL first
        if let Some(ttl) = self.ttl {
            if let Some(entry) = self.entries.get(key) {
                if entry.inserted_at.elapsed() > ttl {
                    self.entries.remove(key);
                    return None;
                }
            }
        }

        self.access_counter += 1;
        let order = self.access_counter;

        if let Some(entry) = self.entries.get_mut(key) {
            entry.access_order = order;
            Some(&entry.value)
        } else {
            None
        }
    }

    /// Look up a key, returning a mutable reference to its value.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        // Check TTL first
        if let Some(ttl) = self.ttl {
            if let Some(entry) = self.entries.get(key) {
                if entry.inserted_at.elapsed() > ttl {
                    self.entries.remove(key);
                    return None;
                }
            }
        }

        self.access_counter += 1;
        let order = self.access_counter;

        if let Some(entry) = self.entries.get_mut(key) {
            entry.access_order = order;
            Some(&mut entry.value)
        } else {
            None
        }
    }

    /// Remove a key from the cache, returning its value if present.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.entries.remove(key).map(|e| e.value)
    }

    /// Check if a key exists and is not expired.
    pub fn contains_key(&self, key: &K) -> bool {
        if let Some(entry) = self.entries.get(key) {
            if let Some(ttl) = self.ttl {
                return entry.inserted_at.elapsed() <= ttl;
            }
            true
        } else {
            false
        }
    }

    /// Return the number of entries (including possibly expired ones).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Return true if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Return the maximum capacity.
    pub fn capacity(&self) -> usize {
        self.max_size
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.access_counter = 0;
    }

    /// Remove all expired entries (if TTL is set).
    pub fn purge_expired(&mut self) {
        if let Some(ttl) = self.ttl {
            self.entries
                .retain(|_, entry| entry.inserted_at.elapsed() <= ttl);
        }
    }

    /// Return all keys currently in the cache.
    pub fn keys(&self) -> Vec<&K> {
        self.entries.keys().collect()
    }

    /// Evict the least-recently-used entry.
    fn evict_lru(&mut self) {
        // Also evict expired entries first if TTL is set
        if self.ttl.is_some() {
            self.purge_expired();
            if self.entries.len() < self.max_size {
                return;
            }
        }

        // Find the entry with the smallest access_order
        let lru_key = self
            .entries
            .iter()
            .min_by_key(|(_, e)| e.access_order)
            .map(|(k, _)| k.clone());

        if let Some(key) = lru_key {
            self.entries.remove(&key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_put_get() {
        let mut cache = LruCache::new(3);
        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);

        assert_eq!(cache.get(&"a"), Some(&1));
        assert_eq!(cache.get(&"b"), Some(&2));
        assert_eq!(cache.get(&"c"), Some(&3));
        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn test_eviction() {
        let mut cache = LruCache::new(2);
        cache.put("a", 1);
        cache.put("b", 2);

        // Access "a" so "b" becomes LRU
        assert_eq!(cache.get(&"a"), Some(&1));

        // Adding "c" should evict "b"
        cache.put("c", 3);
        assert_eq!(cache.get(&"b"), None);
        assert_eq!(cache.get(&"a"), Some(&1));
        assert_eq!(cache.get(&"c"), Some(&3));
    }

    #[test]
    fn test_update_existing() {
        let mut cache = LruCache::new(2);
        cache.put("a", 1);
        let old = cache.put("a", 10);
        assert_eq!(old, Some(1));
        assert_eq!(cache.get(&"a"), Some(&10));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_remove() {
        let mut cache = LruCache::new(3);
        cache.put("a", 1);
        cache.put("b", 2);

        assert_eq!(cache.remove(&"a"), Some(1));
        assert_eq!(cache.get(&"a"), None);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_contains_key() {
        let mut cache = LruCache::new(3);
        cache.put("a", 1);

        assert!(cache.contains_key(&"a"));
        assert!(!cache.contains_key(&"b"));
    }

    #[test]
    fn test_clear() {
        let mut cache = LruCache::new(3);
        cache.put("a", 1);
        cache.put("b", 2);
        cache.clear();

        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
        assert_eq!(cache.get(&"a"), None);
    }

    #[test]
    fn test_get_mut() {
        let mut cache = LruCache::new(3);
        cache.put("a", 1);

        if let Some(val) = cache.get_mut(&"a") {
            *val = 42;
        }
        assert_eq!(cache.get(&"a"), Some(&42));
    }

    #[test]
    fn test_ttl_expiration() {
        let mut cache = LruCache::with_ttl(10, Duration::from_millis(50));
        cache.put("a", 1);

        assert_eq!(cache.get(&"a"), Some(&1));

        // Sleep past TTL
        std::thread::sleep(Duration::from_millis(60));

        assert_eq!(cache.get(&"a"), None);
    }

    #[test]
    fn test_ttl_purge_expired() {
        let mut cache = LruCache::with_ttl(10, Duration::from_millis(50));
        cache.put("a", 1);
        cache.put("b", 2);

        std::thread::sleep(Duration::from_millis(60));

        cache.put("c", 3);
        cache.purge_expired();

        assert_eq!(cache.len(), 1);
        assert_eq!(cache.get(&"c"), Some(&3));
    }

    #[test]
    fn test_max_size_minimum() {
        // max_size is clamped to at least 1
        let mut cache = LruCache::new(0);
        cache.put("a", 1);
        assert_eq!(cache.get(&"a"), Some(&1));
        assert_eq!(cache.capacity(), 1);
    }

    #[test]
    fn test_keys() {
        let mut cache = LruCache::new(5);
        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);

        let mut keys: Vec<&&str> = cache.keys();
        keys.sort();
        assert_eq!(keys, vec![&"a", &"b", &"c"]);
    }
}
