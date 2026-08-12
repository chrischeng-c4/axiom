"""Failure classification and splitting logic for verification results."""
from __future__ import annotations

from typing import Final

from lumen.verification.verdict import (
    AppDomainSlice,
    ClassifiedFailure,
    Failure,
    Ownership,
    Reason,
    Rejection,
    SharedSlice,
    SplitFailureVerdict,
)

__aw_artifact_id__: Final[str] = "artifact:lumen/verification-classification"


def resolve_ownership(value: Ownership | str) -> Ownership | None:
    if isinstance(value, Ownership):
        return value
    if isinstance(value, str):
        try:
            return Ownership(value)
        except ValueError:
            return None
    return None


def classify_failure(failure: Failure) -> ClassifiedFailure | Rejection:
    ownership = resolve_ownership(failure.ownership)
    if ownership is None:
        return Rejection(reason=Reason.UNKNOWN_OWNERSHIP, field_path="ownership")
    return ClassifiedFailure(
        ownership=ownership,
        summary=failure.summary,
        bounded_issue=failure.bounded_issue,
    )


def split_failure(failure: Failure) -> SplitFailureVerdict | Rejection:
    ownership = resolve_ownership(failure.ownership)
    if ownership is None:
        return Rejection(reason=Reason.UNKNOWN_OWNERSHIP, field_path="ownership")

    if ownership != Ownership.MIXED:
        return Rejection(
            reason=Reason.TRACKED_SKIP_REQUIRES_APP_DOMAIN, field_path="ownership"
        )

    shared_summary = failure.shared_summary or failure.summary
    app_summary = failure.app_domain_summary or failure.summary

    shared_slice = SharedSlice(
        ownership=Ownership.SHARED,
        disposition="rerun_required",
        summary=shared_summary,
    )

    issue_ref = failure.bounded_issue
    disposition = f"tracked_skip({issue_ref})" if issue_ref else "tracked_skip"
    app_slice = AppDomainSlice(
        ownership=Ownership.APP_DOMAIN,
        issue_ref=issue_ref,
        disposition=disposition,
        summary=app_summary,
    )

    return SplitFailureVerdict(
        shared_slice=shared_slice,
        app_domain_slice=app_slice,
    )
