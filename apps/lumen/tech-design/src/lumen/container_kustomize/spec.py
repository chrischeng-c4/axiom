"""Spec definitions for container/Kustomize verification decisions."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Final, Union

__aw_artifact_id__: Final[str] = "artifact:lumen/container-kustomize-spec"


class FailureOwner(Enum):
    SHARED = "shared"
    NON_DOMAIN = "non-domain"
    APP_DOMAIN = "app-domain"
    NONE = "none"


class Action(Enum):
    SHARED_REPAIR_REQUIRED = "shared_repair_required"
    REPAIR_AND_RERUN = "repair_and_rerun"
    TRACKED_SKIP = "tracked_skip"


class TerminalState(Enum):
    PASSED = "passed"
    TRACKED_SKIP = "tracked_skip"
    OPEN = "open"


@dataclass(frozen=True)
class BoundedIssue:
    number: int

    @property
    def is_bounded(self) -> bool:
        return self.number > 0


def extract_issue_number(issue: Union[BoundedIssue, int, None]) -> int:
    if isinstance(issue, BoundedIssue):
        return issue.number
    if isinstance(issue, int):
        return issue
    return 0
