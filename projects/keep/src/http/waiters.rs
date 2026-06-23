//! Per-key wait registry for blocking list pops (BLPOP/BRPOP). A pusher signals
//! the key's `Notify`; blocked poppers wait on it with a timeout.

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::Mutex;
use tokio::sync::Notify;

/// Registry of one `Notify` per list key that currently has (or had) waiters.
/// `notify` is a no-op for keys nobody is blocking on, so the common push path
/// stays cheap; an entry is created only when a popper actually blocks.
#[derive(Default)]
pub struct ListWaiters {
    map: Mutex<HashMap<String, Arc<Notify>>>,
}

impl ListWaiters {
    /// Get (or create) the notifier a popper will wait on for `key`.
    pub fn waiter(&self, key: &str) -> Arc<Notify> {
        let mut g = self.map.lock();
        g.entry(key.to_string())
            .or_insert_with(|| Arc::new(Notify::new()))
            .clone()
    }

    /// Wake everyone blocked on `key` (called after a push). No-op if nobody is.
    pub fn notify(&self, key: &str) {
        if let Some(n) = self.map.lock().get(key) {
            n.notify_waiters();
        }
    }
}
