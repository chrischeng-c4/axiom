//! Retry policy configuration

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Retry policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial delay before first retry
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Exponential backoff base (e.g., 2.0 for doubling)
    pub exponential_base: f64,
    /// Whether to add random jitter to delays
    pub jitter: bool,
    /// Error patterns to retry on (empty = retry all)
    pub retry_on: Vec<String>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(300),
            exponential_base: 2.0,
            jitter: true,
            retry_on: vec![],
        }
    }
}

impl RetryPolicy {
    /// Create a policy with no retries
    pub fn no_retry() -> Self {
        Self {
            max_retries: 0,
            ..Default::default()
        }
    }

    /// Create a policy with fixed delay
    pub fn fixed(max_retries: u32, delay: Duration) -> Self {
        Self {
            max_retries,
            initial_delay: delay,
            max_delay: delay,
            exponential_base: 1.0,
            jitter: false,
            retry_on: vec![],
        }
    }

    /// Create a policy with exponential backoff
    pub fn exponential(max_retries: u32, initial_delay: Duration, max_delay: Duration) -> Self {
        Self {
            max_retries,
            initial_delay,
            max_delay,
            exponential_base: 2.0,
            jitter: true,
            retry_on: vec![],
        }
    }

    /// Calculate delay for a given retry attempt
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::ZERO;
        }

        let base_delay = self.initial_delay.as_secs_f64()
            * self.exponential_base.powi(attempt.saturating_sub(1) as i32);

        let capped_delay = base_delay.min(self.max_delay.as_secs_f64());

        let final_delay = if self.jitter {
            // Add up to 25% jitter
            let jitter_factor = 1.0 + (rand_jitter() * 0.25);
            capped_delay * jitter_factor
        } else {
            capped_delay
        };

        Duration::from_secs_f64(final_delay)
    }

    /// Check if an error should trigger a retry
    pub fn should_retry(&self, error: &str, attempt: u32) -> bool {
        if attempt >= self.max_retries {
            return false;
        }

        if self.retry_on.is_empty() {
            return true;
        }

        self.retry_on.iter().any(|pattern| error.contains(pattern))
    }
}

/// Generate a random jitter factor between 0.0 and 1.0
fn rand_jitter() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    (nanos % 1000) as f64 / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries, 3);
        assert!(policy.should_retry("any error", 0));
        assert!(policy.should_retry("any error", 2));
        assert!(!policy.should_retry("any error", 3));
    }

    #[test]
    fn test_no_retry_policy() {
        let policy = RetryPolicy::no_retry();
        assert!(!policy.should_retry("any error", 0));
    }

    #[test]
    fn test_fixed_delay() {
        let policy = RetryPolicy::fixed(3, Duration::from_secs(5));
        assert_eq!(policy.delay_for_attempt(1).as_secs(), 5);
        assert_eq!(policy.delay_for_attempt(2).as_secs(), 5);
    }

    #[test]
    fn test_exponential_backoff() {
        let policy = RetryPolicy {
            jitter: false,
            ..RetryPolicy::exponential(5, Duration::from_secs(1), Duration::from_secs(60))
        };
        assert_eq!(policy.delay_for_attempt(1).as_secs(), 1);
        assert_eq!(policy.delay_for_attempt(2).as_secs(), 2);
        assert_eq!(policy.delay_for_attempt(3).as_secs(), 4);
        assert_eq!(policy.delay_for_attempt(4).as_secs(), 8);
    }
}
