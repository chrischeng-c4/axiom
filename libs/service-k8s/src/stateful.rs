// HANDWRITE-BEGIN gap="missing-generator:stateful-capacity-policy" tracker="#1644" reason="Shared request-only resource defaults, dedicated-node placement, and whole-layer per-shard replica planning for long-running stateful services."
//! Stateful-service capacity primitives shared by every operator adopter.
//!
//! A data workload scales in whole replica layers: with `N` shards, changing
//! replicas-per-shard by one changes the StatefulSet by exactly `N` pods. A
//! vanilla HPA targets total pods and can therefore request an invalid partial
//! layer. [`plan_replica_layer`] performs the HPA utilization calculation in
//! per-shard units and always returns a valid whole-layer total.
//!
//! This module deliberately plans but does not apply a membership change.
//! `raft-runtime` currently has static membership; a controller must complete a
//! Raft membership transition before patching the StatefulSet replica count.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Default CPU request emitted for a data pod when the service leaves it empty.
pub const DEFAULT_CPU_REQUEST: &str = "1";
/// Default memory request emitted for a data pod when the service leaves it empty.
pub const DEFAULT_MEMORY_REQUEST: &str = "4Gi";

/// Resolve an optional/empty request without inventing a node-pool-specific
/// size. Deployers tune the generated Kustomize/CR for their own cluster.
pub fn resource_request_or_default<'a>(value: &'a str, default: &'static str) -> &'a str {
    if value.trim().is_empty() {
        default
    } else {
        value
    }
}

/// Autoscaling policy for a whole replica layer.
///
/// `min_replicas_per_shard` is the starting/floor value represented by an
/// app's `replicasPerShard`. The maximum and utilization targets are policy;
/// observation and stabilization belong to the operator control loop.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReplicaLayerPolicy {
    pub min_replicas_per_shard: u32,
    pub max_replicas_per_shard: u32,
    pub target_cpu_utilization: u32,
    pub target_memory_utilization: u32,
}

impl Default for ReplicaLayerPolicy {
    fn default() -> Self {
        Self {
            min_replicas_per_shard: 1,
            max_replicas_per_shard: 1,
            target_cpu_utilization: 70,
            target_memory_utilization: 80,
        }
    }
}

/// Resource utilization observed across the current data pods.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ObservedUtilization {
    pub cpu_percent: Option<u32>,
    pub memory_percent: Option<u32>,
}

/// A valid whole-layer capacity decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReplicaLayerPlan {
    pub shard_count: u32,
    pub current_replicas_per_shard: u32,
    pub desired_replicas_per_shard: u32,
    pub current_total_pods: u32,
    pub desired_total_pods: u32,
}

