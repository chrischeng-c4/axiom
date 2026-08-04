from __future__ import annotations

from dataclasses import dataclass

from raft_runtime.domain.errors import (
    NodeIdOutOfRange,
    NonPositiveDimension,
    TopologyError,
    VoterCountOutOfRange,
)


@dataclass(frozen=True, slots=True)
class ClusterDims:
    shard_count: int
    replicas_per_shard: int
    voter_count: int
    ordinal: int

    @property
    def shard_index(self) -> int:
        return self.ordinal % self.shard_count

    @property
    def replica_index(self) -> int:
        return self.ordinal // self.shard_count

    @property
    def is_voter(self) -> bool:
        return self.replica_index < self.voter_count


def peer_ordinal(shard_count: int, shard_index: int, replica: int) -> int:
    return replica * shard_count + shard_index


def dims_problem(
    shard_count: int, replicas_per_shard: int, voter_count: int, node_id: int
) -> TopologyError | None:
    if shard_count <= 0:
        return NonPositiveDimension(name="SHARD_COUNT", value=shard_count)
    if replicas_per_shard <= 0:
        return NonPositiveDimension(
            name="REPLICAS_PER_SHARD", value=replicas_per_shard
        )
    if not (1 <= voter_count <= replicas_per_shard):
        return VoterCountOutOfRange(
            voter_count=voter_count, replicas_per_shard=replicas_per_shard
        )
    if not (0 <= node_id < replicas_per_shard):
        return NodeIdOutOfRange(
            node_id=node_id, replicas_per_shard=replicas_per_shard
        )
    return None
