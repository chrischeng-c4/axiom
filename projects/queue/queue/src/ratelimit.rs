//! Rate limiting for task execution.
//!
//! Provides rate limiting to control task execution frequency.
//! Supports multiple algorithms and both in-memory and distributed (Redis) backends.
//!
//! # Example
//! ```rust,ignore
//! use cclab_queue::ratelimit::{RateLimiter, RateLimitConfig, TokenBucket};
//!
//! // Create a rate limiter: 10 tasks per second
//! let limiter = TokenBucket::new(RateLimitConfig {
//!     rate: 10.0,           // 10 tokens per second
//!     capacity: 20,         // Allow burst of up to 20
//!     ..Default::default()
//! });
//!
//! // Check if we can proceed
//! if limiter.acquire().await {
//!     // Execute task
//! } else {
//!     // Rate limited, retry later
//! }
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Tokens per second (rate)
    pub rate: f64,
    /// Maximum burst capacity
    pub capacity: u32,
    /// Key for this rate limit (task name, queue, etc.)
    #[serde(default)]
    pub key: String,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            rate: 10.0,
            capacity: 10,
            key: "default".to_string(),
        }
    }
}

impl RateLimitConfig {
    /// Create config for N tasks per second
    pub fn per_second(n: u32) -> Self {
        Self {
            rate: n as f64,
            capacity: n,
            key: "default".to_string(),
        }
    }

    /// Create config for N tasks per minute
    pub fn per_minute(n: u32) -> Self {
        Self {
            rate: n as f64 / 60.0,
            capacity: n.min(100), // Reasonable burst
            key: "default".to_string(),
        }
    }

    /// Create config for N tasks per hour
    pub fn per_hour(n: u32) -> Self {
        Self {
            rate: n as f64 / 3600.0,
            capacity: (n / 60).clamp(1, 100),
            key: "default".to_string(),
        }
    }

    /// Set the key for this rate limit
    pub fn with_key(mut self, key: &str) -> Self {
        self.key = key.to_string();
        self
    }
}

/// Result of a rate limit check
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    /// Whether the request is allowed
    pub allowed: bool,
    /// Time to wait before retry (if not allowed)
    pub retry_after: Option<Duration>,
    /// Remaining tokens/requests in current window
    pub remaining: u32,
    /// Total limit
    pub limit: u32,
}

impl RateLimitResult {
    /// Create an allowed result
    pub fn allowed(remaining: u32, limit: u32) -> Self {
        Self {
            allowed: true,
            retry_after: None,
            remaining,
            limit,
        }
    }

    /// Create a denied result
    pub fn denied(retry_after: Duration, limit: u32) -> Self {
        Self {
            allowed: false,
            retry_after: Some(retry_after),
            remaining: 0,
            limit,
        }
    }
}

/// Rate limiter trait
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Try to acquire a permit to execute
    async fn acquire(&self, key: &str) -> RateLimitResult;

    /// Try to acquire multiple permits
    async fn acquire_many(&self, key: &str, count: u32) -> RateLimitResult;

    /// Get current state without consuming
    async fn peek(&self, key: &str) -> RateLimitResult;

    /// Reset the rate limiter for a key
    async fn reset(&self, key: &str);
}

/// Token bucket rate limiter (in-memory)
///
/// Allows smooth rate limiting with burst capacity.
/// Tokens are added at a constant rate up to the capacity.
pub struct TokenBucket {
    config: RateLimitConfig,
    buckets: RwLock<HashMap<String, BucketState>>,
}

#[derive(Debug, Clone)]
struct BucketState {
    tokens: f64,
    last_update: Instant,
}

impl TokenBucket {
    /// Create a new token bucket rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            buckets: RwLock::new(HashMap::new()),
        }
    }

    /// Create with rate per second
    pub fn per_second(rate: u32) -> Self {
        Self::new(RateLimitConfig::per_second(rate))
    }

    /// Create with rate per minute
    pub fn per_minute(rate: u32) -> Self {
        Self::new(RateLimitConfig::per_minute(rate))
    }

    fn refill(&self, state: &mut BucketState) {
        let now = Instant::now();
        let elapsed = now.duration_since(state.last_update).as_secs_f64();
        let new_tokens = elapsed * self.config.rate;
        state.tokens = (state.tokens + new_tokens).min(self.config.capacity as f64);
        state.last_update = now;
    }
}

