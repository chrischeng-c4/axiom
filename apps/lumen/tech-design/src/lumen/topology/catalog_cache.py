"""Lumen catalog cache update and last-converged deciders."""
from __future__ import annotations

from typing import Final

from lumen.topology.catalog_state import CatalogState
from lumen.topology.catalog_verdict import CatalogRejectionReason, Rejection

__aw_artifact_id__: Final[str] = "artifact:lumen/topology-catalog-cache"


def decide_cache_update(
    current: CatalogState,
    candidate: CatalogState,
    quorum_available: bool,
) -> CatalogState | Rejection:
    """Decide monotonic catalog cache update or failure on missing quorum / stale generation."""
    if not quorum_available:
        return Rejection(
            reason=CatalogRejectionReason.CATALOG_QUORUM_UNAVAILABLE,
            field_path="quorum_available",
            message="catalog mutation refused: catalog quorum is unavailable",
        )

    if candidate.current_generation < current.current_generation:
        return Rejection(
            reason=CatalogRejectionReason.STALE_CATALOG_GENERATION,
            field_path="candidate.current_generation",
            message=f"candidate generation {candidate.current_generation} is staler than current generation {current.current_generation}",
        )

    return candidate


def last_converged(
    current: CatalogState,
    candidate: CatalogState | None = None,
) -> CatalogState:
    """Return the last-converged topology state, ignoring stale or absent candidates."""
    if candidate is None or candidate.current_generation < current.current_generation:
        return current
    return candidate
