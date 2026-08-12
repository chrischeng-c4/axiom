"""Result deciders for container/Kustomize verification terminal state."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Final, Optional, Union

from lumen.container_kustomize.spec import (
    BoundedIssue,
    FailureOwner,
    TerminalState,
    extract_issue_number,
)
from lumen.container_kustomize.verdict import Reason

__aw_artifact_id__: Final[str] = "artifact:lumen/container-kustomize-result"


@dataclass(frozen=True)
class TerminalResult:
    state: TerminalState
    issue_number: Optional[int] = None
    reason: Optional[Reason] = None
    field_path: str = ""


def decide_terminal_result(
    failure_owner: FailureOwner,
    issue_number: Union[BoundedIssue, int, None] = None,
    shared_rerun_succeeded: bool = False,
) -> TerminalResult:
    """Decide terminal verification state for Lumen container/Kustomize artifacts."""
    if shared_rerun_succeeded:
        return TerminalResult(state=TerminalState.PASSED)

    if failure_owner == FailureOwner.APP_DOMAIN:
        num = extract_issue_number(issue_number)
        if num > 0:
            return TerminalResult(
                state=TerminalState.TRACKED_SKIP,
                issue_number=num,
            )
        return TerminalResult(
            state=TerminalState.OPEN,
            reason=Reason.BOUNDED_ISSUE_REQUIRED,
            field_path="bounded_issue.number",
        )

    return TerminalResult(
        state=TerminalState.OPEN,
        reason=Reason.SHARED_RERUN_REQUIRED,
        field_path="shared_rerun_succeeded",
    )