#[async_trait]
impl RateLimiter for TokenBucket {
    async fn acquire(&self, key: &str) -> RateLimitResult {
        self.acquire_many(key, 1).await
    }

    async fn acquire_many(&self, key: &str, count: u32) -> RateLimitResult {
        let mut buckets = self.buckets.write().await;
        let state = buckets.entry(key.to_string()).or_insert_with(|| BucketState {
            tokens: self.config.capacity as f64,
            last_update: Instant::now(),
        });

        self.refill(state);

        let count_f64 = count as f64;
        if state.tokens >= count_f64 {
            state.tokens -= count_f64;
            RateLimitResult::allowed(state.tokens as u32, self.config.capacity)
        } else {
            // Calculate wait time
            let tokens_needed = count_f64 - state.tokens;
            let wait_secs = tokens_needed / self.config.rate;
            RateLimitResult::denied(
                Duration::from_secs_f64(wait_secs),
                self.config.capacity,
            )
        }
    }

    async fn peek(&self, key: &str) -> RateLimitResult {
        let mut buckets = self.buckets.write().await;
        let state = buckets.entry(key.to_string()).or_insert_with(|| BucketState {
            tokens: self.config.capacity as f64,
            last_update: Instant::now(),
        });

        self.refill(state);
        RateLimitResult::allowed(state.tokens as u32, self.config.capacity)
    }

    async fn reset(&self, key: &str) {
        let mut buckets = self.buckets.write().await;
        buckets.insert(
            key.to_string(),
            BucketState {
                tokens: self.config.capacity as f64,
                last_update: Instant::now(),
            },
        );
    }
}

/// Sliding window rate limiter (in-memory)
///
/// More accurate than fixed windows, tracks requests in a sliding time window.
pub struct SlidingWindow {
    config: RateLimitConfig,
    window_duration: Duration,
    windows: RwLock<HashMap<String, WindowState>>,
}

#[derive(Debug, Clone)]
struct WindowState {
    /// Timestamps of requests in current window
    requests: Vec<Instant>,
}

impl SlidingWindow {
    /// Create a new sliding window rate limiter
    pub fn new(config: RateLimitConfig, window: Duration) -> Self {
        Self {
            config,
            window_duration: window,
            windows: RwLock::new(HashMap::new()),
        }
    }

    /// Create with rate per second (1 second window)
    pub fn per_second(rate: u32) -> Self {
        Self::new(
            RateLimitConfig {
                rate: rate as f64,
                capacity: rate,
                key: "default".to_string(),
            },
            Duration::from_secs(1),
        )
    }

    /// Create with rate per minute (1 minute window)
    pub fn per_minute(rate: u32) -> Self {
        Self::new(
            RateLimitConfig {
                rate: rate as f64 / 60.0,
                capacity: rate,
                key: "default".to_string(),
            },
            Duration::from_secs(60),
        )
    }

    fn cleanup(&self, state: &mut WindowState) {
        let cutoff = Instant::now() - self.window_duration;
        state.requests.retain(|&t| t > cutoff);
    }
}

#[async_trait]
impl RateLimiter for SlidingWindow {
    async fn acquire(&self, key: &str) -> RateLimitResult {
        self.acquire_many(key, 1).await
    }

    async fn acquire_many(&self, key: &str, count: u32) -> RateLimitResult {
        let mut windows = self.windows.write().await;
        let state = windows.entry(key.to_string()).or_insert_with(|| WindowState {
            requests: Vec::new(),
        });

        self.cleanup(state);

        let current_count = state.requests.len() as u32;
        if current_count + count <= self.config.capacity {
            let now = Instant::now();
            for _ in 0..count {
                state.requests.push(now);
            }
            RateLimitResult::allowed(
                self.config.capacity - current_count - count,
                self.config.capacity,
            )
        } else {
            // Calculate when oldest request will expire
            let retry_after = if let Some(&oldest) = state.requests.first() {
                let expires_at = oldest + self.window_duration;
                expires_at.saturating_duration_since(Instant::now())
            } else {
                Duration::from_millis(100)
            };
            RateLimitResult::denied(retry_after, self.config.capacity)
        }
    }

