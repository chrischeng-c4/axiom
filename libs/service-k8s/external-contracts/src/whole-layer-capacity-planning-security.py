from __future__ import annotations

from service_k8s.application.capacity import (
    ObservedShardUsage,
    ObservedUtilization,
    ReplicaLayerError,
    ReplicaLayerPolicy,
    ShardSplitError,
    ShardSplitPolicy,
    U32_MAX,
    plan_replica_layer,
    plan_shard_split,
)

MINIMUM_CHECKS = 16

WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX = (
    ("a_zero_shard_count_is_refused", "ZeroShards"),
    ("a_zero_current_replica_count_is_refused", "ZeroCurrentReplicas"),
    ("a_zero_minimum_is_refused", "ZeroMinimum"),
    ("inverted_replica_bounds_are_refused", "InvalidBounds"),
    (
        "an_out_of_range_cpu_target_is_refused_at_both_ends",
        ("InvalidTarget", "InvalidTarget"),
    ),
    (
        "an_out_of_range_memory_target_is_refused_at_both_ends",
        ("InvalidTarget", "InvalidTarget"),
    ),
    ("a_pod_total_that_overflows_u32_is_refused", "ReplicaOverflow"),
    ("the_shard_count_is_checked_before_every_other_replica_input", "ZeroShards"),
    (
        "every_replica_layer_refusal_names_its_reason",
        (
            "shard_count must be greater than zero",
            "current_replicas_per_shard must be greater than zero",
            "min_replicas_per_shard must be greater than zero",
            "max_replicas_per_shard must be >= min_replicas_per_shard",
            "utilization targets must be in 1..=100",
            "replica total exceeds u32",
        ),
    ),
    ("a_zero_shard_count_is_refused_by_the_split_planner", "ZeroShardCount"),
    ("a_zero_split_threshold_is_refused", "ZeroThreshold"),
    ("a_maximum_below_the_current_shard_count_is_refused", "InvalidMaximum"),
    (
        "an_observation_naming_a_shard_outside_the_topology_is_refused",
        "UnknownShard",
    ),
    ("a_split_that_would_overflow_u32_is_refused", "ShardOverflow"),
    (
        "the_split_planner_checks_the_shard_count_before_the_threshold",
        "ZeroShardCount",
    ),
    (
        "every_split_refusal_names_its_reason",
        (
            "current_shard_count must be greater than zero",
            "split_threshold_bytes must be greater than zero",
            "max_shards must be >= current_shard_count",
            "observed shard index is outside the current topology",
            "shard count exceeds u32",
        ),
    ),
)

POLICY = ReplicaLayerPolicy(
    min_replicas_per_shard=1,
    max_replicas_per_shard=10,
    target_cpu_utilization=70,
    target_memory_utilization=80,
)
SPLIT = ShardSplitPolicy()
NONE = ObservedUtilization()


def replica_refusal(
    shard_count: int,
    current: int,
    policy: ReplicaLayerPolicy,
    observed: ObservedUtilization,
) -> str:
    try:
        plan_replica_layer(shard_count, current, policy, observed)
    except ReplicaLayerError as exc:
        return type(exc).__name__
    return "planned"


def replica_reason(
    shard_count: int,
    current: int,
    policy: ReplicaLayerPolicy,
    observed: ObservedUtilization,
) -> str:
    try:
        plan_replica_layer(shard_count, current, policy, observed)
    except ReplicaLayerError as exc:
        return str(exc)
    return "planned"


def split_refusal(
    current: int,
    policy: ShardSplitPolicy,
    observed: tuple[ObservedShardUsage, ...],
) -> str:
    try:
        plan_shard_split(current, policy, observed)
    except ShardSplitError as exc:
        return type(exc).__name__
    return "planned"


def split_reason(
    current: int,
    policy: ShardSplitPolicy,
    observed: tuple[ObservedShardUsage, ...],
) -> str:
    try:
        plan_shard_split(current, policy, observed)
    except ShardSplitError as exc:
        return str(exc)
    return "planned"


