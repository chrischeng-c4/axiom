"""Topology admission and mutation verdict models."""
from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Final, Union

__aw_artifact_id__: Final[str] = "artifact:lumen/topology-verdict"


class RejectionReason(str, Enum):
    ZERO_SHARD_MINIMUM = "zero_shard_minimum"
    ZERO_VOTERS = "zero_voters"
    EVEN_VOTER_COUNT = "even_voter_count"
    UNSUPPORTED_VOTER_COUNT = "unsupported_voter_count"
    LEGACY_REPLICA_VOCABULARY = "legacy_replica_vocabulary"
    NEGATIVE_READ_REPLICAS = "negative_read_replicas"
    HPA_KNOB_NOT_OWNED = "hpa_knob_not_owned"
    NO_SAFE_TOPOLOGY_MUTATION = "no_safe_topology_mutation"
    DATA_MEMBER_NODE_CONFLICT = "data_member_node_conflict"
    NO_PROMISE_ON_UNEXPECTED_NODE_LOSS = "no_promise_on_unexpected_node_loss"


@dataclass(frozen=True)
class AdmittedTopology:
    shard_count: int
    voters: int
    read_replicas: int


@dataclass(frozen=True)
class Rejection:
    reason: RejectionReason
    field_path: str
    message: str


TopologyVerdict = Union[AdmittedTopology, Rejection]