    async fn peek(&self, key: &str) -> RateLimitResult {
        let mut windows = self.windows.write().await;
        let state = windows.entry(key.to_string()).or_insert_with(|| WindowState {
            requests: Vec::new(),
        });

        self.cleanup(state);
        let current_count = state.requests.len() as u32;
        RateLimitResult::allowed(
            self.config.capacity.saturating_sub(current_count),
            self.config.capacity,
        )
    }

    async fn reset(&self, key: &str) {
        let mut windows = self.windows.write().await;
        windows.insert(key.to_string(), WindowState { requests: Vec::new() });
    }
}

/// Composite rate limiter that manages multiple rate limits
pub struct RateLimitManager {
    /// Per-task rate limits
    task_limits: HashMap<String, Arc<dyn RateLimiter>>,
    /// Per-queue rate limits
    queue_limits: HashMap<String, Arc<dyn RateLimiter>>,
    /// Global rate limit
    global_limit: Option<Arc<dyn RateLimiter>>,
}

impl Default for RateLimitManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimitManager {
    /// Create a new rate limit manager
    pub fn new() -> Self {
        Self {
            task_limits: HashMap::new(),
            queue_limits: HashMap::new(),
            global_limit: None,
        }
    }

    /// Add a per-task rate limit
    pub fn task_limit<R: RateLimiter + 'static>(mut self, task_name: &str, limiter: R) -> Self {
        self.task_limits.insert(task_name.to_string(), Arc::new(limiter));
        self
    }

    /// Add a per-queue rate limit
    pub fn queue_limit<R: RateLimiter + 'static>(mut self, queue: &str, limiter: R) -> Self {
        self.queue_limits.insert(queue.to_string(), Arc::new(limiter));
        self
    }

    /// Set global rate limit
    pub fn global_limit<R: RateLimiter + 'static>(mut self, limiter: R) -> Self {
        self.global_limit = Some(Arc::new(limiter));
        self
    }

    /// Check if a task can be executed
    pub async fn check(&self, task_name: &str, queue: &str) -> RateLimitResult {
        // Check global limit first
        if let Some(global) = &self.global_limit {
            let result = global.acquire("global").await;
            if !result.allowed {
                return result;
            }
        }

        // Check queue limit
        if let Some(limiter) = self.queue_limits.get(queue) {
            let result = limiter.acquire(queue).await;
            if !result.allowed {
                return result;
            }
        }

        // Check task limit
        if let Some(limiter) = self.task_limits.get(task_name) {
            return limiter.acquire(task_name).await;
        }

        // All checks passed
        RateLimitResult::allowed(u32::MAX, u32::MAX)
    }

    /// Check without consuming (for preview)
    pub async fn peek(&self, task_name: &str, queue: &str) -> RateLimitResult {
        if let Some(global) = &self.global_limit {
            let result = global.peek("global").await;
            if !result.allowed {
                return result;
            }
        }

        if let Some(limiter) = self.queue_limits.get(queue) {
            let result = limiter.peek(queue).await;
            if !result.allowed {
                return result;
            }
        }

        if let Some(limiter) = self.task_limits.get(task_name) {
            return limiter.peek(task_name).await;
        }

        RateLimitResult::allowed(u32::MAX, u32::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Existing tests (kept) ===

    #[tokio::test]
    async fn test_token_bucket_basic() {
        let limiter = TokenBucket::per_second(5);
        for i in 0..5 {
            let result = limiter.acquire("test").await;
            assert!(result.allowed, "Request {} should be allowed", i);
        }
        let result = limiter.acquire("test").await;
        assert!(!result.allowed, "6th request should be denied");
        assert!(result.retry_after.is_some());
    }

    #[tokio::test]
    async fn test_token_bucket_refill() {
        let limiter = TokenBucket::new(RateLimitConfig {
            rate: 100.0,
            capacity: 5,
            key: "test".to_string(),
        });
        for _ in 0..5 {
            limiter.acquire("test").await;
        }
        tokio::time::sleep(Duration::from_millis(60)).await;
        let result = limiter.acquire("test").await;
        assert!(result.allowed);
    }

    #[tokio::test]
    async fn test_sliding_window_basic() {
        let limiter = SlidingWindow::per_second(3);
        for _ in 0..3 {
            let result = limiter.acquire("test").await;
            assert!(result.allowed);
        }
        let result = limiter.acquire("test").await;
        assert!(!result.allowed);
    }

    #[tokio::test]
    async fn test_rate_limit_manager() {
        let manager = RateLimitManager::new()
            .task_limit("slow_task", TokenBucket::per_second(1))
            .queue_limit("limited", SlidingWindow::per_second(2))
            .global_limit(TokenBucket::per_second(100));
        let result = manager.check("slow_task", "default").await;
        assert!(result.allowed);
        let result = manager.check("slow_task", "default").await;
        assert!(!result.allowed);
        let result = manager.check("fast_task", "limited").await;
        assert!(result.allowed);
    }

    #[tokio::test]
    async fn test_per_minute_config() {
        let config = RateLimitConfig::per_minute(60);
        assert_eq!(config.rate, 1.0);
    }

    #[tokio::test]
    async fn test_reset() {
        let limiter = TokenBucket::per_second(1);
        limiter.acquire("test").await;
        let result = limiter.acquire("test").await;
        assert!(!result.allowed);
        limiter.reset("test").await;
        let result = limiter.acquire("test").await;
        assert!(result.allowed);
    }

    // === T1-T11: RateLimitConfig ===

    #[test]
    fn config_default() {
        let c = RateLimitConfig::default();
        assert_eq!(c.rate, 10.0);
        assert_eq!(c.capacity, 10);
        assert_eq!(c.key, "default");
    }

    #[test]
    fn config_per_second() {
        let c = RateLimitConfig::per_second(5);
        assert_eq!(c.rate, 5.0);
        assert_eq!(c.capacity, 5);
        assert_eq!(c.key, "default");
    }

    #[test]
    fn config_per_minute() {
        let c = RateLimitConfig::per_minute(60);
        assert_eq!(c.rate, 1.0);
        assert_eq!(c.capacity, 60);
    }

    #[test]
    fn config_per_minute_cap_clamped() {
        let c = RateLimitConfig::per_minute(200);
        assert_eq!(c.capacity, 100);
    }

    #[test]
    fn config_per_hour() {
        let c = RateLimitConfig::per_hour(3600);
        assert_eq!(c.rate, 1.0);
        assert_eq!(c.capacity, 60); // 3600/60
    }

    #[test]
    fn config_per_hour_clamp_min() {
        let c = RateLimitConfig::per_hour(1);
        assert_eq!(c.capacity, 1);
    }

    #[test]
    fn config_per_hour_clamp_max() {
        let c = RateLimitConfig::per_hour(360_000);
        assert_eq!(c.capacity, 100);
    }

    #[test]
    fn config_with_key() {
        let c = RateLimitConfig::per_second(1).with_key("custom");
        assert_eq!(c.key, "custom");
    }

    #[test]
    fn config_serde_roundtrip() {
        let c = RateLimitConfig::per_second(5);
        let json = serde_json::to_string(&c).unwrap();
        let c2: RateLimitConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(c2.rate, c.rate);
        assert_eq!(c2.capacity, c.capacity);
    }

    #[test]
    fn config_debug_impl() {
        let s = format!("{:?}", RateLimitConfig::default());
        assert!(s.contains("rate"));
    }

    #[test]
    fn config_clone() {
        let c = RateLimitConfig::per_second(7);
        let c2 = c.clone();
        assert_eq!(c2.rate, 7.0);
        assert_eq!(c2.capacity, 7);
    }

    // === T12-T15: RateLimitResult ===

    #[test]
    fn result_allowed_fields() {
        let r = RateLimitResult::allowed(5, 10);
        assert!(r.allowed);
        assert!(r.retry_after.is_none());
        assert_eq!(r.remaining, 5);
        assert_eq!(r.limit, 10);
    }

    #[test]
    fn result_denied_fields() {
        let r = RateLimitResult::denied(Duration::from_secs(2), 10);
        assert!(!r.allowed);
        assert_eq!(r.retry_after, Some(Duration::from_secs(2)));
        assert_eq!(r.remaining, 0);
        assert_eq!(r.limit, 10);
    }

    #[test]
    fn result_debug_impl() {
        let r = RateLimitResult::allowed(1, 1);
        let _ = format!("{:?}", r);
    }

    #[test]
    fn result_clone() {
        let r = RateLimitResult::denied(Duration::from_secs(1), 5);
        let r2 = r.clone();
        assert_eq!(r2.allowed, r.allowed);
        assert_eq!(r2.remaining, r.remaining);
        assert_eq!(r2.limit, r.limit);
    }

    // === T16-T27: TokenBucket ===

    #[tokio::test]
    async fn tb_new_starts_at_capacity() {
        let tb = TokenBucket::new(RateLimitConfig::per_second(5));
        let r = tb.peek("k").await;
        assert_eq!(r.remaining, 5);
    }

    #[tokio::test]
    async fn tb_per_minute_constructor() {
        let tb = TokenBucket::per_minute(60);
        let r = tb.acquire("k").await;
        assert!(r.allowed);
    }

    #[tokio::test]
    async fn tb_acquire_decrements_tokens() {
        let tb = TokenBucket::new(RateLimitConfig::per_second(3));
        tb.acquire("k").await;
        let r = tb.peek("k").await;
        assert_eq!(r.remaining, 2);
    }

    #[tokio::test]
    async fn tb_acquire_many_success() {
        let tb = TokenBucket::new(RateLimitConfig::per_second(5));
        let r = tb.acquire_many("k", 3).await;
        assert!(r.allowed);
        assert_eq!(r.remaining, 2);
    }

    #[tokio::test]
    async fn tb_acquire_many_denied() {
        let tb = TokenBucket::new(RateLimitConfig::per_second(5));
        let r = tb.acquire_many("k", 6).await;
        assert!(!r.allowed);
        assert!(r.retry_after.is_some());
    }

    #[tokio::test]
    async fn tb_acquire_many_retry_after_calculation() {
        let tb = TokenBucket::new(RateLimitConfig {
            rate: 10.0,
            capacity: 5,
            key: "default".to_string(),
        });
        let r = tb.acquire_many("k", 6).await;
        assert!(!r.allowed);
        // Need 1 token at rate 10/s => ~0.1s
        let wait = r.retry_after.unwrap();
        assert!(wait.as_secs_f64() > 0.05 && wait.as_secs_f64() < 0.2);
    }

    #[tokio::test]
    async fn tb_peek_does_not_consume() {
        let tb = TokenBucket::new(RateLimitConfig::per_second(5));
        let r1 = tb.peek("k").await;
        let r2 = tb.peek("k").await;
        assert_eq!(r1.remaining, r2.remaining);
    }

    #[tokio::test]
    async fn tb_key_isolation() {
        let tb = TokenBucket::new(RateLimitConfig::per_second(1));
        tb.acquire("a").await;
        let result_a = tb.acquire("a").await;
        let result_b = tb.acquire("b").await;
        assert!(!result_a.allowed);
        assert!(result_b.allowed);
    }

    #[tokio::test]
    async fn tb_refill_restores_tokens() {
        let tb = TokenBucket::new(RateLimitConfig {
            rate: 100.0,
            capacity: 5,
            key: "default".to_string(),
        });
        for _ in 0..5 {
            tb.acquire("k").await;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
        let r = tb.acquire("k").await;
        assert!(r.allowed);
    }

    #[tokio::test]
    async fn tb_refill_capped_at_capacity() {
        let tb = TokenBucket::new(RateLimitConfig::per_second(3));
        tokio::time::sleep(Duration::from_millis(100)).await;
        let r = tb.peek("k").await;
        assert!(r.remaining <= 3);
    }

    #[tokio::test]
    async fn tb_reset_restores_full_capacity() {
        let tb = TokenBucket::new(RateLimitConfig::per_second(5));
        for _ in 0..5 {
            tb.acquire("k").await;
        }
        tb.reset("k").await;
        let r = tb.peek("k").await;
        assert_eq!(r.remaining, 5);
    }

    #[tokio::test]
    async fn tb_acquire_delegates_to_acquire_many() {
        let tb = TokenBucket::new(RateLimitConfig::per_second(3));
        let r1 = tb.acquire("a").await;
        let tb2 = TokenBucket::new(RateLimitConfig::per_second(3));
        let r2 = tb2.acquire_many("a", 1).await;
        assert_eq!(r1.allowed, r2.allowed);
        assert_eq!(r1.remaining, r2.remaining);
    }

    // === T28-T40: SlidingWindow ===

    #[tokio::test]
    async fn sw_new_starts_empty() {
        let sw = SlidingWindow::per_second(3);
        let r = sw.peek("k").await;
        assert_eq!(r.remaining, 3);
    }

    #[tokio::test]
    async fn sw_per_second_constructor() {
        let sw = SlidingWindow::per_second(3);
        for _ in 0..3 {
            assert!(sw.acquire("k").await.allowed);
        }
        assert!(!sw.acquire("k").await.allowed);
    }

    #[tokio::test]
    async fn sw_per_minute_constructor() {
        let sw = SlidingWindow::per_minute(60);
        let r = sw.acquire("k").await;
        assert!(r.allowed);
    }

    #[tokio::test]
    async fn sw_acquire_many_success() {
        let sw = SlidingWindow::per_second(3);
        let r = sw.acquire_many("k", 2).await;
        assert!(r.allowed);
        assert_eq!(r.remaining, 1);
    }

    #[tokio::test]
    async fn sw_acquire_many_denied() {
        let sw = SlidingWindow::per_second(3);
        let r = sw.acquire_many("k", 4).await;
        assert!(!r.allowed);
    }

    #[tokio::test]
    async fn sw_peek_does_not_consume() {
        let sw = SlidingWindow::per_second(5);
        let r1 = sw.peek("k").await;
        let r2 = sw.peek("k").await;
        assert_eq!(r1.remaining, r2.remaining);
    }

    #[tokio::test]
    async fn sw_key_isolation() {
        let sw = SlidingWindow::per_second(1);
        sw.acquire("a").await;
        assert!(!sw.acquire("a").await.allowed);
        assert!(sw.acquire("b").await.allowed);
    }

    #[tokio::test]
    async fn sw_window_expiry() {
        let sw = SlidingWindow::new(
            RateLimitConfig::per_second(1),
            Duration::from_millis(50),
        );
        sw.acquire("k").await;
        assert!(!sw.acquire("k").await.allowed);
        tokio::time::sleep(Duration::from_millis(60)).await;
        assert!(sw.acquire("k").await.allowed);
    }

    #[tokio::test]
    async fn sw_retry_after_is_positive() {
        let sw = SlidingWindow::per_second(1);
        sw.acquire("k").await;
        let r = sw.acquire("k").await;
        assert!(!r.allowed);
        assert!(r.retry_after.unwrap() > Duration::ZERO);
    }

    #[tokio::test]
    async fn sw_retry_after_empty_requests_fallback() {
        // The fallback is 100ms when requests Vec is empty but denied
        // This edge case is hard to trigger in practice but we verify the denied path code
        let sw = SlidingWindow::per_second(1);
        sw.acquire("k").await;
        let r = sw.acquire("k").await;
        assert!(!r.allowed);
        // retry_after should be positive regardless of path
        assert!(r.retry_after.unwrap() > Duration::ZERO);
    }

    #[tokio::test]
    async fn sw_reset_clears_window() {
        let sw = SlidingWindow::per_second(1);
        sw.acquire("k").await;
        assert!(!sw.acquire("k").await.allowed);
        sw.reset("k").await;
        assert!(sw.acquire("k").await.allowed);
    }

    #[tokio::test]
    async fn sw_remaining_decrements_correctly() {
        let sw = SlidingWindow::per_second(5);
        sw.acquire("k").await;
        sw.acquire("k").await;
        let r = sw.peek("k").await;
        assert_eq!(r.remaining, 3);
    }

    #[tokio::test]
    async fn sw_acquire_delegates_to_acquire_many() {
        let sw1 = SlidingWindow::per_second(3);
        let r1 = sw1.acquire("a").await;
        let sw2 = SlidingWindow::per_second(3);
        let r2 = sw2.acquire_many("a", 1).await;
        assert_eq!(r1.allowed, r2.allowed);
        assert_eq!(r1.remaining, r2.remaining);
    }

    // === T41-T51: RateLimitManager ===

    #[tokio::test]
    async fn manager_default() {
        let m = RateLimitManager::default();
        let r = m.check("any", "any").await;
        assert!(r.allowed);
    }

    #[tokio::test]
    async fn manager_no_limits_allows_all() {
        let m = RateLimitManager::new();
        let r = m.check("any", "any").await;
        assert!(r.allowed);
        assert_eq!(r.remaining, u32::MAX);
        assert_eq!(r.limit, u32::MAX);
    }

    #[tokio::test]
    async fn manager_global_blocks_first() {
        let m = RateLimitManager::new()
            .global_limit(TokenBucket::per_second(1));
        assert!(m.check("t", "q").await.allowed);
        assert!(!m.check("t2", "q2").await.allowed);
    }

    #[tokio::test]
    async fn manager_queue_blocks_before_task() {
        let m = RateLimitManager::new()
            .queue_limit("q", TokenBucket::per_second(1))
            .task_limit("t", TokenBucket::per_second(10));
        assert!(m.check("t", "q").await.allowed);
        assert!(!m.check("t", "q").await.allowed); // queue blocks
    }

    #[tokio::test]
    async fn manager_task_limit_enforced() {
        let m = RateLimitManager::new()
            .task_limit("t", TokenBucket::per_second(1));
        assert!(m.check("t", "q").await.allowed);
        assert!(!m.check("t", "q").await.allowed);
    }

    #[tokio::test]
    async fn manager_different_task_not_affected() {
        let m = RateLimitManager::new()
            .task_limit("a", TokenBucket::per_second(1));
        m.check("a", "q").await;
        m.check("a", "q").await; // denied
        let r = m.check("b", "q").await;
        assert!(r.allowed); // b has no limit
    }

    #[tokio::test]
    async fn manager_peek_no_limits() {
        let m = RateLimitManager::new();
        let r = m.peek("t", "q").await;
        assert!(r.allowed);
        assert_eq!(r.remaining, u32::MAX);
    }

    #[tokio::test]
    async fn manager_peek_global() {
        let m = RateLimitManager::new()
            .global_limit(TokenBucket::per_second(5));
        let r = m.peek("t", "q").await;
        assert!(r.allowed);
        // peek falls through when allowed — remaining is u32::MAX from the default path
        // since no task_limit matched to return early
        assert_eq!(r.remaining, u32::MAX);
    }

    #[tokio::test]
    async fn manager_peek_queue() {
        let m = RateLimitManager::new()
            .queue_limit("q", TokenBucket::per_second(3));
        let r = m.peek("t", "q").await;
        assert!(r.allowed);
        // peek falls through when allowed — remaining is u32::MAX from the default path
        assert_eq!(r.remaining, u32::MAX);
    }

    #[tokio::test]
    async fn manager_peek_task() {
        let m = RateLimitManager::new()
            .task_limit("t", TokenBucket::per_second(7));
        let r = m.peek("t", "q").await;
        assert!(r.allowed);
        assert_eq!(r.remaining, 7);
    }

    #[tokio::test]
    async fn manager_builder_chaining() {
        let m = RateLimitManager::new()
            .task_limit("t", TokenBucket::per_second(1))
            .queue_limit("q", SlidingWindow::per_second(2))
            .global_limit(TokenBucket::per_second(100));
        let r = m.check("t", "q").await;
        assert!(r.allowed);
    }

    // === T52-T54: Trait + Thread Safety ===

    #[test]
    fn token_bucket_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TokenBucket>();
    }

    #[test]
    fn sliding_window_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SlidingWindow>();
    }

    #[tokio::test]
    async fn concurrent_token_bucket() {
        let tb = Arc::new(TokenBucket::new(RateLimitConfig::per_second(10)));
        let mut handles = vec![];
        for _ in 0..10 {
            let tb = Arc::clone(&tb);
            handles.push(tokio::spawn(async move {
                tb.acquire("k").await
            }));
        }
        let mut allowed = 0;
        for h in handles {
            if h.await.unwrap().allowed {
                allowed += 1;
            }
        }
        assert_eq!(allowed, 10);
    }
}
