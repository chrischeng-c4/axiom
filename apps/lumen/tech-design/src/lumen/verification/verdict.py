"""Data models and enums for verification verdicts, failures, and records."""
from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Any, Final

__aw_artifact_id__: Final[str] = "artifact:lumen/verification-verdict"


class Ownership(Enum):
    APP_DOMAIN = "app_domain"
    SHARED = "shared"
    NON_DOMAIN = "non_domain"
    MIXED = "mixed"


class Disposition(Enum):
    SHARED_REPAIR_REQUIRED = "shared_repair_required"
    APP_DOMAIN_TRACKABLE = "app_domain_trackable"


class Reason(Enum):
    GATE_EXIT_NONZERO = "gate_exit_nonzero"
    NO_APPLICABLE_WORK = "no_applicable_work"
    MISSING_COMMIT = "missing_commit"
    MISSING_ENVIRONMENT = "missing_environment"
    MISSING_COMMAND = "missing_command"
    MISSING_OUTPUT_SUMMARY = "missing_output_summary"
    MISSING_EVIDENCE_PATH = "missing_evidence_path"
    TRACKED_SKIP_REQUIRES_APP_DOMAIN = "tracked_skip_requires_app_domain"
    TRACKED_SKIP_REQUIRES_BOUNDED_ISSUE = "tracked_skip_requires_bounded_issue"
    UNKNOWN_OWNERSHIP = "unknown_ownership"

    UNKNOWN_FAILURE_OWNER = "unknown_failure_owner"
    SHARED_REPAIR_REQUIRED = "shared_repair_required"
    RERUN_INCOMPLETE = "rerun_incomplete"
    EXACTLY_ONE_ISSUE_REFERENCE = "exactly_one_issue_reference"


class TerminalResult(Enum):
    PASSED = "passed"
    TRACKED_SKIP = "tracked_skip"


@dataclass(frozen=True)
class Rejection:
    reason: Reason
    field_path: str


@dataclass(frozen=True)
class TerminalVerdict:
    result: TerminalResult = TerminalResult.PASSED
    terminal: TerminalResult = TerminalResult.PASSED
    issue_ref: str | None = None

    def __post_init__(self) -> None:
        if self.result != self.terminal:
            if self.terminal != TerminalResult.PASSED and self.result == TerminalResult.PASSED:
                object.__setattr__(self, "result", self.terminal)
            elif self.result != TerminalResult.PASSED and self.terminal == TerminalResult.PASSED:
                object.__setattr__(self, "terminal", self.result)


TerminalDecision = TerminalVerdict


@dataclass(frozen=True)
class Failure:
    failure_id: str = ""
    owner: str | None = None
    ownership: Ownership | str | None = None
    summary: str = ""
    shared_summary: str | None = None
    app_domain_summary: str | None = None
    bounded_issue: str | None = None


@dataclass(frozen=True)
class VerificationRecord:
    gate_exit_code: int
    applicable_work_count: int
    commit: str | None
    environment: str | None
    command: str | None
    output_summary: str | None
    evidence_path: str | None
    failure: Failure | None = None
    terminal_intent: str = "passed"


@dataclass(frozen=True)
class ClassifiedFailure:
    failure_id: str = ""
    owner: str = ""
    disposition: Disposition = Disposition.SHARED_REPAIR_REQUIRED
    summary: str = ""
    bounded_issue: str | None = None
    ownership: Ownership | str | None = None


@dataclass(frozen=True)
class SharedSlice:
    ownership: Ownership = Ownership.SHARED
    disposition: str = "rerun_required"
    summary: str | None = None


@dataclass(frozen=True)
class AppDomainSlice:
    ownership: Ownership = Ownership.APP_DOMAIN
    issue_ref: str | None = None
    disposition: str = "tracked_skip"
    summary: str | None = None


@dataclass(frozen=True)
class SplitFailureVerdict:
    shared_failures: tuple[Failure, ...] = ()
    app_domain_failures: tuple[Failure, ...] = ()
    shared_slice: SharedSlice | None = None
    app_domain_slice: AppDomainSlice | None = None


def decide_terminal_result(
    classifications: Any = (),
    issue_refs: Any = (),
    rerun_complete: bool = True,
) -> TerminalVerdict | Rejection:
    if isinstance(classifications, VerificationRecord):
        return _decide_terminal_result_from_record(classifications)

    if not rerun_complete:
        return Rejection(reason=Reason.RERUN_INCOMPLETE, field_path="rerun_complete")

    if isinstance(classifications, (tuple, list, set)):
        class_tuple = tuple(classifications)
    else:
        class_tuple = (classifications,) if classifications is not None else ()

    if isinstance(issue_refs, (tuple, list, set)):
        refs_tuple = tuple(issue_refs)
    elif issue_refs:
        refs_tuple = (issue_refs,)
    else:
        refs_tuple = ()

    if not class_tuple:
        return TerminalVerdict(result=TerminalResult.PASSED)

    has_shared = False
    all_app_domain = True

    for c in class_tuple:
        if isinstance(c, Rejection):
            return c

        disp = getattr(c, "disposition", None)
        owner = getattr(c, "owner", None)
        if owner is None:
            ownership_attr = getattr(c, "ownership", None)
            owner = ownership_attr.value if isinstance(ownership_attr, Enum) else (str(ownership_attr) if ownership_attr else None)

        if disp == Disposition.SHARED_REPAIR_REQUIRED or owner in ("shared", "non_domain"):
            has_shared = True
            all_app_domain = False
        elif disp == Disposition.APP_DOMAIN_TRACKABLE or owner == "app_domain":
            pass
        else:
            return Rejection(reason=Reason.UNKNOWN_FAILURE_OWNER, field_path="classifications")

    if has_shared:
        return Rejection(reason=Reason.SHARED_REPAIR_REQUIRED, field_path="classifications")

    if not all_app_domain:
        return Rejection(reason=Reason.UNKNOWN_FAILURE_OWNER, field_path="classifications")

    if len(refs_tuple) != 1 or not refs_tuple[0]:
        return Rejection(reason=Reason.EXACTLY_ONE_ISSUE_REFERENCE, field_path="issue_refs")

    return TerminalVerdict(result=TerminalResult.TRACKED_SKIP, issue_ref=refs_tuple[0])


def _decide_terminal_result_from_record(record: VerificationRecord) -> TerminalVerdict | Rejection:
    from lumen.verification.classification import resolve_ownership

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
        ownership = resolve_ownership(record.failure.ownership or record.failure.owner)
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
                return TerminalVerdict(
                    result=TerminalResult.TRACKED_SKIP,
                    issue_ref=record.failure.bounded_issue,
                )
            return Rejection(
                reason=Reason.TRACKED_SKIP_REQUIRES_BOUNDED_ISSUE,
                field_path="bounded_issue",
            )

    if record.terminal_intent == "passed":
        return TerminalVerdict(result=TerminalResult.PASSED)

    if record.terminal_intent == "tracked_skip":
        return Rejection(
            reason=Reason.TRACKED_SKIP_REQUIRES_BOUNDED_ISSUE,
            field_path="bounded_issue",
        )

    return Rejection(
        reason=Reason.TRACKED_SKIP_REQUIRES_BOUNDED_ISSUE,
        field_path="bounded_issue",
    )
