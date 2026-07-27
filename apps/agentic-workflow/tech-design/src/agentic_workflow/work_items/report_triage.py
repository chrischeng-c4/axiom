"""Typed terminal triage for inbound Report work items.

@spec #2596
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum


class ReportVerdict(StrEnum):
    ACCEPTED = "accepted"
    DUPLICATE = "duplicate"
    INVALID = "invalid"
    BY_DESIGN = "by-design"


@dataclass(frozen=True)
class ReportTriage:
    verdict: ReportVerdict
    spawned_work_item: str | None = None

    def is_terminal(self) -> bool:
        return self.verdict is not ReportVerdict.ACCEPTED or bool(self.spawned_work_item)


def may_mutate_type_label() -> bool:
    """Triage spawns a new WI; it never rewrites the Report type."""

    return False
