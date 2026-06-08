// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
// CODEGEN-BEGIN
//! Bounded retry loop for E2E actions and assertions (#2878).
//!
//! Some product flows are eventually-consistent: a click on a "save"
//! button only finishes after a backend write, and an assertion that
//! reads the post-save text needs a few retries before the DOM
//! settles. This module owns the bounded retry policy and the
//! metadata the runner records: attempt count, final outcome, last
//! failure reason. The actual sleep/clock comes from a closure so
//! tests can drive deterministic time.
//!
//! Flake classification and a global retry policy are out of scope
//! here (split into later issues); this slice covers one
//! action/assertion path.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Stable schema tag for [`RetryRecord`].
pub const RETRY_SCHEMA_VERSION: &str = "jet.e2e.retry.v1";

/// Configured bounds for a single retry loop. The runner clones one
/// of these next to the step it covers.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryBudget {
    pub max_attempts: u32,
    pub interval_ms: u64,
    pub timeout_ms: u64,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl Default for RetryBudget {
    fn default() -> Self {
        // 30 attempts × 100 ms = 3s, capped by an explicit 3s timeout
        // — matches the Playwright actionability default and keeps
        // the runner predictable.
        Self {
            max_attempts: 30,
            interval_ms: 100,
            timeout_ms: 3_000,
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl RetryBudget {
    pub fn interval(self) -> Duration {
        Duration::from_millis(self.interval_ms)
    }
}

/// Terminal outcome of a retry loop.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum RetryOutcome {
    Passed {
        attempts: u32,
        elapsed_ms: u64,
    },
    FailedAfterAttempts {
        attempts: u32,
        last_reason: String,
        elapsed_ms: u64,
    },
    TimedOut {
        attempts: u32,
        last_reason: String,
        elapsed_ms: u64,
    },
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl RetryOutcome {
    pub fn is_pass(&self) -> bool {
        matches!(self, Self::Passed { .. })
    }

    pub fn attempts(&self) -> u32 {
        match self {
            Self::Passed { attempts, .. }
            | Self::FailedAfterAttempts { attempts, .. }
            | Self::TimedOut { attempts, .. } => *attempts,
        }
    }
}

/// Evidence row the runner attaches to the step. Captures attempts +
/// outcome so an inspector reviewer can tell whether the retry budget
/// was almost exhausted or comfortably within bounds.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryRecord {
    pub schema_version: String,
    pub budget: RetryBudget,
    pub outcome: RetryOutcome,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl RetryRecord {
    pub fn from_outcome(budget: RetryBudget, outcome: RetryOutcome) -> Self {
        Self {
            schema_version: RETRY_SCHEMA_VERSION.to_string(),
            budget,
            outcome,
        }
    }
}

/// One attempt result the action/assertion closure returns. `Pending`
/// triggers another attempt; `Failed` ends the loop with the supplied
/// reason; `Passed` ends the loop with success.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttemptResult {
    Passed,
    Pending { reason: String },
    Failed { reason: String },
}

/// Drive the retry loop. `attempt` runs once per try; the closure
/// gets the 1-indexed attempt number. `now_ms` returns the current
/// monotonic clock in ms so tests can pin time.
///
/// The loop stops at the first of: passed, hard failure,
/// max_attempts reached, or timeout exceeded.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn drive_retry(
    budget: RetryBudget,
    mut now_ms: impl FnMut() -> u64,
    mut attempt: impl FnMut(u32) -> AttemptResult,
) -> RetryOutcome {
    let started = now_ms();
    let mut last_reason: Option<String> = None;
    let mut attempts: u32 = 0;
    loop {
        attempts += 1;
        let elapsed = now_ms().saturating_sub(started);
        if elapsed >= budget.timeout_ms {
            return RetryOutcome::TimedOut {
                attempts: attempts - 1,
                last_reason: last_reason.unwrap_or_else(|| "no attempt completed".into()),
                elapsed_ms: elapsed,
            };
        }
        match attempt(attempts) {
            AttemptResult::Passed => {
                return RetryOutcome::Passed {
                    attempts,
                    elapsed_ms: now_ms().saturating_sub(started),
                };
            }
            AttemptResult::Pending { reason } => {
                last_reason = Some(reason);
            }
            AttemptResult::Failed { reason } => {
                return RetryOutcome::FailedAfterAttempts {
                    attempts,
                    last_reason: reason,
                    elapsed_ms: now_ms().saturating_sub(started),
                };
            }
        }
        if attempts >= budget.max_attempts {
            return RetryOutcome::FailedAfterAttempts {
                attempts,
                last_reason: last_reason.unwrap_or_else(|| "max attempts reached".into()),
                elapsed_ms: now_ms().saturating_sub(started),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn fixture_passes_after_delayed_readiness() {
        // Stop condition (#2878): retry fixture passes after delayed
        // readiness — and the record names the attempt count.
        let budget = RetryBudget {
            max_attempts: 5,
            interval_ms: 10,
            timeout_ms: 1_000,
        };
        let clock = Cell::new(0u64);
        let outcome = drive_retry(
            budget,
            || {
                let t = clock.get();
                clock.set(t + 50);
                t
            },
            |attempt| {
                if attempt < 3 {
                    AttemptResult::Pending {
                        reason: format!("not ready (attempt {attempt})"),
                    }
                } else {
                    AttemptResult::Passed
                }
            },
        );
        assert!(outcome.is_pass());
        assert_eq!(outcome.attempts(), 3);

        let record = RetryRecord::from_outcome(budget, outcome);
        assert!(matches!(
            record.outcome,
            RetryOutcome::Passed { attempts: 3, .. }
        ));
    }

    #[test]
    fn fixture_fails_after_timeout_with_deterministic_exit() {
        // Stop condition (#2878): retry fixture fails after timeout
        // with deterministic exit behaviour.
        let budget = RetryBudget {
            max_attempts: 100,
            interval_ms: 50,
            timeout_ms: 200,
        };
        let clock = Cell::new(0u64);
        let outcome = drive_retry(
            budget,
            || {
                let t = clock.get();
                clock.set(t + 90);
                t
            },
            |_| AttemptResult::Pending {
                reason: "still waiting".into(),
            },
        );
        assert!(!outcome.is_pass());
        assert!(
            matches!(outcome, RetryOutcome::TimedOut { ref last_reason, .. } if last_reason == "still waiting")
        );
    }

    #[test]
    fn hard_failure_stops_the_loop_with_reason() {
        let budget = RetryBudget::default();
        let outcome = drive_retry(
            budget,
            || 0,
            |_| AttemptResult::Failed {
                reason: "selector resolution failed".into(),
            },
        );
        match outcome {
            RetryOutcome::FailedAfterAttempts {
                attempts,
                last_reason,
                ..
            } => {
                assert_eq!(attempts, 1);
                assert_eq!(last_reason, "selector resolution failed");
            }
            _ => panic!("expected hard-fail outcome"),
        }
    }

    #[test]
    fn max_attempts_exhaustion_records_last_pending_reason() {
        let budget = RetryBudget {
            max_attempts: 3,
            interval_ms: 1,
            timeout_ms: 60_000,
        };
        let outcome = drive_retry(
            budget,
            || 0,
            |attempt| AttemptResult::Pending {
                reason: format!("pending #{attempt}"),
            },
        );
        match outcome {
            RetryOutcome::FailedAfterAttempts {
                attempts,
                last_reason,
                ..
            } => {
                assert_eq!(attempts, 3);
                assert_eq!(last_reason, "pending #3");
            }
            _ => panic!("expected exhaustion outcome, got {outcome:?}"),
        }
    }

    #[test]
    fn retry_record_round_trips_through_json() {
        let record = RetryRecord::from_outcome(
            RetryBudget::default(),
            RetryOutcome::Passed {
                attempts: 2,
                elapsed_ms: 75,
            },
        );
        let json = serde_json::to_string(&record).unwrap();
        let back: RetryRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back, record);
        assert!(json.contains("\"outcome\":\"passed\""), "{json}");
    }
}
// CODEGEN-END
