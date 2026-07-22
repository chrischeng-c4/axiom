//! Generic bounded asynchronous execution for service-owned work items.
//!
//! This crate owns concurrency mechanics only. Durable ownership, fencing,
//! retries, target semantics, and outcome persistence remain in each domain's
//! committed state machine.

use futures::stream::{self, StreamExt};

/// Execute every input with at most `max_concurrency` futures in flight.
///
/// Results are returned in completion order. A zero concurrency setting is
/// normalized to one so configuration cannot silently drop work.
pub async fn run_bounded<I, F, Fut, T>(items: I, max_concurrency: usize, execute: F) -> Vec<T>
where
    I: IntoIterator,
    F: Fn(I::Item) -> Fut,
    Fut: std::future::Future<Output = T>,
{
    stream::iter(items)
        .map(execute)
        .buffer_unordered(max_concurrency.max(1))
        .collect()
        .await
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    use super::run_bounded;

    #[tokio::test]
    async fn processes_every_item_without_exceeding_bound() {
        let active = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));
        let results = run_bounded(0..20usize, 3, |item| {
            let active = active.clone();
            let peak = peak.clone();
            async move {
                let now = active.fetch_add(1, Ordering::SeqCst) + 1;
                peak.fetch_max(now, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis(2)).await;
                active.fetch_sub(1, Ordering::SeqCst);
                item * 2
            }
        })
        .await;
        assert_eq!(results.len(), 20);
        assert!(results.iter().all(|value| value % 2 == 0));
        assert!(peak.load(Ordering::SeqCst) <= 3);
        assert!(peak.load(Ordering::SeqCst) > 1);
    }
}
