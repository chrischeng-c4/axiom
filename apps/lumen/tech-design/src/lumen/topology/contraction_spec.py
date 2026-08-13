"""Specification models for deferred dynamic shard contraction (#2528)."""
from __future__ import annotations

from dataclasses import dataclass
from typing import Final

__aw_artifact_id__: Final[str] = "artifact:lumen/contraction-spec"


@dataclass(frozen=True)
class ContractionState:
    """State of a proposed or in-progress shard contraction."""

    phase: str
    catalog_from: int
    catalog_to: int
    live_data_consolidated: bool
    wal_consolidated: bool
    cutover_committed: bool
    rollback_requested: bool


@dataclass(frozen=True)
class EntryGateEvidence:
    """Measured evidence required before opening contraction implementation work."""

    risk_quantified: bool
    temporary_capacity_quantified: bool
    recovery_time_quantified: bool
    cost_benefit_quantified: bool


@dataclass(frozen=True)
class V1Dependency:
    """Dependency declaration for v1 architecture features."""

    kind: str
