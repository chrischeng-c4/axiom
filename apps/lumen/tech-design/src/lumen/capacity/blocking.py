"""Capacity blockage and correction recovery decider."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Final, Union

from lumen.capacity.status import BlockedCondition, CapacityBlockedVerdict, OldMember
from lumen.capacity.verdict import CapacityReason, CapacityRejection

__aw_artifact_id__: Final[str] = "artifact:lumen/capacity-blocking"


@dataclass(frozen=True)
class ResumedCapacityBlock:
    resume_generation: int
    generation: int
    old_member: OldMember
    condition: BlockedCondition


def _get(obj: Any, key: str, default: Any = None) -> Any:
    if isinstance(obj, dict):
        return obj.get(key, default)
    return getattr(obj, key, default)


def decide_capacity_block(
    condition: Any,
    old_member: Any,
    generation: int,
) -> Union[CapacityBlockedVerdict, ResumedCapacityBlock, CapacityRejection]:
    """Decide capacity blocked status and generation preservation on error or correction."""
    cond_kind = str(_get(condition, "kind", "")).lower()
    mem_id = str(_get(old_member, "identifier", ""))
    mem_healthy = bool(_get(old_member, "healthy", True))
    gen_int = int(generation)

    old_mem_spec = OldMember(identifier=mem_id, healthy=mem_healthy)

    if cond_kind == "corrected":
        cond_spec = BlockedCondition(type="Corrected", kind=cond_kind)
        return ResumedCapacityBlock(
            resume_generation=gen_int,
            generation=gen_int,
            old_member=old_mem_spec,
            condition=cond_spec,
        )

    if cond_kind in ("absent", "at_maximum", "quota_blocked", "unschedulable"):
        cond_spec = BlockedCondition(type="CapacityBlocked", kind=cond_kind)
        return CapacityBlockedVerdict(
            condition=cond_spec,
            old_member=old_mem_spec,
            generation=gen_int,
            resume_generation=gen_int,
        )

    return CapacityRejection(
        reason=CapacityReason.INVALID_INPUT,
        field_path="condition.kind",
        message=f"unrecognized capacity condition kind {cond_kind!r}",
    )
