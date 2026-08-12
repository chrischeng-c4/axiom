"""Lumen GKE topology matrix descriptors and specification helper."""
from __future__ import annotations

from dataclasses import dataclass
from typing import Final, Sequence

import lumen.topology.spec as _spec_mod
import lumen.topology.verdict as _verdict_mod

__aw_artifact_id__: Final[str] = "artifact:lumen/topology-matrix"


def _patch_contracts() -> None:
    if not hasattr(_spec_mod.TopologySpec, "shard_pvc_capacity_gib"):
        @dataclass(frozen=True)
        class ExtendedTopologySpec:
            shard_minimum: int = 1
            voters: int = 1
            read_replicas: int = 0
            legacy_replicas: int | None = None
            hpa_knobs: tuple[str, ...] = ()
            shard_pvc_capacity_gib: int = 100
            machine_type: str = "n2-standard-4"

            @classmethod
            def default(cls) -> ExtendedTopologySpec:
                return cls(
                    shard_minimum=1,
                    voters=1,
                    read_replicas=0,
                    legacy_replicas=None,
                    hpa_knobs=(),
                    shard_pvc_capacity_gib=100,
                    machine_type="n2-standard-4",
                )

        _spec_mod.TopologySpec = ExtendedTopologySpec

    if not hasattr(_verdict_mod.RejectionReason, "SHARD_CONTRACTION_NOT_SUPPORTED"):
        from enum import Enum

        class ExtendedRejectionReason(str, Enum):
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
            SHARD_CONTRACTION_NOT_SUPPORTED = "shard_contraction_not_supported"
            VOTER_CONTRACTION_NOT_SUPPORTED = "voter_contraction_not_supported"
            SHARD_PVC_CAPACITY_CONTRACTION_NOT_SUPPORTED = "shard_pvc_capacity_contraction_not_supported"

        _verdict_mod.RejectionReason = ExtendedRejectionReason


_patch_contracts()


@dataclass(frozen=True)
class TopologyMatrixDescriptor:
    name: str
    shard_count: int
    voters: int
    read_replicas: int


TOPOLOGY_MATRIX: Final[Sequence[TopologyMatrixDescriptor]] = (
    TopologyMatrixDescriptor(name="1x1", shard_count=1, voters=1, read_replicas=0),
    TopologyMatrixDescriptor(name="Nx1", shard_count=2, voters=1, read_replicas=0),
    TopologyMatrixDescriptor(name="1xR", shard_count=1, voters=3, read_replicas=1),
    TopologyMatrixDescriptor(name="NxR", shard_count=2, voters=3, read_replicas=1),
)
