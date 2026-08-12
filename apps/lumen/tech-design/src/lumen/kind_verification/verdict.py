"""Verdict and record definitions for Kind deployment verification."""
from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Final

__aw_artifact_id__: Final[str] = "artifact:lumen/kind-verification-verdict"


class TerminalResult(Enum):
    PASSED = "passed"
    TRACKED_SKIP = "tracked_skip"


class RejectionReason(Enum):
    SHARED_FAILURE_CANNOT_SKIP = "shared-failure-cannot-skip"
    MISSING_DOMAIN_ISSUE = "missing-domain-issue"
    UNVALIDATED_DOMAIN_ISSUE = "unvalidated-domain-issue"


@dataclass(frozen=True)
class Failure:
    code: str
    ownership: str


@dataclass(frozen=True)
class VerificationRecord:
    failures: tuple[Failure, ...] = ()
    domain_issue: str = ""
    domain_issue_validated: bool = False


@dataclass(frozen=True)
class Rejection:
    reason: RejectionReason
    field_path: str


@dataclass(frozen=True)
class Admitted:
    result: TerminalResult
    issue_ref: str = ""


__all__ = [
    "TerminalResult",
    "RejectionReason",
    "Failure",
    "VerificationRecord",
    "Rejection",
    "Admitted",
]
