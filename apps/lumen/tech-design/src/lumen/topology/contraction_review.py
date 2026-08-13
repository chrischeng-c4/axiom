"""Review completeness decider for deferred dynamic shard contraction (#2528)."""
from __future__ import annotations

from dataclasses import dataclass
from typing import Final

__aw_artifact_id__: Final[str] = "artifact:lumen/contraction-review"


@dataclass(frozen=True)
class ContractionDecisions:
    """Design decision records required for contraction review."""

    durability: str
    routing: str
    rollback: str
    pvc_retention: str


@dataclass(frozen=True)
class ReviewCompletenessResult:
    """Completeness verdict for design decisions."""

    missing_decisions: tuple[str, ...]


def review_completeness(decisions: ContractionDecisions) -> ReviewCompletenessResult:
    """Identify missing required design decisions for AC1."""
    missing: list[str] = []

    if not decisions.durability or not decisions.durability.strip():
        missing.append("durability")
    if not decisions.routing or not decisions.routing.strip():
        missing.append("routing")
    if not decisions.rollback or not decisions.rollback.strip():
        missing.append("rollback")
    if not decisions.pvc_retention or not decisions.pvc_retention.strip():
        missing.append("pvc_retention")

    return ReviewCompletenessResult(missing_decisions=tuple(missing))
