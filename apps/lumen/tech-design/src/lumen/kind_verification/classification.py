"""Failure partition logic for Kind deployment verification."""
from __future__ import annotations

from dataclasses import dataclass
from typing import Final, Iterable

from lumen.kind_verification.verdict import Failure

__aw_artifact_id__: Final[str] = "artifact:lumen/kind-verification-classification"


@dataclass(frozen=True)
class PartitionResult:
    shared_non_domain: tuple[Failure, ...]
    app_domain_only: tuple[Failure, ...]


def partition_failures(failures: Iterable[Failure]) -> PartitionResult:
    """Partition verification failures into shared/non-domain and app-domain-only slices."""
    shared_list: list[Failure] = []
    app_list: list[Failure] = []

    for failure in failures:
        if failure.ownership == "APP_DOMAIN_ONLY":
            app_list.append(failure)
        else:
            shared_list.append(failure)

    return PartitionResult(
        shared_non_domain=tuple(shared_list),
        app_domain_only=tuple(app_list),
    )


__all__ = [
    "PartitionResult",
    "partition_failures",
]
