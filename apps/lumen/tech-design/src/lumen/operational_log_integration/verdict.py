"""Verdict models and reason vocabulary for #2377 operational log integration."""
from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Final, Optional

from lumen.operational_log_integration.spec import (
    FailureOwnership,
    GateRecord,
    TerminalResult,
)

__aw_artifact_id__: Final[str] = "artifact:lumen/operational-log-integration/verdict"


class Reason(Enum):
    MISSING_REQUIRED_EVIDENCE = "missing_required_evidence"
    REQUIRED_EVENT_KIND_MISSING = "required_event_kind_missing"
    BOUNDED_ISSUE_REQUIRED = "bounded_issue_required"
    EXACT_REPRODUCTION_REQUIRED = "exact_reproduction_required"
    EXISTING_WI_ACCEPTANCE_CHECK_REQUIRED = "existing_wi_acceptance_check_required"
    SHARED_NON_DOMAIN_FAILURE_REQUIRES_REPAIR = "shared_non_domain_failure_requires_repair"
    MIXED_FAILURE_REQUIRES_SPLIT = "mixed_failure_requires_split"


@dataclass(frozen=True)
class Rejection:
    reason: Reason
    field_path: str = ""
    missing_kind: str = ""


@dataclass(frozen=True)
class GateRecordVerdict:
    record: GateRecord


@dataclass(frozen=True)
class CoverageVerdict:
    kinds: tuple[str, ...]


@dataclass(frozen=True)
class SharedSliceVerdict:
    action: str = "repair_and_rerun"


@dataclass(frozen=True)
class AppDomainSliceVerdict:
    action: str = "tracked_skip"


@dataclass(frozen=True)
class ClassificationVerdict:
    classification: FailureOwnership
    action: str
    issue_ref: str = ""
    exact_reproduction: str = ""


@dataclass(frozen=True)
class MixedFailureVerdict:
    classification: FailureOwnership = FailureOwnership.MIXED
    shared: SharedSliceVerdict = SharedSliceVerdict()
    app_domain: AppDomainSliceVerdict = AppDomainSliceVerdict()


@dataclass(frozen=True)
class TerminalVerdict:
    terminal: TerminalResult
    issue_ref: Optional[str] = None
