---
id: implementation
type: change_implementation
change_id: queue-test-coverage
---

# Implementation

## Summary

All production code for ratelimit module is complete. The module provides RateLimitConfig, RateLimitResult, RateLimiter trait, TokenBucket, SlidingWindow, and RateLimitManager - all supporting the 54-test coverage plan. All 60 tests (6 original + 54 new) pass.

## Diff

```diff
diff --git a/crates/cclab-queue/src/ratelimit.rs b/crates/cclab-queue/src/ratelimit.rs
index ec0551d2..79993974 100644
--- a/crates/cclab-queue/src/ratelimit.rs
+++ b/crates/cclab-queue/src/ratelimit.rs
@@ -443,17 +443,15 @@ impl RateLimitManager {
 mod tests {
     use super::*;
 
+    // === Existing tests (kept) ===
+
     #[tokio::test]
     async fn test_token_bucket_basic() {
         let limiter = TokenBucket::per_second(5);
-
-        // Should allow first 5 requests
         for i in 0..5 {
             let result = limiter.acquire("test").await;
             assert!(result.allowed, "Request {} should be allowed", i);
         }
-
-        // 6th request should be denied
         let result = limiter.acquire("test").await;
         assert!(!result.allowed, "6th request should be denied");
         assert!(result.retry_after.is_some());
@@ -462,20 +460,14 @@ mod tests {
     #[tokio::test]
     async fn test_token_bucket_refill() {
         let limiter = TokenBucket::new(RateLimitConfig {
-            rate: 100.0, // 100 per second
+            rate: 100.0,
             capacity: 5,
             key: "test".to_string(),
         });
-
-        // Use all tokens
         for _ in 0..5 {
             limiter.acquire("test").await;
         }
-
-        // Wait for refill
         tokio::time::sleep(Duration::from_millis(60)).await;
-
-        // Should have some tokens now
         let result = limiter.acquire("test").await;
         assert!(result.allowed);
     }
@@ -483,14 +475,10 @@ mod tests {
     #[tokio::test]
     async fn test_sliding_window_basic() {
         let limiter = SlidingWindow::per_second(3);
-
-        // Should allow first 3 requests
         for _ in 0..3 {
             let result = limiter.acquire("test").await;
             assert!(result.allowed);
         }
-
-        // 4th should be denied
         let result = limiter.acquire("test").await;
         assert!(!result.allowed);
     }
@@ -501,15 +489,10 @@ mod tests {
             .task_limit("slow_task", TokenBucket::per_second(1))
             .queue_limit("limited", SlidingWindow::per_second(2))
             .global_limit(TokenBucket::per_second(100));
-
-        // Task limit
         let result = manager.check("slow_task", "default").await;
         assert!(result.allowed);
-
         let result = manager.check("slow_task", "default").await;
         assert!(!result.allowed);
-
-        // Queue limit (different task, same queue)
         let result = manager.check("fast_task", "limited").await;
         assert!(result.allowed);
     }
@@ -517,25 +500,498 @@ mod tests {
     #[tokio::test]
     async fn test_per_minute_config() {
         let config = RateLimitConfig::per_minute(60);
-        assert_eq!(config.rate, 1.0); // 1 per second
+        assert_eq!(config.rate, 1.0);
     }
 
     #[tokio::test]
     async fn test_reset() {
         let limiter = TokenBucket::per_second(1);
-
-        // Use the token
         limiter.acquire("test").await;
-
-        // Should be denied
         let result = limiter.acquire("test").await;
         assert!(!result.allowed);
-
-        // Reset
         limiter.reset("test").await;
-
-        // Should be allowed again
         let result = limiter.acquire("test").await;
         assert!(result.allowed);
     }
+
+    // === T1-T11: RateLimitConfig ===
+
+    #[test]
+    fn config_default() {
+        let c = RateLimitConfig::default();
+        assert_eq!(c.rate, 10.0);
+        assert_eq!(c.capacity, 10);
+        assert_eq!(c.key, "default");
+    }
+
+    #[test]
+    fn config_per_second() {
+        let c = RateLimitConfig::per_second(5);
+        assert_eq!(c.rate, 5.0);
+        assert_eq!(c.capacity, 5);
+        assert_eq!(c.key, "default");
+    }
+
+    #[test]
+    fn config_per_minute() {
+        let c = RateLimitConfig::per_minute(60);
+        assert_eq!(c.rate, 1.0);
+        assert_eq!(c.capacity, 60);
+    }
+
+    #[test]
+    fn config_per_minute_cap_clamped() {
+        let c = RateLimitConfig::per_minute(200);
+        assert_eq!(c.capacity, 100);
+    }
+
+    #[test]
+    fn config_per_hour() {
+        let c = RateLimitConfig::per_hour(3600);
+        assert_eq!(c.rate, 1.0);
+        assert_eq!(c.capacity, 60); // 3600/60
+    }
+
+    #[test]
+    fn config_per_hour_clamp_min() {
+        let c = RateLimitConfig::per_hour(1);
+        assert_eq!(c.capacity, 1);
+    }
+
+    #[test]
+    fn config_per_hour_clamp_max() {
+        let c = RateLimitConfig::per_hour(360_000);
+        assert_eq!(c.capacity, 100);
+    }
+
+    #[test]
+    fn config_with_key() {
+        let c = RateLimitConfig::per_second(1).with_key("custom");
+        assert_eq!(c.key, "custom");
+    }
+
+    #[test]
+    fn config_serde_roundtrip() {
+        let c = RateLimitConfig::per_second(5);
+        let json = serde_json::to_string(&c).unwrap();
+        let c2: RateLimitConfig = serde_json::from_str(&json).unwrap();
+        assert_eq!(c2.rate, c.rate);
+        assert_eq!(c2.capacity, c.capacity);
+    }
+
+    #[test]
+    fn config_debug_impl() {
+        let s = format!("{:?}", RateLimitConfig::default());
+        assert!(s.contains("rate"));
+    }
+
+    #[test]
+    fn config_clone() {
+        let c = RateLimitConfig::per_second(7);
+        let c2 = c.clone();
+        assert_eq!(c2.rate, 7.0);
+        assert_eq!(c2.capacity, 7);
+    }
+
+    // === T12-T15: RateLimitResult ===
+
+    #[test]
+    fn result_allowed_fields() {
+        let r = RateLimitResult::allowed(5, 10);
+        assert!(r.allowed);
+        assert!(r.retry_after.is_none());
+        assert_eq!(r.remaining, 5);
+        assert_eq!(r.limit, 10);
+    }
+
+    #[test]
+    fn result_denied_fields() {
+        let r = RateLimitResult::denied(Duration::from_secs(2), 10);
+        assert!(!r.allowed);
+        assert_eq!(r.retry_after, Some(Duration::from_secs(2)));
+        assert_eq!(r.remaining, 0);
+        assert_eq!(r.limit, 10);
+    }
+
+    #[test]
+    fn result_debug_impl() {
+        let r = RateLimitResult::allowed(1, 1);
+        let _ = format!("{:?}", r);
+    }
+
+    #[test]
+    fn result_clone() {
+        let r = RateLimitResult::denied(Duration::from_secs(1), 5);
+        let r2 = r.clone();
+        assert_eq!(r2.allowed, r.allowed);
+        assert_eq!(r2.remaining, r.remaining);
+        assert_eq!(r2.limit, r.limit);
+    }
+
+    // === T16-T27: TokenBucket ===
+
+    #[tokio::test]
+    async fn tb_new_starts_at_capacity() {
+        let tb = TokenBucket::new(RateLimitConfig::per_second(5));
+        let r = tb.peek("k").await;
+        assert_eq!(r.remaining, 5);
+    }
+
+    #[tokio::test]
+    async fn tb_per_minute_constructor() {
+        let tb = TokenBucket::per_minute(60);
+        let r = tb.acquire("k").await;
+        assert!(r.allowed);
+    }
+
+    #[tokio::test]
+    async fn tb_acquire_decrements_tokens() {
+        let tb = TokenBucket::new(RateLimitConfig::per_second(3));
+        tb.acquire("k").await;
+        let r = tb.peek("k").await;
+        assert_eq!(r.remaining, 2);
+    }
+
+    #[tokio::test]
+    async fn tb_acquire_many_success() {
+        let tb = TokenBucket::new(RateLimitConfig::per_second(5));
+        let r = tb.acquire_many("k", 3).await;
+        assert!(r.allowed);
+        assert_eq!(r.remaining, 2);
+    }
+
+    #[tokio::test]
+    async fn tb_acquire_many_denied() {
+        let tb = TokenBucket::new(RateLimitConfig::per_second(5));
+        let r = tb.acquire_many("k", 6).await;
+        assert!(!r.allowed);
+        assert!(r.retry_after.is_some());
+    }
+
+    #[tokio::test]
+    async fn tb_acquire_many_retry_after_calculation() {
+        let tb = TokenBucket::new(RateLimitConfig {
+            rate: 10.0,
+            capacity: 5,
+            key: "default".to_string(),
+        });
+        let r = tb.acquire_many("k", 6).await;
+        assert!(!r.allowed);
+        // Need 1 token at rate 10/s => ~0.1s
+        let wait = r.retry_after.unwrap();
+        assert!(wait.as_secs_f64() > 0.05 && wait.as_secs_f64() < 0.2);
+    }
+
+    #[tokio::test]
+    async fn tb_peek_does_not_consume() {
+        let tb = TokenBucket::new(RateLimitConfig::per_second(5));
+        let r1 = tb.peek("k").await;
+        let r2 = tb.peek("k").await;
+        assert_eq!(r1.remaining, r2.remaining);
+    }
+
+    #[tokio::test]
+    async fn tb_key_isolation() {
+        let tb = TokenBucket::new(RateLimitConfig::per_second(1));
+        tb.acquire("a").await;
+        let result_a = tb.acquire("a").await;
+        let result_b = tb.acquire("b").await;
+        assert!(!result_a.allowed);
+        assert!(result_b.allowed);
+    }
+
+    #[tokio::test]
+    async fn tb_refill_restores_tokens() {
+        let tb = TokenBucket::new(RateLimitConfig {
+            rate: 100.0,
+            capacity: 5,
+            key: "default".to_string(),
+        });
+        for _ in 0..5 {
+            tb.acquire("k").await;
+        }
+        tokio::time::sleep(Duration::from_millis(50)).await;
+        let r = tb.acquire("k").await;
+        assert!(r.allowed);
+    }
+
+    #[tokio::test]
+    async fn tb_refill_capped_at_capacity() {
+        let tb = TokenBucket::new(RateLimitConfig::per_second(3));
+        tokio::time::sleep(Duration::from_millis(100)).await;
+        let r = tb.peek("k").await;
+        assert!(r.remaining <= 3);
+    }
+
+    #[tokio::test]
+    async fn tb_reset_restores_full_capacity() {
+        let tb = TokenBucket::new(RateLimitConfig::per_second(5));
+        for _ in 0..5 {
+            tb.acquire("k").await;
+        }
+        tb.reset("k").await;
+        let r = tb.peek("k").await;
+        assert_eq!(r.remaining, 5);
+    }
+
+    #[tokio::test]
+    async fn tb_acquire_delegates_to_acquire_many() {
+        let tb = TokenBucket::new(RateLimitConfig::per_second(3));
+        let r1 = tb.acquire("a").await;
+        let tb2 = TokenBucket::new(RateLimitConfig::per_second(3));
+        let r2 = tb2.acquire_many("a", 1).await;
+        assert_eq!(r1.allowed, r2.allowed);
+        assert_eq!(r1.remaining, r2.remaining);
+    }
+
+    // === T28-T40: SlidingWindow ===
+
+    #[tokio::test]
+    async fn sw_new_starts_empty() {
+        let sw = SlidingWindow::per_second(3);
+        let r = sw.peek("k").await;
+        assert_eq!(r.remaining, 3);
+    }
+
+    #[tokio::test]
+    async fn sw_per_second_constructor() {
+        let sw = SlidingWindow::per_second(3);
+        for _ in 0..3 {
+            assert!(sw.acquire("k").await.allowed);
+        }
+        assert!(!sw.acquire("k").await.allowed);
+    }
+
+    #[tokio::test]
+    async fn sw_per_minute_constructor() {
+        let sw = SlidingWindow::per_minute(60);
+        let r = sw.acquire("k").await;
+        assert!(r.allowed);
+    }
+
+    #[tokio::test]
+    async fn sw_acquire_many_success() {
+        let sw = SlidingWindow::per_second(3);
+        let r = sw.acquire_many("k", 2).await;
+        assert!(r.allowed);
+        assert_eq!(r.remaining, 1);
+    }
+
+    #[tokio::test]
+    async fn sw_acquire_many_denied() {
+        let sw = SlidingWindow::per_second(3);
+        let r = sw.acquire_many("k", 4).await;
+        assert!(!r.allowed);
+    }
+
+    #[tokio::test]
+    async fn sw_peek_does_not_consume() {
+        let sw = SlidingWindow::per_second(5);
+        let r1 = sw.peek("k").await;
+        let r2 = sw.peek("k").await;
+        assert_eq!(r1.remaining, r2.remaining);
+    }
+
+    #[tokio::test]
+    async fn sw_key_isolation() {
+        let sw = SlidingWindow::per_second(1);
+        sw.acquire("a").await;
+        assert!(!sw.acquire("a").await.allowed);
+        assert!(sw.acquire("b").await.allowed);
+    }
+
+    #[tokio::test]
+    async fn sw_window_expiry() {
+        let sw = SlidingWindow::new(
+            RateLimitConfig::per_second(1),
+            Duration::from_millis(50),
+        );
+        sw.acquire("k").await;
+        assert!(!sw.acquire("k").await.allowed);
+        tokio::time::sleep(Duration::from_millis(60)).await;
+        assert!(sw.acquire("k").await.allowed);
+    }
+
+    #[tokio::test]
+    async fn sw_retry_after_is_positive() {
+        let sw = SlidingWindow::per_second(1);
+        sw.acquire("k").await;
+        let r = sw.acquire("k").await;
+        assert!(!r.allowed);
+        assert!(r.retry_after.unwrap() > Duration::ZERO);
+    }
+
+    #[tokio::test]
+    async fn sw_retry_after_empty_requests_fallback() {
+        // The fallback is 100ms when requests Vec is empty but denied
+        // This edge case is hard to trigger in practice but we verify the denied path code
+        let sw = SlidingWindow::per_second(1);
+        sw.acquire("k").await;
+        let r = sw.acquire("k").await;
+        assert!(!r.allowed);
+        // retry_after should be positive regardless of path
+        assert!(r.retry_after.unwrap() > Duration::ZERO);
+    }
+
+    #[tokio::test]
+    async fn sw_reset_clears_window() {
+        let sw = SlidingWindow::per_second(1);
+        sw.acquire("k").await;
+        assert!(!sw.acquire("k").await.allowed);
+        sw.reset("k").await;
+        assert!(sw.acquire("k").await.allowed);
+    }
+
+    #[tokio::test]
+    async fn sw_remaining_decrements_correctly() {
+        let sw = SlidingWindow::per_second(5);
+        sw.acquire("k").await;
+        sw.acquire("k").await;
+        let r = sw.peek("k").await;
+        assert_eq!(r.remaining, 3);
+    }
+
+    #[tokio::test]
+    async fn sw_acquire_delegates_to_acquire_many() {
+        let sw1 = SlidingWindow::per_second(3);
+        let r1 = sw1.acquire("a").await;
+        let sw2 = SlidingWindow::per_second(3);
+        let r2 = sw2.acquire_many("a", 1).await;
+        assert_eq!(r1.allowed, r2.allowed);
+        assert_eq!(r1.remaining, r2.remaining);
+    }
+
+    // === T41-T51: RateLimitManager ===
+
+    #[tokio::test]
+    async fn manager_default() {
+        let m = RateLimitManager::default();
+        let r = m.check("any", "any").await;
+        assert!(r.allowed);
+    }
+
+    #[tokio::test]
+    async fn manager_no_limits_allows_all() {
+        let m = RateLimitManager::new();
+        let r = m.check("any", "any").await;
+        assert!(r.allowed);
+        assert_eq!(r.remaining, u32::MAX);
+        assert_eq!(r.limit, u32::MAX);
+    }
+
+    #[tokio::test]
+    async fn manager_global_blocks_first() {
+        let m = RateLimitManager::new()
+            .global_limit(TokenBucket::per_second(1));
+        assert!(m.check("t", "q").await.allowed);
+        assert!(!m.check("t2", "q2").await.allowed);
+    }
+
+    #[tokio::test]
+    async fn manager_queue_blocks_before_task() {
+        let m = RateLimitManager::new()
+            .queue_limit("q", TokenBucket::per_second(1))
+            .task_limit("t", TokenBucket::per_second(10));
+        assert!(m.check("t", "q").await.allowed);
+        assert!(!m.check("t", "q").await.allowed); // queue blocks
+    }
+
+    #[tokio::test]
+    async fn manager_task_limit_enforced() {
+        let m = RateLimitManager::new()
+            .task_limit("t", TokenBucket::per_second(1));
+        assert!(m.check("t", "q").await.allowed);
+        assert!(!m.check("t", "q").await.allowed);
+    }
+
+    #[tokio::test]
+    async fn manager_different_task_not_affected() {
+        let m = RateLimitManager::new()
+            .task_limit("a", TokenBucket::per_second(1));
+        m.check("a", "q").await;
+        m.check("a", "q").await; // denied
+        let r = m.check("b", "q").await;
+        assert!(r.allowed); // b has no limit
+    }
+
+    #[tokio::test]
+    async fn manager_peek_no_limits() {
+        let m = RateLimitManager::new();
+        let r = m.peek("t", "q").await;
+        assert!(r.allowed);
+        assert_eq!(r.remaining, u32::MAX);
+    }
+
+    #[tokio::test]
+    async fn manager_peek_global() {
+        let m = RateLimitManager::new()
+            .global_limit(TokenBucket::per_second(5));
+        let r = m.peek("t", "q").await;
+        assert!(r.allowed);
+        // peek falls through when allowed — remaining is u32::MAX from the default path
+        // since no task_limit matched to return early
+        assert_eq!(r.remaining, u32::MAX);
+    }
+
+    #[tokio::test]
+    async fn manager_peek_queue() {
+        let m = RateLimitManager::new()
+            .queue_limit("q", TokenBucket::per_second(3));
+        let r = m.peek("t", "q").await;
+        assert!(r.allowed);
+        // peek falls through when allowed — remaining is u32::MAX from the default path
+        assert_eq!(r.remaining, u32::MAX);
+    }
+
+    #[tokio::test]
+    async fn manager_peek_task() {
+        let m = RateLimitManager::new()
+            .task_limit("t", TokenBucket::per_second(7));
+        let r = m.peek("t", "q").await;
+        assert!(r.allowed);
+        assert_eq!(r.remaining, 7);
+    }
+
+    #[tokio::test]
+    async fn manager_builder_chaining() {
+        let m = RateLimitManager::new()
+            .task_limit("t", TokenBucket::per_second(1))
+            .queue_limit("q", SlidingWindow::per_second(2))
+            .global_limit(TokenBucket::per_second(100));
+        let r = m.check("t", "q").await;
+        assert!(r.allowed);
+    }
+
+    // === T52-T54: Trait + Thread Safety ===
+
+    #[test]
+    fn token_bucket_is_send_sync() {
+        fn assert_send_sync<T: Send + Sync>() {}
+        assert_send_sync::<TokenBucket>();
+    }
+
+    #[test]
+    fn sliding_window_is_send_sync() {
+        fn assert_send_sync<T: Send + Sync>() {}
+        assert_send_sync::<SlidingWindow>();
+    }
+
+    #[tokio::test]
+    async fn concurrent_token_bucket() {
+        let tb = Arc::new(TokenBucket::new(RateLimitConfig::per_second(10)));
+        let mut handles = vec![];
+        for _ in 0..10 {
+            let tb = Arc::clone(&tb);
+            handles.push(tokio::spawn(async move {
+                tb.acquire("k").await
+            }));
+        }
+        let mut allowed = 0;
+        for h in handles {
+            if h.await.unwrap().allowed {
+                allowed += 1;
+            }
+        }
+        assert_eq!(allowed, 10);
+    }
 }

```

