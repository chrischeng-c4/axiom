"""Capacity downgrade transition decider."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Final, Union

from lumen.capacity.verdict import CapacityReason, CapacityRejection, TransitionKind

__aw_artifact_id__: Final[str] = "artifact:lumen/capacity-transition"


@dataclass(frozen=True)
class AdmittedTransition:
    kind: TransitionKind
    target_machine_type: str
    target_node_count: int


def _get(obj: Any, key: str, default: Any = None) -> Any:
    if isinstance(obj, dict):
        return obj.get(key, default)
    return getattr(obj, key, default)


def decide_downgrade(
    policy: Any,
    current: Any,
    proposed: Any,
    observed_at: int,
) -> Union[AdmittedTransition, CapacityRejection]:
    """Decide whether a proposed machine/node downgrade is admitted."""
    stable_since = int(_get(current, "stable_since", 0))
    stable_window = int(_get(policy, "stable_window_seconds", 300))
    elapsed_stable = observed_at - stable_since
    if elapsed_stable < stable_window:
        return CapacityRejection(
            reason=CapacityReason.STABLE_WINDOW_NOT_ELAPSED,
            field_path="current.stable_since",
            message=f"stable window of {stable_window}s has not elapsed (elapsed: {elapsed_stable}s)",
        )

    last_transition_at = int(_get(current, "last_transition_at", 0))
    cooldown = int(_get(policy, "cooldown_seconds", 600))
    elapsed_cooldown = observed_at - last_transition_at
    if elapsed_cooldown < cooldown:
        return CapacityRejection(
            reason=CapacityReason.COOLDOWN_ACTIVE,
            field_path="current.last_transition_at",
            message=f"cooldown window of {cooldown}s is still active (elapsed: {elapsed_cooldown}s)",
        )

    proposed_node_count = int(_get(proposed, "node_count", 0))
    pool_maximum = int(_get(policy, "pool_maximum", 0))
    if proposed_node_count > pool_maximum:
        return CapacityRejection(
            reason=CapacityReason.POOL_MAXIMUM_EXCEEDED,
            field_path="proposed.node_count",
            message=f"proposed node count {proposed_node_count} exceeds pool maximum {pool_maximum}",
        )

    headroom = int(_get(proposed, "projected_allocatable_headroom", 0))
    if headroom < 1:
        return CapacityRejection(
            reason=CapacityReason.INSUFFICIENT_HEADROOM,
            field_path="proposed.projected_allocatable_headroom",
            message=f"projected allocatable headroom {headroom} is insufficient",
        )

    proposed_mt = str(_get(proposed, "machine_type", ""))
    return AdmittedTransition(
        kind=TransitionKind.ADMITTED,
        target_machine_type=proposed_mt,
        target_node_count=proposed_node_count,
    )
