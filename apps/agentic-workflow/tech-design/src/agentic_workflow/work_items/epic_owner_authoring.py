"""Typed epic-owner authoring for work-item create and update.

@spec #2688
"""

from __future__ import annotations

from dataclasses import dataclass


__aw_artifact_id__ = "artifact:work-item-planning/epic-owner-authoring"
__aw_work_item__ = "2688"


@dataclass(frozen=True)
class EpicOwnerDeclaration:
    """Canonical ownership written by ``aw wi create|update --epic``."""

    epic_id: str
    label: str

    @classmethod
    def from_id(cls, epic_id: str) -> "EpicOwnerDeclaration":
        normalized = epic_id.strip().lstrip("#").rstrip(".,;)]")
        if not normalized:
            raise ValueError("epic id must not be empty")
        return cls(epic_id=normalized, label=f"epic:{normalized}")


def assert_no_body_conflict(
    declaration: EpicOwnerDeclaration,
    body_parent_ids: tuple[str, ...],
) -> None:
    """Reject typed ownership that disagrees with decode-only body prose."""

    conflicts = tuple(
        parent for parent in body_parent_ids if parent != declaration.epic_id
    )
    if conflicts:
        raise ValueError(
            f"typed epic {declaration.epic_id} conflicts with body parents {conflicts}"
        )
