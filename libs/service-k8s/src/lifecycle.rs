//! Kubernetes pod lifecycle and termination budget validation.
//!
//! A workload pod's shutdown sequence must execute within the Kubernetes
//! `terminationGracePeriodSeconds` allocated to it. The budget includes
//! an in-process runtime deadline (`runtime_deadline_seconds`), a trailing
//! SIGKILL reserve (`sigkill_reserve_seconds`), application-declared minimum
//! hook duration (`min_hook_duration_seconds`), and probe timing definitions.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Standard HTTP path for liveness probes.
pub const HEALTH_ENDPOINT_PATH: &str = "/healthz";
/// Standard HTTP path for readiness and startup probes.
pub const READY_ENDPOINT_PATH: &str = "/readyz";

/// Standard environment variable name for the in-process runtime shutdown deadline (seconds).
pub const ENV_SERVICE_RUNTIME_DEADLINE_SECONDS: &str = "SERVICE_RUNTIME_DEADLINE_SECONDS";
/// Standard environment variable name for the trailing SIGKILL safety reserve (seconds).
pub const ENV_SERVICE_SIGKILL_RESERVE_SECONDS: &str = "SERVICE_SIGKILL_RESERVE_SECONDS";

/// Configured timing for Kubernetes container probes.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProbeTiming {
    pub period_seconds: u32,
    pub timeout_seconds: u32,
    pub failure_threshold: u32,
    pub success_threshold: u32,
}

impl Default for ProbeTiming {
    fn default() -> Self {
        Self {
            period_seconds: 10,
            timeout_seconds: 1,
            failure_threshold: 3,
            success_threshold: 1,
        }
    }
}

/// Unvalidated input configuration for a workload's termination lifecycle policy.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LifecyclePolicy {
    pub total_grace_period_seconds: u64,
    pub runtime_deadline_seconds: u64,
    pub sigkill_reserve_seconds: u64,
    pub min_hook_duration_seconds: u64,
    #[serde(default)]
    pub probe_timing: ProbeTiming,
}

impl Default for LifecyclePolicy {
    fn default() -> Self {
        Self {
            total_grace_period_seconds: 30,
            runtime_deadline_seconds: 25,
            sigkill_reserve_seconds: 5,
            min_hook_duration_seconds: 1,
            probe_timing: ProbeTiming::default(),
        }
    }
}

/// A validated termination budget.
///
/// Constructed only via [`LifecyclePolicy::validate`] or [`TryFrom`].
/// Guarantees that:
/// - `total_grace_period_seconds > 0`
/// - `runtime_deadline_seconds + sigkill_reserve_seconds <= total_grace_period_seconds`
/// - `runtime_deadline_seconds >= min_hook_duration_seconds`
/// - Addition does not overflow `u64`
/// - Probe timing fields (period, timeout, failure, success) are all >= 1
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminationBudget {
    total_grace_period_seconds: u64,
    runtime_deadline_seconds: u64,
    sigkill_reserve_seconds: u64,
    min_hook_duration_seconds: u64,
    probe_timing: ProbeTiming,
}

impl TerminationBudget {
    pub fn total_grace_period_seconds(&self) -> u64 {
        self.total_grace_period_seconds
    }

    pub fn runtime_deadline_seconds(&self) -> u64 {
        self.runtime_deadline_seconds
    }

    pub fn sigkill_reserve_seconds(&self) -> u64 {
        self.sigkill_reserve_seconds
    }

    pub fn min_hook_duration_seconds(&self) -> u64 {
        self.min_hook_duration_seconds
    }

    pub fn probe_timing(&self) -> ProbeTiming {
        self.probe_timing
    }

    pub fn raw_policy(&self) -> LifecyclePolicy {
        LifecyclePolicy {
            total_grace_period_seconds: self.total_grace_period_seconds,
            runtime_deadline_seconds: self.runtime_deadline_seconds,
            sigkill_reserve_seconds: self.sigkill_reserve_seconds,
            min_hook_duration_seconds: self.min_hook_duration_seconds,
            probe_timing: self.probe_timing,
        }
    }

    /// Render the liveness probe (`GET /healthz`) targeting `port`.
    pub fn render_liveness_probe(&self, port: u16) -> serde_json::Value {
        serde_json::json!({
            "httpGet": {
                "path": HEALTH_ENDPOINT_PATH,
                "port": port,
            },
            "periodSeconds": self.probe_timing.period_seconds,
            "timeoutSeconds": self.probe_timing.timeout_seconds,
            "failureThreshold": self.probe_timing.failure_threshold,
            "successThreshold": 1,
        })
    }

