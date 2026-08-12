"""Availability promise evaluation."""
from __future__ import annotations

from enum import Enum
from typing import Final

__aw_artifact_id__: Final[str] = "artifact:lumen/topology-availability"


class AvailabilityPromise(str, Enum):
    SURVIVES_ONE_UNEXPECTED_NODE_LOSS = "survives_one_unexpected_node_loss"
    NO_PROMISE_ON_UNEXPECTED_NODE_LOSS = "no_promise_on_unexpected_node_loss"


def availability_promise(voters: int) -> AvailabilityPromise:
    """Return the availability promise for an admitted voter count."""
    if voters == 1:
        return AvailabilityPromise.NO_PROMISE_ON_UNEXPECTED_NODE_LOSS
    if voters == 3:
        return AvailabilityPromise.SURVIVES_ONE_UNEXPECTED_NODE_LOSS
    raise ValueError(f"unsupported or unadmitted voter count for availability promise: {voters}")
