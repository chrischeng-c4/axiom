"""Verdict definitions for competitor semantics and efficiency verification."""
from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Final

from lumen.competitor_verification.spec import EvidenceSpec

__aw_artifact_id__: Final[str] = "artifact:lumen/competitor-verification-verdict"


class RejectionReason(Enum):
    SEMANTIC_PROOF_MUST_PRECEDE_METRICS = "semantic_proof_must_precede_metrics"
    PEER_NOT_DECLARED_APPROPRIATE = "peer_not_declared_appropriate"
    WORKLOAD_NOT_DECLARED_COMPARABLE = "workload_not_declared_comparable"
    REQUIRED_EVIDENCE_FIELDS_REQUIRED = "required_evidence_fields_required"
    REQUIRED_METRIC_VOCABULARY_MISSING = "required_metric_vocabulary_missing"
    INTENTIONAL_DELTAS_REQUIRED = "intentional_deltas_required"
    APP_DOMAIN_DELTA_ROUTE_MISSING = "app_domain_delta_route_missing"
    BOUNDED_ISSUE_REQUIRED = "bounded_issue_required"
    EXACT_REPRODUCTION_REQUIRED = "exact_reproduction_required"
    EXISTING_WI_ACCEPTANCE_CHECK_REQUIRED = "existing_wi_acceptance_check_required"
    VALIDATED_ISSUE_REQUIRED = "validated_issue_required"
    SHARED_OR_NON_DOMAIN_FAILURE_REQUIRES_REPAIR = "shared_or_non_domain_failure_requires_repair"
    MIXED_FAILURE_REQUIRES_SPLIT = "mixed_failure_requires_split"


@dataclass(frozen=True)
class Rejection:
    reason: RejectionReason
    field_path: str


@dataclass(frozen=True)
class AdmittedEvidenceSpec:
    spec: EvidenceSpec


@dataclass(frozen=True)
class SingleFailureDisposition:
    action: str
    issue_ref: str = ""


@dataclass(frozen=True)
class MixedFailureDisposition:
    shared: SingleFailureDisposition
    app_domain: SingleFailureDisposition


@dataclass(frozen=True)
class AdmittedTerminalResult:
    terminal: str
    issue_ref: str = ""


__all__ = [
    "RejectionReason",
    "Rejection",
    "AdmittedEvidenceSpec",
    "SingleFailureDisposition",
    "MixedFailureDisposition",
    "AdmittedTerminalResult",
]
