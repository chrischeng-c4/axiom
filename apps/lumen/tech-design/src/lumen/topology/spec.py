"""Lumen topology spec dataclass definition."""
from __future__ import annotations

from dataclasses import dataclass
from typing import Final

__aw_artifact_id__: Final[str] = "artifact:lumen/topology-spec"


@dataclass(frozen=True)
class TopologySpec:
    shard_minimum: int = 1
    voters: int = 1
    read_replicas: int = 0
    legacy_replicas: int | None = None
    hpa_knobs: tuple[str, ...] = ()
    shard_pvc_capacity_gib: int = 100
    machine_type: str = "n2-standard-4"

    @classmethod
    def default(cls) -> TopologySpec:
        return cls(
            shard_minimum=1,
            voters=1,
            read_replicas=0,
            legacy_replicas=None,
            hpa_knobs=(),
            shard_pvc_capacity_gib=100,
            machine_type="n2-standard-4",
        )
