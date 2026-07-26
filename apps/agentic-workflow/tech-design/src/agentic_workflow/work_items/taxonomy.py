"""Canonical work-item kinds and executable-backlog eligibility."""

from enum import StrEnum


class WorkItemType(StrEnum):
    """Closed terminology-first work-item taxonomy."""

    EPIC = "epic"
    CHANGE = "change"
    SPIKE = "spike"
    REPORT = "report"

    @property
    def backlog_eligible(self) -> bool:
        """Only bounded changes may enter the executable backlog."""

        return self is WorkItemType.CHANGE
