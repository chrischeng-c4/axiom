"""Canonical typed priority-label authoring.

@spec #2767
"""

from __future__ import annotations

from enum import StrEnum


__aw_artifact_id__ = "artifact:work-item-planning/typed-priority-label"
__aw_work_item__ = "2767"


class WorkItemPriority(StrEnum):
    """Closed priority values accepted by ``aw wi create --priority``."""

    P0 = "p0"
    P1 = "p1"
    P2 = "p2"
    P3 = "p3"


def canonical_priority_label(priority: WorkItemPriority) -> str:
    """Return the exact tracker label consumed by graph and list queries."""

    return f"priority:{priority.value}"


def priority_help_contract() -> str:
    """Document the public label projection without a second separator."""

    return "Emits a `priority:<value>` scoped label."
