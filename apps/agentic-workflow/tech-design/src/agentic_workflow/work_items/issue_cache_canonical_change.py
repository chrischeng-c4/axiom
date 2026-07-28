"""Canonical Change codec at the issue-cache boundary.

@spec #2772
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum


__aw_artifact_id__ = "artifact:work-item-planning/issue-cache-canonical-change"
__aw_work_item__ = "2772"


class CanonicalIssueType(StrEnum):
    """The only issue types emitted by current AW producers."""

    EPIC = "epic"
    CHANGE = "change"
    SPIKE = "spike"
    REPORT = "report"


_LEGACY_CHANGE_ALIASES = frozenset(
    {"bug", "enhancement", "feature", "refactor", "test"}
)


def decode_issue_type(raw: str) -> CanonicalIssueType:
    """Accept canonical values and decode legacy delivery aliases as Change."""

    normalized = raw.lower()
    if normalized in _LEGACY_CHANGE_ALIASES:
        return CanonicalIssueType.CHANGE
    try:
        return CanonicalIssueType(normalized)
    except ValueError as error:
        raise ValueError(
            f"unsupported issue type {raw!r}; expected epic, change, spike, or report"
        ) from error


def encode_issue_type(issue_type: CanonicalIssueType | str) -> str:
    """Serialize every accepted delivery alias as canonical lowercase change."""

    if isinstance(issue_type, str):
        issue_type = decode_issue_type(issue_type)
    return issue_type.value


@dataclass(frozen=True)
class CachedIssue:
    """Minimum cache record needed to keep inventory reads failure-isolated."""

    issue_id: str
    issue_type: CanonicalIssueType

    @classmethod
    def from_frontmatter(cls, issue_id: str, raw_type: str) -> CachedIssue:
        return cls(issue_id=issue_id, issue_type=decode_issue_type(raw_type))

    def frontmatter_type(self) -> str:
        return encode_issue_type(self.issue_type)
