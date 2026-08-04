from __future__ import annotations

from service_k8s.application.capacity import (
    DEFAULT_CPU_REQUEST,
    DEFAULT_MEMORY_REQUEST,
    ObservedShardUsage,
    ObservedUtilization,
    ReplicaLayerPolicy,
    ShardSplitPlan,
    ShardSplitPolicy,
    plan_replica_layer,
    plan_shard_split,
    resource_request_or_default,
)

MINIMUM_CHECKS = 17

WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX = (
    ("cpu_pressure_scales_the_whole_layer", (4, 6, 12)),
    ("memory_pressure_alone_also_drives_the_layer", (4, 12)),
    ("the_two_axes_are_taken_independently_and_the_larger_wins", (4, 4)),
    ("the_ratio_rounds_up_rather_than_truncating", (5, 15)),
    ("a_total_is_always_a_whole_number_of_shard_layers", (4, 8, 12, 16, 20)),
    ("falling_utilization_scales_the_layer_down", (2, 6)),
    ("a_metrics_gap_holds_the_layer_steady", (4, 12, False)),
    ("one_surviving_signal_is_still_enough_to_scale_down", (1, 2)),
    ("the_upper_bound_clamps_the_layer", (3, 9)),
    ("the_lower_bound_clamps_the_layer", (2, 6)),
    ("a_replica_change_is_a_membership_change_and_a_hold_is_not", (True, False)),
    ("the_split_threshold_is_strict_at_exactly_one_gibibyte", (False, 1, True, 2)),
    ("a_split_adds_exactly_one_shard_even_when_every_shard_is_over", (3, 4)),
    ("ties_choose_the_lowest_shard_index_in_either_observation_order", (1, 1)),
    ("the_shard_ceiling_holds_a_shard_that_is_over_the_threshold", (True, 4, False)),
    ("no_observation_means_no_busiest_shard_and_no_split", (-1, False, 2)),
    ("an_empty_resource_request_falls_back_to_the_shared_default", ("1", "4Gi", " 2 ")),
)

POLICY = ReplicaLayerPolicy(
    min_replicas_per_shard=1,
    max_replicas_per_shard=10,
    target_cpu_utilization=70,
    target_memory_utilization=80,
)
SPLIT = ShardSplitPolicy()


def busiest_index(plan: ShardSplitPlan) -> int:
    busiest = plan.busiest_shard
    if busiest is None:
        return -1
    return busiest.shard_index


