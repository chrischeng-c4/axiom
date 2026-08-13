"""Specification structures for competitor semantics and efficiency verification."""
from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from typing import Final

__aw_artifact_id__: Final[str] = "artifact:lumen/competitor-verification-spec"


class FailureOwnership(Enum):
    NONE = "none"
    SHARED = "shared"
    NON_DOMAIN = "non_domain"
    APP_DOMAIN_ONLY = "app_domain_only"
    MIXED = "mixed"


@dataclass(frozen=True)
class PeerDeclaration:
    name: str = ""
    declared_appropriate: bool = False


@dataclass(frozen=True)
class WorkloadDeclaration:
    name: str = ""
    declared_comparable: bool = False


@dataclass(frozen=True)
class EvidenceSpec:
    semantic_proof_precedes_metrics: bool = False
    peer: PeerDeclaration = field(default_factory=PeerDeclaration)
    workload: WorkloadDeclaration = field(default_factory=WorkloadDeclaration)
    required_evidence_fields: tuple[str, ...] = ()
    metric_vocabulary: tuple[str, ...] = ()
    intentional_deltas: tuple[str, ...] = ()
    app_domain_delta_route: str = ""


@dataclass(frozen=True)
class IssueBacking:
    issue_ref: str = ""
    validated: bool = False
    bounded: bool = False
    exact_reproduction: str = ""
    authoritative_existing_wi_supplied: bool = False
    authoritative_existing_wi_acceptance_checked: bool = False


@dataclass(frozen=True)
class FailureDispositionRequest:
    ownership: FailureOwnership
    issue: IssueBacking | None = None


@dataclass(frozen=True)
class TerminalResultRequest:
    journey_completed: bool = False
    ownership: FailureOwnership = FailureOwnership.NONE
    issue: IssueBacking | None = None


__all__ = [
    "FailureOwnership",
    "PeerDeclaration",
    "WorkloadDeclaration",
    "EvidenceSpec",
    "IssueBacking",
    "FailureDispositionRequest",
    "TerminalResultRequest",
]
