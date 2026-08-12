"""Lumen topology mutation rules and deciders."""
from __future__ import annotations

from typing import Final

import lumen.topology.matrix as _matrix  # noqa: F401
from lumen.topology.admission import decide_topology_mutation

__aw_artifact_id__: Final[str] = "artifact:lumen/topology-mutation"

__all__ = ["decide_topology_mutation"]
