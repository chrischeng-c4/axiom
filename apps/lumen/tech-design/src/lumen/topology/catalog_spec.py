"""Lumen topology catalog specification, eligible member, and bootstrap seed dataclasses."""
from __future__ import annotations

from dataclasses import dataclass
from typing import Final

__aw_artifact_id__: Final[str] = "artifact:lumen/topology-catalog-spec"


@dataclass(frozen=True)
class CatalogSpec:
    instance_id: str
    mode: str = "non-ha"


@dataclass(frozen=True)
class EligibleMember:
    member_id: str
    hostname: str
    zone: str


@dataclass(frozen=True)
class BootstrapSeed:
    instance_id: str
    seed_id: str
    hostname: str
    zone: str
    generation: int
