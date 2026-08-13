"""Capacity decision verdict models and reason vocabulary."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Final

__aw_artifact_id__: Final[str] = "artifact:lumen/capacity-verdict"


class CapacityReason(str, Enum):
    ANOTHER_MUTATION_ACTIVE = "another_mutation_active"
    STABLE_WINDOW_NOT_ELAPSED = "stable-window-not-elapsed"
    INSUFFICIENT_HEADROOM = "insufficient-headroom"
    POOL_MAXIMUM_EXCEEDED = "pool-maximum-exceeded"
    COOLDOWN_ACTIVE = "cooldown-active"
    DATA_MEMBER_NODE_CONFLICT = "data_member_node_conflict"
    CAPACITY_BLOCKED = "CapacityBlocked"
    INVALID_INPUT = "invalid_input"
    COMPETING_MUTATION = "competing_mutation"
    INTERRUPTED_MUTATION_MISMATCH = "interrupted_mutation_mismatch"


class ReclaimAction(str, Enum):
    RECLAIM = "reclaim"
    RETAIN = "retain"


class ReapplyAction(str, Enum):
    NO_OP = "no_op"
    REAPPLY = "reapply"


class TransitionKind(str, Enum):
    ADMITTED = "admitted"
    REJECTED = "rejected"


@dataclass(frozen=True)
class CapacityRejection:
    reason: CapacityReason
    field_path: str
    message: str

    @property
    def kind(self) -> TransitionKind:
        return TransitionKind.REJECTED
