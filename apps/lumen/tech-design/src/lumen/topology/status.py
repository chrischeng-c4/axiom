"""Topology status compartment definition."""
from __future__ import annotations

from dataclasses import dataclass
from typing import Final

from lumen.topology.spec import TopologySpec
from lumen.topology.verdict import AdmittedTopology

__aw_artifact_id__: Final[str] = "artifact:lumen/topology-status"


@dataclass(frozen=True)
class TopologyStatus:
    policy: TopologySpec
    current: AdmittedTopology
    target: AdmittedTopology
    observed_generation: int
    converged_generation: int
    render_committed: bool = False

    def is_converged(self) -> bool:
        """Return True if render is committed and generations and topology states match."""
        return (
            self.render_committed
            and self.observed_generation == self.converged_generation
            and self.current == self.target
        )