def verify_whole_layer_capacity_planning_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. a_zero_shard_count_is_refused
    exp1 = WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[0][1]
    obs1 = replica_refusal(0, 1, POLICY, NONE)
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. a_zero_current_replica_count_is_refused
    exp2 = WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[1][1]
    obs2 = replica_refusal(1, 0, POLICY, NONE)
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. a_zero_minimum_is_refused
    exp3 = WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[2][1]
    obs3 = replica_refusal(1, 1, ReplicaLayerPolicy(0, 5, 70, 80), NONE)
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. inverted_replica_bounds_are_refused
    exp4 = WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[3][1]
    obs4 = replica_refusal(1, 1, ReplicaLayerPolicy(5, 2, 70, 80), NONE)
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. an_out_of_range_cpu_target_is_refused_at_both_ends
    exp5 = WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[4][1]
    obs5 = (
        replica_refusal(1, 1, ReplicaLayerPolicy(1, 10, 0, 80), NONE),
        replica_refusal(1, 1, ReplicaLayerPolicy(1, 10, 101, 80), NONE),
    )
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. an_out_of_range_memory_target_is_refused_at_both_ends
    exp6 = WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[5][1]
    obs6 = (
        replica_refusal(1, 1, ReplicaLayerPolicy(1, 10, 70, 0), NONE),
        replica_refusal(1, 1, ReplicaLayerPolicy(1, 10, 70, 101), NONE),
    )
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. a_pod_total_that_overflows_u32_is_refused
    exp7 = WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[6][1]
    obs7 = replica_refusal(U32_MAX, 2, ReplicaLayerPolicy(1, U32_MAX, 70, 80), NONE)
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. the_shard_count_is_checked_before_every_other_replica_input
    exp8 = WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[7][1]
    obs8 = replica_refusal(0, 0, ReplicaLayerPolicy(0, 0, 0, 0), NONE)
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. every_replica_layer_refusal_names_its_reason
    exp9 = WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[8][1]
    obs9 = (
        replica_reason(0, 1, POLICY, NONE),
        replica_reason(1, 0, POLICY, NONE),
        replica_reason(1, 1, ReplicaLayerPolicy(0, 5, 70, 80), NONE),
        replica_reason(1, 1, ReplicaLayerPolicy(5, 2, 70, 80), NONE),
        replica_reason(1, 1, ReplicaLayerPolicy(1, 10, 0, 80), NONE),
        replica_reason(U32_MAX, 2, ReplicaLayerPolicy(1, U32_MAX, 70, 80), NONE),
    )
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. a_zero_shard_count_is_refused_by_the_split_planner
    exp10 = WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[9][1]
    obs10 = split_refusal(0, SPLIT, ())
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. a_zero_split_threshold_is_refused
    exp11 = WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[10][1]
    obs11 = split_refusal(1, ShardSplitPolicy(split_threshold_bytes=0), ())
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. a_maximum_below_the_current_shard_count_is_refused
    exp12 = WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[11][1]
    obs12 = split_refusal(4, ShardSplitPolicy(max_shards=2), ())
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    # 13. an_observation_naming_a_shard_outside_the_topology_is_refused
    exp13 = WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[12][1]
    obs13 = split_refusal(4, SPLIT, (ObservedShardUsage(4, 500),))
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[12][0],
            "expected": exp13,
            "observed": obs13,
            "passed": obs13 == exp13,
        }
    )

    # 14. a_split_that_would_overflow_u32_is_refused
    exp14 = WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[13][1]
    obs14 = split_refusal(
        U32_MAX,
        ShardSplitPolicy(split_threshold_bytes=100),
        (ObservedShardUsage(0, 500),),
    )
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[13][0],
            "expected": exp14,
            "observed": obs14,
            "passed": obs14 == exp14,
        }
    )

    # 15. the_split_planner_checks_the_shard_count_before_the_threshold
    exp15 = WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[14][1]
    obs15 = split_refusal(0, ShardSplitPolicy(split_threshold_bytes=0), ())
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[14][0],
            "expected": exp15,
            "observed": obs15,
            "passed": obs15 == exp15,
        }
    )

    # 16. every_split_refusal_names_its_reason
    exp16 = WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[15][1]
    obs16 = (
        split_reason(0, SPLIT, ()),
        split_reason(1, ShardSplitPolicy(split_threshold_bytes=0), ()),
        split_reason(4, ShardSplitPolicy(max_shards=2), ()),
        split_reason(4, SPLIT, (ObservedShardUsage(4, 500),)),
        split_reason(
            U32_MAX,
            ShardSplitPolicy(split_threshold_bytes=100),
            (ObservedShardUsage(0, 500),),
        ),
    )
    checks.append(
        {
            "name": WHOLE_LAYER_CAPACITY_PLANNING_SECURITY_MATRIX[15][0],
            "expected": exp16,
            "observed": obs16,
            "passed": obs16 == exp16,
        }
    )

    return {
        "case_id": "whole-layer-capacity-planning-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
