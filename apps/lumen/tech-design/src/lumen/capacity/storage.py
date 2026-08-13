"""Member storage and PVC lifecycle decider."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Final, Union

from lumen.capacity.verdict import CapacityReason, CapacityRejection, ReclaimAction

__aw_artifact_id__: Final[str] = "artifact:lumen/capacity-storage"


@dataclass(frozen=True)
class StorageVerdict:
    desired_size: str
    reclaim: ReclaimAction
    member_role: str
    lifecycle_event: str


def _get(obj: Any, key: str, default: Any = None) -> Any:
    if isinstance(obj, dict):
        return obj.get(key, default)
    return getattr(obj, key, default)


def decide_member_storage(
    catalog: Any,
    member_role: str,
    lifecycle_event: str,
) -> Union[StorageVerdict, CapacityRejection]:
    """Decide desired storage size and PVC reclamation eligibility."""
    desired_size = _get(catalog, "committed_desired_size")
    if not desired_size:
        return CapacityRejection(
            reason=CapacityReason.INVALID_INPUT,
            field_path="catalog.committed_desired_size",
            message="catalog committed_desired_size is required",
        )

    role_str = str(member_role).lower()
    event_str = str(lifecycle_event).lower()

    if event_str == "drained":
        if role_str == "read_replica":
            reclaim = ReclaimAction.RECLAIM
        else:
            reclaim = ReclaimAction.RETAIN
    else:
        reclaim = ReclaimAction.RETAIN

    return StorageVerdict(
        desired_size=str(desired_size),
        reclaim=reclaim,
        member_role=role_str,
        lifecycle_event=event_str,
    )