def verify_whole_layer_capacity_planning_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. cpu_pressure_scales_the_whole_layer
    exp1 = WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[0][1]
    plan1 = plan_replica_layer(3, 2, POLICY, ObservedUtilization(cpu_percent=140))
    obs1 = (
        plan1.desired_replicas_per_shard,
        plan1.current_total_pods,
        plan1.desired_total_pods,
    )
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. memory_pressure_alone_also_drives_the_layer
    exp2 = WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[1][1]
    plan2 = plan_replica_layer(3, 2, POLICY, ObservedUtilization(memory_percent=160))
    obs2 = (plan2.desired_replicas_per_shard, plan2.desired_total_pods)
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. the_two_axes_are_taken_independently_and_the_larger_wins
    exp3 = WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[2][1]
    plan3a = plan_replica_layer(
        3, 2, POLICY, ObservedUtilization(cpu_percent=140, memory_percent=40)
    )
    plan3b = plan_replica_layer(
        3, 2, POLICY, ObservedUtilization(cpu_percent=35, memory_percent=160)
    )
    obs3 = (plan3a.desired_replicas_per_shard, plan3b.desired_replicas_per_shard)
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. the_ratio_rounds_up_rather_than_truncating
    exp4 = WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[3][1]
    plan4 = plan_replica_layer(3, 3, POLICY, ObservedUtilization(cpu_percent=100))
    obs4 = (plan4.desired_replicas_per_shard, plan4.desired_total_pods)
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. a_total_is_always_a_whole_number_of_shard_layers
    exp5 = WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[4][1]
    obs5 = (
        plan_replica_layer(
            1, 2, POLICY, ObservedUtilization(cpu_percent=140)
        ).desired_total_pods,
        plan_replica_layer(
            2, 2, POLICY, ObservedUtilization(cpu_percent=140)
        ).desired_total_pods,
        plan_replica_layer(
            3, 2, POLICY, ObservedUtilization(cpu_percent=140)
        ).desired_total_pods,
        plan_replica_layer(
            4, 2, POLICY, ObservedUtilization(cpu_percent=140)
        ).desired_total_pods,
        plan_replica_layer(
            5, 2, POLICY, ObservedUtilization(cpu_percent=140)
        ).desired_total_pods,
    )
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. falling_utilization_scales_the_layer_down
    exp6 = WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[5][1]
    plan6 = plan_replica_layer(
        3, 4, POLICY, ObservedUtilization(cpu_percent=35, memory_percent=40)
    )
    obs6 = (plan6.desired_replicas_per_shard, plan6.desired_total_pods)
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. a_metrics_gap_holds_the_layer_steady
    exp7 = WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[6][1]
    plan7 = plan_replica_layer(3, 4, POLICY, ObservedUtilization())
    obs7 = (
        plan7.desired_replicas_per_shard,
        plan7.desired_total_pods,
        plan7.requires_membership_change(),
    )
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. one_surviving_signal_is_still_enough_to_scale_down
    exp8 = WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[7][1]
    plan8 = plan_replica_layer(2, 3, POLICY, ObservedUtilization(cpu_percent=10))
    obs8 = (plan8.desired_replicas_per_shard, plan8.desired_total_pods)
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. the_upper_bound_clamps_the_layer
    exp9 = WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[8][1]
    plan9 = plan_replica_layer(
        3, 2, ReplicaLayerPolicy(1, 3, 70, 80), ObservedUtilization(cpu_percent=700)
    )
    obs9 = (plan9.desired_replicas_per_shard, plan9.desired_total_pods)
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. the_lower_bound_clamps_the_layer
    exp10 = WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[9][1]
    plan10 = plan_replica_layer(
        3, 4, ReplicaLayerPolicy(2, 10, 70, 80), ObservedUtilization(cpu_percent=1)
    )
    obs10 = (plan10.desired_replicas_per_shard, plan10.desired_total_pods)
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. a_replica_change_is_a_membership_change_and_a_hold_is_not
    exp11 = WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[10][1]
    obs11 = (
        plan_replica_layer(
            3, 2, POLICY, ObservedUtilization(cpu_percent=140)
        ).requires_membership_change(),
        plan_replica_layer(
            3, 4, POLICY, ObservedUtilization()
        ).requires_membership_change(),
    )
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. the_split_threshold_is_strict_at_exactly_one_gibibyte
    exp12 = WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[11][1]
    plan12a = plan_shard_split(1, SPLIT, (ObservedShardUsage(0, 1073741824),))
    plan12b = plan_shard_split(1, SPLIT, (ObservedShardUsage(0, 1073741825),))
    obs12 = (
        plan12a.requires_split(),
        plan12a.desired_shard_count,
        plan12b.requires_split(),
        plan12b.desired_shard_count,
    )
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    # 13. a_split_adds_exactly_one_shard_even_when_every_shard_is_over
    exp13 = WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[12][1]
    plan13 = plan_shard_split(
        3,
        SPLIT,
        (
            ObservedShardUsage(0, 5000000000),
            ObservedShardUsage(1, 5000000000),
            ObservedShardUsage(2, 5000000000),
        ),
    )
    obs13 = (plan13.current_shard_count, plan13.desired_shard_count)
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[12][0],
            "expected": exp13,
            "observed": obs13,
            "passed": obs13 == exp13,
        }
    )

    # 14. ties_choose_the_lowest_shard_index_in_either_observation_order
    exp14 = WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[13][1]
    plan14a = plan_shard_split(
        3,
        SPLIT,
        (ObservedShardUsage(2, 2000000000), ObservedShardUsage(1, 2000000000)),
    )
    plan14b = plan_shard_split(
        3,
        SPLIT,
        (ObservedShardUsage(1, 2000000000), ObservedShardUsage(2, 2000000000)),
    )
    obs14 = (busiest_index(plan14a), busiest_index(plan14b))
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[13][0],
            "expected": exp14,
            "observed": obs14,
            "passed": obs14 == exp14,
        }
    )

    # 15. the_shard_ceiling_holds_a_shard_that_is_over_the_threshold
    exp15 = WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[14][1]
    plan15 = plan_shard_split(
        4, ShardSplitPolicy(max_shards=4), (ObservedShardUsage(0, 5000000000),)
    )
    obs15 = (
        plan15.max_shards_reached,
        plan15.desired_shard_count,
        plan15.requires_split(),
    )
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[14][0],
            "expected": exp15,
            "observed": obs15,
            "passed": obs15 == exp15,
        }
    )

    # 16. no_observation_means_no_busiest_shard_and_no_split
    exp16 = WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[15][1]
    plan16 = plan_shard_split(2, SPLIT, ())
    obs16 = (
        busiest_index(plan16),
        plan16.requires_split(),
        plan16.desired_shard_count,
    )
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[15][0],
            "expected": exp16,
            "observed": obs16,
            "passed": obs16 == exp16,
        }
    )

    # 17. an_empty_resource_request_falls_back_to_the_shared_default
    exp17 = WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[16][1]
    obs17 = (
        resource_request_or_default("", DEFAULT_CPU_REQUEST),
        resource_request_or_default("   ", DEFAULT_MEMORY_REQUEST),
        resource_request_or_default(" 2 ", DEFAULT_CPU_REQUEST),
    )
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_BEHAVIOR_MATRIX[16][0],
            "expected": exp17,
            "observed": obs17,
            "passed": obs17 == exp17,
        }
    )

    return {
        "case_id": "whole-layer-capacity-planning-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
