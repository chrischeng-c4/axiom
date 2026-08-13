"""Capacity status representation and generation binding check."""
from __future__ import annotations

from dataclasses import dataclass
from typing import Final, Optional

__aw_artifact_id__: Final[str] = "artifact:lumen/capacity/status"


@dataclass(frozen=True)
class BlockedCondition:
    type: str = "CapacityBlocked"
    kind: str = "absent"


@dataclass(frozen=True)
class OldMember:
    identifier: str
    healthy: bool = True


@dataclass(frozen=True)
class CapacityBlockedVerdict:
    condition: BlockedCondition
    old_member: OldMember
    generation: int
    resume_generation: int


@dataclass(frozen=True)
class CapacityStatus:
    recommendation_generation: int
    action_generation: int
    status_generation: int

    def is_generation_bound(self) -> bool:
        return (
            self.recommendation_generation == self.action_generation
            and self.action_generation == self.status_generation
        )
