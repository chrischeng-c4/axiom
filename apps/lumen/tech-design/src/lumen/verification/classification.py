"""Failure classification and splitting logic for verification results."""
from __future__ import annotations

from enum import Enum
from typing import Final, Iterable

from lumen.verification.verdict import (
    AppDomainSlice,
    ClassifiedFailure,
    Disposition,
    Failure,
    Ownership,
    Reason,
    Rejection,
    SharedSlice,
    SplitFailureVerdict,
)

__aw_artifact_id__: Final[str] = "artifact:lumen/verification-classification"


def resolve_ownership(value: Ownership | str | None) -> Ownership | None:
    if isinstance(value, Ownership):
        return value
    if isinstance(value, str):
        try:
            return Ownership(value)
        except ValueError:
            return None
    return None


def classify_failure(failure: Failure) -> ClassifiedFailure | Rejection:
    if failure.owner is not None and failure.ownership is None:
        owner_str = failure.owner
        field_name = "owner"
        unknown_reason = Reason.UNKNOWN_FAILURE_OWNER
    elif failure.ownership is not None and failure.owner is None:
        owner_str = failure.ownership.value if isinstance(failure.ownership, Enum) else str(failure.ownership)
        field_name = "ownership"
        unknown_reason = Reason.UNKNOWN_OWNERSHIP
    elif failure.owner is not None:
        owner_str = failure.owner
        field_name = "owner"
        unknown_reason = Reason.UNKNOWN_FAILURE_OWNER
    else:
        owner_str = ""
        field_name = "owner"
        unknown_reason = Reason.UNKNOWN_FAILURE_OWNER

    resolved_ownership = resolve_ownership(failure.ownership or failure.owner)

    if owner_str in ("shared", "non_domain"):
        disposition = Disposition.SHARED_REPAIR_REQUIRED
    elif owner_str == "app_domain":
        disposition = Disposition.APP_DOMAIN_TRACKABLE
    else:
        return Rejection(reason=unknown_reason, field_path=field_name)

    return ClassifiedFailure(
        failure_id=failure.failure_id,
        owner=owner_str,
        disposition=disposition,
        summary=failure.summary,
        bounded_issue=failure.bounded_issue,
        ownership=resolved_ownership or owner_str,
    )


def split_failure(failures: Iterable[Failure] | Failure) -> SplitFailureVerdict | Rejection:
    if isinstance(failures, Failure):
        if failures.ownership == Ownership.MIXED or failures.owner == "mixed":
            shared_summary = failures.shared_summary or failures.summary
            app_summary = failures.app_domain_summary or failures.summary
            shared_f = Failure(
                failure_id=failures.failure_id,
                owner="shared",
                ownership=Ownership.SHARED,
                summary=shared_summary,
            )
            app_f = Failure(
                failure_id=failures.failure_id,
                owner="app_domain",
                ownership=Ownership.APP_DOMAIN,
                summary=app_summary,
                bounded_issue=failures.bounded_issue,
            )
            shared_slice = SharedSlice(
                ownership=Ownership.SHARED,
                disposition="rerun_required",
                summary=shared_summary,
            )
            issue_ref = failures.bounded_issue
            app_slice = AppDomainSlice(
                ownership=Ownership.APP_DOMAIN,
                issue_ref=issue_ref,
                disposition=f"tracked_skip({issue_ref})" if issue_ref else "tracked_skip",
                summary=app_summary,
            )
            return SplitFailureVerdict(
                shared_failures=(shared_f,),
                app_domain_failures=(app_f,),
                shared_slice=shared_slice,
                app_domain_slice=app_slice,
            )
        elif failures.owner in ("shared", "non_domain", "app_domain") or resolve_ownership(failures.ownership) is not None:
            return Rejection(
                reason=Reason.TRACKED_SKIP_REQUIRES_APP_DOMAIN, field_path="ownership"
            )
        else:
            return Rejection(reason=Reason.UNKNOWN_FAILURE_OWNER, field_path="owner")

    items = tuple(failures)
    shared_list: list[Failure] = []
    app_list: list[Failure] = []

    for f in items:
        owner_str = f.owner
        if owner_str is None and f.ownership is not None:
            owner_str = f.ownership.value if isinstance(f.ownership, Enum) else str(f.ownership)

        if owner_str in ("shared", "non_domain"):
            shared_list.append(f)
        elif owner_str == "app_domain":
            app_list.append(f)
        elif owner_str == "mixed":
            shared_list.append(
                Failure(
                    failure_id=f.failure_id,
                    owner="shared",
                    ownership=Ownership.SHARED,
                    summary=f.shared_summary or f.summary,
                )
            )
            app_list.append(
                Failure(
                    failure_id=f.failure_id,
                    owner="app_domain",
                    ownership=Ownership.APP_DOMAIN,
                    summary=f.app_domain_summary or f.summary,
                    bounded_issue=f.bounded_issue,
                )
            )
        else:
            return Rejection(reason=Reason.UNKNOWN_FAILURE_OWNER, field_path="owner")

    return SplitFailureVerdict(
        shared_failures=tuple(shared_list),
        app_domain_failures=tuple(app_list),
    )