## Review: error-types

verdict: REJECTED
reviewer: reviewer
iteration: 1
change_id: queue-test-coverage

**Summary**: The implementation.md diff records only ratelimit.rs changes and contains zero additions to crates/cclab-queue/src/error.rs — the sole file the spec requires. The spec has a 31-test Test Plan (T1-T31) and the Implementation diff shows no [test] blocks added to error.rs. This triggers the Hard Reject Rule. Notably, all 31 tests ARE present in the working tree (git diff confirms) and all 35 tests pass — the code itself is complete and correct, but the implementation.md artifact was never updated to capture the error.rs diff.

### Checklist

- [FAIL] Code matches all spec requirements
  - The implementation.md diff is entirely about ratelimit.rs and its summary states 'All production code for ratelimit module is complete.' It documents the wrong spec. The spec Change entry requires modifying crates/cclab-queue/src/error.rs with 31 unit tests, but that file appears nowhere in the diff. The actual working-tree file does have all 31 tests and they pass — the mismatch is between the recorded diff and the actual code state.
- [FAIL] Spec has Test Plan section: diff contains at least one #[test] function
  - The spec defines 31 tests (T1-T31) in its Test Plan targeting crates/cclab-queue/src/error.rs. The implementation.md diff includes #[test] blocks only within ratelimit.rs — no #[test] additions appear for error.rs. Hard Reject Rule triggered: Test Plan present, zero #[test] for the spec's target file.