    /// Render the readiness probe (`GET /readyz`) targeting `port`.
    pub fn render_readiness_probe(&self, port: u16) -> serde_json::Value {
        serde_json::json!({
            "httpGet": {
                "path": READY_ENDPOINT_PATH,
                "port": port,
            },
            "periodSeconds": self.probe_timing.period_seconds,
            "timeoutSeconds": self.probe_timing.timeout_seconds,
            "failureThreshold": self.probe_timing.failure_threshold,
            "successThreshold": self.probe_timing.success_threshold,
        })
    }

    /// Render the startup probe (`GET /readyz`) targeting `port`.
    pub fn render_startup_probe(&self, port: u16) -> serde_json::Value {
        serde_json::json!({
            "httpGet": {
                "path": READY_ENDPOINT_PATH,
                "port": port,
            },
            "periodSeconds": self.probe_timing.period_seconds,
            "timeoutSeconds": self.probe_timing.timeout_seconds,
            "failureThreshold": self.probe_timing.failure_threshold,
            "successThreshold": 1,
        })
    }
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum LifecyclePolicyError {
    #[error("total_grace_period_seconds must be greater than zero")]
    ZeroTotalGrace,
    #[error(
        "runtime deadline ({runtime_deadline_seconds}s) + SIGKILL reserve ({sigkill_reserve_seconds}s) exceeds total grace period ({total_grace_period_seconds}s)"
    )]
    BudgetExceedsTotal {
        total_grace_period_seconds: u64,
        runtime_deadline_seconds: u64,
        sigkill_reserve_seconds: u64,
    },
    #[error(
        "runtime deadline ({runtime_deadline_seconds}s) is below application minimum hook duration ({min_hook_duration_seconds}s)"
    )]
    RuntimeBelowMinimumHook {
        runtime_deadline_seconds: u64,
        min_hook_duration_seconds: u64,
    },
    #[error("budget calculation overflowed seconds field width")]
    Overflow,
    #[error(
        "probe timing fields must be at least 1 (period={period_seconds}s, timeout={timeout_seconds}s, failure={failure_threshold}, success={success_threshold})"
    )]
    InvalidProbeTiming {
        period_seconds: u32,
        timeout_seconds: u32,
        failure_threshold: u32,
        success_threshold: u32,
    },
}

impl LifecyclePolicy {
    pub fn validate(&self) -> Result<TerminationBudget, LifecyclePolicyError> {
        if self.total_grace_period_seconds == 0 {
            return Err(LifecyclePolicyError::ZeroTotalGrace);
        }

        let required = self
            .runtime_deadline_seconds
            .checked_add(self.sigkill_reserve_seconds)
            .ok_or(LifecyclePolicyError::Overflow)?;

        if required > self.total_grace_period_seconds {
            return Err(LifecyclePolicyError::BudgetExceedsTotal {
                total_grace_period_seconds: self.total_grace_period_seconds,
                runtime_deadline_seconds: self.runtime_deadline_seconds,
                sigkill_reserve_seconds: self.sigkill_reserve_seconds,
            });
        }

        if self.runtime_deadline_seconds < self.min_hook_duration_seconds {
            return Err(LifecyclePolicyError::RuntimeBelowMinimumHook {
                runtime_deadline_seconds: self.runtime_deadline_seconds,
                min_hook_duration_seconds: self.min_hook_duration_seconds,
            });
        }

        if self.probe_timing.period_seconds == 0
            || self.probe_timing.timeout_seconds == 0
            || self.probe_timing.failure_threshold == 0
            || self.probe_timing.success_threshold == 0
        {
            return Err(LifecyclePolicyError::InvalidProbeTiming {
                period_seconds: self.probe_timing.period_seconds,
                timeout_seconds: self.probe_timing.timeout_seconds,
                failure_threshold: self.probe_timing.failure_threshold,
                success_threshold: self.probe_timing.success_threshold,
            });
        }

        Ok(TerminationBudget {
            total_grace_period_seconds: self.total_grace_period_seconds,
            runtime_deadline_seconds: self.runtime_deadline_seconds,
            sigkill_reserve_seconds: self.sigkill_reserve_seconds,
            min_hook_duration_seconds: self.min_hook_duration_seconds,
            probe_timing: self.probe_timing,
        })
    }
}

impl TryFrom<&LifecyclePolicy> for TerminationBudget {
    type Error = LifecyclePolicyError;

    fn try_from(policy: &LifecyclePolicy) -> Result<Self, Self::Error> {
        policy.validate()
    }
}

impl TryFrom<LifecyclePolicy> for TerminationBudget {
    type Error = LifecyclePolicyError;

    fn try_from(policy: LifecyclePolicy) -> Result<Self, Self::Error> {
        policy.validate()
    }
}
