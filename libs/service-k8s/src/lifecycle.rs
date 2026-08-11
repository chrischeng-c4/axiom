//! Kubernetes pod lifecycle and termination budget validation.
//!
//! A workload pod's shutdown sequence must execute within the Kubernetes
//! `terminationGracePeriodSeconds` allocated to it. The budget includes
//! an in-process runtime deadline (`runtime_deadline_seconds`), a trailing
//! SIGKILL reserve (`sigkill_reserve_seconds`), application-declared minimum
//! hook duration (`min_hook_duration_seconds`), and probe timing definitions.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
