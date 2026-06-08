//! Cooperative cancellation + timeout for agentkit (#2070).
//!
//! Every long-running surface (`Agent::run`, `Graph::run`, individual
//! LLM calls, MCP `tools/call`) accepts a [`CancellationToken`]. The
//! token is cheap to clone (one `Arc`), composable (child tokens fire
//! when the parent does), and observable from both polling
//! (`is_cancelled`) and async (`cancelled().await`) code paths.
//!
//! Two helpers wrap the token for the two most common patterns:
//!
//! * [`run_until_cancelled`] races an arbitrary future against the
//!   token and maps cancellation to [`NovaError::Cancelled`].
//! * [`with_timeout`] does the same against a deadline, mapping
//!   expiry to [`NovaError::Timeout`].
//!
//! No threads, no global state — backed by [`tokio::sync::Notify`].
//
// HANDWRITE-BEGIN reason: no rust-runtime generator emits cooperative
// cancellation primitives + future combinators yet. Once a
// `async-utilities` section type lands (planned alongside the
// agentkit observability sweep), this will move into CODEGEN markers.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::Notify;

use crate::error::{NovaError, NovaResult};

#[derive(Debug)]
struct Inner {
    cancelled: AtomicBool,
    notify: Notify,
}

/// Cooperative cancellation handle.
///
/// Cloning a token is `O(1)` and produces another handle to the same
/// underlying signal. Calling [`CancellationToken::cancel`] on any
/// clone fires every other clone (and every child) exactly once;
/// subsequent calls are no-ops.
///
/// `Serialize`/`Deserialize` are intentionally surface-only — a token
/// serialises to its current state (`is_cancelled`) so it can travel
/// across an RPC boundary as a snapshot. Deserialising builds a fresh
/// token; remote cancellation propagation is the RPC carrier's job.
#[derive(Debug, Clone)]
pub struct CancellationToken {
    inner: Arc<Inner>,
}

impl CancellationToken {
    /// Build a fresh, uncancelled root token.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                cancelled: AtomicBool::new(false),
                notify: Notify::new(),
            }),
        }
    }

    /// Build a token that starts already cancelled — useful in tests
    /// or when surfacing a synchronously-known cancellation.
    pub fn cancelled_now() -> Self {
        let t = Self::new();
        t.cancel();
        t
    }

    /// Signal cancellation. Idempotent; later callers see no effect.
    pub fn cancel(&self) {
        if !self.inner.cancelled.swap(true, Ordering::SeqCst) {
            self.inner.notify.notify_waiters();
        }
    }

    /// Snapshot the current state without awaiting.
    pub fn is_cancelled(&self) -> bool {
        self.inner.cancelled.load(Ordering::SeqCst)
    }

    /// Future that resolves the next time the token is cancelled. If
    /// the token is already cancelled when awaited, resolves
    /// immediately on the next poll.
    pub async fn cancelled(&self) {
        if self.is_cancelled() {
            return;
        }
        let notified = self.inner.notify.notified();
        if self.is_cancelled() {
            return;
        }
        notified.await;
    }

    /// Build a child token. Child tokens cancel themselves when the
    /// parent cancels but cancelling a child never affects the parent.
    pub fn child(&self) -> CancellationToken {
        let child = CancellationToken::new();
        let parent = self.clone();
        let child_handle = child.clone();
        tokio::spawn(async move {
            parent.cancelled().await;
            child_handle.cancel();
        });
        child
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Serialised snapshot of a token. `cancelled` is the state at
/// serialisation time; nothing more travels over the wire.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CancellationSnapshot {
    pub cancelled: bool,
}

