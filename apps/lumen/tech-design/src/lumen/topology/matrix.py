"""Lumen GKE topology matrix descriptors and specification helper."""
from __future__ import annotations

from dataclasses import dataclass
from typing import Final, Sequence

__aw_artifact_id__: Final[str] = "artifact:lumen/topology-matrix"


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
