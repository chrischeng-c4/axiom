"""Topology catalog admission and mutation verdict models."""
from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Final, Union

__aw_artifact_id__: Final[str] = "artifact:lumen/topology-catalog-verdict"


class CatalogRejectionReason(str, Enum):
    UNSUPPORTED_CATALOG_MODE = "unsupported_catalog_mode"
    INSUFFICIENT_ELIGIBLE_MEMBERS = "insufficient_eligible_members"
    OPERATOR_NOT_SERVING_AUTHORITY = "operator_not_serving_authority"
    INSTANCE_ID_MISMATCH = "instance_id_mismatch"
    STALE_SEED_GENERATION = "stale_seed_generation"
    STALE_CATALOG_GENERATION = "stale_catalog_generation"
    CATALOG_QUORUM_UNAVAILABLE = "catalog_quorum_unavailable"


@dataclass(frozen=True)
class AdmittedCatalogPlan:
    voter_count: int
    member_ids: tuple[str, ...]
    hostnames: tuple[str, ...]
    zones: tuple[str, ...]
    limitation: str | None = None


@dataclass(frozen=True)
class Rejection:
    reason: CatalogRejectionReason
    field_path: str
    message: str


CatalogVerdict = Union[AdmittedCatalogPlan, Rejection]
