from __future__ import annotations

from dataclasses import dataclass
from typing import Final

DEFAULT_CPU_REQUEST: Final[str] = "1"
DEFAULT_MEMORY_REQUEST: Final[str] = "4Gi"
DEFAULT_SHARD_SPLIT_THRESHOLD_BYTES: Final[int] = 1073741824
U32_MAX: Final[int] = 4294967295


def resource_request_or_default(value: str, default: str) -> str:
    if not value.strip():
        return default
    return value


@dataclass(frozen=True)
class ReplicaLayerPolicy:
    min_replicas_per_shard: int = 1
    max_replicas_per_shard: int = 1
    target_cpu_utilization: int = 70
    target_memory_utilization: int = 80


@dataclass(frozen=True)
class ObservedUtilization:
    cpu_percent: int | None = None
    memory_percent: int | None = None


@dataclass(frozen=True)
class ReplicaLayerPlan:
    shard_count: int
    current_replicas_per_shard: int
    desired_replicas_per_shard: int
    current_total_pods: int
    desired_total_pods: int

    def requires_membership_change(self) -> bool:
        return self.current_replicas_per_shard != self.desired_replicas_per_shard


class ReplicaLayerError(Exception):
    """Base for every replica-layer planning refusal."""


class ZeroShards(ReplicaLayerError):
    def __init__(self) -> None:
        super().__init__("shard_count must be greater than zero")


class ZeroCurrentReplicas(ReplicaLayerError):
    def __init__(self) -> None:
        super().__init__("current_replicas_per_shard must be greater than zero")


class ZeroMinimum(ReplicaLayerError):
    def __init__(self) -> None:
        super().__init__("min_replicas_per_shard must be greater than zero")


class InvalidBounds(ReplicaLayerError):
    def __init__(self) -> None:
        super().__init__("max_replicas_per_shard must be >= min_replicas_per_shard")


class InvalidTarget(ReplicaLayerError):
    def __init__(self) -> None:
        super().__init__("utilization targets must be in 1..=100")


class ReplicaOverflow(ReplicaLayerError):
    def __init__(self) -> None:
        super().__init__("replica total exceeds u32")


def _ceil_div(numerator: int, denominator: int) -> int:
    return (numerator + denominator - 1) // denominator


def _ratio_desired(
    current_replicas_per_shard: int, observed: int, target: int
) -> int:
    numerator = current_replicas_per_shard * observed
    value = _ceil_div(numerator, target)
    return min(value, U32_MAX)


def plan_replica_layer(
    shard_count: int,
    current_replicas_per_shard: int,
    policy: ReplicaLayerPolicy,
    observed: ObservedUtilization,
) -> ReplicaLayerPlan:
    if shard_count == 0:
        raise ZeroShards()
    if current_replicas_per_shard == 0:
        raise ZeroCurrentReplicas()
    if policy.min_replicas_per_shard == 0:
        raise ZeroMinimum()
    if policy.max_replicas_per_shard < policy.min_replicas_per_shard:
        raise InvalidBounds()
    if not (1 <= policy.target_cpu_utilization <= 100) or not (
        1 <= policy.target_memory_utilization <= 100
    ):
        raise InvalidTarget()

    if observed.cpu_percent is None and observed.memory_percent is None:
        desired = current_replicas_per_shard
    else:
        candidates: list[int] = []
        if observed.cpu_percent is not None:
            candidates.append(
                _ratio_desired(
                    current_replicas_per_shard,
                    observed.cpu_percent,
                    policy.target_cpu_utilization,
                )
            )
        if observed.memory_percent is not None:
            candidates.append(
                _ratio_desired(
                    current_replicas_per_shard,
                    observed.memory_percent,
                    policy.target_memory_utilization,
                )
            )
        desired = max(candidates)

    desired = min(
        max(desired, policy.min_replicas_per_shard),
        policy.max_replicas_per_shard,
    )

    current_total_pods = shard_count * current_replicas_per_shard
    desired_total_pods = shard_count * desired

    if current_total_pods > U32_MAX or desired_total_pods > U32_MAX:
        raise ReplicaOverflow()

    return ReplicaLayerPlan(
        shard_count=shard_count,
        current_replicas_per_shard=current_replicas_per_shard,
        desired_replicas_per_shard=desired,
        current_total_pods=current_total_pods,
        desired_total_pods=desired_total_pods,
    )


@dataclass(frozen=True)
class ShardSplitPolicy:
    split_threshold_bytes: int = DEFAULT_SHARD_SPLIT_THRESHOLD_BYTES
    max_shards: int | None = None


@dataclass(frozen=True)
class ObservedShardUsage:
    shard_index: int
    durable_bytes: int


@dataclass(frozen=True)
class ShardSplitPlan:
    current_shard_count: int
    desired_shard_count: int
    split_threshold_bytes: int
    busiest_shard: ObservedShardUsage | None
    max_shards_reached: bool

    def requires_split(self) -> bool:
        return self.desired_shard_count > self.current_shard_count


class ShardSplitError(Exception):
    """Base for every shard-split planning refusal."""


class ZeroShardCount(ShardSplitError):
    def __init__(self) -> None:
        super().__init__("current_shard_count must be greater than zero")


class ZeroThreshold(ShardSplitError):
    def __init__(self) -> None:
        super().__init__("split_threshold_bytes must be greater than zero")


class InvalidMaximum(ShardSplitError):
    def __init__(self) -> None:
        super().__init__("max_shards must be >= current_shard_count")


class UnknownShard(ShardSplitError):
    def __init__(self) -> None:
        super().__init__("observed shard index is outside the current topology")


class ShardOverflow(ShardSplitError):
    def __init__(self) -> None:
        super().__init__("shard count exceeds u32")


def plan_shard_split(
    current_shard_count: int,
    policy: ShardSplitPolicy,
    observed: tuple[ObservedShardUsage, ...],
) -> ShardSplitPlan:
    if current_shard_count == 0:
        raise ZeroShardCount()
    if policy.split_threshold_bytes == 0:
        raise ZeroThreshold()
    if (
        policy.max_shards is not None
        and policy.max_shards < current_shard_count
    ):
        raise InvalidMaximum()
    if any(u.shard_index >= current_shard_count for u in observed):
        raise UnknownShard()

    if not observed:
        busiest = None
    else:
        busiest = max(observed, key=lambda u: (u.durable_bytes, -u.shard_index))

    max_shards_reached = (
        policy.max_shards is not None
        and current_shard_count >= policy.max_shards
    )

    threshold_crossed = (
        busiest is not None
        and busiest.durable_bytes > policy.split_threshold_bytes
    )

    if threshold_crossed and not max_shards_reached:
        desired = current_shard_count + 1
        if desired > U32_MAX:
            raise ShardOverflow()
    else:
        desired = current_shard_count

    return ShardSplitPlan(
        current_shard_count=current_shard_count,
        desired_shard_count=desired,
        split_threshold_bytes=policy.split_threshold_bytes,
        busiest_shard=busiest,
        max_shards_reached=max_shards_reached,
    )