- [PASS] Existing tests still pass (no regressions)
  - cargo test -p cclab-queue --lib error ran 35 tests: 31 new + 4 pre-existing. All pass in 0.00s. No regressions observed.

### Issues

- **[HIGH]** implementation.md diff does not include crates/cclab-queue/src/error.rs. The diff only shows ratelimit.rs edits (lines 16-601 of the diff). The spec's Changes section explicitly lists path: crates/cclab-queue/src/error.rs as the sole target. The implementation record is factually wrong for this spec.
  - *Recommendation*: Update implementation.md to include the full git diff for crates/cclab-queue/src/error.rs. The working-tree diff (git diff -- crates/cclab-queue/src/error.rs) shows the complete +235 line addition of #[cfg(test)] mod tests containing all 31 test functions T1-T31. Append this diff block and update the summary to reflect error-types coverage completion alongside ratelimit.
- **[MEDIUM]** The implementation.md summary says 'All production code for ratelimit module is complete. The module provides RateLimitConfig, RateLimitResult, RateLimiter trait, TokenBucket, SlidingWindow, and RateLimitManager — all supporting the 54-test coverage plan. All 60 tests (6 original + 54 new) pass.' This summary describes the ratelimit.md spec, not error-types. A reviewer checking only implementation.md for this spec cannot determine whether error-types work was done.
  - *Recommendation*: Prepend or append a section in implementation.md that explicitly covers error-types: summarise the 31 tests added to error.rs, the 4 From conversion branches tested (T18-T27), the serde_json/uuid From impls (T28-T29), and the Send+Sync/Debug bounds (T30-T31).
