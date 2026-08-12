"""Classification deciders for container/Kustomize verification failures."""

from __future__ import annotations

from typing import Final, Union

from lumen.container_kustomize.spec import (
    Action,
    BoundedIssue,
    FailureOwner,
    extract_issue_number,
)
from lumen.container_kustomize.verdict import (
    FailureOutcome,
    MixedFailureOutcome,
    Reason,
    Rejection,
)

__aw_artifact_id__: Final[str] = "artifact:lumen/container-kustomize-classification"


def decide_failure_outcome(
    failure_owner: FailureOwner,
    bounded_issue: Union[BoundedIssue, int, None] = None,
) -> Union[FailureOutcome, Rejection]:
    """Decide single failure outcome based on failure ownership and issue status."""
    if failure_owner in (FailureOwner.SHARED, FailureOwner.NON_DOMAIN):
        return FailureOutcome(action=Action.SHARED_REPAIR_REQUIRED)

    if failure_owner == FailureOwner.APP_DOMAIN:
        num = extract_issue_number(bounded_issue)
        if num > 0:
            return FailureOutcome(action=Action.TRACKED_SKIP, issue_number=num)
        return Rejection(
            reason=Reason.BOUNDED_ISSUE_REQUIRED,
            field_path="bounded_issue.number",
        )

    return FailureOutcome(action=Action.SHARED_REPAIR_REQUIRED)


def decide_mixed_failure(
    shared_failure: FailureOwner,
    domain_failure: FailureOwner,
    bounded_issue: Union[BoundedIssue, int, None] = None,
) -> Union[MixedFailureOutcome, Rejection]:
    """Decide split outcome for mixed failures containing shared and domain slices."""
    num = extract_issue_number(bounded_issue)
    if num <= 0:
        return Rejection(
            reason=Reason.BOUNDED_ISSUE_REQUIRED,
            field_path="bounded_issue.number",
        )

    return MixedFailureOutcome(
        shared=FailureOutcome(action=Action.REPAIR_AND_RERUN),
        domain=FailureOutcome(action=Action.TRACKED_SKIP, issue_number=num),
    )