impl Serialize for CancellationToken {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        CancellationSnapshot {
            cancelled: self.is_cancelled(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CancellationToken {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let snap = CancellationSnapshot::deserialize(deserializer)?;
        Ok(if snap.cancelled {
            Self::cancelled_now()
        } else {
            Self::new()
        })
    }
}

/// Race `future` against `token`. If the token fires first, returns
/// [`NovaError::Cancelled`]; otherwise returns the future's output.
pub async fn run_until_cancelled<F, T>(future: F, token: &CancellationToken) -> NovaResult<T>
where
    F: std::future::Future<Output = T>,
{
    if token.is_cancelled() {
        return Err(NovaError::Cancelled);
    }
    tokio::select! {
        biased;
        _ = token.cancelled() => Err(NovaError::Cancelled),
        out = future => Ok(out),
    }
}

/// Race `future` against `duration`. On expiry returns
/// [`NovaError::Timeout`] carrying the requested duration (seconds,
/// rounded up); otherwise returns the future's output.
pub async fn with_timeout<F, T>(future: F, duration: Duration) -> NovaResult<T>
where
    F: std::future::Future<Output = T>,
{
    match tokio::time::timeout(duration, future).await {
        Ok(v) => Ok(v),
        Err(_) => Err(NovaError::Timeout(duration.as_secs().max(1))),
    }
}

/// Combined helper: cancel-or-timeout, whichever fires first.
pub async fn run_with_deadline<F, T>(
    future: F,
    token: &CancellationToken,
    duration: Duration,
) -> NovaResult<T>
where
    F: std::future::Future<Output = T>,
{
    if token.is_cancelled() {
        return Err(NovaError::Cancelled);
    }
    tokio::select! {
        biased;
        _ = token.cancelled() => Err(NovaError::Cancelled),
        out = tokio::time::timeout(duration, future) => match out {
            Ok(v) => Ok(v),
            Err(_) => Err(NovaError::Timeout(duration.as_secs().max(1))),
        }
    }
}

// HANDWRITE-END

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test(flavor = "current_thread")]
    async fn token_starts_uncancelled() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cancel_is_observed_by_clones() {
        let a = CancellationToken::new();
        let b = a.clone();
        a.cancel();
        assert!(b.is_cancelled());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cancelled_future_resolves_after_signal() {
        let token = CancellationToken::new();
        let watcher = token.clone();
        let handle = tokio::spawn(async move {
            watcher.cancelled().await;
            "done"
        });
        // Yield once so the spawned task registers on `notify`.
        tokio::task::yield_now().await;
        token.cancel();
        let v = handle.await.unwrap();
        assert_eq!(v, "done");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cancelled_future_returns_immediately_if_already_cancelled() {
        let token = CancellationToken::cancelled_now();
        // No timeout wrapper needed — this must complete synchronously
        // on the first poll.
        token.cancelled().await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_until_cancelled_happy_path_returns_value() {
        let token = CancellationToken::new();
        let v = run_until_cancelled(async { 42 }, &token).await.unwrap();
        assert_eq!(v, 42);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_until_cancelled_returns_cancelled_when_pre_cancelled() {
        let token = CancellationToken::cancelled_now();
        let err = run_until_cancelled(async { 1 }, &token).await.unwrap_err();
        assert!(matches!(err, NovaError::Cancelled));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_until_cancelled_fires_mid_flight() {
        let token = CancellationToken::new();
        let cancel_handle = token.clone();
        tokio::spawn(async move {
            tokio::task::yield_now().await;
            cancel_handle.cancel();
        });
        let err = run_until_cancelled(
            async {
                tokio::time::sleep(Duration::from_secs(5)).await;
                "should not arrive"
            },
            &token,
        )
        .await
        .unwrap_err();
        assert!(matches!(err, NovaError::Cancelled));
    }

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn with_timeout_returns_typed_timeout_error() {
        let err = with_timeout(
            async {
                tokio::time::sleep(Duration::from_secs(60)).await;
                "never"
            },
            Duration::from_secs(2),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, NovaError::Timeout(s) if s == 2));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn with_timeout_passes_value_through_when_future_wins() {
        let v = with_timeout(async { "fast" }, Duration::from_secs(5))
            .await
            .unwrap();
        assert_eq!(v, "fast");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_with_deadline_prefers_cancellation_over_timeout() {
        let token = CancellationToken::new();
        let cancel_handle = token.clone();
        tokio::spawn(async move {
            tokio::task::yield_now().await;
            cancel_handle.cancel();
        });
        let err = run_with_deadline(
            async {
                tokio::time::sleep(Duration::from_secs(5)).await;
                "never"
            },
            &token,
            Duration::from_secs(10),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, NovaError::Cancelled));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn child_token_cancels_when_parent_cancels() {
        let parent = CancellationToken::new();
        let child = parent.child();
        tokio::task::yield_now().await; // let child spawn watcher
        parent.cancel();
        // Race-free wait via the child's own future.
        child.cancelled().await;
        assert!(child.is_cancelled());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn child_cancellation_does_not_propagate_upward() {
        let parent = CancellationToken::new();
        let child = parent.child();
        tokio::task::yield_now().await;
        child.cancel();
        assert!(!parent.is_cancelled());
    }

    #[test]
    fn snapshot_round_trips_through_serde() {
        let token = CancellationToken::cancelled_now();
        let json = serde_json::to_string(&token).unwrap();
        assert!(json.contains("\"cancelled\":true"));
        let revived: CancellationToken = serde_json::from_str(&json).unwrap();
        assert!(revived.is_cancelled());

        let fresh = CancellationToken::new();
        let json = serde_json::to_string(&fresh).unwrap();
        assert!(json.contains("\"cancelled\":false"));
        let revived: CancellationToken = serde_json::from_str(&json).unwrap();
        assert!(!revived.is_cancelled());
    }
}
