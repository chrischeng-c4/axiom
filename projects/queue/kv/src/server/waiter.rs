//! WaiterManager for blocking list operations
//!
//! Manages clients waiting for data on list keys using tokio::sync::Notify.

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;
use tokio::sync::Notify;

/// A single waiter waiting for data on a key
struct Waiter {
    notify: Arc<Notify>,
}

/// Manages waiters for blocking list operations (BLPOP/BRPOP)
pub struct WaiterManager {
    /// Map from key to list of waiters (FIFO order)
    waiters: Mutex<HashMap<String, Vec<Waiter>>>,
}

impl WaiterManager {
    /// Create a new WaiterManager
    pub fn new() -> Self {
        Self {
            waiters: Mutex::new(HashMap::new()),
        }
    }

    /// Register a waiter for a key. Returns a Notify handle to wait on.
    pub fn register(&self, key: &str) -> Arc<Notify> {
        let notify = Arc::new(Notify::new());
        let waiter = Waiter {
            notify: notify.clone(),
        };

        let mut waiters = self.waiters.lock();
        waiters
            .entry(key.to_string())
            .or_insert_with(Vec::new)
            .push(waiter);

        notify
    }

    /// Remove a waiter for a key (e.g., on timeout or cancellation)
    pub fn unregister(&self, key: &str, notify: &Arc<Notify>) {
        let mut waiters = self.waiters.lock();
        if let Some(key_waiters) = waiters.get_mut(key) {
            key_waiters.retain(|w| !Arc::ptr_eq(&w.notify, notify));
            if key_waiters.is_empty() {
                waiters.remove(key);
            }
        }
    }

    /// Notify and remove the first waiter for a key (called on LPUSH/RPUSH)
    /// Returns the Notify handle if a waiter was notified (caller should not re-register)
    pub fn notify_one(&self, key: &str) -> Option<Arc<Notify>> {
        let mut waiters = self.waiters.lock();
        if let Some(key_waiters) = waiters.get_mut(key) {
            if !key_waiters.is_empty() {
                // Pop first waiter (FIFO order) and notify
                let waiter = key_waiters.remove(0);
                waiter.notify.notify_one();

                // Clean up empty entry
                if key_waiters.is_empty() {
                    waiters.remove(key);
                }

                return Some(waiter.notify);
            }
        }
        None
    }

    /// Get the number of waiters for a key
    #[allow(dead_code)]
    pub fn waiter_count(&self, key: &str) -> usize {
        let waiters = self.waiters.lock();
        waiters.get(key).map(|v| v.len()).unwrap_or(0)
    }
}

impl Default for WaiterManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_and_notify() {
        let manager = WaiterManager::new();

        let _notify = manager.register("queue");
        assert_eq!(manager.waiter_count("queue"), 1);

        // Notify should wake up and remove the waiter
        assert!(manager.notify_one("queue").is_some());
        assert_eq!(manager.waiter_count("queue"), 0);
    }

    #[tokio::test]
    async fn test_multiple_waiters() {
        let manager = WaiterManager::new();

        let _notify1 = manager.register("queue");
        let _notify2 = manager.register("queue");
        assert_eq!(manager.waiter_count("queue"), 2);

        // First notify wakes and removes first waiter
        assert!(manager.notify_one("queue").is_some());
        assert_eq!(manager.waiter_count("queue"), 1);

        // Second notify wakes and removes second waiter
        assert!(manager.notify_one("queue").is_some());
        assert_eq!(manager.waiter_count("queue"), 0);
    }

    #[test]
    fn test_notify_empty() {
        let manager = WaiterManager::new();
        assert!(manager.notify_one("nonexistent").is_none());
    }
}
