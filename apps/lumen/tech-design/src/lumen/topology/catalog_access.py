"""Lumen catalog serving topology decider."""
from __future__ import annotations

from dataclasses import dataclass
from typing import Final

from lumen.topology.catalog_verdict import CatalogRejectionReason, Rejection

__aw_artifact_id__: Final[str] = "artifact:lumen/topology-catalog-access"


@dataclass(frozen=True)
class AdmittedServingSource:
    source: str


def decide_serving_topology(source: str) -> AdmittedServingSource | Rejection:
    """Decide if a requested topology source is an admitted serving authority."""
    if source in ("catalog", "last-converged-cache"):
        return AdmittedServingSource(source=source)

    return Rejection(
        reason=CatalogRejectionReason.OPERATOR_NOT_SERVING_AUTHORITY,
        field_path="source",
        message=f"serving topology source '{source}' is not a valid catalog authority; Kubernetes operator is not serving authority",
    )