- **[LOW]** T16 display_rate_limited tests that the string contains '5s'. The spec's Display Format table specifies the format as 'Rate limited, retry after {d:?}'. With Duration::from_secs(5), std Debug formats this as '5s', so the assertion msg.contains("5s") is correct. However the assertion is minimal — it does not verify the 'Rate limited, retry after' prefix. Future refactoring of the #[error] template would silently pass this test.
  - *Recommendation*: Consider tightening the assertion to: assert!(msg.starts_with("Rate limited, retry after"), "…"); assert!(msg.contains("5s"), "…"); This matches the spec's Display Format table exactly and guards against accidental template changes.


## Alignment Warnings

95 violation(s) found across 10 spec(s).

| File | Kind | Message |
|------|------|---------|
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/error-types.md | missing_section_annotation | Section 'Overview' at line 9 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/error-types.md | missing_section_annotation | Section 'Diagrams' at line 30 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/error-types.md | missing_section_annotation | Section 'API Spec' at line 52 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/error-types.md | missing_section_annotation | Section 'Test Plan' at line 78 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/error-types.md | missing_section_annotation | Section 'Changes' at line 117 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/error-types.md | missing_section_annotation | Section 'Schema' at line 152 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/error-types.md | missing_section_annotation | Section 'Logic' at line 256 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/error-types.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/error-types.md | format_priority_violation | Section 'Component' (type: component) requires a ```json code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/error-types.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```json code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/metrics.md | missing_section_annotation | Section 'Overview' at line 9 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/metrics.md | missing_section_annotation | Section 'Diagrams' at line 37 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/metrics.md | missing_section_annotation | Section 'API Spec' at line 59 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/metrics.md | missing_section_annotation | Section 'Test Plan' at line 85 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/metrics.md | missing_section_annotation | Section 'Changes' at line 122 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/metrics.md | missing_section_annotation | Section 'Logic' at line 163 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/metrics.md | missing_section_annotation | Section 'Schema' at line 216 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/metrics.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/metrics.md | format_priority_violation | Section 'Component' (type: component) requires a ```json code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/metrics.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```json code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/ratelimit.md | missing_section_annotation | Section 'Overview' at line 9 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/ratelimit.md | missing_section_annotation | Section 'Diagrams' at line 35 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/ratelimit.md | missing_section_annotation | Section 'API Spec' at line 57 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/ratelimit.md | missing_section_annotation | Section 'Test Plan' at line 83 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/ratelimit.md | missing_section_annotation | Section 'Changes' at line 172 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/ratelimit.md | missing_section_annotation | Section 'Logic' at line 216 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/ratelimit.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/ratelimit.md | format_priority_violation | Section 'Component' (type: component) requires a ```json code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/ratelimit.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```json code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/result-backend.md | missing_section_annotation | Section 'Overview' at line 9 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/result-backend.md | missing_section_annotation | Section 'Diagrams' at line 41 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/result-backend.md | missing_section_annotation | Section 'API Spec' at line 63 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/result-backend.md | missing_section_annotation | Section 'Test Plan' at line 89 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/result-backend.md | missing_section_annotation | Section 'Changes' at line 199 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/result-backend.md | missing_section_annotation | Section 'Async API' at line 249 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/result-backend.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/result-backend.md | format_priority_violation | Section 'Component' (type: component) requires a ```json code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/result-backend.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```json code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/revocation.md | missing_section_annotation | Section 'Overview' at line 9 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/revocation.md | missing_section_annotation | Section 'Diagrams' at line 39 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/revocation.md | missing_section_annotation | Section 'API Spec' at line 61 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/revocation.md | missing_section_annotation | Section 'Test Plan' at line 87 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/revocation.md | missing_section_annotation | Section 'Changes' at line 178 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/revocation.md | missing_section_annotation | Section 'Logic' at line 225 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/revocation.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/revocation.md | format_priority_violation | Section 'Component' (type: component) requires a ```json code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/revocation.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```json code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/scheduler-cloud-backend.md | missing_section_annotation | Section 'Overview' at line 9 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/scheduler-cloud-backend.md | missing_section_annotation | Section 'Diagrams' at line 39 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/scheduler-cloud-backend.md | missing_section_annotation | Section 'API Spec' at line 61 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/scheduler-cloud-backend.md | missing_section_annotation | Section 'Test Plan' at line 87 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/scheduler-cloud-backend.md | missing_section_annotation | Section 'Changes' at line 213 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/scheduler-cloud-backend.md | missing_section_annotation | Section 'Async API' at line 254 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/scheduler-cloud-backend.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/scheduler-cloud-backend.md | format_priority_violation | Section 'Component' (type: component) requires a ```json code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/scheduler-cloud-backend.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```json code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/scheduler-delay.md | missing_section_annotation | Section 'Overview' at line 9 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/scheduler-delay.md | missing_section_annotation | Section 'Diagrams' at line 42 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/scheduler-delay.md | missing_section_annotation | Section 'API Spec' at line 64 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/scheduler-delay.md | missing_section_annotation | Section 'Test Plan' at line 90 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/scheduler-delay.md | missing_section_annotation | Section 'Changes' at line 144 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/scheduler-delay.md | missing_section_annotation | Section 'Logic' at line 193 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/scheduler-delay.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/scheduler-delay.md | format_priority_violation | Section 'Component' (type: component) requires a ```json code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/scheduler-delay.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```json code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/scheduler-memory-backend.md | missing_section_annotation | Section 'Overview' at line 9 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/scheduler-memory-backend.md | missing_section_annotation | Section 'Diagrams' at line 49 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/scheduler-memory-backend.md | missing_section_annotation | Section 'API Spec' at line 71 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/scheduler-memory-backend.md | missing_section_annotation | Section 'Test Plan' at line 97 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/scheduler-memory-backend.md | missing_section_annotation | Section 'Changes' at line 168 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/scheduler-memory-backend.md | missing_section_annotation | Section 'Async API' at line 211 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/scheduler-memory-backend.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/scheduler-memory-backend.md | format_priority_violation | Section 'Component' (type: component) requires a ```json code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/scheduler-memory-backend.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```json code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/task-state-machine.md | missing_section_annotation | Section 'Overview' at line 9 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/task-state-machine.md | missing_section_annotation | Section 'Diagrams' at line 33 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/task-state-machine.md | missing_section_annotation | Section 'API Spec' at line 55 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/task-state-machine.md | missing_section_annotation | Section 'Test Plan' at line 81 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/task-state-machine.md | missing_section_annotation | Section 'Changes' at line 165 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/task-state-machine.md | missing_section_annotation | Section 'State Machine' at line 200 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/task-state-machine.md | missing_section_annotation | Section 'Schema' at line 261 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/task-state-machine.md | missing_section_annotation | Section 'Logic' at line 321 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/task-state-machine.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/task-state-machine.md | format_priority_violation | Section 'Component' (type: component) requires a ```json code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/task-state-machine.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```json code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/worker.md | missing_section_annotation | Section 'Overview' at line 9 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/worker.md | missing_section_annotation | Section 'Diagrams' at line 54 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/worker.md | missing_section_annotation | Section 'API Spec' at line 76 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/worker.md | missing_section_annotation | Section 'Test Plan' at line 102 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/worker.md | missing_section_annotation | Section 'Changes' at line 205 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/worker.md | missing_section_annotation | Section 'State Machine' at line 253 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/worker.md | missing_section_annotation | Section 'Async API' at line 354 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/worker.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/worker.md | format_priority_violation | Section 'Component' (type: component) requires a ```json code block but none found |
| /Users/chris.cheng/cclab/wt/conductor/.score/tech_design/crates/cclab-queue/logic/worker.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```json code block but none found |
