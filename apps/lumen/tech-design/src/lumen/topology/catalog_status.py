"""Lumen topology catalog status model."""
from __future__ import annotations

from dataclasses import dataclass
from typing import Final

from lumen.topology.catalog_spec import CatalogSpec
from lumen.topology.catalog_state import CatalogState

__aw_artifact_id__: Final[str] = "artifact:lumen/topology-catalog-status"


@dataclass(frozen=True)
class CatalogStatus:
    spec: CatalogSpec
    state: CatalogState
    quorum_available: bool = True

    def is_converged(self) -> bool:
        """Return True if current and converged generations match."""
        return self.state.current_generation == self.state.converged_generation
