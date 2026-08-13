"""Ownership reapply decider for GitOps transitions."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Final, Union

from lumen.capacity.spec import CapacitySpec
from lumen.capacity.verdict import CapacityReason, CapacityRejection, ReapplyAction

__aw_artifact_id__: Final[str] = "artifact:lumen/capacity-ownership"


@dataclass(frozen=True)
class OwnershipVerdict:
    initial: CapacitySpec
    current: CapacitySpec
    target: CapacitySpec
    action: ReapplyAction


def _get(obj: Any, key: str, default: Any = None) -> Any:
    if isinstance(obj, dict):
        return obj.get(key, default)
    return getattr(obj, key, default)


def decide_reapply(
    initial: Any,
    current: Any,
    target: Any,
    rendered_input: Any,
) -> Union[OwnershipVerdict, CapacityRejection]:
    """Decide reapplication behavior preserving automatic targets."""
    init_mt = _get(initial, "machine_type")
    init_owner = _get(initial, "owner", "user")

    curr_mt = _get(current, "machine_type")
    curr_owner = _get(current, "owner", "automatic")

    targ_mt = _get(target, "machine_type")
    targ_owner = _get(target, "owner", "automatic")

    rendered_mt = _get(rendered_input, "machine_type")

    if rendered_mt not in (init_mt, curr_mt, targ_mt):
        return CapacityRejection(
            reason=CapacityReason.COMPETING_MUTATION,
            field_path="rendered_input.machine_type",
            message=f"rendered input machine type {rendered_mt!r} competes with automatic target {targ_mt!r}",
        )

    initial_spec = CapacitySpec(machine_type=str(init_mt), owner=str(init_owner))
    current_spec = CapacitySpec(machine_type=str(curr_mt), owner=str(curr_owner))
    target_spec = CapacitySpec(machine_type=str(targ_mt), owner=str(targ_owner))

    return OwnershipVerdict(
        initial=initial_spec,
        current=current_spec,
        target=target_spec,
        action=ReapplyAction.NO_OP,
    )
