// HANDWRITE-BEGIN gap="missing-generator:stateful-capacity-policy" tracker="#1644" reason="Shared request-only resource defaults, dedicated-node placement, whole-layer CPU/memory replica planning, and one-GiB disk-driven shard-split planning for long-running stateful services."
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
//! Storage pressure is a separate axis: [`plan_shard_split`] plans one physical
//! shard at a time from the busiest shard's durable bytes. The service still
//! owns its domain-safe routing-map cutover and data movement; this library
//! never treats adding StatefulSet pods as a completed shard split.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Default CPU request emitted for a data pod when the service leaves it empty.
pub const DEFAULT_CPU_REQUEST: &str = "1";
/// Default memory request emitted for a data pod when the service leaves it empty.
pub const DEFAULT_MEMORY_REQUEST: &str = "4Gi";
/// Default per-shard durable-byte threshold. A shard split is planned only
/// when observed usage is strictly greater than one GiB.
pub const DEFAULT_SHARD_SPLIT_THRESHOLD_BYTES: u64 = 1024 * 1024 * 1024;

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

/// Storage-pressure policy for adding physical shards.
///
/// The default is intentionally small enough for low-cost integration proof:
/// a busiest shard must exceed one GiB. Production deployments may raise the
/// threshold while preserving the same one-shard-at-a-time transition.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShardSplitPolicy {
    pub split_threshold_bytes: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_shards: Option<u32>,
}

impl Default for ShardSplitPolicy {
    fn default() -> Self {
        Self {
            split_threshold_bytes: DEFAULT_SHARD_SPLIT_THRESHOLD_BYTES,
            max_shards: None,
        }
    }
}

/// Durable bytes observed for one physical shard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservedShardUsage {
    pub shard_index: u32,
    pub durable_bytes: u64,
}

/// One safe topology step. A plan grows by at most one physical shard; the
/// service's domain actuator must complete routing/data migration before
/// asking for another plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShardSplitPlan {
    pub current_shard_count: u32,
    pub desired_shard_count: u32,
    pub split_threshold_bytes: u64,
    pub busiest_shard: Option<ObservedShardUsage>,
    pub max_shards_reached: bool,
}

impl ShardSplitPlan {
    pub fn requires_split(self) -> bool {
        self.desired_shard_count > self.current_shard_count
    }
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

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum ShardSplitError {
    #[error("current_shard_count must be greater than zero")]
    ZeroShards,
    #[error("split_threshold_bytes must be greater than zero")]
    ZeroThreshold,
    #[error("max_shards must be >= current_shard_count")]
    InvalidMaximum,
    #[error("observed shard index is outside the current topology")]
    UnknownShard,
    #[error("shard count exceeds u32")]
    ShardOverflow,
}

/// Plan at most one new physical shard from live durable-byte observations.
///
/// The threshold is strict: exactly one GiB holds steady, while one GiB plus
/// one byte plans `shard_count + 1`. Ties choose the lowest shard index so the
/// plan is deterministic. This is deliberately a pure decision; the caller
/// must execute its domain-specific routing-map, migration, fencing, and Raft
/// membership workflow before changing the workload topology.
pub fn plan_shard_split(
    current_shard_count: u32,
    policy: ShardSplitPolicy,
    observed: &[ObservedShardUsage],
) -> Result<ShardSplitPlan, ShardSplitError> {
    if current_shard_count == 0 {
        return Err(ShardSplitError::ZeroShards);
    }
    if policy.split_threshold_bytes == 0 {
        return Err(ShardSplitError::ZeroThreshold);
    }
    if policy
        .max_shards
        .is_some_and(|max| max < current_shard_count)
    {
        return Err(ShardSplitError::InvalidMaximum);
    }
    if observed
        .iter()
        .any(|usage| usage.shard_index >= current_shard_count)
    {
        return Err(ShardSplitError::UnknownShard);
    }

    let busiest_shard = observed.iter().copied().max_by(|left, right| {
        left.durable_bytes
            .cmp(&right.durable_bytes)
            .then_with(|| right.shard_index.cmp(&left.shard_index))
    });
    let max_shards_reached = policy
        .max_shards
        .is_some_and(|max| current_shard_count >= max);
    let threshold_crossed =
        busiest_shard.is_some_and(|usage| usage.durable_bytes > policy.split_threshold_bytes);
    let desired_shard_count = if threshold_crossed && !max_shards_reached {
        current_shard_count
            .checked_add(1)
            .ok_or(ShardSplitError::ShardOverflow)?
    } else {
        current_shard_count
    };

    Ok(ShardSplitPlan {
        current_shard_count,
        desired_shard_count,
        split_threshold_bytes: policy.split_threshold_bytes,
        busiest_shard,
        max_shards_reached,
    })
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
    fn disk_split_threshold_is_strictly_greater_than_one_gib() {
        let policy = ShardSplitPolicy::default();
        let at_threshold = plan_shard_split(
            1,
            policy,
            &[ObservedShardUsage {
                shard_index: 0,
                durable_bytes: DEFAULT_SHARD_SPLIT_THRESHOLD_BYTES,
            }],
        )
        .unwrap();
        assert!(!at_threshold.requires_split());

        let crossed = plan_shard_split(
            1,
            policy,
            &[ObservedShardUsage {
                shard_index: 0,
                durable_bytes: DEFAULT_SHARD_SPLIT_THRESHOLD_BYTES + 1,
            }],
        )
        .unwrap();
        assert!(crossed.requires_split());
        assert_eq!(crossed.desired_shard_count, 2);
    }

    #[test]
    fn disk_split_adds_one_shard_and_honors_the_ceiling() {
        let policy = ShardSplitPolicy {
            split_threshold_bytes: 100,
            max_shards: Some(4),
        };
        let usage = [
            ObservedShardUsage {
                shard_index: 0,
                durable_bytes: 101,
            },
            ObservedShardUsage {
                shard_index: 1,
                durable_bytes: 500,
            },
            ObservedShardUsage {
                shard_index: 2,
                durable_bytes: 500,
            },
        ];
        let split = plan_shard_split(3, policy, &usage).unwrap();
        assert_eq!(split.desired_shard_count, 4);
        assert_eq!(split.busiest_shard.unwrap().shard_index, 1);

        let at_limit = plan_shard_split(
            4,
            policy,
            &[ObservedShardUsage {
                shard_index: 1,
                durable_bytes: 500,
            }],
        )
        .unwrap();
        assert!(!at_limit.requires_split());
        assert!(at_limit.max_shards_reached);
    }

    #[test]
    fn disk_split_rejects_invalid_policy_and_observations() {
        assert_eq!(
            plan_shard_split(0, ShardSplitPolicy::default(), &[]),
            Err(ShardSplitError::ZeroShards)
        );
        assert_eq!(
            plan_shard_split(
                1,
                ShardSplitPolicy {
                    split_threshold_bytes: 0,
                    max_shards: None,
                },
                &[]
            ),
            Err(ShardSplitError::ZeroThreshold)
        );
        assert_eq!(
            plan_shard_split(
                2,
                ShardSplitPolicy {
                    split_threshold_bytes: 1,
                    max_shards: Some(1),
                },
                &[]
            ),
            Err(ShardSplitError::InvalidMaximum)
        );
        assert_eq!(
            plan_shard_split(
                1,
                ShardSplitPolicy::default(),
                &[ObservedShardUsage {
                    shard_index: 1,
                    durable_bytes: 1,
                }]
            ),
            Err(ShardSplitError::UnknownShard)
        );
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
