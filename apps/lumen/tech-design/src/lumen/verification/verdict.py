"""Data models and enums for verification verdicts, failures, and records."""
from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Final

__aw_artifact_id__: Final[str] = "artifact:lumen/verification-verdict"


class Ownership(Enum):
    APP_DOMAIN = "app_domain"
    SHARED = "shared"
    NON_DOMAIN = "non_domain"
    MIXED = "mixed"


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


class TerminalResult(Enum):
    PASSED = "passed"
    TRACKED_SKIP = "tracked_skip"


@dataclass(frozen=True)
class Rejection:
    reason: Reason
    field_path: str


@dataclass(frozen=True)
class TerminalDecision:
    terminal: TerminalResult
    issue_ref: str | None = None


@dataclass(frozen=True)
class Failure:
    ownership: Ownership | str
    summary: str
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
    ownership: Ownership
    summary: str
    bounded_issue: str | None = None


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
    shared_slice: SharedSlice
    app_domain_slice: AppDomainSlice
