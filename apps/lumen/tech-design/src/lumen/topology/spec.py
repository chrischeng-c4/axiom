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

    @classmethod
    def default(cls) -> TopologySpec:
        return cls(
            shard_minimum=1,
            voters=1,
            read_replicas=0,
            legacy_replicas=None,
            hpa_knobs=(),
        )
