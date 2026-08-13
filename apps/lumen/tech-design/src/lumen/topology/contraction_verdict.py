"""Verdict and enum models for deferred dynamic shard contraction (#2528)."""
from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Final

__aw_artifact_id__: Final[str] = "artifact:lumen/contraction-verdict"


class PvcDisposition(str, Enum):
    """Closed PVC disposition vocabulary -- retained or reclaimable only."""

    RECLAIMABLE = "reclaimable"
    RETAINED = "retained"


class ContractionReason(str, Enum):
    """Closed refusal reason vocabulary for contraction admission."""

    WAL_NOT_CONSOLIDATED = "wal_not_consolidated"
    LIVE_DATA_NOT_CONSOLIDATED = "live_data_not_consolidated"
    INVALID_CATALOG_TRANSITION = "invalid_catalog_transition"
    EVIDENCE_INCOMPLETE = "evidence_incomplete"
    CONTRACTION_DEPENDENCY_NOT_PERMITTED = "contraction_dependency_not_permitted"
    ENTRY_GATE_NOT_PASSED = "entry_gate_not_passed"


@dataclass(frozen=True)
class ContractionVerdict:
    """Verdict for a proposed contraction state transition."""

    outcome: str
    next_phase: str = ""
    catalog_version_transition: tuple[int, int] = (0, 0)
    rollback_status: str = "not_eligible"
    source_retirement_status: str = "not_eligible"
    reason: str = ""
    field_path: str = ""
    message: str = ""


@dataclass(frozen=True)
class EntryGateVerdict:
    """Verdict for measured entry gate evidence."""

    outcome: str
    reason: str = ""
    field_path: str = ""
    message: str = ""


@dataclass(frozen=True)
class V1DependencyVerdict:
    """Verdict for a proposed v1 architecture dependency."""

    outcome: str
    reason: str = ""
    field_path: str = ""
    message: str = ""


@dataclass(frozen=True)
class ImplementationChildrenVerdict:
    """Verdict for allowing child implementation issues."""

    outcome: str
    reason: str = ""
    field_path: str = ""
    message: str = ""