impl ReplicaLayerPlan {
    /// A replica-layer change is also a Raft membership change. Callers must
    /// not apply the StatefulSet replica delta before the membership workflow
    /// has admitted/promoted or demoted/removed the affected members.
    pub fn requires_membership_change(self) -> bool {
        self.current_replicas_per_shard != self.desired_replicas_per_shard
    }
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum ReplicaLayerError {
    #[error("shard_count must be greater than zero")]
    ZeroShards,
    #[error("current_replicas_per_shard must be greater than zero")]
    ZeroCurrentReplicas,
    #[error("min_replicas_per_shard must be greater than zero")]
    ZeroMinimum,
    #[error("max_replicas_per_shard must be >= min_replicas_per_shard")]
    InvalidBounds,
    #[error("CPU and memory utilization targets must be in 1..=100")]
    InvalidTarget,
    #[error("replica total exceeds u32")]
    ReplicaOverflow,
}

/// Plan the next whole replica layer using the same utilization ratio as HPA:
/// `ceil(current * observed / target)`, evaluated independently for CPU and
/// memory and taking the larger result. Missing metrics do not force a scale.
/// The result is clamped to the per-shard min/max and total pods are always a
/// multiple of `shard_count`.
pub fn plan_replica_layer(
    shard_count: u32,
    current_replicas_per_shard: u32,
    policy: ReplicaLayerPolicy,
    observed: ObservedUtilization,
) -> Result<ReplicaLayerPlan, ReplicaLayerError> {
    if shard_count == 0 {
        return Err(ReplicaLayerError::ZeroShards);
    }
    if current_replicas_per_shard == 0 {
        return Err(ReplicaLayerError::ZeroCurrentReplicas);
    }
    if policy.min_replicas_per_shard == 0 {
        return Err(ReplicaLayerError::ZeroMinimum);
    }
    if policy.max_replicas_per_shard < policy.min_replicas_per_shard {
        return Err(ReplicaLayerError::InvalidBounds);
    }
    if !(1..=100).contains(&policy.target_cpu_utilization)
        || !(1..=100).contains(&policy.target_memory_utilization)
    {
        return Err(ReplicaLayerError::InvalidTarget);
    }

    let ratio_desired = |observed: u32, target: u32| -> u32 {
        let numerator = current_replicas_per_shard as u64 * observed as u64;
        numerator.div_ceil(target as u64).min(u32::MAX as u64) as u32
    };
    let mut desired = current_replicas_per_shard;
    if let Some(cpu) = observed.cpu_percent {
        desired = desired.max(ratio_desired(cpu, policy.target_cpu_utilization));
    }
    if let Some(memory) = observed.memory_percent {
        desired = desired.max(ratio_desired(memory, policy.target_memory_utilization));
    }
    // Utilization below target may scale down. Recompute from the observed
    // signals instead of seeding with `current`, but only when at least one
    // signal exists; a missing metrics sample must hold steady.
    if observed.cpu_percent.is_some() || observed.memory_percent.is_some() {
        desired = [
            observed
                .cpu_percent
                .map(|v| ratio_desired(v, policy.target_cpu_utilization)),
            observed
                .memory_percent
                .map(|v| ratio_desired(v, policy.target_memory_utilization)),
        ]
        .into_iter()
        .flatten()
        .max()
        .unwrap_or(current_replicas_per_shard);
    }
    desired = desired.clamp(policy.min_replicas_per_shard, policy.max_replicas_per_shard);

    let total = |replicas_per_shard: u32| {
        shard_count
            .checked_mul(replicas_per_shard)
            .ok_or(ReplicaLayerError::ReplicaOverflow)
    };
    Ok(ReplicaLayerPlan {
        shard_count,
        current_replicas_per_shard,
        desired_replicas_per_shard: desired,
        current_total_pods: total(current_replicas_per_shard)?,
        desired_total_pods: total(desired)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(min: u32, max: u32) -> ReplicaLayerPolicy {
        ReplicaLayerPolicy {
            min_replicas_per_shard: min,
            max_replicas_per_shard: max,
            target_cpu_utilization: 70,
            target_memory_utilization: 80,
        }
    }

    #[test]
    fn cpu_scale_out_is_a_whole_shard_layer() {
        let plan = plan_replica_layer(
            3,
            2,
            policy(2, 5),
            ObservedUtilization {
                cpu_percent: Some(90),
                memory_percent: Some(60),
            },
        )
        .unwrap();
        assert_eq!(plan.desired_replicas_per_shard, 3);
        assert_eq!(plan.desired_total_pods, 9);
        assert!(plan.requires_membership_change());
    }

    #[test]
    fn memory_can_drive_the_larger_layer() {
        let plan = plan_replica_layer(
            2,
            2,
            policy(1, 6),
            ObservedUtilization {
                cpu_percent: Some(40),
                memory_percent: Some(170),
            },
        )
        .unwrap();
        assert_eq!(plan.desired_replicas_per_shard, 5);
        assert_eq!(plan.desired_total_pods, 10);
    }

    #[test]
    fn missing_metrics_hold_and_bounds_clamp() {
        let held = plan_replica_layer(4, 3, policy(2, 5), ObservedUtilization::default()).unwrap();
        assert_eq!(held.desired_replicas_per_shard, 3);
        assert_eq!(held.desired_total_pods, 12);

        let floor = plan_replica_layer(
            4,
            3,
            policy(2, 5),
            ObservedUtilization {
                cpu_percent: Some(1),
                memory_percent: Some(1),
            },
        )
        .unwrap();
        assert_eq!(floor.desired_replicas_per_shard, 2);
        assert_eq!(floor.desired_total_pods, 8);
    }

    #[test]
    fn invalid_partial_topologies_are_rejected() {
        assert_eq!(
            plan_replica_layer(0, 1, policy(1, 3), ObservedUtilization::default()),
            Err(ReplicaLayerError::ZeroShards)
        );
        assert_eq!(
            plan_replica_layer(1, 1, policy(3, 2), ObservedUtilization::default()),
            Err(ReplicaLayerError::InvalidBounds)
        );
    }
}
// HANDWRITE-END
