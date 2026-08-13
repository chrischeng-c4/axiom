"""Admission deciders for #2377 operational log integration."""
from __future__ import annotations

from typing import Final, Iterable, Optional

from lumen.operational_log_integration.spec import (
    Failure,
    FailureOwnership,
    GateRecord,
    TerminalResult,
)
from lumen.operational_log_integration.verdict import (
    AppDomainSliceVerdict,
    ClassificationVerdict,
    CoverageVerdict,
    GateRecordVerdict,
    MixedFailureVerdict,
    Reason,
    Rejection,
    SharedSliceVerdict,
    TerminalVerdict,
)

__aw_artifact_id__: Final[str] = "artifact:lumen/operational-log-integration/admission"

REQUIRED_EVENT_KINDS: Final[tuple[str, ...]] = (
    "success",
    "retry_or_failure",
    "security_audit",
    "lifecycle",
)


def decide_gate_record(record: GateRecord) -> GateRecordVerdict | Rejection:
    """R1 / AC1 -- Decide admissibility of a gate record."""
    if not record.commit:
        return Rejection(reason=Reason.MISSING_REQUIRED_EVIDENCE, field_path="commit")
    if not record.environment:
        return Rejection(reason=Reason.MISSING_REQUIRED_EVIDENCE, field_path="environment")
    if not record.command:
        return Rejection(reason=Reason.MISSING_REQUIRED_EVIDENCE, field_path="command")
    if not record.output_summary:
        return Rejection(reason=Reason.MISSING_REQUIRED_EVIDENCE, field_path="output_summary")
    if not record.evidence_path:
        return Rejection(reason=Reason.MISSING_REQUIRED_EVIDENCE, field_path="evidence_path")

    return GateRecordVerdict(record=record)


def decide_coverage(observed_kinds: Iterable[str]) -> CoverageVerdict | Rejection:
    """R2 -- Decide event coverage admissibility."""
    observed_set = set(observed_kinds)
    for kind in REQUIRED_EVENT_KINDS:
        if kind not in observed_set:
            return Rejection(reason=Reason.REQUIRED_EVENT_KIND_MISSING, missing_kind=kind)
    return CoverageVerdict(kinds=tuple(observed_kinds))


def classify_failure(failure: Failure) -> ClassificationVerdict | Rejection:
    """R3 -- Classify failure by ownership."""
    if failure.authoritative_existing_wi_supplied and not failure.authoritative_existing_wi_accepted:
        return Rejection(
            reason=Reason.EXISTING_WI_ACCEPTANCE_CHECK_REQUIRED,
            field_path="authoritative_existing_wi_accepted",
        )

    if failure.ownership == FailureOwnership.SHARED_NON_DOMAIN:
        return ClassificationVerdict(
            classification=FailureOwnership.SHARED_NON_DOMAIN,
            action="repair_and_rerun",
            issue_ref=failure.issue_ref,
            exact_reproduction=failure.exact_reproduction,
        )

    if failure.ownership == FailureOwnership.APP_DOMAIN_ONLY:
        if not failure.issue_ref:
            return Rejection(reason=Reason.BOUNDED_ISSUE_REQUIRED, field_path="issue_ref")
        if not failure.exact_reproduction:
            return Rejection(reason=Reason.EXACT_REPRODUCTION_REQUIRED, field_path="exact_reproduction")
        return ClassificationVerdict(
            classification=FailureOwnership.APP_DOMAIN_ONLY,
            action="tracked_skip",
            issue_ref=failure.issue_ref,
            exact_reproduction=failure.exact_reproduction,
        )

    if failure.ownership == FailureOwnership.MIXED:
        return ClassificationVerdict(
            classification=FailureOwnership.MIXED,
            action="split",
            issue_ref=failure.issue_ref,
            exact_reproduction=failure.exact_reproduction,
        )

    return ClassificationVerdict(
        classification=FailureOwnership.NONE,
        action="none",
        issue_ref=failure.issue_ref,
        exact_reproduction=failure.exact_reproduction,
    )


def decide_mixed_failure(failure: Failure) -> MixedFailureVerdict | Rejection:
    """R4 -- Split mixed failures into shared repair and app-domain skip."""
    if failure.authoritative_existing_wi_supplied and not failure.authoritative_existing_wi_accepted:
        return Rejection(
            reason=Reason.EXISTING_WI_ACCEPTANCE_CHECK_REQUIRED,
            field_path="authoritative_existing_wi_accepted",
        )

    return MixedFailureVerdict(
        classification=FailureOwnership.MIXED,
        shared=SharedSliceVerdict(action="repair_and_rerun"),
        app_domain=AppDomainSliceVerdict(action="tracked_skip"),
    )


def decide_terminal_result(
    classification: FailureOwnership | ClassificationVerdict | Failure | Rejection,
    issue_ref: Optional[str] = None,
    shared_rerun_complete: bool = False,
) -> TerminalVerdict | Rejection:
    """AC3 -- Decide terminal disposition."""
    if isinstance(classification, Rejection):
        return classification

    ownership: Optional[FailureOwnership] = None
    ref: Optional[str] = issue_ref

    if isinstance(classification, FailureOwnership):
        ownership = classification
    elif hasattr(classification, "classification"):
        ownership = classification.classification
        if not ref and hasattr(classification, "issue_ref"):
            ref = classification.issue_ref
    elif hasattr(classification, "ownership"):
        ownership = classification.ownership
        if not ref and hasattr(classification, "issue_ref"):
            ref = classification.issue_ref

    if ownership == FailureOwnership.NONE:
        return TerminalVerdict(terminal=TerminalResult.PASSED, issue_ref="")

    if ownership == FailureOwnership.SHARED_NON_DOMAIN:
        return Rejection(reason=Reason.SHARED_NON_DOMAIN_FAILURE_REQUIRES_REPAIR)

    if ownership == FailureOwnership.MIXED:
        return Rejection(reason=Reason.MIXED_FAILURE_REQUIRES_SPLIT)

    if ownership == FailureOwnership.APP_DOMAIN_ONLY:
        if not ref:
            return Rejection(reason=Reason.BOUNDED_ISSUE_REQUIRED, field_path="issue_ref")
        return TerminalVerdict(terminal=TerminalResult.TRACKED_SKIP, issue_ref=ref)

    return Rejection(reason=Reason.MIXED_FAILURE_REQUIRES_SPLIT)
