"""Admission deciders for competitor semantics and efficiency verification."""
from __future__ import annotations

from typing import Final

from lumen.competitor_verification.spec import (
    EvidenceSpec,
    FailureDispositionRequest,
    FailureOwnership,
    IssueBacking,
    TerminalResultRequest,
)
from lumen.competitor_verification.verdict import (
    AdmittedEvidenceSpec,
    AdmittedTerminalResult,
    MixedFailureDisposition,
    Rejection,
    RejectionReason,
    SingleFailureDisposition,
)

__aw_artifact_id__: Final[str] = "artifact:lumen/competitor-verification-admission"

REQUIRED_EVIDENCE_FIELDS: Final[tuple[str, ...]] = (
    "command",
    "work_count",
    "output_summary",
    "evidence_path",
    "duration",
    "resources",
)

REQUIRED_METRIC_VOCABULARY: Final[tuple[str, ...]] = (
    "throughput",
    "latency",
    "cpu",
    "memory",
    "lifecycle_overhead",
    "cost",
)


def decide_evidence_spec(spec: EvidenceSpec) -> AdmittedEvidenceSpec | Rejection:
    """Decide whether an evidence specification meets competitor verification requirements."""
    if not spec.semantic_proof_precedes_metrics:
        return Rejection(
            reason=RejectionReason.SEMANTIC_PROOF_MUST_PRECEDE_METRICS,
            field_path="semantic_proof_precedes_metrics",
        )

    if not spec.peer or not spec.peer.declared_appropriate:
        return Rejection(
            reason=RejectionReason.PEER_NOT_DECLARED_APPROPRIATE,
            field_path="peer.declared_appropriate",
        )

    if not spec.workload or not spec.workload.declared_comparable:
        return Rejection(
            reason=RejectionReason.WORKLOAD_NOT_DECLARED_COMPARABLE,
            field_path="workload.declared_comparable",
        )

    if not spec.required_evidence_fields or not set(REQUIRED_EVIDENCE_FIELDS).issubset(
        set(spec.required_evidence_fields)
    ):
        return Rejection(
            reason=RejectionReason.REQUIRED_EVIDENCE_FIELDS_REQUIRED,
            field_path="required_evidence_fields",
        )

    if not spec.metric_vocabulary or not set(REQUIRED_METRIC_VOCABULARY).issubset(
        set(spec.metric_vocabulary)
    ):
        return Rejection(
            reason=RejectionReason.REQUIRED_METRIC_VOCABULARY_MISSING,
            field_path="metric_vocabulary",
        )

    if not spec.intentional_deltas:
        return Rejection(
            reason=RejectionReason.INTENTIONAL_DELTAS_REQUIRED,
            field_path="intentional_deltas",
        )

    if not spec.app_domain_delta_route or not spec.app_domain_delta_route.strip():
        return Rejection(
            reason=RejectionReason.APP_DOMAIN_DELTA_ROUTE_MISSING,
            field_path="app_domain_delta_route",
        )

    return AdmittedEvidenceSpec(spec=spec)


def _validate_app_domain_issue(issue: IssueBacking | None) -> Rejection | None:
    """Validate an issue backing for an app-domain failure skip."""
    if not issue or not issue.bounded:
        return Rejection(
            reason=RejectionReason.BOUNDED_ISSUE_REQUIRED,
            field_path="issue.bounded",
        )

    if not issue.exact_reproduction or not issue.exact_reproduction.strip():
        return Rejection(
            reason=RejectionReason.EXACT_REPRODUCTION_REQUIRED,
            field_path="issue.exact_reproduction",
        )

    if not issue.validated:
        return Rejection(
            reason=RejectionReason.VALIDATED_ISSUE_REQUIRED,
            field_path="issue.validated",
        )

    if (
        issue.authoritative_existing_wi_supplied
        and not issue.authoritative_existing_wi_acceptance_checked
    ):
        return Rejection(
            reason=RejectionReason.EXISTING_WI_ACCEPTANCE_CHECK_REQUIRED,
            field_path="issue.authoritative_existing_wi_acceptance_checked",
        )

    return None


def decide_failure_disposition(
    request: FailureDispositionRequest,
) -> SingleFailureDisposition | MixedFailureDisposition | Rejection:
    """Decide failure disposition based on failure ownership and issue backing."""
    if request.ownership in (FailureOwnership.SHARED, FailureOwnership.NON_DOMAIN):
        return SingleFailureDisposition(action="repair_and_rerun")

    if request.ownership == FailureOwnership.APP_DOMAIN_ONLY:
        rejection = _validate_app_domain_issue(request.issue)
        if rejection:
            return rejection
        return SingleFailureDisposition(
            action="tracked_skip",
            issue_ref=request.issue.issue_ref if request.issue else "",
        )

    if request.ownership == FailureOwnership.MIXED:
        rejection = _validate_app_domain_issue(request.issue)
        if rejection:
            return rejection
        return MixedFailureDisposition(
            shared=SingleFailureDisposition(action="repair_and_rerun"),
            app_domain=SingleFailureDisposition(
                action="tracked_skip",
                issue_ref=request.issue.issue_ref if request.issue else "",
            ),
        )

    return SingleFailureDisposition(action="repair_and_rerun")


def decide_terminal_result(
    request: TerminalResultRequest,
) -> AdmittedTerminalResult | Rejection:
    """Decide terminal verification result."""
    if request.ownership in (FailureOwnership.SHARED, FailureOwnership.NON_DOMAIN):
        return Rejection(
            reason=RejectionReason.SHARED_OR_NON_DOMAIN_FAILURE_REQUIRES_REPAIR,
            field_path="ownership",
        )

    if request.ownership == FailureOwnership.MIXED:
        return Rejection(
            reason=RejectionReason.MIXED_FAILURE_REQUIRES_SPLIT,
            field_path="ownership",
        )

    if request.ownership == FailureOwnership.APP_DOMAIN_ONLY:
        rejection = _validate_app_domain_issue(request.issue)
        if rejection:
            return rejection
        return AdmittedTerminalResult(
            terminal="tracked_skip",
            issue_ref=request.issue.issue_ref if request.issue else "",
        )

    if request.journey_completed and request.ownership == FailureOwnership.NONE:
        return AdmittedTerminalResult(
            terminal="passed",
            issue_ref="",
        )

    return Rejection(
        reason=RejectionReason.SHARED_OR_NON_DOMAIN_FAILURE_REQUIRES_REPAIR,
        field_path="ownership",
    )


__all__ = [
    "decide_evidence_spec",
    "decide_failure_disposition",
    "decide_terminal_result",
]
