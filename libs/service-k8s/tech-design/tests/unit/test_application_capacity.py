from __future__ import annotations

import sys
import unittest

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from service_k8s.application.capacity import (
    DEFAULT_CPU_REQUEST,
    DEFAULT_MEMORY_REQUEST,
    DEFAULT_SHARD_SPLIT_THRESHOLD_BYTES,
    U32_MAX,
    InvalidBounds,
    InvalidMaximum,
    InvalidTarget,
    ObservedShardUsage,
    ObservedUtilization,
    ReplicaLayerError,
    ReplicaLayerPlan,
    ReplicaLayerPolicy,
    ReplicaOverflow,
    ShardOverflow,
    ShardSplitError,
    ShardSplitPlan,
    ShardSplitPolicy,
    UnknownShard,
    ZeroCurrentReplicas,
    ZeroMinimum,
    ZeroShardCount,
    ZeroShards,
    ZeroThreshold,
    _ceil_div,
    _ratio_desired,
    plan_replica_layer,
    plan_shard_split,
    resource_request_or_default,
)


class TestApplicationCapacity(unittest.TestCase):
    # --- Resource defaults (3) ---
    def test_resource_default_empty_string(self) -> None:
        self.assertEqual(resource_request_or_default("", "4Gi"), "4Gi")

    def test_resource_default_whitespace_only(self) -> None:
        self.assertEqual(resource_request_or_default("   ", "4Gi"), "4Gi")

    def test_resource_default_preserves_value_and_whitespace(self) -> None:
        self.assertEqual(resource_request_or_default(" 2 ", "1"), " 2 ")

    # --- Replica ladder (7) ---
    def test_replica_ladder_zero_shards(self) -> None:
        pol = ReplicaLayerPolicy()
        obs = ObservedUtilization()
        with self.assertRaises(ZeroShards) as cm:
            plan_replica_layer(0, 1, pol, obs)
        self.assertIn("shard_count must be greater than zero", str(cm.exception))

    def test_replica_ladder_zero_current_replicas(self) -> None:
        pol = ReplicaLayerPolicy()
        obs = ObservedUtilization()
        with self.assertRaises(ZeroCurrentReplicas) as cm:
            plan_replica_layer(1, 0, pol, obs)
        self.assertIn("current_replicas_per_shard must be greater than zero", str(cm.exception))

    def test_replica_ladder_zero_minimum(self) -> None:
        pol = ReplicaLayerPolicy(min_replicas_per_shard=0, max_replicas_per_shard=5)
        obs = ObservedUtilization()
        with self.assertRaises(ZeroMinimum) as cm:
            plan_replica_layer(1, 1, pol, obs)
        self.assertIn("min_replicas_per_shard must be greater than zero", str(cm.exception))

    def test_replica_ladder_invalid_bounds(self) -> None:
        pol = ReplicaLayerPolicy(min_replicas_per_shard=5, max_replicas_per_shard=2)
        obs = ObservedUtilization()
        with self.assertRaises(InvalidBounds) as cm:
            plan_replica_layer(1, 1, pol, obs)
        self.assertIn("max_replicas_per_shard must be >= min_replicas_per_shard", str(cm.exception))

    def test_replica_ladder_invalid_target(self) -> None:
        pol = ReplicaLayerPolicy(target_cpu_utilization=0)
        obs = ObservedUtilization()
        with self.assertRaises(InvalidTarget) as cm:
            plan_replica_layer(1, 1, pol, obs)
        self.assertIn("utilization targets must be in 1..=100", str(cm.exception))

    def test_replica_ladder_overflow(self) -> None:
        pol = ReplicaLayerPolicy(min_replicas_per_shard=1, max_replicas_per_shard=U32_MAX)
        obs = ObservedUtilization()
        with self.assertRaises(ReplicaOverflow) as cm:
            plan_replica_layer(U32_MAX, 2, pol, obs)
        self.assertIn("replica total exceeds u32", str(cm.exception))

    def test_replica_ladder_order_zero_shards_first(self) -> None:
        pol = ReplicaLayerPolicy()
        obs = ObservedUtilization()
        with self.assertRaises(ZeroShards):
            plan_replica_layer(0, 0, pol, obs)

    # --- Replica arithmetic (10 table rows + membership change assertions) ---
    def test_replica_row_1(self) -> None:
        pol = ReplicaLayerPolicy(min_replicas_per_shard=1, max_replicas_per_shard=10, target_cpu_utilization=70, target_memory_utilization=80)
        obs = ObservedUtilization(cpu_percent=140, memory_percent=None)
        plan = plan_replica_layer(3, 2, pol, obs)
        self.assertEqual(plan.desired_replicas_per_shard, 4)
        self.assertEqual(plan.desired_total_pods, 12)

    def test_replica_row_2(self) -> None:
        pol = ReplicaLayerPolicy(min_replicas_per_shard=1, max_replicas_per_shard=10, target_cpu_utilization=70, target_memory_utilization=80)
        obs = ObservedUtilization(cpu_percent=None, memory_percent=160)
        plan = plan_replica_layer(3, 2, pol, obs)
        self.assertEqual(plan.desired_replicas_per_shard, 4)
        self.assertEqual(plan.desired_total_pods, 12)

    def test_replica_row_3_ceiling_witness(self) -> None:
        pol = ReplicaLayerPolicy(min_replicas_per_shard=1, max_replicas_per_shard=10, target_cpu_utilization=70, target_memory_utilization=80)
        obs = ObservedUtilization(cpu_percent=100, memory_percent=None)
        plan = plan_replica_layer(3, 3, pol, obs)
        self.assertEqual(plan.desired_replicas_per_shard, 5)
        self.assertEqual(plan.desired_total_pods, 15)

    def test_replica_row_4(self) -> None:
        pol = ReplicaLayerPolicy(min_replicas_per_shard=1, max_replicas_per_shard=10, target_cpu_utilization=70, target_memory_utilization=80)
        obs = ObservedUtilization(cpu_percent=140, memory_percent=160)
        plan = plan_replica_layer(3, 2, pol, obs)
        self.assertEqual(plan.desired_replicas_per_shard, 4)
        self.assertEqual(plan.desired_total_pods, 12)

    def test_replica_row_5(self) -> None:
        pol = ReplicaLayerPolicy(min_replicas_per_shard=1, max_replicas_per_shard=10, target_cpu_utilization=70, target_memory_utilization=80)
        obs = ObservedUtilization(cpu_percent=140, memory_percent=40)
        plan = plan_replica_layer(3, 2, pol, obs)
        self.assertEqual(plan.desired_replicas_per_shard, 4)
        self.assertEqual(plan.desired_total_pods, 12)

    def test_replica_row_6_scale_down(self) -> None:
        pol = ReplicaLayerPolicy(min_replicas_per_shard=1, max_replicas_per_shard=10, target_cpu_utilization=70, target_memory_utilization=80)
        obs = ObservedUtilization(cpu_percent=35, memory_percent=40)
        plan = plan_replica_layer(3, 4, pol, obs)
        self.assertEqual(plan.desired_replicas_per_shard, 2)
        self.assertEqual(plan.desired_total_pods, 6)

    def test_replica_row_7_hold_steady(self) -> None:
        pol = ReplicaLayerPolicy(min_replicas_per_shard=1, max_replicas_per_shard=10, target_cpu_utilization=70, target_memory_utilization=80)
        obs = ObservedUtilization(cpu_percent=None, memory_percent=None)
        plan = plan_replica_layer(3, 4, pol, obs)
        self.assertEqual(plan.desired_replicas_per_shard, 4)
        self.assertEqual(plan.desired_total_pods, 12)

    def test_replica_row_8_clamp_max(self) -> None:
        pol = ReplicaLayerPolicy(min_replicas_per_shard=1, max_replicas_per_shard=3, target_cpu_utilization=70, target_memory_utilization=80)
        obs = ObservedUtilization(cpu_percent=700, memory_percent=None)
        plan = plan_replica_layer(3, 2, pol, obs)
        self.assertEqual(plan.desired_replicas_per_shard, 3)
        self.assertEqual(plan.desired_total_pods, 9)

    def test_replica_row_9_clamp_min(self) -> None:
        pol = ReplicaLayerPolicy(min_replicas_per_shard=2, max_replicas_per_shard=10, target_cpu_utilization=70, target_memory_utilization=80)
        obs = ObservedUtilization(cpu_percent=1, memory_percent=None)
        plan = plan_replica_layer(3, 4, pol, obs)
        self.assertEqual(plan.desired_replicas_per_shard, 2)
        self.assertEqual(plan.desired_total_pods, 6)

    def test_replica_row_10(self) -> None:
        pol = ReplicaLayerPolicy(min_replicas_per_shard=1, max_replicas_per_shard=5, target_cpu_utilization=100, target_memory_utilization=100)
        obs = ObservedUtilization(cpu_percent=100, memory_percent=100)
        plan = plan_replica_layer(1, 1, pol, obs)
        self.assertEqual(plan.desired_replicas_per_shard, 1)
        self.assertEqual(plan.desired_total_pods, 1)

    def test_replica_membership_change_flags(self) -> None:
        pol = ReplicaLayerPolicy(min_replicas_per_shard=1, max_replicas_per_shard=10, target_cpu_utilization=70, target_memory_utilization=80)
        p1 = plan_replica_layer(3, 2, pol, ObservedUtilization(140, None))
        self.assertTrue(p1.requires_membership_change())

        p6 = plan_replica_layer(3, 4, pol, ObservedUtilization(35, 40))
        self.assertTrue(p6.requires_membership_change())

        p7 = plan_replica_layer(3, 4, pol, ObservedUtilization(None, None))
        self.assertFalse(p7.requires_membership_change())

    # --- Shard-split ladder (6) ---
    def test_shard_ladder_zero_shard_count(self) -> None:
        pol = ShardSplitPolicy()
        with self.assertRaises(ZeroShardCount) as cm:
            plan_shard_split(0, pol, ())
        self.assertIn("current_shard_count must be greater than zero", str(cm.exception))

    def test_shard_ladder_zero_threshold(self) -> None:
        pol = ShardSplitPolicy(split_threshold_bytes=0)
        with self.assertRaises(ZeroThreshold) as cm:
            plan_shard_split(1, pol, ())
        self.assertIn("split_threshold_bytes must be greater than zero", str(cm.exception))

    def test_shard_ladder_invalid_maximum(self) -> None:
        pol = ShardSplitPolicy(max_shards=2)
        with self.assertRaises(InvalidMaximum) as cm:
            plan_shard_split(4, pol, ())
        self.assertIn("max_shards must be >= current_shard_count", str(cm.exception))

    def test_shard_ladder_unknown_shard(self) -> None:
        pol = ShardSplitPolicy()
        obs = (ObservedShardUsage(shard_index=4, durable_bytes=500),)
        with self.assertRaises(UnknownShard) as cm:
            plan_shard_split(4, pol, obs)
        self.assertIn("observed shard index is outside the current topology", str(cm.exception))

    def test_shard_ladder_overflow(self) -> None:
        pol = ShardSplitPolicy(split_threshold_bytes=100)
        obs = (ObservedShardUsage(shard_index=0, durable_bytes=500),)
        with self.assertRaises(ShardOverflow) as cm:
            plan_shard_split(U32_MAX, pol, obs)
        self.assertIn("shard count exceeds u32", str(cm.exception))

    def test_shard_ladder_order_zero_shard_count_first(self) -> None:
        pol = ShardSplitPolicy(split_threshold_bytes=0)
        with self.assertRaises(ZeroShardCount):
            plan_shard_split(0, pol, ())

    # --- Shard-split decision (9) ---
    def test_split_exact_threshold_bytes_no_split(self) -> None:
        pol = ShardSplitPolicy()
        obs = (ObservedShardUsage(0, 1073741824),)
        plan = plan_shard_split(1, pol, obs)
        self.assertFalse(plan.requires_split())
        self.assertEqual(plan.desired_shard_count, 1)

    def test_split_one_byte_over_threshold_causes_split(self) -> None:
        pol = ShardSplitPolicy()
        obs = (ObservedShardUsage(0, 1073741825),)
        plan = plan_shard_split(1, pol, obs)
        self.assertTrue(plan.requires_split())
        self.assertEqual(plan.desired_shard_count, 2)

    def test_split_empty_observed_no_split(self) -> None:
        pol = ShardSplitPolicy()
        plan = plan_shard_split(2, pol, ())
        self.assertIsNone(plan.busiest_shard)
        self.assertFalse(plan.requires_split())
        self.assertEqual(plan.desired_shard_count, 2)

    def test_split_tie_break_lowest_index(self) -> None:
        pol = ShardSplitPolicy()
        obs = (
            ObservedShardUsage(shard_index=2, durable_bytes=2000000000),
            ObservedShardUsage(shard_index=1, durable_bytes=2000000000),
        )
        plan = plan_shard_split(3, pol, obs)
        self.assertIsNotNone(plan.busiest_shard)
        assert plan.busiest_shard is not None
        self.assertEqual(plan.busiest_shard.shard_index, 1)

    def test_split_at_most_one_shard_per_call(self) -> None:
        pol = ShardSplitPolicy()
        obs = (
            ObservedShardUsage(0, 5000000000),
            ObservedShardUsage(1, 5000000000),
            ObservedShardUsage(2, 5000000000),
        )
        plan = plan_shard_split(3, pol, obs)
        self.assertEqual(plan.desired_shard_count, 4)

    def test_split_max_shards_reached_over_threshold(self) -> None:
        pol = ShardSplitPolicy(max_shards=4)
        obs = (ObservedShardUsage(0, 5000000000),)
        plan = plan_shard_split(4, pol, obs)
        self.assertTrue(plan.max_shards_reached)
        self.assertEqual(plan.desired_shard_count, 4)
        self.assertFalse(plan.requires_split())

    def test_split_max_shards_reached_under_threshold(self) -> None:
        pol = ShardSplitPolicy(max_shards=4)
        obs = (ObservedShardUsage(0, 500),)
        plan = plan_shard_split(4, pol, obs)
        self.assertTrue(plan.max_shards_reached)
        self.assertEqual(plan.desired_shard_count, 4)

    def test_split_threshold_bytes_echoed(self) -> None:
        pol = ShardSplitPolicy(split_threshold_bytes=5000)
        plan = plan_shard_split(1, pol, ())
        self.assertEqual(plan.split_threshold_bytes, 5000)

    def test_split_custom_threshold(self) -> None:
        pol = ShardSplitPolicy(split_threshold_bytes=5000)
        obs = (ObservedShardUsage(0, 5001),)
        plan = plan_shard_split(1, pol, obs)
        self.assertTrue(plan.requires_split())
        self.assertEqual(plan.desired_shard_count, 2)

    # --- Immutability (2) ---
    def test_replica_plan_frozen(self) -> None:
        pol = ReplicaLayerPolicy()
        obs = ObservedUtilization()
        plan = plan_replica_layer(1, 1, pol, obs)
        with self.assertRaises(Exception):
            plan.desired_replicas_per_shard = 9  # type: ignore[misc]

    def test_shard_plan_frozen(self) -> None:
        pol = ShardSplitPolicy()
        plan = plan_shard_split(1, pol, ())
        with self.assertRaises(Exception):
            plan.desired_shard_count = 9  # type: ignore[misc]


if __name__ == "__main__":
    unittest.main()
