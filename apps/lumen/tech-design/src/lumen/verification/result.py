"""Terminal result decision logic for verification records."""
from __future__ import annotations

from typing import Final

from lumen.verification.classification import resolve_ownership
from lumen.verification.verdict import (
    Ownership,
    Reason,
    Rejection,
    TerminalDecision,
    TerminalResult,
    VerificationRecord,
)

__aw_artifact_id__: Final[str] = "artifact:lumen/verification-result"


def decide_terminal_result(record: VerificationRecord) -> TerminalDecision | Rejection:
    if record.gate_exit_code != 0:
        return Rejection(reason=Reason.GATE_EXIT_NONZERO, field_path="gate_exit_code")

    if record.applicable_work_count is None or record.applicable_work_count <= 0:
        return Rejection(reason=Reason.NO_APPLICABLE_WORK, field_path="applicable_work_count")

    if not record.commit:
        return Rejection(reason=Reason.MISSING_COMMIT, field_path="commit")

    if not record.environment:
        return Rejection(reason=Reason.MISSING_ENVIRONMENT, field_path="environment")

    if not record.command:
        return Rejection(reason=Reason.MISSING_COMMAND, field_path="command")

    if not record.output_summary:
        return Rejection(reason=Reason.MISSING_OUTPUT_SUMMARY, field_path="output_summary")

    if not record.evidence_path:
        return Rejection(reason=Reason.MISSING_EVIDENCE_PATH, field_path="evidence_path")

    if record.failure is not None:
        ownership = resolve_ownership(record.failure.ownership)
        if ownership is None:
            return Rejection(reason=Reason.UNKNOWN_OWNERSHIP, field_path="ownership")

        if ownership in (Ownership.SHARED, Ownership.NON_DOMAIN, Ownership.MIXED):
            return Rejection(
                reason=Reason.TRACKED_SKIP_REQUIRES_APP_DOMAIN, field_path="ownership"
            )

        if ownership == Ownership.APP_DOMAIN:
            if not record.failure.bounded_issue:
                return Rejection(
                    reason=Reason.TRACKED_SKIP_REQUIRES_BOUNDED_ISSUE,
                    field_path="bounded_issue",
                )
            if record.terminal_intent == "tracked_skip":
                return TerminalDecision(
                    terminal=TerminalResult.TRACKED_SKIP,
                    issue_ref=record.failure.bounded_issue,
                )
            return Rejection(
                reason=Reason.TRACKED_SKIP_REQUIRES_BOUNDED_ISSUE,
                field_path="bounded_issue",
            )

    if record.terminal_intent == "passed":
        return TerminalDecision(terminal=TerminalResult.PASSED)

    if record.terminal_intent == "tracked_skip":
        return Rejection(
            reason=Reason.TRACKED_SKIP_REQUIRES_BOUNDED_ISSUE,
            field_path="bounded_issue",
        )

    return Rejection(
        reason=Reason.TRACKED_SKIP_REQUIRES_BOUNDED_ISSUE,
        field_path="bounded_issue",
    )
