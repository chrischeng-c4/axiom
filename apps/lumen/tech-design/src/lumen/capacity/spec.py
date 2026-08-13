"""Capacity specifications and models."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Final, Optional

__aw_artifact_id__: Final[str] = "artifact:lumen/capacity-spec"


@dataclass(frozen=True)
class CapacitySpec:
    machine_type: str
    owner: str = "automatic"


@dataclass(frozen=True)
class MemberSpec:
    identifier: str
    healthy: bool = True
    role: str = "voter"


@dataclass(frozen=True)
class ActionSpec:
    identifier: str
    kind: str
