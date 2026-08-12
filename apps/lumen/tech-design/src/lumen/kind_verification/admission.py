"""Terminal verdict admission decider for Kind deployment verification."""
from __future__ import annotations

from typing import Final

from lumen.kind_verification.classification import partition_failures
from lumen.kind_verification.verdict import (
    Admitted,
    Rejection,
    RejectionReason,
    TerminalResult,
    VerificationRecord,
)

__aw_artifact_id__: Final[str] = "artifact:lumen/kind-verification-admission"


def decide_terminal(record: VerificationRecord) -> Admitted | Rejection:
    """Decide terminal outcome for a verification record.

    Returns an Admitted verdict (PASSED or TRACKED_SKIP) or a fail-closed Rejection.
    """
    if not record.failures:
        return Admitted(result=TerminalResult.PASSED, issue_ref="")

    partition = partition_failures(record.failures)

    if partition.shared_non_domain:
        return Rejection(
            reason=RejectionReason.SHARED_FAILURE_CANNOT_SKIP,
            field_path="failures",
        )

    if not record.domain_issue or not record.domain_issue.strip():
        return Rejection(
            reason=RejectionReason.MISSING_DOMAIN_ISSUE,
            field_path="domain_issue",
        )

    if not record.domain_issue_validated:
        return Rejection(
            reason=RejectionReason.UNVALIDATED_DOMAIN_ISSUE,
            field_path="domain_issue_validated",
        )

    return Admitted(
        result=TerminalResult.TRACKED_SKIP,
        issue_ref=record.domain_issue,
    )


__all__ = [
    "decide_terminal",
]
